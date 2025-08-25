use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::str::FromStr;
use super::{SourceId, SourceInfo, SourceManager, SourceState, VideoSource};

pub trait SourceAddition {
    fn add_video_source(&self, uri: &str) -> Result<SourceId>;
    fn add_source_with_id(&self, id: SourceId, uri: &str) -> Result<()>;
    fn add_multiple_sources(&self, uris: &[String]) -> Result<Vec<SourceId>>;
}

impl SourceAddition for SourceManager {
    fn add_video_source(&self, uri: &str) -> Result<SourceId> {
        let source_id = self.generate_source_id()?;
        self.add_source_with_id(source_id, uri)?;
        Ok(source_id)
    }
    
    fn add_source_with_id(&self, id: SourceId, uri: &str) -> Result<()> {
        let pipeline = self.get_pipeline()
            .ok_or_else(|| DeepStreamError::NotInitialized("Pipeline not set".to_string()))?;
        
        let streammux = self.get_streammux()
            .ok_or_else(|| DeepStreamError::NotInitialized("Streammux not set".to_string()))?;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        println!("[{:.3}] Adding source {} with URI: {}", now.as_secs_f64(), id, uri);
        
        let mut video_source = VideoSource::new(id, uri)?;
        
        video_source.connect_pad_added_default(streammux)?;
        
        let source_element = video_source.element();
        pipeline.add_element(source_element)?;
        
        // For test sources, connect after adding to pipeline
        if uri == "videotestsrc://" {
            video_source.connect_test_source(streammux)?;
        }
        
        video_source.update_state(SourceState::Initializing)?;
        
        // CRITICAL: Use sync_state_with_parent() instead of set_state() for dynamic elements
        // This ensures the element inherits the pipeline's clock and base time
        println!("[{:.3}] Syncing source {} state with parent pipeline", crate::timestamp(), id);
        source_element.sync_state_with_parent()?;
        
        println!("[{:.3}] Source {} successfully synced with parent pipeline", crate::timestamp(), id);
        video_source.update_state(SourceState::Playing)?;
        
        let source_info = SourceInfo {
            id,
            uri: uri.to_string(),
            source: video_source,
            state: SourceState::Playing,
            enabled: true,
        };
        
        self.add_source(id, source_info)?;
        
        println!("Successfully added source {} - Total sources: {}", 
                 id, self.num_sources()?);
        
        Ok(())
    }
    
    fn add_multiple_sources(&self, uris: &[String]) -> Result<Vec<SourceId>> {
        let mut source_ids = Vec::new();
        
        for uri in uris {
            match self.add_video_source(uri) {
                Ok(id) => source_ids.push(id),
                Err(e) => {
                    eprintln!("Failed to add source {}: {:?}", uri, e);
                    for added_id in &source_ids {
                        if let Err(remove_err) = self.remove_source(*added_id) {
                            eprintln!("Failed to rollback source {}: {:?}", added_id, remove_err);
                        }
                    }
                    return Err(e);
                }
            }
        }
        
        Ok(source_ids)
    }
}

pub struct SourceAddConfig {
    pub uri: String,
    pub buffer_size: Option<i32>,
    pub caps: Option<String>,
    pub do_timestamp: Option<bool>,
}

impl SourceAddConfig {
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            buffer_size: None,
            caps: None,
            do_timestamp: None,
        }
    }
    
    pub fn with_buffer_size(mut self, size: i32) -> Self {
        self.buffer_size = Some(size);
        self
    }
    
    pub fn with_caps(mut self, caps: impl Into<String>) -> Self {
        self.caps = Some(caps.into());
        self
    }
    
    pub fn with_timestamp(mut self, timestamp: bool) -> Self {
        self.do_timestamp = Some(timestamp);
        self
    }
}

pub fn validate_uri(uri: &str) -> Result<()> {
    if uri.is_empty() {
        return Err(DeepStreamError::InvalidInput("URI cannot be empty".to_string()));
    }
    
    if !uri.starts_with("file://") && 
       !uri.starts_with("rtsp://") && 
       !uri.starts_with("http://") && 
       !uri.starts_with("https://") &&
       !uri.starts_with("rtspt://") {
        return Err(DeepStreamError::InvalidInput(
            format!("Invalid URI scheme. Supported: file://, rtsp://, http://, https://. Got: {}", uri)
        ));
    }
    
    Ok(())
}

pub fn create_source_bin_with_config(
    source_id: SourceId,
    config: &SourceAddConfig,
) -> Result<VideoSource> {
    validate_uri(&config.uri)?;
    
    let source = VideoSource::new(source_id, &config.uri)?;
    
    let element = source.element();
    
    if let Some(buffer_size) = config.buffer_size {
        element.set_property("buffer-size", buffer_size);
    }
    
    if let Some(ref caps_str) = config.caps {
        if let Ok(caps) = gst::Caps::from_str(caps_str) {
            element.set_property("caps", &caps);
        }
    }
    
    if let Some(do_timestamp) = config.do_timestamp {
        element.set_property("do-timestamp", do_timestamp);
    }
    
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_uri() {
        assert!(validate_uri("file:///tmp/test.mp4").is_ok());
        assert!(validate_uri("rtsp://localhost:8554/stream").is_ok());
        assert!(validate_uri("http://example.com/video.mp4").is_ok());
        assert!(validate_uri("https://example.com/video.mp4").is_ok());
        
        assert!(validate_uri("").is_err());
        assert!(validate_uri("invalid://uri").is_err());
        assert!(validate_uri("/tmp/test.mp4").is_err());
    }
    
    #[test]
    fn test_source_add_config() {
        let config = SourceAddConfig::new("file:///tmp/test.mp4")
            .with_buffer_size(4096)
            .with_caps("video/x-raw")
            .with_timestamp(true);
        
        assert_eq!(config.uri, "file:///tmp/test.mp4");
        assert_eq!(config.buffer_size, Some(4096));
        assert_eq!(config.caps, Some("video/x-raw".to_string()));
        assert_eq!(config.do_timestamp, Some(true));
    }
}
