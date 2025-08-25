use source_videos::{
    RtspServerBuilder, VideoSourceConfig, VideoSourceType,
    config_types::{FileContainer, Resolution, Framerate, VideoFormat}
};
use tempfile::TempDir;
use std::fs;

fn setup() {
    let _ = gstreamer::init();
}

#[test]
fn test_rtsp_server_with_file_source() {
    setup();
    
    // Create a temporary test video file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test_video.mp4");
    
    // For this test, we'll just create an empty file
    // In a real scenario, you'd have an actual video file
    fs::write(&test_file, b"fake video content").unwrap();
    
    // Create RTSP server with file source
    let config = VideoSourceConfig {
        name: "test_file".to_string(),
        source_type: VideoSourceType::File {
            path: test_file.display().to_string(),
            container: FileContainer::Mp4,
        },
        resolution: Resolution {
            width: 1920,
            height: 1080,
        },
        framerate: Framerate {
            numerator: 30,
            denominator: 1,
        },
        format: VideoFormat::I420,
        duration: None,
        num_buffers: None,
        is_live: false,
    };
    
    let server = RtspServerBuilder::new()
        .port(8555) // Use different port to avoid conflicts
        .add_source(config)
        .build();
    
    assert!(server.is_ok());
    let server = server.unwrap();
    
    // Verify the source was added
    let sources = server.list_sources();
    assert_eq!(sources.len(), 1);
    assert!(sources[0].contains("test_file"));
    
    // Verify URL generation
    let url = server.get_url(&sources[0]);
    assert!(url.contains("rtsp://"));
    assert!(url.contains("8555"));
    assert!(url.contains("test_file"));
}

#[test]
fn test_rtsp_server_without_video_source_manager() {
    setup();
    
    // This test verifies that RTSP server can work independently
    // without creating any VideoSource instances
    
    let server = RtspServerBuilder::new()
        .port(8556)
        .add_test_pattern("pattern1", "smpte")
        .add_test_pattern("pattern2", "ball")
        .build();
    
    assert!(server.is_ok());
    let server = server.unwrap();
    
    // Start the server
    assert!(server.start().is_ok());
    
    // Verify sources are available
    let sources = server.list_sources();
    assert_eq!(sources.len(), 2);
    
    // Verify URLs
    for source in &sources {
        let url = server.get_url(source);
        assert!(url.starts_with("rtsp://"));
        assert!(url.contains("8556"));
    }
}

#[test]
fn test_rtsp_server_file_watching_integration() {
    setup();
    
    let temp_dir = TempDir::new().unwrap();
    let mut server = RtspServerBuilder::new()
        .port(8557)
        .build()
        .unwrap();
    
    // Test adding a file through file system event
    let test_file = temp_dir.path().join("new_video.mp4");
    fs::write(&test_file, b"video content").unwrap();
    
    use source_videos::FileSystemEvent;
    use source_videos::FileEventMetadata;
    
    let event = FileSystemEvent::Created(FileEventMetadata {
        path: test_file.clone(),
        size: Some(13),
        modified: Some(std::time::SystemTime::now()),
        watcher_id: "test".to_string(),
    });
    
    // Handle the file creation event
    assert!(server.handle_file_event(&event).is_ok());
    
    // Verify the source was added
    let sources = server.list_sources();
    assert!(sources.iter().any(|s| s.contains("new_video")));
    
    // Test file deletion
    let event = FileSystemEvent::Deleted(FileEventMetadata {
        path: test_file.clone(),
        size: Some(13),
        modified: Some(std::time::SystemTime::now()),
        watcher_id: "test".to_string(),
    });
    
    assert!(server.handle_file_event(&event).is_ok());
    
    // Verify the source was removed
    let sources = server.list_sources();
    assert!(!sources.iter().any(|s| s.contains("new_video")));
}

#[test]
fn test_rtsp_server_multiple_file_sources() {
    setup();
    
    let temp_dir = TempDir::new().unwrap();
    let mut configs = Vec::new();
    
    // Create multiple test files
    for i in 0..5 {
        let file_path = temp_dir.path().join(format!("video_{}.mp4", i));
        fs::write(&file_path, format!("video {} content", i)).unwrap();
        
        let config = VideoSourceConfig {
            name: format!("video_{}", i),
            source_type: VideoSourceType::File {
                path: file_path.display().to_string(),
                container: FileContainer::Mp4,
            },
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            framerate: Framerate {
                numerator: 30,
                denominator: 1,
            },
            format: VideoFormat::I420,
            duration: None,
            num_buffers: None,
            is_live: false,
        };
        
        configs.push(config);
    }
    
    // Build server with multiple sources
    let mut builder = RtspServerBuilder::new().port(8558);
    for config in configs {
        builder = builder.add_source(config);
    }
    
    let server = builder.build().unwrap();
    
    // Verify all sources were added
    let sources = server.list_sources();
    assert_eq!(sources.len(), 5);
    
    for i in 0..5 {
        assert!(sources.iter().any(|s| s.contains(&format!("video_{}", i))));
    }
}

#[test]
fn test_rtsp_server_windows_path_handling() {
    setup();
    
    // Test that Windows paths are correctly handled
    let windows_path = r"C:\Users\test\videos\test_video.mp4";
    
    let config = VideoSourceConfig {
        name: "windows_test".to_string(),
        source_type: VideoSourceType::File {
            path: windows_path.to_string(),
            container: FileContainer::Mp4,
        },
        resolution: Resolution {
            width: 1920,
            height: 1080,
        },
        framerate: Framerate {
            numerator: 30,
            denominator: 1,
        },
        format: VideoFormat::I420,
        duration: None,
        num_buffers: None,
        is_live: false,
    };
    
    let server = RtspServerBuilder::new()
        .port(8559)
        .add_source(config)
        .build();
    
    // The server should be created successfully even with Windows paths
    assert!(server.is_ok());
}

#[test]
fn test_no_port_conflict_with_rtsp_only() {
    setup();
    
    // This test verifies that using RTSP server alone doesn't create
    // any local playback pipelines that could cause port conflicts
    
    // Create first server
    let server1 = RtspServerBuilder::new()
        .port(8560)
        .add_test_pattern("test1", "smpte")
        .build()
        .unwrap();
    
    server1.start().unwrap();
    
    // Create second server on different port - should work
    let server2 = RtspServerBuilder::new()
        .port(8561)
        .add_test_pattern("test2", "ball")
        .build()
        .unwrap();
    
    server2.start().unwrap();
    
    // Both servers should have their sources
    assert_eq!(server1.list_sources().len(), 1);
    assert_eq!(server2.list_sources().len(), 1);
}