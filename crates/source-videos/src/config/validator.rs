#![allow(unused)]
use crate::config::{AppConfig, VideoSourceConfig, VideoSourceType, Resolution, Framerate};
use crate::error::{Result, SourceVideoError};
use std::collections::HashSet;

pub struct DefaultConfigValidator {
    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
    min_framerate: i32,
    max_framerate: i32,
}

impl DefaultConfigValidator {
    pub fn new() -> Self {
        Self {
            min_width: 160,
            max_width: 7680,  // 8K
            min_height: 120,
            max_height: 4320,  // 8K
            min_framerate: 1,
            max_framerate: 120,
        }
    }
    
    pub fn with_constraints(
        min_width: u32,
        max_width: u32,
        min_height: u32,
        max_height: u32,
        min_framerate: i32,
        max_framerate: i32,
    ) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
            min_framerate,
            max_framerate,
        }
    }
    
    fn validate_resolution(&self, resolution: &Resolution) -> Result<()> {
        if resolution.width < self.min_width || resolution.width > self.max_width {
            return Err(SourceVideoError::config(format!(
                "Width {} is out of range [{}, {}]",
                resolution.width, self.min_width, self.max_width
            )));
        }
        
        if resolution.height < self.min_height || resolution.height > self.max_height {
            return Err(SourceVideoError::config(format!(
                "Height {} is out of range [{}, {}]",
                resolution.height, self.min_height, self.max_height
            )));
        }
        
        // Check aspect ratio is reasonable
        let aspect_ratio = resolution.width as f64 / resolution.height as f64;
        if aspect_ratio < 0.25 || aspect_ratio > 4.0 {
            return Err(SourceVideoError::config(format!(
                "Aspect ratio {:.2} is unreasonable (expected 0.25-4.0)",
                aspect_ratio
            )));
        }
        
        Ok(())
    }
    
    fn validate_framerate(&self, framerate: &Framerate) -> Result<()> {
        if framerate.denominator <= 0 {
            return Err(SourceVideoError::config(
                "Framerate denominator must be positive".to_string()
            ));
        }
        
        let fps = framerate.numerator / framerate.denominator;
        if fps < self.min_framerate || fps > self.max_framerate {
            return Err(SourceVideoError::config(format!(
                "Framerate {} fps is out of range [{}, {}]",
                fps, self.min_framerate, self.max_framerate
            )));
        }
        
        Ok(())
    }
    
    fn validate_source_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(SourceVideoError::config("Source name cannot be empty".to_string()));
        }
        
        if name.len() > 64 {
            return Err(SourceVideoError::config(
                "Source name cannot exceed 64 characters".to_string()
            ));
        }
        
        // Check for valid characters
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(SourceVideoError::config(
                "Source name can only contain alphanumeric characters, hyphens, and underscores".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn validate_rtsp_mount_point(&self, mount_point: &str, existing: &HashSet<String>) -> Result<()> {
        if mount_point.is_empty() {
            return Err(SourceVideoError::config("RTSP mount point cannot be empty".to_string()));
        }
        
        if existing.contains(mount_point) {
            return Err(SourceVideoError::config(format!(
                "RTSP mount point '{}' is already in use",
                mount_point
            )));
        }
        
        // Check for valid path characters
        if !mount_point.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/') {
            return Err(SourceVideoError::config(
                "RTSP mount point can only contain alphanumeric characters, hyphens, underscores, and slashes".to_string()
            ));
        }
        
        Ok(())
    }
}

impl super::loader::ConfigValidator for DefaultConfigValidator {
    fn validate(&self, config: &AppConfig) -> Result<()> {
        // Validate server configuration
        if config.server.port == 0 {
            return Err(SourceVideoError::config("Server port cannot be 0".to_string()));
        }
        
        if config.server.max_connections == 0 {
            return Err(SourceVideoError::config(
                "Max connections must be greater than 0".to_string()
            ));
        }
        
        // Check for duplicate source names
        let mut source_names = HashSet::new();
        let mut rtsp_mount_points = HashSet::new();
        
        for source in &config.sources {
            if !source_names.insert(source.name.clone()) {
                return Err(SourceVideoError::config(format!(
                    "Duplicate source name: {}",
                    source.name
                )));
            }
            
            if let VideoSourceType::Rtsp { mount_point, .. } = &source.source_type {
                if !rtsp_mount_points.insert(mount_point.clone()) {
                    return Err(SourceVideoError::config(format!(
                        "Duplicate RTSP mount point: {}",
                        mount_point
                    )));
                }
            }
            
            self.validate_source(source)?;
        }
        
        Ok(())
    }
    
    fn validate_source(&self, source: &VideoSourceConfig) -> Result<()> {
        self.validate_source_name(&source.name)?;
        self.validate_resolution(&source.resolution)?;
        self.validate_framerate(&source.framerate)?;
        
        // Validate source-specific configuration
        match &source.source_type {
            VideoSourceType::TestPattern { pattern } => {
                // Validate pattern is recognized
                let valid_patterns = [
                    "smpte", "smpte75", "smpte100", "snow", "black", "white",
                    "red", "green", "blue", "checkers-1", "checkers-2", "checkers-4",
                    "checkers-8", "circular", "blink", "ball", "gradient", "pinwheel",
                    "spokes", "gamut", "chroma-zone-plate", "solid-color", "bar",
                ];
                
                if !valid_patterns.contains(&pattern.as_str()) {
                    return Err(SourceVideoError::config(format!(
                        "Unknown test pattern: {}",
                        pattern
                    )));
                }
            }
            VideoSourceType::File { path, .. } => {
                if path.is_empty() {
                    return Err(SourceVideoError::config("File path cannot be empty".to_string()));
                }
            }
            VideoSourceType::Rtsp { port, .. } => {
                if *port == 0 {
                    return Err(SourceVideoError::config("RTSP port cannot be 0".to_string()));
                }
            }
            VideoSourceType::Directory { config } => {
                if config.path.is_empty() {
                    return Err(SourceVideoError::config("Directory path cannot be empty".to_string()));
                }
            }
            VideoSourceType::FileList { config } => {
                if config.files.is_empty() {
                    return Err(SourceVideoError::config("File list cannot be empty".to_string()));
                }
            }
        }
        
        // Validate duration if specified
        if let Some(duration) = source.duration {
            if duration == 0 {
                return Err(SourceVideoError::config("Duration cannot be 0".to_string()));
            }
            if duration > 86400 {  // 24 hours
                return Err(SourceVideoError::config(
                    "Duration cannot exceed 24 hours".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

impl Default for DefaultConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resolution_validation() {
        let validator = DefaultConfigValidator::new();
        
        // Valid resolutions
        assert!(validator.validate_resolution(&Resolution { width: 1920, height: 1080 }).is_ok());
        assert!(validator.validate_resolution(&Resolution { width: 640, height: 480 }).is_ok());
        
        // Invalid resolutions
        assert!(validator.validate_resolution(&Resolution { width: 100, height: 100 }).is_err());
        assert!(validator.validate_resolution(&Resolution { width: 10000, height: 10000 }).is_err());
        assert!(validator.validate_resolution(&Resolution { width: 1920, height: 10 }).is_err());
    }
    
    #[test]
    fn test_framerate_validation() {
        let validator = DefaultConfigValidator::new();
        
        // Valid framerates
        assert!(validator.validate_framerate(&Framerate { numerator: 30, denominator: 1 }).is_ok());
        assert!(validator.validate_framerate(&Framerate { numerator: 60, denominator: 1 }).is_ok());
        
        // Invalid framerates
        assert!(validator.validate_framerate(&Framerate { numerator: 30, denominator: 0 }).is_err());
        assert!(validator.validate_framerate(&Framerate { numerator: 200, denominator: 1 }).is_err());
    }
    
    #[test]
    fn test_source_name_validation() {
        let validator = DefaultConfigValidator::new();
        
        // Valid names
        assert!(validator.validate_source_name("test-source").is_ok());
        assert!(validator.validate_source_name("source_123").is_ok());
        
        // Invalid names
        assert!(validator.validate_source_name("").is_err());
        assert!(validator.validate_source_name("source with spaces").is_err());
        assert!(validator.validate_source_name("a".repeat(100).as_str()).is_err());
    }
}
