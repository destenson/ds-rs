use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::broadcast;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct FileEventMetadata {
    pub path: PathBuf,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
    pub watcher_id: String,
}

#[derive(Debug, Clone)]
pub enum FileSystemEvent {
    Created(FileEventMetadata),
    Modified(FileEventMetadata),
    Deleted(FileEventMetadata),
    Accessed(FileEventMetadata),
    Renamed {
        from: FileEventMetadata,
        to: FileEventMetadata,
    },
    Error {
        path: PathBuf,
        error: String,
        watcher_id: String,
    },
}

impl FileSystemEvent {
    pub fn path(&self) -> &PathBuf {
        match self {
            FileSystemEvent::Created(meta)
            | FileSystemEvent::Modified(meta)
            | FileSystemEvent::Deleted(meta)
            | FileSystemEvent::Accessed(meta) => &meta.path,
            FileSystemEvent::Renamed { to, .. } => &to.path,
            FileSystemEvent::Error { path, .. } => path,
        }
    }
    
    pub fn watcher_id(&self) -> &str {
        match self {
            FileSystemEvent::Created(meta)
            | FileSystemEvent::Modified(meta)
            | FileSystemEvent::Deleted(meta)
            | FileSystemEvent::Accessed(meta) => &meta.watcher_id,
            FileSystemEvent::Renamed { to, .. } => &to.watcher_id,
            FileSystemEvent::Error { watcher_id, .. } => watcher_id,
        }
    }
    
    pub fn is_actionable(&self) -> bool {
        matches!(
            self,
            FileSystemEvent::Created(_) | FileSystemEvent::Modified(_) | FileSystemEvent::Deleted(_)
        )
    }
    
    pub fn event_type(&self) -> &'static str {
        match self {
            FileSystemEvent::Created(_) => "created",
            FileSystemEvent::Modified(_) => "modified",
            FileSystemEvent::Deleted(_) => "deleted",
            FileSystemEvent::Accessed(_) => "accessed",
            FileSystemEvent::Renamed { .. } => "renamed",
            FileSystemEvent::Error { .. } => "error",
        }
    }
}

pub trait FileEventHandler: Send + Sync {
    fn handle_created(&self, metadata: &FileEventMetadata) -> impl Future<Output=Result<(), String>> + Send;
    fn handle_modified(&self, metadata: &FileEventMetadata) -> impl Future<Output=Result<(), String>> + Send;
    fn handle_deleted(&self, metadata: &FileEventMetadata) -> impl Future<Output=Result<(), String>> + Send;
    fn handle_error(&self, path: &PathBuf, error: &str) -> impl Future<Output=Result<(), String>> + Send;
}

pub struct EventRouter {
    handlers: Vec<EventHandlerType>,
}

pub enum EventHandlerType {
    // We'll add concrete handler types here as needed
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    
    pub fn add_handler(&mut self, handler: EventHandlerType) {
        self.handlers.push(handler);
    }
    
    pub async fn route_event(&self, event: &FileSystemEvent) {
        log::debug!("Routing event: {:?}", event.event_type());
        // For now, just log the event type
        // Concrete handlers can be added as needed
    }
}

pub struct EventFilter {
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    include_extensions: Vec<String>,
    exclude_extensions: Vec<String>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            include_extensions: Vec::new(),
            exclude_extensions: Vec::new(),
        }
    }
    
    pub fn with_include_pattern(mut self, pattern: String) -> Self {
        self.include_patterns.push(pattern);
        self
    }
    
    pub fn with_exclude_pattern(mut self, pattern: String) -> Self {
        self.exclude_patterns.push(pattern);
        self
    }
    
    pub fn with_include_extension(mut self, ext: String) -> Self {
        self.include_extensions.push(ext.to_lowercase());
        self
    }
    
    pub fn with_exclude_extension(mut self, ext: String) -> Self {
        self.exclude_extensions.push(ext.to_lowercase());
        self
    }
    
    pub fn should_process(&self, path: &PathBuf) -> bool {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        
        // Check include patterns
        if !self.include_patterns.is_empty() {
            let mut matches_include = false;
            for pattern in &self.include_patterns {
                if self.matches_pattern(file_name, pattern) {
                    matches_include = true;
                    break;
                }
            }
            if !matches_include {
                return false;
            }
        }
        
        // Check exclude patterns
        for pattern in &self.exclude_patterns {
            if self.matches_pattern(file_name, pattern) {
                return false;
            }
        }
        
        // Check include extensions
        if !self.include_extensions.is_empty() {
            if !self.include_extensions.contains(&extension) {
                return false;
            }
        }
        
        // Check exclude extensions
        if self.exclude_extensions.contains(&extension) {
            return false;
        }
        
        true
    }
    
    fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.is_empty() {
                return true;
            }
            
            let mut pos = 0;
            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                
                if i == 0 && !pattern.starts_with('*') {
                    if !file_name.starts_with(part) {
                        return false;
                    }
                    pos = part.len();
                } else if i == parts.len() - 1 && !pattern.ends_with('*') {
                    if !file_name.ends_with(part) {
                        return false;
                    }
                } else if let Some(index) = file_name[pos..].find(part) {
                    pos += index + part.len();
                } else {
                    return false;
                }
            }
            true
        } else {
            file_name == pattern
        }
    }
}

pub struct EventAggregator {
    pending_events: HashMap<PathBuf, FileSystemEvent>,
    window_duration: Duration,
    last_flush: Instant,
    tx: broadcast::Sender<Vec<FileSystemEvent>>,
}

impl EventAggregator {
    pub fn new(window_duration: Duration) -> Self {
        let (tx, _) = broadcast::channel(1000);
        
        Self {
            pending_events: HashMap::new(),
            window_duration,
            last_flush: Instant::now(),
            tx,
        }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<FileSystemEvent>> {
        self.tx.subscribe()
    }
    
    pub async fn add_event(&mut self, event: FileSystemEvent) {
        let path = event.path().clone();
        
        // Replace any previous event for this path
        self.pending_events.insert(path, event);
        
        // Check if we should flush based on time window
        if self.last_flush.elapsed() >= self.window_duration {
            self.flush().await;
        }
    }
    
    pub async fn flush(&mut self) {
        if self.pending_events.is_empty() {
            return;
        }
        
        let events: Vec<FileSystemEvent> = self.pending_events.drain().map(|(_, event)| event).collect();
        
        log::debug!("Flushing {} aggregated events", events.len());
        
        if let Err(e) = self.tx.send(events) {
            log::warn!("No subscribers for aggregated events: {}", e);
        }
        
        self.last_flush = Instant::now();
    }
    
    pub async fn start_periodic_flush(&mut self) {
        let mut interval = tokio::time::interval(self.window_duration);
        
        loop {
            interval.tick().await;
            self.flush().await;
        }
    }
    
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }
}

pub struct EventBatch {
    events: Vec<FileSystemEvent>,
    created_at: Instant,
}

impl EventBatch {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            created_at: Instant::now(),
        }
    }
    
    pub fn add_event(&mut self, event: FileSystemEvent) {
        self.events.push(event);
    }
    
    pub fn events(&self) -> &[FileSystemEvent] {
        &self.events
    }
    
    pub fn len(&self) -> usize {
        self.events.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
    
    pub fn created_events(&self) -> impl Iterator<Item = &FileSystemEvent> {
        self.events.iter().filter(|e| matches!(e, FileSystemEvent::Created(_)))
    }
    
    pub fn modified_events(&self) -> impl Iterator<Item = &FileSystemEvent> {
        self.events.iter().filter(|e| matches!(e, FileSystemEvent::Modified(_)))
    }
    
    pub fn deleted_events(&self) -> impl Iterator<Item = &FileSystemEvent> {
        self.events.iter().filter(|e| matches!(e, FileSystemEvent::Deleted(_)))
    }
}

#[derive(Clone)]
pub struct EventStats {
    pub created_count: u64,
    pub modified_count: u64,
    pub deleted_count: u64,
    pub error_count: u64,
    pub start_time: Instant,
}

impl EventStats {
    pub fn new() -> Self {
        Self {
            created_count: 0,
            modified_count: 0,
            deleted_count: 0,
            error_count: 0,
            start_time: Instant::now(),
        }
    }
    
    pub fn record_event(&mut self, event: &FileSystemEvent) {
        match event {
            FileSystemEvent::Created(_) => self.created_count += 1,
            FileSystemEvent::Modified(_) => self.modified_count += 1,
            FileSystemEvent::Deleted(_) => self.deleted_count += 1,
            FileSystemEvent::Error { .. } => self.error_count += 1,
            _ => {}
        }
    }
    
    pub fn total_events(&self) -> u64 {
        self.created_count + self.modified_count + self.deleted_count + self.error_count
    }
    
    pub fn events_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_events() as f64 / elapsed
        } else {
            0.0
        }
    }
    
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for EventStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_file_system_event_methods() {
        let metadata = FileEventMetadata {
            path: PathBuf::from("/test/video.mp4"),
            size: Some(1024),
            modified: None,
            watcher_id: "test-id".to_string(),
        };
        
        let event = FileSystemEvent::Created(metadata.clone());
        
        assert_eq!(event.path(), &PathBuf::from("/test/video.mp4"));
        assert_eq!(event.watcher_id(), "test-id");
        assert_eq!(event.event_type(), "created");
        assert!(event.is_actionable());
    }
    
    #[test]
    fn test_event_filter_patterns() {
        let filter = EventFilter::new()
            .with_include_pattern("*.mp4".to_string())
            .with_exclude_pattern("temp_*".to_string());
        
        assert!(filter.should_process(&PathBuf::from("video.mp4")));
        assert!(!filter.should_process(&PathBuf::from("video.avi")));
        assert!(!filter.should_process(&PathBuf::from("temp_video.mp4")));
    }
    
    #[test]
    fn test_event_filter_extensions() {
        let filter = EventFilter::new()
            .with_include_extension("mp4".to_string())
            .with_include_extension("avi".to_string())
            .with_exclude_extension("tmp".to_string());
        
        assert!(filter.should_process(&PathBuf::from("video.mp4")));
        assert!(filter.should_process(&PathBuf::from("video.avi")));
        assert!(!filter.should_process(&PathBuf::from("video.mkv")));
        assert!(!filter.should_process(&PathBuf::from("video.tmp")));
    }
    
    #[test]
    fn test_event_batch() {
        let mut batch = EventBatch::new();
        
        let metadata = FileEventMetadata {
            path: PathBuf::from("/test/video.mp4"),
            size: Some(1024),
            modified: None,
            watcher_id: "test-id".to_string(),
        };
        
        batch.add_event(FileSystemEvent::Created(metadata.clone()));
        batch.add_event(FileSystemEvent::Modified(metadata));
        
        assert_eq!(batch.len(), 2);
        assert_eq!(batch.created_events().count(), 1);
        assert_eq!(batch.modified_events().count(), 1);
        assert_eq!(batch.deleted_events().count(), 0);
    }
    
    #[test]
    fn test_event_stats() {
        let mut stats = EventStats::new();
        
        let metadata = FileEventMetadata {
            path: PathBuf::from("/test/video.mp4"),
            size: Some(1024),
            modified: None,
            watcher_id: "test-id".to_string(),
        };
        
        stats.record_event(&FileSystemEvent::Created(metadata.clone()));
        stats.record_event(&FileSystemEvent::Modified(metadata.clone()));
        stats.record_event(&FileSystemEvent::Deleted(metadata));
        
        assert_eq!(stats.created_count, 1);
        assert_eq!(stats.modified_count, 1);
        assert_eq!(stats.deleted_count, 1);
        assert_eq!(stats.total_events(), 3);
    }
}
