#![allow(unused)]

use source_videos::{
    DirectoryConfig, DirectoryScanner, FilterConfig, Result, RtspServerBuilder, init,
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Initialize GStreamer
    init()?;

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let directory = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let port = args
        .get(2)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8554);

    println!("Serving video files from: {}", directory.display());
    println!("RTSP server port: {}", port);

    // Configure directory scanning
    let dir_config = DirectoryConfig {
        path: directory.display().to_string(),
        recursive: true,
        filters: Some(FilterConfig {
            include: vec![],
            exclude: vec!["*.tmp".to_string(), ".*".to_string()], // Exclude temp and hidden files
            extensions: vec![
                "mp4".to_string(),
                "avi".to_string(),
                "mkv".to_string(),
                "webm".to_string(),
            ],
        }),
        lazy_loading: false,
        mount_prefix: Some("videos".to_string()),
    };

    // Scan directory for video files
    println!("Scanning directory for video files...");
    let mut scanner = DirectoryScanner::new(dir_config);
    let source_configs = scanner.scan()?;

    if source_configs.is_empty() {
        println!("No video files found in directory");
        return Ok(());
    }

    println!("Found {} video files:", source_configs.len());

    // Build RTSP server with discovered files
    let mut server_builder = RtspServerBuilder::new().port(port).address("0.0.0.0");

    for config in &source_configs {
        println!("  - {}", config.name);
        server_builder = server_builder.add_source(config.clone());
    }

    // Build and start server
    let mut server = server_builder.build()?;
    server.start()?;

    println!("\n========================================");
    println!("RTSP streams available:");
    println!("========================================");

    let sources = server.list_sources();
    for mount in &sources {
        println!("  rtsp://localhost:{}/{}", port, mount);
    }

    if !sources.is_empty() {
        println!("\n========================================");
        println!("Understanding RTSP Mount Points:");
        println!("========================================");
        println!(
            "Each video file in your directory becomes an RTSP stream with a unique 'mount point'."
        );
        println!(
            "The mount point is derived from the file's path relative to the scanned directory."
        );
        println!();
        println!("For example:");
        println!("  File: /videos/movies/action.mp4");
        println!("  Mount: rtsp://localhost:{}/videos/movies/action", port);
        println!();
        println!("View any stream with RTSP-compatible players:");
        println!("  ffplay rtsp://localhost:{}/[mount_point]", port);
        println!("  vlc rtsp://localhost:{}/[mount_point]", port);
        println!(
            "  gst-launch-1.0 rtspsrc location=rtsp://localhost:{}/[mount_point] ! decodebin ! autovideosink",
            port
        );
        println!();
        println!("Try these examples:");
        for (i, mount) in sources.iter().take(3).enumerate() {
            println!("  {}. ffplay rtsp://localhost:{}/{}", i + 1, port, mount);
        }
    } else {
        println!("No video files found to serve.");
    }

    println!("\nPress Ctrl+C to stop the server");

    // Keep server running until interrupted
    signal::ctrl_c().await?;

    println!("\nShutting down server...");
    // Note: RtspServer stops automatically when dropped

    Ok(())
}
