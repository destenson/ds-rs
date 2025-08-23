#![allow(unused)]
pub mod video_source;
pub mod manager;
pub mod removal;
pub mod events;
pub mod synchronization;
pub mod controller;

use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicUsize;
use crate::pipeline::Pipeline;

pub use video_source::VideoSource;
pub use manager::SourceAddition;
pub use removal::SourceRemoval;
pub use events::{SourceEvent, SourceEventHandler};
pub use synchronization::SourceSynchronizer;
pub use controller::SourceController;

pub const MAX_NUM_SOURCES: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(pub usize);

impl std::fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source-{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceState {
    Idle,
    Initializing,
    Playing,
    Paused,
    Stopping,
    Stopped,
    Error(String),
}

pub struct SourceInfo {
    pub id: SourceId,
    pub uri: String,
    pub source: VideoSource,
    pub state: SourceState,
    pub enabled: bool,
}

pub struct SourceManager {
    sources: Arc<RwLock<HashMap<SourceId, SourceInfo>>>,
    next_id: AtomicUsize,
    max_sources: usize,
    source_enabled: Arc<RwLock<Vec<bool>>>,
    pipeline: Option<Arc<Pipeline>>,
    streammux: Option<gst::Element>,
}

impl SourceManager {
    pub fn new(max_sources: usize) -> Self {
        let mut source_enabled = Vec::with_capacity(max_sources);
        source_enabled.resize(max_sources, false);
        
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicUsize::new(0),
            max_sources,
            source_enabled: Arc::new(RwLock::new(source_enabled)),
            pipeline: None,
            streammux: None,
        }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(MAX_NUM_SOURCES)
    }
    
    pub fn set_pipeline(&mut self, pipeline: Arc<Pipeline>) {
        self.pipeline = Some(pipeline);
    }
    
    pub fn set_streammux(&mut self, streammux: gst::Element) {
        self.streammux = Some(streammux);
    }
    
    pub fn generate_source_id(&self) -> Result<SourceId> {
        let enabled = self.source_enabled.read()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock source_enabled".to_string()))?;
        
        for i in 0..self.max_sources {
            if !enabled[i] {
                return Ok(SourceId(i));
            }
        }
        
        Err(DeepStreamError::Pipeline(
            format!("Maximum number of sources ({}) reached", self.max_sources)
        ))
    }
    
    pub fn mark_source_enabled(&self, id: SourceId, enabled: bool) -> Result<()> {
        let mut source_enabled = self.source_enabled.write()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock source_enabled".to_string()))?;
        
        if id.0 >= self.max_sources {
            return Err(DeepStreamError::InvalidInput(
                format!("Source ID {} exceeds maximum {}", id.0, self.max_sources)
            ));
        }
        
        source_enabled[id.0] = enabled;
        Ok(())
    }
    
    pub fn add_source(&self, id: SourceId, info: SourceInfo) -> Result<()> {
        let mut sources = self.sources.write()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock sources".to_string()))?;
        
        if sources.contains_key(&id) {
            return Err(DeepStreamError::InvalidInput(
                format!("Source {} already exists", id)
            ));
        }
        
        sources.insert(id, info);
        self.mark_source_enabled(id, true)?;
        Ok(())
    }
    
    pub fn remove_source(&self, id: SourceId) -> Result<SourceInfo> {
        let mut sources = self.sources.write()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock sources".to_string()))?;
        
        let info = sources.remove(&id)
            .ok_or_else(|| DeepStreamError::InvalidInput(
                format!("Source {} not found", id)
            ))?;
        
        self.mark_source_enabled(id, false)?;
        Ok(info)
    }
    
    pub fn get_source(&self, id: SourceId) -> Result<VideoSource> {
        let sources = self.sources.read()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock sources".to_string()))?;
        
        sources.get(&id)
            .map(|info| info.source.clone())
            .ok_or_else(|| DeepStreamError::InvalidInput(
                format!("Source {} not found", id)
            ))
    }
    
    pub fn get_source_info(&self, id: SourceId) -> Result<SourceInfo> {
        let sources = self.sources.read()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock sources".to_string()))?;
        
        sources.get(&id)
            .cloned()
            .ok_or_else(|| DeepStreamError::InvalidInput(
                format!("Source {} not found", id)
            ))
    }
    
    pub fn update_source_state(&self, id: SourceId, state: SourceState) -> Result<()> {
        let mut sources = self.sources.write()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock sources".to_string()))?;
        
        let info = sources.get_mut(&id)
            .ok_or_else(|| DeepStreamError::InvalidInput(
                format!("Source {} not found", id)
            ))?;
        
        info.state = state;
        Ok(())
    }
    
    pub fn list_sources(&self) -> Result<Vec<SourceId>> {
        let sources = self.sources.read()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock sources".to_string()))?;
        
        Ok(sources.keys().cloned().collect())
    }
    
    pub fn num_sources(&self) -> Result<usize> {
        let sources = self.sources.read()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock sources".to_string()))?;
        
        Ok(sources.len())
    }
    
    pub fn is_source_enabled(&self, id: SourceId) -> Result<bool> {
        let enabled = self.source_enabled.read()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock source_enabled".to_string()))?;
        
        if id.0 >= self.max_sources {
            return Ok(false);
        }
        
        Ok(enabled[id.0])
    }
    
    pub fn get_pipeline(&self) -> Option<Arc<Pipeline>> {
        self.pipeline.clone()
    }
    
    pub fn get_streammux(&self) -> Option<&gst::Element> {
        self.streammux.as_ref()
    }
}

impl Clone for SourceInfo {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            uri: self.uri.clone(),
            source: self.source.clone(),
            state: self.state.clone(),
            enabled: self.enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_source_manager_creation() {
        let manager = SourceManager::with_defaults();
        assert_eq!(manager.max_sources, MAX_NUM_SOURCES);
        assert_eq!(manager.num_sources().unwrap(), 0);
    }
    
    #[test]
    fn test_source_id_generation() {
        let manager = SourceManager::new(3);
        
        let id1 = manager.generate_source_id().unwrap();
        manager.mark_source_enabled(id1, true).unwrap();
        
        let id2 = manager.generate_source_id().unwrap();
        manager.mark_source_enabled(id2, true).unwrap();
        
        let id3 = manager.generate_source_id().unwrap();
        manager.mark_source_enabled(id3, true).unwrap();
        
        assert!(manager.generate_source_id().is_err());
        
        manager.mark_source_enabled(id2, false).unwrap();
        let id4 = manager.generate_source_id().unwrap();
        assert_eq!(id4.0, id2.0);
    }
}
