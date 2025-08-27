#![allow(unused)]

use source_videos::{RtspServerBuilder, init, Result};
use source_videos::network::{NetworkProfile, NetworkScenario, NetworkConditions};
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    init()?;
    
    println!("Network Simulation Demo");
    println!("========================");
    println!();
    
    // Build server with different network conditions per source
    let server = RtspServerBuilder::new()
        .port(8554)
        // Perfect network for control source
        .add_test_pattern("control", "smpte")
        // 3G mobile network simulation
        .add_test_pattern_with_network("mobile-3g", "ball", NetworkProfile::Mobile3G)
        // Noisy radio link
        .add_test_pattern_with_network("radio", "snow", NetworkProfile::NoisyRadio)
        // Intermittent satellite
        .add_test_pattern_with_network("satellite", "circular", NetworkProfile::IntermittentSatellite)
        // Poor network conditions
        .add_test_pattern_with_network("poor", "gamut", NetworkProfile::Poor)
        .build()?;
    
    server.start()?;
    
    println!("RTSP streams with network simulation:");
    println!();
    println!("1. Control (perfect network):");
    println!("   rtsp://localhost:8554/control");
    println!();
    println!("2. Mobile 3G simulation:");
    println!("   rtsp://localhost:8554/mobile-3g");
    println!("   - Packet loss: 2%");
    println!("   - Latency: 150ms");
    println!("   - Bandwidth: 2 Mbps");
    println!();
    println!("3. Noisy radio link:");
    println!("   rtsp://localhost:8554/radio");
    println!("   - Packet loss: 15% (interference)");
    println!("   - Latency: 80ms");
    println!("   - Bandwidth: 1 Mbps");
    println!("   - High jitter: 150ms");
    println!();
    println!("4. Intermittent satellite:");
    println!("   rtsp://localhost:8554/satellite");
    println!("   - Packet loss: 3% (when connected)");
    println!("   - Latency: 750ms (very high)");
    println!("   - Bandwidth: 5 Mbps");
    println!("   - Note: Will experience periodic disconnections");
    println!();
    println!("5. Poor network:");
    println!("   rtsp://localhost:8554/poor");
    println!("   - Packet loss: 10%");
    println!("   - Latency: 500ms");
    println!("   - Bandwidth: 500 kbps");
    println!();
    
    println!("Test with VLC or GStreamer:");
    println!("  vlc rtsp://localhost:8554/control");
    println!("  vlc rtsp://localhost:8554/radio");
    println!("  vlc rtsp://localhost:8554/satellite");
    println!();
    println!("Compare the stream quality between perfect and poor network conditions!");
    println!();
    println!("Press Ctrl+C to stop...");
    
    signal::ctrl_c().await?;
    println!("\nStopping server...");
    
    Ok(())
}
