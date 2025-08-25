pub mod events;

use crate::error::{Result, SourceVideoError};
use crate::file_utils::is_video_file;
pub(crate) use events::{FileSystemEvent, FileEventMetadata};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use uuid::Uuid;

pub trait FileWatcher {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    async fn recv(&mut self) -> Option<FileSystemEvent>;
    fn is_watching(&self) -> bool;
}

pub struct DirectoryWatcher {
    id: String,
    path: PathBuf,
    recursive: bool,
    tx: mpsc::Sender<FileSystemEvent>,
    rx: Option<mpsc::Receiver<FileSystemEvent>>,
    watcher: Option<RecommendedWatcher>,
    debounce_duration: Duration,
    last_events: HashMap<PathBuf, SystemTime>,
}

impl DirectoryWatcher {
    pub fn new<P: AsRef<Path>>(path: P, recursive: bool) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let (tx, rx) = mpsc::channel(1000);
        
        if !path.exists() {
            return Err(SourceVideoError::config(format!(
                "Watch path does not exist: {}",
                path.display()
            )));
        }
        
        if !path.is_dir() {
            return Err(SourceVideoError::config(format!(
                "Watch path is not a directory: {}",
                path.display()
            )));
        }
        
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            path,
            recursive,
            tx,
            rx: Some(rx),
            watcher: None,
            debounce_duration: Duration::from_millis(500),
            last_events: HashMap::new(),
        })
    }
    
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }
    
    pub fn get_id(&self) -> &str {
        &self.id
    }
    
    pub fn get_path(&self) -> &Path {
        &self.path
    }
    
    pub fn is_recursive(&self) -> bool {
        self.recursive
    }
    
    fn should_process_event(&mut self, path: &Path, event_kind: &EventKind) -> bool {
        let now = SystemTime::now();
        
        // Check debouncing
        if let Some(last_time) = self.last_events.get(path) {
            if let Ok(duration) = now.duration_since(*last_time) {
                if duration < self.debounce_duration {
                    return false;
                }
            }
        }
        
        // Update last event time
        self.last_events.insert(path.to_path_buf(), now);
        
        // Only process video files for create/modify/delete
        match event_kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                is_video_file(path)
            }
            _ => false,
        }
    }
    
    fn create_file_event(&self, path: PathBuf, kind: EventKind) -> Option<FileSystemEvent> {
        let metadata = FileEventMetadata {
            path: path.clone(),
            size: if path.exists() {
                std::fs::metadata(&path).ok().map(|m| m.len())
            } else {
                None
            },
            modified: if path.exists() {
                std::fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .ok()
            } else {
                None
            },
            watcher_id: self.id.clone(),
        };
        
        match kind {
            EventKind::Create(_) => Some(FileSystemEvent::Created(metadata)),
            EventKind::Modify(_) => Some(FileSystemEvent::Modified(metadata)),
            EventKind::Remove(_) => Some(FileSystemEvent::Deleted(metadata)),
            EventKind::Access(_) => Some(FileSystemEvent::Accessed(metadata)),
            _ => None,
        }
    }
}

impl FileWatcher for DirectoryWatcher {
    async fn start(&mut self) -> Result<()> {
        if self.watcher.is_some() {
            return Ok(());
        }
        
        let path = self.path.clone();
        let tx = self.tx.clone();
        let recursive = self.recursive;
        let watcher_id = self.id.clone();
        
        // Create async watcher with channel
        let (notify_tx, mut notify_rx) = mpsc::channel(1000);
        
        let mut watcher = RecommendedWatcher::new(
            move |res: std::result::Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if let Err(e) = notify_tx.blocking_send(event) {
                            log::error!("Failed to send notify event: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("File watcher error: {}", e);
                    }
                }
            },
            Config::default(),
        ).map_err(|e| SourceVideoError::config(format!("Failed to create directory watcher: {}", e)))?;
        
        let recursive_mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        
        watcher.watch(&path, recursive_mode)
            .map_err(|e| SourceVideoError::config(format!("Failed to watch directory: {}", e)))?;
        
        self.watcher = Some(watcher);
        
        // Spawn async task to handle events
        let tx_clone = tx.clone();
        let path_clone = path.clone();
        tokio::spawn(async move {
            let mut last_events: HashMap<PathBuf, SystemTime> = HashMap::new();
            let debounce = Duration::from_millis(500);
            
            while let Some(event) = notify_rx.recv().await {
                for event_path in event.paths {
                    // Skip if not a video file
                    if !is_video_file(&event_path) {
                        continue;
                    }
                    
                    // Debouncing check
                    let now = SystemTime::now();
                    if let Some(last_time) = last_events.get(&event_path) {
                        if let Ok(duration) = now.duration_since(*last_time) {
                            if duration < debounce {
                                continue;
                            }
                        }
                    }
                    last_events.insert(event_path.clone(), now);
                    
                    // Create file event
                    let metadata = FileEventMetadata {
                        path: event_path.clone(),
                        size: if event_path.exists() {
                            std::fs::metadata(&event_path).ok().map(|m| m.len())
                        } else {
                            None
                        },
                        modified: if event_path.exists() {
                            std::fs::metadata(&event_path)
                                .and_then(|m| m.modified())
                                .ok()
                        } else {
                            None
                        },
                        watcher_id: watcher_id.clone(),
                    };
                    
                    let fs_event = match event.kind {
                        EventKind::Create(_) => {
                            log::info!("Video file created: {}", event_path.display());
                            FileSystemEvent::Created(metadata)
                        }
                        EventKind::Modify(_) => {
                            log::info!("Video file modified: {}", event_path.display());
                            FileSystemEvent::Modified(metadata)
                        }
                        EventKind::Remove(_) => {
                            log::info!("Video file deleted: {}", event_path.display());
                            FileSystemEvent::Deleted(metadata)
                        }
                        EventKind::Access(_) => {
                            FileSystemEvent::Accessed(metadata)
                        }
                        _ => continue,
                    };
                    
                    if let Err(e) = tx_clone.send(fs_event).await {
                        log::error!("Failed to send file system event: {}", e);
                        break;
                    }
                }
            }
            
            log::info!("Directory watcher task ended for: {}", path_clone.display());
        });
        
        log::info!(
            "Started watching directory: {} (recursive: {})",
            self.path.display(),
            self.recursive
        );
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.watcher = None;
        self.rx = None;
        self.last_events.clear();
        
        log::info!("Stopped watching directory: {}", self.path.display());
        Ok(())
    }
    
    async fn recv(&mut self) -> Option<FileSystemEvent> {
        if let Some(ref mut rx) = self.rx {
            rx.recv().await
        } else {
            None
        }
    }
    
    fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }
}

pub struct FileWatcherInstance {
    path: PathBuf,
    tx: mpsc::Sender<FileSystemEvent>,
    rx: Option<mpsc::Receiver<FileSystemEvent>>,
    watcher: Option<RecommendedWatcher>,
    id: String,
}

impl FileWatcherInstance {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let (tx, rx) = mpsc::channel(100);
        
        if !path.exists() {
            return Err(SourceVideoError::config(format!(
                "File does not exist: {}",
                path.display()
            )));
        }
        
        if !path.is_file() {
            return Err(SourceVideoError::config(format!(
                "Path is not a file: {}",
                path.display()
            )));
        }
        
        Ok(Self {
            path,
            tx,
            rx: Some(rx),
            watcher: None,
            id: Uuid::new_v4().to_string(),
        })
    }
    
    pub fn get_id(&self) -> &str {
        &self.id
    }
    
    pub fn get_path(&self) -> &Path {
        &self.path
    }
}

impl FileWatcher for FileWatcherInstance {
    async fn start(&mut self) -> Result<()> {
        if self.watcher.is_some() {
            return Ok(());
        }
        
        let path = self.path.clone();
        let tx = self.tx.clone();
        let watcher_id = self.id.clone();
        
        let (notify_tx, mut notify_rx) = mpsc::channel(100);
        
        let mut watcher = RecommendedWatcher::new(
            move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = notify_tx.blocking_send(event);
                }
            },
            Config::default(),
        ).map_err(|e| SourceVideoError::config(format!("Failed to create file watcher: {}", e)))?;
        
        watcher.watch(&path, RecursiveMode::NonRecursive)
            .map_err(|e| SourceVideoError::config(format!("Failed to watch file: {}", e)))?;
        
        self.watcher = Some(watcher);
        
        tokio::spawn(async move {
            while let Some(event) = notify_rx.recv().await {
                for event_path in event.paths {
                    if event_path != path {
                        continue;
                    }
                    
                    let metadata = FileEventMetadata {
                        path: event_path.clone(),
                        size: if event_path.exists() {
                            std::fs::metadata(&event_path).ok().map(|m| m.len())
                        } else {
                            None
                        },
                        modified: if event_path.exists() {
                            std::fs::metadata(&event_path)
                                .and_then(|m| m.modified())
                                .ok()
                        } else {
                            None
                        },
                        watcher_id: watcher_id.clone(),
                    };
                    
                    let fs_event = match event.kind {
                        EventKind::Modify(_) => FileSystemEvent::Modified(metadata),
                        EventKind::Remove(_) => FileSystemEvent::Deleted(metadata),
                        EventKind::Access(_) => FileSystemEvent::Accessed(metadata),
                        _ => continue,
                    };
                    
                    if let Err(e) = tx.send(fs_event).await {
                        log::error!("Failed to send file event: {}", e);
                        break;
                    }
                }
            }
        });
        
        log::info!("Started watching file: {}", self.path.display());
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.watcher = None;
        self.rx = None;
        log::info!("Stopped watching file: {}", self.path.display());
        Ok(())
    }
    
    async fn recv(&mut self) -> Option<FileSystemEvent> {
        if let Some(ref mut rx) = self.rx {
            rx.recv().await
        } else {
            None
        }
    }
    
    fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }
}

pub enum WatcherType {
    Directory(DirectoryWatcher),
    File(FileWatcherInstance),
}

impl WatcherType {
    pub async fn start(&mut self) -> Result<()> {
        match self {
            WatcherType::Directory(w) => w.start().await,
            WatcherType::File(w) => w.start().await,
        }
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        match self {
            WatcherType::Directory(w) => w.stop().await,
            WatcherType::File(w) => w.stop().await,
        }
    }
    
    pub async fn recv(&mut self) -> Option<FileSystemEvent> {
        match self {
            WatcherType::Directory(w) => w.recv().await,
            WatcherType::File(w) => w.recv().await,
        }
    }
    
    pub fn is_watching(&self) -> bool {
        match self {
            WatcherType::Directory(w) => w.is_watching(),
            WatcherType::File(w) => w.is_watching(),
        }
    }
    
    pub fn get_id(&self) -> &str {
        match self {
            WatcherType::Directory(w) => w.get_id(),
            WatcherType::File(w) => w.get_id(),
        }
    }
}

pub struct WatcherManager {
    watchers: HashMap<String, WatcherType>,
    tx: mpsc::Sender<FileSystemEvent>,
    rx: Option<mpsc::Receiver<FileSystemEvent>>,
}

impl WatcherManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);
        
        Self {
            watchers: HashMap::new(),
            tx,
            rx: Some(rx),
        }
    }
    
    pub async fn add_directory_watcher<P: AsRef<Path>>(
        &mut self,
        path: P,
        recursive: bool,
    ) -> Result<String> {
        let mut watcher = DirectoryWatcher::new(path, recursive)?;
        let id = watcher.get_id().to_string();
        
        watcher.start().await?;
        
        self.watchers.insert(id.clone(), WatcherType::Directory(watcher));
        log::info!("Added directory watcher: {}", id);
        
        Ok(id)
    }
    
    pub async fn add_file_watcher<P: AsRef<Path>>(&mut self, path: P) -> Result<String> {
        let mut watcher = FileWatcherInstance::new(path)?;
        let id = watcher.get_id().to_string();
        
        watcher.start().await?;
        
        self.watchers.insert(id.clone(), WatcherType::File(watcher));
        log::info!("Added file watcher: {}", id);
        
        Ok(id)
    }
    
    pub async fn remove_watcher(&mut self, id: &str) -> Result<()> {
        if let Some(mut watcher) = self.watchers.remove(id) {
            watcher.stop().await?;
            log::info!("Removed watcher: {}", id);
        }
        
        Ok(())
    }
    
    pub async fn stop_all(&mut self) -> Result<()> {
        for (id, mut watcher) in self.watchers.drain() {
            if let Err(e) = watcher.stop().await {
                log::error!("Error stopping watcher {}: {}", id, e);
            }
        }
        
        self.rx = None;
        log::info!("Stopped all watchers");
        Ok(())
    }
    
    pub fn list_watchers(&self) -> Vec<&str> {
        self.watchers.keys().map(|s| s.as_str()).collect()
    }
    
    pub fn is_watching(&self, id: &str) -> bool {
        self.watchers
            .get(id)
            .map(|w| w.is_watching())
            .unwrap_or(false)
    }
    
    pub async fn recv(&mut self) -> Option<FileSystemEvent> {
        if let Some(ref mut rx) = self.rx {
            rx.recv().await
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::{sleep, timeout};
    
    #[tokio::test]
    async fn test_directory_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = DirectoryWatcher::new(temp_dir.path(), false);
        assert!(watcher.is_ok());
        
        let watcher = watcher.unwrap();
        assert_eq!(watcher.get_path(), temp_dir.path());
        assert!(!watcher.is_recursive());
    }
    
    #[tokio::test]
    async fn test_file_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.mp4");
        fs::write(&file_path, b"dummy content").unwrap();
        
        let watcher = FileWatcherInstance::new(&file_path);
        assert!(watcher.is_ok());
        
        let watcher = watcher.unwrap();
        assert_eq!(watcher.get_path(), &file_path);
    }
    
    #[tokio::test]
    async fn test_watcher_manager() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WatcherManager::new();
        
        let id = manager.add_directory_watcher(temp_dir.path(), false).await;
        assert!(id.is_ok());
        
        let id = id.unwrap();
        assert!(manager.is_watching(&id));
        
        let watchers = manager.list_watchers();
        assert_eq!(watchers.len(), 1);
        assert!(watchers.contains(&id.as_str()));
    }
}
