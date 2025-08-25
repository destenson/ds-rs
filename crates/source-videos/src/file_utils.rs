use crate::config_types::FileContainer;
use crate::error::{Result, SourceVideoError};
use mime_guess::from_path;
use std::path::{Path, PathBuf};

/// Common video file extensions
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg",
    "3gp", "ogv", "ts", "mts", "m2ts", "vob", "rmvb", "rm", "asf", "divx",
    "f4v", "f4p", "f4a", "f4b",
];

/// Check if a file is a video file based on extension and MIME type
pub fn is_video_file(path: &Path) -> bool {
    // Check extension first (faster)
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            if VIDEO_EXTENSIONS.iter().any(|&e| e.eq_ignore_ascii_case(ext_str)) {
                return true;
            }
        }
    }
    
    // Fallback to MIME type detection
    let mime = from_path(path);
    if let Some(mime_type) = mime.first() {
        return mime_type.type_() == "video";
    }
    
    false
}

/// Detect the container format from a file path
pub fn detect_container_format(path: &Path) -> Option<FileContainer> {
    let extension = path.extension()?.to_str()?.to_lowercase();
    
    match extension.as_str() {
        "mp4" | "m4v" | "f4v" | "f4p" => Some(FileContainer::Mp4),
        "mkv" | "mka" => Some(FileContainer::Mkv),
        "avi" | "divx" => Some(FileContainer::Avi),
        "webm" => Some(FileContainer::WebM),
        _ => None,
    }
}

/// Convert a file path to an RTSP mount point
pub fn path_to_mount_point(
    file_path: &Path,
    base_dir: &str,
    mount_prefix: Option<&str>,
) -> Result<String> {
    let base_path = Path::new(base_dir);
    
    // Get relative path from base directory
    let relative_path = if file_path.starts_with(base_path) {
        file_path.strip_prefix(base_path)
            .map_err(|e| SourceVideoError::config(format!("Failed to strip prefix: {}", e)))?
    } else {
        file_path
    };
    
    // Convert path to URL-safe string
    let mut mount_point = String::new();
    
    // Add prefix if provided
    if let Some(prefix) = mount_prefix {
        mount_point.push_str(prefix);
        if !mount_point.ends_with('/') {
            mount_point.push('/');
        }
    }
    
    // Convert path components to URL format
    for component in relative_path.components() {
        if let std::path::Component::Normal(os_str) = component {
            if let Some(s) = os_str.to_str() {
                if !mount_point.is_empty() && !mount_point.ends_with('/') {
                    mount_point.push('/');
                }
                mount_point.push_str(&url_encode(s));
            }
        }
    }
    
    // Remove file extension for cleaner URLs
    if let Some(pos) = mount_point.rfind('.') {
        mount_point.truncate(pos);
    }
    
    Ok(mount_point)
}

/// Simple URL encoding for mount points
fn url_encode(s: &str) -> String {
    let mut result = String::new();
    
    for ch in s.chars() {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(ch);
            }
            ' ' => {
                result.push('_');
            }
            _ => {
                for byte in ch.to_string().bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    
    result
}

/// Extract video metadata (placeholder for future implementation)
pub struct VideoMetadata {
    pub duration: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub framerate: Option<f32>,
    pub codec: Option<String>,
    pub bitrate: Option<u32>,
}

impl VideoMetadata {
    pub fn from_file(_path: &Path) -> Result<Self> {
        // TODO: Implement actual metadata extraction using GStreamer discoverer
        Ok(Self {
            duration: None,
            width: None,
            height: None,
            framerate: None,
            codec: None,
            bitrate: None,
        })
    }
}

/// Find all video files in a directory (non-recursive)
pub fn find_video_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if !dir.is_dir() {
        return Err(SourceVideoError::config(format!(
            "Not a directory: {}",
            dir.display()
        )));
    }
    
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && is_video_file(&path) {
            files.push(path);
        }
    }
    
    Ok(files)
}

/// Normalize a file path for consistent handling
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(os_str) => {
                normalized.push(os_str);
            }
            std::path::Component::RootDir => {
                normalized.push("/");
            }
            std::path::Component::Prefix(prefix) => {
                normalized.push(prefix.as_os_str());
            }
            std::path::Component::CurDir => {}
        }
    }
    
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_is_video_file() {
        let video_files = vec![
            Path::new("test.mp4"),
            Path::new("video.avi"),
            Path::new("movie.mkv"),
            Path::new("clip.webm"),
        ];
        
        for path in video_files {
            assert!(is_video_file(path), "Failed for: {:?}", path);
        }
        
        let non_video_files = vec![
            Path::new("document.pdf"),
            Path::new("image.jpg"),
            Path::new("audio.mp3"),
            Path::new("script.sh"),
        ];
        
        for path in non_video_files {
            assert!(!is_video_file(path), "Failed for: {:?}", path);
        }
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
        assert_eq!(
            detect_container_format(Path::new("unknown.xyz")),
            None
        );
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
    fn test_url_encoding() {
        assert_eq!(url_encode("hello world"), "hello_world");
        assert_eq!(url_encode("test-file_123.mp4"), "test-file_123.mp4");
        assert_eq!(url_encode("file@#$"), "file%40%23%24");
    }
    
    #[test]
    fn test_normalize_path() {
        let path = Path::new("/videos/../movies/./action/movie.mp4");
        let normalized = normalize_path(path);
        
        #[cfg(unix)]
        assert_eq!(normalized, PathBuf::from("/movies/action/movie.mp4"));
        
        #[cfg(windows)]
        {
            let expected = normalized.to_string_lossy();
            assert!(expected.ends_with("movies\\action\\movie.mp4"));
        }
    }
}