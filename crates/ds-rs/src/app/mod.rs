pub mod config;
pub mod runner;
pub mod timers;

use crate::error::Result;
use crate::pipeline::Pipeline;
use crate::source::SourceController;
use crate::backend::BackendManager;
use crate::elements::factory::ElementFactory;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Main application demonstrating runtime source addition/deletion
pub struct Application {
    pipeline: Arc<Pipeline>,
    source_controller: Arc<Mutex<SourceController>>,
    backend_manager: Arc<BackendManager>,
    initial_uri: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

impl Application {
    pub fn new(uri: String) -> Result<Self> {
        let backend_manager = Arc::new(BackendManager::new()?);
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        
        Ok(Self {
            pipeline: Arc::new(Pipeline::new("ds-runtime-demo")?),
            source_controller: Arc::new(Mutex::new(SourceController::new(
                Arc::new(Pipeline::new("dummy")?),
                gst::ElementFactory::make("fakesink").build()?,
            ))),
            backend_manager,
            initial_uri: uri,
            running: Arc::new(Mutex::new(false)),
            shutdown_tx,
            shutdown_rx: Some(shutdown_rx),
        })
    }
    
    pub fn init(&mut self) -> Result<()> {
        println!("Initializing pipeline with {} backend...", self.backend_manager.backend_type().name());
        
        let factory = ElementFactory::new(self.backend_manager.clone());
        
        // Create stream muxer for dynamic source management
        let streammux = factory.create_stream_mux(Some("stream-muxer"))?;
        
        // Only set nvstreammux-specific properties if using DeepStream backend
        if self.backend_manager.backend_type() == crate::backend::BackendType::DeepStream {
            streammux.set_property("batch-size", 30i32);
            streammux.set_property("batched-push-timeout", 25000i32);
            streammux.set_property("width", config::MUXER_OUTPUT_WIDTH as i32);
            streammux.set_property("height", config::MUXER_OUTPUT_HEIGHT as i32);
            streammux.set_property("live-source", true);
        } else if self.backend_manager.backend_type() == crate::backend::BackendType::Standard {
            // For standard backend (compositor), set different properties
            streammux.set_property_from_str("background", "black");
        }
        
        // Create processing elements based on backend capabilities
        let caps = self.backend_manager.capabilities();
        
        let mut elements = vec![streammux.clone()];
        
        // Only add inference if backend supports it
        if caps.supports_inference {
            let pgie = factory.create_inference(Some("primary-nvinference-engine"), config::PGIE_CONFIG_FILE)?;
            elements.push(pgie);
            
            let sgie1 = factory.create_inference(Some("secondary-nvinference-engine1"), config::SGIE1_CONFIG_FILE)?;
            elements.push(sgie1);
            
            let sgie2 = factory.create_inference(Some("secondary-nvinference-engine2"), config::SGIE2_CONFIG_FILE)?;
            elements.push(sgie2);
            
            let sgie3 = factory.create_inference(Some("secondary-nvinference-engine3"), config::SGIE3_CONFIG_FILE)?;
            elements.push(sgie3);
        }
        
        // Only add tracker if backend supports it
        if caps.supports_tracking {
            let tracker = factory.create_tracker(Some("nvtracker"))?;
            // Only set tracker-config-file for DeepStream backend
            if self.backend_manager.backend_type() == crate::backend::BackendType::DeepStream {
                tracker.set_property_from_str("tracker-config-file", config::TRACKER_CONFIG_FILE);
            }
            elements.push(tracker);
        }
        
        // Add tiler for multi-source display
        let tiler = factory.create_tiler(Some("nvtiler"))?;
        if self.backend_manager.backend_type() == crate::backend::BackendType::DeepStream {
            tiler.set_property("rows", config::TILER_ROWS as u32);
            tiler.set_property("columns", config::TILER_COLUMNS as u32);
            tiler.set_property("width", config::TILED_OUTPUT_WIDTH as u32);
            tiler.set_property("height", config::TILED_OUTPUT_HEIGHT as u32);
        }
        // Compositor doesn't have these properties, it uses pads for positioning
        elements.push(tiler);
        
        // Add conversion and output
        let convert = factory.create_video_convert(Some("nvvideo-converter"))?;
        elements.push(convert);
        
        if caps.supports_osd {
            let osd = factory.create_osd(Some("nv-onscreendisplay"))?;
            elements.push(osd);
        }
        
        let sink = factory.create_video_sink(Some("video-sink"))?;
        sink.set_property("sync", false);
        // autovideosink doesn't have qos property
        if self.backend_manager.backend_type() == crate::backend::BackendType::DeepStream {
            sink.set_property("qos", false);
        }
        elements.push(sink);
        
        // Add all elements to pipeline
        for element in &elements {
            self.pipeline.add_element(element)?;
        }
        
        // Link elements
        for i in 0..elements.len() - 1 {
            elements[i].link(&elements[i + 1])?;
        }
        
        // Create source controller with the streammux
        let pipeline_clone = self.pipeline.clone();
        self.source_controller = Arc::new(Mutex::new(
            SourceController::with_max_sources(
                pipeline_clone,
                streammux,
                config::MAX_NUM_SOURCES,
            )
        ));
        
        Ok(())
    }
    
    pub fn add_initial_source(&self) -> Result<()> {
        let controller = self.source_controller.lock().unwrap();
        controller.add_source(&self.initial_uri)?;
        println!("Added initial source: {}", self.initial_uri);
        Ok(())
    }
    
    pub async fn run(&mut self) -> Result<()> {
        {
            let mut running = self.running.lock().unwrap();
            *running = true;
        }
        
        self.pipeline.set_state(gst::State::Paused)?;
        self.add_initial_source()?;
        self.pipeline.set_state(gst::State::Playing)?;
        
        println!("Now playing: {}", self.initial_uri);
        println!("Pipeline running...");
        println!("Sources will be automatically added every {} seconds", config::SOURCE_ADD_INTERVAL_SECS);
        
        // Start source addition timer
        let source_controller = self.source_controller.clone();
        let running = self.running.clone();
        let initial_uri = self.initial_uri.clone();
        
        let add_sources_handle = tokio::spawn(async move {
            timers::source_addition_timer(source_controller, running, initial_uri).await
        });
        
        // Run main event loop
        let shutdown_rx = self.shutdown_rx.take().unwrap();
        runner::run_main_loop(self.pipeline.clone(), shutdown_rx).await?;
        
        add_sources_handle.abort();
        
        self.cleanup()?;
        Ok(())
    }
    
    pub fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        *running = false;
        
        let _ = self.shutdown_tx.try_send(());
        Ok(())
    }
    
    fn cleanup(&self) -> Result<()> {
        println!("Returned, stopping playback");
        self.pipeline.set_state(gst::State::Null)?;
        
        println!("Deleting pipeline");
        let controller = self.source_controller.lock().unwrap();
        controller.remove_all_sources()?;
        
        println!("Cleanup complete");
        Ok(())
    }
}