#![allow(unused)]
use clap::Parser;
use ds_rs::{init, app::Application};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
#[command(
    name = "ds-runtime-demo",
    about = "DeepStream Rust - Runtime Source Addition/Deletion Demo",
    long_about = "Demonstrates dynamic video source management in AI-powered video analytics pipelines.\n\
                  This application showcases the runtime source control APIs by automatically adding\n\
                  sources every 10 seconds up to MAX_NUM_SOURCES, then removing them periodically."
)]
struct Args {
    /// URI of the video source (file:///path/to/video.mp4 or rtsp://...)
    #[arg(help = "Video source URI")]
    uri: String,
    
    /// Enable debug logging
    #[arg(short, long, help = "Enable debug output")]
    debug: bool,
    
    /// Force a specific backend (mock, standard, deepstream)
    #[arg(short, long, help = "Force backend selection")]
    backend: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Set logging level
    if args.debug {
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }
    }
    
    // Force backend if specified
    if let Some(backend) = args.backend {
        unsafe {
            std::env::set_var("FORCE_BACKEND", backend);
        }
    }
    
    // Initialize GStreamer and the library
    init()?;
    
    println!("DeepStream Rust - Runtime Source Addition/Deletion Demo");
    println!("========================================================\n");
    
    // Create the runtime for async operations
    let runtime = Runtime::new()?;
    
    // Create and run the application
    runtime.block_on(async {
        let mut app = Application::new(args.uri)?;
        
        // Initialize the pipeline
        app.init()?;
        
        // Create a separate shutdown channel for the signal handler
        let shutdown_requested = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_clone = shutdown_requested.clone();
        
        // Set up signal handler for graceful shutdown
        ctrlc::set_handler(move || {
            println!("\nReceived interrupt signal, shutting down...");
            shutdown_clone.store(true, std::sync::atomic::Ordering::Relaxed);
        })?;
        
        // Pass the shutdown flag to the application
        app.set_shutdown_flag(shutdown_requested.clone());
        
        // Run the application
        app.run().await?;
        
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    
    println!("\nApplication exited successfully");
    Ok(())
}
