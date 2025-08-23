pub mod factory;

use crate::config::{ServerConfig, VideoSourceConfig};
use crate::error::{Result, SourceVideoError};
use factory::MediaFactoryBuilder;
use gstreamer_rtsp_server as rtsp_server;
use gstreamer_rtsp_server::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct RtspServer {
    server: rtsp_server::RTSPServer,
    mounts: rtsp_server::RTSPMountPoints,
    sources: Arc<Mutex<HashMap<String, VideoSourceConfig>>>,
    port: u16,
    address: String,
}

impl RtspServer {
    pub fn new(config: ServerConfig) -> Result<Self> {
        let server = rtsp_server::RTSPServer::new();
        
        server.set_service(&config.port.to_string());
        server.set_address(&config.address);
        
        if let Some(max_conn) = config.max_connections.checked_add(0) {
            server.set_property("max-threads", max_conn);
        }
        
        let mounts = server.mount_points()
            .ok_or_else(|| SourceVideoError::server("Failed to get mount points"))?;
        
        Ok(Self {
            server,
            mounts,
            sources: Arc::new(Mutex::new(HashMap::new())),
            port: config.port,
            address: config.address,
        })
    }
    
    pub fn add_source(&mut self, config: VideoSourceConfig) -> Result<String> {
        let mount_point = if let crate::config::VideoSourceType::Rtsp { mount_point, .. } = &config.source_type {
            format!("/{}", mount_point)
        } else {
            format!("/{}", config.name)
        };
        
        let factory = MediaFactoryBuilder::new()
            .from_config(&config)?
            .build()?;
        
        self.mounts.add_factory(&mount_point, factory);
        
        if let Ok(mut sources) = self.sources.lock() {
            sources.insert(mount_point.clone(), config);
        }
        
        log::info!("Added RTSP source at: rtsp://{}:{}{}", 
                   self.address, self.port, mount_point);
        
        Ok(mount_point)
    }
    
    pub fn remove_source(&mut self, mount_point: &str) -> Result<()> {
        let path = if mount_point.starts_with('/') {
            mount_point.to_string()
        } else {
            format!("/{}", mount_point)
        };
        
        self.mounts.remove_factory(&path);
        
        if let Ok(mut sources) = self.sources.lock() {
            sources.remove(&path);
        }
        
        log::info!("Removed RTSP source: {}", path);
        Ok(())
    }
    
    pub fn list_sources(&self) -> Vec<String> {
        self.sources.lock()
            .map(|sources| sources.keys().cloned().collect())
            .unwrap_or_default()
    }
    
    pub fn start(&self) -> Result<()> {
        let _id = self.server.attach(None);
        // Note: attach returns Option<SourceId>, we'll assume it succeeded if no error is returned
        
        log::info!("RTSP server started on {}:{}", self.address, self.port);
        Ok(())
    }
    
    pub fn get_url(&self, mount_point: &str) -> String {
        let path = if mount_point.starts_with('/') {
            mount_point.to_string()
        } else {
            format!("/{}", mount_point)
        };
        
        format!("rtsp://{}:{}{}", self.address, self.port, path)
    }
    
    pub fn get_port(&self) -> u16 {
        self.port
    }
    
    pub fn get_address(&self) -> &str {
        &self.address
    }
}

pub struct RtspServerBuilder {
    config: ServerConfig,
    sources: Vec<VideoSourceConfig>,
}

impl RtspServerBuilder {
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
            sources: Vec::new(),
        }
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }
    
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.config.address = address.into();
        self
    }
    
    pub fn max_connections(mut self, max: u32) -> Self {
        self.config.max_connections = max;
        self
    }
    
    pub fn add_source(mut self, config: VideoSourceConfig) -> Self {
        self.sources.push(config);
        self
    }
    
    pub fn add_test_pattern(mut self, name: &str, _pattern: &str) -> Self {
        let config = VideoSourceConfig::rtsp(name, name);
        self.sources.push(config);
        self
    }
    
    pub fn build(self) -> Result<RtspServer> {
        let mut server = RtspServer::new(self.config)?;
        
        for source in self.sources {
            server.add_source(source)?;
        }
        
        Ok(server)
    }
}

pub fn create_test_rtsp_server(port: u16) -> Result<RtspServer> {
    RtspServerBuilder::new()
        .port(port)
        .add_test_pattern("test1", "smpte")
        .add_test_pattern("test2", "ball")
        .add_test_pattern("test3", "snow")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rtsp_server_builder() {
        gstreamer::init().unwrap();
        
        let server = RtspServerBuilder::new()
            .port(8554)
            .address("127.0.0.1")
            .max_connections(10)
            .build();
        
        assert!(server.is_ok());
        let server = server.unwrap();
        assert_eq!(server.get_port(), 8554);
        assert_eq!(server.get_address(), "127.0.0.1");
    }
    
    #[test]
    fn test_url_generation() {
        gstreamer::init().unwrap();
        
        let server = RtspServerBuilder::new()
            .port(8554)
            .address("localhost")
            .build()
            .unwrap();
        
        assert_eq!(server.get_url("/test"), "rtsp://localhost:8554/test");
        assert_eq!(server.get_url("test"), "rtsp://localhost:8554/test");
    }
}