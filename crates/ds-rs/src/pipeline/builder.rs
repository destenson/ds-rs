#![allow(unused)]

use crate::backend::{BackendManager, BackendType};
use crate::elements::factory::ElementFactory;
use crate::error::{DeepStreamError, Result};
use crate::rendering::{RenderingConfig, RendererFactory, MetadataBridge};
use super::{Pipeline, StateManager};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::glib;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Builder for creating configured pipelines with fluent API
pub struct PipelineBuilder {
    name: String,
    backend_type: Option<BackendType>,
    elements: Vec<ElementConfig>,
    links: Vec<LinkConfig>,
    properties: HashMap<String, HashMap<String, glib::Value>>,
    string_properties: HashMap<String, HashMap<String, String>>,
    auto_flush_bus: bool,
    use_clock: Option<gst::Clock>,
    start_paused: bool,
    rendering_config: Option<RenderingConfig>,
    enable_dynamic_rendering: bool,
    metadata_bridge: Option<Arc<Mutex<MetadataBridge>>>,
}

#[derive(Debug, Clone)]
struct ElementConfig {
    name: String,
    factory_name: String,
    properties: HashMap<String, glib::Value>,
}

#[derive(Debug, Clone)]
struct LinkConfig {
    source: String,
    destination: String,
    caps: Option<gst::Caps>,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            backend_type: None,
            elements: Vec::new(),
            links: Vec::new(),
            properties: HashMap::new(),
            string_properties: HashMap::new(),
            auto_flush_bus: true,
            use_clock: None,
            start_paused: false,
            rendering_config: None,
            enable_dynamic_rendering: false,
            metadata_bridge: None,
        }
    }
    
    /// Set the backend type for element creation
    pub fn backend(mut self, backend_type: BackendType) -> Self {
        self.backend_type = Some(backend_type);
        self
    }
    
    /// Add an element to the pipeline
    pub fn add_element(mut self, name: impl Into<String>, factory_name: impl Into<String>) -> Self {
        let element_config = ElementConfig {
            name: name.into(),
            factory_name: factory_name.into(),
            properties: HashMap::new(),
        };
        self.elements.push(element_config);
        self
    }
    
    /// Add an element with properties
    pub fn add_element_with_props(
        mut self,
        name: impl Into<String>,
        factory_name: impl Into<String>,
        properties: HashMap<String, glib::Value>,
    ) -> Self {
        let element_config = ElementConfig {
            name: name.into(),
            factory_name: factory_name.into(),
            properties,
        };
        self.elements.push(element_config);
        self
    }
    
    /// Set a property on an element
    pub fn set_property(
        mut self,
        element_name: impl Into<String>,
        property_name: impl Into<String>,
        value: impl Into<glib::Value>,
    ) -> Self {
        let element_name = element_name.into();
        let property_name = property_name.into();
        let value = value.into();
        
        self.properties
            .entry(element_name)
            .or_insert_with(HashMap::new)
            .insert(property_name, value);
        
        self
    }
    
    /// Set a property using a string value (useful for enums)
    /// This will use GStreamer's set_property_from_str method which handles enum conversion
    pub fn set_property_from_str(
        mut self,
        element_name: impl Into<String>,
        property_name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        let element_name = element_name.into();
        let property_name = property_name.into();
        let value = value.into();
        
        self.string_properties
            .entry(element_name)
            .or_insert_with(HashMap::new)
            .insert(property_name, value);
        
        self
    }
    
    /// Link two elements
    pub fn link(mut self, source: impl Into<String>, destination: impl Into<String>) -> Self {
        self.links.push(LinkConfig {
            source: source.into(),
            destination: destination.into(),
            caps: None,
        });
        self
    }
    
    /// Link two elements with caps filter
    pub fn link_filtered(
        mut self,
        source: impl Into<String>,
        destination: impl Into<String>,
        caps: gst::Caps,
    ) -> Self {
        self.links.push(LinkConfig {
            source: source.into(),
            destination: destination.into(),
            caps: Some(caps),
        });
        self
    }
    
    /// Link multiple elements in sequence
    pub fn link_many(mut self, elements: Vec<String>) -> Self {
        for i in 0..elements.len() - 1 {
            self.links.push(LinkConfig {
                source: elements[i].clone(),
                destination: elements[i + 1].clone(),
                caps: None,
            });
        }
        self
    }
    
    /// Set whether to automatically flush the bus on NULL state
    pub fn auto_flush_bus(mut self, auto_flush: bool) -> Self {
        self.auto_flush_bus = auto_flush;
        self
    }
    
    /// Set a specific clock for the pipeline
    pub fn use_clock(mut self, clock: gst::Clock) -> Self {
        self.use_clock = Some(clock);
        self
    }
    
    /// Start the pipeline in paused state
    pub fn start_paused(mut self, paused: bool) -> Self {
        self.start_paused = paused;
        self
    }
    
    /// Add a source element (uridecodebin)
    pub fn add_source(self, name: impl Into<String>, uri: impl Into<String>) -> Self {
        let name = name.into();
        let uri = uri.into();
        
        self.add_element(&name, "uridecodebin")
            .set_property(&name, "uri", uri)
    }
    
    /// Add a file source
    pub fn add_file_source(self, name: impl Into<String>, location: impl Into<String>) -> Self {
        let name = name.into();
        let location = location.into();
        
        self.add_element(&name, "filesrc")
            .set_property(&name, "location", location)
    }
    
    /// Add a test source
    pub fn add_test_source(self, name: impl Into<String>) -> Self {
        self.add_element(name, "videotestsrc")
    }
    
    /// Add a sink element
    pub fn add_sink(self, name: impl Into<String>, sink_type: impl Into<String>) -> Self {
        self.add_element(name, sink_type)
    }
    
    /// Add an autovideosink
    pub fn add_auto_sink(self, name: impl Into<String>) -> Self {
        self.add_element(name, "autovideosink")
    }
    
    /// Add a queue element
    pub fn add_queue(self, name: impl Into<String>) -> Self {
        self.add_element(name, "queue")
    }
    
    /// Add a caps filter
    pub fn add_caps_filter(self, name: impl Into<String>, caps: gst::Caps) -> Self {
        let name = name.into();
        self.add_element(&name, "capsfilter")
            .set_property(&name, "caps", caps)
    }
    
    /// Enable dynamic bounding box rendering
    pub fn with_rendering(mut self, config: RenderingConfig) -> Self {
        self.rendering_config = Some(config);
        self.enable_dynamic_rendering = true;
        self
    }
    
    /// Enable default bounding box rendering
    pub fn with_default_rendering(mut self) -> Self {
        self.rendering_config = Some(RenderingConfig::default());
        self.enable_dynamic_rendering = true;
        self
    }
    
    /// Enable ball tracking visualization
    pub fn with_ball_tracking_rendering(mut self) -> Self {
        self.rendering_config = Some(RenderingConfig::for_ball_tracking());
        self.enable_dynamic_rendering = true;
        self
    }
    
    /// Set a custom metadata bridge
    pub fn with_metadata_bridge(mut self, bridge: Arc<Mutex<MetadataBridge>>) -> Self {
        self.metadata_bridge = Some(bridge);
        self
    }
    
    /// Add a dynamic OSD element with rendering support
    pub fn add_dynamic_osd(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.enable_dynamic_rendering = true;
        
        // Add OSD element
        self = self.add_element(&name, "nvdsosd");
        
        // If no rendering config is set, use default
        if self.rendering_config.is_none() {
            self.rendering_config = Some(RenderingConfig::default());
        }
        
        self
    }
    
    /// Build the pipeline
    pub fn build(self) -> Result<Pipeline> {
        // Initialize GStreamer if not already done
        let _ = gst::init();
        
        // Create backend manager
        let backend_manager = match self.backend_type {
            Some(backend_type) => Arc::new(BackendManager::with_backend(backend_type)?),
            None => Arc::new(BackendManager::new()?),
        };
        
        // Create the GStreamer pipeline
        let gst_pipeline = gst::Pipeline::builder()
            .name(&self.name)
            .build();
        
        // Create element factory
        let factory = ElementFactory::new(backend_manager.clone());
        
        // Create and add elements
        let mut elements_map = HashMap::new();
        
        for element_config in &self.elements {
            let element = if element_config.factory_name.starts_with("nv") {
                // Use backend-specific element creation for DeepStream elements
                match element_config.factory_name.as_str() {
                    "nvstreammux" => factory.create_stream_mux(Some(&element_config.name))?,
                    "nvinfer" => {
                        // For inference, we need a config path
                        let config_path = element_config.properties.get("config-file-path")
                            .and_then(|v| v.get::<String>().ok())
                            .unwrap_or_default();
                        factory.create_inference(Some(&element_config.name), &config_path)?
                    }
                    "nvtracker" => factory.create_tracker(Some(&element_config.name))?,
                    "nvtiler" => factory.create_tiler(Some(&element_config.name))?,
                    "nvosd" | "nvdsosd" => factory.create_osd(Some(&element_config.name))?,
                    "nvvideoconvert" => factory.create_video_convert(Some(&element_config.name))?,
                    _ => {
                        // Fallback to standard element creation
                        factory.create_standard_element(&element_config.factory_name, Some(&element_config.name))?
                    }
                }
            } else {
                // Standard GStreamer element
                factory.create_standard_element(&element_config.factory_name, Some(&element_config.name))?
            };
            
            // Set element properties
            for (prop_name, prop_value) in &element_config.properties {
                element.set_property_from_value(prop_name, prop_value);
            }
            
            // Apply properties from the separate properties map
            if let Some(props) = self.properties.get(&element_config.name) {
                for (prop_name, prop_value) in props {
                    element.set_property_from_value(prop_name, prop_value);
                }
            }
            
            // Apply string properties using set_property_from_str
            if let Some(str_props) = self.string_properties.get(&element_config.name) {
                for (prop_name, prop_value) in str_props {
                    element.set_property_from_str(prop_name, prop_value);
                }
            }
            
            gst_pipeline.add(&element).map_err(|_| {
                DeepStreamError::Pipeline(format!(
                    "Failed to add element {} to pipeline",
                    element_config.name
                ))
            })?;
            
            elements_map.insert(element_config.name.clone(), element);
        }
        
        // Link elements
        for link_config in &self.links {
            let source = elements_map.get(&link_config.source)
                .ok_or_else(|| DeepStreamError::ElementNotFound {
                    element: link_config.source.clone(),
                })?;
            
            let destination = elements_map.get(&link_config.destination)
                .ok_or_else(|| DeepStreamError::ElementNotFound {
                    element: link_config.destination.clone(),
                })?;
            
            if let Some(caps) = &link_config.caps {
                source.link_filtered(destination, caps).map_err(|_| {
                    DeepStreamError::PadLinking(format!(
                        "Failed to link {} to {} with caps filter",
                        link_config.source, link_config.destination
                    ))
                })?;
            } else {
                source.link(destination).map_err(|_| {
                    DeepStreamError::PadLinking(format!(
                        "Failed to link {} to {}",
                        link_config.source, link_config.destination
                    ))
                })?;
            }
        }
        
        // Configure pipeline settings
        gst_pipeline.set_auto_flush_bus(self.auto_flush_bus);
        if let Some(clock) = self.use_clock {
            gst_pipeline.use_clock(Some(&clock));
        }
        
        // Configure dynamic rendering if enabled
        if self.enable_dynamic_rendering {
            // Create metadata bridge if not provided
            let metadata_bridge = self.metadata_bridge.unwrap_or_else(|| {
                Arc::new(Mutex::new(MetadataBridge::new()))
            });
            
            // Find OSD elements and configure them
            for (element_name, element) in &elements_map {
                if element_name.contains("osd") || element_name.contains("OSD") {
                    // Configure the OSD element with rendering config
                    if let Some(ref config) = self.rendering_config {
                        configure_osd_for_rendering(
                            element,
                            config,
                            metadata_bridge.clone(),
                            backend_manager.backend_type(),
                        )?;
                    }
                }
            }
        }
        
        // Create state manager
        let state_manager = Arc::new(Mutex::new(StateManager::new()));
        
        // Create the pipeline wrapper
        let pipeline = Pipeline {
            gst_pipeline,
            state_manager,
            bus_watcher: None,
            backend_manager,
            name: self.name,
        };
        
        // Set initial state if requested
        if self.start_paused {
            pipeline.pause()?;
        }
        
        Ok(pipeline)
    }
}

/// Configure an OSD element for dynamic rendering
fn configure_osd_for_rendering(
    osd_element: &gst::Element,
    config: &RenderingConfig,
    metadata_bridge: Arc<Mutex<MetadataBridge>>,
    backend_type: BackendType,
) -> Result<()> {
    use crate::rendering::BoundingBoxRenderer;
    
    // Create backend-specific renderer
    let mut renderer = RendererFactory::create_renderer_with_config(
        backend_type,
        Some(&format!("{}-renderer", osd_element.name())),
        config.clone(),
    )?;
    
    // Connect metadata bridge to renderer
    renderer.connect_metadata_source(metadata_bridge)?;
    
    // Configure OSD element based on rendering config
    if config.enable_bbox {
        osd_element.set_property("display-bbox", 1i32);
    }
    
    if config.enable_labels {
        osd_element.set_property("display-text", 1i32);
        
        // Set font configuration if supported
        let font_desc = format!("{} {}", 
            config.font_config.family,
            config.font_config.size as i32
        );
        osd_element.set_property("font-desc", &font_desc);
    }
    
    log::info!("Configured OSD element '{}' for dynamic rendering with {} backend",
              osd_element.name(), backend_type);
    
    Ok(())
}

/// Builder extensions for DeepStream-specific pipelines
impl PipelineBuilder {
    /// Add DeepStream inference element
    pub fn add_deepstream_inference(
        self,
        name: impl Into<String>,
        config_path: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let config_path = config_path.into();
        
        self.add_element(&name, "nvinfer")
            .set_property(&name, "config-file-path", config_path)
    }
    
    /// Add DeepStream tracker
    pub fn add_deepstream_tracker(
        self,
        name: impl Into<String>,
        config_path: Option<String>,
    ) -> Self {
        let name = name.into();
        let mut builder = self.add_element(&name, "nvtracker");
        
        if let Some(config) = config_path {
            builder = builder.set_property(&name, "ll-config-file", config);
        }
        
        builder
    }
    
    /// Add DeepStream stream muxer
    pub fn add_deepstream_mux(
        self,
        name: impl Into<String>,
        batch_size: u32,
        width: u32,
        height: u32,
    ) -> Self {
        let name = name.into();
        
        self.add_element(&name, "nvstreammux")
            .set_property(&name, "batch-size", batch_size)
            .set_property(&name, "width", width)
            .set_property(&name, "height", height)
    }
    
    /// Add DeepStream OSD
    pub fn add_deepstream_osd(self, name: impl Into<String>) -> Self {
        self.add_element(name, "nvdsosd")
    }
    
    /// Add DeepStream tiler
    pub fn add_deepstream_tiler(
        self,
        name: impl Into<String>,
        rows: u32,
        columns: u32,
    ) -> Self {
        let name = name.into();
        
        self.add_element(&name, "nvtiler")
            .set_property(&name, "rows", rows)
            .set_property(&name, "columns", columns)
    }
    
    /// Build a simple DeepStream pipeline
    pub fn build_deepstream_pipeline(
        name: impl Into<String>,
        uri: impl Into<String>,
        inference_config: impl Into<String>,
    ) -> Result<Pipeline> {
        Self::new(name)
            .backend(BackendType::DeepStream)
            .add_source("source", uri)
            .add_deepstream_mux("mux", 1, 1920, 1080)
            .add_deepstream_inference("pgie", inference_config)
            .add_deepstream_osd("osd")
            .add_element("converter", "nvvideoconvert")
            .add_auto_sink("sink")
            .link_many(vec![
                "mux".to_string(),
                "pgie".to_string(),
                "osd".to_string(),
                "converter".to_string(),
                "sink".to_string(),
            ])
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pipeline_builder() {
        let _ = gst::init();
        
        let pipeline = PipelineBuilder::new("test-pipeline")
            .backend(BackendType::Mock)
            .add_test_source("source")
            .add_queue("queue")
            .add_auto_sink("sink")
            .link("source", "queue")
            .link("queue", "sink")
            .build();
        
        assert!(pipeline.is_ok());
    }
    
    #[test]
    fn test_builder_properties() {
        let _ = gst::init();
        
        let pipeline = PipelineBuilder::new("test-pipeline")
            .backend(BackendType::Mock)
            .add_element("source", "videotestsrc")
            .set_property("source", "num-buffers", 100i32)
            .add_auto_sink("sink")
            .link("source", "sink")
            .auto_flush_bus(false)
            .start_paused(true)
            .build();
        
        assert!(pipeline.is_ok());
        let pipeline = pipeline.unwrap();
        assert!(pipeline.is_paused());
    }
    
    #[test]
    fn test_caps_filter() {
        let _ = gst::init();
        
        let caps = gst::Caps::builder("video/x-raw")
            .field("width", 640)
            .field("height", 480)
            .build();
        
        let pipeline = PipelineBuilder::new("test-pipeline")
            .backend(BackendType::Mock)
            .add_test_source("source")
            .add_caps_filter("filter", caps.clone())
            .add_auto_sink("sink")
            .link("source", "filter")
            .link("filter", "sink")
            .build();
        
        assert!(pipeline.is_ok());
    }
}
