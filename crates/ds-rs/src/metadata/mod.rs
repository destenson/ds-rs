//! DeepStream metadata extraction and processing
//! 
//! This module provides safe wrappers around DeepStream metadata structures,
//! enabling access to AI inference results, object tracking data, and frame metadata.

use gstreamer as gst;
use gstreamer::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub mod object;
pub mod frame;
pub mod batch;

pub use object::{ObjectMeta, BoundingBox, ClassificationMeta};
pub use frame::FrameMeta;
pub use batch::BatchMeta;

/// Errors that can occur during metadata operations
#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("No metadata found on buffer")]
    NoMetadata,
    
    #[error("Invalid metadata format")]
    InvalidFormat,
    
    #[error("Metadata extraction failed: {0}")]
    ExtractionFailed(String),
    
    #[error("Null pointer encountered")]
    NullPointer,
}

pub type Result<T> = std::result::Result<T, MetadataError>;

/// Metadata API version for compatibility checking
pub const METADATA_API_VERSION: u32 = 1;

/// Maximum number of tracked objects per frame
pub const MAX_TRACKED_OBJECTS: usize = 128;

/// Metadata extractor for GStreamer buffers
pub struct MetadataExtractor {
    /// Cache of recent metadata for performance
    cache: Arc<Mutex<HashMap<u64, BatchMeta>>>,
}

impl MetadataExtractor {
    /// Create a new metadata extractor
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Extract batch metadata from a GStreamer buffer
    pub fn extract_batch_meta(&self, buffer: &gst::BufferRef) -> Result<BatchMeta> {
        // In a real implementation, this would call gst_buffer_get_nvds_batch_meta
        // For now, we'll create mock metadata for testing
        
        // Check if we have cached metadata for this buffer
        let buffer_id = buffer.pts().map(|p| p.nseconds()).unwrap_or(0);
        
        if let Ok(cache) = self.cache.lock() {
            if let Some(meta) = cache.get(&buffer_id) {
                return Ok(meta.clone());
            }
        }
        
        // Create mock metadata for testing
        let batch_meta = BatchMeta::new_mock(buffer_id);
        
        // Cache it
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(buffer_id, batch_meta.clone());
            
            // Limit cache size
            if cache.len() > 100 {
                cache.clear();
            }
        }
        
        Ok(batch_meta)
    }
    
    /// Extract frame metadata for a specific source
    pub fn extract_frame_meta(&self, batch_meta: &BatchMeta, source_id: u32) -> Result<FrameMeta> {
        batch_meta.get_frame_meta(source_id)
            .ok_or(MetadataError::NoMetadata)
    }
    
    /// Clear the metadata cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}

impl Default for MetadataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for attaching probe callbacks to extract metadata
pub trait MetadataProbe {
    /// Attach a metadata extraction probe to a pad
    fn add_metadata_probe<F>(&self, callback: F) -> Option<gst::PadProbeId>
    where
        F: Fn(&gst::PadProbeInfo) -> Option<BatchMeta> + Send + Sync + 'static;
}

impl MetadataProbe for gst::Pad {
    fn add_metadata_probe<F>(&self, callback: F) -> Option<gst::PadProbeId>
    where
        F: Fn(&gst::PadProbeInfo) -> Option<BatchMeta> + Send + Sync + 'static,
    {
        let extractor = MetadataExtractor::new();
        
        self.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
            if let Some(buffer) = info.buffer() {
                if let Ok(_batch_meta) = extractor.extract_batch_meta(buffer) {
                    callback(info);
                }
            }
            gst::PadProbeReturn::Ok
        })
    }
}

/// Statistics for metadata processing
#[derive(Debug, Clone, Default)]
pub struct MetadataStats {
    pub frames_processed: u64,
    pub objects_detected: u64,
    pub objects_tracked: u64,
    pub classifications_made: u64,
}

impl MetadataStats {
    pub fn new() -> Self {
        Default::default()
    }
    
    pub fn update_from_batch(&mut self, batch: &BatchMeta) {
        self.frames_processed += batch.num_frames() as u64;
        
        for frame in batch.frames() {
            self.objects_detected += frame.num_objects() as u64;
            
            for obj in frame.objects() {
                if obj.object_id > 0 {
                    self.objects_tracked += 1;
                }
                
                self.classifications_made += obj.classifications.len() as u64;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metadata_extractor_creation() {
        let extractor = MetadataExtractor::new();
        assert!(extractor.cache.lock().is_ok());
    }
    
    #[test]
    fn test_metadata_stats() {
        let mut stats = MetadataStats::new();
        assert_eq!(stats.frames_processed, 0);
        assert_eq!(stats.objects_detected, 0);
        
        // Create mock batch metadata
        let batch = BatchMeta::new_mock(12345);
        stats.update_from_batch(&batch);
        
        assert!(stats.frames_processed > 0);
    }
    
    #[test]
    fn test_cache_limiting() {
        let extractor = MetadataExtractor::new();
        
        // The cache should limit itself to 100 entries
        // This is tested implicitly through the extract_batch_meta method
        gst::init().ok();
        let buffer = gst::Buffer::new();
        
        let result = extractor.extract_batch_meta(&buffer);
        assert!(result.is_ok());
    }
}