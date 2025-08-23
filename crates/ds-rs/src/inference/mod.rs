//! AI inference result processing and configuration

use crate::metadata::{ObjectMeta, ClassificationMeta, BoundingBox};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub mod config;

pub use config::{InferenceConfig, ModelConfig};

/// Errors that can occur during inference operations
#[derive(Debug, Error)]
pub enum InferenceError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Invalid class ID: {0}")]
    InvalidClassId(i32),
    
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
}

pub type Result<T> = std::result::Result<T, InferenceError>;

/// Inference results from a detection model
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Detected objects
    pub objects: Vec<ObjectMeta>,
    
    /// Frame ID
    pub frame_id: u64,
    
    /// Source ID
    pub source_id: u32,
    
    /// Model name that produced this result
    pub model_name: String,
    
    /// Inference timestamp
    pub timestamp: u64,
}

impl DetectionResult {
    /// Create new detection result
    pub fn new(frame_id: u64, source_id: u32, model_name: String) -> Self {
        Self {
            objects: Vec::new(),
            frame_id,
            source_id,
            model_name,
            timestamp: 0,
        }
    }
    
    /// Add a detected object
    pub fn add_object(&mut self, object: ObjectMeta) {
        self.objects.push(object);
    }
    
    /// Filter objects by confidence threshold
    pub fn filter_by_confidence(&self, threshold: f32) -> Vec<&ObjectMeta> {
        self.objects.iter()
            .filter(|obj| obj.confidence >= threshold)
            .collect()
    }
    
    /// Filter objects by class ID
    pub fn filter_by_class(&self, class_id: i32) -> Vec<&ObjectMeta> {
        self.objects.iter()
            .filter(|obj| obj.class_id == class_id)
            .collect()
    }
    
    /// Get object count by class
    pub fn count_by_class(&self) -> HashMap<i32, usize> {
        let mut counts = HashMap::new();
        
        for obj in &self.objects {
            *counts.entry(obj.class_id).or_insert(0) += 1;
        }
        
        counts
    }
}

/// Classification results from a classifier model
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    /// Object ID this classification applies to
    pub object_id: u64,
    
    /// Classification metadata
    pub classification: ClassificationMeta,
    
    /// Model name
    pub model_name: String,
}

/// Label mapping for class IDs
#[derive(Debug, Clone)]
pub struct LabelMap {
    /// Map from class ID to label name
    labels: HashMap<i32, String>,
    
    /// Map from label name to class ID
    reverse_map: HashMap<String, i32>,
}

impl LabelMap {
    /// Create a new label map
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }
    
    /// Create default label map for common objects
    pub fn default_coco() -> Self {
        let mut map = Self::new();
        
        // Add common COCO classes
        map.add_label(0, "person");
        map.add_label(1, "bicycle");
        map.add_label(2, "car");
        map.add_label(3, "motorcycle");
        map.add_label(4, "airplane");
        map.add_label(5, "bus");
        map.add_label(6, "train");
        map.add_label(7, "truck");
        map.add_label(8, "boat");
        
        map
    }
    
    /// Create label map for traffic/vehicle detection
    pub fn traffic() -> Self {
        let mut map = Self::new();
        
        map.add_label(0, "vehicle");
        map.add_label(1, "person");
        map.add_label(2, "bicycle");
        map.add_label(3, "motorcycle");
        map.add_label(4, "bus");
        map.add_label(5, "truck");
        
        map
    }
    
    /// Add a label mapping
    pub fn add_label(&mut self, class_id: i32, label: &str) {
        self.labels.insert(class_id, label.to_string());
        self.reverse_map.insert(label.to_string(), class_id);
    }
    
    /// Get label for class ID
    pub fn get_label(&self, class_id: i32) -> Option<&str> {
        self.labels.get(&class_id).map(|s| s.as_str())
    }
    
    /// Get class ID for label
    pub fn get_class_id(&self, label: &str) -> Option<i32> {
        self.reverse_map.get(label).copied()
    }
    
    /// Load label map from file
    pub fn load_from_file<P: AsRef<Path>>(_path: P) -> Result<Self> {
        // In a real implementation, this would parse a label file
        // For now, return a default map
        Ok(Self::default_coco())
    }
}

/// Inference processor for handling model outputs
pub struct InferenceProcessor {
    /// Label maps for different models
    label_maps: HashMap<String, LabelMap>,
    
    /// Confidence thresholds per model
    thresholds: HashMap<String, f32>,
}

impl InferenceProcessor {
    /// Create new inference processor
    pub fn new() -> Self {
        Self {
            label_maps: HashMap::new(),
            thresholds: HashMap::new(),
        }
    }
    
    /// Register a model with its label map
    pub fn register_model(&mut self, model_name: &str, label_map: LabelMap, threshold: f32) {
        self.label_maps.insert(model_name.to_string(), label_map);
        self.thresholds.insert(model_name.to_string(), threshold);
    }
    
    /// Process detection output
    pub fn process_detection(
        &self,
        model_name: &str,
        raw_output: Vec<f32>,
        frame_id: u64,
        source_id: u32,
    ) -> Result<DetectionResult> {
        let mut result = DetectionResult::new(frame_id, source_id, model_name.to_string());
        
        // This is a simplified processing - real implementation would parse
        // the raw tensor output based on model architecture
        
        // Mock processing for demonstration
        let threshold = self.thresholds.get(model_name).copied().unwrap_or(0.5);
        let label_map = self.label_maps.get(model_name);
        
        // Create mock detections
        if !raw_output.is_empty() {
            let mut obj = ObjectMeta::new(1);
            obj.class_id = 0;
            obj.confidence = 0.95;
            obj.detector_bbox_info = BoundingBox::new(100.0, 100.0, 50.0, 60.0);
            obj.rect_params = obj.detector_bbox_info.clone();
            
            if let Some(map) = label_map {
                if let Some(label) = map.get_label(obj.class_id) {
                    obj.obj_label = label.to_string();
                }
            }
            
            if obj.confidence >= threshold {
                result.add_object(obj);
            }
        }
        
        Ok(result)
    }
    
    /// Post-process detections with NMS (Non-Maximum Suppression)
    pub fn apply_nms(detections: &mut Vec<ObjectMeta>, iou_threshold: f32) {
        // Sort by confidence
        detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        let mut keep = vec![true; detections.len()];
        
        for i in 0..detections.len() {
            if !keep[i] {
                continue;
            }
            
            for j in (i + 1)..detections.len() {
                if !keep[j] {
                    continue;
                }
                
                // Only suppress if same class
                if detections[i].class_id != detections[j].class_id {
                    continue;
                }
                
                let iou = detections[i].rect_params.iou(&detections[j].rect_params);
                if iou > iou_threshold {
                    keep[j] = false;
                }
            }
        }
        
        // Filter out suppressed detections
        let filtered: Vec<ObjectMeta> = detections.iter()
            .zip(keep.iter())
            .filter(|&(_, &k)| k)
            .map(|(obj, _)| obj.clone())
            .collect();
        
        *detections = filtered;
    }
}

impl Default for InferenceProcessor {
    fn default() -> Self {
        let mut processor = Self::new();
        
        // Register default models
        processor.register_model("primary-detector", LabelMap::traffic(), 0.5);
        processor.register_model("secondary-detector", LabelMap::default_coco(), 0.4);
        
        processor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_label_map() {
        let mut map = LabelMap::new();
        map.add_label(0, "car");
        map.add_label(1, "person");
        
        assert_eq!(map.get_label(0), Some("car"));
        assert_eq!(map.get_class_id("person"), Some(1));
    }
    
    #[test]
    fn test_detection_result() {
        let mut result = DetectionResult::new(1, 0, "test-model".to_string());
        
        let mut obj1 = ObjectMeta::new(1);
        obj1.confidence = 0.9;
        obj1.class_id = 0;
        
        let mut obj2 = ObjectMeta::new(2);
        obj2.confidence = 0.3;
        obj2.class_id = 1;
        
        result.add_object(obj1);
        result.add_object(obj2);
        
        let high_conf = result.filter_by_confidence(0.5);
        assert_eq!(high_conf.len(), 1);
        
        let counts = result.count_by_class();
        assert_eq!(counts.get(&0), Some(&1));
        assert_eq!(counts.get(&1), Some(&1));
    }
    
    #[test]
    fn test_nms() {
        let mut detections = vec![];
        
        // Create overlapping detections
        let mut obj1 = ObjectMeta::new(1);
        obj1.confidence = 0.9;
        obj1.class_id = 0;
        obj1.rect_params = BoundingBox::new(100.0, 100.0, 50.0, 50.0);
        
        let mut obj2 = ObjectMeta::new(2);
        obj2.confidence = 0.8;
        obj2.class_id = 0;
        obj2.rect_params = BoundingBox::new(105.0, 105.0, 50.0, 50.0);
        
        let mut obj3 = ObjectMeta::new(3);
        obj3.confidence = 0.7;
        obj3.class_id = 1;
        obj3.rect_params = BoundingBox::new(200.0, 200.0, 40.0, 40.0);
        
        detections.push(obj1);
        detections.push(obj2);
        detections.push(obj3);
        
        InferenceProcessor::apply_nms(&mut detections, 0.5);
        
        // Should keep obj1 (highest confidence) and obj3 (different class)
        assert_eq!(detections.len(), 2);
        assert_eq!(detections[0].object_id, 1);
        assert_eq!(detections[1].object_id, 3);
    }
}