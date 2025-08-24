//! Metadata bridge for connecting inference results to OSD rendering

use crate::metadata::object::ObjectMeta;
use gstreamer as gst;
use std::collections::VecDeque;
use std::sync::Arc;

/// Maximum number of frames to buffer
const MAX_FRAME_BUFFER: usize = 30;

/// Bridge between inference metadata and rendering systems
#[derive(Debug, Clone)]
pub struct MetadataBridge {
    /// Buffer of frame metadata indexed by timestamp
    frame_buffer: VecDeque<FrameMetadata>,
    
    /// Current frame being rendered
    current_frame: Option<FrameMetadata>,
    
    /// Maximum latency in nanoseconds
    max_latency: u64,
    
    /// Statistics
    stats: BridgeStatistics,
}

/// Metadata for a single frame
#[derive(Debug, Clone)]
struct FrameMetadata {
    /// Frame timestamp
    timestamp: gst::ClockTime,
    
    /// Detected objects in this frame
    objects: Vec<ObjectMeta>,
    
    /// Frame number
    frame_number: u64,
    
    /// Processing time in milliseconds
    processing_time_ms: f64,
}

/// Statistics for the metadata bridge
#[derive(Debug, Clone, Default)]
pub struct BridgeStatistics {
    /// Total frames processed
    pub frames_processed: u64,
    
    /// Frames dropped due to latency
    pub frames_dropped: u64,
    
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    
    /// Peak latency in milliseconds
    pub peak_latency_ms: f64,
    
    /// Current buffer size
    pub buffer_size: usize,
}

impl MetadataBridge {
    /// Create a new metadata bridge
    pub fn new() -> Self {
        Self {
            frame_buffer: VecDeque::with_capacity(MAX_FRAME_BUFFER),
            current_frame: None,
            max_latency: 100_000_000, // 100ms default
            stats: BridgeStatistics::default(),
        }
    }
    
    /// Create with custom max latency
    pub fn with_max_latency(max_latency_ms: u64) -> Self {
        Self {
            frame_buffer: VecDeque::with_capacity(MAX_FRAME_BUFFER),
            current_frame: None,
            max_latency: max_latency_ms * 1_000_000,
            stats: BridgeStatistics::default(),
        }
    }
    
    /// Update objects for the current frame
    pub fn update_objects(&mut self, objects: Vec<ObjectMeta>, timestamp: gst::ClockTime) {
        let frame = FrameMetadata {
            timestamp,
            objects,
            frame_number: self.stats.frames_processed,
            processing_time_ms: 0.0,
        };
        
        // Add to buffer
        self.add_frame(frame);
        
        // Update statistics
        self.stats.frames_processed += 1;
        self.stats.buffer_size = self.frame_buffer.len();
    }
    
    /// Add a frame to the buffer
    fn add_frame(&mut self, frame: FrameMetadata) {
        // Remove old frames beyond max latency
        if let Some(current_time) = frame.timestamp.nseconds() {
            while let Some(oldest) = self.frame_buffer.front() {
                if let Some(oldest_time) = oldest.timestamp.nseconds() {
                    if current_time > oldest_time && 
                       current_time - oldest_time > self.max_latency {
                        self.frame_buffer.pop_front();
                        self.stats.frames_dropped += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        
        // Add new frame
        self.frame_buffer.push_back(frame.clone());
        
        // Limit buffer size
        while self.frame_buffer.len() > MAX_FRAME_BUFFER {
            self.frame_buffer.pop_front();
            self.stats.frames_dropped += 1;
        }
        
        // Update current frame
        self.current_frame = Some(frame);
    }
    
    /// Get metadata for a specific timestamp
    pub fn get_frame_metadata(&self, timestamp: gst::ClockTime) -> Option<Vec<ObjectMeta>> {
        // Find closest frame to the requested timestamp
        let target_ns = timestamp.nseconds()?;
        
        let mut best_frame = None;
        let mut best_diff = u64::MAX;
        
        for frame in &self.frame_buffer {
            if let Some(frame_ns) = frame.timestamp.nseconds() {
                let diff = if frame_ns > target_ns {
                    frame_ns - target_ns
                } else {
                    target_ns - frame_ns
                };
                
                if diff < best_diff {
                    best_diff = diff;
                    best_frame = Some(frame);
                }
                
                // Exact match
                if diff == 0 {
                    break;
                }
            }
        }
        
        best_frame.map(|f| f.objects.clone())
    }
    
    /// Get the current frame's objects
    pub fn get_current_objects(&self) -> Option<(Vec<ObjectMeta>, gst::ClockTime)> {
        self.current_frame.as_ref()
            .map(|f| (f.objects.clone(), f.timestamp))
    }
    
    /// Clear all buffered metadata
    pub fn clear(&mut self) {
        self.frame_buffer.clear();
        self.current_frame = None;
        self.stats.buffer_size = 0;
    }
    
    /// Get bridge statistics
    pub fn get_statistics(&self) -> BridgeStatistics {
        self.stats.clone()
    }
    
    /// Process inference results and prepare for rendering
    pub fn process_inference_results(
        &mut self,
        detections: Vec<crate::backend::cpu_vision::detector::Detection>,
        timestamp: gst::ClockTime,
        frame_width: u32,
        frame_height: u32,
    ) -> Vec<ObjectMeta> {
        let mut objects = Vec::new();
        
        for (i, detection) in detections.into_iter().enumerate() {
            let mut obj_meta = ObjectMeta::new(i as u64);
            
            // Set class information
            obj_meta.set_class(detection.class_id as i32, &detection.class_name);
            
            // Convert detection coordinates to bounding box
            let bbox = crate::metadata::object::BoundingBox::new(
                detection.x,
                detection.y,
                detection.width,
                detection.height,
            );
            
            // Set detection bbox with confidence
            obj_meta.set_detection_bbox(bbox, detection.confidence);
            
            objects.push(obj_meta);
        }
        
        // Update bridge with new objects
        self.update_objects(objects.clone(), timestamp);
        
        objects
    }
    
    /// Synchronize metadata with pipeline timing
    pub fn sync_with_pipeline(&mut self, pipeline_clock: &gst::Clock) -> Result<(), String> {
        let current_time = pipeline_clock.time();
        
        // Clean up old frames based on pipeline time
        if let Some(current_ns) = current_time.nseconds() {
            while let Some(oldest) = self.frame_buffer.front() {
                if let Some(oldest_ns) = oldest.timestamp.nseconds() {
                    if current_ns > oldest_ns && current_ns - oldest_ns > self.max_latency {
                        self.frame_buffer.pop_front();
                        self.stats.frames_dropped += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        
        self.stats.buffer_size = self.frame_buffer.len();
        Ok(())
    }
}

impl Default for MetadataBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe wrapper for MetadataBridge
pub struct SharedMetadataBridge {
    inner: Arc<parking_lot::RwLock<MetadataBridge>>,
}

impl SharedMetadataBridge {
    /// Create a new shared metadata bridge
    pub fn new() -> Self {
        Self {
            inner: Arc::new(parking_lot::RwLock::new(MetadataBridge::new())),
        }
    }
    
    /// Get a clone of the inner Arc for sharing between threads
    pub fn clone_inner(&self) -> Arc<parking_lot::RwLock<MetadataBridge>> {
        self.inner.clone()
    }
    
    /// Update objects (write operation)
    pub fn update_objects(&self, objects: Vec<ObjectMeta>, timestamp: gst::ClockTime) {
        self.inner.write().update_objects(objects, timestamp);
    }
    
    /// Get current objects (read operation)
    pub fn get_current_objects(&self) -> Option<(Vec<ObjectMeta>, gst::ClockTime)> {
        self.inner.read().get_current_objects()
    }
    
    /// Get frame metadata (read operation)
    pub fn get_frame_metadata(&self, timestamp: gst::ClockTime) -> Option<Vec<ObjectMeta>> {
        self.inner.read().get_frame_metadata(timestamp)
    }
    
    /// Clear all metadata (write operation)
    pub fn clear(&self) {
        self.inner.write().clear();
    }
    
    /// Get statistics (read operation)
    pub fn get_statistics(&self) -> BridgeStatistics {
        self.inner.read().get_statistics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metadata_bridge_creation() {
        let bridge = MetadataBridge::new();
        assert_eq!(bridge.stats.frames_processed, 0);
        assert!(bridge.current_frame.is_none());
    }
    
    #[test]
    fn test_update_objects() {
        gst::init().unwrap();
        
        let mut bridge = MetadataBridge::new();
        let mut objects = Vec::new();
        
        for i in 0..3 {
            let mut obj = ObjectMeta::new(i);
            obj.set_class(0, &format!("object_{}", i));
            objects.push(obj);
        }
        
        let timestamp = gst::ClockTime::from_seconds(1);
        bridge.update_objects(objects.clone(), timestamp);
        
        assert_eq!(bridge.stats.frames_processed, 1);
        assert_eq!(bridge.stats.buffer_size, 1);
        
        let current = bridge.get_current_objects();
        assert!(current.is_some());
        
        let (retrieved_objects, retrieved_timestamp) = current.unwrap();
        assert_eq!(retrieved_objects.len(), 3);
        assert_eq!(retrieved_timestamp, timestamp);
    }
    
    #[test]
    fn test_frame_buffer_overflow() {
        gst::init().unwrap();
        
        let mut bridge = MetadataBridge::new();
        
        // Add more than MAX_FRAME_BUFFER frames
        for i in 0..MAX_FRAME_BUFFER + 10 {
            let obj = ObjectMeta::new(i as u64);
            let timestamp = gst::ClockTime::from_seconds(i as u64);
            bridge.update_objects(vec![obj], timestamp);
        }
        
        // Buffer should be limited to MAX_FRAME_BUFFER
        assert!(bridge.frame_buffer.len() <= MAX_FRAME_BUFFER);
        assert!(bridge.stats.frames_dropped > 0);
    }
    
    #[test]
    fn test_shared_metadata_bridge() {
        gst::init().unwrap();
        
        let shared = SharedMetadataBridge::new();
        
        let obj = ObjectMeta::new(1);
        let timestamp = gst::ClockTime::from_seconds(1);
        
        shared.update_objects(vec![obj], timestamp);
        
        let stats = shared.get_statistics();
        assert_eq!(stats.frames_processed, 1);
        
        let current = shared.get_current_objects();
        assert!(current.is_some());
    }
}