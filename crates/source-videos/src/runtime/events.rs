use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationEvent {
    ConfigLoaded {
        path: String,
    },
    ConfigApplied {
        changes: usize,
    },
    ConfigFailed {
        error: String,
    },
    ConfigRolledBack,
    SourceAdded {
        source: String,
    },
    SourceRemoved {
        source: String,
    },
    SourceUpdated {
        source: String,
    },
    SourceError {
        source: String,
        error: String,
    },
    ValidationError {
        error: String,
    },
    FileSystemChange {
        event_type: String,
        path: PathBuf,
        source_id: Option<String>,
        watcher_id: String,
    },
}

pub struct EventBus {
    sender: broadcast::Sender<ConfigurationEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
    
    pub async fn emit(&self, event: ConfigurationEvent) {
        log::debug!("Emitting event: {:?}", event);
        
        if let Err(e) = self.sender.send(event.clone()) {
            log::warn!("No subscribers for event: {:?} ({})", event, e);
        }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigurationEvent> {
        self.sender.subscribe()
    }
}

pub struct EventFilter {
    receiver: broadcast::Receiver<ConfigurationEvent>,
}

impl EventFilter {
    pub fn new(receiver: broadcast::Receiver<ConfigurationEvent>) -> Self {
        Self { receiver }
    }
    
    pub async fn next_matching<F>(&mut self, filter: F) -> Option<ConfigurationEvent>
    where
        F: Fn(&ConfigurationEvent) -> bool,
    {
        while let Ok(event) = self.receiver.recv().await {
            if filter(&event) {
                return Some(event);
            }
        }
        None
    }
    
    pub async fn collect_until<F>(&mut self, stop_condition: F, max_events: usize) -> Vec<ConfigurationEvent>
    where
        F: Fn(&ConfigurationEvent) -> bool,
    {
        let mut events = Vec::with_capacity(max_events);
        
        while events.len() < max_events {
            match self.receiver.recv().await {
                Ok(event) => {
                    let should_stop = stop_condition(&event);
                    events.push(event);
                    if should_stop {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        
        events
    }
}

pub struct EventLogger {
    receiver: broadcast::Receiver<ConfigurationEvent>,
    persist: bool,
    log_path: Option<String>,
}

impl EventLogger {
    pub fn new(receiver: broadcast::Receiver<ConfigurationEvent>) -> Self {
        Self {
            receiver,
            persist: false,
            log_path: None,
        }
    }
    
    pub fn with_persistence(mut self, path: String) -> Self {
        self.persist = true;
        self.log_path = Some(path);
        self
    }
    
    pub async fn start(mut self) {
        tokio::spawn(async move {
            while let Ok(event) = self.receiver.recv().await {
                log::info!("Configuration event: {:?}", event);
                
                if self.persist {
                    if let Some(ref path) = self.log_path {
                        if let Ok(json) = serde_json::to_string(&event) {
                            let _ = async {
                                use tokio::io::AsyncWriteExt;
                                let mut file = tokio::fs::OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open(path)
                                    .await
                                    .ok()?;
                                file.write_all(json.as_bytes()).await.ok()?;
                                file.write_all(b"\n").await.ok()
                            }.await;
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe();
        
        bus.emit(ConfigurationEvent::SourceAdded {
            source: "test".to_string(),
        }).await;
        
        let event = timeout(Duration::from_secs(1), receiver.recv()).await;
        assert!(event.is_ok());
        
        if let Ok(Ok(ConfigurationEvent::SourceAdded { source })) = event {
            assert_eq!(source, "test");
        } else {
            panic!("Unexpected event type");
        }
    }
    
    #[tokio::test]
    async fn test_event_filter() {
        let bus = EventBus::new();
        let receiver = bus.subscribe();
        let mut filter = EventFilter::new(receiver);
        
        // Emit various events
        bus.emit(ConfigurationEvent::SourceAdded {
            source: "source1".to_string(),
        }).await;
        
        bus.emit(ConfigurationEvent::SourceRemoved {
            source: "source2".to_string(),
        }).await;
        
        bus.emit(ConfigurationEvent::SourceAdded {
            source: "source3".to_string(),
        }).await;
        
        // Filter for SourceAdded events
        let task = tokio::spawn(async move {
            let event = filter.next_matching(|e| matches!(e, ConfigurationEvent::SourceAdded { .. })).await;
            event
        });
        
        let result = timeout(Duration::from_secs(1), task).await;
        assert!(result.is_ok());
    }
}