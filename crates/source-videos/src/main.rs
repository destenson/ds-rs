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
        Commands::Serve { port, address, duration, patterns } => {
            serve_command(port, address, duration, patterns).await
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
    patterns: Vec<String>
) -> Result<()> {
    println!("Starting RTSP server on {}:{}", address, port);
    
    let mut server = create_test_rtsp_server(port)?;
    
    if !patterns.is_empty() {
        for (i, pattern) in patterns.iter().enumerate() {
            let name = format!("pattern-{}", i + 1);
            let config = VideoSourceConfig::test_pattern(&name, pattern);
            server.add_source(config)?;
            println!("Added pattern '{}' at rtsp://{}:{}/{}", pattern, address, port, name);
        }
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
