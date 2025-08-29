#![allow(unused)]
//! Object-level metadata for detected/tracked objects

use std::collections::HashMap;

/// Unique ID for untracked objects
pub const UNTRACKED_OBJECT_ID: u64 = u64::MAX;

/// Primary detector component ID
pub const PRIMARY_DETECTOR_UID: i32 = 1;

/// Secondary detector component ID  
pub const SECONDARY_DETECTOR_UID: i32 = 2;

/// Common object class IDs
pub mod class_ids {
    pub const VEHICLE: i32 = 0;
    pub const PERSON: i32 = 1;
    pub const FACE: i32 = 2;
    pub const LICENSE_PLATE: i32 = 3;
    pub const BICYCLE: i32 = 4;
    pub const ROADSIGN: i32 = 5;
}

/// Bounding box coordinates
#[derive(Debug, Clone, Default)]
pub struct BoundingBox {
    /// Left coordinate (x)
    pub left: f32,

    /// Top coordinate (y)
    pub top: f32,

    /// Width of bounding box
    pub width: f32,

    /// Height of bounding box
    pub height: f32,
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    /// Get the right coordinate
    pub fn right(&self) -> f32 {
        self.left + self.width
    }

    /// Get the bottom coordinate
    pub fn bottom(&self) -> f32 {
        self.top + self.height
    }

    /// Get the center point
    pub fn center(&self) -> (f32, f32) {
        (self.left + self.width / 2.0, self.top + self.height / 2.0)
    }

    /// Calculate area
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Check if a point is inside the bounding box
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.left && x <= self.right() && y >= self.top && y <= self.bottom()
    }

    /// Calculate IoU (Intersection over Union) with another box
    pub fn iou(&self, other: &BoundingBox) -> f32 {
        let x1 = self.left.max(other.left);
        let y1 = self.top.max(other.top);
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());

        if x2 < x1 || y2 < y1 {
            return 0.0;
        }

        let intersection = (x2 - x1) * (y2 - y1);
        let union = self.area() + other.area() - intersection;

        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    }
}

/// Classification metadata for secondary inference
#[derive(Debug, Clone)]
pub struct ClassificationMeta {
    /// Number of labels
    pub num_labels: u32,

    /// Unique component ID
    pub unique_component_id: i32,

    /// Class labels and confidences
    pub labels: Vec<(String, f32)>,
}

impl ClassificationMeta {
    /// Create new classification metadata
    pub fn new(component_id: i32) -> Self {
        Self {
            num_labels: 0,
            unique_component_id: component_id,
            labels: Vec::new(),
        }
    }

    /// Add a classification label
    pub fn add_label(&mut self, label: String, confidence: f32) {
        self.labels.push((label, confidence));
        self.num_labels += 1;
    }

    /// Get the top classification
    pub fn top_label(&self) -> Option<(&str, f32)> {
        self.labels
            .first()
            .map(|(label, conf)| (label.as_str(), *conf))
    }
}

/// Metadata for a detected/tracked object
#[derive(Debug, Clone)]
pub struct ObjectMeta {
    /// Unique object ID for tracking (UNTRACKED_OBJECT_ID if not tracked)
    pub object_id: u64,

    /// Class ID from inference
    pub class_id: i32,

    /// Component ID that generated this metadata
    pub unique_component_id: i32,

    /// Detection confidence
    pub confidence: f32,

    /// Tracker confidence
    pub tracker_confidence: f32,

    /// Bounding box from detector
    pub detector_bbox_info: BoundingBox,

    /// Bounding box from tracker
    pub tracker_bbox_info: BoundingBox,

    /// Current bounding box (clipped to frame boundaries)
    pub rect_params: BoundingBox,

    /// Object label text
    pub obj_label: String,

    /// Classification metadata list
    pub classifications: Vec<ClassificationMeta>,

    /// Parent object (for secondary detections like face on person)
    pub parent: Option<Box<ObjectMeta>>,

    /// Tracking age (frames since first detection)
    pub tracking_age: u32,

    /// User metadata
    pub user_meta: HashMap<String, String>,

    /// Miscellaneous object info
    pub misc_obj_info: Vec<i64>,

    /// Reserved for internal use
    reserved: Vec<u8>,
}

impl ObjectMeta {
    /// Create new object metadata
    pub fn new(object_id: u64) -> Self {
        Self {
            object_id,
            class_id: -1,
            unique_component_id: PRIMARY_DETECTOR_UID,
            confidence: 0.0,
            tracker_confidence: -0.1,
            detector_bbox_info: BoundingBox::default(),
            tracker_bbox_info: BoundingBox::default(),
            rect_params: BoundingBox::default(),
            obj_label: String::new(),
            classifications: Vec::new(),
            parent: None,
            tracking_age: 0,
            user_meta: HashMap::new(),
            misc_obj_info: vec![0; 4],
            reserved: vec![0; 256],
        }
    }

    /// Create an untracked object
    pub fn new_untracked() -> Self {
        Self::new(UNTRACKED_OBJECT_ID)
    }

    /// Check if object is being tracked
    pub fn is_tracked(&self) -> bool {
        self.object_id != UNTRACKED_OBJECT_ID
    }

    /// Set object class and label
    pub fn set_class(&mut self, class_id: i32, label: &str) {
        self.class_id = class_id;
        self.obj_label = label.to_string();
    }

    /// Set detection bounding box
    pub fn set_detection_bbox(&mut self, bbox: BoundingBox, confidence: f32) {
        self.detector_bbox_info = bbox.clone();
        self.rect_params = bbox;
        self.confidence = confidence;
    }

    /// Set tracker bounding box
    pub fn set_tracker_bbox(&mut self, bbox: BoundingBox, confidence: f32) {
        self.tracker_bbox_info = bbox.clone();
        self.rect_params = bbox;
        self.tracker_confidence = confidence;
    }

    /// Add classification result
    pub fn add_classification(&mut self, classification: ClassificationMeta) {
        self.classifications.push(classification);
    }

    /// Set parent object (for secondary detections)
    pub fn set_parent(&mut self, parent: ObjectMeta) {
        self.parent = Some(Box::new(parent));
    }

    /// Get object class name
    pub fn class_name(&self) -> &str {
        match self.class_id {
            0 => "vehicle",
            1 => "person",
            2 => "face",
            3 => "license_plate",
            _ => &self.obj_label,
        }
    }

    /// Check if this is a primary detection
    pub fn is_primary(&self) -> bool {
        self.unique_component_id == PRIMARY_DETECTOR_UID
    }

    /// Check if this is a secondary detection
    pub fn is_secondary(&self) -> bool {
        self.unique_component_id == SECONDARY_DETECTOR_UID
    }

    /// Get current bounding box
    pub fn bbox(&self) -> &BoundingBox {
        &self.rect_params
    }

    /// Increment tracking age
    pub fn increment_age(&mut self) {
        self.tracking_age += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(10.0, 20.0, 30.0, 40.0);

        assert_eq!(bbox.right(), 40.0);
        assert_eq!(bbox.bottom(), 60.0);
        assert_eq!(bbox.center(), (25.0, 40.0));
        assert_eq!(bbox.area(), 1200.0);

        assert!(bbox.contains_point(25.0, 40.0));
        assert!(!bbox.contains_point(5.0, 40.0));
    }

    #[test]
    fn test_iou_calculation() {
        let bbox1 = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        let bbox2 = BoundingBox::new(50.0, 50.0, 100.0, 100.0);

        let iou = bbox1.iou(&bbox2);
        assert!(iou > 0.0 && iou < 1.0);

        let bbox3 = BoundingBox::new(200.0, 200.0, 50.0, 50.0);
        assert_eq!(bbox1.iou(&bbox3), 0.0);
    }

    #[test]
    fn test_object_meta() {
        let mut obj = ObjectMeta::new(123);
        assert_eq!(obj.object_id, 123);
        assert!(obj.is_tracked());

        obj.set_class(class_ids::VEHICLE, "car");
        assert_eq!(obj.class_id, 0);
        assert_eq!(obj.class_name(), "vehicle");

        let bbox = BoundingBox::new(10.0, 20.0, 30.0, 40.0);
        obj.set_detection_bbox(bbox, 0.95);
        assert_eq!(obj.confidence, 0.95);
    }

    #[test]
    fn test_classification_meta() {
        let mut classification = ClassificationMeta::new(2);
        classification.add_label("sedan".to_string(), 0.85);
        classification.add_label("suv".to_string(), 0.10);

        assert_eq!(classification.num_labels, 2);

        let top = classification.top_label();
        assert!(top.is_some());
        assert_eq!(top.unwrap().0, "sedan");
    }
}
