use crate::config_types::{VideoSourceConfig, VideoSourceType, DirectoryConfig, FileListConfig};
use crate::directory::{DirectoryScanner, BatchSourceLoader};
use crate::error::{Result, SourceVideoError};
use crate::source::{VideoSource, SourceState, create_source};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub struct VideoSourceManager {
    sources: Arc<RwLock<HashMap<String, Box<dyn VideoSource>>>>,
    name_to_id: Arc<RwLock<HashMap<String, String>>>,
}

impl VideoSourceManager {
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn add_source(&self, config: VideoSourceConfig) -> Result<String> {
        let mut source = create_source(config.clone());
        let id = source.get_id().to_string();
        let name = source.get_name().to_string();
        
        {
            let mut sources = self.sources.write()
                .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on sources"))?;
            
            let mut name_map = self.name_to_id.write()
                .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on name map"))?;
            
            if name_map.contains_key(&name) {
                return Err(SourceVideoError::config(format!("Source with name '{}' already exists", name)));
            }
            
            source.start()?;
            
            sources.insert(id.clone(), source);
            name_map.insert(name.clone(), id.clone());
        }
        
        log::info!("Added source '{}' with ID: {}", name, id);
        Ok(id)
    }
    
    pub fn remove_source(&self, id_or_name: &str) -> Result<()> {
        let id = self.resolve_id(id_or_name)?;
        
        {
            let mut sources = self.sources.write()
                .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on sources"))?;
            
            if let Some(mut source) = sources.remove(&id) {
                source.stop()?;
                
                let name = source.get_name().to_string();
                
                let mut name_map = self.name_to_id.write()
                    .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on name map"))?;
                name_map.remove(&name);
                
                log::info!("Removed source '{}' (ID: {})", name, id);
                Ok(())
            } else {
                Err(SourceVideoError::SourceNotFound(id_or_name.to_string()))
            }
        }
    }
    
    pub fn get_source(&self, id_or_name: &str) -> Result<SourceInfo> {
        let id = self.resolve_id(id_or_name)?;
        
        let sources = self.sources.read()
            .map_err(|_| SourceVideoError::resource("Failed to acquire read lock on sources"))?;
        
        if let Some(source) = sources.get(&id) {
            Ok(SourceInfo {
                id: source.get_id().to_string(),
                name: source.get_name().to_string(),
                uri: source.get_uri(),
                state: source.get_state(),
            })
        } else {
            Err(SourceVideoError::SourceNotFound(id_or_name.to_string()))
        }
    }
    
    pub fn list_sources(&self) -> Vec<SourceInfo> {
        let sources = self.sources.read()
            .map(|sources| {
                sources.values()
                    .map(|source| SourceInfo {
                        id: source.get_id().to_string(),
                        name: source.get_name().to_string(),
                        uri: source.get_uri(),
                        state: source.get_state(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        sources
    }
    
    pub fn pause_source(&self, id_or_name: &str) -> Result<()> {
        let id = self.resolve_id(id_or_name)?;
        
        let mut sources = self.sources.write()
            .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on sources"))?;
        
        if let Some(source) = sources.get_mut(&id) {
            source.pause()
        } else {
            Err(SourceVideoError::SourceNotFound(id_or_name.to_string()))
        }
    }
    
    pub fn resume_source(&self, id_or_name: &str) -> Result<()> {
        let id = self.resolve_id(id_or_name)?;
        
        let mut sources = self.sources.write()
            .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on sources"))?;
        
        if let Some(source) = sources.get_mut(&id) {
            source.resume()
        } else {
            Err(SourceVideoError::SourceNotFound(id_or_name.to_string()))
        }
    }
    
    pub fn stop_source(&self, id_or_name: &str) -> Result<()> {
        let id = self.resolve_id(id_or_name)?;
        
        let mut sources = self.sources.write()
            .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on sources"))?;
        
        if let Some(source) = sources.get_mut(&id) {
            source.stop()
        } else {
            Err(SourceVideoError::SourceNotFound(id_or_name.to_string()))
        }
    }
    
    pub fn start_source(&self, id_or_name: &str) -> Result<()> {
        let id = self.resolve_id(id_or_name)?;
        
        let mut sources = self.sources.write()
            .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on sources"))?;
        
        if let Some(source) = sources.get_mut(&id) {
            source.start()
        } else {
            Err(SourceVideoError::SourceNotFound(id_or_name.to_string()))
        }
    }
    
    pub fn clear_all(&self) -> Result<()> {
        let mut sources = self.sources.write()
            .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on sources"))?;
        
        for (_, mut source) in sources.drain() {
            let _ = source.stop();
        }
        
        let mut name_map = self.name_to_id.write()
            .map_err(|_| SourceVideoError::resource("Failed to acquire write lock on name map"))?;
        name_map.clear();
        
        log::info!("Cleared all sources");
        Ok(())
    }
    
    pub fn source_count(&self) -> usize {
        self.sources.read()
            .map(|sources| sources.len())
            .unwrap_or(0)
    }
    
    pub fn update_source(&self, id_or_name: &str, config: VideoSourceConfig) -> Result<()> {
        let id = self.resolve_id(id_or_name)?;
        
        // Get the current source to preserve its state
        let current_state = {
            let sources = self.sources.read()
                .map_err(|_| SourceVideoError::resource("Failed to acquire read lock on sources"))?;
            
            sources.get(&id)
                .map(|s| s.get_state())
                .ok_or_else(|| SourceVideoError::SourceNotFound(id_or_name.to_string()))?
        };
        
        // Remove the old source
        self.remove_source(&id)?;
        
        // Add the new source with updated config
        let new_id = self.add_source(config)?;
        
        // Try to restore the previous state
        match current_state {
            SourceState::Paused => self.pause_source(&new_id)?,
            SourceState::Stopped => self.stop_source(&new_id)?,
            _ => {} // Playing state is default after add_source
        }
        
        log::info!("Updated source '{}' configuration", id_or_name);
        Ok(())
    }
    
    pub fn modify_source_config<F>(&self, _id_or_name: &str, _modify_fn: F) -> Result<()>
    where
        F: FnOnce(&mut VideoSourceConfig) -> Result<()>,
    {
        // This would require storing the config with each source
        // For now, this is a placeholder for future enhancement
        log::warn!("modify_source_config not yet implemented - using update_source instead");
        Err(SourceVideoError::config("In-place modification not yet supported"))
    }
    
    pub fn batch_update(&self, updates: Vec<(String, VideoSourceConfig)>) -> Result<()> {
        let mut errors = Vec::new();
        
        for (name, config) in updates {
            if let Err(e) = self.update_source(&name, config) {
                errors.push(format!("{}: {}", name, e));
            }
        }
        
        if !errors.is_empty() {
            return Err(SourceVideoError::config(format!(
                "Batch update failed for sources: {}",
                errors.join(", ")
            )));
        }
        
        Ok(())
    }
    
    pub fn get_source_configs(&self) -> Vec<(String, SourceState)> {
        self.sources.read()
            .map(|sources| {
                sources.values()
                    .map(|source| (source.get_name().to_string(), source.get_state()))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    pub fn snapshot(&self) -> ManagerSnapshot {
        let sources = self.list_sources();
        ManagerSnapshot { sources }
    }
    
    fn resolve_id(&self, id_or_name: &str) -> Result<String> {
        if Uuid::parse_str(id_or_name).is_ok() {
            Ok(id_or_name.to_string())
        } else {
            let name_map = self.name_to_id.read()
                .map_err(|_| SourceVideoError::resource("Failed to acquire read lock on name map"))?;
            
            name_map.get(id_or_name)
                .cloned()
                .ok_or_else(|| SourceVideoError::SourceNotFound(id_or_name.to_string()))
        }
    }
    
    // Batch operations for directory and file list support
    
    pub fn add_sources_batch(&self, configs: Vec<VideoSourceConfig>) -> Result<Vec<String>> {
        let mut added_ids = Vec::new();
        let mut errors = Vec::new();
        
        for config in configs {
            match self.add_source(config.clone()) {
                Ok(id) => {
                    added_ids.push(id);
                }
                Err(e) => {
                    errors.push(format!("{}: {}", config.name, e));
                }
            }
        }
        
        if !errors.is_empty() {
            log::warn!("Some sources failed to add: {}", errors.join(", "));
        }
        
        log::info!("Added {} sources in batch", added_ids.len());
        Ok(added_ids)
    }
    
    pub fn add_directory(&self, config: DirectoryConfig) -> Result<Vec<String>> {
        let mut scanner = DirectoryScanner::new(config.clone());
        let source_configs = scanner.scan()?;
        
        log::info!(
            "Found {} video files in directory: {}",
            source_configs.len(),
            config.path
        );
        
        if config.lazy_loading {
            // Add sources gradually in background
            // For now, just add all at once
            // TODO: Implement progressive loading
            self.add_sources_batch(source_configs)
        } else {
            self.add_sources_batch(source_configs)
        }
    }
    
    pub fn add_file_list(&self, config: FileListConfig) -> Result<Vec<String>> {
        let mut source_configs = Vec::new();
        
        for (index, file_path) in config.files.iter().enumerate() {
            let container = crate::file_utils::detect_container_format(std::path::Path::new(file_path))
                .unwrap_or(crate::config_types::FileContainer::Mp4);
            
            let name = format!(
                "file_{}_{}",
                index,
                std::path::Path::new(file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("video")
            );
            
            let source_config = VideoSourceConfig {
                name,
                source_type: VideoSourceType::File {
                    path: file_path.clone(),
                    container,
                },
                resolution: crate::config_types::Resolution {
                    width: 1920,
                    height: 1080,
                },
                framerate: crate::config_types::Framerate {
                    numerator: 30,
                    denominator: 1,
                },
                format: crate::config_types::VideoFormat::I420,
                duration: None,
                num_buffers: None,
                is_live: false,
            };
            
            source_configs.push(source_config);
        }
        
        if config.lazy_loading {
            // TODO: Implement lazy loading
            self.add_sources_batch(source_configs)
        } else {
            self.add_sources_batch(source_configs)
        }
    }
    
    pub fn add_from_batch_loader(&self, loader: &mut BatchSourceLoader) -> Result<Vec<String>> {
        let configs = loader.load_all()?;
        self.add_sources_batch(configs)
    }
}

impl Default for VideoSourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for VideoSourceManager {
    fn drop(&mut self) {
        let _ = self.clear_all();
    }
}

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub id: String,
    pub name: String,
    pub uri: String,
    pub state: SourceState,
}

#[derive(Debug, Clone)]
pub struct ManagerSnapshot {
    pub sources: Vec<SourceInfo>,
}

impl SourceInfo {
    pub fn is_playing(&self) -> bool {
        matches!(self.state, SourceState::Playing)
    }
    
    pub fn is_stopped(&self) -> bool {
        matches!(self.state, SourceState::Stopped)
    }
    
    pub fn is_error(&self) -> bool {
        matches!(self.state, SourceState::Error(_))
    }
}

pub struct SourceManagerBuilder {
    configs: Vec<VideoSourceConfig>,
}

impl SourceManagerBuilder {
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }
    
    pub fn add_config(mut self, config: VideoSourceConfig) -> Self {
        self.configs.push(config);
        self
    }
    
    pub fn add_test_pattern(mut self, name: &str, pattern: &str) -> Self {
        let config = VideoSourceConfig::test_pattern(name, pattern);
        self.configs.push(config);
        self
    }
    
    pub fn build(self) -> Result<VideoSourceManager> {
        let manager = VideoSourceManager::new();
        
        for config in self.configs {
            manager.add_source(config)?;
        }
        
        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manager_creation() {
        let manager = VideoSourceManager::new();
        assert_eq!(manager.source_count(), 0);
    }
    
    #[test]
    fn test_add_and_remove_source() {
        gstreamer::init().unwrap();
        
        let manager = VideoSourceManager::new();
        let config = VideoSourceConfig::test_pattern("test", "smpte");
        
        let id = manager.add_source(config).unwrap();
        assert_eq!(manager.source_count(), 1);
        
        let info = manager.get_source("test").unwrap();
        assert_eq!(info.name, "test");
        assert_eq!(info.id, id);
        
        manager.remove_source("test").unwrap();
        assert_eq!(manager.source_count(), 0);
    }
    
    #[test]
    fn test_source_lifecycle() {
        gstreamer::init().unwrap();
        
        let manager = VideoSourceManager::new();
        let config = VideoSourceConfig::test_pattern("lifecycle", "ball");
        
        manager.add_source(config).unwrap();
        
        let info = manager.get_source("lifecycle").unwrap();
        assert!(info.is_playing());
        
        manager.pause_source("lifecycle").unwrap();
        let info = manager.get_source("lifecycle").unwrap();
        assert_eq!(info.state, SourceState::Paused);
        
        manager.resume_source("lifecycle").unwrap();
        let info = manager.get_source("lifecycle").unwrap();
        assert!(info.is_playing());
        
        manager.stop_source("lifecycle").unwrap();
        let info = manager.get_source("lifecycle").unwrap();
        assert!(info.is_stopped());
    }
    
    #[test]
    fn test_builder() {
        gstreamer::init().unwrap();
        
        let manager = SourceManagerBuilder::new()
            .add_test_pattern("test1", "smpte")
            .add_test_pattern("test2", "ball")
            .build()
            .unwrap();
        
        assert_eq!(manager.source_count(), 2);
        
        let sources = manager.list_sources();
        assert_eq!(sources.len(), 2);
    }
}