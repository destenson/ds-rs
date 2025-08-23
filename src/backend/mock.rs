use super::{Backend, BackendCapabilities, BackendType};
use crate::error::{DeepStreamError, Result};
use crate::platform::PlatformInfo;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::collections::HashMap;

pub struct MockBackend {
    capabilities: BackendCapabilities,
    platform: PlatformInfo,
}

impl MockBackend {
    fn create_capabilities() -> BackendCapabilities {
        BackendCapabilities {
            supports_inference: true,  // Mock inference
            supports_tracking: true,   // Mock tracking
            supports_osd: true,        // Mock OSD
            supports_batching: true,   // Mock batching
            supports_hardware_decode: false,
            max_batch_size: 10,
            available_elements: vec![
                "fakesrc".to_string(),
                "fakesink".to_string(),
                "identity".to_string(),
                "queue".to_string(),
                "tee".to_string(),
            ],
        }
    }
    
    fn create_mock_element(name: Option<&str>) -> Result<gst::Element> {
        let element = gst::ElementFactory::make("identity")
            .name(name.unwrap_or("mock-element"))
            .build()
            .map_err(|_| DeepStreamError::ElementCreation {
                element: "identity".to_string(),
            })?;
        
        // Add some properties to simulate processing
        element.set_property("signal-handoffs", false);
        element.set_property("silent", true);
        
        Ok(element)
    }
    
    fn create_mock_bin(name: &str, internal_elements: usize) -> Result<gst::Element> {
        let bin = gst::Bin::builder()
            .name(name)
            .build();
        
        let mut elements = Vec::new();
        
        // Create internal pipeline: identity -> queue -> identity
        for i in 0..internal_elements {
            let identity = gst::ElementFactory::make("identity")
                .name(&format!("{}-identity-{}", name, i))
                .property("silent", true)
                .build()
                .map_err(|_| DeepStreamError::ElementCreation {
                    element: "identity".to_string(),
                })?;
            
            elements.push(identity);
            
            if i < internal_elements - 1 {
                let queue = gst::ElementFactory::make("queue")
                    .name(&format!("{}-queue-{}", name, i))
                    .build()
                    .map_err(|_| DeepStreamError::ElementCreation {
                        element: "queue".to_string(),
                    })?;
                
                elements.push(queue);
            }
        }
        
        // Add all elements to bin
        for element in &elements {
            bin.add(element)?;
        }
        
        // Link elements
        for i in 0..elements.len() - 1 {
            elements[i].link(&elements[i + 1])?;
        }
        
        // Create ghost pads
        if !elements.is_empty() {
            let sink_pad = elements[0].static_pad("sink").unwrap();
            let src_pad = elements[elements.len() - 1].static_pad("src").unwrap();
            
            bin.add_pad(&gst::GhostPad::with_target(&sink_pad)?)?;
            bin.add_pad(&gst::GhostPad::with_target(&src_pad)?)?;
        }
        
        Ok(bin.upcast())
    }
}

impl Backend for MockBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Mock
    }
    
    fn capabilities(&self) -> &BackendCapabilities {
        &self.capabilities
    }
    
    fn is_available() -> bool {
        // Mock backend is always available
        true
    }
    
    fn new(platform: &PlatformInfo) -> Result<Box<dyn Backend>> {
        log::info!("Creating mock backend for testing");
        
        Ok(Box::new(Self {
            capabilities: Self::create_capabilities(),
            platform: platform.clone(),
        }))
    }
    
    fn create_stream_mux(&self, name: Option<&str>) -> Result<gst::Element> {
        // Create a simple tee element to simulate muxing
        let tee = gst::ElementFactory::make("tee")
            .name(name.unwrap_or("mock-streammux"))
            .property("allow-not-linked", true)
            .build()
            .map_err(|_| DeepStreamError::ElementCreation {
                element: "tee".to_string(),
            })?;
        
        log::debug!("Mock backend: Created mock stream mux");
        
        Ok(tee)
    }
    
    fn create_inference(&self, name: Option<&str>, config_path: &str) -> Result<gst::Element> {
        log::debug!("Mock backend: Creating mock inference with config: {}", config_path);
        
        // Create a bin that simulates inference processing
        let bin = Self::create_mock_bin(
            name.unwrap_or("mock-inference"),
            2  // identity -> queue -> identity
        )?;
        
        // Store config path as metadata
        bin.set_property_from_str("name", name.unwrap_or("mock-inference"));
        
        Ok(bin)
    }
    
    fn create_tracker(&self, name: Option<&str>) -> Result<gst::Element> {
        log::debug!("Mock backend: Creating mock tracker");
        
        // Simple identity element for tracking simulation
        Self::create_mock_element(name.or(Some("mock-tracker")))
    }
    
    fn create_tiler(&self, name: Option<&str>) -> Result<gst::Element> {
        log::debug!("Mock backend: Creating mock tiler");
        
        // Create a bin that simulates tiling
        Self::create_mock_bin(
            name.unwrap_or("mock-tiler"),
            1  // Just one identity element
        )
    }
    
    fn create_osd(&self, name: Option<&str>) -> Result<gst::Element> {
        log::debug!("Mock backend: Creating mock OSD");
        
        // Create identity element for OSD simulation
        Self::create_mock_element(name.or(Some("mock-osd")))
    }
    
    fn create_video_convert(&self, name: Option<&str>) -> Result<gst::Element> {
        log::debug!("Mock backend: Creating mock video converter");
        
        // Use identity as mock converter
        Self::create_mock_element(name.or(Some("mock-videoconvert")))
    }
    
    fn create_video_sink(&self, name: Option<&str>) -> Result<gst::Element> {
        let sink = gst::ElementFactory::make("fakesink")
            .name(name.unwrap_or("mock-videosink"))
            .property("sync", false)
            .property("async", false)
            .build()
            .map_err(|_| DeepStreamError::ElementCreation {
                element: "fakesink".to_string(),
            })?;
        
        log::debug!("Mock backend: Created mock video sink");
        
        Ok(sink)
    }
    
    fn create_decoder(&self, name: Option<&str>) -> Result<gst::Element> {
        log::debug!("Mock backend: Creating mock decoder");
        
        // Use identity as mock decoder
        Self::create_mock_element(name.or(Some("mock-decoder")))
    }
    
    fn configure_element(&self, element: &gst::Element, config: &HashMap<String, String>) -> Result<()> {
        log::debug!(
            "Mock backend: Configuring element {} with {} properties",
            element.name(),
            config.len()
        );
        
        // Mock configuration - just log the properties
        for (key, value) in config {
            log::trace!("  {} = {}", key, value);
        }
        
        Ok(())
    }
    
    fn get_element_mapping(&self, deepstream_element: &str) -> Option<&str> {
        // All DeepStream elements map to identity in mock
        match deepstream_element {
            "nvstreammux" => Some("tee"),
            "nvinfer" => Some("identity"),
            "nvtracker" => Some("identity"),
            "nvdsosd" => Some("identity"),
            "nvtiler" => Some("identity"),
            "nvvideoconvert" => Some("identity"),
            "nveglglessink" => Some("fakesink"),
            "nvv4l2decoder" => Some("identity"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_backend_creation() {
        let platform = PlatformInfo::detect().unwrap();
        let backend = MockBackend::new(&platform).unwrap();
        
        assert_eq!(backend.backend_type(), BackendType::Mock);
        assert!(backend.capabilities().supports_inference);
        assert!(backend.capabilities().supports_tracking);
    }
    
    #[test]
    fn test_mock_element_creation() {
        let _ = gst::init();
        let platform = PlatformInfo::detect().unwrap();
        let backend = MockBackend::new(&platform).unwrap();
        
        // Test all element creation methods
        assert!(backend.create_stream_mux(None).is_ok());
        assert!(backend.create_inference(None, "test.txt").is_ok());
        assert!(backend.create_tracker(None).is_ok());
        assert!(backend.create_tiler(None).is_ok());
        assert!(backend.create_osd(None).is_ok());
        assert!(backend.create_video_convert(None).is_ok());
        assert!(backend.create_video_sink(None).is_ok());
        assert!(backend.create_decoder(None).is_ok());
    }
}