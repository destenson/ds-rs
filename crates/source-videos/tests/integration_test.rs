use source_videos::{
    VideoSourceManager, VideoSourceConfig, TestPattern,
    RtspServerBuilder, generate_test_file, SourceVideos
};
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_video_source_manager() {
    source_videos::init().unwrap();
    
    let manager = VideoSourceManager::new();
    assert_eq!(manager.source_count(), 0);
    
    let config = VideoSourceConfig::test_pattern("test1", "smpte");
    let id = manager.add_source(config).unwrap();
    assert_eq!(manager.source_count(), 1);
    
    let info = manager.get_source("test1").unwrap();
    assert_eq!(info.name, "test1");
    assert_eq!(info.id, id);
    
    manager.pause_source("test1").unwrap();
    let info = manager.get_source("test1").unwrap();
    assert_eq!(info.state, source_videos::SourceState::Paused);
    
    manager.remove_source("test1").unwrap();
    assert_eq!(manager.source_count(), 0);
}

#[tokio::test]
async fn test_multiple_sources() {
    source_videos::init().unwrap();
    
    let manager = VideoSourceManager::new();
    
    let patterns = vec!["smpte", "ball", "snow"];
    for (i, pattern) in patterns.iter().enumerate() {
        let name = format!("test{}", i + 1);
        let config = VideoSourceConfig::test_pattern(&name, *pattern);
        manager.add_source(config).unwrap();
    }
    
    assert_eq!(manager.source_count(), 3);
    
    let sources = manager.list_sources();
    assert_eq!(sources.len(), 3);
    
    for info in &sources {
        assert!(info.is_playing());
    }
    
    manager.clear_all().unwrap();
    assert_eq!(manager.source_count(), 0);
}

#[test]
fn test_rtsp_server_builder() {
    source_videos::init().unwrap();
    
    let server = RtspServerBuilder::new()
        .port(8555)
        .add_test_pattern("test1", "smpte")
        .add_test_pattern("test2", "ball")
        .build();
    
    assert!(server.is_ok());
    let server = server.unwrap();
    assert_eq!(server.get_port(), 8555);
    assert_eq!(server.list_sources().len(), 2);
}

#[test]
fn test_file_generation() {
    source_videos::init().unwrap();
    
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("test.mp4");
    
    let result = generate_test_file("smpte", 1, &output_path)
        .inspect_err(|e| eprintln!("Error generating file: {}", e));
    assert!(result.is_ok());
    
    assert!(output_path.exists());
    let metadata = std::fs::metadata(&output_path).unwrap();
    assert!(metadata.len() > 0);
}

#[tokio::test]
async fn test_source_videos_integration() {
    source_videos::init().unwrap();
    
    let mut sv = SourceVideos::new().unwrap();
    
    sv.add_test_pattern("test1", "smpte").unwrap();
    sv.add_test_pattern("test2", "ball").unwrap();
    
    let sources = sv.list_sources();
    assert_eq!(sources.len(), 2);
    
    // sv.start_rtsp_server(8556).unwrap();
    
    let urls = sv.get_rtsp_urls();
    assert!(urls.is_empty());
    
    for url in &urls {
        assert!(url.starts_with("rtsp://"));
        assert!(url.contains(":8556"));
    }
}

#[test]
fn test_pattern_validation() {
    for pattern in TestPattern::all() {
        let pattern_str = format!("{:?}", pattern);
        let parsed = TestPattern::from_str(&pattern_str);
        assert!(parsed.is_ok(), "Failed to parse pattern: {}", pattern_str);
        
        let parsed_pattern = parsed.unwrap();
        assert_eq!(pattern, parsed_pattern);
    }
}

#[test]
fn test_config_serialization() {
    let config = source_videos::AppConfig::default();
    
    let toml_str = toml::to_string(&config).unwrap();
    let parsed: source_videos::AppConfig = toml::from_str(&toml_str).unwrap();
    
    assert_eq!(config.server.port, parsed.server.port);
    assert_eq!(config.sources.len(), parsed.sources.len());
}

#[test]
fn test_concurrent_source_operations() {
    use std::sync::Arc;
    use std::thread;
    
    source_videos::init().unwrap();
    
    let manager = Arc::new(VideoSourceManager::new());
    let mut handles = vec![];
    
    for i in 0..5 {
        let manager = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let name = format!("concurrent-{}", i);
            let config = VideoSourceConfig::test_pattern(&name, "smpte");
            manager.add_source(config).unwrap();
            
            std::thread::sleep(Duration::from_millis(100));
            
            manager.remove_source(&name).unwrap();
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(manager.source_count(), 0);
}
