use crate::error::{Result, SourceVideoError};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio::time::sleep;

pub struct ConfigWatcher {
    path: PathBuf,
    tx: mpsc::Sender<ConfigEvent>,
    rx: Option<mpsc::Receiver<ConfigEvent>>,
    _watcher: Option<RecommendedWatcher>,
    debounce_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    Modified(PathBuf),
    Created(PathBuf),
    Deleted(PathBuf),
    Error(String),
}

impl ConfigWatcher {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let (tx, rx) = mpsc::channel(100);
        
        Ok(Self {
            path,
            tx,
            rx: Some(rx),
            _watcher: None,
            debounce_duration: Duration::from_millis(500),
        })
    }
    
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }
    
    pub async fn start(&mut self) -> Result<()> {
        let path = self.path.clone();
        let tx = self.tx.clone();
        let debounce = self.debounce_duration;
        
        // Create async watcher
        let (notify_tx, mut notify_rx) = mpsc::channel(100);
        
        // Create the file system watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = notify_tx.blocking_send(event);
                }
            },
            Config::default(),
        ).map_err(|e| SourceVideoError::config(format!("Failed to create watcher: {}", e)))?;
        
        // Watch the config file
        watcher.watch(&path, RecursiveMode::NonRecursive)
            .map_err(|e| SourceVideoError::config(format!("Failed to watch path: {}", e)))?;
        
        self._watcher = Some(watcher);
        
        // Spawn async task to handle events with debouncing
        tokio::spawn(async move {
            let mut last_event_time = tokio::time::Instant::now();
            
            while let Some(event) = notify_rx.recv().await {
                let now = tokio::time::Instant::now();
                
                // Debounce: wait if event comes too quickly
                if now.duration_since(last_event_time) < debounce {
                    sleep(debounce).await;
                }
                
                last_event_time = tokio::time::Instant::now();
                
                let config_event = match event.kind {
                    EventKind::Modify(_) => ConfigEvent::Modified(path.clone()),
                    EventKind::Create(_) => ConfigEvent::Created(path.clone()),
                    EventKind::Remove(_) => ConfigEvent::Deleted(path.clone()),
                    _ => continue,
                };
                
                if let Err(e) = tx.send(config_event).await {
                    log::error!("Failed to send config event: {}", e);
                    break;
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn recv(&mut self) -> Option<ConfigEvent> {
        if let Some(ref mut rx) = self.rx {
            rx.recv().await
        } else {
            None
        }
    }
    
    pub fn stop(&mut self) {
        self._watcher = None;
        self.rx = None;
    }
}

pub struct ConfigBroadcaster {
    tx: watch::Sender<Option<ConfigEvent>>,
    rx: watch::Receiver<Option<ConfigEvent>>,
}

impl ConfigBroadcaster {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(None);
        Self { tx, rx }
    }
    
    pub fn send(&self, event: ConfigEvent) -> Result<()> {
        self.tx.send(Some(event))
            .map_err(|_| SourceVideoError::config("Failed to broadcast config event"))
    }
    
    pub fn subscribe(&self) -> watch::Receiver<Option<ConfigEvent>> {
        self.rx.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::time::timeout;
    
    #[tokio::test]
    async fn test_config_watcher_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let watcher = ConfigWatcher::new(temp_file.path());
        assert!(watcher.is_ok());
    }
    
    #[tokio::test]
    async fn test_config_event_broadcast() {
        let broadcaster = ConfigBroadcaster::new();
        let mut subscriber = broadcaster.subscribe();
        
        broadcaster.send(ConfigEvent::Modified(PathBuf::from("/test"))).unwrap();
        
        tokio::spawn(async move {
            if let Ok(Ok(())) = timeout(Duration::from_secs(1), subscriber.changed()).await {
                if let Some(ConfigEvent::Modified(path)) = subscriber.borrow().as_ref() {
                    assert_eq!(path, &PathBuf::from("/test"));
                }
            }
        });
    }
}