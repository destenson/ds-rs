#![allow(unused)]
//! Example application demonstrating metadata extraction and object detection
//!
//! This example shows how to:
//! - Build a pipeline with inference elements
//! - Extract metadata from buffers
//! - Process detection results
//! - Track objects across frames
//! - Handle DeepStream messages

use ds_rs::elements::factory::ElementFactory;
use ds_rs::{
    BackendManager, DSMessageHandler, DSMessageType, InferenceProcessor, MetadataExtractor,
    ObjectTracker, init,
};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Detection statistics
struct DetectionStats {
    frames_processed: u64,
    total_detections: u64,
    detections_by_class: HashMap<String, u64>,
}

impl DetectionStats {
    fn new() -> Self {
        Self {
            frames_processed: 0,
            total_detections: 0,
            detections_by_class: HashMap::new(),
        }
    }

    fn update(&mut self, class_name: &str) {
        self.total_detections += 1;
        *self
            .detections_by_class
            .entry(class_name.to_string())
            .or_insert(0) += 1;
    }

    fn print(&self) {
        println!("\n=== Detection Statistics ===");
        println!("Frames processed: {}", self.frames_processed);
        println!("Total detections: {}", self.total_detections);
        println!("Detections by class:");
        for (class, count) in &self.detections_by_class {
            println!("  {}: {}", class, count);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the library
    init()?;

    println!("DeepStream Rust - Object Detection Example");
    println!("==========================================\n");

    // Create backend manager
    let backend_manager = Arc::new(BackendManager::new()?);
    println!("Using backend: {}", backend_manager.backend_type().name());

    // Create element factory
    let factory = ElementFactory::new(backend_manager.clone());

    // Build detection pipeline
    let pipeline = build_detection_pipeline(&factory)?;

    // Set up metadata extraction
    let metadata_extractor = Arc::new(MetadataExtractor::new());
    let object_tracker = Arc::new(Mutex::new(ObjectTracker::new(100, 30, 50)));
    let _inference_processor = Arc::new(InferenceProcessor::default());
    let stats = Arc::new(Mutex::new(DetectionStats::new()));

    // Set up message handler
    let message_handler = Arc::new(DSMessageHandler::new());
    setup_message_callbacks(&message_handler);

    // Add probe to extract metadata
    add_metadata_probe(
        &pipeline,
        metadata_extractor.clone(),
        object_tracker.clone(),
        stats.clone(),
    )?;

    // Set up bus watch
    setup_bus_watch(&pipeline, message_handler.clone())?;

    // Start pipeline
    println!("Starting pipeline...");
    pipeline.set_state(gst::State::Playing)?;

    // Run for a while
    println!("Processing... Press Ctrl+C to stop\n");

    // Set up signal handler
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        println!("\nStopping pipeline...");
        if let Ok(mut r) = running_clone.lock() {
            *r = false;
        }
    })?;

    // Main loop
    while *running.lock().unwrap() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Stop pipeline
    pipeline.set_state(gst::State::Null)?;

    // Print final statistics
    if let Ok(s) = stats.lock() {
        s.print();
    }

    println!("\nExample completed successfully!");

    Ok(())
}

/// Build a detection pipeline
fn build_detection_pipeline(
    factory: &ElementFactory,
) -> Result<gst::Pipeline, Box<dyn std::error::Error>> {
    let pipeline = gst::Pipeline::builder().name("detection-pipeline").build();

    // Create elements
    let source = gst::ElementFactory::make("videotestsrc")
        .name("source")
        .build()?;
    let caps = gst::Caps::builder("video/x-raw")
        .field("width", 1920i32)
        .field("height", 1080i32)
        .field("framerate", gst::Fraction::new(30, 1))
        .build();
    let capsfilter = gst::ElementFactory::make("capsfilter")
        .property("caps", &caps)
        .build()?;

    let convert = factory.create_video_convert(Some("convert"))?;
    let scale = gst::ElementFactory::make("videoscale").build()?;
    let sink = gst::ElementFactory::make("fakesink").name("sink").build()?;

    // Configure test source
    source.set_property_from_str("pattern", "ball");
    source.set_property("num-buffers", 300i32);

    // Add elements to pipeline
    pipeline.add_many([&source, &capsfilter, &convert, &scale, &sink])?;

    // Link elements
    gst::Element::link_many([&source, &capsfilter, &convert, &scale, &sink])?;

    Ok(pipeline)
}

/// Add metadata extraction probe
fn add_metadata_probe(
    pipeline: &gst::Pipeline,
    extractor: Arc<MetadataExtractor>,
    tracker: Arc<Mutex<ObjectTracker>>,
    stats: Arc<Mutex<DetectionStats>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the sink element
    let sink = pipeline.by_name("sink").ok_or("Sink element not found")?;

    // Get sink pad
    let sinkpad = sink.static_pad("sink").ok_or("Sink pad not found")?;

    // Add probe
    sinkpad.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
        if let Some(buffer) = info.buffer() {
            // Extract metadata
            if let Ok(batch_meta) = extractor.extract_batch_meta(buffer) {
                // Process each frame
                for frame in batch_meta.frames() {
                    // Update statistics
                    if let Ok(mut s) = stats.lock() {
                        s.frames_processed += 1;

                        // Process objects
                        for obj in frame.objects() {
                            // Track object
                            if let Ok(mut t) = tracker.lock() {
                                if obj.is_tracked() {
                                    t.update_track(obj.object_id, obj, buffer.pts().map(|p| p.nseconds()).unwrap_or(0)).ok();
                                } else {
                                    let track_id = t.create_track(obj);
                                    println!("New track created: ID={}", track_id);
                                }
                            }

                            // Update detection stats
                            s.update(obj.class_name());

                            // Print detection info
                            if s.total_detections % 10 == 0 {
                                println!(
                                    "Detection: class={}, confidence={:.2}, bbox=({:.0},{:.0},{:.0},{:.0})",
                                    obj.class_name(),
                                    obj.confidence,
                                    obj.rect_params.left,
                                    obj.rect_params.top,
                                    obj.rect_params.width,
                                    obj.rect_params.height
                                );
                            }
                        }
                    }
                }

                // Print tracker statistics periodically
                if let Ok(s) = stats.lock() {
                    if s.frames_processed % 100 == 0 {
                        if let Ok(t) = tracker.lock() {
                            let tracker_stats = t.get_stats();
                            println!(
                                "Frame {}: Active tracks={}, Total tracks={}",
                                s.frames_processed,
                                tracker_stats.active_tracks,
                                tracker_stats.total_tracks
                            );
                        }
                    }
                }
            }
        }

        gst::PadProbeReturn::Ok
    });

    Ok(())
}

/// Set up message callbacks
fn setup_message_callbacks(handler: &Arc<DSMessageHandler>) {
    // Handle stream EOS
    handler.register_callback("stream_eos", |msg| {
        if let DSMessageType::StreamEos(stream_id) = msg {
            println!("Stream {} received EOS", stream_id);
        }
    });

    // Handle stream added
    handler.register_callback("stream_added", |msg| {
        if let DSMessageType::StreamAdded(stream_id) = msg {
            println!("Stream {} added", stream_id);
        }
    });
}

/// Set up bus watch
fn setup_bus_watch(
    pipeline: &gst::Pipeline,
    handler: Arc<DSMessageHandler>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bus = pipeline.bus().ok_or("Pipeline has no bus")?;

    bus.add_watch(move |_bus, msg| {
        // Handle DeepStream messages
        handler.handle_message(msg).ok();

        // Handle standard GStreamer messages
        match msg.view() {
            gst::MessageView::Eos(_) => {
                println!("End of stream");
                gstreamer::glib::ControlFlow::Break
            }
            gst::MessageView::Error(err) => {
                eprintln!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                gstreamer::glib::ControlFlow::Break
            }
            gst::MessageView::StateChanged(state_changed) => {
                if state_changed
                    .src()
                    .map(|s| s == msg.src().unwrap())
                    .unwrap_or(false)
                {
                    println!(
                        "[{:.3}] Pipeline state changed from {:?} to {:?}",
                        ds_rs::timestamp(),
                        state_changed.old(),
                        state_changed.current()
                    );
                }
                gstreamer::glib::ControlFlow::Continue
            }
            _ => gstreamer::glib::ControlFlow::Continue,
        }
    })?;

    Ok(())
}
