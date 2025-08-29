#![allow(unused)]
use crate::config_types::{AppConfig, VideoSourceConfig};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigChange {
    SourceAdded {
        config: VideoSourceConfig,
    },
    SourceRemoved {
        name: String,
    },
    SourceModified {
        name: String,
        old_config: VideoSourceConfig,
        new_config: VideoSourceConfig,
    },
    ServerPortChanged {
        old_port: u16,
        new_port: u16,
    },
    ServerAddressChanged {
        old_address: String,
        new_address: String,
    },
    LogLevelChanged {
        old_level: String,
        new_level: String,
    },
}

pub struct ConfigDiffer;

impl ConfigDiffer {
    pub fn new() -> Self {
        Self
    }

    pub fn diff(&self, old: &AppConfig, new: &AppConfig) -> Vec<ConfigChange> {
        let mut changes = Vec::new();

        // Check server configuration changes
        if old.server.port != new.server.port {
            changes.push(ConfigChange::ServerPortChanged {
                old_port: old.server.port,
                new_port: new.server.port,
            });
        }

        if old.server.address != new.server.address {
            changes.push(ConfigChange::ServerAddressChanged {
                old_address: old.server.address.clone(),
                new_address: new.server.address.clone(),
            });
        }

        // Check log level changes
        if old.log_level != new.log_level {
            changes.push(ConfigChange::LogLevelChanged {
                old_level: old.log_level.clone(),
                new_level: new.log_level.clone(),
            });
        }

        // Check source changes
        let old_sources: HashMap<String, VideoSourceConfig> = old
            .sources
            .iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect();

        let new_sources: HashMap<String, VideoSourceConfig> = new
            .sources
            .iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect();

        let old_names: HashSet<String> = old_sources.keys().cloned().collect();
        let new_names: HashSet<String> = new_sources.keys().cloned().collect();

        // Find added sources
        for name in new_names.difference(&old_names) {
            if let Some(config) = new_sources.get(name) {
                changes.push(ConfigChange::SourceAdded {
                    config: config.clone(),
                });
            }
        }

        // Find removed sources
        for name in old_names.difference(&new_names) {
            changes.push(ConfigChange::SourceRemoved { name: name.clone() });
        }

        // Find modified sources
        for name in old_names.intersection(&new_names) {
            if let (Some(old_config), Some(new_config)) =
                (old_sources.get(name), new_sources.get(name))
            {
                if !self.sources_equal(old_config, new_config) {
                    changes.push(ConfigChange::SourceModified {
                        name: name.clone(),
                        old_config: old_config.clone(),
                        new_config: new_config.clone(),
                    });
                }
            }
        }

        changes
    }

    fn sources_equal(&self, old: &VideoSourceConfig, new: &VideoSourceConfig) -> bool {
        // Compare all relevant fields
        old.name == new.name
            && old.resolution.width == new.resolution.width
            && old.resolution.height == new.resolution.height
            && old.framerate.numerator == new.framerate.numerator
            && old.framerate.denominator == new.framerate.denominator
            && format!("{:?}", old.format) == format!("{:?}", new.format)
            && old.duration == new.duration
            && old.num_buffers == new.num_buffers
            && old.is_live == new.is_live
            && format!("{:?}", old.source_type) == format!("{:?}", new.source_type)
    }

    pub fn generate_change_plan(&self, changes: &[ConfigChange]) -> ChangePlan {
        let mut plan = ChangePlan::new();

        // Separate changes by type for ordered application
        for change in changes {
            match change {
                ConfigChange::SourceRemoved { .. } => plan.removals.push(change.clone()),
                ConfigChange::SourceAdded { .. } => plan.additions.push(change.clone()),
                ConfigChange::SourceModified { .. } => plan.modifications.push(change.clone()),
                _ => plan.other.push(change.clone()),
            }
        }

        plan
    }
}

pub struct ChangePlan {
    pub removals: Vec<ConfigChange>,
    pub modifications: Vec<ConfigChange>,
    pub additions: Vec<ConfigChange>,
    pub other: Vec<ConfigChange>,
}

impl ChangePlan {
    pub fn new() -> Self {
        Self {
            removals: Vec::new(),
            modifications: Vec::new(),
            additions: Vec::new(),
            other: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.removals.is_empty()
            && self.modifications.is_empty()
            && self.additions.is_empty()
            && self.other.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.removals.len() + self.modifications.len() + self.additions.len() + self.other.len()
    }

    pub fn get_ordered_changes(&self) -> Vec<ConfigChange> {
        let mut changes = Vec::new();

        // Apply in order: removals, modifications, additions, other
        changes.extend_from_slice(&self.removals);
        changes.extend_from_slice(&self.modifications);
        changes.extend_from_slice(&self.additions);
        changes.extend_from_slice(&self.other);

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_types::{Framerate, Resolution, VideoFormat, VideoSourceType};

    #[test]
    fn test_no_changes() {
        let differ = ConfigDiffer::new();
        let config = AppConfig::default();
        let changes = differ.diff(&config, &config);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_source_added() {
        let differ = ConfigDiffer::new();
        let mut old_config = AppConfig::default();
        old_config.sources.clear();

        let mut new_config = old_config.clone();
        new_config
            .sources
            .push(VideoSourceConfig::test_pattern("new-source", "smpte"));

        let changes = differ.diff(&old_config, &new_config);
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], ConfigChange::SourceAdded { .. }));
    }

    #[test]
    fn test_source_removed() {
        let differ = ConfigDiffer::new();
        let mut old_config = AppConfig::default();
        old_config.sources.clear();
        old_config
            .sources
            .push(VideoSourceConfig::test_pattern("test-source", "smpte"));

        let mut new_config = old_config.clone();
        new_config.sources.clear();

        let changes = differ.diff(&old_config, &new_config);
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], ConfigChange::SourceRemoved { .. }));
    }

    #[test]
    fn test_source_modified() {
        let differ = ConfigDiffer::new();
        let mut old_config = AppConfig::default();
        let mut source = VideoSourceConfig::test_pattern("test", "smpte");
        source.resolution = Resolution {
            width: 1920,
            height: 1080,
        };
        old_config.sources = vec![source];

        let mut new_config = old_config.clone();
        new_config.sources[0].resolution = Resolution {
            width: 1280,
            height: 720,
        };

        let changes = differ.diff(&old_config, &new_config);
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], ConfigChange::SourceModified { .. }));
    }

    #[test]
    fn test_change_plan() {
        let differ = ConfigDiffer::new();
        let changes = vec![
            ConfigChange::SourceAdded {
                config: VideoSourceConfig::test_pattern("add", "smpte"),
            },
            ConfigChange::SourceRemoved {
                name: "remove".to_string(),
            },
            ConfigChange::SourceModified {
                name: "modify".to_string(),
                old_config: VideoSourceConfig::test_pattern("old", "smpte"),
                new_config: VideoSourceConfig::test_pattern("new", "ball"),
            },
        ];

        let plan = differ.generate_change_plan(&changes);
        assert_eq!(plan.removals.len(), 1);
        assert_eq!(plan.additions.len(), 1);
        assert_eq!(plan.modifications.len(), 1);
        assert_eq!(plan.total_changes(), 3);

        let ordered = plan.get_ordered_changes();
        assert!(matches!(ordered[0], ConfigChange::SourceRemoved { .. }));
        assert!(matches!(ordered[1], ConfigChange::SourceModified { .. }));
        assert!(matches!(ordered[2], ConfigChange::SourceAdded { .. }));
    }
}
