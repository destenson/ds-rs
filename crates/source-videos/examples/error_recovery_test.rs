//! Example demonstrating network simulation for testing error recovery in ds-rs
//!
//! This example shows how to use source-videos network simulation to test
//! the error recovery features in ds-rs pipelines.

use anyhow::Result;
use gstreamer as gst;
use gstreamer::prelude::*;
use source_videos::network::{
    GStreamerNetworkSimulator, NetworkConditions, NetworkController, NetworkProfile,
    StandardProfiles,
};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<()> {
    // Initialize GStreamer
    gst::init()?;

    println!("Error Recovery Testing with Network Simulation");
    println!("==============================================\n");

    // Create a test pipeline that simulates a video source
    let pipeline = create_test_pipeline()?;

    // Add network simulation to the pipeline
    let simulator = add_network_simulation_to_pipeline(&pipeline)?;

    // Set up bus watch for error monitoring
    setup_bus_watch(&pipeline, Arc::clone(&simulator));

    // Start the pipeline
    pipeline.set_state(gst::State::Playing)?;
    println!("Pipeline started");

    // Run test scenarios
    run_test_scenarios(simulator)?;

    // Stop the pipeline
    pipeline.set_state(gst::State::Null)?;
    println!("\nTest complete");

    Ok(())
}

/// Create a test pipeline that simulates video streaming
fn create_test_pipeline() -> Result<gst::Pipeline> {
    let pipeline = gst::Pipeline::with_name("test-pipeline");

    // Create elements
    let source = gst::ElementFactory::make("videotestsrc")
        .name("source")
        .property("pattern", "ball")
        .property("is-live", true)
        .build()?;

    let convert = gst::ElementFactory::make("videoconvert")
        .name("convert")
        .build()?;

    let sink = gst::ElementFactory::make("autovideosink")
        .name("sink")
        .property("sync", true)
        .build()?;

    // Add elements to pipeline
    pipeline.add_many(&[&source, &convert, &sink])?;

    // Link elements (we'll insert simulation between convert and sink)
    source.link(&convert)?;
    convert.link(&sink)?;

    Ok(pipeline)
}

/// Add network simulation to the pipeline
fn add_network_simulation_to_pipeline(
    pipeline: &gst::Pipeline,
) -> Result<Arc<GStreamerNetworkSimulator>> {
    let simulator = Arc::new(GStreamerNetworkSimulator::new());

    // Get the elements we want to insert simulation between
    let convert = pipeline.by_name("convert").unwrap();
    let sink = pipeline.by_name("sink").unwrap();

    // Insert simulation elements
    simulator.insert_into_pipeline(pipeline, &convert, &sink, "error_test")?;

    println!("Network simulation added to pipeline");

    Ok(simulator)
}

/// Set up bus watch to monitor pipeline errors
fn setup_bus_watch(pipeline: &gst::Pipeline, simulator: Arc<GStreamerNetworkSimulator>) {
    let bus = pipeline.bus().unwrap();

    let _bus_watch = bus
        .add_watch(move |_, msg| {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    println!(
                        "ERROR: {} from {:?}",
                        err.error(),
                        err.src().map(|s| s.path_string())
                    );

                    // In a real error recovery system, this would trigger recovery
                    println!("  -> Triggering error recovery...");

                    // Restore network conditions as part of recovery
                    simulator.reset();
                    println!("  -> Network conditions reset");
                }
                gst::MessageView::Warning(warn) => {
                    println!(
                        "WARNING: {} from {:?}",
                        warn.error(),
                        warn.src().map(|s| s.path_string())
                    );
                }
                gst::MessageView::StateChanged(state) => {
                    if let Some(src) = state.src() {
                        if src.type_() == gst::Pipeline::static_type() {
                            println!(
                                "Pipeline state changed: {:?} -> {:?}",
                                state.old(),
                                state.current()
                            );
                        }
                    }
                }
                gst::MessageView::Buffering(buff) => {
                    let percent = buff.percent();
                    println!("Buffering: {}%", percent);
                }
                _ => {}
            }

            gst::glib::ControlFlow::Continue
        })
        .unwrap();
}

/// Run various network simulation test scenarios
fn run_test_scenarios(simulator: Arc<GStreamerNetworkSimulator>) -> Result<()> {
    println!("\n=== Test Scenario 1: Perfect Network ===");
    simulator.reset();
    println!("Running with perfect network conditions");
    std::thread::sleep(Duration::from_secs(3));

    println!("\n=== Test Scenario 2: Packet Loss ===");
    simulator.enable_with_conditions(NetworkConditions {
        packet_loss: 5.0,
        latency_ms: 0,
        bandwidth_kbps: 0,
        connection_dropped: false,
        jitter_ms: 0,
        duplicate_probability: 0.0,
        allow_reordering: true,
        min_delay_ms: 0,
        max_delay_ms: 0,
        delay_probability: 0.0,
    });
    println!("Applied 5% packet loss");
    std::thread::sleep(Duration::from_secs(3));

    println!("\n=== Test Scenario 3: High Latency ===");
    simulator.enable_with_conditions(NetworkConditions {
        packet_loss: 0.0,
        latency_ms: 500,
        bandwidth_kbps: 0,
        connection_dropped: false,
        jitter_ms: 100,
        duplicate_probability: 0.0,
        allow_reordering: true,
        min_delay_ms: 0,
        max_delay_ms: 0,
        delay_probability: 0.0,
    });
    println!("Applied 500ms latency with 100ms jitter");
    std::thread::sleep(Duration::from_secs(3));

    println!("\n=== Test Scenario 4: Bandwidth Limit ===");
    simulator.enable_with_conditions(NetworkConditions {
        packet_loss: 0.0,
        latency_ms: 0,
        bandwidth_kbps: 500, // 500 kbps
        connection_dropped: false,
        jitter_ms: 0,
        duplicate_probability: 0.0,
        allow_reordering: true,
        min_delay_ms: 0,
        max_delay_ms: 0,
        delay_probability: 0.0,
    });
    println!("Applied 500 kbps bandwidth limit");
    std::thread::sleep(Duration::from_secs(3));

    println!("\n=== Test Scenario 5: Connection Drop ===");
    println!("Dropping connection for 2 seconds...");
    simulator.simulate_connection_drop(Duration::from_secs(2));
    std::thread::sleep(Duration::from_secs(3));
    println!("Connection should be restored");

    println!("\n=== Test Scenario 6: 3G Mobile Network ===");
    simulator.enable_with_profile(NetworkProfile::Mobile3G);
    println!("Applied 3G mobile network profile");
    std::thread::sleep(Duration::from_secs(3));

    println!("\n=== Test Scenario 7: Poor Network (Stress Test) ===");
    simulator.enable_with_profile(NetworkProfile::Poor);
    println!("Applied poor network conditions for stress testing");
    println!("This should trigger error recovery mechanisms");
    std::thread::sleep(Duration::from_secs(5));

    println!("\n=== Test Scenario 8: Recovery Test ===");
    println!("Testing recovery from poor to perfect conditions");
    simulator.enable_with_profile(StandardProfiles::for_error_recovery());
    std::thread::sleep(Duration::from_secs(2));
    simulator.reset();
    println!("Recovered to perfect conditions");
    std::thread::sleep(Duration::from_secs(2));

    Ok(())
}
