//! Example demonstrating network simulation with source-videos
//!
//! This example shows how to use the network simulation features to test
//! error recovery in video streaming applications.

use source_videos::network::{
    NetworkProfile, NetworkSimulator, NetworkController, 
    StandardProfiles, NetworkConditions
};
use source_videos::rtsp::factory::create_test_pattern_with_network;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_rtsp_server as rtsp_server;
use gstreamer_rtsp_server::prelude::*;
use std::time::Duration;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize GStreamer
    gst::init()?;
    
    println!("Network Simulation Example");
    println!("==========================");
    
    // Example 1: Create a standalone network simulator
    demo_standalone_simulator();
    
    // Example 2: Create an RTSP server with network simulation
    demo_rtsp_with_simulation()?;
    
    // Example 3: Test different network profiles
    demo_network_profiles();
    
    // Example 4: Simulate connection drops and recovery
    demo_connection_drops();
    
    Ok(())
}

/// Demonstrate standalone network simulator
fn demo_standalone_simulator() {
    println!("\n1. Standalone Network Simulator");
    println!("--------------------------------");
    
    let simulator = NetworkSimulator::new();
    
    // Enable simulation
    simulator.enable();
    println!("Simulator enabled: {}", simulator.is_enabled());
    
    // Apply poor network conditions
    let poor_conditions = NetworkConditions {
        packet_loss: 5.0,
        latency_ms: 200,
        bandwidth_kbps: 1000,
        connection_dropped: false,
        jitter_ms: 50,
    };
    
    simulator.apply_conditions(poor_conditions.clone());
    println!("Applied conditions: {:?}", simulator.get_conditions());
    
    // Test packet dropping
    let mut dropped = 0;
    let mut passed = 0;
    for _ in 0..1000 {
        if simulator.should_drop_packet() {
            dropped += 1;
        } else {
            passed += 1;
        }
    }
    println!("Packets: {} passed, {} dropped (~{}% loss)", 
             passed, dropped, dropped as f32 / 10.0);
    
    // Get latency delay
    let delay = simulator.get_latency_delay();
    println!("Latency delay: {:?}", delay);
    
    // Reset to perfect conditions
    simulator.reset();
    println!("Reset to perfect conditions");
}

/// Demonstrate RTSP server with network simulation
fn demo_rtsp_with_simulation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. RTSP Server with Network Simulation");
    println!("---------------------------------------");
    
    // Create RTSP server
    let server = rtsp_server::RTSPServer::new();
    server.set_service("8554");
    
    let mounts = server.mount_points().unwrap();
    
    // Create factory with 3G network simulation
    let factory = create_test_pattern_with_network("ball", NetworkProfile::Mobile3G)?;
    println!("Created RTSP factory with 3G network profile");
    
    // Mount the factory
    mounts.add_factory("/test3g", factory);
    
    // Create factory with satellite network simulation
    let factory = create_test_pattern_with_network("smpte", NetworkProfile::Satellite)?;
    println!("Created RTSP factory with Satellite network profile");
    mounts.add_factory("/testsat", factory);
    
    // Create factory with poor network conditions
    let factory = create_test_pattern_with_network("snow", NetworkProfile::Poor)?;
    println!("Created RTSP factory with Poor network profile");
    mounts.add_factory("/testpoor", factory);
    
    // Attach server to main context
    let _id = server.attach(None)?;
    
    println!("\nRTSP streams available at:");
    println!("  rtsp://localhost:8554/test3g   - 3G network simulation");
    println!("  rtsp://localhost:8554/testsat  - Satellite network simulation");
    println!("  rtsp://localhost:8554/testpoor - Poor network simulation");
    
    // Run for a short time in the example
    println!("\nRunning for 5 seconds...");
    std::thread::sleep(Duration::from_secs(5));
    
    // Server will be cleaned up when it goes out of scope
    
    Ok(())
}

/// Demonstrate different network profiles
fn demo_network_profiles() {
    println!("\n3. Network Profiles");
    println!("-------------------");
    
    for profile in NetworkProfile::all() {
        let conditions = profile.into_conditions();
        println!("\n{}: {}", profile, profile.description());
        println!("  Packet Loss: {}%", conditions.packet_loss);
        println!("  Latency: {}ms", conditions.latency_ms);
        println!("  Bandwidth: {} kbps", 
                 if conditions.bandwidth_kbps == 0 { 
                     "unlimited".to_string() 
                 } else { 
                     conditions.bandwidth_kbps.to_string() 
                 });
        println!("  Jitter: {}ms", conditions.jitter_ms);
    }
    
    // Standard profiles for testing
    println!("\nStandard Test Profiles:");
    println!("  Error Recovery: {:?}", StandardProfiles::for_error_recovery());
    println!("  Buffer Test: {:?}", StandardProfiles::for_buffer_test());
    println!("  Latency Test: {:?}", StandardProfiles::for_latency_test());
}

/// Demonstrate connection drops and recovery
fn demo_connection_drops() {
    println!("\n4. Connection Drops and Recovery");
    println!("---------------------------------");
    
    let simulator = Arc::new(NetworkSimulator::new());
    simulator.enable();
    
    // Apply 4G profile
    simulator.apply_profile(NetworkProfile::Mobile4G);
    println!("Applied 4G profile");
    
    // Simulate connection drop
    println!("Dropping connection...");
    simulator.drop_connection();
    assert!(simulator.is_connection_dropped());
    println!("Connection dropped: {}", simulator.is_connection_dropped());
    
    // Simulate some time passing
    std::thread::sleep(Duration::from_millis(500));
    
    // Restore connection
    println!("Restoring connection...");
    simulator.restore_connection();
    assert!(!simulator.is_connection_dropped());
    println!("Connection restored: {}", !simulator.is_connection_dropped());
    
    // Demonstrate periodic drops
    println!("\nSimulating periodic connection drops...");
    let sim_clone = Arc::clone(&simulator);
    std::thread::spawn(move || {
        for i in 0..3 {
            std::thread::sleep(Duration::from_secs(2));
            println!("  Drop #{}", i + 1);
            sim_clone.drop_connection();
            std::thread::sleep(Duration::from_millis(500));
            sim_clone.restore_connection();
            println!("  Restored #{}", i + 1);
        }
    });
    
    // Wait for periodic drops to complete
    std::thread::sleep(Duration::from_secs(7));
    
    println!("\nConnection simulation complete");
}