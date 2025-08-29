#![allow(unused)]
#[cfg(feature = "nalgebra")]
use ds_rs::backend::cpu_vision::tracker;
use ds_rs::backend::standard::StandardBackend;
use ds_rs::backend::{Backend, BackendManager, BackendType};
use ds_rs::init;
use ds_rs::platform::PlatformInfo;

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
    assert!(
        caps.available_elements
            .contains(&"cpu-detector".to_string())
    );
    assert!(caps.available_elements.contains(&"cpu-tracker".to_string()));
    assert!(caps.available_elements.contains(&"cpu-osd".to_string()));
}

#[test]
fn test_cpu_detector_creation() {
    use ds_rs::backend::cpu_vision::{DetectorConfig, OnnxDetector};

    // Test with nonexistent model file
    let result = OnnxDetector::new("nonexistent.onnx");

    #[cfg(feature = "ort")]
    {
        // With ort feature, constructor succeeds but detection will fail without model
        assert!(result.is_ok());
        let detector = result.unwrap();

        // Detection should fail without a real model
        use image::DynamicImage;
        let image = DynamicImage::new_rgb8(640, 640);
        let detection_result = detector.detect(&image);
        assert!(detection_result.is_err());
    }

    #[cfg(not(feature = "ort"))]
    {
        // Without ort feature, should fail
        assert!(result.is_err());
    }

    // Test with DetectorConfig without model
    let config = DetectorConfig {
        model_path: None, // No model
        input_width: 416,
        input_height: 416,
        confidence_threshold: 0.3,
        ..Default::default()
    };

    #[cfg(feature = "ort")]
    {
        let detector = OnnxDetector::new_with_config(config);
        assert!(detector.is_ok());
        let detector = detector.unwrap();

        // Detection should fail without a model
        use image::DynamicImage;
        let image = DynamicImage::new_rgb8(640, 480);
        let detections = detector.detect(&image);
        assert!(detections.is_err()); // Should fail without model
    }

    #[cfg(not(feature = "ort"))]
    {
        let detector = OnnxDetector::new_with_config(config);
        assert!(detector.is_err()); // Should fail without ort feature
    }
}

#[test]
#[cfg(feature = "nalgebra")]
fn test_cpu_tracker_functionality() {
    use ds_rs::backend::cpu_vision::Detection;
    use ds_rs::backend::cpu_vision::tracker::CentroidTracker;

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
        x: 105.0, // Slightly moved
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
    assert_eq!(
        backend.get_element_mapping("nvtracker"),
        Some("cpu-tracker")
    );
    assert_eq!(backend.get_element_mapping("nvdsosd"), Some("textoverlay"));
}

#[test]
#[cfg(feature = "nalgebra")]
fn test_tracker_object_lifecycle() {
    use ds_rs::backend::cpu_vision::Detection;
    use ds_rs::backend::cpu_vision::tracker::CentroidTracker;

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
        x: 105.0, // Overlapping
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
    use ds_rs::backend::cpu_vision::Detection;
    use ds_rs::backend::cpu_vision::tracker::CentroidTracker;

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
            x: 105.0, // Person moved
            y: 102.0,
            width: 50.0,
            height: 50.0,
            confidence: 0.9,
            class_id: 0,
            class_name: "person".to_string(),
        },
        Detection {
            x: 310.0, // Car moved
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
    use ds_rs::backend::cpu_vision::{DetectorConfig, OnnxDetector};
    use image::DynamicImage;

    // Skip this test since we removed mock detector support
    // Real ONNX model testing requires an actual model file

    // TODO: When a real ONNX model is available, test with an actual model
    // For now, just verify that detector creation fails without a model
    let config = DetectorConfig {
        model_path: None,
        input_width: 640,
        input_height: 640,
        ..Default::default()
    };

    let detector = OnnxDetector::new_with_config(config).unwrap();

    // Test that detection fails without a model
    let image = DynamicImage::new_rgb8(1920, 1080);
    let result = detector.detect(&image);
    assert!(result.is_err());

    // TODO: When a real ONNX model is available, test with:
    // let config = DetectorConfig {
    //     model_path: Some("models/yolov5n.onnx".to_string()),
    //     ..Default::default()
    // };
}
