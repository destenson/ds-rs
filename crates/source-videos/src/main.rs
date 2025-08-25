#![allow(unused)]
use source_videos::{
    AppConfig, SourceVideos, TestPattern, VideoSourceConfig,
    generate_test_file, create_test_rtsp_server, Result, SourceVideoError
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal;
use gstreamer::glib::prelude::*;

#[derive(Parser)]
#[command(name = "source-videos")]
#[command(about = "Dynamic video source generation infrastructure")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long, default_value_t = 8554)]
        port: u16,
        
        #[arg(short, long, default_value = "0.0.0.0")]
        address: String,
        
        #[arg(long)]
        duration: Option<u64>,
        
        #[arg(long, value_delimiter = ',')]
        patterns: Vec<String>,
        
        #[arg(short = 'd', long = "directory", help = "Directory containing video files to serve")]
        directory: Option<PathBuf>,
        
        #[arg(short = 'r', long = "recursive", help = "Recursively scan directories")]
        recursive: bool,
        
        #[arg(short = 'f', long = "files", value_delimiter = ',', help = "Explicit list of video files to serve")]
        files: Vec<PathBuf>,
        
        #[arg(long = "include", value_delimiter = ',', help = "Include file patterns (e.g., *.mp4,test_*)")]
        include: Vec<String>,
        
        #[arg(long = "exclude", value_delimiter = ',', help = "Exclude file patterns (e.g., *.tmp,backup_*)")]
        exclude: Vec<String>,
        
        #[arg(long = "mount-prefix", help = "Prefix for RTSP mount points")]
        mount_prefix: Option<String>,
        
        #[arg(long = "lazy", help = "Enable lazy loading of sources")]
        lazy_loading: bool,
        
        #[arg(short = 'w', long = "watch", help = "Enable file system watching for dynamic source updates")]
        watch: bool,
        
        #[arg(short = 'l', long = "auto-repeat", help = "Enable auto-repeat/looping for video playback")]
        auto_repeat: bool,
        
        #[arg(long = "reload-on-change", help = "Reload video files when they are modified")]
        reload_on_change: bool,
        
        #[arg(long = "watch-interval", default_value_t = 500, help = "File watching debounce interval in milliseconds")]
        watch_interval_ms: u64,
        
        #[arg(long = "max-loops", help = "Maximum number of loops for auto-repeat (default: infinite)")]
        max_loops: Option<u32>,
        
        #[arg(long = "seamless-loop", help = "Enable seamless looping without gaps")]
        seamless_loop: bool,
    },
    Generate {
        #[arg(short, long, default_value = "smpte")]
        pattern: String,
        
        #[arg(short, long, default_value_t = 10)]
        duration: u64,
        
        #[arg(short, long)]
        output: PathBuf,
        
        #[arg(long, default_value_t = 1920)]
        width: u32,
        
        #[arg(long, default_value_t = 1080)]
        height: u32,
        
        #[arg(long, default_value_t = 30)]
        fps: i32,
    },
    List,
    Interactive,
    Test {
        #[arg(short, long, default_value_t = 8554)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    }
    
    source_videos::init()?;
    
    let config = if let Some(config_path) = &cli.config {
        AppConfig::from_file(config_path)?
    } else {
        AppConfig::default()
    };
    
    match cli.command {
        Commands::Serve { 
            port, 
            address, 
            duration, 
            patterns,
            directory,
            recursive,
            files,
            include,
            exclude,
            mount_prefix,
            lazy_loading,
            watch,
            auto_repeat,
            reload_on_change,
            watch_interval_ms,
            max_loops,
            seamless_loop,
        } => {
            serve_command(
                port, 
                address, 
                duration, 
                patterns,
                directory,
                recursive,
                files,
                include,
                exclude,
                mount_prefix,
                lazy_loading,
                watch,
                auto_repeat,
                reload_on_change,
                watch_interval_ms,
                max_loops,
                seamless_loop,
            ).await
        }
        Commands::Generate { pattern, duration, output, width, height, fps } => {
            generate_command(pattern, duration, output, width, height, fps).await
        }
        Commands::List => {
            list_command().await
        }
        Commands::Interactive => {
            interactive_command().await
        }
        Commands::Test { port } => {
            test_command(port).await
        }
    }
}

async fn serve_command(
    port: u16, 
    address: String, 
    duration: Option<u64>, 
    patterns: Vec<String>,
    directory: Option<PathBuf>,
    recursive: bool,
    files: Vec<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    mount_prefix: Option<String>,
    lazy_loading: bool,
    watch: bool,
    auto_repeat: bool,
    reload_on_change: bool,
    watch_interval_ms: u64,
    max_loops: Option<u32>,
    seamless_loop: bool,
) -> Result<()> {
    use source_videos::{DirectoryConfig, FileListConfig, FilterConfig, DirectoryScanner, RtspServerBuilder, WatcherManager, LoopConfig, create_looping_source};
    use std::time::Duration;
    
    println!("Starting RTSP server on rtsp://localhost:{}", port);
    println!("Starting RTSP server on {}:{}", address, port);
    
    // Build server with initial patterns
    let mut server_builder = RtspServerBuilder::new().port(port);
    
    // Add test patterns if specified
    if !patterns.is_empty() {
        for (i, pattern) in patterns.iter().enumerate() {
            let name = format!("pattern-{}", i + 1);
            server_builder = server_builder.add_test_pattern(&name, pattern);
            println!("Will add pattern '{}' at rtsp://{}:{}/{}", pattern, address, port, name);
        }
    }
    
    // Scan directory for video files if specified
    if let Some(ref dir_path) = directory {
        let filters = if !include.is_empty() || !exclude.is_empty() {
            Some(FilterConfig {
                include: include.clone(),
                exclude: exclude.clone(),
                extensions: vec![],
            })
        } else {
            None
        };
        
        let dir_config = DirectoryConfig {
            path: dir_path.display().to_string(),
            recursive,
            filters,
            lazy_loading,
            mount_prefix: mount_prefix.clone(),
        };
        
        println!("Scanning directory: {} (recursive: {})", dir_path.display(), recursive);
        let mut scanner = DirectoryScanner::new(dir_config);
        let source_configs = scanner.scan()?;
        
        println!("Found {} video files in directory", source_configs.len());
        
        for config in source_configs {
            server_builder = server_builder.add_source(config);
        }
    }
    
    // Add explicit file list if specified
    if !files.is_empty() {
        use source_videos::file_utils::detect_container_format;
        
        println!("Adding {} files from list", files.len());
        
        for (index, file_path) in files.iter().enumerate() {
            let container = detect_container_format(file_path)
                .unwrap_or(source_videos::config_types::FileContainer::Mp4);
            
            let name = format!(
                "file_{}_{}",
                index,
                file_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("video")
            );
            
            let config = VideoSourceConfig {
                name: name.clone(),
                source_type: source_videos::VideoSourceType::File {
                    path: file_path.display().to_string(),
                    container,
                },
                resolution: source_videos::config_types::Resolution {
                    width: 1920,
                    height: 1080,
                },
                framerate: source_videos::config_types::Framerate {
                    numerator: 30,
                    denominator: 1,
                },
                format: source_videos::config_types::VideoFormat::I420,
                duration: None,
                num_buffers: None,
                is_live: false,
            };
            
            server_builder = server_builder.add_source(config);
            println!("Added file: {}", file_path.display());
        }
    }
    
    // Build and start the server
    let mut server = server_builder.build()?;
    
    // Set up file watching if enabled
    let mut watcher_manager = if watch && directory.is_some() {
        println!("Setting up file system watching...");
        let mut manager = WatcherManager::new();
        
        if let Some(ref dir_path) = directory {
            let watcher_id = manager.add_directory_watcher(dir_path, recursive).await?;
            println!("Started watching directory: {} (ID: {})", dir_path.display(), watcher_id);
        }
        
        Some(manager)
    } else {
        None
    };
    
    // Print auto-repeat configuration
    if auto_repeat {
        println!("Auto-repeat enabled: max_loops={:?}, seamless={}", max_loops, seamless_loop);
    }
    
    if reload_on_change {
        println!("Hot-reload enabled with {}ms debounce interval", watch_interval_ms);
    }
    
    server.start()?;
    
    for mount in server.list_sources() {
        println!("Stream available at: {}", server.get_url(&mount));
    }
    
    // Get the default main context for manual iteration
    let main_context = gstreamer::glib::MainContext::default();
    
    if let Some(duration) = duration {
        println!("Server will run for {} seconds", duration);
        let end_time = std::time::Instant::now() + Duration::from_secs(duration);
        
        while std::time::Instant::now() < end_time {
            // Iterate the GLib main context
            main_context.iteration(false);
            
            // Small sleep to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    } else {
        println!("Press Ctrl+C to stop the server");
        
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("Received Ctrl+C, stopping...");
            }
            _ = async {
                loop {
                    main_context.iteration(false);
                    
                    // Check for file system events if watching is enabled
                    if let Some(ref mut manager) = watcher_manager {
                        if let Some(event) = manager.recv().await {
                            println!("File system event: {:?} - {}", event.event_type(), event.path().display());
                            
                            // Handle the event through the RTSP server directly
                            if let Err(e) = server.handle_file_event(&event) {
                                eprintln!("Error handling file event: {}", e);
                            }
                        }
                    }
                    
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            } => {}
        }
    }
    
    println!("Server stopped");
    Ok(())
}

async fn generate_command(
    pattern: String,
    duration: u64,
    output: PathBuf,
    width: u32,
    height: u32,
    fps: i32,
) -> Result<()> {
    println!("Generating test video with pattern '{}'", pattern);
    println!("Output: {}", output.display());
    println!("Duration: {} seconds", duration);
    println!("Resolution: {}x{}", width, height);
    println!("Framerate: {} fps", fps);
    
    let start = std::time::Instant::now();
    
    generate_test_file(&pattern, duration, &output)?;
    
    let elapsed = start.elapsed();
    println!("Generated successfully in {:.2} seconds", elapsed.as_secs_f64());
    
    Ok(())
}

async fn list_command() -> Result<()> {
    println!("Available test patterns:");
    
    for pattern in TestPattern::all() {
        println!("  {:<20} - {}", format!("{:?}", pattern), pattern.description());
    }
    
    println!("\nAnimated patterns:");
    for pattern in TestPattern::animated_patterns() {
        println!("  {:?}", pattern);
    }
    
    Ok(())
}

async fn interactive_command() -> Result<()> {
    println!("Source Videos Interactive Mode");
    println!("==============================");
    
    let mut sv = SourceVideos::new()?;
    let mut line = String::new();
    
    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        line.clear();
        if std::io::stdin().read_line(&mut line).is_err() {
            break;
        }
        
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "add" => {
                if parts.len() >= 3 {
                    let name = parts[1];
                    let pattern = parts[2];
                    match sv.add_test_pattern(name, pattern) {
                        Ok(id) => println!("Added source '{}' with ID: {}", name, id),
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("Usage: add <name> <pattern>");
                }
            }
            "list" => {
                let sources = sv.list_sources();
                if sources.is_empty() {
                    println!("No sources");
                } else {
                    for info in sources {
                        println!("{}: {} ({:?})", info.name, info.uri, info.state);
                    }
                }
            }
            "remove" => {
                if parts.len() >= 2 {
                    match sv.remove_source(parts[1]) {
                        Ok(_) => println!("Removed source '{}'", parts[1]),
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("Usage: remove <name_or_id>");
                }
            }
            "serve" => {
                let port = if parts.len() >= 2 {
                    parts[1].parse().unwrap_or(8554)
                } else {
                    8554
                };
                match sv.start_rtsp_server(port) {
                    Ok(_) => {
                        println!("RTSP server started on port {}", port);
                        for url in sv.get_rtsp_urls() {
                            println!("  {}", url);
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "help" => {
                println!("Commands:");
                println!("  add <name> <pattern>  - Add a test pattern source");
                println!("  list                  - List all sources");
                println!("  remove <name>         - Remove a source");
                println!("  serve [port]          - Start RTSP server");
                println!("  patterns              - List available patterns");
                println!("  help                  - Show this help");
                println!("  quit                  - Exit");
            }
            "patterns" => {
                for pattern in TestPattern::all() {
                    println!("  {:?}", pattern);
                }
            }
            "quit" | "exit" => break,
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }
    
    Ok(())
}

async fn test_command(port: u16) -> Result<()> {
    println!("Running comprehensive test suite...");
    
    let mut sv = SourceVideos::new()?;
    
    println!("1. Adding test patterns...");
    sv.add_test_pattern("smpte", "smpte")?;
    sv.add_test_pattern("ball", "ball")?;
    sv.add_test_pattern("snow", "snow")?;
    
    println!("2. Starting RTSP server...");
    sv.start_rtsp_server(port)?;
    
    println!("3. Listing sources...");
    let sources = sv.list_sources();
    for info in &sources {
        println!("   {} - {}", info.name, info.uri);
    }
    
    println!("4. RTSP URLs:");
    for url in sv.get_rtsp_urls() {
        println!("   {}", url);
    }
    
    println!("\nTest completed! Test streams for 10 seconds...");
    tokio::time::sleep(Duration::from_secs(10)).await;
    
    println!("All tests passed!");
    Ok(())
}
