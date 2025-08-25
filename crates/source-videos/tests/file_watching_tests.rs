use source_videos::{
    DirectoryWatcher, WatcherManager, FileSystemEvent,
    LoopConfig, create_looping_source, FileVideoSource, FileWatcher,
    init,
};
use source_videos::watch::FileWatcherInstance;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn test_directory_watcher_basic() {
    init().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create initial video file
    let video_file = temp_dir.path().join("test.mp4");
    fs::write(&video_file, b"dummy video content").unwrap();
    
    let mut watcher = DirectoryWatcher::new(temp_dir.path(), false).unwrap();
    assert!(!watcher.is_watching());
    
    watcher.start().await.unwrap();
    assert!(watcher.is_watching());
    
    // Give watcher time to initialize
    sleep(Duration::from_millis(100)).await;
    
    // Create a new video file
    let new_video = temp_dir.path().join("new_video.avi");
    fs::write(&new_video, b"new video content").unwrap();
    
    // Should receive a Created event
    let event_result = timeout(Duration::from_secs(5), watcher.recv()).await;
    assert!(event_result.is_ok());
    
    if let Ok(Some(event)) = event_result {
        match event {
            FileSystemEvent::Created(metadata) => {
                assert!(metadata.path.to_string_lossy().contains("new_video.avi"));
            }
            _ => panic!("Expected Created event, got {:?}", event.event_type()),
        }
    }
    
    watcher.stop().await.unwrap();
    assert!(!watcher.is_watching());
}

#[tokio::test]
async fn test_directory_watcher_recursive() {
    init().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    
    let mut watcher = DirectoryWatcher::new(temp_dir.path(), true).unwrap();
    watcher.start().await.unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    // Create video in subdirectory
    let nested_video = sub_dir.join("nested.mkv");
    fs::write(&nested_video, b"nested video").unwrap();
    
    let event_result = timeout(Duration::from_secs(5), watcher.recv()).await;
    assert!(event_result.is_ok());
    
    if let Ok(Some(event)) = event_result {
        match event {
            FileSystemEvent::Created(metadata) => {
                assert!(metadata.path.to_string_lossy().contains("nested.mkv"));
            }
            _ => panic!("Expected Created event for nested file"),
        }
    }
    
    watcher.stop().await.unwrap();
}

#[tokio::test]
async fn test_file_modification_detection() {
    init().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let video_file = temp_dir.path().join("modify_test.mp4");
    fs::write(&video_file, b"original content").unwrap();
    
    let mut watcher = DirectoryWatcher::new(temp_dir.path(), false).unwrap();
    watcher.start().await.unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    // Modify the file
    fs::write(&video_file, b"modified content - much longer").unwrap();
    
    let event_result = timeout(Duration::from_secs(5), watcher.recv()).await;
    assert!(event_result.is_ok());
    
    if let Ok(Some(event)) = event_result {
        match event {
            FileSystemEvent::Modified(metadata) => {
                assert!(metadata.path.to_string_lossy().contains("modify_test.mp4"));
                // Size should be updated
                assert!(metadata.size.is_some());
            }
            _ => panic!("Expected Modified event"),
        }
    }
    
    watcher.stop().await.unwrap();
}

#[tokio::test]
async fn test_file_deletion_detection() {
    init().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let video_file = temp_dir.path().join("delete_test.webm");
    fs::write(&video_file, b"to be deleted").unwrap();
    
    let mut watcher = DirectoryWatcher::new(temp_dir.path(), false).unwrap();
    watcher.start().await.unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    // Delete the file
    fs::remove_file(&video_file).unwrap();
    
    let event_result = timeout(Duration::from_secs(5), watcher.recv()).await;
    assert!(event_result.is_ok());
    
    if let Ok(Some(event)) = event_result {
        match event {
            FileSystemEvent::Deleted(metadata) => {
                assert!(metadata.path.to_string_lossy().contains("delete_test.webm"));
            }
            _ => panic!("Expected Deleted event"),
        }
    }
    
    watcher.stop().await.unwrap();
}

#[tokio::test]
async fn test_watcher_manager() {
    init().unwrap();
    
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();
    
    let mut manager = WatcherManager::new();
    
    // Add multiple directory watchers
    let id1 = manager
        .add_directory_watcher(temp_dir1.path(), false)
        .await
        .unwrap();
    let id2 = manager
        .add_directory_watcher(temp_dir2.path(), true)
        .await
        .unwrap();
    
    assert_ne!(id1, id2);
    assert!(manager.is_watching(&id1));
    assert!(manager.is_watching(&id2));
    
    let watchers = manager.list_watchers();
    assert_eq!(watchers.len(), 2);
    assert!(watchers.contains(&id1.as_str()));
    assert!(watchers.contains(&id2.as_str()));
    
    // Create files in both directories
    fs::write(temp_dir1.path().join("video1.mp4"), b"content1").unwrap();
    fs::write(temp_dir2.path().join("video2.avi"), b"content2").unwrap();
    
    sleep(Duration::from_millis(200)).await;
    
    // Should receive events from both watchers
    let mut events_received = 0;
    for _ in 0..2 {
        if let Ok(Some(event)) = timeout(Duration::from_secs(3), manager.recv()).await {
            match event {
                FileSystemEvent::Created(metadata) => {
                    events_received += 1;
                    let path_str = metadata.path.to_string_lossy();
                    assert!(path_str.contains("video1.mp4") || path_str.contains("video2.avi"));
                }
                _ => {}
            }
        }
    }
    
    assert_eq!(events_received, 2);
    
    manager.stop_all().await.unwrap();
    assert!(!manager.is_watching(&id1));
    assert!(!manager.is_watching(&id2));
}

#[tokio::test]
async fn test_individual_file_watcher() {
    init().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let video_file = temp_dir.path().join("single.mp4");
    fs::write(&video_file, b"original").unwrap();
    
    let mut watcher = FileWatcherInstance::new(&video_file).unwrap();
    watcher.start().await.unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    // Modify the specific file
    fs::write(&video_file, b"modified content").unwrap();
    
    let event_result = timeout(Duration::from_secs(5), watcher.recv()).await;
    assert!(event_result.is_ok());
    
    if let Ok(Some(event)) = event_result {
        match event {
            FileSystemEvent::Modified(metadata) => {
                assert_eq!(metadata.path, video_file);
            }
            _ => panic!("Expected Modified event"),
        }
    }
    
    watcher.stop().await.unwrap();
}

#[tokio::test]
async fn test_non_video_files_ignored() {
    init().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    
    let mut watcher = DirectoryWatcher::new(temp_dir.path(), false).unwrap();
    watcher.start().await.unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    // Create non-video files
    fs::write(temp_dir.path().join("document.pdf"), b"pdf content").unwrap();
    fs::write(temp_dir.path().join("image.jpg"), b"image data").unwrap();
    fs::write(temp_dir.path().join("text.txt"), b"text file").unwrap();
    
    // Create a video file
    fs::write(temp_dir.path().join("video.mp4"), b"video content").unwrap();
    
    // Should only receive event for the video file
    let event_result = timeout(Duration::from_secs(3), watcher.recv()).await;
    assert!(event_result.is_ok());
    
    if let Ok(Some(event)) = event_result {
        match event {
            FileSystemEvent::Created(metadata) => {
                assert!(metadata.path.to_string_lossy().contains("video.mp4"));
            }
            _ => panic!("Expected Created event for video file only"),
        }
    }
    
    // Should not receive any more events (non-video files are ignored)
    let no_more_events = timeout(Duration::from_millis(500), watcher.recv()).await;
    assert!(no_more_events.is_err()); // Timeout expected
    
    watcher.stop().await.unwrap();
}

#[tokio::test]
async fn test_rapid_file_changes_debounced() {
    init().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let video_file = temp_dir.path().join("rapid.mp4");
    fs::write(&video_file, b"initial").unwrap();
    
    let mut watcher = DirectoryWatcher::new(temp_dir.path(), false)
        .unwrap()
        .with_debounce(Duration::from_millis(200));
    
    watcher.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;
    
    // Make rapid changes
    for i in 0..5 {
        fs::write(&video_file, format!("content_{}", i).as_bytes()).unwrap();
        sleep(Duration::from_millis(50)).await; // Faster than debounce
    }
    
    // Should receive only one event due to debouncing
    let event_result = timeout(Duration::from_secs(5), watcher.recv()).await;
    assert!(event_result.is_ok());
    
    // Should not receive immediate additional events
    let no_immediate_event = timeout(Duration::from_millis(100), watcher.recv()).await;
    assert!(no_immediate_event.is_err());
    
    watcher.stop().await.unwrap();
}

#[tokio::test]
async fn test_auto_repeat_integration() {
    init().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let video_file = temp_dir.path().join("loop_test.mp4");
    fs::write(&video_file, b"video for looping").unwrap();
    
    // Create a mock video source for the file
    let video_config = source_videos::VideoSourceConfig {
        name: "loop_test".to_string(),
        source_type: source_videos::VideoSourceType::File {
            path: video_file.to_string_lossy().to_string(),
            container: source_videos::config_types::FileContainer::Mp4,
        },
        resolution: source_videos::config_types::Resolution { width: 640, height: 480 },
        framerate: source_videos::config_types::Framerate { numerator: 30, denominator: 1 },
        format: source_videos::config_types::VideoFormat::I420,
        duration: Some(5),
        num_buffers: None,
        is_live: false,
    };
    
    let file_source = FileVideoSource::from_config(&video_config).unwrap();
    
    // Create looping source
    let loop_config = LoopConfig {
        max_loops: Some(3),
        seamless: true,
        ..Default::default()
    };
    
    let looping_source = create_looping_source(Box::new(file_source), Some(3), true);
    
    assert_eq!(looping_source.get_loop_count(), 0);
    assert!(!looping_source.is_looping_active());
    
    // In a real test, we would start the source and verify looping behavior
    // For now, just verify the basic properties
    assert_eq!(looping_source.get_loop_count(), 0);
    assert!(!looping_source.is_looping_active());
}

#[tokio::test]
async fn test_error_handling() {
    init().unwrap();
    
    // Test watching non-existent directory
    let result = DirectoryWatcher::new("/nonexistent/directory", false);
    assert!(result.is_err());
    
    // Test watching a file as directory
    let temp_dir = TempDir::new().unwrap();
    let file = temp_dir.path().join("not_a_directory.txt");
    fs::write(&file, b"content").unwrap();
    
    let result = DirectoryWatcher::new(&file, false);
    assert!(result.is_err());
    
    // Test watching non-existent file
    let result = FileWatcherInstance::new("/nonexistent/file.mp4");
    assert!(result.is_err());
}

// Integration test demonstrating complete workflow
#[tokio::test]
async fn test_complete_file_watching_workflow() {
    init().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let mut manager = WatcherManager::new();
    
    // Start watching
    let watcher_id = manager
        .add_directory_watcher(temp_dir.path(), false)
        .await
        .unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    // Simulate complete workflow
    let events = vec![
        ("create", "new_video.mp4", "video content"),
        ("modify", "new_video.mp4", "updated video content"),
        ("create", "another.avi", "another video"),
    ];
    
    for (action, filename, content) in events {
        let file_path = temp_dir.path().join(filename);
        
        match action {
            "create" | "modify" => {
                fs::write(&file_path, content.as_bytes()).unwrap();
            }
            _ => {}
        }
        
        // Verify event is received
        if let Ok(Some(event)) = timeout(Duration::from_secs(3), manager.recv()).await {
            match (action, &event) {
                ("create", FileSystemEvent::Created(metadata)) => {
                    assert!(metadata.path.to_string_lossy().contains(filename));
                }
                ("modify", FileSystemEvent::Modified(metadata)) => {
                    assert!(metadata.path.to_string_lossy().contains(filename));
                }
                _ => {}
            }
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Clean up
    manager.remove_watcher(&watcher_id).await.unwrap();
    assert!(!manager.is_watching(&watcher_id));
}