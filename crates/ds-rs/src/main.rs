#![allow(unused)]
use clap::Parser;
use ds_rs::{init, app::Application};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use gstreamer::glib;

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
    
    // Create and initialize the application
    let mut app = Application::new(args.uri)?;
    app.init()?;
    
    // Create atomic flag for shutdown signaling
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    // Set up signal handler for graceful shutdown
    ctrlc::set_handler(move || {
        println!("\nReceived interrupt signal, shutting down...");
        running_clone.store(false, Ordering::SeqCst);
        // Wake up the main context so it checks the flag
        glib::MainContext::default().wakeup();
    })?;
    
    // Run the application with manual main context iteration
    app.run_with_main_context(running)?;
    
    println!("\nApplication exited successfully");
    Ok(())
}
