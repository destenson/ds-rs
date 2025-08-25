use crate::config_types::{DirectoryConfig, FilterConfig, VideoSourceConfig, VideoSourceType, FileContainer};
use crate::error::{Result, SourceVideoError};
use crate::file_utils::{is_video_file, path_to_mount_point, detect_container_format};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::{WalkDir, DirEntry};

pub struct DirectoryScanner {
    config: DirectoryConfig,
    discovered_files: Vec<PathBuf>,
}

impl DirectoryScanner {
    pub fn new(config: DirectoryConfig) -> Self {
        Self {
            config,
            discovered_files: Vec::new(),
        }
    }
    
    pub fn scan(&mut self) -> Result<Vec<VideoSourceConfig>> {
        let path = Path::new(&self.config.path);
        
        if !path.exists() {
            return Err(SourceVideoError::config(format!(
                "Directory does not exist: {}",
                self.config.path
            )));
        }
        
        if !path.is_dir() {
            return Err(SourceVideoError::config(format!(
                "Path is not a directory: {}",
                self.config.path
            )));
        }
        
        self.discovered_files.clear();
        
        let walker = if self.config.recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };
        
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if self.should_include_entry(&entry) {
                self.discovered_files.push(entry.path().to_path_buf());
            }
        }
        
        log::info!(
            "Discovered {} video files in directory: {}",
            self.discovered_files.len(),
            self.config.path
        );
        
        let configs = self.create_source_configs()?;
        Ok(configs)
    }
    
    pub fn scan_async(&mut self) -> Result<Vec<VideoSourceConfig>> {
        // For now, just use synchronous scanning
        // Future: Implement background scanning with progress updates
        self.scan()
    }
    
    fn should_include_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();
        
        // Skip directories
        if path.is_dir() {
            return false;
        }
        
        // Check if it's a video file
        if !is_video_file(path) {
            return false;
        }
        
        // Apply filters if configured
        if let Some(filters) = &self.config.filters {
            if !self.passes_filters(path, filters) {
                return false;
            }
        }
        
        true
    }
    
    fn passes_filters(&self, path: &Path, filters: &FilterConfig) -> bool {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Check include patterns
        if !filters.include.is_empty() {
            let mut matches_include = false;
            for pattern in &filters.include {
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
        for pattern in &filters.exclude {
            if self.matches_pattern(file_name, pattern) {
                return false;
            }
        }
        
        // Check extensions
        if !filters.extensions.is_empty() {
            let extension = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            
            if !filters.extensions.iter().any(|ext| ext.eq_ignore_ascii_case(extension)) {
                return false;
            }
        }
        
        true
    }
    
    fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        // Simple glob pattern matching
        // Future: Use proper glob library for more complex patterns
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
    
    fn create_source_configs(&self) -> Result<Vec<VideoSourceConfig>> {
        let mut configs = Vec::new();
        
        for (index, file_path) in self.discovered_files.iter().enumerate() {
            let mount_point = path_to_mount_point(
                file_path,
                &self.config.path,
                self.config.mount_prefix.as_deref()
            )?;
            
            let container = detect_container_format(file_path)
                .unwrap_or(FileContainer::Mp4);
            
            let source_name = format!(
                "{}_{}",
                file_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("file"),
                index
            );
            
            let config = VideoSourceConfig {
                name: source_name,
                source_type: VideoSourceType::File {
                    path: file_path.to_string_lossy().to_string(),
                    container,
                },
                resolution: crate::config_types::Resolution {
                    width: 1920,
                    height: 1080,
                },
                framerate: crate::config_types::Framerate {
                    numerator: 30,
                    denominator: 1,
                },
                format: crate::config_types::VideoFormat::I420,
                duration: None,
                num_buffers: None,
                is_live: false,
            };
            
            configs.push(config);
        }
        
        Ok(configs)
    }
    
    pub fn get_discovered_files(&self) -> &[PathBuf] {
        &self.discovered_files
    }
}

pub struct BatchSourceLoader {
    directories: Vec<DirectoryConfig>,
    file_lists: Vec<Vec<String>>,
}

impl BatchSourceLoader {
    pub fn new() -> Self {
        Self {
            directories: Vec::new(),
            file_lists: Vec::new(),
        }
    }
    
    pub fn add_directory(&mut self, config: DirectoryConfig) {
        self.directories.push(config);
    }
    
    pub fn add_file_list(&mut self, files: Vec<String>) {
        self.file_lists.push(files);
    }
    
    pub fn load_all(&mut self) -> Result<Vec<VideoSourceConfig>> {
        let mut all_configs = Vec::new();
        
        // Process directories
        for dir_config in &self.directories {
            let mut scanner = DirectoryScanner::new(dir_config.clone());
            let configs = scanner.scan()?;
            all_configs.extend(configs);
        }
        
        // Process file lists
        for (list_index, file_list) in self.file_lists.iter().enumerate() {
            for (file_index, file_path) in file_list.iter().enumerate() {
                let path = Path::new(file_path);
                
                if !path.exists() {
                    log::warn!("File does not exist: {}", file_path);
                    continue;
                }
                
                if !is_video_file(path) {
                    log::warn!("Not a video file: {}", file_path);
                    continue;
                }
                
                let container = detect_container_format(path)
                    .unwrap_or(FileContainer::Mp4);
                
                let source_name = format!(
                    "list{}_file{}_{}",
                    list_index,
                    file_index,
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("video")
                );
                
                let config = VideoSourceConfig {
                    name: source_name,
                    source_type: VideoSourceType::File {
                        path: file_path.clone(),
                        container,
                    },
                    resolution: crate::config_types::Resolution {
                        width: 1920,
                        height: 1080,
                    },
                    framerate: crate::config_types::Framerate {
                        numerator: 30,
                        denominator: 1,
                    },
                    format: crate::config_types::VideoFormat::I420,
                    duration: None,
                    num_buffers: None,
                    is_live: false,
                };
                
                all_configs.push(config);
            }
        }
        
        Ok(all_configs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_directory_scanner_creation() {
        let config = DirectoryConfig {
            path: "/tmp/videos".to_string(),
            recursive: false,
            filters: None,
            lazy_loading: true,
            mount_prefix: None,
        };
        
        let scanner = DirectoryScanner::new(config);
        assert!(scanner.get_discovered_files().is_empty());
    }
    
    #[test]
    fn test_pattern_matching() {
        let config = DirectoryConfig {
            path: ".".to_string(),
            recursive: false,
            filters: None,
            lazy_loading: true,
            mount_prefix: None,
        };
        
        let scanner = DirectoryScanner::new(config);
        
        assert!(scanner.matches_pattern("test.mp4", "*.mp4"));
        assert!(scanner.matches_pattern("video.mp4", "*video*"));
        assert!(scanner.matches_pattern("test_video.mp4", "test_*"));
        assert!(!scanner.matches_pattern("test.avi", "*.mp4"));
        assert!(!scanner.matches_pattern("video.mp4", "audio*"));
    }
    
    #[test]
    fn test_batch_source_loader() {
        let mut loader = BatchSourceLoader::new();
        
        loader.add_file_list(vec![
            "/tmp/video1.mp4".to_string(),
            "/tmp/video2.avi".to_string(),
        ]);
        
        assert_eq!(loader.file_lists.len(), 1);
        assert_eq!(loader.directories.len(), 0);
    }
}