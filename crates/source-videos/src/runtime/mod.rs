pub mod events;
pub mod differ;
pub mod applicator;
pub mod signal_handler;

use crate::config::{AppConfig, VideoSourceConfig};
use crate::error::{Result, SourceVideoError};
use crate::manager::VideoSourceManager;
use events::{ConfigurationEvent, EventBus};
use differ::{ConfigDiffer, ConfigChange};
use applicator::ChangeApplicator;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::VecDeque;

pub struct RuntimeManager {
    manager: Arc<VideoSourceManager>,
    event_bus: Arc<EventBus>,
    current_config: Arc<RwLock<AppConfig>>,
    config_history: Arc<RwLock<VecDeque<AppConfig>>>,
    max_history: usize,
}

impl RuntimeManager {
    pub fn new(manager: Arc<VideoSourceManager>, initial_config: AppConfig) -> Self {
        Self {
            manager,
            event_bus: Arc::new(EventBus::new()),
            current_config: Arc::new(RwLock::new(initial_config)),
            config_history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 10,
        }
    }
    
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }
    
    pub async fn apply_config(&self, new_config: AppConfig) -> Result<()> {
        let current = self.current_config.read().await;
        
        // Detect changes
        let differ = ConfigDiffer::new();
        let changes = differ.diff(&*current, &new_config);
        
        if changes.is_empty() {
            log::info!("No configuration changes detected");
            return Ok(());
        }
        
        log::info!("Detected {} configuration changes", changes.len());
        
        // Create a snapshot for potential rollback
        let snapshot = (*current).clone();
        drop(current);
        
        // Apply changes
        let applicator = ChangeApplicator::new(self.manager.clone());
        
        match applicator.apply_changes(changes.clone()).await {
            Ok(()) => {
                // Update current config
                let mut current = self.current_config.write().await;
                *current = new_config.clone();
                
                // Add to history
                let mut history = self.config_history.write().await;
                history.push_back(snapshot);
                if history.len() > self.max_history {
                    history.pop_front();
                }
                
                // Emit success event
                self.event_bus.emit(ConfigurationEvent::ConfigApplied {
                    changes: changes.len(),
                }).await;
                
                log::info!("Configuration successfully applied");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to apply configuration: {}", e);
                
                // Attempt rollback
                self.rollback_to(snapshot).await?;
                
                // Emit failure event
                self.event_bus.emit(ConfigurationEvent::ConfigFailed {
                    error: e.to_string(),
                }).await;
                
                Err(e)
            }
        }
    }
    
    pub async fn rollback(&self) -> Result<()> {
        let mut history = self.config_history.write().await;
        
        if let Some(previous) = history.pop_back() {
            drop(history);
            self.rollback_to(previous).await
        } else {
            Err(SourceVideoError::config("No configuration history available for rollback"))
        }
    }
    
    async fn rollback_to(&self, config: AppConfig) -> Result<()> {
        log::info!("Rolling back to previous configuration");
        
        let current = self.current_config.read().await;
        let differ = ConfigDiffer::new();
        let changes = differ.diff(&*current, &config);
        drop(current);
        
        let applicator = ChangeApplicator::new(self.manager.clone());
        applicator.apply_changes(changes).await?;
        
        let mut current = self.current_config.write().await;
        *current = config;
        
        self.event_bus.emit(ConfigurationEvent::ConfigRolledBack).await;
        
        log::info!("Rollback completed successfully");
        Ok(())
    }
    
    pub async fn get_current_config(&self) -> AppConfig {
        self.current_config.read().await.clone()
    }
    
    pub fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<ConfigurationEvent> {
        self.event_bus.subscribe()
    }
    
    pub async fn update_source(&self, source_name: &str, config: VideoSourceConfig) -> Result<()> {
        let mut current = self.current_config.write().await;
        
        // Find and update the source in config
        let source_index = current.sources.iter()
            .position(|s| s.name == source_name)
            .ok_or_else(|| SourceVideoError::config(format!("Source '{}' not found", source_name)))?;
        
        let old_config = current.sources[source_index].clone();
        current.sources[source_index] = config.clone();
        
        // Apply the change
        let change = ConfigChange::SourceModified {
            name: source_name.to_string(),
            old_config,
            new_config: config,
        };
        
        let applicator = ChangeApplicator::new(self.manager.clone());
        applicator.apply_change(change).await?;
        
        self.event_bus.emit(ConfigurationEvent::SourceUpdated {
            source: source_name.to_string(),
        }).await;
        
        Ok(())
    }
    
    pub async fn add_source(&self, config: VideoSourceConfig) -> Result<String> {
        let source_id = self.manager.add_source(config.clone())?;
        
        let mut current = self.current_config.write().await;
        current.sources.push(config.clone());
        
        self.event_bus.emit(ConfigurationEvent::SourceAdded {
            source: config.name.clone(),
        }).await;
        
        Ok(source_id)
    }
    
    pub async fn remove_source(&self, source_name: &str) -> Result<()> {
        self.manager.remove_source(source_name)?;
        
        let mut current = self.current_config.write().await;
        current.sources.retain(|s| s.name != source_name);
        
        self.event_bus.emit(ConfigurationEvent::SourceRemoved {
            source: source_name.to_string(),
        }).await;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_runtime_manager_creation() {
        gstreamer::init().unwrap();
        
        let manager = Arc::new(VideoSourceManager::new());
        let config = AppConfig::default();
        let runtime = RuntimeManager::new(manager, config);
        
        let current = runtime.get_current_config().await;
        assert_eq!(current.sources.len(), 2); // Default has 2 test sources
    }
}