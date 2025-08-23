use ds_rs::{init, BackendManager, BackendType, PlatformInfo};
use ds_rs::backend::detector;
use ds_rs::elements::factory::{ElementFactory, PipelineElements};
use ds_rs::elements::abstracted::AbstractedPipeline;
use std::sync::Arc;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize GStreamer and logging
    init()?;
    
    println!("DeepStream Rust - Cross-Platform Example");
    println!("=========================================\n");
    
    // Detect platform
    let platform = PlatformInfo::detect()?;
    println!("Platform Detection:");
    println!("  Platform: {:?}", platform.platform);
    println!("  CUDA Version: {:?}", platform.cuda_version);
    println!("  GPU ID: {}", platform.get_gpu_id());
    println!("  Has NVIDIA Hardware: {}", platform.has_nvidia_hardware());
    println!();
    
    // Detect available backends
    let available_backends = detector::detect_available_backends();
    println!("Available Backends:");
    for backend in &available_backends {
        println!("  - {}", backend.name());
    }
    println!();
    
    // Allow backend selection via command line or auto-detect
    let backend_type = if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "deepstream" => BackendType::DeepStream,
            "standard" => BackendType::Standard,
            "mock" => BackendType::Mock,
            _ => {
                println!("Unknown backend '{}', using auto-detection", arg);
                BackendManager::new()?.backend_type()
            }
        }
    } else {
        // Auto-detect best backend
        BackendManager::new()?.backend_type()
    };
    
    println!("Selected Backend: {}", backend_type.name());
    println!();
    
    // Create backend manager with selected backend
    let manager = match BackendManager::with_backend(backend_type) {
        Ok(m) => Arc::new(m),
        Err(e) => {
            eprintln!("Failed to create {} backend: {}", backend_type.name(), e);
            eprintln!("Falling back to Mock backend");
            Arc::new(BackendManager::with_backend(BackendType::Mock)?)
        }
    };
    
    // Display backend capabilities
    let capabilities = manager.capabilities();
    println!("Backend Capabilities:");
    println!("  Supports Inference: {}", capabilities.supports_inference);
    println!("  Supports Tracking: {}", capabilities.supports_tracking);
    println!("  Supports OSD: {}", capabilities.supports_osd);
    println!("  Supports Batching: {}", capabilities.supports_batching);
    println!("  Supports Hardware Decode: {}", capabilities.supports_hardware_decode);
    println!("  Max Batch Size: {}", capabilities.max_batch_size);
    println!();
    
    // Create element factory
    let factory = ElementFactory::new(manager.clone());
    
    // Build a sample pipeline
    println!("Building Pipeline...");
    let mut pipeline = PipelineElements::create_base_pipeline(&factory, "cross-platform-demo")?;
    
    // Add inference if supported
    if capabilities.supports_inference {
        println!("  Adding inference element...");
        pipeline.add_inference(&factory, "dstest_pgie_config.txt")?;
    } else {
        println!("  Skipping inference (not supported by backend)");
    }
    
    // Add tracker if supported
    if capabilities.supports_tracking {
        println!("  Adding tracker element...");
        pipeline.add_tracker(&factory)?;
    } else {
        println!("  Skipping tracker (not supported by backend)");
    }
    
    // Add tiler for multi-stream display
    println!("  Adding tiler element...");
    pipeline.add_tiler(&factory)?;
    
    // Link pipeline
    println!("  Linking pipeline elements...");
    pipeline.link_pipeline()?;
    
    println!("Pipeline built successfully!");
    println!();
    
    // Demonstrate abstracted pipeline
    demonstrate_abstracted_pipeline(manager.backend_type())?;
    
    // Print element mappings
    print_element_mappings(&factory)?;
    
    println!("\nExample completed successfully!");
    println!("You can run this example with different backends:");
    println!("  cargo run --example cross_platform mock");
    println!("  cargo run --example cross_platform standard");
    println!("  cargo run --example cross_platform deepstream");
    
    Ok(())
}

fn demonstrate_abstracted_pipeline(backend_type: BackendType) -> Result<(), Box<dyn std::error::Error>> {
    println!("Abstracted Pipeline Demo:");
    println!("========================");
    
    let pipeline = AbstractedPipeline::new("abstracted-demo", backend_type);
    
    // Report capabilities for the abstracted pipeline
    pipeline.report_capabilities();
    
    println!();
    Ok(())
}

fn print_element_mappings(factory: &ElementFactory) -> Result<(), Box<dyn std::error::Error>> {
    println!("Element Mappings for Current Backend:");
    println!("=====================================");
    
    let deepstream_elements = vec![
        "nvstreammux",
        "nvinfer",
        "nvtracker",
        "nvdsosd",
        "nvtiler",
        "nvvideoconvert",
        "nveglglessink",
        "nvv4l2decoder",
    ];
    
    for ds_element in deepstream_elements {
        if let Some(mapped) = factory.get_backend_mapping(ds_element) {
            println!("  {} -> {}", ds_element, mapped);
        } else {
            println!("  {} -> (no mapping)", ds_element);
        }
    }
    
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_runs() {
        // Just verify the example compiles and basic initialization works
        assert!(init().is_ok());
        assert!(PlatformInfo::detect().is_ok());
    }
}