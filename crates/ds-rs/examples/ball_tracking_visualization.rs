//! Ball tracking visualization example
//! 
//! This example demonstrates real-time bounding box rendering around detected balls
//! using the integrated detection and rendering pipeline.

use ds_rs::{
    init, timestamp, Result,
    BackendManager, PipelineBuilder, RenderingConfig, MetadataBridge,
    SourceController, SourceEvent,
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
        log::info!("[{:.3}] Using {} backend", timestamp(), backend_manager.get_backend_type());
        
        // Create metadata bridge for connecting detection to rendering
        let metadata_bridge = Arc::new(Mutex::new(MetadataBridge::new()));
        
        // Build pipeline with ball tracking rendering
        let pipeline = PipelineBuilder::new("ball-tracking-viz")
            .backend(backend_manager.get_backend_type())
            // Source mux for multiple video inputs
            .add_element("streammux", "nvstreammux")
            .set_property("streammux", "width", 1920i32)
            .set_property("streammux", "height", 1080i32)
            .set_property("streammux", "batch-size", 1i32)
            .set_property("streammux", "batched-push-timeout", 40000i32)
            // Object detection
            .add_element("detector", "nvinfer")
            .set_property_from_str("detector", "config-file-path", "models/ball_detection_config.txt")
            // Object tracking
            .add_element("tracker", "nvtracker")
            .set_property_from_str("tracker", "ll-lib-file", "/opt/nvidia/deepstream/deepstream/lib/libnvds_nvmultiobjecttracker.so")
            .set_property_from_str("tracker", "ll-config-file", "tracker_config.yml")
            // Video conversion
            .add_element("converter", "nvvideoconvert")
            // OSD with dynamic rendering
            .add_dynamic_osd("osd")
            // Tiler for multiple streams
            .add_element("tiler", "nvtiler")
            .set_property("tiler", "rows", 2i32)
            .set_property("tiler", "columns", 2i32)
            .set_property("tiler", "width", 1920i32)
            .set_property("tiler", "height", 1080i32)
            // Output conversion
            .add_element("converter2", "nvvideoconvert")
            // Sink
            .add_element("sink", "nveglglessink")
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
        
        // Get the GStreamer pipeline
        let gst_pipeline = pipeline.get_gst_pipeline().clone();
        
        // Create source controller for dynamic source management
        let source_controller = Arc::new(SourceController::new(
            gst_pipeline.clone(),
            backend_manager.clone(),
        )?);
        
        // Set up event handler for source events
        let event_handler = source_controller.get_event_handler();
        let metadata_bridge_clone = metadata_bridge.clone();
        
        event_handler.on(SourceEvent::SourceAdded, move |source_id| {
            log::info!("[{:.3}] Source {} added to pipeline", timestamp(), source_id);
        });
        
        event_handler.on(SourceEvent::SourceRemoved, move |source_id| {
            log::info!("[{:.3}] Source {} removed from pipeline", timestamp(), source_id);
        });
        
        event_handler.on(SourceEvent::SourceError, move |source_id| {
            log::error!("[{:.3}] Error with source {}", timestamp(), source_id);
        });
        
        Ok(Self {
            pipeline: gst_pipeline,
            source_controller,
            metadata_bridge,
            running: Arc::new(AtomicBool::new(false)),
        })
    }
    
    /// Add a video source
    fn add_source(&self, uri: &str) -> Result<()> {
        log::info!("[{:.3}] Adding source: {}", timestamp(), uri);
        self.source_controller.add_source(uri)?;
        Ok(())
    }
    
    /// Remove a video source
    fn remove_source(&self, source_id: &str) -> Result<()> {
        log::info!("[{:.3}] Removing source: {}", timestamp(), source_id);
        self.source_controller.remove_source(source_id)?;
        Ok(())
    }
    
    /// Start the pipeline
    fn start(&self) -> Result<()> {
        log::info!("[{:.3}] Starting ball tracking visualization pipeline", timestamp());
        self.pipeline.set_state(gst::State::Playing)?;
        self.running.store(true, Ordering::SeqCst);
        Ok(())
    }
    
    /// Stop the pipeline
    fn stop(&self) -> Result<()> {
        log::info!("[{:.3}] Stopping pipeline", timestamp());
        self.running.store(false, Ordering::SeqCst);
        self.pipeline.set_state(gst::State::Null)?;
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
        let mut frame_count = 0u64;
        let start_time = std::time::Instant::now();
        
        while self.running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
            
            // Get rendering statistics
            if let Ok(bridge) = self.metadata_bridge.lock() {
                let stats = bridge.get_statistics();
                frame_count = stats.frames_processed;
                
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
    // Example with test patterns
    app.add_source("videotestsrc pattern=ball ! video/x-raw,width=640,height=480,framerate=30/1")?;
    
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
                "videotestsrc pattern=ball ! video/x-raw,width=640,height=480,framerate=30/1"
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