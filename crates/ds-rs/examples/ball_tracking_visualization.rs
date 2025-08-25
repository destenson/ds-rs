#![allow(unused)]
//! Ball tracking visualization example
//! 
//! This example demonstrates real-time bounding box rendering around detected balls
//! using the integrated detection and rendering pipeline.

use ds_rs::{
    init, timestamp, Result, DeepStreamError,
    BackendManager, BackendType, PipelineBuilder, MetadataBridge,
    SourceController, SourceEvent, SourceId,
};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Main application state
struct BallTrackingApp {
    pipeline: gst::Pipeline,
    source_controller: Arc<SourceController>,
    metadata_bridge: Arc<Mutex<MetadataBridge>>,
    running: Arc<AtomicBool>,
}

impl BallTrackingApp {
    /// Create a new ball tracking application
    fn new() -> Result<Self> {
        // Initialize DeepStream/GStreamer
        init()?;
        
        // Create backend manager
        let backend_manager = Arc::new(BackendManager::new()?);
        log::info!("[{:.3}] Using {} backend", timestamp(), backend_manager.backend_type());
        
        // Create metadata bridge for connecting detection to rendering
        let metadata_bridge = Arc::new(Mutex::new(MetadataBridge::new()));
        
        // Build pipeline with ball tracking rendering
        let mut builder = PipelineBuilder::new("ball-tracking-viz")
            .backend(backend_manager.backend_type())
            // Source mux for multiple video inputs
            .add_element("streammux", "nvstreammux");
        
        // Configure properties based on backend type
        // Standard backend uses compositor which doesn't have width/height properties
        if backend_manager.backend_type() == BackendType::DeepStream {
            builder = builder
                .set_property("streammux", "width", 1920i32)
                .set_property("streammux", "height", 1080i32)
                .set_property("streammux", "batch-size", 1i32)
                .set_property("streammux", "batched-push-timeout", 40000i32);
        }
        
        // Object detection
        builder = builder.add_element("detector", "nvinfer");
        
        // Only set config-file-path for DeepStream backend
        // Standard backend CPU detector doesn't have this property
        if backend_manager.backend_type() == BackendType::DeepStream {
            builder = builder
                .set_property_from_str("detector", "config-file-path", "models/ball_detection_config.txt");
        }
        
        // Object tracking  
        builder = builder.add_element("tracker", "nvtracker");
        
        // Only set tracker properties for DeepStream backend
        if backend_manager.backend_type() == BackendType::DeepStream {
            builder = builder
                .set_property_from_str("tracker", "ll-lib-file", "/opt/nvidia/deepstream/deepstream/lib/libnvds_nvmultiobjecttracker.so")
                .set_property_from_str("tracker", "ll-config-file", "tracker_config.yml");
        }
        
        builder = builder
            // Video conversion
            .add_element("converter", "nvvideoconvert")
            // OSD with dynamic rendering
            .add_dynamic_osd("osd")
            // Tiler for multiple streams
            .add_element("tiler", "nvtiler");
        
        // Only set tiler width/height for DeepStream backend
        if backend_manager.backend_type() == BackendType::DeepStream {
            builder = builder
                .set_property("tiler", "rows", 2i32)
                .set_property("tiler", "columns", 2i32)
                .set_property("tiler", "width", 1920i32)
                .set_property("tiler", "height", 1080i32);
        }

        // Use platform-appropriate sink
        let sink_type = if cfg!(target_os = "windows") {
            "d3dvideosink"
        } else {
            "nveglglessink"
        };

        
        let pipeline = builder
            // Output conversion
            .add_element("converter2", "nvvideoconvert")
            // Sink
            .add_element("sink", sink_type)
            .set_property("sink", "sync", false)
            // Link elements
            .link("streammux", "detector")
            .link("detector", "tracker")
            .link("tracker", "converter")
            .link("converter", "osd")
            .link("osd", "tiler")
            .link("tiler", "converter2")
            .link("converter2", "sink")
            // Enable ball tracking rendering
            .with_ball_tracking_rendering()
            .with_metadata_bridge(metadata_bridge.clone())
            .build()?;
        
        // Store the pipeline returned from builder as Arc for SourceController
        let pipeline_arc = Arc::new(pipeline);
        
        // Get the streammux element from the pipeline
        let streammux = pipeline_arc.get_by_name("streammux")
            .ok_or_else(|| DeepStreamError::ElementNotFound { element: "streammux".to_string() })?;
        
        // Create source controller for dynamic source management
        let source_controller = Arc::new(SourceController::new(
            pipeline_arc.clone(),
            streammux,
        ));
        
        // Set up event handler for source events
        let event_handler = source_controller.get_event_handler();
        let metadata_bridge_clone = metadata_bridge.clone();
        
        event_handler.register_callback(move |event| {
            match event {
                SourceEvent::SourceAdded { id, uri } => {
                    log::info!("[{:.3}] Source {:?} added to pipeline (URI: {})", timestamp(), id, uri);
                },
                SourceEvent::SourceRemoved { id } => {
                    log::info!("[{:.3}] Source {:?} removed from pipeline", timestamp(), id);
                },
                SourceEvent::StateChanged { id, old_state, new_state } => {
                    log::debug!("[{:.3}] Source {:?} state changed from {:?} to {:?}", timestamp(), id, old_state, new_state);
                },
                SourceEvent::Eos { id } => {
                    log::info!("[{:.3}] Source {:?} reached end-of-stream", timestamp(), id);
                },
                SourceEvent::PadAdded { id, pad_name } => {
                    log::info!("[{:.3}] Source {:?} pad added: {:?}", timestamp(), id, pad_name);
                },
                SourceEvent::PadRemoved { id, pad_name } => {
                    log::info!("[{:.3}] Source {:?} pad removed: {:?}", timestamp(), id, pad_name);
                },
                SourceEvent::Warning { id, warning } => {
                    log::warn!("[{:.3}] Source {:?} warning: {:?}", timestamp(), id, warning);
                },
                SourceEvent::Error { id, error } => {
                    log::error!("[{:.3}] Source {:?} error: {:?}", timestamp(), id, error);
                },
            }
        });
        
        let gst_pipeline = pipeline_arc.gst_pipeline().clone();
        
        Ok(Self {
            pipeline: gst_pipeline,
            source_controller,
            metadata_bridge,
            running: Arc::new(AtomicBool::new(false)),
        })
    }
    
    /// Add a video source
    fn add_source(&self, uri: &str) -> Result<SourceId> {
        log::info!("[{:.3}] Adding source: {}", timestamp(), uri);
        let id = self.source_controller.add_source(uri)?;
        Ok(id)
    }
    
    /// Remove a video source
    fn remove_source(&self, source_id: SourceId) -> Result<()> {
        log::info!("[{:.3}] Removing source: {:?}", timestamp(), source_id);
        self.source_controller.remove_source(source_id)?;
        Ok(())
    }
    
    /// Start the pipeline
    fn start(&self) -> Result<()> {
        log::info!("[{:.3}] Starting ball tracking visualization pipeline", timestamp());
        self.pipeline.set_state(gst::State::Playing)
            .map_err(|e| DeepStreamError::Pipeline(format!("Failed to set pipeline to playing: {:?}", e)))?;
        self.running.store(true, Ordering::SeqCst);
        Ok(())
    }
    
    /// Stop the pipeline
    fn stop(&self) -> Result<()> {
        log::info!("[{:.3}] Stopping pipeline", timestamp());
        self.running.store(false, Ordering::SeqCst);
        self.pipeline.set_state(gst::State::Null)
            .map_err(|e| DeepStreamError::Pipeline(format!("Failed to set pipeline to null: {:?}", e)))?;
        Ok(())
    }
    
    /// Run the main loop
    fn run(&self) -> Result<()> {
        // Set up Ctrl+C handler
        let running = self.running.clone();
        ctrlc::set_handler(move || {
            log::info!("\n[{:.3}] Received interrupt signal, shutting down...", timestamp());
            running.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");
        
        // Monitor loop
        let start_time = std::time::Instant::now();
        
        while self.running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
            
            // Get rendering statistics
            if let Ok(bridge) = self.metadata_bridge.lock() {
                let stats = bridge.get_statistics();
                let frame_count = stats.frames_processed;
                
                let elapsed = start_time.elapsed().as_secs_f64();
                let fps = if elapsed > 0.0 {
                    frame_count as f64 / elapsed
                } else {
                    0.0
                };
                
                log::info!(
                    "[{:.3}] Frames: {} | FPS: {:.1} | Objects rendered: {} | Buffer: {}",
                    timestamp(),
                    frame_count,
                    fps,
                    stats.frames_processed,
                    stats.buffer_size
                );
            }
            
            // Check pipeline state
            let (_, state, _) = self.pipeline.state(gst::ClockTime::from_seconds(0));
            if state != gst::State::Playing && self.running.load(Ordering::SeqCst) {
                log::warn!("[{:.3}] Pipeline not in PLAYING state: {:?}", timestamp(), state);
            }
        }
        
        Ok(())
    }
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    log::info!("[{:.3}] Ball Tracking Visualization Example", timestamp());
    log::info!("[{:.3}] =====================================", timestamp());
    
    // Create application
    let app = BallTrackingApp::new()?;
    
    // Add video sources (RTSP streams or files)
    // For test patterns, use a special URI format that will be handled
    app.add_source("videotestsrc://")?;
    
    // For RTSP streams from source-videos server:
    // app.add_source("rtsp://127.0.0.1:8554/test1")?;
    // app.add_source("rtsp://127.0.0.1:8554/test2")?;
    
    // For video files:
    // app.add_source("file:///path/to/video.mp4")?;
    
    // Start pipeline
    app.start()?;
    
    // Add another source after 5 seconds (demonstrates dynamic addition)
    thread::spawn({
        let app_controller = app.source_controller.clone();
        move || {
            thread::sleep(Duration::from_secs(5));
            log::info!("[{:.3}] Adding second source dynamically", timestamp());
            let _ = app_controller.add_source(
                "videotestsrc://"
            );
        }
    });
    
    // Run main loop
    app.run()?;
    
    // Clean shutdown
    app.stop()?;
    
    log::info!("[{:.3}] Ball tracking visualization example completed", timestamp());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_creation() {
        let result = BallTrackingApp::new();
        
        // May fail on systems without proper GStreamer plugins
        if let Err(e) = result {
            eprintln!("App creation failed (expected on CI): {}", e);
        }
    }
}
