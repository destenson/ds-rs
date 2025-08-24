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
use gstreamer::glib;
use std::sync::{Arc, Mutex};

/// Main application demonstrating runtime source addition/deletion
pub struct Application {
    pipeline: Arc<Pipeline>,
    source_controller: Arc<Mutex<SourceController>>,
    backend_manager: Arc<BackendManager>,
    initial_uri: String,
}

impl Application {
    pub fn new(uri: String) -> Result<Self> {
        let backend_manager = Arc::new(BackendManager::new()?);
        
        Ok(Self {
            pipeline: Arc::new(Pipeline::new("ds-runtime-demo")?),
            source_controller: Arc::new(Mutex::new(SourceController::new(
                Arc::new(Pipeline::new("dummy")?),
                gst::ElementFactory::make("fakesink").build()?,
            ))),
            backend_manager,
            initial_uri: uri,
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
            // Compositor doesn't have width/height properties - those are set on pads or with caps
        }
        
        // Create processing elements based on backend capabilities
        let caps = self.backend_manager.capabilities();
        
        let mut elements = vec![streammux.clone()];
        
        // Skip inference for Standard backend since it's causing issues
        if self.backend_manager.backend_type() != crate::backend::BackendType::Standard {
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
        }
        
        // Add tiler for multi-source display
        let tiler = factory.create_tiler(Some("nvtiler"))?;
        if self.backend_manager.backend_type() == crate::backend::BackendType::DeepStream {
            tiler.set_property("rows", config::TILER_ROWS as u32);
            tiler.set_property("columns", config::TILER_COLUMNS as u32);
            tiler.set_property("width", config::TILED_OUTPUT_WIDTH as u32);
            tiler.set_property("height", config::TILED_OUTPUT_HEIGHT as u32);
        }
        elements.push(tiler);
        
        // Add conversion and output
        let convert = factory.create_video_convert(Some("nvvideo-converter"))?;
        elements.push(convert);
        
        if caps.supports_osd && self.backend_manager.backend_type() != crate::backend::BackendType::Standard {
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
        let source_id = controller.add_source(&self.initial_uri)?;
        println!("Added initial source: {} (ID: {:?})", self.initial_uri, source_id);
        Ok(())
    }
    
    
    pub fn run_with_glib_signals(&mut self) -> Result<()> {
        println!("Starting pipeline...");
        
        // Create the GLib main loop
        let main_loop = glib::MainLoop::new(None, false);
        let main_loop_quit = main_loop.clone();
        
        // Get the bus for message handling
        let bus = self.pipeline.bus().unwrap();
        
        // Add bus watch for GStreamer messages  
        let _bus_watch = bus.add_watch(move |_, msg| {
            use gst::MessageView;
            
            match msg.view() {
                MessageView::Eos(..) => {
                    println!("End of stream");
                    main_loop_quit.quit();
                    glib::ControlFlow::Break
                }
                MessageView::Error(err) => {
                    eprintln!("Error: {}", err.error());
                    if let Some(debug) = err.debug() {
                        eprintln!("Debug: {}", debug);
                    }
                    main_loop_quit.quit();
                    glib::ControlFlow::Break
                }
                MessageView::Warning(warn) => {
                    // Log warnings but don't stop playback
                    if let Some(debug) = warn.debug() {
                        eprintln!("Warning: {} - {}", warn.error(), debug);
                    }
                    glib::ControlFlow::Continue
                }
                MessageView::StateChanged(state) => {
                    println!("State changed: {:?} -> {:?} ({})", 
                        state.old(), 
                        state.current(),
                        state.src().map(|s| s.name()).unwrap_or_else(|| "unknown".into())
                    );
                    if state.current() == gst::State::Playing {
                        println!("🎬 Pipeline is now PLAYING!");
                    }
                    glib::ControlFlow::Continue
                }
                _ => glib::ControlFlow::Continue,
            }
        })?;
        
        // Add SIGINT handler using GLib's signal handling
        #[cfg(unix)]
        {
            let main_loop_signal = main_loop.clone();
            let _signal_handler = glib::unix_signal_add(
                glib::Signal::SIGINT,
                move || {
                    println!("\nReceived interrupt signal, shutting down...");
                    main_loop_signal.quit();
                    glib::ControlFlow::Break
                }
            );
        }
        
        // On Windows, we'll still use ctrlc as glib unix signals don't work
        #[cfg(windows)]
        {
            let main_loop_ctrlc = main_loop.clone();
            ctrlc::set_handler(move || {
                println!("\nReceived interrupt signal, shutting down...");
                main_loop_ctrlc.quit();
            }).expect("Error setting Ctrl+C handler");
        }
        
        // Start the pipeline
        self.pipeline.set_state(gst::State::Paused)?;
        self.add_initial_source()?;
        println!("Setting pipeline to PLAYING state...");
        let state_change_result = self.pipeline.set_state(gst::State::Playing)?;
        println!("Pipeline state change result: {:?}", state_change_result);
        
        // Wait for the pipeline to reach playing state
        println!("Waiting for pipeline to reach PLAYING state...");
        match self.pipeline.get_state(Some(std::time::Duration::from_secs(5))) {
            Ok((result, current, pending)) => {
                println!("Final state: {:?} (current: {:?}, pending: {:?})", result, current, pending);
            }
            Err(err) => {
                eprintln!("Failed to get final state: {:?}", err);
            }
        }
        
        println!("Now playing: {}", self.initial_uri);
        println!("Pipeline running... Press Ctrl+C to exit");
        
        // Run the main loop - this will block until main_loop.quit() is called
        main_loop.run();
        
        println!("Shutting down pipeline...");
        self.cleanup()?;
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
