#![allow(unused, non_snake_case)]
//! Frame-level metadata for individual video frames

use super::{ObjectMeta, BoundingBox};

/// Metadata for a single frame from a source
#[derive(Debug, Clone)]
pub struct FrameMeta {
    /// Unique source ID
    pub source_id: u32,
    
    /// Batch ID this frame belongs to
    pub batch_id: u64,
    
    /// Frame number from the source
    pub frame_num: i64,
    
    /// Presentation timestamp
    pub buf_pts: u64,
    
    /// NTP timestamp
    pub ntp_timestamp: u64,
    
    /// Source frame width
    pub source_frame_width: u32,
    
    /// Source frame height  
    pub source_frame_height: u32,
    
    /// Surface index
    pub surface_index: u32,
    
    /// Surface type
    pub surface_type: u32,
    
    /// Number of surfaces per frame
    pub num_surfaces_per_frame: u32,
    
    /// Object metadata list
    objects: Vec<ObjectMeta>,
    
    /// Number of objects in frame
    pub num_obj_meta: u32,
    
    /// Whether objects in this frame should be inferred
    pub bInferDone: bool,
    
    /// Reserved for internal use
    reserved: Vec<u8>,
}

impl FrameMeta {
    /// Create new frame metadata
    pub fn new(source_id: u32, batch_id: u64) -> Self {
        Self {
            source_id,
            batch_id,
            frame_num: 0,
            buf_pts: 0,
            ntp_timestamp: 0,
            source_frame_width: 1920,
            source_frame_height: 1080,
            surface_index: 0,
            surface_type: 0,
            num_surfaces_per_frame: 1,
            objects: Vec::new(),
            num_obj_meta: 0,
            bInferDone: false,
            reserved: vec![0; 256],
        }
    }
    
    /// Add an object to the frame
    pub fn add_object(&mut self, object: ObjectMeta) {
        self.objects.push(object);
        self.num_obj_meta += 1;
    }
    
    /// Add a mock vehicle object for testing
    pub fn add_mock_vehicle(&mut self, object_id: u64, x: f32, y: f32, width: f32, height: f32, confidence: f32) {
        let mut obj = ObjectMeta::new(object_id);
        obj.class_id = 0; // PGIE_CLASS_ID_VEHICLE
        obj.confidence = confidence;
        obj.detector_bbox_info = BoundingBox {
            left: x,
            top: y,
            width,
            height,
        };
        obj.rect_params = obj.detector_bbox_info.clone();
        obj.obj_label = "vehicle".to_string();
        
        self.add_object(obj);
    }
    
    /// Add a mock person object for testing
    pub fn add_mock_person(&mut self, object_id: u64, x: f32, y: f32, width: f32, height: f32, confidence: f32) {
        let mut obj = ObjectMeta::new(object_id);
        obj.class_id = 1; // PGIE_CLASS_ID_PERSON
        obj.confidence = confidence;
        obj.detector_bbox_info = BoundingBox {
            left: x,
            top: y,
            width,
            height,
        };
        obj.rect_params = obj.detector_bbox_info.clone();
        obj.obj_label = "person".to_string();
        
        self.add_object(obj);
    }
    
    /// Get all objects in the frame
    pub fn objects(&self) -> &[ObjectMeta] {
        &self.objects
    }
    
    /// Get mutable access to objects
    pub fn objects_mut(&mut self) -> &mut Vec<ObjectMeta> {
        &mut self.objects
    }
    
    /// Get the number of objects in the frame
    pub fn num_objects(&self) -> usize {
        self.objects.len()
    }
    
    /// Find objects by class ID
    pub fn find_objects_by_class(&self, class_id: i32) -> Vec<&ObjectMeta> {
        self.objects.iter()
            .filter(|obj| obj.class_id == class_id)
            .collect()
    }
    
    /// Find object by tracking ID
    pub fn find_object_by_id(&self, object_id: u64) -> Option<&ObjectMeta> {
        self.objects.iter()
            .find(|obj| obj.object_id == object_id)
    }
    
    /// Clear all objects from the frame
    pub fn clear_objects(&mut self) {
        self.objects.clear();
        self.num_obj_meta = 0;
    }
    
    /// Check if inference has been done on this frame
    pub fn is_inferred(&self) -> bool {
        self.bInferDone
    }
    
    /// Mark frame as inferred
    pub fn set_inferred(&mut self, done: bool) {
        self.bInferDone = done;
    }
    
    /// Get frame dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.source_frame_width, self.source_frame_height)
    }
    
    /// Set frame dimensions
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.source_frame_width = width;
        self.source_frame_height = height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_meta_creation() {
        let frame = FrameMeta::new(0, 123);
        assert_eq!(frame.source_id, 0);
        assert_eq!(frame.batch_id, 123);
        assert_eq!(frame.num_objects(), 0);
    }
    
    #[test]
    fn test_add_objects() {
        let mut frame = FrameMeta::new(0, 1);
        
        frame.add_mock_vehicle(1, 10.0, 20.0, 30.0, 40.0, 0.95);
        frame.add_mock_person(2, 50.0, 60.0, 20.0, 70.0, 0.88);
        
        assert_eq!(frame.num_objects(), 2);
        assert_eq!(frame.num_obj_meta, 2);
    }
    
    #[test]
    fn test_find_objects() {
        let mut frame = FrameMeta::new(0, 1);
        
        frame.add_mock_vehicle(1, 10.0, 20.0, 30.0, 40.0, 0.95);
        frame.add_mock_person(2, 50.0, 60.0, 20.0, 70.0, 0.88);
        frame.add_mock_vehicle(3, 100.0, 120.0, 35.0, 45.0, 0.92);
        
        let vehicles = frame.find_objects_by_class(0);
        assert_eq!(vehicles.len(), 2);
        
        let persons = frame.find_objects_by_class(1);
        assert_eq!(persons.len(), 1);
        
        let obj = frame.find_object_by_id(2);
        assert!(obj.is_some());
        assert_eq!(obj.unwrap().class_id, 1);
    }
    
    #[test]
    fn test_frame_dimensions() {
        let mut frame = FrameMeta::new(0, 1);
        assert_eq!(frame.dimensions(), (1920, 1080));
        
        frame.set_dimensions(1280, 720);
        assert_eq!(frame.dimensions(), (1280, 720));
    }
}
