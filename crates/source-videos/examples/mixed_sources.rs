use source_videos::{
    init, RtspServerBuilder, DirectoryConfig, FilterConfig, DirectoryScanner,
    VideoSourceConfig, TestPattern, Result,
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal;

/// Example demonstrating mixed video sources:
/// - Test patterns
/// - Files from directory
/// - Explicit file list
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Initialize GStreamer
    init()?;
    
    let port = 8554;
    println!("Starting mixed source RTSP server on port {}", port);
    
    // Start building the server
    let mut server_builder = RtspServerBuilder::new()
        .port(port)
        .address("0.0.0.0");
    
    // Add test patterns
    println!("\nAdding test patterns:");
    server_builder = server_builder
        .add_test_pattern("pattern_smpte", "smpte")
        .add_test_pattern("pattern_ball", "ball")
        .add_test_pattern("pattern_snow", "snow");
    println!("  - SMPTE color bars");
    println!("  - Bouncing ball");
    println!("  - Snow/noise");
    
    // Try to add files from current directory
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dir_config = DirectoryConfig {
        path: current_dir.display().to_string(),
        recursive: false,
        filters: Some(FilterConfig {
            include: vec![],
            exclude: vec![],
            extensions: vec!["mp4".to_string(), "avi".to_string(), "mkv".to_string()],
        }),
        lazy_loading: false,
        mount_prefix: Some("local".to_string()),
    };
    
    println!("\nScanning current directory for video files...");
    let mut scanner = DirectoryScanner::new(dir_config);
    if let Ok(source_configs) = scanner.scan() {
        if !source_configs.is_empty() {
            println!("Found {} video files:", source_configs.len());
            for config in source_configs {
                println!("  - {}", config.name);
                server_builder = server_builder.add_source(config);
            }
        } else {
            println!("No video files found in current directory");
        }
    }
    
    // Add some example files if they exist
    let example_files = vec![
        "/tmp/sample.mp4",
        "C:/Videos/example.avi",
        "~/Movies/test.mkv",
    ];
    
    println!("\nChecking for example files:");
    for file_path in example_files {
        let path = PathBuf::from(file_path);
        if path.exists() && path.is_file() {
            let name = format!(
                "example_{}",
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("video")
            );
            
            let config = VideoSourceConfig::file(name.clone(), file_path);
            server_builder = server_builder.add_source(config);
            println!("  - Added: {}", file_path);
        }
    }
    
    // Build and start the server
    let mut server = server_builder.build()?;
    server.start()?;
    
    // List all available streams with detailed explanations
    println!("\n========================================");
    println!("RTSP STREAMS & MOUNT POINTS EXPLAINED");
    println!("========================================");
    
    let sources = server.list_sources();
    if sources.is_empty() {
        println!("No sources available");
    } else {
        println!("This server provides {} different video streams:", sources.len());
        println!();
        
        for (i, mount) in sources.iter().enumerate() {
            let url = format!("rtsp://localhost:{}/{}", port, mount);
            let source_type = if mount.starts_with("pattern_") {
                "Test Pattern"
            } else if mount.starts_with("local/") {
                "Local Video File"
            } else if mount.starts_with("example_") {
                "Example File"
            } else {
                "Video Stream"
            };
            
            println!("  {}. {} - {}", i + 1, source_type, url);
        }
        
        println!("\n========================================");
        println!("WHAT ARE MOUNT POINTS?");
        println!("========================================");
        println!("Mount points are unique identifiers for each video stream on the RTSP server.");
        println!("Think of them like TV channels - each mount point gives you access to a different video.");
        println!();
        println!("In the URL 'rtsp://localhost:8554/pattern_smpte':");
        println!("  - 'rtsp://' is the protocol");
        println!("  - 'localhost:8554' is the server address and port");
        println!("  - 'pattern_smpte' is the mount point (the specific video stream)");
        println!();
        println!("========================================");
        println!("HOW TO WATCH THE STREAMS:");
        println!("========================================");
        println!("Copy any URL above and use it with these players:");
        println!();
        println!("FFplay (command line):");
        println!("  ffplay rtsp://localhost:{}/[mount_point]", port);
        println!();
        println!("VLC Media Player:");
        println!("  1. Open VLC");
        println!("  2. File -> Open Network Stream");
        println!("  3. Paste the RTSP URL");
        println!();
        println!("GStreamer:");
        println!("  gst-launch-1.0 rtspsrc location=rtsp://localhost:{}/[mount_point] ! decodebin ! autovideosink", port);
        println!();
        println!("Example commands for your streams:");
        for (i, mount) in sources.iter().take(3).enumerate() {
            println!("  ffplay rtsp://localhost:{}/{}", port, mount);
        }
        println!("========================================");
    }
    
    println!("\nServer is running. Press Ctrl+C to stop...");
    
    // Keep server running until interrupted
    signal::ctrl_c().await?;
    
    println!("\nShutting down server...");
    // Note: RtspServer stops automatically when dropped
    
    println!("Server stopped successfully");
    Ok(())
}