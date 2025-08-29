#![allow(unused)]

//! Multi-stream detection pipeline example
//!
//! Demonstrates concurrent detection processing on multiple RTSP streams
//! with fault tolerance, resource management, and performance monitoring.

use ds_rs::{
    MultiStreamConfig, MultiStreamConfigBuilder, MultiStreamManager, Pipeline, PipelineBuilder,
    ResourceLimits, StreamPriority, backend::cpu_vision::DetectorConfig, init, timestamp,
};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize GStreamer
    init()?;

    println!("[{:.3}] Starting multi-stream detection demo", timestamp());

    // Configure multi-stream system
    let config = MultiStreamConfigBuilder::new()
        .max_streams(8)
        .resource_limits(ResourceLimits {
            max_cpu_percent: 80.0,
            max_memory_mb: 2048.0,
            max_streams: 8,
            adaptive_throttling: true,
            memory_per_stream_mb: 200.0,
        })
        .detector_config(DetectorConfig {
            model_path: Some("models/yolov8n.onnx".to_string()),
            input_width: 640,
            input_height: 640,
            confidence_threshold: 0.5,
            nms_threshold: 0.4,
            num_threads: 2,
            yolo_version: ds_rs::backend::cpu_vision::YoloVersion::V8,
            class_names: None,
        })
        .worker_threads(4)
        .debug_mode(true)
        .build();

    // Create pipeline
    let pipeline = Arc::new(Pipeline::new("multistream-detection")?);

    // Create a simple identity element as placeholder for streammux
    // In a real implementation, this would be nvstreammux or compositor
    let streammux = gst::ElementFactory::make("identity")
        .name("streammux")
        .build()?;

    pipeline.add_element(&streammux)?;

    // Create multi-stream manager
    let manager = Arc::new(MultiStreamManager::new(
        pipeline.clone(),
        streammux,
        config,
    )?);

    // Start monitoring
    manager.start_monitoring()?;

    // Handle Ctrl+C
    let manager_clone = manager.clone();
    ctrlc::set_handler(move || {
        println!(
            "\n[{:.3}] Received interrupt signal, shutting down...",
            timestamp()
        );

        // Print final statistics
        let stats = manager_clone.get_stats();
        println!("\nFinal Statistics:");
        println!("  Active streams: {}", stats.active_streams);
        println!("  Total frames: {}", stats.total_frames_processed);
        println!("  Total detections: {}", stats.total_detections);
        println!("  Average FPS: {:.1}", stats.average_fps);
        println!("  CPU usage: {:.1}%", stats.cpu_usage);
        println!("  Memory usage: {:.1} MB", stats.memory_usage_mb);

        std::process::exit(0);
    })?;

    // Define test streams with different priorities
    let test_streams = vec![
        ("rtsp://127.0.0.1:8554/test1", StreamPriority::High),
        ("rtsp://127.0.0.1:8554/test2", StreamPriority::Normal),
        ("rtsp://127.0.0.1:8554/test3", StreamPriority::Normal),
        ("file:///path/to/video.mp4", StreamPriority::Low),
    ];

    println!(
        "[{:.3}] Adding {} test streams for detection",
        timestamp(),
        test_streams.len()
    );

    // Add streams with different priorities
    for (uri, priority) in &test_streams {
        match manager.add_stream(uri) {
            Ok(id) => {
                println!(
                    "[{:.3}] Added stream {}: {} (priority: {:?})",
                    timestamp(),
                    id,
                    uri,
                    priority
                );

                // Set stream priority
                if let Some(coordinator) = get_coordinator(&manager) {
                    coordinator.set_stream_priority(id, *priority).ok();
                }
            }
            Err(e) => {
                println!("[{:.3}] Failed to add stream {}: {}", timestamp(), uri, e);
            }
        }
    }

    // Start pipeline
    pipeline.set_state(gst::State::Playing)?;
    println!(
        "[{:.3}] Pipeline started, processing {} streams",
        timestamp(),
        manager.get_all_stream_states().len()
    );

    // Monitor performance and apply adaptive quality
    let manager_monitor = manager.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));

            // Apply adaptive quality control
            if let Err(e) = manager_monitor.apply_adaptive_quality() {
                eprintln!("Failed to apply adaptive quality: {:?}", e);
            }

            // Get and display metrics
            let states = manager_monitor.get_all_stream_states();
            println!("\n[{:.3}] Stream Status Report:", timestamp());
            for state in states {
                println!(
                    "  {} - Pipeline #{}, FPS: {:.1}, Frames: {}, Detections: {}",
                    state.source_id,
                    state.pipeline_id,
                    state.fps,
                    state.frames_processed,
                    state.detections_count
                );

                if let Some(error) = &state.last_error {
                    println!("    Error: {}", error);
                }
            }

            // Get resource usage
            let stats = manager_monitor.get_stats();
            println!("\n[{:.3}] Resource Usage:", timestamp());
            println!("  CPU: {:.1}%", stats.cpu_usage);
            println!("  Memory: {:.1} MB", stats.memory_usage_mb);
            println!("  Average FPS: {:.1}", stats.average_fps);
        }
    });

    // Simulate dynamic stream management
    let manager_dynamic = manager.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(30));

        println!(
            "\n[{:.3}] Adding additional stream dynamically",
            timestamp()
        );

        // Try to add another stream
        match manager_dynamic.add_stream("rtsp://127.0.0.1:8554/test4") {
            Ok(id) => {
                println!(
                    "[{:.3}] Successfully added dynamic stream: {}",
                    timestamp(),
                    id
                );
            }
            Err(e) => {
                println!("[{:.3}] Failed to add dynamic stream: {}", timestamp(), e);
            }
        }

        // Test stream recovery
        thread::sleep(Duration::from_secs(20));

        let states = manager_dynamic.get_all_stream_states();
        if let Some(state) = states.first() {
            println!(
                "\n[{:.3}] Testing stream recovery for {}",
                timestamp(),
                state.source_id
            );
            if let Err(e) = manager_dynamic.restart_stream(state.source_id) {
                println!("[{:.3}] Failed to restart stream: {}", timestamp(), e);
            } else {
                println!("[{:.3}] Stream restart initiated", timestamp());
            }
        }
    });

    println!(
        "[{:.3}] Multi-stream detection running... Press Ctrl+C to exit",
        timestamp()
    );

    // Keep the application running
    loop {
        thread::sleep(Duration::from_secs(1));

        // Check for resource warnings
        let stats = manager.get_stats();
        if stats.cpu_usage > 90.0 {
            println!(
                "\n[{:.3}] WARNING: High CPU usage detected: {:.1}%",
                timestamp(),
                stats.cpu_usage
            );
        }
        if stats.memory_usage_mb > 1800.0 {
            println!(
                "\n[{:.3}] WARNING: High memory usage detected: {:.1} MB",
                timestamp(),
                stats.memory_usage_mb
            );
        }
    }
}

// Helper function to get coordinator (would be exposed in real implementation)
fn get_coordinator(_manager: &MultiStreamManager) -> Option<Arc<ds_rs::StreamCoordinator>> {
    // In a real implementation, this would access the coordinator from the manager
    None
}
