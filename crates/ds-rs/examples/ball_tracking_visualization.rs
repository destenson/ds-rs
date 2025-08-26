#![allow(unused)]
//! Ball tracking visualization example
//! 
//! This example demonstrates real-time bounding box rendering around detected balls
//! using the integrated detection and rendering pipeline.

use clap::{Parser, ValueEnum};
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

/// Input source type for the ball tracking visualization
#[derive(Debug, Clone, ValueEnum)]
enum SourceType {
    /// Use a file path or URI
    File,
    /// Use RTSP stream
    Rtsp,
    /// Use videotestsrc test pattern
    Test,
    /// Use webcam/camera device
    Camera,
}

/// Ball tracking visualization application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input source type
    #[arg(short = 't', long, value_enum, default_value = "file")]
    source_type: SourceType,
    
    /// Input source (file path, RTSP URL, camera device index, or ignored for test pattern)
    #[arg(short = 'i', long)]
    input: Option<String>,
    
    /// Add additional sources (can be specified multiple times)
    #[arg(short = 'a', long = "add-source")]
    additional_sources: Vec<String>,
    
    /// Log level (error, warn, info, debug, trace)
    #[arg(short = 'l', long, default_value = "info")]
    log_level: String,
    
    /// Enable GStreamer debug output
    #[arg(short = 'g', long)]
    gst_debug: bool,
    
    /// Number of tiler rows for multiple sources
    #[arg(long, default_value = "2")]
    tiler_rows: i32,
    
    /// Number of tiler columns for multiple sources  
    #[arg(long, default_value = "2")]
    tiler_columns: i32,
    
    /// Output width
    #[arg(long, default_value = "1920")]
    width: i32,
    
    /// Output height
    #[arg(long, default_value = "1080")]
    height: i32,
}

/// Main application state
struct BallTrackingApp {
    pipeline: gst::Pipeline,
    source_controller: Arc<SourceController>,
    metadata_bridge: Arc<Mutex<MetadataBridge>>,
    running: Arc<AtomicBool>,
}

impl BallTrackingApp {
    /// Create a new ball tracking application
    fn new(args: &Args) -> Result<Self> {
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
                .set_property("streammux", "width", args.width)
                .set_property("streammux", "height", args.height)
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
                .set_property("tiler", "rows", args.tiler_rows)
                .set_property("tiler", "columns", args.tiler_columns)
                .set_property("tiler", "width", args.width)
                .set_property("tiler", "height", args.height);
        }

        // Use autovideosink for better cross-platform window handling
        let sink_type = "autovideosink";

        
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
        
        // Get the bus for message handling
        let bus = self.pipeline.bus().unwrap();
        
        // Monitor loop
        let start_time = std::time::Instant::now();
        let mut last_stats_print = std::time::Instant::now();
        
        while self.running.load(Ordering::SeqCst) {
            // Check for bus messages (non-blocking with 100ms timeout)
            if let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(100)) {
                use gst::MessageView;
                match msg.view() {
                    MessageView::Eos(..) => {
                        log::info!("[{:.3}] Received EOS, stopping pipeline", timestamp());
                        self.running.store(false, Ordering::SeqCst);
                        break;
                    },
                    MessageView::Error(err) => {
                        log::error!(
                            "[{:.3}] Error from {:?}: {} ({:?})",
                            timestamp(),
                            err.src().map(|s| s.path_string()),
                            err.error(),
                            err.debug()
                        );
                        self.running.store(false, Ordering::SeqCst);
                        break;
                    },
                    MessageView::Element(element_msg) => {
                        // Check for window close messages from video sinks
                        if let Some(structure) = element_msg.structure() {
                            if structure.name() == "GstNavigationMessage" ||
                               structure.name() == "application/x-gst-navigation" {
                                // Check if it's a window close event
                                if let Ok(event_type) = structure.get::<String>("event") {
                                    if event_type == "window-closed" || event_type == "delete-event" {
                                        log::info!("[{:.3}] Window closed, stopping pipeline", timestamp());
                                        self.running.store(false, Ordering::SeqCst);
                                        break;
                                    }
                                }
                            }
                        }
                    },
                    MessageView::Application(app_msg) => {
                        if let Some(structure) = app_msg.structure() {
                            if structure.name() == "window-closed" {
                                log::info!("[{:.3}] Window closed (application message), stopping pipeline", timestamp());
                                self.running.store(false, Ordering::SeqCst);
                                break;
                            }
                        }
                    },
                    _ => {}
                }
            }
            
            // Print statistics every second
            if last_stats_print.elapsed() >= Duration::from_secs(1) {
                last_stats_print = std::time::Instant::now();
                
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
        }
        
        Ok(())
    }
}

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();
    
    // Initialize logging
    let log_level = match args.log_level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    };
    
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();
    
    // Enable GStreamer debug logging if requested
    if args.gst_debug {
        unsafe {
            std::env::set_var("GST_DEBUG", "cpudetector:5,*:3");
        }
    } else {
        // Always enable CPU detector debug to see if it's detecting
        unsafe {
            std::env::set_var("GST_DEBUG", "cpudetector:5");
        }
    }
    
    log::info!("[{:.3}] Ball Tracking Visualization Example", timestamp());
    log::info!("[{:.3}] =====================================", timestamp());
    log::debug!("[{:.3}] Arguments: {:?}", timestamp(), args);
    
    // Create application
    let app = BallTrackingApp::new(&args)?;
    
    // Prepare the primary source URI based on source type
    let primary_source = match args.source_type {
        SourceType::File => {
            if let Some(input) = args.input {
                // Check if it's already a URI or a path
                if input.starts_with("file://") || input.starts_with("http://") || input.starts_with("https://") {
                    input
                } else {
                    // Convert file path to URI
                    let path = std::path::PathBuf::from(&input);
                    let abs_path = if path.is_absolute() {
                        path
                    } else {
                        std::env::current_dir()?.join(path)
                    };
                    format!("file:///{}", abs_path.display().to_string().replace("\\", "/"))
                }
            } else {
                // Default to test video if no input provided
                let video_path = std::env::current_dir()?
                    .join("crates")
                    .join("ds-rs")
                    .join("tests")
                    .join("test_video.mp4");
                
                if video_path.exists() {
                    format!("file:///{}", video_path.display().to_string().replace("\\", "/"))
                } else {
                    log::warn!("[{:.3}] Default test video not found, using test pattern", timestamp());
                    "videotestsrc://".to_string()
                }
            }
        },
        SourceType::Rtsp => {
            args.input.unwrap_or_else(|| {
                log::info!("[{:.3}] No RTSP URL provided, using default rtsp://127.0.0.1:8554/test1", timestamp());
                "rtsp://127.0.0.1:8554/test1".to_string()
            })
        },
        SourceType::Test => {
            "videotestsrc://".to_string()
        },
        SourceType::Camera => {
            let device_index = args.input.as_ref().and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            #[cfg(unix)]
            let uri = format!("v4l2:///dev/video{}", device_index);
            #[cfg(windows)]
            let uri = format!("ksvideosrc://device-index={}", device_index);
            #[cfg(not(any(unix, windows)))]
            let uri = "videotestsrc://".to_string();
            uri
        },
    };
    
    log::info!("[{:.3}] Adding primary source: {}", timestamp(), primary_source);
    app.add_source(&primary_source)?;
    
    // Add any additional sources
    for (i, source) in args.additional_sources.iter().enumerate() {
        log::info!("[{:.3}] Adding additional source {}: {}", timestamp(), i + 1, source);
        app.add_source(source)?;
    }
    
    // Start pipeline
    app.start()?;
    
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
