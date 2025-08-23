#![allow(unused)]
use ds_rs::{init, BackendManager, BackendType, PlatformInfo};
use ds_rs::backend::detector;
use ds_rs::elements::factory::ElementFactory;
use std::sync::Arc;

#[test]
fn test_backend_detection() {
    let _ = init();
    
    let backends = detector::detect_available_backends();
    
    // At minimum, mock backend should always be available
    assert!(!backends.is_empty());
    assert!(backends.contains(&BackendType::Mock));
    
    println!("Detected backends: {:?}", backends);
    
    for backend in &backends {
        println!("  - {}: Available", backend.name());
    }
}

#[test]
fn test_backend_manager_auto_detection() {
    let _ = init();
    
    let manager = BackendManager::new();
    assert!(manager.is_ok());
    
    let manager = manager.unwrap();
    let backend_type = manager.backend_type();
    
    println!("Auto-selected backend: {}", backend_type.name());
    
    let capabilities = manager.capabilities();
    println!("Backend capabilities:");
    println!("  - Supports inference: {}", capabilities.supports_inference);
    println!("  - Supports tracking: {}", capabilities.supports_tracking);
    println!("  - Supports OSD: {}", capabilities.supports_osd);
    println!("  - Supports batching: {}", capabilities.supports_batching);
    println!("  - Max batch size: {}", capabilities.max_batch_size);
}

#[test]
fn test_mock_backend_creation() {
    let _ = init();
    
    let manager = BackendManager::with_backend(BackendType::Mock);
    assert!(manager.is_ok());
    
    let manager = manager.unwrap();
    assert_eq!(manager.backend_type(), BackendType::Mock);
    
    // Mock backend should support all features (simulated)
    let caps = manager.capabilities();
    assert!(caps.supports_inference);
    assert!(caps.supports_tracking);
    assert!(caps.supports_osd);
    assert!(caps.supports_batching);
}

#[test]
fn test_element_creation_with_mock_backend() {
    let _ = init();
    
    let manager = Arc::new(BackendManager::with_backend(BackendType::Mock).unwrap());
    let factory = ElementFactory::new(manager);
    
    // Test creating various elements
    let mux = factory.create_stream_mux(Some("test-mux"));
    assert!(mux.is_ok());
    
    let inference = factory.create_inference(Some("test-inference"), "config.txt");
    assert!(inference.is_ok());
    
    let tracker = factory.create_tracker(Some("test-tracker"));
    assert!(tracker.is_ok());
    
    let tiler = factory.create_tiler(Some("test-tiler"));
    assert!(tiler.is_ok());
    
    let osd = factory.create_osd(Some("test-osd"));
    assert!(osd.is_ok());
    
    let convert = factory.create_video_convert(Some("test-convert"));
    assert!(convert.is_ok());
    
    let sink = factory.create_video_sink(Some("test-sink"));
    assert!(sink.is_ok());
}

#[test]
fn test_standard_backend_availability() {
    let _ = init();
    
    // Check if standard GStreamer elements are available
    let has_compositor = detector::check_element_availability("compositor");
    let has_videoconvert = detector::check_element_availability("videoconvert");
    
    println!("Standard GStreamer elements:");
    println!("  - compositor: {}", has_compositor);
    println!("  - videoconvert: {}", has_videoconvert);
    
    if has_compositor && has_videoconvert {
        let manager = BackendManager::with_backend(BackendType::Standard);
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert_eq!(manager.backend_type(), BackendType::Standard);
    }
}

#[test]
fn test_deepstream_backend_availability() {
    let _ = init();
    
    // Check if DeepStream elements are available
    let has_nvstreammux = detector::check_element_availability("nvstreammux");
    let has_nvinfer = detector::check_element_availability("nvinfer");
    
    println!("DeepStream elements:");
    println!("  - nvstreammux: {}", has_nvstreammux);
    println!("  - nvinfer: {}", has_nvinfer);
    
    if has_nvstreammux && has_nvinfer {
        let platform = PlatformInfo::detect().unwrap();
        if platform.has_nvidia_hardware() {
            let manager = BackendManager::with_backend(BackendType::DeepStream);
            assert!(manager.is_ok());
            
            let manager = manager.unwrap();
            assert_eq!(manager.backend_type(), BackendType::DeepStream);
        }
    }
}

#[test]
fn test_backend_element_mapping() {
    let _ = init();
    
    let manager = BackendManager::with_backend(BackendType::Mock).unwrap();
    let backend = manager.backend();
    
    // Test element mappings for mock backend
    assert_eq!(backend.get_element_mapping("nvstreammux"), Some("tee"));
    assert_eq!(backend.get_element_mapping("nvinfer"), Some("identity"));
    assert_eq!(backend.get_element_mapping("nvtracker"), Some("identity"));
    assert_eq!(backend.get_element_mapping("nvdsosd"), Some("identity"));
    assert_eq!(backend.get_element_mapping("nvtiler"), Some("identity"));
    assert_eq!(backend.get_element_mapping("nvvideoconvert"), Some("identity"));
    assert_eq!(backend.get_element_mapping("nveglglessink"), Some("fakesink"));
}

#[test]
fn test_pipeline_creation_with_different_backends() {
    let _ = init();
    
    // Test with mock backend (always available)
    let mock_manager = Arc::new(BackendManager::with_backend(BackendType::Mock).unwrap());
    let mock_factory = ElementFactory::new(mock_manager);
    
    let pipeline = ds_rs::elements::factory::PipelineElements::create_base_pipeline(
        &mock_factory,
        "mock-pipeline"
    );
    assert!(pipeline.is_ok());
    
    let mut pipeline = pipeline.unwrap();
    assert!(pipeline.add_inference(&mock_factory, "test.txt").is_ok());
    assert!(pipeline.add_tracker(&mock_factory).is_ok());
    assert!(pipeline.link_pipeline().is_ok());
}

#[test]
fn test_platform_specific_properties() {
    let _ = init();
    
    let platform = PlatformInfo::detect().unwrap();
    let manager = BackendManager::new().unwrap();
    
    println!("Platform: {:?}", platform.platform);
    println!("CUDA Version: {:?}", platform.cuda_version);
    println!("GPU ID: {}", platform.get_gpu_id());
    println!("Batch Timeout: {}", platform.get_batch_timeout());
    println!("Compute Mode: {}", platform.get_compute_mode());
    
    // Verify platform-specific properties are applied
    assert!(platform.get_batch_timeout() > 0);
    assert!(platform.get_compute_mode() >= 0);
}
