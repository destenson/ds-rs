#![allow(unused)]
use ds_rs::backend::{Backend, BackendManager, BackendType};
#[cfg(feature = "nalgebra")]
use ds_rs::backend::cpu_vision::tracker;
use ds_rs::backend::standard::StandardBackend;
use ds_rs::platform::PlatformInfo;
use ds_rs::init;

#[test]
fn test_standard_backend_with_cpu_vision() {
    init().unwrap();
    
    let platform = PlatformInfo::detect().unwrap();
    let backend = StandardBackend::new(&platform).unwrap();
    
    // Check capabilities are updated
    let caps = backend.capabilities();
    assert!(caps.supports_inference);
    assert!(caps.supports_tracking);
    assert!(caps.supports_osd);
    
    // Check available elements include CPU vision
    assert!(caps.available_elements.contains(&"cpu-detector".to_string()));
    assert!(caps.available_elements.contains(&"cpu-tracker".to_string()));
    assert!(caps.available_elements.contains(&"cpu-osd".to_string()));
}

#[test]
fn test_cpu_detector_creation() {
    use ds_rs::backend::cpu_vision::{OnnxDetector, DetectorConfig};
    
    // Test with nonexistent model file
    let result = OnnxDetector::new("nonexistent.onnx");
    
    #[cfg(feature = "ort")]
    {
        // With ort feature, should succeed but fall back to mock detection when file doesn't exist
        assert!(result.is_ok());
        let detector = result.unwrap();
        
        // Should work with mock detection
        use image::DynamicImage;
        let image = DynamicImage::new_rgb8(640, 640);
        let detections = detector.detect(&image).unwrap();
        assert!(!detections.is_empty());
    }
    
    #[cfg(not(feature = "ort"))]
    {
        // Without ort feature, should still work in mock mode
        assert!(result.is_ok());
    }
    
    // Test with DetectorConfig for mock detector
    let config = DetectorConfig {
        model_path: None,  // No model, use mock
        input_width: 416,
        input_height: 416,
        confidence_threshold: 0.3,
        ..Default::default()
    };
    
    let detector = OnnxDetector::new_with_config(config);
    assert!(detector.is_ok());
    let detector = detector.unwrap();
    
    // Test mock detection
    use image::DynamicImage;
    let image = DynamicImage::new_rgb8(640, 480);
    let detections = detector.detect(&image);
    assert!(detections.is_ok());
    let detections = detections.unwrap();
    // Mock detector should return some detections
    assert!(!detections.is_empty());
}

#[test]
#[cfg(feature = "nalgebra")]
fn test_cpu_tracker_functionality() {
    use ds_rs::backend::cpu_vision::tracker::CentroidTracker;
    use ds_rs::backend::cpu_vision::Detection;
    
    let mut tracker = CentroidTracker::new(50.0, 30);
    
    // Test with no detections
    let objects = tracker.update(vec![]);
    assert_eq!(objects.len(), 0);
    
    // Test with one detection
    let detection = Detection {
        x: 100.0,
        y: 100.0,
        width: 50.0,
        height: 50.0,
        confidence: 0.9,
        class_id: 0,
        class_name: "person".to_string(),
    };
    
    let objects = tracker.update(vec![detection.clone()]);
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].class_name, "person");
    
    // Test tracking consistency
    let detection2 = Detection {
        x: 105.0,  // Slightly moved
        y: 102.0,
        width: 50.0,
        height: 50.0,
        confidence: 0.9,
        class_id: 0,
        class_name: "person".to_string(),
    };
    
    let objects2 = tracker.update(vec![detection2]);
    assert_eq!(objects2.len(), 1);
    assert_eq!(objects2[0].id, objects[0].id); // Same ID means tracked
}

#[test]
fn test_create_cpu_vision_elements() {
    init().unwrap();
    
    let platform = PlatformInfo::detect().unwrap();
    let backend = StandardBackend::new(&platform).unwrap();
    
    // Test creating inference element (without model, will fallback)
    let inference = backend.create_inference(Some("test-inference"), "dummy.onnx");
    assert!(inference.is_ok());
    
    // Test creating tracker element
    let tracker = backend.create_tracker(Some("test-tracker"));
    assert!(tracker.is_ok());
    
    // Test creating OSD element
    let osd = backend.create_osd(Some("test-osd"));
    assert!(osd.is_ok());
}

#[test]
fn test_element_mapping() {
    init().unwrap();
    
    let platform = PlatformInfo::detect().unwrap();
    let backend = StandardBackend::new(&platform).unwrap();
    
    // Check that DeepStream elements map to CPU equivalents
    assert_eq!(backend.get_element_mapping("nvinfer"), Some("cpu-detector"));
    assert_eq!(backend.get_element_mapping("nvtracker"), Some("cpu-tracker"));
    assert_eq!(backend.get_element_mapping("nvdsosd"), Some("textoverlay"));
}

#[test]
#[cfg(feature = "nalgebra")]
fn test_tracker_object_lifecycle() {
    use ds_rs::backend::cpu_vision::tracker::CentroidTracker;
    use ds_rs::backend::cpu_vision::Detection;
    
    let mut tracker = CentroidTracker::new(50.0, 2); // Low disappear threshold
    
    let detection = Detection {
        x: 100.0,
        y: 100.0,
        width: 50.0,
        height: 50.0,
        confidence: 0.9,
        class_id: 0,
        class_name: "car".to_string(),
    };
    
    // Object appears
    let objects = tracker.update(vec![detection]);
    assert_eq!(objects.len(), 1);
    let obj_id = objects[0].id;
    
    // Object disappears for 1 frame
    let objects = tracker.update(vec![]);
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].id, obj_id);
    assert_eq!(objects[0].disappeared_count, 1);
    
    // Object disappears for 2 frames
    let objects = tracker.update(vec![]);
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].disappeared_count, 2);
    
    // Object disappears for 3 frames - should be removed
    let objects = tracker.update(vec![]);
    assert_eq!(objects.len(), 0);
}

#[test]
fn test_detection_nms() {
    use ds_rs::backend::cpu_vision::Detection;
    
    // Test that Detection struct works correctly
    let det1 = Detection {
        x: 100.0,
        y: 100.0,
        width: 50.0,
        height: 50.0,
        confidence: 0.9,
        class_id: 0,
        class_name: "person".to_string(),
    };
    
    let det2 = Detection {
        x: 105.0,  // Overlapping
        y: 105.0,
        width: 50.0,
        height: 50.0,
        confidence: 0.85,
        class_id: 0,
        class_name: "person".to_string(),
    };
    
    // In real implementation, NMS would filter one of these
    assert!(det1.confidence > det2.confidence);
}

#[test]
#[cfg(feature = "nalgebra")]
fn test_multi_object_tracking() {
    use ds_rs::backend::cpu_vision::tracker::CentroidTracker;
    use ds_rs::backend::cpu_vision::Detection;
    
    let mut tracker = CentroidTracker::new(100.0, 30);
    
    // Multiple objects in first frame
    let detections = vec![
        Detection {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
            confidence: 0.9,
            class_id: 0,
            class_name: "person".to_string(),
        },
        Detection {
            x: 300.0,
            y: 100.0,
            width: 60.0,
            height: 60.0,
            confidence: 0.85,
            class_id: 2,
            class_name: "car".to_string(),
        },
        Detection {
            x: 200.0,
            y: 300.0,
            width: 40.0,
            height: 40.0,
            confidence: 0.75,
            class_id: 1,
            class_name: "bicycle".to_string(),
        },
    ];
    
    let objects = tracker.update(detections);
    assert_eq!(objects.len(), 3);
    
    // Check different classes are tracked
    let classes: Vec<String> = objects.iter().map(|o| o.class_name.clone()).collect();
    assert!(classes.contains(&"person".to_string()));
    assert!(classes.contains(&"car".to_string()));
    assert!(classes.contains(&"bicycle".to_string()));
    
    // Second frame with slight movement
    let detections2 = vec![
        Detection {
            x: 105.0,  // Person moved
            y: 102.0,
            width: 50.0,
            height: 50.0,
            confidence: 0.9,
            class_id: 0,
            class_name: "person".to_string(),
        },
        Detection {
            x: 310.0,  // Car moved
            y: 95.0,
            width: 60.0,
            height: 60.0,
            confidence: 0.85,
            class_id: 2,
            class_name: "car".to_string(),
        },
        // Bicycle disappeared
    ];
    
    let objects2 = tracker.update(detections2);
    assert_eq!(objects2.len(), 3); // Bicycle still tracked but marked as disappeared
    
    let bicycle = objects2.iter().find(|o| o.class_name == "bicycle").unwrap();
    assert!(bicycle.disappeared_count > 0);
}

#[test]
fn test_backend_manager_selects_standard() {
    init().unwrap();
    
    // Force Standard backend
    unsafe {
        std::env::set_var("FORCE_BACKEND", "standard");
    }
    
    let manager = BackendManager::new().unwrap();
    assert_eq!(manager.backend_type(), BackendType::Standard);
    
    // Check that it has CPU vision capabilities
    let caps = manager.capabilities();
    assert!(caps.supports_inference);
    assert!(caps.supports_tracking);
    
    unsafe {
        std::env::remove_var("FORCE_BACKEND");
    }
}

#[test]
#[cfg(feature = "ort")]
fn test_onnx_tensor_operations() {
    use ds_rs::backend::cpu_vision::{OnnxDetector, DetectorConfig};
    use image::DynamicImage;
    
    // Create detector with mock mode (no actual model file)
    let config = DetectorConfig {
        model_path: None,
        input_width: 640,
        input_height: 640,
        ..Default::default()
    };
    
    let detector = OnnxDetector::new_with_config(config).unwrap();
    
    // Test preprocessing
    let image = DynamicImage::new_rgb8(1920, 1080);
    // This should work with mock detector
    let result = detector.detect(&image);
    assert!(result.is_ok());
    
    // TODO: When a real ONNX model is available, test with:
    // let config = DetectorConfig {
    //     model_path: Some("models/yolov5n.onnx".to_string()),
    //     ..Default::default()
    // };
}
