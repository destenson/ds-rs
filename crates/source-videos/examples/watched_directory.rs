#![allow(unused)]

use source_videos::{
    init, DirectoryWatcher, WatcherManager, FileSystemEvent, LoopConfig, 
    create_looping_source, FileVideoSource, Result,
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Initialize GStreamer
    init()?;
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let directory = args.get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    
    let auto_repeat = args.get(2)
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);
    
    let recursive = args.get(3)
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);
    
    println!("=== Watched Directory Example ===");
    println!("Directory: {}", directory.display());
    println!("Auto-repeat: {}", auto_repeat);
    println!("Recursive: {}", recursive);
    println!("Usage: {} <directory> [auto_repeat=true/false] [recursive=true/false]", args[0]);
    println!();
    
    if !directory.exists() {
        eprintln!("Error: Directory '{}' does not exist", directory.display());
        return Ok(());
    }
    
    if !directory.is_dir() {
        eprintln!("Error: '{}' is not a directory", directory.display());
        return Ok(());
    }
    
    // Set up watcher manager
    let mut watcher_manager = WatcherManager::new();
    
    // Add directory watcher
    println!("Setting up directory watcher...");
    let watcher_id = watcher_manager
        .add_directory_watcher(&directory, recursive)
        .await?;
    
    println!("✅ Started watching directory: {} (ID: {})", directory.display(), watcher_id);
    
    if recursive {
        println!("   📁 Recursive watching enabled - subdirectories will be monitored");
    }
    
    // Configure auto-repeat if enabled
    if auto_repeat {
        let loop_config = LoopConfig {
            max_loops: None, // Infinite loops
            seamless: true,
            gap_duration: Duration::from_millis(100),
            ..Default::default()
        };
        
        println!("   🔄 Auto-repeat configured: seamless={}, infinite loops", loop_config.seamless);
    }
    
    println!();
    println!("Monitoring file system events...");
    println!("Press Ctrl+C to stop");
    println!();
    
    let mut event_count = 0;
    let start_time = std::time::Instant::now();
    
    // Main event loop
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("\nReceived Ctrl+C, stopping...");
                break;
            }
            event = watcher_manager.recv() => {
                if let Some(event) = event {
                    event_count += 1;
                    let elapsed = start_time.elapsed();
                    
                    println!("[{:>6.1}s] Event #{}: {:>8} - {}", 
                        elapsed.as_secs_f32(),
                        event_count,
                        event.event_type().to_uppercase(),
                        event.path().display()
                    );
                    
                    match &event {
                        FileSystemEvent::Created(metadata) => {
                            println!("           📄 New video file detected");
                            if let Some(size) = metadata.size {
                                println!("           📊 Size: {} bytes", size);
                            }
                            
                            // Demonstrate creating a video source
                            match FileVideoSource::from_config(&create_test_config(&metadata.path)) {
                                Ok(mut source) => {
                                    if auto_repeat {
                                        println!("           🔄 Would create looping source");
                                        // In a real application, you would add this to your source manager
                                    } else {
                                        println!("           ▶️  Would create standard source");
                                    }
                                }
                                Err(e) => {
                                    println!("           ❌ Error creating source: {}", e);
                                }
                            }
                        }
                        FileSystemEvent::Modified(metadata) => {
                            println!("           🔄 Video file changed");
                            if let Some(size) = metadata.size {
                                println!("           📊 New size: {} bytes", size);
                            }
                            println!("           🔥 Hot-reload would trigger here");
                        }
                        FileSystemEvent::Deleted(metadata) => {
                            println!("           🗑️  Video file removed");
                            println!("           ⏹️  Would stop and remove associated source");
                        }
                        FileSystemEvent::Accessed(metadata) => {
                            println!("           👁️  File accessed (read)");
                        }
                        FileSystemEvent::Renamed { from, to } => {
                            println!("           📝 File renamed");
                            println!("               From: {}", from.path.display());
                            println!("               To:   {}", to.path.display());
                        }
                        FileSystemEvent::Error { path, error, .. } => {
                            println!("           ❌ Error: {} - {}", error, path.display());
                        }
                    }
                    
                    println!();
                }
            }
        }
    }
    
    // Stop watching
    println!("Stopping watchers...");
    watcher_manager.stop_all().await?;
    
    // Print statistics
    let total_time = start_time.elapsed();
    println!();
    println!("=== Session Summary ===");
    println!("Directory watched: {}", directory.display());
    println!("Total events: {}", event_count);
    println!("Session duration: {:.1} seconds", total_time.as_secs_f32());
    if event_count > 0 {
        println!("Average rate: {:.1} events/minute", 
            (event_count as f32) / (total_time.as_secs_f32() / 60.0));
    }
    
    println!("✅ Graceful shutdown complete");
    
    Ok(())
}

fn create_test_config(path: &std::path::Path) -> source_videos::VideoSourceConfig {
    use source_videos::{VideoSourceConfig, VideoSourceType};
    use source_videos::config_types::{FileContainer, Resolution, Framerate, VideoFormat};
    
    let container = match path.extension().and_then(|e| e.to_str()) {
        Some("mp4") => FileContainer::Mp4,
        Some("mkv") => FileContainer::Mkv,
        Some("avi") => FileContainer::Avi,
        Some("webm") => FileContainer::WebM,
        _ => FileContainer::Mp4,
    };
    
    VideoSourceConfig {
        name: path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video")
            .to_string(),
        source_type: VideoSourceType::File {
            path: path.display().to_string(),
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
    }
}
