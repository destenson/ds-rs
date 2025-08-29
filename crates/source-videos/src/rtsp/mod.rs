pub mod factory;

use crate::config::{RtspServerConfig, VideoSourceConfig};
use crate::error::{Result, SourceVideoError};
use crate::network::{NetworkConditions, NetworkProfile};
use crate::watch::FileSystemEvent;
use factory::MediaFactoryBuilder;
use gstreamer_rtsp_server as rtsp_server;
use gstreamer_rtsp_server::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct RtspServer {
    server: rtsp_server::RTSPServer,
    mounts: rtsp_server::RTSPMountPoints,
    sources: Arc<Mutex<HashMap<String, VideoSourceConfig>>>,
    port: u16,
    address: String,
    global_network_profile: Option<NetworkProfile>,
    per_source_network: HashMap<String, NetworkProfile>,
}

impl RtspServer {
    pub fn new(config: RtspServerConfig) -> Result<Self> {
        let server = rtsp_server::RTSPServer::new();

        server.set_service(&config.port.to_string());
        server.set_address(&config.address);

        // Set max threads on the thread pool instead of the server directly
        if config.max_connections > 0 {
            if let Some(thread_pool) = server.thread_pool() {
                thread_pool.set_max_threads(config.max_connections as i32);
            }
        }

        let mounts = server
            .mount_points()
            .ok_or_else(|| SourceVideoError::server("Failed to get mount points"))?;

        Ok(Self {
            server,
            mounts,
            sources: Arc::new(Mutex::new(HashMap::new())),
            port: config.port,
            address: config.address,
            global_network_profile: None,
            per_source_network: HashMap::new(),
        })
    }

    pub fn add_source(&mut self, config: VideoSourceConfig) -> Result<String> {
        let mount_point =
            if let crate::config::VideoSourceType::Rtsp { mount_point, .. } = &config.source_type {
                format!("/{}", mount_point)
            } else {
                format!("/{}", config.name)
            };

        // Build factory with network profile if configured
        let mut factory_builder = MediaFactoryBuilder::new().from_config(&config)?;

        // Apply per-source network profile if exists, otherwise use global
        if let Some(profile) = self.per_source_network.get(&config.name) {
            factory_builder = factory_builder.network_profile(*profile);
        } else if let Some(profile) = self.global_network_profile {
            factory_builder = factory_builder.network_profile(profile);
        }

        let factory = factory_builder.build()?;

        self.mounts.add_factory(&mount_point, factory);

        if let Ok(mut sources) = self.sources.lock() {
            sources.insert(mount_point.clone(), config);
        }

        log::info!(
            "Added RTSP source at: rtsp://{}:{}{}",
            self.address,
            self.port,
            mount_point
        );

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
        self.sources
            .lock()
            .map(|sources| sources.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn start(&self) -> Result<()> {
        let _source_id = self.server.attach(None);

        log::info!("RTSP server started on {}:{}", self.address, self.port);
        Ok(())
    }

    pub fn get_url(&self, mount_point: &str) -> String {
        let path = if mount_point.starts_with('/') {
            mount_point.to_string()
        } else {
            format!("/{}", mount_point)
        };

        let addr = match self.address.as_str() {
            "0.0.0.0" => "localhost",
            _ => &self.address,
        };
        format!("rtsp://{}:{}{}", addr, self.port, path)
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    // File watching integration methods for RTSP server
    pub fn update_source(&mut self, mount_point: &str, config: VideoSourceConfig) -> Result<()> {
        // Remove existing source if it exists
        let _ = self.remove_source(mount_point);

        // Add the updated source
        self.add_source(config)?;

        log::info!("Updated RTSP source at mount point: {}", mount_point);
        Ok(())
    }

    pub fn handle_file_event(&mut self, event: &FileSystemEvent) -> Result<()> {
        let path = event.path();

        match event {
            FileSystemEvent::Created(metadata) => {
                if crate::file_utils::is_video_file(&metadata.path) {
                    log::info!("New video file detected for RTSP: {}", path.display());

                    let container = crate::file_utils::detect_container_format(path)
                        .unwrap_or(crate::config_types::FileContainer::Mp4);

                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("video")
                        .to_string();

                    let config = VideoSourceConfig {
                        name: name.clone(),
                        source_type: crate::config_types::VideoSourceType::File {
                            path: path.display().to_string(),
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

                    self.add_source(config)?;
                }
            }
            FileSystemEvent::Modified(metadata) => {
                log::info!(
                    "Video file modified (RTSP will reload on next client connect): {}",
                    path.display()
                );
                // RTSP server creates new pipelines on client connect, so no action needed
            }
            FileSystemEvent::Deleted(metadata) => {
                log::info!("Video file deleted from RTSP: {}", path.display());
                // Find and remove the source with this file path
                let sources = self.sources.lock().map(|s| s.clone()).unwrap_or_default();

                for (mount_point, config) in sources {
                    if let crate::config_types::VideoSourceType::File {
                        path: file_path, ..
                    } = &config.source_type
                    {
                        if PathBuf::from(file_path) == metadata.path {
                            self.remove_source(&mount_point)?;
                            break;
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn get_source_for_path(&self, path: &PathBuf) -> Option<(String, VideoSourceConfig)> {
        let sources = self.sources.lock().ok()?;

        for (mount_point, config) in sources.iter() {
            if let crate::config_types::VideoSourceType::File {
                path: file_path, ..
            } = &config.source_type
            {
                if PathBuf::from(file_path) == *path {
                    return Some((mount_point.clone(), config.clone()));
                }
            }
        }

        None
    }
}

pub struct RtspServerBuilder {
    config: RtspServerConfig,
    sources: Vec<VideoSourceConfig>,
    global_network_profile: Option<NetworkProfile>,
    per_source_network: HashMap<String, NetworkProfile>,
    custom_network_conditions: Option<NetworkConditions>,
}

impl RtspServerBuilder {
    pub fn new() -> Self {
        Self {
            config: RtspServerConfig::default(),
            sources: Vec::new(),
            global_network_profile: None,
            per_source_network: HashMap::new(),
            custom_network_conditions: None,
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

    pub fn add_test_pattern(mut self, name: &str, pattern: &str) -> Self {
        let config = VideoSourceConfig::test_pattern(name, pattern);
        self.sources.push(config);
        self
    }

    pub fn add_test_pattern_with_network(
        mut self,
        name: &str,
        pattern: &str,
        profile: NetworkProfile,
    ) -> Self {
        let config = VideoSourceConfig::test_pattern(name, pattern);
        self.sources.push(config);
        self.per_source_network.insert(name.to_string(), profile);
        self
    }

    pub fn network_profile(mut self, profile: NetworkProfile) -> Self {
        self.global_network_profile = Some(profile);
        self
    }

    pub fn custom_network_conditions(
        mut self,
        packet_loss: f32,
        latency_ms: u32,
        bandwidth_kbps: u32,
        jitter_ms: u32,
    ) -> Self {
        self.custom_network_conditions = Some(NetworkConditions {
            packet_loss,
            latency_ms,
            bandwidth_kbps,
            jitter_ms,
            connection_dropped: false,
            duplicate_probability: 0.0,
            allow_reordering: true,
            min_delay_ms: latency_ms.saturating_sub(jitter_ms / 2),
            max_delay_ms: latency_ms + jitter_ms,
            delay_probability: if latency_ms > 0 { 100.0 } else { 0.0 },
        });
        self
    }

    pub fn per_source_network(mut self, source_name: &str, profile: NetworkProfile) -> Self {
        self.per_source_network
            .insert(source_name.to_string(), profile);
        self
    }

    pub fn build(self) -> Result<RtspServer> {
        let mut server = RtspServer::new(self.config)?;

        // Apply network configuration
        server.global_network_profile = self.global_network_profile;
        server.per_source_network = self.per_source_network.clone();

        // If custom conditions are set, convert to Custom profile
        if let Some(conditions) = self.custom_network_conditions {
            // We'll handle custom conditions by setting them directly when creating factories
            server.global_network_profile = Some(NetworkProfile::Custom);
        }

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
