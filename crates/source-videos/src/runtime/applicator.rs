use crate::error::{Result, SourceVideoError};
use crate::manager::VideoSourceManager;
use super::differ::{ConfigChange, ConfigDiffer, ChangePlan};
use std::sync::Arc;
use std::time::Instant;

pub struct ChangeApplicator {
    manager: Arc<VideoSourceManager>,
}

impl ChangeApplicator {
    pub fn new(manager: Arc<VideoSourceManager>) -> Self {
        Self { manager }
    }
    
    pub async fn apply_changes(&self, changes: Vec<ConfigChange>) -> Result<()> {
        if changes.is_empty() {
            return Ok(());
        }
        
        let start = Instant::now();
        log::info!("Applying {} configuration changes", changes.len());
        
        // Generate an ordered plan
        let differ = ConfigDiffer::new();
        let plan = differ.generate_change_plan(&changes);
        
        // Track applied changes for potential rollback
        let mut applied = Vec::new();
        
        // Apply changes in order
        let ordered_changes = plan.get_ordered_changes();
        for change in ordered_changes {
            match self.apply_change(change.clone()).await {
                Ok(()) => {
                    applied.push(change);
                }
                Err(e) => {
                    log::error!("Failed to apply change: {:?}, error: {}", change, e);
                    
                    // Attempt to rollback applied changes
                    if !applied.is_empty() {
                        log::info!("Attempting to rollback {} applied changes", applied.len());
                        self.rollback_changes(applied).await;
                    }
                    
                    return Err(e);
                }
            }
        }
        
        let elapsed = start.elapsed();
        log::info!("All changes applied successfully in {:?}", elapsed);
        
        Ok(())
    }
    
    pub async fn apply_change(&self, change: ConfigChange) -> Result<()> {
        match change {
            ConfigChange::SourceAdded { config } => {
                log::info!("Adding source: {}", config.name);
                self.manager.add_source(config)?;
            }
            
            ConfigChange::SourceRemoved { name } => {
                log::info!("Removing source: {}", name);
                self.manager.remove_source(&name)?;
            }
            
            ConfigChange::SourceModified { name, old_config: _, new_config } => {
                log::info!("Modifying source: {}", name);
                // For now, we'll remove and re-add the source
                // In the future, this could be optimized to update in-place
                self.manager.remove_source(&name)?;
                self.manager.add_source(new_config)?;
            }
            
            ConfigChange::ServerPortChanged { old_port: _, new_port } => {
                log::info!("Server port changed to: {}", new_port);
                // This would require restarting the RTSP server
                // For now, just log the change
                log::warn!("Server port changes require application restart");
            }
            
            ConfigChange::ServerAddressChanged { old_address: _, new_address } => {
                log::info!("Server address changed to: {}", new_address);
                // This would require restarting the RTSP server
                // For now, just log the change
                log::warn!("Server address changes require application restart");
            }
            
            ConfigChange::LogLevelChanged { old_level: _, new_level } => {
                log::info!("Log level changed to: {}", new_level);
                // Update the log level dynamically
                self.update_log_level(&new_level)?;
            }
        }
        
        Ok(())
    }
    
    async fn rollback_changes(&self, changes: Vec<ConfigChange>) {
        // Apply inverse operations in reverse order
        for change in changes.into_iter().rev() {
            let rollback_result = match change {
                ConfigChange::SourceAdded { config } => {
                    // Rollback: remove the added source
                    self.manager.remove_source(&config.name)
                }
                
                ConfigChange::SourceRemoved { name: _ } => {
                    // Rollback: cannot re-add removed source without config
                    // This would require storing the original config
                    log::warn!("Cannot rollback source removal without original configuration");
                    Ok(())
                }
                
                ConfigChange::SourceModified { name, old_config, new_config: _ } => {
                    // Rollback: restore the old configuration
                    self.manager.remove_source(&name)
                        .and_then(|_| self.manager.add_source(old_config))
                        .map(|_| ())
                }
                
                _ => Ok(()),
            };
            
            if let Err(e) = rollback_result {
                log::error!("Failed to rollback change: {}", e);
            }
        }
    }
    
    fn update_log_level(&self, level: &str) -> Result<()> {
        let log_level = match level.to_lowercase().as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" | "warning" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            "off" => log::LevelFilter::Off,
            _ => {
                return Err(SourceVideoError::config(format!("Invalid log level: {}", level)));
            }
        };
        
        log::set_max_level(log_level);
        log::info!("Log level updated to: {}", level);
        
        Ok(())
    }
    
    pub fn validate_dependencies(&self, changes: &[ConfigChange]) -> Result<()> {
        // Check for conflicting changes
        let mut source_names = std::collections::HashSet::new();
        
        for change in changes {
            match change {
                ConfigChange::SourceAdded { config } => {
                    if !source_names.insert(config.name.clone()) {
                        return Err(SourceVideoError::config(format!(
                            "Conflicting changes for source: {}",
                            config.name
                        )));
                    }
                }
                
                ConfigChange::SourceRemoved { name } | ConfigChange::SourceModified { name, .. } => {
                    if !source_names.insert(name.clone()) {
                        return Err(SourceVideoError::config(format!(
                            "Conflicting changes for source: {}",
                            name
                        )));
                    }
                }
                
                _ => {}
            }
        }
        
        Ok(())
    }
}

pub struct PerformanceMonitor {
    changes: Vec<(ConfigChange, std::time::Duration)>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
    
    pub fn record(&mut self, change: ConfigChange, duration: std::time::Duration) {
        self.changes.push((change, duration));
    }
    
    pub fn total_time(&self) -> std::time::Duration {
        self.changes.iter().map(|(_, d)| *d).sum()
    }
    
    pub fn average_time(&self) -> std::time::Duration {
        if self.changes.is_empty() {
            std::time::Duration::ZERO
        } else {
            self.total_time() / self.changes.len() as u32
        }
    }
    
    pub fn slowest_change(&self) -> Option<(&ConfigChange, std::time::Duration)> {
        self.changes.iter()
            .max_by_key(|(_, d)| *d)
            .map(|(c, d)| (c, *d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VideoSourceConfig;
    
    #[tokio::test]
    async fn test_applicator_creation() {
        gstreamer::init().unwrap();
        
        let manager = Arc::new(VideoSourceManager::new());
        let applicator = ChangeApplicator::new(manager);
        
        let changes = vec![];
        let result = applicator.apply_changes(changes).await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_dependencies() {
        gstreamer::init().unwrap();
        
        let manager = Arc::new(VideoSourceManager::new());
        let applicator = ChangeApplicator::new(manager);
        
        // No conflicts
        let changes = vec![
            ConfigChange::SourceAdded {
                config: VideoSourceConfig::test_pattern("source1", "smpte"),
            },
            ConfigChange::SourceAdded {
                config: VideoSourceConfig::test_pattern("source2", "ball"),
            },
        ];
        
        assert!(applicator.validate_dependencies(&changes).is_ok());
        
        // Conflicting changes
        let changes = vec![
            ConfigChange::SourceAdded {
                config: VideoSourceConfig::test_pattern("source1", "smpte"),
            },
            ConfigChange::SourceRemoved {
                name: "source1".to_string(),
            },
        ];
        
        assert!(applicator.validate_dependencies(&changes).is_err());
    }
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.record(
            ConfigChange::SourceAdded {
                config: VideoSourceConfig::test_pattern("test", "smpte"),
            },
            std::time::Duration::from_millis(50),
        );
        
        monitor.record(
            ConfigChange::SourceRemoved {
                name: "test".to_string(),
            },
            std::time::Duration::from_millis(30),
        );
        
        assert_eq!(monitor.total_time(), std::time::Duration::from_millis(80));
        assert_eq!(monitor.average_time(), std::time::Duration::from_millis(40));
        
        let slowest = monitor.slowest_change();
        assert!(slowest.is_some());
        assert_eq!(slowest.unwrap().1, std::time::Duration::from_millis(50));
    }
}