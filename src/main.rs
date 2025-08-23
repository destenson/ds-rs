
use ds_rs::{init, BackendManager, PlatformInfo};

fn main() -> Result<(), i32> {
    // Initialize the library
    init()
        .inspect_err(|e| eprintln!("Initialization error: {}", e))
        .map_err(|_| 1)?;
    
    println!("DeepStream Rust Application");
    println!("===========================\n");
    
    // Detect platform
    let platform = PlatformInfo::detect()
        .inspect_err(|e| eprintln!("Platform detection error: {}", e))
        .map_err(|_| 2)?;
    println!("Platform: {:?}", platform.platform);
    println!("CUDA Version: {:?}", platform.cuda_version);
    println!("Has NVIDIA Hardware: {}\n", platform.has_nvidia_hardware());
    
    // Create backend manager
    let manager = BackendManager::new()
        .inspect_err(|e| eprintln!("Backend manager error: {}", e))
        .map_err(|_| 3)?;
    println!("Selected Backend: {}", manager.backend_type().name());
    
    let caps = manager.capabilities();
    println!("Backend Capabilities:");
    println!("  - Inference: {}", caps.supports_inference);
    println!("  - Tracking: {}", caps.supports_tracking);
    println!("  - OSD: {}", caps.supports_osd);
    println!("  - Batching: {}", caps.supports_batching);
    
    println!("\nApplication initialized successfully!");
    println!("Run 'cargo run --example cross_platform' for a full demo.");
    
    Ok(())
}
