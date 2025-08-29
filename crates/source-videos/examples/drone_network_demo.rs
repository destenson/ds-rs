use source_videos::network::NetworkProfile;
use source_videos::{Result, RtspServerBuilder, init};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    init()?;

    println!("Drone Network Simulation Demo");
    println!("==============================");
    println!();
    println!("Simulating various drone communication scenarios");
    println!();

    // Build server with different drone network conditions
    let server = RtspServerBuilder::new()
        .port(8554)
        // Control - perfect conditions
        .add_test_pattern("control", "smpte")
        // Urban drone with building interference
        .add_test_pattern_with_network("drone-urban", "ball", NetworkProfile::DroneUrban)
        // Mountain drone with terrain masking
        .add_test_pattern_with_network("drone-mountain", "circular", NetworkProfile::DroneMountain)
        // Noisy radio link for comparison
        .add_test_pattern_with_network("noisy-radio", "snow", NetworkProfile::NoisyRadio)
        .build()?;

    server.start()?;

    println!("RTSP streams simulating drone communications:");
    println!();
    println!("1. Control (perfect conditions):");
    println!("   rtsp://localhost:8554/control");
    println!("   - No packet loss, no latency");
    println!();
    println!("2. Urban Drone (UHF/VHF through buildings):");
    println!("   rtsp://localhost:8554/drone-urban");
    println!("   - Packet loss: 20% (building obstruction)");
    println!("   - Latency: 40ms");
    println!("   - Bandwidth: 800 kbps");
    println!("   - High jitter: 120ms (multipath reflections)");
    println!();
    println!("3. Mountain Drone (open/mountain terrain):");
    println!("   rtsp://localhost:8554/drone-mountain");
    println!("   - Packet loss: 5% (occasional terrain masking)");
    println!("   - Latency: 60ms (distance effects)");
    println!("   - Bandwidth: 1.5 Mbps");
    println!("   - Low jitter: 30ms (stable when clear)");
    println!();
    println!("4. Noisy Radio (for comparison):");
    println!("   rtsp://localhost:8554/noisy-radio");
    println!("   - Packet loss: 15%");
    println!("   - Latency: 80ms");
    println!("   - Bandwidth: 1 Mbps");
    println!();
    println!("Test with VLC or GStreamer:");
    println!("  vlc rtsp://localhost:8554/control");
    println!("  vlc rtsp://localhost:8554/drone-urban");
    println!("  vlc rtsp://localhost:8554/drone-mountain");
    println!();
    println!("Compare how different environments affect drone video transmission!");
    println!();
    println!("Tips:");
    println!("- Urban drone simulates multipath and building obstruction");
    println!("- Mountain drone simulates distance and terrain masking");
    println!("- Watch for video artifacts and buffering differences");
    println!();
    println!("Press Ctrl+C to stop...");

    signal::ctrl_c().await?;
    println!("\nStopping server...");

    Ok(())
}
