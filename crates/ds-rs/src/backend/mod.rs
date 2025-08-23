pub mod detector;
pub mod deepstream;
pub mod standard;
pub mod mock;

use crate::error::Result;
use crate::platform::PlatformInfo;
use gstreamer as gst;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    DeepStream,
    Standard,
    Mock,
}

impl BackendType {
    pub fn name(&self) -> &'static str {
        match self {
            BackendType::DeepStream => "DeepStream",
            BackendType::Standard => "Standard GStreamer",
            BackendType::Mock => "Mock",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    pub supports_inference: bool,
    pub supports_tracking: bool,
    pub supports_osd: bool,
    pub supports_batching: bool,
    pub supports_hardware_decode: bool,
    pub max_batch_size: u32,
    pub available_elements: Vec<String>,
}

impl Default for BackendCapabilities {
    fn default() -> Self {
        Self {
            supports_inference: false,
            supports_tracking: false,
            supports_osd: false,
            supports_batching: false,
            supports_hardware_decode: false,
            max_batch_size: 1,
            available_elements: Vec::new(),
        }
    }
}

pub trait Backend: Send + Sync {
    fn backend_type(&self) -> BackendType;
    
    fn capabilities(&self) -> &BackendCapabilities;
    
    fn is_available() -> bool where Self: Sized;
    
    fn new(platform: &PlatformInfo) -> Result<Box<dyn Backend>> where Self: Sized;
    
    fn create_stream_mux(&self, name: Option<&str>) -> Result<gst::Element>;
    
    fn create_inference(&self, name: Option<&str>, config_path: &str) -> Result<gst::Element>;
    
    fn create_tracker(&self, name: Option<&str>) -> Result<gst::Element>;
    
    fn create_tiler(&self, name: Option<&str>) -> Result<gst::Element>;
    
    fn create_osd(&self, name: Option<&str>) -> Result<gst::Element>;
    
    fn create_video_convert(&self, name: Option<&str>) -> Result<gst::Element>;
    
    fn create_video_sink(&self, name: Option<&str>) -> Result<gst::Element>;
    
    fn create_decoder(&self, name: Option<&str>) -> Result<gst::Element>;
    
    fn configure_element(&self, element: &gst::Element, config: &HashMap<String, String>) -> Result<()>;
    
    fn get_element_mapping(&self, deepstream_element: &str) -> Option<&str>;
}

pub struct BackendManager {
    backend: Box<dyn Backend>,
    platform: PlatformInfo,
}

impl BackendManager {
    pub fn new() -> Result<Self> {
        let platform = PlatformInfo::detect()?;
        let backend = detector::detect_and_create_backend(&platform)?;
        
        log::info!(
            "Initialized {} backend on {:?} platform",
            backend.backend_type().name(),
            platform.platform
        );
        
        Ok(Self { backend, platform })
    }
    
    pub fn with_backend(backend_type: BackendType) -> Result<Self> {
        let platform = PlatformInfo::detect()?;
        let backend = match backend_type {
            BackendType::DeepStream => {
                if deepstream::DeepStreamBackend::is_available() {
                    deepstream::DeepStreamBackend::new(&platform)?
                } else {
                    return Err(crate::error::DeepStreamError::BackendNotAvailable {
                        backend: "DeepStream".to_string(),
                    });
                }
            }
            BackendType::Standard => standard::StandardBackend::new(&platform)?,
            BackendType::Mock => mock::MockBackend::new(&platform)?,
        };
        
        log::info!(
            "Initialized {} backend on {:?} platform",
            backend.backend_type().name(),
            platform.platform
        );
        
        Ok(Self { backend, platform })
    }
    
    pub fn backend(&self) -> &dyn Backend {
        self.backend.as_ref()
    }
    
    pub fn platform(&self) -> &PlatformInfo {
        &self.platform
    }
    
    pub fn capabilities(&self) -> &BackendCapabilities {
        self.backend.capabilities()
    }
    
    pub fn backend_type(&self) -> BackendType {
        self.backend.backend_type()
    }
}