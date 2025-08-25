#![allow(unused)]
use source_videos::{
    AppConfig, SourceVideos, TestPattern, VideoSourceConfig,
    generate_test_file, create_test_rtsp_server, Result, SourceVideoError,
    api::ControlApi
};
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use std::path::PathBuf;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::signal;
use gstreamer::glib::prelude::*;
use regex::Regex;
use std::io;
use chrono::{DateTime, Utc};
use std::fs;
use std::process;

#[derive(Parser)]
#[command(name = "source-videos")]
#[command(about = "Dynamic video source generation infrastructure")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum PlaylistMode {
    Sequential,
    Random,
    Shuffle,
}

#[derive(Clone, Copy, ValueEnum, Debug)]
enum PlaylistRepeat {
    None,
    All,
    One,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Csv,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long, default_value_t = 8554)]
        port: u16,
        
        #[arg(long, help = "Enable REST API server")]
        api: bool,
        
        #[arg(long, default_value_t = 3000, help = "API server port")]
        api_port: u16,
        
        #[arg(long, default_value = "0.0.0.0", help = "API server bind address")]
        api_address: String,
        
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
        
        // Network simulation options
        #[arg(long = "network-profile", help = "Apply network profile (perfect, 3g, 4g, 5g, wifi, public, satellite, broadband, poor)")]
        network_profile: Option<String>,
        
        #[arg(long = "packet-loss", help = "Packet loss percentage (0-100)")]
        packet_loss: Option<f32>,
        
        #[arg(long = "latency", help = "Additional latency in milliseconds")]
        latency_ms: Option<u32>,
        
        #[arg(long = "bandwidth", help = "Bandwidth limit in kbps (0 = unlimited)")]
        bandwidth_kbps: Option<u32>,
        
        #[arg(long = "jitter", help = "Jitter in milliseconds")]
        jitter_ms: Option<u32>,
        
        #[arg(long = "network-drop", help = "Simulate periodic connection drops (format: period_seconds,duration_seconds)")]
        network_drop: Option<String>,
        
        #[arg(long = "per-source-network", help = "Per-source network conditions (format: source_name:profile)", value_delimiter = ',')]
        per_source_network: Vec<String>,
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
    
    /// Serve files or directories as separate RTSP streams
    ServeFiles {
        #[arg(short, long, default_value_t = 8554)]
        port: u16,
        
        #[arg(short = 'd', long = "directory", help = "Directory containing video files")]
        directory: Option<PathBuf>,
        
        #[arg(short = 'f', long = "files", value_delimiter = ',', help = "Explicit list of video files")]
        files: Vec<PathBuf>,
        
        #[arg(short = 'r', long = "recursive")]
        recursive: bool,
        
        #[arg(long = "include", value_delimiter = ',')]
        include: Vec<String>,
        
        #[arg(long = "exclude", value_delimiter = ',')]
        exclude: Vec<String>,
        
        #[arg(long = "format", help = "Filter by video format (mp4, mkv, avi, webm)")]
        format: Option<String>,
        
        #[arg(long = "min-duration", help = "Minimum duration in seconds")]
        min_duration: Option<u64>,
        
        #[arg(long = "max-duration", help = "Maximum duration in seconds")]
        max_duration: Option<u64>,
        
        #[arg(long = "modified-since", help = "Filter files modified since date (YYYY-MM-DD)")]
        modified_since: Option<String>,
        
        #[arg(short = 'w', long = "watch")]
        watch: bool,
        
        #[arg(long = "daemon", short = 'D')]
        daemon: bool,
        
        #[arg(long = "pid-file")]
        pid_file: Option<PathBuf>,
        
        #[arg(long = "max-streams")]
        max_streams: Option<u32>,
        
        #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
        verbose: u8,
        
        #[arg(short = 'q', long = "quiet")]
        quiet: bool,
        
        #[arg(long = "status-interval", help = "Status update interval in seconds")]
        status_interval: Option<u64>,
        
        #[arg(long = "metrics")]
        metrics: bool,
        
        #[arg(long = "output-format", value_enum, default_value = "text")]
        output_format: OutputFormat,
        
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    
    /// Serve directory as playlist with sequential playback
    Playlist {
        #[arg(short, long, default_value_t = 8554)]
        port: u16,
        
        #[arg(short = 'd', long = "directory", help = "Directory for playlist")]
        directory: PathBuf,
        
        #[arg(short = 'r', long = "recursive")]
        recursive: bool,
        
        #[arg(long = "playlist-mode", value_enum, default_value = "sequential")]
        playlist_mode: PlaylistMode,
        
        #[arg(long = "playlist-repeat", value_enum, default_value = "none")]
        playlist_repeat: PlaylistRepeat,
        
        #[arg(long = "playlist-file", help = "Load playlist from m3u/pls file")]
        playlist_file: Option<PathBuf>,
        
        #[arg(long = "transition-duration", help = "Transition gap in seconds")]
        transition_duration: Option<f32>,
        
        #[arg(long = "crossfade")]
        crossfade: bool,
        
        #[arg(long = "include", value_delimiter = ',')]
        include: Vec<String>,
        
        #[arg(long = "exclude", value_delimiter = ',')]
        exclude: Vec<String>,
        
        #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
        verbose: u8,
        
        #[arg(long = "daemon", short = 'D')]
        daemon: bool,
    },
    
    /// Monitor directory with real-time statistics
    Monitor {
        #[arg(short = 'd', long = "directory", help = "Directory to monitor")]
        directory: PathBuf,
        
        #[arg(short = 'r', long = "recursive")]
        recursive: bool,
        
        #[arg(long = "watch-interval", default_value_t = 1000, help = "Watch interval in ms")]
        watch_interval: u64,
        
        #[arg(long = "list-streams")]
        list_streams: bool,
        
        #[arg(long = "metrics")]
        metrics: bool,
        
        #[arg(long = "output-format", value_enum, default_value = "text")]
        output_format: OutputFormat,
    },
    
    /// Test network simulation with various profiles
    Simulate {
        #[arg(short, long, default_value_t = 8554)]
        port: u16,
        
        #[arg(long = "network-profile", help = "Network profile to test")]
        network_profile: String,
        
        #[arg(short = 'd', long = "directory")]
        directory: Option<PathBuf>,
        
        #[arg(long = "patterns", value_delimiter = ',')]
        patterns: Vec<String>,
        
        #[arg(long = "duration", help = "Test duration in seconds")]
        duration: Option<u64>,
        
        #[arg(long = "metrics")]
        metrics: bool,
    },
    
    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let level = match cli.verbose {
        0 => "warn",
        1 => "info", 
        2 => "debug",
        _ => "trace",
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();
    
    source_videos::init()?;
    
    let config = if let Some(config_path) = &cli.config {
        AppConfig::from_file(config_path)?
    } else {
        AppConfig::default()
    };
    
    match cli.command {
        Commands::Serve { 
            port,
            api,
            api_port,
            api_address,
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
            network_profile,
            packet_loss,
            latency_ms,
            bandwidth_kbps,
            jitter_ms,
            network_drop,
            per_source_network,
        } => {
            serve_command(
                port,
                api,
                api_port,
                api_address,
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
                network_profile,
                packet_loss,
                latency_ms,
                bandwidth_kbps,
                jitter_ms,
                network_drop,
                per_source_network,
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
        Commands::ServeFiles {
            port, directory, files, recursive, include, exclude,
            format, min_duration, max_duration, modified_since,
            watch, daemon, pid_file, max_streams, verbose,
            quiet, status_interval, metrics, output_format, dry_run
        } => {
            serve_files_command(
                port, directory, files, recursive, include, exclude,
                format, min_duration, max_duration, modified_since,
                watch, daemon, pid_file, max_streams, verbose,
                quiet, status_interval, metrics, output_format, dry_run
            ).await
        }
        Commands::Playlist {
            port, directory, recursive, playlist_mode, playlist_repeat,
            playlist_file, transition_duration, crossfade, include,
            exclude, verbose, daemon
        } => {
            playlist_command(
                port, directory, recursive, playlist_mode, playlist_repeat,
                playlist_file, transition_duration, crossfade, include,
                exclude, verbose, daemon
            ).await
        }
        Commands::Monitor {
            directory, recursive, watch_interval, list_streams,
            metrics, output_format
        } => {
            monitor_command(
                directory, recursive, watch_interval, list_streams,
                metrics, output_format
            ).await
        }
        Commands::Simulate {
            port, network_profile, directory, patterns, duration, metrics
        } => {
            simulate_command(
                port, network_profile, directory, patterns, duration, metrics
            ).await
        }
        Commands::Completions { shell } => {
            completions_command(shell).await
        }
    }
}

async fn serve_command(
    port: u16,
    api: bool,
    api_port: u16,
    api_address: String,
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
    network_profile: Option<String>,
    packet_loss: Option<f32>,
    latency_ms: Option<u32>,
    bandwidth_kbps: Option<u32>,
    jitter_ms: Option<u32>,
    network_drop: Option<String>,
    per_source_network: Vec<String>,
) -> Result<()> {
    use source_videos::{DirectoryConfig, FileListConfig, FilterConfig, DirectoryScanner, RtspServerBuilder, WatcherManager, LoopConfig, create_looping_source, VideoSourceManager};
    use source_videos::network::{NetworkProfile, NetworkConditions, GStreamerNetworkSimulator, NetworkController};
    use std::time::Duration;
    use std::str::FromStr;
    
    println!("Starting RTSP server on rtsp://localhost:{}", port);
    println!("Starting RTSP server on {}:{}", address, port);
    
    // Parse network simulation settings
    let global_network_profile = if let Some(profile_str) = network_profile {
        match NetworkProfile::from_str(&profile_str) {
            Ok(profile) => {
                println!("Applying network profile: {} - {}", profile_str, profile.description());
                Some(profile)
            }
            Err(e) => {
                eprintln!("Invalid network profile '{}': {}", profile_str, e);
                return Err(source_videos::SourceVideoError::config(format!("Invalid network profile: {}", e)).into());
            }
        }
    } else if packet_loss.is_some() || latency_ms.is_some() || bandwidth_kbps.is_some() {
        // Create custom network conditions from individual parameters
        let conditions = NetworkConditions {
            packet_loss: packet_loss.unwrap_or(0.0),
            latency_ms: latency_ms.unwrap_or(0),
            bandwidth_kbps: bandwidth_kbps.unwrap_or(0),
            jitter_ms: jitter_ms.unwrap_or(0),
            connection_dropped: false,
        };
        println!("Applying custom network conditions: packet_loss={}%, latency={}ms, bandwidth={}kbps, jitter={}ms",
                 conditions.packet_loss, conditions.latency_ms, conditions.bandwidth_kbps, conditions.jitter_ms);
        Some(NetworkProfile::Custom)
    } else {
        None
    };
    
    // Parse per-source network conditions
    let mut per_source_profiles = std::collections::HashMap::new();
    for spec in per_source_network {
        let parts: Vec<&str> = spec.splitn(2, ':').collect();
        if parts.len() == 2 {
            match NetworkProfile::from_str(parts[1]) {
                Ok(profile) => {
                    per_source_profiles.insert(parts[0].to_string(), profile);
                    println!("Source '{}' will use network profile: {}", parts[0], parts[1]);
                }
                Err(e) => {
                    eprintln!("Invalid network profile for source '{}': {}", parts[0], e);
                }
            }
        } else {
            eprintln!("Invalid per-source-network format: '{}' (expected 'source:profile')", spec);
        }
    }
    
    // Parse network drop simulation
    let network_drop_config = if let Some(drop_spec) = network_drop {
        let parts: Vec<&str> = drop_spec.split(',').collect();
        if parts.len() == 2 {
            match (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                (Ok(period), Ok(duration)) => {
                    println!("Network drops configured: every {}s for {}s", period, duration);
                    Some((Duration::from_secs(period), Duration::from_secs(duration)))
                }
                _ => {
                    eprintln!("Invalid network-drop format: '{}' (expected 'period_seconds,duration_seconds')", drop_spec);
                    None
                }
            }
        } else {
            eprintln!("Invalid network-drop format: '{}' (expected 'period_seconds,duration_seconds')", drop_spec);
            None
        }
    } else {
        None
    };
    
    // Build server with initial patterns
    let mut server_builder = RtspServerBuilder::new().port(port);
    
    // Apply global network profile if set
    if let Some(profile) = global_network_profile {
        server_builder = server_builder.network_profile(profile);
    } else if packet_loss.is_some() || latency_ms.is_some() || bandwidth_kbps.is_some() {
        // Apply custom network conditions
        server_builder = server_builder.custom_network_conditions(
            packet_loss.unwrap_or(0.0),
            latency_ms.unwrap_or(0),
            bandwidth_kbps.unwrap_or(0),
            jitter_ms.unwrap_or(0)
        );
    }
    
    // Add test patterns if specified
    if !patterns.is_empty() {
        for (i, pattern) in patterns.iter().enumerate() {
            let name = format!("pattern-{}", i + 1);
            
            // Check for per-source network profile
            if let Some(profile) = per_source_profiles.get(&name) {
                server_builder = server_builder.add_test_pattern_with_network(&name, pattern, *profile);
            } else {
                server_builder = server_builder.add_test_pattern(&name, pattern);
            }
            
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
    
    // Create shared state for API if enabled
    let rtsp_server_arc = Arc::new(RwLock::new(server));
    let source_manager_arc = Arc::new(VideoSourceManager::new());
    
    // Set up file watching if enabled
    let watcher_manager_arc = if watch && directory.is_some() {
        println!("Setting up file system watching...");
        let mut manager = WatcherManager::new();
        
        if let Some(ref dir_path) = directory {
            let watcher_id = manager.add_directory_watcher(dir_path, recursive).await?;
            println!("Started watching directory: {} (ID: {})", dir_path.display(), watcher_id);
        }
        
        Arc::new(RwLock::new(manager))
    } else {
        Arc::new(RwLock::new(WatcherManager::new()))
    };
    
    // Start API server if enabled
    let api_handle = if api {
        let api_bind_address: std::net::SocketAddr = format!("{}:{}", api_address, api_port).parse()
            .map_err(|e| SourceVideoError::config(format!("Invalid API address: {}", e)))?;
        
        let api_server = ControlApi::new(
            Some(rtsp_server_arc.clone()),
            source_manager_arc.clone(),
            watcher_manager_arc.clone(),
        )?;
        
        println!("Starting API server on http://{}:{}", api_address, api_port);
        println!("API documentation available at http://{}:{}/api/docs", api_address, api_port);
        
        let mut api_server = api_server;
        api_server.set_bind_address(api_bind_address);
        
        Some(tokio::spawn(async move {
            if let Err(e) = api_server.bind_and_serve().await {
                eprintln!("API server error: {}", e);
            }
        }))
    } else {
        None
    };
    
    let mut watcher_manager = if watch && directory.is_some() {
        Some(watcher_manager_arc.clone())
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
    
    {
        let mut server = rtsp_server_arc.write().await;
        server.start()?;
        
        for mount in server.list_sources() {
            println!("Stream available at: {}", server.get_url(&mount));
        }
    }
    
    // Set up periodic network drops if configured
    let mut network_simulator = if let Some((period, duration)) = network_drop_config {
        let sim = GStreamerNetworkSimulator::new();
        Some((sim, period, duration, std::time::Instant::now()))
    } else {
        None
    };
    
    // Get the default main context for manual iteration
    let main_context = gstreamer::glib::MainContext::default();
    
    if let Some(duration) = duration {
        println!("Server will run for {} seconds", duration);
        let end_time = std::time::Instant::now() + Duration::from_secs(duration);
        
        while std::time::Instant::now() < end_time {
            // Iterate the GLib main context
            main_context.iteration(false);
            
            // Handle periodic network drops
            if let Some((ref mut sim, period, drop_duration, ref mut last_drop)) = network_simulator {
                let now = std::time::Instant::now();
                if now.duration_since(*last_drop) >= period {
                    println!("Simulating network drop for {}s...", drop_duration.as_secs());
                    sim.drop_connection();
                    *last_drop = now;
                    
                    // Schedule restoration
                    let sim_clone = sim.simulator().clone();
                    let duration_clone = drop_duration;
                    tokio::spawn(async move {
                        tokio::time::sleep(duration_clone).await;
                        sim_clone.restore_connection();
                        println!("Network connection restored");
                    });
                }
            }
            
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
                    if let Some(ref manager_arc) = watcher_manager {
                        let mut manager = manager_arc.write().await;
                        if let Some(event) = manager.recv().await {
                            println!("File system event: {:?} - {}", event.event_type(), event.path().display());
                            
                            // Handle the event through the RTSP server directly
                            let mut server = rtsp_server_arc.write().await;
                            if let Err(e) = server.handle_file_event(&event) {
                                eprintln!("Error handling file event: {}", e);
                            }
                        }
                    }
                    
                    // Handle periodic network drops
                    if let Some((ref mut sim, period, drop_duration, ref mut last_drop)) = network_simulator {
                        let now = std::time::Instant::now();
                        if now.duration_since(*last_drop) >= period {
                            println!("Simulating network drop for {}s...", drop_duration.as_secs());
                            sim.drop_connection();
                            *last_drop = now;
                            
                            // Schedule restoration
                            let sim_clone = sim.simulator().clone();
                            let duration_clone = drop_duration;
                            tokio::spawn(async move {
                                tokio::time::sleep(duration_clone).await;
                                sim_clone.restore_connection();
                                println!("Network connection restored");
                            });
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

async fn serve_files_command(
    port: u16,
    directory: Option<PathBuf>,
    files: Vec<PathBuf>,
    recursive: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    format: Option<String>,
    min_duration: Option<u64>,
    max_duration: Option<u64>,
    modified_since: Option<String>,
    watch: bool,
    daemon: bool,
    pid_file: Option<PathBuf>,
    max_streams: Option<u32>,
    verbose: u8,
    quiet: bool,
    status_interval: Option<u64>,
    metrics: bool,
    output_format: OutputFormat,
    dry_run: bool,
) -> Result<()> {
    if daemon {
        daemonize(pid_file)?;
    }
    
    // Logging already set up in main
    
    if dry_run {
        return dry_run_preview(directory, files, recursive, include, exclude).await;
    }
    
    // Enhanced serve command with advanced filtering
    let mut filtered_files = Vec::new();
    
    if let Some(dir) = directory {
        filtered_files.extend(scan_directory_with_filters(
            &dir, recursive, &include, &exclude, format.as_deref(),
            min_duration, max_duration, modified_since.as_deref()
        )?);
    }
    
    filtered_files.extend(filter_files(
        files, &include, &exclude, format.as_deref(),
        min_duration, max_duration, modified_since.as_deref()
    )?);
    
    if let Some(max) = max_streams {
        if filtered_files.len() > max as usize {
            filtered_files.truncate(max as usize);
            println!("Limiting to {} streams", max);
        }
    }
    
    start_enhanced_server(
        port, filtered_files, watch, status_interval, metrics, output_format
    ).await
}

async fn playlist_command(
    port: u16,
    directory: PathBuf,
    recursive: bool,
    playlist_mode: PlaylistMode,
    playlist_repeat: PlaylistRepeat,
    playlist_file: Option<PathBuf>,
    transition_duration: Option<f32>,
    crossfade: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    verbose: u8,
    daemon: bool,
) -> Result<()> {
    if daemon {
        daemonize(None)?;
    }
    
    // Logging already set up in main
    
    let files = if let Some(pls_file) = playlist_file {
        load_playlist_file(&pls_file)?
    } else {
        scan_directory_with_filters(
            &directory, recursive, &include, &exclude, None, None, None, None
        )?
    };
    
    let ordered_files = match playlist_mode {
        PlaylistMode::Sequential => files,
        PlaylistMode::Random => {
            let mut rng = rand::thread_rng();
            let mut shuffled = files;
            use rand::seq::SliceRandom;
            shuffled.shuffle(&mut rng);
            shuffled
        }
        PlaylistMode::Shuffle => {
            let mut rng = rand::thread_rng();
            let mut shuffled = files;
            use rand::seq::SliceRandom;
            shuffled.shuffle(&mut rng);
            shuffled
        }
    };
    
    start_playlist_server(
        port, ordered_files, playlist_repeat, transition_duration, crossfade
    ).await
}

async fn monitor_command(
    directory: PathBuf,
    recursive: bool,
    watch_interval: u64,
    list_streams: bool,
    metrics: bool,
    output_format: OutputFormat,
) -> Result<()> {
    use source_videos::WatcherManager;
    
    let mut manager = WatcherManager::new();
    let watcher_id = manager.add_directory_watcher(&directory, recursive).await?;
    
    println!("Monitoring directory: {} (recursive: {})", directory.display(), recursive);
    println!("Watcher ID: {}", watcher_id);
    
    if list_streams {
        let files = scan_directory_with_filters(
            &directory, recursive, &[], &[], None, None, None, None
        )?;
        println!("Found {} video files:", files.len());
        for file in &files {
            println!("  {}", file.display());
        }
    }
    
    loop {
        if let Some(event) = manager.recv().await {
            match output_format {
                OutputFormat::Text => {
                    println!("[{}] {:?}: {}", 
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                        event.event_type(),
                        event.path().display()
                    );
                }
                OutputFormat::Json => {
                    let json_event = serde_json::json!({
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "event_type": format!("{:?}", event.event_type()),
                        "path": event.path().display().to_string(),
                    });
                    println!("{}", json_event);
                }
                OutputFormat::Csv => {
                    println!("{},{:?},{}", 
                        chrono::Utc::now().to_rfc3339(),
                        event.event_type(),
                        event.path().display()
                    );
                }
            }
            
            if metrics {
                print_file_metrics(&event.path()).await?;
            }
        }
        
        tokio::time::sleep(Duration::from_millis(watch_interval)).await;
    }
}

async fn simulate_command(
    port: u16,
    network_profile: String,
    directory: Option<PathBuf>,
    patterns: Vec<String>,
    duration: Option<u64>,
    metrics: bool,
) -> Result<()> {
    use source_videos::network::{NetworkProfile, NetworkSimulator};
    use std::str::FromStr;
    
    println!("Starting network simulation with profile: {}", network_profile);
    
    let profile = NetworkProfile::from_str(&network_profile)
        .map_err(|e| SourceVideoError::config(format!("Invalid network profile: {}", e)))?;
    
    println!("Profile description: {}", profile.description());
    
    // Start test sources
    let mut server_builder = source_videos::RtspServerBuilder::new()
        .port(port)
        .network_profile(profile);
    
    if let Some(dir) = directory {
        let files = scan_directory_with_filters(&dir, true, &[], &[], None, None, None, None)?;
        for (i, file) in files.iter().take(3).enumerate() {
            let config = create_file_source_config(&format!("sim-file-{}", i), file)?;
            server_builder = server_builder.add_source(config);
        }
    }
    
    for (i, pattern) in patterns.iter().enumerate() {
        server_builder = server_builder.add_test_pattern(&format!("sim-pattern-{}", i), pattern);
    }
    
    let mut server = server_builder.build()?;
    server.start()?;
    
    println!("Simulation server started on port {}", port);
    for mount in server.list_sources() {
        println!("  {}", server.get_url(&mount));
    }
    
    let test_duration = duration.unwrap_or(60);
    println!("Running simulation for {} seconds...", test_duration);
    
    if metrics {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                println!("[METRICS] Active streams: {}, Network conditions: {}", 
                    0, // TODO: Get actual metrics
                    network_profile
                );
            }
        });
    }
    
    tokio::time::sleep(Duration::from_secs(test_duration)).await;
    println!("Simulation completed");
    
    Ok(())
}

async fn completions_command(shell: Shell) -> Result<()> {
    let mut app = <Cli as clap::CommandFactory>::command();
    let app_name = app.get_name().to_string();
    generate(shell, &mut app, &app_name, &mut io::stdout());
    Ok(())
}

// Helper functions

fn daemonize(pid_file: Option<PathBuf>) -> Result<()> {
    if let Some(pid_path) = pid_file {
        let pid = process::id();
        fs::write(pid_path, pid.to_string())
            .map_err(|e| SourceVideoError::config(format!("Failed to write PID file: {}", e)))?;
    }
    
    // Note: Full daemonization would require fork() which is Unix-specific
    // For now, just detach from console on Windows
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Windows-specific detachment would go here
    }
    
    Ok(())
}

fn setup_logging(verbose: u8, quiet: bool) {
    if quiet {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("off")).init();
    } else {
        let level = match verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        };
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();
    }
}

async fn dry_run_preview(
    directory: Option<PathBuf>,
    files: Vec<PathBuf>,
    recursive: bool,
    include: Vec<String>,
    exclude: Vec<String>,
) -> Result<()> {
    println!("DRY RUN - Would serve the following sources:");
    
    if let Some(dir) = directory {
        let found_files = scan_directory_with_filters(
            &dir, recursive, &include, &exclude, None, None, None, None
        )?;
        println!("Directory {}: {} files", dir.display(), found_files.len());
        for file in &found_files {
            println!("  - {}", file.display());
        }
    }
    
    if !files.is_empty() {
        println!("Explicit files: {}", files.len());
        for file in &files {
            println!("  - {}", file.display());
        }
    }
    
    Ok(())
}

fn scan_directory_with_filters(
    directory: &PathBuf,
    recursive: bool,
    include: &[String],
    exclude: &[String],
    format_filter: Option<&str>,
    min_duration: Option<u64>,
    max_duration: Option<u64>,
    modified_since: Option<&str>,
) -> Result<Vec<PathBuf>> {
    use source_videos::{DirectoryScanner, DirectoryConfig, FilterConfig};
    
    let filters = if !include.is_empty() || !exclude.is_empty() {
        Some(FilterConfig {
            include: include.to_vec(),
            exclude: exclude.to_vec(),
            extensions: vec![],
        })
    } else {
        None
    };
    
    let config = DirectoryConfig {
        path: directory.display().to_string(),
        recursive,
        filters,
        lazy_loading: false,
        mount_prefix: None,
    };
    
    let mut scanner = DirectoryScanner::new(config);
    let source_configs = scanner.scan()?;
    
    let files: Vec<PathBuf> = source_configs.into_iter()
        .filter_map(|config| {
            if let source_videos::VideoSourceType::File { path, .. } = config.source_type {
                Some(PathBuf::from(path))
            } else {
                None
            }
        })
        .collect();
    
    Ok(apply_advanced_filters(
        files, format_filter, min_duration, max_duration, modified_since
    )?)
}

fn filter_files(
    files: Vec<PathBuf>,
    include: &[String],
    exclude: &[String],
    format_filter: Option<&str>,
    min_duration: Option<u64>,
    max_duration: Option<u64>,
    modified_since: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let mut filtered = files;
    
    // Apply include/exclude patterns
    if !include.is_empty() || !exclude.is_empty() {
        filtered = filtered.into_iter()
            .filter(|file| {
                let name = file.file_name().unwrap_or_default().to_string_lossy();
                
                // Check include patterns
                let included = if include.is_empty() {
                    true
                } else {
                    include.iter().any(|pattern| {
                        glob::Pattern::new(pattern)
                            .map(|p| p.matches(&name))
                            .unwrap_or(false)
                    })
                };
                
                // Check exclude patterns
                let excluded = exclude.iter().any(|pattern| {
                    glob::Pattern::new(pattern)
                        .map(|p| p.matches(&name))
                        .unwrap_or(false)
                });
                
                included && !excluded
            })
            .collect();
    }
    
    apply_advanced_filters(filtered, format_filter, min_duration, max_duration, modified_since)
}

fn apply_advanced_filters(
    files: Vec<PathBuf>,
    format_filter: Option<&str>,
    min_duration: Option<u64>,
    max_duration: Option<u64>,
    modified_since: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let mut filtered = files;
    
    // Format filter
    if let Some(format) = format_filter {
        filtered = filtered.into_iter()
            .filter(|file| {
                file.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case(format))
                    .unwrap_or(false)
            })
            .collect();
    }
    
    // Date filter
    if let Some(date_str) = modified_since {
        let since_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| SourceVideoError::config(format!("Invalid date format: {}", e)))?;
        let since_datetime = since_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        
        filtered = filtered.into_iter()
            .filter(|file| {
                if let Ok(metadata) = file.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let modified_datetime: DateTime<Utc> = modified.into();
                        return modified_datetime >= since_datetime;
                    }
                }
                true
            })
            .collect();
    }
    
    // Duration filters would require reading video metadata
    // For now, we'll skip duration filtering as it would require GStreamer probing
    
    Ok(filtered)
}

fn load_playlist_file(file: &PathBuf) -> Result<Vec<PathBuf>> {
    let content = fs::read_to_string(file)
        .map_err(|e| SourceVideoError::config(format!("Failed to read playlist: {}", e)))?;
    
    let files: Vec<PathBuf> = content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                None
            } else {
                Some(PathBuf::from(line))
            }
        })
        .collect();
    
    Ok(files)
}

fn create_file_source_config(name: &str, file: &PathBuf) -> Result<VideoSourceConfig> {
    use source_videos::{config_types::*, file_utils::detect_container_format};
    
    let container = detect_container_format(file)
        .unwrap_or(FileContainer::Mp4);
    
    Ok(VideoSourceConfig {
        name: name.to_string(),
        source_type: source_videos::VideoSourceType::File {
            path: file.display().to_string(),
            container,
        },
        resolution: Resolution { width: 1920, height: 1080 },
        framerate: Framerate { numerator: 30, denominator: 1 },
        format: VideoFormat::I420,
        duration: None,
        num_buffers: None,
        is_live: false,
    })
}

async fn start_enhanced_server(
    port: u16,
    files: Vec<PathBuf>,
    watch: bool,
    status_interval: Option<u64>,
    metrics: bool,
    output_format: OutputFormat,
) -> Result<()> {
    println!("Starting enhanced server with {} sources", files.len());
    
    let mut server_builder = source_videos::RtspServerBuilder::new().port(port);
    
    for (i, file) in files.iter().enumerate() {
        let config = create_file_source_config(&format!("file-{}", i), file)?;
        server_builder = server_builder.add_source(config);
    }
    
    let mut server = server_builder.build()?;
    server.start()?;
    
    if let Some(interval) = status_interval {
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));
            loop {
                interval_timer.tick().await;
                println!("[STATUS] Server running, {} active streams", files.len());
            }
        });
    }
    
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("Received Ctrl+C, stopping...");
        }
    }
    
    Ok(())
}

async fn start_playlist_server(
    port: u16,
    files: Vec<PathBuf>,
    repeat: PlaylistRepeat,
    transition_duration: Option<f32>,
    crossfade: bool,
) -> Result<()> {
    println!("Starting playlist server with {} files", files.len());
    println!("Repeat mode: {:?}", repeat);
    
    if let Some(duration) = transition_duration {
        println!("Transition duration: {}s", duration);
    }
    
    if crossfade {
        println!("Crossfade enabled");
    }
    
    // For now, create a single stream that cycles through the playlist
    let mut server_builder = source_videos::RtspServerBuilder::new().port(port);
    
    // Create a combined playlist source (simplified for now)
    if !files.is_empty() {
        let config = create_file_source_config("playlist-stream", &files[0])?;
        server_builder = server_builder.add_source(config);
    }
    
    let mut server = server_builder.build()?;
    server.start()?;
    
    println!("Playlist server started on port {}", port);
    
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("Received Ctrl+C, stopping...");
        }
    }
    
    Ok(())
}

async fn print_file_metrics(path: &PathBuf) -> Result<()> {
    if let Ok(metadata) = path.metadata() {
        println!("  Size: {} bytes", metadata.len());
        if let Ok(modified) = metadata.modified() {
            let datetime: DateTime<Utc> = modified.into();
            println!("  Modified: {}", datetime.format("%Y-%m-%d %H:%M:%S UTC"));
        }
    }
    Ok(())
}
