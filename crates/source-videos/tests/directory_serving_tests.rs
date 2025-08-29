#![allow(unused)]

use source_videos::{
    BatchSourceLoader, DirectoryConfig, DirectoryScanner, FileListConfig, FilterConfig,
    VideoSourceManager, VideoSourceType, config_types::FileContainer, detect_container_format,
    is_video_file, path_to_mount_point,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn create_test_directory() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create some test video files (just empty files with video extensions)
    let video_files = vec![
        "video1.mp4",
        "video2.avi",
        "movie.mkv",
        "clip.webm",
        "subdir/nested.mp4",
        "subdir/deep/test.avi",
        "ignore.txt",
        "document.pdf",
    ];

    for file in video_files {
        let path = temp_dir.path().join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, b"dummy content").unwrap();
    }

    temp_dir
}

#[test]
fn test_directory_scanner_basic() {
    let temp_dir = create_test_directory();
    let config = DirectoryConfig {
        path: temp_dir.path().display().to_string(),
        recursive: false,
        filters: None,
        lazy_loading: false,
        mount_prefix: None,
    };

    let mut scanner = DirectoryScanner::new(config);
    let configs = scanner.scan().unwrap();

    // Should find 4 video files in root directory (not recursive)
    assert_eq!(configs.len(), 4);

    // Check that all configs are File type
    for config in &configs {
        assert!(matches!(config.source_type, VideoSourceType::File { .. }));
    }
}

#[test]
fn test_directory_scanner_recursive() {
    let temp_dir = create_test_directory();
    let config = DirectoryConfig {
        path: temp_dir.path().display().to_string(),
        recursive: true,
        filters: None,
        lazy_loading: false,
        mount_prefix: None,
    };

    let mut scanner = DirectoryScanner::new(config);
    let configs = scanner.scan().unwrap();

    // Should find 6 video files total (4 in root + 2 in subdirs)
    assert_eq!(configs.len(), 6);
}

#[test]
fn test_directory_scanner_with_filters() {
    let temp_dir = create_test_directory();
    let config = DirectoryConfig {
        path: temp_dir.path().display().to_string(),
        recursive: true,
        filters: Some(FilterConfig {
            include: vec!["*.mp4".to_string()],
            exclude: vec![],
            extensions: vec![],
        }),
        lazy_loading: false,
        mount_prefix: None,
    };

    let mut scanner = DirectoryScanner::new(config);
    let configs = scanner.scan().unwrap();

    // Should find only mp4 files
    assert_eq!(configs.len(), 2); // video1.mp4 and nested.mp4

    for config in &configs {
        if let VideoSourceType::File { path, .. } = &config.source_type {
            assert!(path.ends_with(".mp4"));
        }
    }
}

#[test]
fn test_directory_scanner_with_exclude() {
    let temp_dir = create_test_directory();
    let config = DirectoryConfig {
        path: temp_dir.path().display().to_string(),
        recursive: true,
        filters: Some(FilterConfig {
            include: vec![],
            exclude: vec!["*nested*".to_string()],
            extensions: vec![],
        }),
        lazy_loading: false,
        mount_prefix: None,
    };

    let mut scanner = DirectoryScanner::new(config);
    let configs = scanner.scan().unwrap();

    // Should exclude nested.mp4
    assert_eq!(configs.len(), 5);

    for config in &configs {
        if let VideoSourceType::File { path, .. } = &config.source_type {
            assert!(!path.contains("nested"));
        }
    }
}

#[test]
fn test_directory_scanner_with_mount_prefix() {
    let temp_dir = create_test_directory();
    let config = DirectoryConfig {
        path: temp_dir.path().display().to_string(),
        recursive: false,
        filters: None,
        lazy_loading: false,
        mount_prefix: Some("videos".to_string()),
    };

    let mut scanner = DirectoryScanner::new(config);
    let configs = scanner.scan().unwrap();

    // Mount prefix should be applied (though not directly visible in VideoSourceConfig)
    assert!(!configs.is_empty());
}

#[test]
fn test_batch_source_loader() {
    let temp_dir = create_test_directory();

    let mut loader = BatchSourceLoader::new();

    // Add a directory
    loader.add_directory(DirectoryConfig {
        path: temp_dir.path().display().to_string(),
        recursive: false,
        filters: None,
        lazy_loading: false,
        mount_prefix: None,
    });

    // Add a file list
    loader.add_file_list(vec![
        temp_dir.path().join("video1.mp4").display().to_string(),
        temp_dir.path().join("video2.avi").display().to_string(),
    ]);

    let configs = loader.load_all().unwrap();

    // Should have configs from both directory and file list
    assert!(configs.len() >= 6); // 4 from directory + 2 from file list
}

#[test]
fn test_is_video_file() {
    assert!(is_video_file(Path::new("test.mp4")));
    assert!(is_video_file(Path::new("video.avi")));
    assert!(is_video_file(Path::new("movie.mkv")));
    assert!(is_video_file(Path::new("clip.webm")));
    assert!(is_video_file(Path::new("test.MOV"))); // Case insensitive

    assert!(!is_video_file(Path::new("document.pdf")));
    assert!(!is_video_file(Path::new("image.jpg")));
    assert!(!is_video_file(Path::new("audio.mp3")));
    assert!(!is_video_file(Path::new("script.sh")));
}

#[test]
fn test_detect_container_format() {
    assert_eq!(
        detect_container_format(Path::new("video.mp4")),
        Some(FileContainer::Mp4)
    );
    assert_eq!(
        detect_container_format(Path::new("movie.mkv")),
        Some(FileContainer::Mkv)
    );
    assert_eq!(
        detect_container_format(Path::new("clip.avi")),
        Some(FileContainer::Avi)
    );
    assert_eq!(
        detect_container_format(Path::new("stream.webm")),
        Some(FileContainer::WebM)
    );
    assert_eq!(detect_container_format(Path::new("unknown.xyz")), None);
}

#[test]
fn test_path_to_mount_point() {
    let file_path = Path::new("/videos/movies/action/movie.mp4");
    let base_dir = "/videos";

    let mount = path_to_mount_point(file_path, base_dir, None).unwrap();
    assert_eq!(mount, "movies/action/movie");

    let mount_with_prefix = path_to_mount_point(file_path, base_dir, Some("stream")).unwrap();
    assert_eq!(mount_with_prefix, "stream/movies/action/movie");
}

#[test]
fn test_manager_batch_operations() {
    source_videos::init().unwrap();

    let manager = VideoSourceManager::new();
    let temp_dir = create_test_directory();

    // Test add_directory
    let dir_config = DirectoryConfig {
        path: temp_dir.path().display().to_string(),
        recursive: false,
        filters: None,
        lazy_loading: false,
        mount_prefix: None,
    };

    let added = manager.add_directory(dir_config).unwrap();
    assert_eq!(added.len(), 4); // 4 video files in root
    assert_eq!(manager.source_count(), 4);

    // Test add_file_list
    let file_config = FileListConfig {
        files: vec![
            temp_dir
                .path()
                .join("subdir/nested.mp4")
                .display()
                .to_string(),
        ],
        mount_prefix: None,
        lazy_loading: false,
    };

    let added = manager.add_file_list(file_config).unwrap();
    assert_eq!(added.len(), 1);
    assert_eq!(manager.source_count(), 5); // 4 + 1
}

#[test]
fn test_directory_scanner_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config = DirectoryConfig {
        path: temp_dir.path().display().to_string(),
        recursive: false,
        filters: None,
        lazy_loading: false,
        mount_prefix: None,
    };

    let mut scanner = DirectoryScanner::new(config);
    let configs = scanner.scan().unwrap();

    assert_eq!(configs.len(), 0);
}

#[test]
fn test_directory_scanner_nonexistent_directory() {
    let config = DirectoryConfig {
        path: "/nonexistent/directory/path".to_string(),
        recursive: false,
        filters: None,
        lazy_loading: false,
        mount_prefix: None,
    };

    let mut scanner = DirectoryScanner::new(config);
    let result = scanner.scan();

    assert!(result.is_err());
}

#[test]
fn test_filter_extensions() {
    let temp_dir = create_test_directory();
    let config = DirectoryConfig {
        path: temp_dir.path().display().to_string(),
        recursive: true,
        filters: Some(FilterConfig {
            include: vec![],
            exclude: vec![],
            extensions: vec!["mp4".to_string(), "mkv".to_string()],
        }),
        lazy_loading: false,
        mount_prefix: None,
    };

    let mut scanner = DirectoryScanner::new(config);
    let configs = scanner.scan().unwrap();

    // Should find only mp4 and mkv files
    for config in &configs {
        if let VideoSourceType::File { path, .. } = &config.source_type {
            assert!(path.ends_with(".mp4") || path.ends_with(".mkv"));
        }
    }
}
