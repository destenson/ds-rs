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

/// ONNX-based object detector for CPU inference
pub struct OnnxDetector {
    #[cfg(feature = "ort")]
    session: Option<ort::Session>,
    input_width: u32,
    input_height: u32,
    confidence_threshold: f32,
    nms_threshold: f32,
    class_names: Vec<String>,
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
            log::info!("Loaded ONNX model from: {}", model_path);
            
            return Ok(Self {
                session: Some(session),
                input_width: 640,
                input_height: 640,
                confidence_threshold: 0.5,
                nms_threshold: 0.4,
                class_names: Self::default_class_names(),
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
            let session = self.session.as_ref().ok_or_else(|| DeepStreamError::Configuration(
                "ONNX session not initialized".to_string()
            ))?;
            
            // Preprocess image
            let input_tensor = self.preprocess_image(image)?;
            
            // Run inference
            log::debug!("Running ONNX inference on {}x{} image", image.width(), image.height());
            
            // Create an OrtOwnedTensor from raw data
            use ort::tensor::OrtOwnedTensor;
            
            let shape = vec![1, 3, self.input_height as usize, self.input_width as usize];
            let input_ort_tensor = OrtOwnedTensor::from_shape_vec(
                shape,
                input_tensor
            ).map_err(|e| DeepStreamError::Configuration(
                format!("Failed to create ORT tensor: {}", e)
            ))?;
            
            // Run the model
            let outputs = session.run(vec![input_ort_tensor])
                .map_err(|e| DeepStreamError::Configuration(
                    format!("Failed to run ONNX inference: {}", e)
                ))?;
            
            // Extract output tensor
            // The output is already an OrtOwnedTensor
            let output_tensor = &outputs[0];
            let output = output_tensor.as_slice()
                .ok_or_else(|| DeepStreamError::Configuration(
                    "Failed to get output tensor slice".to_string()
                ))?
                .to_vec();
            
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
        
        log::debug!("Postprocessed {} detections, {} after NMS", 
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
        let detector = OnnxDetector::new("dummy.onnx").unwrap();
        assert_eq!(detector.input_width, 640);
        assert_eq!(detector.input_height, 640);
    }
    
    #[test]
    fn test_iou_calculation() {
        let detector = OnnxDetector::new("dummy.onnx").unwrap();
        
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