// ONNX-based object detector supporting multiple YOLO versions
// Supports: YOLOv3-v12, YOLO-RD, and future versions with auto-detection
// Ultralytics (v3-v12): https://docs.ultralytics.com/models/
// Official YOLOv7/v9/RD: https://github.com/WongKinYiu/YOLO (../MultimediaTechLab--YOLO)
// Note: YOLOv10 introduces NMS-free inference, v12 achieves best mAP improvements

use crate::error::{DeepStreamError, Result};
use image::{DynamicImage, imageops::FilterType};
use std::path::Path;

/// Detection result from the model
#[derive(Debug, Clone)]
pub struct Detection {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub class_id: usize,
    pub class_name: String,
}

/// YOLO model version for output format handling
#[derive(Debug, Clone, Copy)]
pub enum YoloVersion {
    V3,     // Output: [1, num_anchors, 85] classic format
    V4,     // Output: Similar to V3 with improvements
    V5,     // Output: [1, 25200, 85] with objectness for 640x640
    V6,     // Output: Similar to V5 (MT-YOLOv6 different format)
    V7,     // Output: Similar to V5 with objectness
    V8,     // Output: [1, 84, 8400] transposed, no objectness  
    V9,     // Output: [1, 84, 8400] similar to V8
    V10,    // Output: NMS-free, one-to-one predictions
    V11,    // Output: Ultralytics model, similar to V8 but optimized  
    V12,    // Output: Latest production model with significant accuracy improvements
    RD,     // YOLO-RD: Retriever-Dictionary variant
    Auto,   // Auto-detect based on output shape
}

/// ONNX-based object detector for CPU inference
pub struct OnnxDetector {
    #[cfg(feature = "ort")]
    session: Option<ort::Session>,
    #[cfg(feature = "ort")]
    allocator: Option<*mut ort::sys::OrtAllocator>,
    input_width: u32,
    input_height: u32,
    confidence_threshold: f32,
    nms_threshold: f32,
    class_names: Vec<String>,
    yolo_version: YoloVersion,
}

impl OnnxDetector {
    /// Create a new ONNX detector with the specified model
    pub fn new(model_path: &str) -> Result<Self> {
        #[cfg(feature = "ort")]
        {
            // Try to load ONNX model if the feature is enabled
            if !Path::new(model_path).exists() {
                return Err(DeepStreamError::Configuration(
                    format!("Model file not found: {}", model_path)
                ));
            }
            
            let session = Self::load_onnx_model(model_path)?;
            let allocator = session.allocator();
            log::info!("Loaded ONNX model from: {}", model_path);
            
            return Ok(Self {
                session: Some(session),
                allocator: Some(allocator),
                input_width: 640,
                input_height: 640,
                confidence_threshold: 0.5,
                nms_threshold: 0.4,
                class_names: Self::default_class_names(),
                yolo_version: YoloVersion::Auto,
            });
        }
        
        #[cfg(not(feature = "ort"))]
        {
            Err(DeepStreamError::Configuration(
                "ONNX Runtime feature not enabled. Build with --features ort".to_string()
            ))
        }
    }
    
    #[cfg(feature = "ort")]
    fn load_onnx_model(model_path: &str) -> Result<ort::Session> {
        use ort::{Environment, SessionBuilder, GraphOptimizationLevel};
        use std::sync::Arc;
        
        // Create environment first
        let environment = Arc::new(Environment::builder()
            .with_name("onnx_detector")
            .build()
            .map_err(|e| DeepStreamError::Configuration(
                format!("Failed to create ONNX environment: {}", e)
            ))?);
        
        // Create session with the environment
        let session = SessionBuilder::new(&environment)
            .map_err(|e| DeepStreamError::Configuration(
                format!("Failed to create session builder: {}", e)
            ))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| DeepStreamError::Configuration(
                format!("Failed to set optimization level: {}", e)
            ))?
            .with_intra_threads(4)
            .map_err(|e| DeepStreamError::Configuration(
                format!("Failed to set intra threads: {}", e)
            ))?
            .with_model_from_file(model_path)
            .map_err(|e| DeepStreamError::Configuration(
                format!("Failed to load model from file: {}", e)
            ))?;
            
        Ok(session)
    }
    
    /// Perform detection on an image
    pub fn detect(&self, image: &DynamicImage) -> Result<Vec<Detection>> {
        #[cfg(feature = "ort")]
        {
            use ndarray::{Array, CowArray, IxDyn};
            use ort::Value;
            
            let session = self.session.as_ref().ok_or_else(|| DeepStreamError::Configuration(
                "ONNX session not initialized".to_string()
            ))?;
            
            let allocator = self.allocator.ok_or_else(|| DeepStreamError::Configuration(
                "ONNX allocator not initialized".to_string()
            ))?;
            
            // Preprocess image
            let input_tensor = self.preprocess_image(image)?;
            
            // Run inference
            log::debug!("Running ONNX inference on {}x{} image", image.width(), image.height());
            
            // Create ndarray with correct shape for YOLO (batch, channels, height, width)
            let shape = vec![1, 3, self.input_height as usize, self.input_width as usize];
            let array = Array::from_shape_vec(shape.clone(), input_tensor)
                .map_err(|e| DeepStreamError::Configuration(
                    format!("Failed to create ndarray: {}", e)
                ))?;
            
            // Convert to CowArray with dynamic dimensions
            let cow_array: CowArray<f32, IxDyn> = CowArray::from(array.into_dyn());
            
            // Create Value using the allocator
            let input_value = Value::from_array(allocator, &cow_array)
                .map_err(|e| DeepStreamError::Configuration(
                    format!("Failed to create ORT value: {}", e)
                ))?;
            
            // Run the model
            let outputs: Vec<Value> = session.run(vec![input_value])
                .map_err(|e| DeepStreamError::Configuration(
                    format!("Failed to run ONNX inference: {}", e)
                ))?;
            
            // Extract output tensor using try_extract
            let output_tensor: ort::tensor::OrtOwnedTensor<f32, _> = outputs[0].try_extract()
                .map_err(|e| DeepStreamError::Configuration(
                    format!("Failed to extract output tensor: {}", e)
                ))?;
            
            // Get view and convert to Vec
            let output_view = output_tensor.view();
            let output: Vec<f32> = output_view.iter().cloned().collect();
            
            // Postprocess outputs
            return self.postprocess_outputs(&output, image.width(), image.height());
        }
        
        #[cfg(not(feature = "ort"))]
        {
            Err(DeepStreamError::Configuration(
                "ONNX Runtime feature not enabled".to_string()
            ))
        }
    }
    
    /// Preprocess image for model input
    fn preprocess_image(&self, image: &DynamicImage) -> Result<Vec<f32>> {
        // Resize image to model input size
        let resized = image.resize_exact(
            self.input_width,
            self.input_height,
            FilterType::Triangle
        );
        
        // Convert to RGB if needed
        let rgb_image = resized.to_rgb8();
        
        // Create tensor in CHW format (Channels, Height, Width) for YOLO
        let mut tensor = Vec::with_capacity((3 * self.input_width * self.input_height) as usize);
        
        // Normalize and arrange in CHW format
        // YOLO typically expects values normalized to [0, 1]
        for channel in 0..3 {
            for y in 0..self.input_height {
                for x in 0..self.input_width {
                    let pixel = rgb_image.get_pixel(x, y);
                    let value = pixel[channel as usize] as f32 / 255.0;
                    tensor.push(value);
                }
            }
        }
        
        log::trace!("Preprocessed image from {}x{} to {}x{} tensor", 
                   image.width(), image.height(), 
                   self.input_width, self.input_height);
        
        Ok(tensor)
    }
    
    /// Process model outputs to detections
    fn postprocess_outputs(&self, outputs: &[f32], img_width: u32, img_height: u32) -> Result<Vec<Detection>> {
        // Auto-detect YOLO version based on output shape
        let version = match self.yolo_version {
            YoloVersion::Auto => self.detect_yolo_version(outputs),
            v => v,
        };
        
        log::debug!("Processing outputs with YOLO version: {:?}", version);
        
        match version {
            // Classic format with objectness (v3-v7)
            YoloVersion::V3 | YoloVersion::V4 | YoloVersion::V5 | 
            YoloVersion::V6 | YoloVersion::V7 => {
                self.postprocess_yolov5(outputs, img_width, img_height)
            },
            // Modern format without objectness (v8-v12)
            YoloVersion::V8 | YoloVersion::V9 | YoloVersion::V11 | 
            YoloVersion::V12 | YoloVersion::RD => {
                self.postprocess_yolov8(outputs, img_width, img_height)
            },
            // Special handling for v10 (NMS-free)
            YoloVersion::V10 => {
                // V10 uses one-to-one predictions, may need special handling
                // For now, treat similar to v8 but log the difference
                log::info!("Processing YOLOv10 with NMS-free design");
                self.postprocess_yolov8(outputs, img_width, img_height)
            },
            YoloVersion::Auto => {
                // Fallback to V5 if auto-detection somehow fails
                self.postprocess_yolov5(outputs, img_width, img_height)
            }
        }
    }
    
    /// Detect YOLO version based on output tensor shape
    fn detect_yolo_version(&self, outputs: &[f32]) -> YoloVersion {
        let len = outputs.len();
        
        // Common output patterns:
        // YOLOv3-v7: [1, num_anchors, 85] where 85 = 4 bbox + 1 objectness + 80 classes
        // YOLOv8-v11: [1, 84, num_anchors] where 84 = 4 bbox + 80 classes (no objectness)
        // YOLOv10: May have different format due to NMS-free design
        
        // Check for v8+ transposed format (84 values per anchor)
        if len % 84 == 0 {
            let num_anchors = len / 84;
            if num_anchors > 1000 && num_anchors < 10000 {
                // Likely v8, v9, v11 format
                log::debug!("Detected YOLOv8+ format with {} anchors", num_anchors);
                return YoloVersion::V8;
            }
        }
        
        // Check for v3-v7 format (85 values per anchor)
        if len % 85 == 0 {
            let num_anchors = len / 85;
            if num_anchors > 1000 {
                // Likely v3-v7 format
                log::debug!("Detected YOLOv3-v7 format with {} anchors", num_anchors);
                return YoloVersion::V5; // Use v5 processing for v3-v7
            }
        }
        
        // Check for smaller models or different input sizes
        if len % 85 == 0 || len % 84 == 0 {
            log::info!("Detected YOLO format with {} total values", len);
            // Default to newer format for smaller outputs
            return if len % 84 == 0 { YoloVersion::V8 } else { YoloVersion::V5 };
        }
        
        // Unknown format
        log::warn!("Unknown YOLO output format with {} elements, defaulting to V5", len);
        YoloVersion::V5
    }
    
    /// Process YOLOv5 outputs
    fn postprocess_yolov5(&self, outputs: &[f32], img_width: u32, img_height: u32) -> Result<Vec<Detection>> {
        let mut detections = Vec::new();
        
        // YOLOv5 output format: [batch_size, num_anchors, 85]
        // where 85 = cx, cy, w, h, objectness, class_scores[80]
        let num_classes = 80; // COCO dataset
        let output_size = 85; // 4 bbox + 1 objectness + 80 classes
        let num_anchors = outputs.len() / output_size;
        
        // Scale factors to convert from model coordinates to image coordinates
        let x_scale = img_width as f32 / self.input_width as f32;
        let y_scale = img_height as f32 / self.input_height as f32;
        
        // Process each anchor/detection
        for i in 0..num_anchors {
            let offset = i * output_size;
            
            // Extract bbox and scores
            let cx = outputs[offset];
            let cy = outputs[offset + 1];
            let w = outputs[offset + 2];
            let h = outputs[offset + 3];
            let objectness = outputs[offset + 4];
            
            // Skip low confidence detections
            if objectness < self.confidence_threshold {
                continue;
            }
            
            // Find best class
            let mut max_class_score = 0.0;
            let mut best_class_id = 0;
            
            for class_id in 0..num_classes {
                let class_score = outputs[offset + 5 + class_id];
                if class_score > max_class_score {
                    max_class_score = class_score;
                    best_class_id = class_id;
                }
            }
            
            // Combined confidence
            let confidence = objectness * max_class_score;
            
            if confidence >= self.confidence_threshold {
                // Convert from center format to top-left format
                // and scale to original image size
                let x = ((cx - w / 2.0) * x_scale).max(0.0);
                let y = ((cy - h / 2.0) * y_scale).max(0.0);
                let width = (w * x_scale).min(img_width as f32 - x);
                let height = (h * y_scale).min(img_height as f32 - y);
                
                detections.push(Detection {
                    x,
                    y,
                    width,
                    height,
                    confidence,
                    class_id: best_class_id,
                    class_name: self.class_names.get(best_class_id)
                        .unwrap_or(&"unknown".to_string())
                        .clone(),
                });
            }
        }
        
        // Apply Non-Maximum Suppression
        let filtered_detections = self.apply_nms(detections);
        
        log::debug!("YOLOv5: Postprocessed {} anchors, {} detections after NMS", 
                   num_anchors, filtered_detections.len());
        
        Ok(filtered_detections)
    }
    
    /// Process YOLOv8/v9/v11/v12 outputs  
    /// Note: v12 achieves mAP of 40.6-55.2 depending on model size (n/s/m/l/x)
    fn postprocess_yolov8(&self, outputs: &[f32], img_width: u32, img_height: u32) -> Result<Vec<Detection>> {
        let mut detections = Vec::new();
        
        // YOLOv8/v9 output format: [batch_size, 84, 8400]
        // where 84 = cx, cy, w, h, class_scores[80] (no objectness)
        // Note: Output is transposed compared to v5
        let num_classes = 80;
        let num_values = 84; // 4 bbox + 80 classes
        let num_anchors = outputs.len() / num_values;
        
        // Scale factors
        let x_scale = img_width as f32 / self.input_width as f32;
        let y_scale = img_height as f32 / self.input_height as f32;
        
        // Process transposed format
        for anchor_idx in 0..num_anchors {
            // In v8/v9, data is arranged as [84, 8400]
            // So for each anchor, we need to gather values across the first dimension
            let cx = outputs[0 * num_anchors + anchor_idx];
            let cy = outputs[1 * num_anchors + anchor_idx];
            let w = outputs[2 * num_anchors + anchor_idx];
            let h = outputs[3 * num_anchors + anchor_idx];
            
            // Find best class (no objectness in v8/v9)
            let mut max_class_score = 0.0;
            let mut best_class_id = 0;
            
            for class_id in 0..num_classes {
                let class_score = outputs[(4 + class_id) * num_anchors + anchor_idx];
                if class_score > max_class_score {
                    max_class_score = class_score;
                    best_class_id = class_id;
                }
            }
            
            // Use class score directly as confidence (no objectness)
            let confidence = max_class_score;
            
            if confidence >= self.confidence_threshold {
                // Convert and scale
                let x = ((cx - w / 2.0) * x_scale).max(0.0);
                let y = ((cy - h / 2.0) * y_scale).max(0.0);
                let width = (w * x_scale).min(img_width as f32 - x);
                let height = (h * y_scale).min(img_height as f32 - y);
                
                detections.push(Detection {
                    x,
                    y,
                    width,
                    height,
                    confidence,
                    class_id: best_class_id,
                    class_name: self.class_names.get(best_class_id)
                        .unwrap_or(&"unknown".to_string())
                        .clone(),
                });
            }
        }
        
        // Apply NMS
        let filtered_detections = self.apply_nms(detections);
        
        log::debug!("YOLOv8/v9: Postprocessed {} anchors, {} detections after NMS", 
                   num_anchors, filtered_detections.len());
        
        Ok(filtered_detections)
    }
    
    /// Apply Non-Maximum Suppression
    fn apply_nms(&self, mut detections: Vec<Detection>) -> Vec<Detection> {
        detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        let mut keep = Vec::new();
        
        while !detections.is_empty() {
            let current = detections.remove(0);
            keep.push(current.clone());
            
            detections.retain(|det| {
                if det.class_id != current.class_id {
                    return true;
                }
                
                let iou = self.calculate_iou(&current, det);
                iou < self.nms_threshold
            });
        }
        
        keep
    }
    
    /// Calculate Intersection over Union
    fn calculate_iou(&self, a: &Detection, b: &Detection) -> f32 {
        let x1 = a.x.max(b.x);
        let y1 = a.y.max(b.y);
        let x2 = (a.x + a.width).min(b.x + b.width);
        let y2 = (a.y + a.height).min(b.y + b.height);
        
        if x2 < x1 || y2 < y1 {
            return 0.0;
        }
        
        let intersection = (x2 - x1) * (y2 - y1);
        let area_a = a.width * a.height;
        let area_b = b.width * b.height;
        let union = area_a + area_b - intersection;
        
        intersection / union
    }
    
    /// Create mock YOLO output for testing
    fn create_mock_yolo_output(&self) -> Vec<f32> {
        let num_anchors = 25200; // Typical for 640x640 YOLOv5
        let output_size = 85; // 4 bbox + 1 objectness + 80 classes
        let mut outputs = vec![0.0; num_anchors * output_size];
        
        // Add a few mock detections
        // Detection 1: Person at center
        outputs[0] = 320.0;  // cx
        outputs[1] = 320.0;  // cy
        outputs[2] = 100.0;  // width
        outputs[3] = 200.0;  // height
        outputs[4] = 0.9;    // objectness
        outputs[5] = 0.95;   // person class score
        
        // Detection 2: Car
        outputs[85] = 200.0;  // cx
        outputs[86] = 400.0;  // cy
        outputs[87] = 150.0;  // width
        outputs[88] = 80.0;   // height
        outputs[89] = 0.85;   // objectness
        outputs[90 + 2] = 0.9; // car class score
        
        outputs
    }
    
    /// Get default COCO class names
    fn default_class_names() -> Vec<String> {
        vec![
            "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck",
            "boat", "traffic light", "fire hydrant", "stop sign", "parking meter", "bench",
            "bird", "cat", "dog", "horse", "sheep", "cow", "elephant", "bear", "zebra",
            "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee",
            "skis", "snowboard", "sports ball", "kite", "baseball bat", "baseball glove",
            "skateboard", "surfboard", "tennis racket", "bottle", "wine glass", "cup",
            "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange",
            "broccoli", "carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch",
            "potted plant", "bed", "dining table", "toilet", "tv", "laptop", "mouse",
            "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink",
            "refrigerator", "book", "clock", "vase", "scissors", "teddy bear", "hair drier",
            "toothbrush"
        ].iter().map(|s| s.to_string()).collect()
    }
    
    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold;
    }
    
    pub fn set_nms_threshold(&mut self, threshold: f32) {
        self.nms_threshold = threshold;
    }
    
    /// Set the YOLO version for output processing
    pub fn set_yolo_version(&mut self, version: YoloVersion) {
        self.yolo_version = version;
        log::info!("Set YOLO version to: {:?}", version);
    }
    
    /// Create a mock detector for testing without an actual model
    #[cfg(test)]
    pub fn new_mock() -> Self {
        Self {
            #[cfg(feature = "ort")]
            session: None,
            #[cfg(feature = "ort")]
            allocator: None,
            input_width: 640,
            input_height: 640,
            confidence_threshold: 0.5,
            nms_threshold: 0.4,
            class_names: Self::default_class_names(),
            yolo_version: YoloVersion::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detection_creation() {
        let detection = Detection {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
            confidence: 0.9,
            class_id: 0,
            class_name: "person".to_string(),
        };
        
        assert_eq!(detection.class_name, "person");
        assert_eq!(detection.confidence, 0.9);
    }
    
    #[test]
    fn test_detector_creation() {
        let detector = OnnxDetector::new_mock();
        assert_eq!(detector.input_width, 640);
        assert_eq!(detector.input_height, 640);
    }
    
    #[test]
    fn test_iou_calculation() {
        let detector = OnnxDetector::new_mock();
        
        let det1 = Detection {
            x: 0.0, y: 0.0, width: 10.0, height: 10.0,
            confidence: 0.9, class_id: 0, class_name: "test".to_string(),
        };
        
        let det2 = Detection {
            x: 5.0, y: 5.0, width: 10.0, height: 10.0,
            confidence: 0.8, class_id: 0, class_name: "test".to_string(),
        };
        
        let iou = detector.calculate_iou(&det1, &det2);
        assert!(iou > 0.0);
        assert!(iou < 1.0);
    }
}