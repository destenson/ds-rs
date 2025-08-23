#![allow(unused)]
//! Batch-level metadata containing frames from multiple sources

use super::{FrameMeta, MetadataError, Result};
use std::collections::HashMap;

/// Represents metadata for a batch of frames from multiple sources
#[derive(Debug, Clone)]
pub struct BatchMeta {
    /// Unique batch ID
    pub batch_id: u64,
    
    /// Number of frames in this batch
    pub num_frames_in_batch: u32,
    
    /// Maximum number of frames this batch can hold
    pub max_frames_in_batch: u32,
    
    /// Frame metadata list
    frames: Vec<FrameMeta>,
    
    /// User-specific batch information
    pub misc_batch_info: Vec<u64>,
    
    /// Reserved for internal use
    reserved: Vec<u8>,
}

impl BatchMeta {
    /// Create a new batch metadata
    pub fn new(batch_id: u64, max_frames: u32) -> Self {
        Self {
            batch_id,
            num_frames_in_batch: 0,
            max_frames_in_batch: max_frames,
            frames: Vec::new(),
            misc_batch_info: vec![0; 4],
            reserved: vec![0; 256],
        }
    }
    
    /// Create mock batch metadata for testing
    pub fn new_mock(batch_id: u64) -> Self {
        let mut batch = Self::new(batch_id, 4);
        
        // Add some mock frames with objects
        for source_id in 0..2 {
            let mut frame = FrameMeta::new(source_id, batch_id);
            
            // Add some mock objects to the frame
            if source_id == 0 {
                frame.add_mock_vehicle(0, 100.0, 100.0, 50.0, 60.0, 0.95);
                frame.add_mock_person(1, 200.0, 150.0, 30.0, 80.0, 0.88);
            } else {
                frame.add_mock_vehicle(2, 150.0, 120.0, 55.0, 65.0, 0.92);
            }
            
            batch.add_frame(frame);
        }
        
        batch
    }
    
    /// Add a frame to the batch
    pub fn add_frame(&mut self, frame: FrameMeta) -> Result<()> {
        if self.num_frames_in_batch >= self.max_frames_in_batch {
            return Err(MetadataError::ExtractionFailed(
                "Batch is full".to_string()
            ));
        }
        
        self.frames.push(frame);
        self.num_frames_in_batch += 1;
        Ok(())
    }
    
    /// Get frame metadata for a specific source
    pub fn get_frame_meta(&self, source_id: u32) -> Option<FrameMeta> {
        self.frames.iter()
            .find(|f| f.source_id == source_id)
            .cloned()
    }
    
    /// Get all frames in the batch
    pub fn frames(&self) -> &[FrameMeta] {
        &self.frames
    }
    
    /// Get mutable access to frames
    pub fn frames_mut(&mut self) -> &mut Vec<FrameMeta> {
        &mut self.frames
    }
    
    /// Get the number of frames in the batch
    pub fn num_frames(&self) -> u32 {
        self.num_frames_in_batch
    }
    
    /// Clear all frames from the batch
    pub fn clear(&mut self) {
        self.frames.clear();
        self.num_frames_in_batch = 0;
    }
    
    /// Get total object count across all frames
    pub fn total_object_count(&self) -> usize {
        self.frames.iter()
            .map(|f| f.num_objects())
            .sum()
    }
    
    /// Get objects by class ID across all frames
    pub fn get_objects_by_class(&self, class_id: i32) -> Vec<super::ObjectMeta> {
        let mut objects = Vec::new();
        
        for frame in &self.frames {
            for obj in frame.objects() {
                if obj.class_id == class_id {
                    objects.push(obj.clone());
                }
            }
        }
        
        objects
    }
    
    /// Get frame statistics
    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        stats.insert("total_frames".to_string(), self.num_frames_in_batch as usize);
        stats.insert("total_objects".to_string(), self.total_object_count());
        
        // Count objects by type
        let mut vehicle_count = 0;
        let mut person_count = 0;
        let mut face_count = 0;
        
        for frame in &self.frames {
            for obj in frame.objects() {
                match obj.class_id {
                    0 => vehicle_count += 1,  // PGIE_CLASS_ID_VEHICLE
                    1 => person_count += 1,   // PGIE_CLASS_ID_PERSON
                    2 => face_count += 1,     // SGIE_CLASS_ID_FACE
                    _ => {}
                }
            }
        }
        
        stats.insert("vehicles".to_string(), vehicle_count);
        stats.insert("persons".to_string(), person_count);
        stats.insert("faces".to_string(), face_count);
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_batch_meta_creation() {
        let batch = BatchMeta::new(123, 8);
        assert_eq!(batch.batch_id, 123);
        assert_eq!(batch.max_frames_in_batch, 8);
        assert_eq!(batch.num_frames_in_batch, 0);
    }
    
    #[test]
    fn test_add_frame() {
        let mut batch = BatchMeta::new(1, 4);
        let frame = FrameMeta::new(0, 1);
        
        assert!(batch.add_frame(frame).is_ok());
        assert_eq!(batch.num_frames_in_batch, 1);
    }
    
    #[test]
    fn test_batch_full() {
        let mut batch = BatchMeta::new(1, 2);
        
        assert!(batch.add_frame(FrameMeta::new(0, 1)).is_ok());
        assert!(batch.add_frame(FrameMeta::new(1, 1)).is_ok());
        assert!(batch.add_frame(FrameMeta::new(2, 1)).is_err());
    }
    
    #[test]
    fn test_mock_batch() {
        let batch = BatchMeta::new_mock(999);
        assert!(batch.num_frames() > 0);
        assert!(batch.total_object_count() > 0);
        
        let stats = batch.get_stats();
        assert!(stats.contains_key("vehicles"));
        assert!(stats.contains_key("persons"));
    }
}
