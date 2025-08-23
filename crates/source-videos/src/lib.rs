pub mod config;
pub mod error;
pub mod file;
pub mod manager;
pub mod patterns;
pub mod pipeline;
pub mod rtsp;
pub mod source;

pub use config::{AppConfig, ServerConfig, VideoSourceConfig, VideoSourceType};
pub use error::{Result, SourceVideoError};
pub use file::{FileGenerator, BatchFileGenerator, generate_test_file};
pub use manager::{VideoSourceManager, SourceInfo, SourceManagerBuilder};
pub use patterns::{TestPattern, PatternRotator};
pub use rtsp::{RtspServer, RtspServerBuilder, create_test_rtsp_server};
pub use source::{VideoSource, SourceState};

use once_cell::sync::OnceCell;

static GST_INITIALIZED: OnceCell<()> = OnceCell::new();

pub fn init() -> Result<()> {
    GST_INITIALIZED.get_or_try_init(|| {
        gstreamer::init()
            .map_err(|e| SourceVideoError::config(format!("Failed to initialize GStreamer: {}", e)))
    })?;
    Ok(())
}

pub fn ensure_initialized() {
    if GST_INITIALIZED.get().is_none() {
        init().expect("Failed to initialize GStreamer");
    }
}

pub struct SourceVideos {
    manager: VideoSourceManager,
    rtsp_server: Option<RtspServer>,
}

impl SourceVideos {
    pub fn new() -> Result<Self> {
        ensure_initialized();
        Ok(Self {
            manager: VideoSourceManager::new(),
            rtsp_server: None,
        })
    }
    
    pub fn with_config(config: AppConfig) -> Result<Self> {
        ensure_initialized();
        
        let manager = VideoSourceManager::new();
        
        for source_config in config.sources {
            manager.add_source(source_config)?;
        }
        
        let rtsp_server = if !manager.list_sources().is_empty() {
            let mut server = RtspServer::new(config.server)?;
            
            for info in manager.list_sources() {
                let source_config = VideoSourceConfig::rtsp(&info.name, &info.name);
                server.add_source(source_config)?;
            }
            
            server.start()?;
            Some(server)
        } else {
            None
        };
        
        Ok(Self {
            manager,
            rtsp_server,
        })
    }
    
    pub fn add_test_pattern(&mut self, name: &str, pattern: &str) -> Result<String> {
        let config = VideoSourceConfig::test_pattern(name, pattern);
        
        let id = self.manager.add_source(config.clone())?;
        
        if let Some(server) = &mut self.rtsp_server {
            server.add_source(VideoSourceConfig::rtsp(name, name))?;
        }
        
        Ok(id)
    }
    
    pub fn add_source(&mut self, config: VideoSourceConfig) -> Result<String> {
        let id = self.manager.add_source(config.clone())?;
        
        if let Some(server) = &mut self.rtsp_server {
            if matches!(config.source_type, VideoSourceType::Rtsp { .. }) {
                server.add_source(config)?;
            }
        }
        
        Ok(id)
    }
    
    pub fn remove_source(&mut self, id_or_name: &str) -> Result<()> {
        if let Some(server) = &mut self.rtsp_server {
            let _ = server.remove_source(id_or_name);
        }
        
        self.manager.remove_source(id_or_name)
    }
    
    pub fn list_sources(&self) -> Vec<SourceInfo> {
        self.manager.list_sources()
    }
    
    pub fn start_rtsp_server(&mut self, port: u16) -> Result<()> {
        if self.rtsp_server.is_some() {
            return Ok(());
        }
        
        let mut server = RtspServerBuilder::new()
            .port(port)
            .build()?;
        
        for info in self.manager.list_sources() {
            let config = VideoSourceConfig::rtsp(&info.name, &info.name);
            server.add_source(config)?;
        }
        
        server.start()?;
        self.rtsp_server = Some(server);
        
        Ok(())
    }
    
    pub fn get_rtsp_urls(&self) -> Vec<String> {
        if let Some(server) = &self.rtsp_server {
            server.list_sources()
                .into_iter()
                .map(|mount| server.get_url(&mount))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn manager(&self) -> &VideoSourceManager {
        &self.manager
    }
    
    pub fn rtsp_server(&self) -> Option<&RtspServer> {
        self.rtsp_server.as_ref()
    }
}

impl Default for SourceVideos {
    fn default() -> Self {
        Self::new().expect("Failed to create SourceVideos")
    }
}

pub fn quick_start() -> Result<SourceVideos> {
    let mut sv = SourceVideos::new()?;
    
    sv.add_test_pattern("test1", "smpte")?;
    sv.add_test_pattern("test2", "ball")?;
    sv.add_test_pattern("test3", "snow")?;
    
    sv.start_rtsp_server(8554)?;
    
    Ok(sv)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initialization() {
        init().unwrap();
        ensure_initialized();
    }
    
    #[test]
    fn test_source_videos_creation() {
        let sv = SourceVideos::new().unwrap();
        assert_eq!(sv.list_sources().len(), 0);
    }
    
    #[test]
    fn test_add_test_pattern() {
        let mut sv = SourceVideos::new().unwrap();
        sv.add_test_pattern("test", "smpte").unwrap();
        
        let sources = sv.list_sources();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "test");
    }
}