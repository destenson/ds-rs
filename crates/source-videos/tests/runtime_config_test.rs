use source_videos::{
    AppConfig, VideoSourceConfig, VideoSourceManager, RuntimeManager,
    ConfigurationEvent, Result,
};
use std::sync::Arc;
use tempfile::NamedTempFile;
use std::io::Write;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_runtime_manager_basic() {
    gstreamer::init().unwrap();
    
    let manager = Arc::new(VideoSourceManager::new());
    let config = AppConfig::default();
    let runtime = RuntimeManager::new(manager, config);
    
    let current = runtime.get_current_config().await;
    assert!(!current.sources.is_empty());
}

#[tokio::test]
async fn test_config_file_monitoring() {
    gstreamer::init().unwrap();
    
    use source_videos::config::{ConfigWatcher, ConfigEvent};
    
    // Create temp config file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, r#"
        log_level = "info"
        
        [[sources]]
        name = "test-source"
        type = "test_pattern"
        pattern = "smpte"
    "#).unwrap();
    
    let mut watcher = ConfigWatcher::new(temp_file.path()).unwrap();
    watcher.start().await.unwrap();
    
    // Modify the file
    writeln!(temp_file, r#"
        log_level = "debug"
        
        [[sources]]
        name = "test-source"
        type = "test_pattern"
        pattern = "ball"
    "#).unwrap();
    temp_file.flush().unwrap();
    
    // Wait for event
    let event = timeout(Duration::from_secs(2), watcher.recv()).await;
    assert!(event.is_ok());
    
    if let Ok(Some(ConfigEvent::Modified(_))) = event {
        // Success
    } else {
        panic!("Expected Modified event");
    }
}

#[tokio::test]
async fn test_config_validation() {
    use source_videos::config::{
        loader::{ConfigLoader, TomlConfigLoader},
        validator::DefaultConfigValidator,
    };
    use std::sync::Arc;
    
    let validator = Arc::new(DefaultConfigValidator::new());
    let loader = TomlConfigLoader::new(validator);
    
    // Create config with invalid resolution
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, r#"
        log_level = "info"
        
        [[sources]]
        name = "test"
        type = "test_pattern"
        pattern = "smpte"
        
        [sources.resolution]
        width = 10
        height = 10
    "#).unwrap();
    temp_file.flush().unwrap();
    
    let result = loader.load(temp_file.path());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_differ() {
    use source_videos::runtime::differ::{ConfigDiffer, ConfigChange};
    
    let differ = ConfigDiffer::new();
    
    let mut old_config = AppConfig::default();
    old_config.sources.clear();
    old_config.sources.push(VideoSourceConfig::test_pattern("test1", "smpte"));
    
    let mut new_config = old_config.clone();
    new_config.sources.push(VideoSourceConfig::test_pattern("test2", "ball"));
    new_config.server.port = 9000;
    
    let changes = differ.diff(&old_config, &new_config);
    
    assert_eq!(changes.len(), 2); // One source added, one port change
    
    let has_source_added = changes.iter().any(|c| matches!(c, ConfigChange::SourceAdded { .. }));
    let has_port_changed = changes.iter().any(|c| matches!(c, ConfigChange::ServerPortChanged { .. }));
    
    assert!(has_source_added);
    assert!(has_port_changed);
}

#[tokio::test]
async fn test_runtime_config_updates() {
    gstreamer::init().unwrap();
    
    let manager = Arc::new(VideoSourceManager::new());
    let initial_config = AppConfig::default();
    let runtime = RuntimeManager::new(manager.clone(), initial_config);
    
    // Subscribe to events
    let mut event_rx = runtime.subscribe_events();
    
    // Add a new source
    let new_source = VideoSourceConfig::test_pattern("new-test", "ball");
    runtime.add_source(new_source).await.unwrap();
    
    // Check event was emitted
    let event = timeout(Duration::from_secs(1), event_rx.recv()).await;
    assert!(event.is_ok());
    
    if let Ok(Ok(ConfigurationEvent::SourceAdded { source })) = event {
        assert_eq!(source, "new-test");
    } else {
        panic!("Expected SourceAdded event");
    }
    
    // Verify source was added
    assert_eq!(manager.source_count(), 3); // 2 default + 1 new
}

#[tokio::test]
async fn test_config_rollback() {
    gstreamer::init().unwrap();
    
    let manager = Arc::new(VideoSourceManager::new());
    let initial_config = AppConfig::default();
    let runtime = RuntimeManager::new(manager.clone(), initial_config.clone());
    
    // Apply a new config
    let mut new_config = initial_config.clone();
    new_config.sources.push(VideoSourceConfig::test_pattern("rollback-test", "snow"));
    
    runtime.apply_config(new_config).await.unwrap();
    assert_eq!(manager.source_count(), 3); // 2 default + 1 new
    
    // Rollback
    runtime.rollback().await.unwrap();
    assert_eq!(manager.source_count(), 2); // Back to 2 default
}

#[tokio::test]
async fn test_atomic_config_loader() {
    use source_videos::config::{
        loader::{AtomicConfigLoader, TomlConfigLoader},
        validator::DefaultConfigValidator,
    };
    use std::sync::Arc;
    
    let validator = Arc::new(DefaultConfigValidator::new());
    let loader = Arc::new(TomlConfigLoader::new(validator));
    let atomic_loader = AtomicConfigLoader::new(loader, AppConfig::default());
    
    // Get initial config
    let initial = atomic_loader.get_current().await;
    assert_eq!(initial.sources.len(), 2); // Default has 2 sources
    
    // Update config
    let updated = atomic_loader.update_if_valid(|config| {
        let mut new_config = config.clone();
        new_config.sources.push(VideoSourceConfig::test_pattern("atomic-test", "gradient"));
        Ok(new_config)
    }).await.unwrap();
    
    assert_eq!(updated.sources.len(), 3);
    
    // Verify update persisted
    let current = atomic_loader.get_current().await;
    assert_eq!(current.sources.len(), 3);
}

#[tokio::test]
async fn test_signal_handler() {
    use source_videos::runtime::signal_handler::{SignalHandler, SignalEvent};
    
    let handler = SignalHandler::new();
    let mut rx = handler.start().await.unwrap();
    
    // Trigger reload manually
    handler.trigger_reload();
    
    let event = timeout(Duration::from_secs(1), rx.recv()).await
        .expect("Timeout")
        .expect("No event");
    
    assert!(matches!(event, SignalEvent::Reload));
}

#[tokio::test]
async fn test_performance_monitoring() {
    use source_videos::runtime::applicator::PerformanceMonitor;
    use source_videos::runtime::differ::ConfigChange;
    
    let mut monitor = PerformanceMonitor::new();
    
    monitor.record(
        ConfigChange::SourceAdded {
            config: VideoSourceConfig::test_pattern("perf1", "smpte"),
        },
        Duration::from_millis(100),
    );
    
    monitor.record(
        ConfigChange::SourceRemoved {
            name: "perf1".to_string(),
        },
        Duration::from_millis(50),
    );
    
    assert_eq!(monitor.total_time(), Duration::from_millis(150));
    assert_eq!(monitor.average_time(), Duration::from_millis(75));
    
    let slowest = monitor.slowest_change();
    assert!(slowest.is_some());
    assert_eq!(slowest.unwrap().1, Duration::from_millis(100));
}