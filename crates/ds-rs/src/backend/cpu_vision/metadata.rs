//! CPU Vision metadata structures compatible with DeepStream format
//! 
//! This module provides metadata structures that can be attached to GStreamer buffers
//! to carry object detection and tracking information through the pipeline.

use crate::backend::cpu_vision::detector::Detection;
use gstreamer as gst;

/// Object detection metadata that can be attached to GStreamer buffers
#[derive(Debug, Clone)]
pub struct DetectionMeta {
    pub detections: Vec<Detection>,
    pub frame_number: u64,
    pub timestamp: gst::ClockTime,
    pub frame_width: u32,
    pub frame_height: u32,
}

impl DetectionMeta {
    pub fn new(
        detections: Vec<Detection>,
        frame_number: u64,
        timestamp: gst::ClockTime,
        frame_width: u32,
        frame_height: u32,
    ) -> Self {
        Self {
            detections,
            frame_number,
            timestamp,
            frame_width,
            frame_height,
        }
    }
    
    /// Get number of detections
    pub fn num_objects(&self) -> usize {
        self.detections.len()
    }
    
    /// Filter detections by class ID
    pub fn filter_by_class(&self, class_id: usize) -> Vec<&Detection> {
        self.detections.iter()
            .filter(|d| d.class_id == class_id)
            .collect()
    }
    
    /// Filter detections by confidence threshold
    pub fn filter_by_confidence(&self, min_confidence: f32) -> Vec<&Detection> {
        self.detections.iter()
            .filter(|d| d.confidence >= min_confidence)
            .collect()
    }
}

/// Tracking metadata for objects with persistent IDs
#[derive(Debug, Clone)]
pub struct TrackingMeta {
    pub tracked_objects: Vec<TrackedDetection>,
    pub frame_number: u64,
    pub timestamp: gst::ClockTime,
}

/// A detection with tracking information
#[derive(Debug, Clone)]
pub struct TrackedDetection {
    pub detection: Detection,
    pub track_id: u64,
    pub age: u32,  // Number of frames this object has been tracked
}

impl TrackedDetection {
    pub fn new(detection: Detection, track_id: u64, age: u32) -> Self {
        Self {
            detection,
            track_id,
            age,
        }
    }
}

/// Buffer probe data that can be shared between probe callbacks
pub struct ProbeData {
    pub detections: Vec<Detection>,
    pub frame_count: u64,
}

impl ProbeData {
    pub fn new() -> Self {
        Self {
            detections: Vec::new(),
            frame_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::cpu_vision::detector::Detection;
    
    #[test]
    fn test_detection_meta_creation() {
        let detection = Detection {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
            confidence: 0.9,
            class_id: 0,
            class_name: "person".to_string(),
        };
        
        let meta = DetectionMeta::new(
            vec![detection],
            1,
            gst::ClockTime::from_nseconds(1000000),
            640,
            480,
        );
        
        assert_eq!(meta.num_objects(), 1);
        assert_eq!(meta.frame_number, 1);
        assert_eq!(meta.frame_width, 640);
        assert_eq!(meta.frame_height, 480);
    }
    
    #[test]
    fn test_filter_by_confidence() {
        let high_conf = Detection {
            x: 100.0, y: 100.0, width: 50.0, height: 50.0,
            confidence: 0.9, class_id: 0, class_name: "person".to_string(),
        };
        
        let low_conf = Detection {
            x: 200.0, y: 200.0, width: 50.0, height: 50.0,
            confidence: 0.3, class_id: 1, class_name: "car".to_string(),
        };
        
        let meta = DetectionMeta::new(
            vec![high_conf, low_conf],
            1,
            gst::ClockTime::from_nseconds(1000000),
            640,
            480,
        );
        
        let filtered = meta.filter_by_confidence(0.5);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].confidence, 0.9);
    }
    
    #[test]
    fn test_tracked_detection() {
        let detection = Detection {
            x: 100.0, y: 100.0, width: 50.0, height: 50.0,
            confidence: 0.9, class_id: 0, class_name: "person".to_string(),
        };
        
        let tracked = TrackedDetection::new(detection, 42, 10);
        assert_eq!(tracked.track_id, 42);
        assert_eq!(tracked.age, 10);
        assert_eq!(tracked.detection.class_name, "person");
    }
}