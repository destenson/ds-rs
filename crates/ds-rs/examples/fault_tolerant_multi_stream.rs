#![allow(unused)]
use ds_rs::{init, timestamp};
use ds_rs::pipeline::Pipeline;
use ds_rs::source::FaultTolerantSourceController;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize GStreamer
    init()?;
    
    println!("[{:.3}] Starting fault-tolerant multi-stream demo", timestamp());
    
    // Create pipeline
    let pipeline = Arc::new(Pipeline::new("fault-tolerant-demo")?);
    
    // Create a simple streammux element
    let streammux = gst::ElementFactory::make("identity")
        .name("streammux")
        .build()?;
    
    // Add to the pipeline
    pipeline.add_element(&streammux)?;
    
    let controller = Arc::new(FaultTolerantSourceController::new(
        pipeline.clone(),
        streammux,
    ));
    
    // Handle Ctrl+C
    ctrlc::set_handler(move || {
        println!("\n[{:.3}] Received interrupt signal, shutting down...", timestamp());
        std::process::exit(0);
    })?;
    
    // Add test sources - some may fail initially or during operation
    let test_sources = vec![
        "rtsp://127.0.0.1:8554/test1",  // May not exist initially
        "rtsp://127.0.0.1:8554/test2",  // May be intermittent
        "file:///nonexistent.mp4",      // Will fail but recover attempts will be made
    ];
    
    println!("[{:.3}] Adding {} test sources with fault tolerance", timestamp(), test_sources.len());
    
    for uri in &test_sources {
        match controller.add_source(uri) {
            Ok(id) => println!("[{:.3}] Added source {}: {}", timestamp(), id, uri),
            Err(e) => println!("[{:.3}] Failed to add source {}: {}", timestamp(), uri, e),
        }
    }
    
    // Start pipeline
    pipeline.set_state(gstreamer::State::Playing)?;
    println!("[{:.3}] Pipeline started in PLAYING state", timestamp());
    
    // Monitor sources in background
    let controller_clone = controller.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            
            if let Ok(sources) = controller_clone.list_active_sources() {
                println!("[{:.3}] Active sources: {}", timestamp(), sources.len());
                for (id, uri, state) in sources {
                    println!("[{:.3}]   {} - {} [{:?}]", timestamp(), id, uri, state);
                }
            }
        }
    });
    
    // Simulate source failures for testing
    let controller_clone = controller.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(15));
        println!("[{:.3}] Simulating source failure for testing recovery", timestamp());
        
        if let Ok(sources) = controller_clone.list_active_sources() {
            if let Some((id, _, _)) = sources.first() {
                // Force restart to test recovery
                if let Err(e) = controller_clone.restart_source(*id) {
                    println!("[{:.3}] Error restarting source: {}", timestamp(), e);
                } else {
                    println!("[{:.3}] Source {} restarted successfully", timestamp(), id);
                }
            }
        }
    });
    
    println!("[{:.3}] Running... Press Ctrl+C to exit", timestamp());
    
    // Keep the application running
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
