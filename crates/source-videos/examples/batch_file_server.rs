use source_videos::{
    init, RtspServerBuilder, VideoSourceConfig, VideoSourceType,
    config_types::{FileContainer, Resolution, Framerate, VideoFormat},
    detect_container_format, Result,
};
use std::path::PathBuf;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Initialize GStreamer
    init()?;
    
    // Parse command line arguments (expecting list of video files)
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <video1> [video2] [video3] ...", args[0]);
        eprintln!("Example: {} video1.mp4 video2.avi /path/to/video3.mkv", args[0]);
        std::process::exit(1);
    }
    
    let port = 8554;
    let files: Vec<PathBuf> = args[1..].iter().map(PathBuf::from).collect();
    
    println!("Preparing to serve {} video files", files.len());
    println!("RTSP server port: {}", port);
    
    // Build RTSP server
    let mut server_builder = RtspServerBuilder::new()
        .port(port)
        .address("0.0.0.0");
    
    // Add each file to the server
    for (index, file_path) in files.iter().enumerate() {
        if !file_path.exists() {
            eprintln!("Warning: File does not exist: {}", file_path.display());
            continue;
        }
        
        if !file_path.is_file() {
            eprintln!("Warning: Not a file: {}", file_path.display());
            continue;
        }
        
        // Detect container format
        let container = detect_container_format(&file_path)
            .unwrap_or(FileContainer::Mp4);
        
        // Create a friendly name for the mount point
        let name = format!(
            "file{}_{}",
            index + 1,
            file_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("video")
        );
        
        // Create source configuration
        let config = VideoSourceConfig {
            name: name.clone(),
            source_type: VideoSourceType::File {
                path: file_path.display().to_string(),
                container,
            },
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            framerate: Framerate {
                numerator: 30,
                denominator: 1,
            },
            format: VideoFormat::I420,
            duration: None,
            num_buffers: None,
            is_live: false,
        };
        
        server_builder = server_builder.add_source(config);
        println!("Added: {} -> mount point '/{}'", file_path.display(), name);
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
        println!("What is a 'mount point'?");
        println!("========================================");
        println!("A mount point is the path part of an RTSP URL that identifies a specific video stream.");
        println!("For example, in 'rtsp://localhost:8554/file1_video', the mount point is 'file1_video'.");
        println!("Each video file gets its own unique mount point so you can access it independently.");
        println!();
        println!("View streams with any RTSP-compatible player:");
        println!("  ffplay rtsp://localhost:{}/[mount_point]", port);
        println!("  vlc rtsp://localhost:{}/[mount_point]", port);
        println!("  mpv rtsp://localhost:{}/[mount_point]", port);
        println!();
        println!("Example commands for your files:");
        for mount in sources.iter().take(3) {
            println!("  ffplay rtsp://localhost:{}/{}", port, mount);
        }
    }
    
    println!("\nPress Ctrl+C to stop the server");
    
    // Keep server running until interrupted
    signal::ctrl_c().await?;
    
    println!("\nShutting down server...");
    // Note: RtspServer doesn't have explicit stop method, it stops when dropped
    
    Ok(())
}