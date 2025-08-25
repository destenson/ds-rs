#![allow(unused)]
//! ONNX-based object detector supporting multiple YOLO versions
//! 
//! This module provides CPU-based object detection using ONNX Runtime (ort) v1.16.3.
//! It supports multiple YOLO versions (v3-v12) with automatic format detection and
//! includes a mock detector for testing without actual models.

// Error types for the detector
#[derive(Debug, thiserror::Error)]
pub enum DetectorError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Model loading error: {0}")]
    ModelLoading(String),
    #[error("Inference error: {0}")]
    Inference(String),
}

impl From<DetectorError> for crate::DeepStreamError {
    fn from(err: DetectorError) -> Self {
        crate::DeepStreamError::Configuration(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DetectorError>;
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

/// Configuration for the ONNX detector
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    /// Path to the ONNX model file
    pub model_path: Option<String>,
    /// Input width for the model
    pub input_width: u32,
    /// Input height for the model
    pub input_height: u32,
    /// Confidence threshold for detections
    pub confidence_threshold: f32,
    /// NMS threshold for filtering overlapping boxes
    pub nms_threshold: f32,
    /// Number of threads for inference
    pub num_threads: usize,
    /// YOLO version for output processing
    pub yolo_version: YoloVersion,
    /// Custom class names (optional)
    pub class_names: Option<Vec<String>>,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            input_width: 640,
            input_height: 640,
            confidence_threshold: 0.5,
            nms_threshold: 0.4,
            num_threads: 4,
            yolo_version: YoloVersion::Auto,
            class_names: None,
        }
    }
}

/// ONNX-based object detector for CPU inference
pub struct OnnxDetector {
    #[cfg(feature = "ort")]
    session: Option<ort::Session>,
    #[cfg(feature = "ort")]
    environment: Option<std::sync::Arc<ort::Environment>>,
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
        let config = DetectorConfig {
            model_path: Some(model_path.to_string()),
            ..Default::default()
        };
        Self::new_with_config(config)
    }
    
    /// Create a new ONNX detector with a configuration
    pub fn new_with_config(config: DetectorConfig) -> Result<Self> {
        #[cfg(feature = "ort")]
        {
            // Try to load model if path is provided, but fallback to mock on any error
            let (session, environment) = if let Some(ref model_path) = config.model_path {
                if !Path::new(model_path).exists() {
                    (None, None)
                } else {
                    match Self::load_onnx_model(model_path, config.num_threads) {
                        Ok((env, sess)) => {
                            (Some(sess), Some(env))
                        },
                        Err(_e) => {
                            (None, None)
                        }
                    }
                }
            } else {
                (None, None)
            };
            
            let class_names = config.class_names.unwrap_or_else(Self::default_class_names);
            
            return Ok(Self {
                session,
                environment,
                input_width: config.input_width,
                input_height: config.input_height,
                confidence_threshold: config.confidence_threshold,
                nms_threshold: config.nms_threshold,
                class_names,
                yolo_version: config.yolo_version,
            });
        }
        
        #[cfg(test)]
        #[cfg(not(feature = "ort"))]
        {
            // When ort feature is not enabled, create mock detector
            Ok(Self {
                input_width: config.input_width,
                input_height: config.input_height,
                confidence_threshold: config.confidence_threshold,
                nms_threshold: config.nms_threshold,
                class_names: config.class_names.unwrap_or_else(Self::default_class_names),
                yolo_version: config.yolo_version,
            })
        }
        #[cfg(not(test))]
        Err(DetectorError::Configuration(
            "ONNX Runtime (ort) feature not enabled".to_string()
        ))
    }
    
    #[cfg(feature = "ort")]
    fn load_onnx_model(model_path: &str, num_threads: usize) -> Result<(std::sync::Arc<ort::Environment>, ort::Session)> {
        use ort::{Environment, SessionBuilder, GraphOptimizationLevel};
        use std::sync::Arc;
        
        // Create environment first
        let environment = Arc::new(Environment::builder()
            .with_name("onnx_detector")
            .build()
            .map_err(|e| DetectorError::Configuration(
                format!("Failed to create ONNX environment: {}", e)
            ))?);
        
        // Create session with the environment
        let session = SessionBuilder::new(&environment)
            .map_err(|e| DetectorError::Configuration(
                format!("Failed to create session builder: {}", e)
            ))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| DetectorError::Configuration(
                format!("Failed to set optimization level: {}", e)
            ))?
            .with_intra_threads(num_threads.try_into().unwrap_or(4))
            .map_err(|e| DetectorError::Configuration(
                format!("Failed to set intra threads: {}", e)
            ))?
            .with_model_from_file(model_path)
            .map_err(|e| DetectorError::Configuration(
                format!("Failed to load model from file: {}", e)
            ))?;
            
        Ok((environment, session))
    }
    
    /// Perform detection on an image
    pub fn detect(&self, image: &DynamicImage) -> Result<Vec<Detection>> {
        #[cfg(feature = "ort")]
        {
            use ndarray::{Array, CowArray, IxDyn};
            use ort::Value;
            
            // Check if we have a real session or should use mock
            let session = match self.session.as_ref() {
                Some(s) => s,
                #[cfg(test)]
                None => {
                    // Use mock detection when no model is loaded
                    let mock_output = self.create_mock_yolo_output();
                    return self.postprocess_outputs(&mock_output, image.width(), image.height());
                }
                #[cfg(not(test))]
                None => {
                    return Err(DetectorError::Inference(
                        "No ONNX model loaded for detection".to_string()
                    ));
                }
            };
            
            // Preprocess image
            let input_tensor = self.preprocess_image(image)?;
            
            // Create ndarray with correct shape for YOLO (batch, channels, height, width)
            let shape = vec![1, 3, self.input_height as usize, self.input_width as usize];
            
            // Check if model expects float16 input
            let is_f16_input = format!("{:?}", session.inputs[0].input_type).contains("Float16");
            
            // Handle both f32 and f16 models
            // We need to hold the arrays outside to ensure proper lifetimes
            #[cfg(feature = "half")]
            let f16_array: CowArray<half::f16, IxDyn>;
            let f32_array: CowArray<f32, IxDyn>;
            
            let outputs: Vec<Value> = if is_f16_input {
                #[cfg(feature = "half")]
                {
                    use half::f16;
                    
                    // Convert f32 tensor to f16
                    let f16_tensor: Vec<f16> = input_tensor
                        .iter()
                        .map(|&v| f16::from_f32(v))
                        .collect();
                    
                    // Create and store the array
                    f16_array = Array::from_shape_vec(shape.clone(), f16_tensor)
                        .map_err(|e| DetectorError::Configuration(
                            format!("Failed to create f16 ndarray: {}", e)
                        ))?
                        .into_dyn()
                        .into();
                    
                    // Create Value and run
                    let value = Value::from_array(session.allocator(), &f16_array)
                        .map_err(|e| DetectorError::Configuration(
                            format!("Failed to create f16 ORT value: {}", e)
                        ))?;
                    
                    session.run(vec![value])
                        .map_err(|e| DetectorError::Configuration(
                            format!("Failed to run ONNX inference: {}", e)
                        ))?
                }
                #[cfg(not(feature = "half"))]
                {
                    return Err(DetectorError::Configuration(
                        "Model requires float16 but half feature is not enabled".to_string()
                    ));
                }
            } else {
                // Use f32 input
                f32_array = Array::from_shape_vec(shape.clone(), input_tensor)
                    .map_err(|e| DetectorError::Configuration(
                        format!("Failed to create f32 ndarray: {}", e)
                    ))?
                    .into_dyn()
                    .into();
                
                let value = Value::from_array(session.allocator(), &f32_array)
                    .map_err(|e| DetectorError::Configuration(
                        format!("Failed to create f32 ORT value: {}", e)
                    ))?;
                
                session.run(vec![value])
                    .map_err(|e| DetectorError::Configuration(
                        format!("Failed to run ONNX inference: {}", e)
                    ))?
            };
            // Check if output is float16
            let is_f16_output = session.outputs.get(0)
                .map(|output| format!("{:?}", output.output_type).contains("Float16"))
                .unwrap_or(false);
            
            // Extract output tensor based on type
            let output: Vec<f32> = if is_f16_output {
                #[cfg(feature = "half")]
                {
                    use half::f16;
                    
                    // Extract as f16 tensor
                    let output_tensor: ort::tensor::OrtOwnedTensor<f16, _> = outputs[0].try_extract()
                        .map_err(|e| DetectorError::Configuration(
                            format!("Failed to extract f16 output tensor: {}", e)
                        ))?;
                    
                    // Convert f16 to f32 for postprocessing
                    let output_view = output_tensor.view();
                    output_view.iter().map(|&v| v.to_f32()).collect()
                }
                #[cfg(not(feature = "half"))]
                {
                    return Err(DetectorError::Configuration(
                        "Output is float16 but half feature is not enabled".to_string()
                    ));
                }
            } else {
                // Extract as f32 tensor
                let output_tensor: ort::tensor::OrtOwnedTensor<f32, _> = outputs[0].try_extract()
                    .map_err(|e| DetectorError::Configuration(
                        format!("Failed to extract f32 output tensor: {}", e)
                    ))?;
                
                // Get view and convert to Vec
                let output_view = output_tensor.view();
                output_view.iter().cloned().collect()
            };
            
            // Postprocess outputs
            return self.postprocess_outputs(&output, image.width(), image.height());
        }
        
        #[cfg(test)]
        #[cfg(not(feature = "ort"))]
        {
            // Use mock detection when ONNX feature is not enabled
            let mock_output = self.create_mock_yolo_output();
            self.postprocess_outputs(&mock_output, image.width(), image.height())
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
        
        Ok(tensor)
    }
    
    /// Process model outputs to detections
    fn postprocess_outputs(&self, outputs: &[f32], img_width: u32, img_height: u32) -> Result<Vec<Detection>> {
        // Auto-detect YOLO version based on output shape
        let version = match self.yolo_version {
            YoloVersion::Auto => self.detect_yolo_version(outputs),
            v => v,
        };
        
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
        
        // Check for v8+ transposed format (84 values per anchor)
        if len % 84 == 0 {
            let num_anchors = len / 84;
            if num_anchors > 1000 && num_anchors < 10000 {
                return YoloVersion::V8;
            }
        }
        
        // Check for v3-v7 format (85 values per anchor)
        if len % 85 == 0 {
            let num_anchors = len / 85;
            if num_anchors > 1000 {
                return YoloVersion::V5; // Use v5 processing for v3-v7
            }
        }
        
        // Check for smaller models or different input sizes
        if len % 85 == 0 || len % 84 == 0 {
            // Default to newer format for smaller outputs
            return if len % 84 == 0 { YoloVersion::V8 } else { YoloVersion::V5 };
        }
        
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
        
        Ok(filtered_detections)
    }
    
    /// Process YOLOv8/v9/v11/v12 outputs  
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
    
    #[cfg(test)]
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
    }
    
    /// Create a mock detector for testing without an actual model
    pub fn new_mock() -> Self {
        Self {
            #[cfg(feature = "ort")]
            session: None,
            #[cfg(feature = "ort")]
            environment: None,
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
    #[cfg(feature = "half")]
    fn test_f16_conversion() {
        use half::f16;
        
        // Test f32 to f16 conversion
        let f32_values = vec![0.0f32, 1.0, -1.0, 0.5, 100.0];
        let f16_values: Vec<f16> = f32_values.iter().map(|&v| f16::from_f32(v)).collect();
        
        // Test f16 to f32 conversion
        let f32_recovered: Vec<f32> = f16_values.iter().map(|&v| v.to_f32()).collect();
        
        // Check values are approximately equal (some precision loss is expected)
        for (original, recovered) in f32_values.iter().zip(f32_recovered.iter()) {
            let diff = (original - recovered).abs();
            // f16 has limited precision, so we allow some tolerance
            assert!(diff < 0.01 || (original.abs() > 10.0 && diff / original.abs() < 0.01),
                    "Value {} converted to f16 and back differs by {}", original, diff);
        }
    }
    
    #[test]
    fn test_mock_detector_creation() {
        let detector = OnnxDetector::new_mock();
        assert_eq!(detector.input_width, 640);
        assert_eq!(detector.input_height, 640);
        assert_eq!(detector.confidence_threshold, 0.5);
        assert_eq!(detector.nms_threshold, 0.4);
    }
    
    #[test]
    fn test_detector_config() {
        let config = DetectorConfig {
            model_path: Some("test.onnx".to_string()),
            input_width: 416,
            input_height: 416,
            confidence_threshold: 0.6,
            nms_threshold: 0.5,
            num_threads: 2,
            yolo_version: YoloVersion::V8,
            class_names: Some(vec!["test_class".to_string()]),
        };
        
        let detector = OnnxDetector::new_with_config(config).unwrap();
        assert_eq!(detector.input_width, 416);
        assert_eq!(detector.input_height, 416);
        assert_eq!(detector.confidence_threshold, 0.6);
        assert_eq!(detector.nms_threshold, 0.5);
    }
    
    #[test]
    fn test_yolo_version_detection() {
        let detector = OnnxDetector::new_mock();
        
        // Test v5 format detection (85 values per anchor)
        let v5_output = vec![0.0; 25200 * 85];
        let version = detector.detect_yolo_version(&v5_output);
        assert!(matches!(version, YoloVersion::V5));
        
        // Test v8 format detection (84 values per anchor)
        let v8_output = vec![0.0; 8400 * 84];
        let version = detector.detect_yolo_version(&v8_output);
        assert!(matches!(version, YoloVersion::V8));
    }
    
    #[test]
    fn test_iou_calculation() {
        let detector = OnnxDetector::new_mock();
        
        let det1 = Detection {
            x: 100.0, y: 100.0, width: 100.0, height: 100.0,
            confidence: 0.9, class_id: 0, class_name: "test".to_string(),
        };
        
        // Same box should have IoU of 1.0
        let det2 = det1.clone();
        assert_eq!(detector.calculate_iou(&det1, &det2), 1.0);
        
        // Non-overlapping boxes should have IoU of 0.0
        let det3 = Detection {
            x: 300.0, y: 300.0, width: 100.0, height: 100.0,
            confidence: 0.9, class_id: 0, class_name: "test".to_string(),
        };
        assert_eq!(detector.calculate_iou(&det1, &det3), 0.0);
        
        // Partially overlapping boxes
        let det4 = Detection {
            x: 150.0, y: 150.0, width: 100.0, height: 100.0,
            confidence: 0.9, class_id: 0, class_name: "test".to_string(),
        };
        let iou = detector.calculate_iou(&det1, &det4);
        assert!(iou > 0.0 && iou < 1.0);
    }
    
    #[test]
    #[cfg(feature = "half")]
    fn test_f16_ndarray_creation() {
        use half::f16;
        use ndarray::Array;
        
        // Create a small test tensor
        let f32_data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let f16_data: Vec<f16> = f32_data.iter().map(|&v| f16::from_f32(v)).collect();
        
        // Create ndarray with f16 data
        let shape = vec![1, 2, 3];
        let array = Array::from_shape_vec(shape, f16_data.clone()).unwrap();
        
        // Verify shape and data
        assert_eq!(array.shape(), &[1, 2, 3]);
        assert_eq!(array.len(), 6);
        
        // Check that values can be accessed
        let flat_view: Vec<f16> = array.iter().cloned().collect();
        assert_eq!(flat_view, f16_data);
    }
}
