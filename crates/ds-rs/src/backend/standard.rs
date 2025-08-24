#![allow(unused)]
use super::{Backend, BackendCapabilities, BackendType};
use crate::error::{DeepStreamError, Result};
use crate::platform::PlatformInfo;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::collections::HashMap;

pub struct StandardBackend {
    capabilities: BackendCapabilities,
    platform: PlatformInfo,
}

impl StandardBackend {
    fn create_capabilities() -> BackendCapabilities {
        BackendCapabilities {
            supports_inference: true,   // CPU-based inference via ONNX
            supports_tracking: true,    // Centroid tracking
            supports_osd: true,         // Can do basic overlays
            supports_batching: false,   // Limited batching via compositor
            supports_hardware_decode: false, // Software decode only
            max_batch_size: 4,          // Limited by compositor
            available_elements: vec![
                "compositor".to_string(),
                "queue".to_string(),
                "videoconvert".to_string(),
                "videoscale".to_string(),
                "textoverlay".to_string(),
                "videobox".to_string(),
                "identity".to_string(),
                "cpu-detector".to_string(),
                "cpu-tracker".to_string(),
                "cpu-osd".to_string(),
            ],
        }
    }
    
    fn create_element(element_type: &str, name: Option<&str>) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make(element_type);
        
        if let Some(n) = name {
            builder = builder.name(n);
        }
        
        builder.build().map_err(|_| DeepStreamError::ElementCreation {
            element: element_type.to_string(),
        })
    }
}

impl Backend for StandardBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Standard
    }
    
    fn capabilities(&self) -> &BackendCapabilities {
        &self.capabilities
    }
    
    fn is_available() -> bool {
        super::detector::check_element_availability("compositor") &&
        super::detector::check_element_availability("videoconvert")
    }
    
    fn new(platform: &PlatformInfo) -> Result<Box<dyn Backend>> {
        if !Self::is_available() {
            return Err(DeepStreamError::BackendNotAvailable {
                backend: "Standard".to_string(),
            });
        }
        
        Ok(Box::new(Self {
            capabilities: Self::create_capabilities(),
            platform: platform.clone(),
        }))
    }
    
    fn create_stream_mux(&self, name: Option<&str>) -> Result<gst::Element> {
        // Use compositor as a batching replacement for nvstreammux
        let compositor = Self::create_element("compositor", name)?;
        /*
            background          : Background type
                                flags: readable, writable
                                Enum "GstCompositorBackground" Default: 0, "checker"
                                    (0): checker          - Checker pattern
                                    (1): black            - Black
                                    (2): white            - White
                                    (3): transparent      - Transparent Background to enable further compositing
         */
        // Set up compositor for grid layout similar to nvstreammux
        // Use set_property_from_str for enum properties
        compositor.set_property_from_str("background", "black"); // Black background
        
        // Important: Set compositor to not wait for all pads
        // This allows it to start outputting when any pad has data
        compositor.set_property_from_str("start-time-selection", "first"); // Use first pad's timestamp
        
        // Set to ignore inactive pads to ensure continuous output
        compositor.set_property("ignore-inactive-pads", true);
        
        // Note: max-threads property may not exist on all compositor versions

        Ok(compositor)
    }
    
    fn create_inference(&self, name: Option<&str>, config_path: &str) -> Result<gst::Element> {
        // Try to create CPU detector with ONNX model
        match super::cpu_vision::elements::create_cpu_detector(name, Some(config_path)) {
            Ok(detector) => {
                log::info!("Standard backend: Using CPU detector for inference");
                Ok(detector)
            }
            Err(e) => {
                log::warn!("Failed to create CPU detector: {}, falling back to fakesink", e);
                
                // Fallback to fakesink if CPU detector fails
                let fakesink = Self::create_element("fakesink", name)?;
                fakesink.set_property("sync", false);
                fakesink.set_property("async", false);
                Ok(fakesink)
            }
        }
    }
    
    fn create_tracker(&self, name: Option<&str>) -> Result<gst::Element> {
        // Try to create CPU tracker
        match super::cpu_vision::elements::create_cpu_tracker(name) {
            Ok(tracker) => {
                log::info!("Standard backend: Using CPU tracker (Centroid algorithm)");
                Ok(tracker)
            }
            Err(e) => {
                log::warn!("Failed to create CPU tracker: {}, falling back to identity", e);
                
                // Fallback to identity if CPU tracker fails
                let identity = Self::create_element("identity", name)?;
                Ok(identity)
            }
        }
    }
    
    fn create_tiler(&self, name: Option<&str>) -> Result<gst::Element> {
        // Use compositor with manual pad positioning for tiling
        let compositor = Self::create_element("compositor", name)?;
        
        // Configure for 2x2 grid
        // Use set_property_from_str for enum properties
        compositor.set_property_from_str("background", "checker");
        
        log::info!("Standard backend: Using compositor for tiling");
        
        Ok(compositor)
    }
    
    fn create_osd(&self, name: Option<&str>) -> Result<gst::Element> {
        // Try to create CPU OSD for bounding box rendering
        match super::cpu_vision::elements::create_cpu_osd(name) {
            Ok(osd) => {
                log::info!("Standard backend: Using CPU OSD for visualization");
                Ok(osd)
            }
            Err(e) => {
                log::warn!("Failed to create CPU OSD: {}, falling back to text overlay", e);
                
                // Fallback to simple text overlay
                let bin = gst::Bin::builder()
                    .name(name.unwrap_or("osd-bin"))
                    .build();
                
                let convert = Self::create_element("videoconvert", Some("osd-convert"))?;
                let overlay = Self::create_element("textoverlay", Some("osd-overlay"))?;
                
                overlay.set_property("text", "Standard Backend - CPU Vision");
                overlay.set_property_from_str("valignment", "top");
                overlay.set_property_from_str("halignment", "left");
                overlay.set_property("font-desc", "Sans, 12");
                
                bin.add_many([&convert, &overlay])?;
                convert.link(&overlay)?;
                
                let sink_pad = convert.static_pad("sink").unwrap();
                let src_pad = overlay.static_pad("src").unwrap();
                
                bin.add_pad(&gst::GhostPad::with_target(&sink_pad)?)?;
                bin.add_pad(&gst::GhostPad::with_target(&src_pad)?)?;
                
                Ok(bin.upcast())
            }
        }
    }
    
    fn create_video_convert(&self, name: Option<&str>) -> Result<gst::Element> {
        Self::create_element("videoconvert", name)
    }
    
    fn create_video_sink(&self, name: Option<&str>) -> Result<gst::Element> {
        // Try different sinks in order of preference
        let sink = gst::ElementFactory::make("autovideosink")
            .name(name.unwrap_or("video-sink"))
            .build()
            .or_else(|_| {
                gst::ElementFactory::make("xvimagesink")
                    .name(name.unwrap_or("video-sink"))
                    .build()
            })
            .or_else(|_| {
                gst::ElementFactory::make("ximagesink")
                    .name(name.unwrap_or("video-sink"))
                    .build()
            })
            .or_else(|_| {
                gst::ElementFactory::make("fakesink")
                    .name(name.unwrap_or("video-sink"))
                    .build()
            })
            .map_err(|_| DeepStreamError::ElementCreation {
                element: "video sink".to_string(),
            })?;
        
        sink.set_property("sync", false);
        
        Ok(sink)
    }
    
    fn create_decoder(&self, name: Option<&str>) -> Result<gst::Element> {
        // Use software decoder
        let decoder = gst::ElementFactory::make("decodebin")
            .name(name.unwrap_or("decoder"))
            .build()
            .or_else(|_| {
                gst::ElementFactory::make("avdec_h264")
                    .name(name.unwrap_or("decoder"))
                    .build()
            })
            .map_err(|_| DeepStreamError::ElementCreation {
                element: "decoder".to_string(),
            })?;
        
        Ok(decoder)
    }
    
    fn configure_element(&self, element: &gst::Element, config: &HashMap<String, String>) -> Result<()> {
        for (key, value) in config {
            // Skip config-file-path as it's already handled during element creation
            // The CPU detector bin doesn't have this property
            if key == "config-file-path" {
                continue;
            }
            
            // Parse and set properties based on type
            if let Ok(int_val) = value.parse::<i32>() {
                element.set_property_from_str(key, &int_val.to_string());
            } else if let Ok(uint_val) = value.parse::<u32>() {
                element.set_property_from_str(key, &uint_val.to_string());
            } else if let Ok(bool_val) = value.parse::<bool>() {
                element.set_property_from_str(key, &bool_val.to_string());
            } else {
                element.set_property_from_str(key, value);
            }
        }
        Ok(())
    }
    
    fn get_element_mapping(&self, deepstream_element: &str) -> Option<&str> {
        match deepstream_element {
            "nvstreammux" => Some("compositor"),
            "nvinfer" => Some("cpu-detector"),
            "nvtracker" => Some("cpu-tracker"),
            "nvdsosd" => Some("textoverlay"),
            "nvtiler" => Some("compositor"),
            "nvvideoconvert" => Some("videoconvert"),
            "nveglglessink" => Some("autovideosink"),
            "nvv4l2decoder" => Some("decodebin"),
            _ => None,
        }
    }
}
