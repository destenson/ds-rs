#![allow(unused)]
#![cfg(feature = "nalgebra")]

use super::detector::Detection;
use nalgebra::Point2;
use std::collections::HashMap;

/// Tracked object with history
#[derive(Debug, Clone)]
pub struct TrackedObject {
    pub id: u64,
    pub centroid: Point2<f32>,
    pub bbox: BoundingBox,
    pub class_id: usize,
    pub class_name: String,
    pub confidence: f32,
    pub disappeared_count: u32,
    pub trajectory: Vec<Point2<f32>>,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    fn centroid(&self) -> Point2<f32> {
        Point2::new(
            self.x + self.width / 2.0,
            self.y + self.height / 2.0,
        )
    }
}

/// Simple Centroid Tracker
/// Tracks objects by matching centroids between frames using Euclidean distance
pub struct CentroidTracker {
    next_object_id: u64,
    objects: HashMap<u64, TrackedObject>,
    max_distance: f32,
    max_disappeared: u32,
}

impl CentroidTracker {
    /// Create a new centroid tracker
    /// - max_distance: Maximum distance to associate objects between frames
    /// - max_disappeared: Number of frames before removing lost object
    pub fn new(max_distance: f32, max_disappeared: u32) -> Self {
        Self {
            next_object_id: 0,
            objects: HashMap::new(),
            max_distance,
            max_disappeared,
        }
    }
    
    /// Update tracker with new detections
    pub fn update(&mut self, detections: Vec<Detection>) -> Vec<TrackedObject> {
        if detections.is_empty() {
            // Mark all existing objects as disappeared
            let mut to_remove = Vec::new();
            for (id, object) in self.objects.iter_mut() {
                object.disappeared_count += 1;
                if object.disappeared_count > self.max_disappeared {
                    to_remove.push(*id);
                }
            }
            
            for id in to_remove {
                self.objects.remove(&id);
            }
            
            return self.objects.values().cloned().collect();
        }
        
        let input_centroids: Vec<Point2<f32>> = detections
            .iter()
            .map(|d| Point2::new(d.x + d.width / 2.0, d.y + d.height / 2.0))
            .collect();
        
        if self.objects.is_empty() {
            // Register all detections as new objects
            for (i, detection) in detections.iter().enumerate() {
                self.register_object(detection, input_centroids[i]);
            }
        } else {
            // Match existing objects to new detections
            let object_ids: Vec<u64> = self.objects.keys().copied().collect();
            let object_centroids: Vec<Point2<f32>> = object_ids
                .iter()
                .map(|id| self.objects[id].centroid)
                .collect();
            
            // Compute distance matrix
            let mut distances = vec![vec![0.0; input_centroids.len()]; object_centroids.len()];
            for (i, obj_centroid) in object_centroids.iter().enumerate() {
                for (j, input_centroid) in input_centroids.iter().enumerate() {
                    distances[i][j] = nalgebra::distance(obj_centroid, input_centroid);
                }
            }
            
            // Find minimum distance assignments
            let assignments = self.hungarian_assignment(&distances);
            
            let mut used_objects = vec![false; object_ids.len()];
            let mut used_detections = vec![false; detections.len()];
            
            for (obj_idx, det_idx) in assignments {
                if distances[obj_idx][det_idx] < self.max_distance {
                    let object_id = object_ids[obj_idx];
                    self.update_object(object_id, &detections[det_idx], input_centroids[det_idx]);
                    used_objects[obj_idx] = true;
                    used_detections[det_idx] = true;
                }
            }
            
            // Mark unmatched objects as disappeared
            for (i, &object_id) in object_ids.iter().enumerate() {
                if !used_objects[i] {
                    if let Some(object) = self.objects.get_mut(&object_id) {
                        object.disappeared_count += 1;
                        if object.disappeared_count > self.max_disappeared {
                            self.objects.remove(&object_id);
                        }
                    }
                }
            }
            
            // Register unmatched detections as new objects
            for (i, detection) in detections.iter().enumerate() {
                if !used_detections[i] {
                    self.register_object(detection, input_centroids[i]);
                }
            }
        }
        
        self.objects.values().cloned().collect()
    }
    
    /// Register a new object
    fn register_object(&mut self, detection: &Detection, centroid: Point2<f32>) {
        let mut trajectory = Vec::with_capacity(100);
        trajectory.push(centroid);
        
        self.objects.insert(
            self.next_object_id,
            TrackedObject {
                id: self.next_object_id,
                centroid,
                bbox: BoundingBox {
                    x: detection.x,
                    y: detection.y,
                    width: detection.width,
                    height: detection.height,
                },
                class_id: detection.class_id,
                class_name: detection.class_name.clone(),
                confidence: detection.confidence,
                disappeared_count: 0,
                trajectory,
            },
        );
        
        self.next_object_id += 1;
    }
    
    /// Update an existing object
    fn update_object(&mut self, id: u64, detection: &Detection, centroid: Point2<f32>) {
        if let Some(object) = self.objects.get_mut(&id) {
            object.centroid = centroid;
            object.bbox = BoundingBox {
                x: detection.x,
                y: detection.y,
                width: detection.width,
                height: detection.height,
            };
            object.confidence = detection.confidence;
            object.disappeared_count = 0;
            
            // Update trajectory (keep last 100 points)
            object.trajectory.push(centroid);
            if object.trajectory.len() > 100 {
                object.trajectory.remove(0);
            }
        }
    }
    
    /// Simple Hungarian algorithm for assignment
    /// Returns pairs of (object_index, detection_index)
    fn hungarian_assignment(&self, distances: &[Vec<f32>]) -> Vec<(usize, usize)> {
        if distances.is_empty() || distances[0].is_empty() {
            return Vec::new();
        }
        
        let n_objects = distances.len();
        let n_detections = distances[0].len();
        let mut assignments = Vec::new();
        
        // Simple greedy assignment (not optimal but fast)
        let mut used_objects = vec![false; n_objects];
        let mut used_detections = vec![false; n_detections];
        
        // Create sorted list of all distances
        let mut all_distances = Vec::new();
        for i in 0..n_objects {
            for j in 0..n_detections {
                all_distances.push((distances[i][j], i, j));
            }
        }
        all_distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        // Assign in order of increasing distance
        for (_, obj_idx, det_idx) in all_distances {
            if !used_objects[obj_idx] && !used_detections[det_idx] {
                assignments.push((obj_idx, det_idx));
                used_objects[obj_idx] = true;
                used_detections[det_idx] = true;
                
                if assignments.len() == n_objects.min(n_detections) {
                    break;
                }
            }
        }
        
        assignments
    }
    
    /// Get all tracked objects
    pub fn get_objects(&self) -> Vec<TrackedObject> {
        self.objects.values().cloned().collect()
    }
    
    /// Clear all tracked objects
    pub fn clear(&mut self) {
        self.objects.clear();
        self.next_object_id = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_centroid_tracker_creation() {
        let tracker = CentroidTracker::new(50.0, 30);
        assert_eq!(tracker.objects.len(), 0);
        assert_eq!(tracker.next_object_id, 0);
    }
    
    #[test]
    fn test_register_new_object() {
        let mut tracker = CentroidTracker::new(50.0, 30);
        
        let detection = Detection {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
            confidence: 0.9,
            class_id: 0,
            class_name: "person".to_string(),
        };
        
        let objects = tracker.update(vec![detection]);
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].id, 0);
        assert_eq!(objects[0].class_name, "person");
    }
    
    #[test]
    fn test_track_moving_object() {
        let mut tracker = CentroidTracker::new(50.0, 30);
        
        // First frame
        let detection1 = Detection {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
            confidence: 0.9,
            class_id: 0,
            class_name: "person".to_string(),
        };
        
        let objects1 = tracker.update(vec![detection1]);
        assert_eq!(objects1.len(), 1);
        let first_id = objects1[0].id;
        
        // Second frame - object moved slightly
        let detection2 = Detection {
            x: 110.0,  // Moved 10 pixels
            y: 105.0,  // Moved 5 pixels
            width: 50.0,
            height: 50.0,
            confidence: 0.9,
            class_id: 0,
            class_name: "person".to_string(),
        };
        
        let objects2 = tracker.update(vec![detection2]);
        assert_eq!(objects2.len(), 1);
        assert_eq!(objects2[0].id, first_id); // Same ID - object was tracked
        assert_eq!(objects2[0].trajectory.len(), 2); // Has trajectory history
    }
    
    #[test]
    fn test_object_disappearance() {
        let mut tracker = CentroidTracker::new(50.0, 2); // Low max_disappeared for testing
        
        let detection = Detection {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
            confidence: 0.9,
            class_id: 0,
            class_name: "person".to_string(),
        };
        
        // Register object
        tracker.update(vec![detection]);
        assert_eq!(tracker.objects.len(), 1);
        
        // Update with no detections
        tracker.update(vec![]);
        assert_eq!(tracker.objects.len(), 1); // Still tracked
        
        tracker.update(vec![]);
        assert_eq!(tracker.objects.len(), 1); // Still tracked
        
        tracker.update(vec![]);
        assert_eq!(tracker.objects.len(), 0); // Removed after max_disappeared
    }
}
