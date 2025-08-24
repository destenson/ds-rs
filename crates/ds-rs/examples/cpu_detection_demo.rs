//! CPU-based object detection demo
//! 
//! This example demonstrates the ONNX-based CPU detector working with real YOLO models.
//! It creates a test image, runs detection, and prints the results.

use ds_rs::backend::cpu_vision::detector::{OnnxDetector, DetectorConfig, YoloVersion};
use image::{DynamicImage, RgbImage};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with debug level to see model info
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    println!("CPU Object Detection Demo");
    println!("=============================");
    
    // Check for model file
    let model_paths = [
        "models/yolov5n.onnx",
        "crates/ds-rs/models/yolov5n.onnx", 
        "../models/yolov5n.onnx",
    ];
    
    let model_path = model_paths.iter()
        .find(|path| Path::new(path).exists())
        .copied();
    
    match model_path {
        Some(path) => {
            println!("Using ONNX model: {}", path);
            println!("Testing real ONNX detection...");
            
            match test_real_detection(path) {
                Ok(()) => println!("Real ONNX detection completed successfully"),
                Err(e) => {
                    println!("Real detection failed: {}", e);
                    println!("This may be due to model format (float16 vs float32) or compatibility issues");
                    println!("\nTo fix this:");
                    println!("1. Run: python export_yolov5n_float32.py");
                    println!("2. Or install ultralytics and run: yolo export model=yolov5n.pt format=onnx half=False");
                    println!("\nFalling back to mock detection to show the pipeline works:");
                    test_mock_detection();
                }
            }
        },
        None => {
            println!("No ONNX model found, using mock detection");
            println!("To use real ONNX inference:");
            println!("1. Run: python export_yolov5n_float32.py");
            println!("2. Or place a compatible yolov5n.onnx file in models/ directory");
            println!("");
            test_mock_detection();
        }
    }
    
    Ok(())
}

fn test_real_detection(model_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting Real ONNX Detection");
    println!("--------------------------------");
    
    // Create detector configuration
    let config = DetectorConfig {
        model_path: Some(model_path.to_string()),
        input_width: 640,
        input_height: 640,
        confidence_threshold: 0.3,  // Lower threshold to see more detections
        nms_threshold: 0.4,
        num_threads: 4,
        yolo_version: YoloVersion::Auto,
        class_names: None,  // Use default COCO classes
    };
    
    println!("Creating detector with config:");
    println!("   Input size: {}x{}", config.input_width, config.input_height);
    println!("   Confidence threshold: {}", config.confidence_threshold);
    println!("   NMS threshold: {}", config.nms_threshold);
    println!("   Threads: {}", config.num_threads);
    
    let detector = OnnxDetector::new_with_config(config)?;
    
    // Create test images
    let images = vec![
        ("Test Pattern", create_test_pattern(640, 640)),
        ("Car-like Rectangle", create_car_pattern(640, 640)),
        ("Person-like Shape", create_person_pattern(640, 640)),
    ];
    
    for (name, image) in images {
        println!("\n{}", name);
        println!("   {}", "-".repeat(name.len()));
        
        match detector.detect(&image) {
            Ok(detections) => {
                println!("   Found {} detections:", detections.len());
                
                for (i, detection) in detections.iter().enumerate() {
                    println!("   Detection {}: {} (confidence: {:.2})", 
                           i + 1, detection.class_name, detection.confidence);
                    println!("     Bounding box: ({:.1}, {:.1}) {}x{}", 
                           detection.x, detection.y, detection.width, detection.height);
                }
                
                if detections.is_empty() {
                    println!("   (No objects detected above confidence threshold)");
                }
            },
            Err(e) => {
                println!("   Detection failed: {}", e);
            }
        }
    }
    
    Ok(())
}

fn test_mock_detection() {
    println!("\nTesting Mock Detection");
    println!("-------------------------");
    
    let detector = OnnxDetector::new_mock();
    let test_image = create_test_pattern(640, 640);
    
    match detector.detect(&test_image) {
        Ok(detections) => {
            println!("Mock detector found {} detections:", detections.len());
            
            for (i, detection) in detections.iter().enumerate() {
                println!("   Detection {}: {} (confidence: {:.2})", 
                       i + 1, detection.class_name, detection.confidence);
                println!("     Bounding box: ({:.1}, {:.1}) {}x{}", 
                       detection.x, detection.y, detection.width, detection.height);
            }
        },
        Err(e) => {
            println!("Mock detection failed: {}", e);
        }
    }
}

/// Create a test pattern image
fn create_test_pattern(width: u32, height: u32) -> DynamicImage {
    let mut img_data = vec![64u8; (width * height * 3) as usize]; // Dark gray background
    
    // Add some geometric shapes that might be detected
    
    // Rectangle in the center (could be detected as various objects)
    let rect_x = width / 3;
    let rect_y = height / 3;  
    let rect_w = width / 3;
    let rect_h = height / 4;
    
    for y in rect_y..(rect_y + rect_h) {
        for x in rect_x..(rect_x + rect_w) {
            let idx = ((y * width + x) * 3) as usize;
            if idx + 2 < img_data.len() {
                img_data[idx] = 200;     // R
                img_data[idx + 1] = 200; // G
                img_data[idx + 2] = 200; // B (light gray rectangle)
            }
        }
    }
    
    // Add some noise/texture
    for y in (0..height).step_by(10) {
        for x in (0..width).step_by(10) {
            let idx = ((y * width + x) * 3) as usize;
            if idx + 2 < img_data.len() {
                img_data[idx] = 128;
                img_data[idx + 1] = 128;
                img_data[idx + 2] = 128;
            }
        }
    }
    
    DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, img_data).unwrap())
}

/// Create a car-like rectangular pattern
fn create_car_pattern(width: u32, height: u32) -> DynamicImage {
    let mut img_data = vec![50u8; (width * height * 3) as usize]; // Dark background
    
    // Main car body (rectangular)
    let car_x = width / 4;
    let car_y = height / 2;
    let car_w = width / 2;
    let car_h = height / 8;
    
    for y in car_y..(car_y + car_h) {
        for x in car_x..(car_x + car_w) {
            let idx = ((y * width + x) * 3) as usize;
            if idx + 2 < img_data.len() {
                img_data[idx] = 100;     // R - blue-ish car
                img_data[idx + 1] = 150; // G
                img_data[idx + 2] = 200; // B
            }
        }
    }
    
    // Add "wheels" (small dark rectangles)
    let wheel_size = car_h / 3;
    let wheel_y = car_y + car_h - wheel_size / 2;
    
    for wheel_x in [car_x + car_w / 4, car_x + 3 * car_w / 4] {
        for y in wheel_y..(wheel_y + wheel_size) {
            for x in wheel_x..(wheel_x + wheel_size) {
                let idx = ((y * width + x) * 3) as usize;
                if idx + 2 < img_data.len() {
                    img_data[idx] = 20;      // R - dark wheels
                    img_data[idx + 1] = 20;  // G  
                    img_data[idx + 2] = 20;  // B
                }
            }
        }
    }
    
    DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, img_data).unwrap())
}

/// Create a person-like vertical pattern  
fn create_person_pattern(width: u32, height: u32) -> DynamicImage {
    let mut img_data = vec![80u8; (width * height * 3) as usize]; // Medium background
    
    // Person body (tall rectangle)
    let person_x = width / 2 - width / 16;
    let person_y = height / 3;
    let person_w = width / 8;
    let person_h = height / 2;
    
    for y in person_y..(person_y + person_h) {
        for x in person_x..(person_x + person_w) {
            let idx = ((y * width + x) * 3) as usize;
            if idx + 2 < img_data.len() {
                img_data[idx] = 180;     // R - flesh-toned
                img_data[idx + 1] = 140; // G
                img_data[idx + 2] = 120; // B
            }
        }
    }
    
    // "Head" (circle approximation)
    let head_center_x = person_x + person_w / 2;
    let head_center_y = person_y - person_h / 8;
    let head_radius = person_w / 2;
    
    for y in (head_center_y - head_radius)..(head_center_y + head_radius) {
        for x in (head_center_x - head_radius)..(head_center_x + head_radius) {
            if y < height && x < width {
                let dx = x as i32 - head_center_x as i32;
                let dy = y as i32 - head_center_y as i32;
                if dx * dx + dy * dy <= (head_radius as i32 * head_radius as i32) {
                    let idx = ((y * width + x) * 3) as usize;
                    if idx + 2 < img_data.len() {
                        img_data[idx] = 200;     // R - lighter head
                        img_data[idx + 1] = 160; // G  
                        img_data[idx + 2] = 140; // B
                    }
                }
            }
        }
    }
    
    DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, img_data).unwrap())
}