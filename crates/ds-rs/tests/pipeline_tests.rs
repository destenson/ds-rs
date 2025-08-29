use ds_rs::{BackendType, Pipeline, PipelineBuilder, init};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_simple_pipeline_creation() {
    init().unwrap();

    let pipeline = Pipeline::builder("test-pipeline")
        .backend(BackendType::Mock)
        .add_test_source("source")
        .add_auto_sink("sink")
        .link("source", "sink")
        .build();

    assert!(pipeline.is_ok());
    let pipeline = pipeline.unwrap();
    assert_eq!(pipeline.name(), "test-pipeline");
}

#[test]
fn test_pipeline_with_queue() {
    init().unwrap();

    let pipeline = PipelineBuilder::new("queue-pipeline")
        .backend(BackendType::Mock)
        .add_test_source("source")
        .add_queue("queue")
        .add_auto_sink("sink")
        .link("source", "queue")
        .link("queue", "sink")
        .build();

    assert!(pipeline.is_ok());
}

#[test]
fn test_pipeline_state_transitions() {
    init().unwrap();

    let pipeline = Pipeline::builder("state-test")
        .backend(BackendType::Mock)
        .add_test_source("source")
        .add_auto_sink("sink")
        .link("source", "sink")
        .build()
        .unwrap();

    // Test NULL -> READY -> PAUSED -> PLAYING
    assert_eq!(pipeline.current_state().unwrap(), gst::State::Null);

    pipeline.set_state(gst::State::Ready).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(pipeline.current_state().unwrap(), gst::State::Ready);

    pipeline.pause().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(pipeline.is_paused());

    pipeline.play().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(pipeline.is_playing());

    // Test PLAYING -> PAUSED -> READY -> NULL
    pipeline.pause().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(pipeline.is_paused());

    pipeline.stop().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(pipeline.current_state().unwrap(), gst::State::Null);
}

#[test]
fn test_pipeline_with_properties() {
    init().unwrap();

    let pipeline = PipelineBuilder::new("property-test")
        .backend(BackendType::Mock)
        .add_element("source", "videotestsrc")
        .set_property_from_str("source", "pattern", "smpte") // Use string for enum
        .set_property("source", "num-buffers", 100i32)
        .set_property("source", "is-live", false)
        .add_auto_sink("sink")
        .link("source", "sink")
        .build();

    assert!(pipeline.is_ok());
    let pipeline = pipeline.unwrap();

    // Get the source element and verify properties
    let source = pipeline.get_by_name("source").unwrap();
    assert_eq!(source.property::<i32>("num-buffers"), 100);
    assert_eq!(source.property::<bool>("is-live"), false);
    // Pattern is an enum, just check it was set (can't easily compare enum values)
}

#[test]
fn test_pipeline_with_caps_filter() {
    init().unwrap();

    let caps = gst::Caps::builder("video/x-raw")
        .field("width", 320)
        .field("height", 240)
        .field("framerate", gst::Fraction::new(30, 1))
        .build();

    let pipeline = PipelineBuilder::new("caps-test")
        .backend(BackendType::Mock)
        .add_test_source("source")
        .add_caps_filter("filter", caps.clone())
        .add_auto_sink("sink")
        .link("source", "filter")
        .link("filter", "sink")
        .build();

    assert!(pipeline.is_ok());
    let pipeline = pipeline.unwrap();

    // Verify caps filter was set correctly
    let filter = pipeline.get_by_name("filter").unwrap();
    let filter_caps = filter.property::<gst::Caps>("caps");
    assert_eq!(filter_caps, caps);
}

#[test]
fn test_pipeline_element_management() {
    init().unwrap();

    let pipeline = Pipeline::new("element-test").unwrap();

    // Create elements
    let source = gst::ElementFactory::make("fakesrc")
        .name("test-source")
        .build()
        .unwrap();
    let sink = gst::ElementFactory::make("fakesink")
        .name("test-sink")
        .build()
        .unwrap();

    // Add elements to pipeline
    pipeline.add_element(&source).unwrap();
    pipeline.add_element(&sink).unwrap();

    // Link elements
    pipeline.link_elements(&source, &sink).unwrap();

    // Verify elements are in pipeline
    assert!(pipeline.get_by_name("test-source").is_some());
    assert!(pipeline.get_by_name("test-sink").is_some());

    // Remove an element
    pipeline.remove_element(&source).unwrap();
    assert!(pipeline.get_by_name("test-source").is_none());
}

#[test]
fn test_pipeline_bus_messages() {
    init().unwrap();

    let message_count = Arc::new(Mutex::new(0));
    let message_count_clone = message_count.clone();

    let mut pipeline = PipelineBuilder::new("bus-test")
        .backend(BackendType::Mock)
        .add_element("source", "videotestsrc")
        .set_property("source", "num-buffers", 10i32)
        .add_auto_sink("sink")
        .link("source", "sink")
        .build()
        .unwrap();

    // Start watching bus
    pipeline
        .start_bus_watch(move |_bus, msg| {
            if let Ok(mut count) = message_count_clone.lock() {
                *count += 1;
            }

            match msg.view() {
                gst::MessageView::Eos(_) => {
                    log::info!("Received EOS in test");
                }
                gst::MessageView::Error(err) => {
                    log::error!("Error in test: {}", err.error());
                }
                _ => {}
            }

            gst::BusSyncReply::Pass
        })
        .unwrap();

    // Play pipeline briefly
    pipeline.play().unwrap();
    std::thread::sleep(Duration::from_millis(500));
    pipeline.stop().unwrap();

    // Check that we received some messages
    assert!(*message_count.lock().unwrap() > 0);
}

#[test]
fn test_pipeline_eos_handling() {
    init().unwrap();

    let pipeline = PipelineBuilder::new("eos-test")
        .backend(BackendType::Mock)
        .add_element("source", "videotestsrc")
        .set_property("source", "num-buffers", 1i32)
        .add_auto_sink("sink")
        .link("source", "sink")
        .build()
        .unwrap();

    // Play and wait for EOS
    pipeline.play().unwrap();
    let result = pipeline.wait_for_eos(Some(Duration::from_secs(5)));

    // With limited buffers, we should get EOS
    assert!(result.is_ok());

    pipeline.stop().unwrap();
}

#[test]
fn test_pipeline_builder_fluent_api() {
    init().unwrap();

    // Test the fluent API with method chaining
    let pipeline = Pipeline::builder("fluent-test")
        .backend(BackendType::Mock)
        .add_test_source("src1")
        .add_test_source("src2")
        .add_element("mixer", "videomixer")
        .add_auto_sink("sink")
        .link("src1", "mixer")
        .link("src2", "mixer")
        .link("mixer", "sink")
        .auto_flush_bus(true)
        .start_paused(true)
        .build();

    assert!(pipeline.is_ok());
    let pipeline = pipeline.unwrap();

    // Should start paused
    assert!(pipeline.is_paused());

    // Verify all elements exist
    assert!(pipeline.get_by_name("src1").is_some());
    assert!(pipeline.get_by_name("src2").is_some());
    assert!(pipeline.get_by_name("mixer").is_some());
    assert!(pipeline.get_by_name("sink").is_some());
}

#[test]
fn test_pipeline_with_file_source() {
    init().unwrap();

    let pipeline = PipelineBuilder::new("file-test")
        .backend(BackendType::Mock)
        .add_file_source("source", "/tmp/test.mp4")
        .add_element("decoder", "decodebin")
        .add_auto_sink("sink")
        .link("source", "decoder")
        // Note: decodebin has dynamic pads, would need pad-added handler
        .build();

    assert!(pipeline.is_ok());
    let pipeline = pipeline.unwrap();

    // Verify file source was configured
    let source = pipeline.get_by_name("source").unwrap();
    assert_eq!(source.property::<String>("location"), "/tmp/test.mp4");
}

#[test]
fn test_standard_backend_pipeline() {
    init().unwrap();

    // Test with standard GStreamer backend
    let pipeline = PipelineBuilder::new("standard-test")
        .backend(BackendType::Standard)
        .add_test_source("source")
        .add_queue("queue")
        .add_auto_sink("sink")
        .link("source", "queue")
        .link("queue", "sink")
        .build();

    assert!(pipeline.is_ok());
    let pipeline = pipeline.unwrap();

    // Verify backend type
    assert_eq!(
        pipeline.backend_manager().backend_type(),
        BackendType::Standard
    );
}

#[test]
fn test_pipeline_clock_management() {
    init().unwrap();

    let pipeline = Pipeline::builder("clock-test")
        .backend(BackendType::Mock)
        .add_test_source("source")
        .add_auto_sink("sink")
        .link("source", "sink")
        .build()
        .unwrap();

    // Test setting a system clock
    let system_clock = gst::SystemClock::obtain();
    pipeline.use_clock(Some(&system_clock));

    // The pipeline might not have a clock until it's running
    // Just verify we can call the methods without panicking
    let _ = pipeline.clock();

    // Test removing clock
    pipeline.use_clock(None);
}

#[test]
fn test_multiple_element_linking() {
    init().unwrap();

    let pipeline = PipelineBuilder::new("chain-test")
        .backend(BackendType::Mock)
        .add_test_source("source")
        .add_queue("queue1")
        .add_element("convert", "videoconvert")
        .add_queue("queue2")
        .add_auto_sink("sink")
        .link_many(vec![
            "source".to_string(),
            "queue1".to_string(),
            "convert".to_string(),
            "queue2".to_string(),
            "sink".to_string(),
        ])
        .build();

    assert!(pipeline.is_ok());
    let pipeline = pipeline.unwrap();

    // Verify all elements exist and can play
    pipeline.play().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(pipeline.is_playing());
    pipeline.stop().unwrap();
}
