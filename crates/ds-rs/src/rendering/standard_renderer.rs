#![allow(unused)]
//! Standard backend bounding box renderer using Cairo or text overlay

use super::{BoundingBoxRenderer, PerformanceMetrics, RenderingConfig};
use crate::error::{DeepStreamError, Result};
use crate::metadata::object::ObjectMeta;
use crate::rendering::metadata_bridge::MetadataBridge;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

/// Current frame data for rendering
#[derive(Clone, Default)]
struct FrameData {
    objects: Vec<ObjectMeta>,
    width: u32,
    height: u32,
    timestamp: Option<gst::ClockTime>,
}

/// Standard renderer using Cairo overlay or text overlay fallback
pub struct StandardRenderer {
    bin: gst::Bin,
    overlay_element: gst::Element,
    metrics: Arc<Mutex<PerformanceMetrics>>,
    config: Arc<Mutex<RenderingConfig>>,
    metadata_bridge: Option<Arc<Mutex<MetadataBridge>>>,
    frame_data: Arc<RwLock<FrameData>>,
    use_cairo: bool,
}

impl StandardRenderer {
    /// Create a new Standard backend renderer
    pub fn new(name: Option<&str>) -> Result<Self> {
        let bin = gst::Bin::builder()
            .name(name.unwrap_or("standard-renderer"))
            .build();

        // Create videoconvert for format conversion
        let convert_in = gst::ElementFactory::make("videoconvert")
            .name("render-convert-in")
            .build()
            .map_err(|_| DeepStreamError::ElementCreation {
                element: "videoconvert".to_string(),
            })?;

        // Try to create cairooverlay, fallback to textoverlay
        let (overlay_element, use_cairo) = match gst::ElementFactory::make("cairooverlay")
            .name("render-overlay")
            .build()
        {
            Ok(cairo) => {
                log::info!("Standard renderer using Cairo for bounding box rendering");
                (cairo, true)
            }
            Err(_) => {
                log::warn!("Cairo not available, using text overlay fallback");
                let text = gst::ElementFactory::make("textoverlay")
                    .name("render-textoverlay")
                    .property("text", "Standard Backend")
                    .property_from_str("valignment", "top")
                    .property_from_str("halignment", "left")
                    .property("font-desc", "Sans, 12")
                    .build()
                    .map_err(|_| DeepStreamError::ElementCreation {
                        element: "textoverlay".to_string(),
                    })?;
                (text, false)
            }
        };

        // Create output videoconvert
        let convert_out = gst::ElementFactory::make("videoconvert")
            .name("render-convert-out")
            .build()
            .map_err(|_| DeepStreamError::ElementCreation {
                element: "videoconvert".to_string(),
            })?;

        // Add elements to bin and link
        bin.add_many([&convert_in, &overlay_element, &convert_out])?;
        convert_in.link(&overlay_element)?;
        overlay_element.link(&convert_out)?;

        // Create ghost pads
        let sink_pad = convert_in.static_pad("sink").unwrap();
        let src_pad = convert_out.static_pad("src").unwrap();

        bin.add_pad(&gst::GhostPad::with_target(&sink_pad)?)?;
        bin.add_pad(&gst::GhostPad::with_target(&src_pad)?)?;

        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
        let config = Arc::new(Mutex::new(RenderingConfig::default()));
        let frame_data = Arc::new(RwLock::new(FrameData::default()));

        // Set up Cairo drawing callback if available
        if use_cairo {
            let config_clone = config.clone();
            let frame_data_clone = frame_data.clone();
            let metrics_clone = metrics.clone();

            // Cairo drawing is only available when cairo-rs is available
            // For now, we'll skip the signal connection and use probes instead
            log::info!(
                "Cairo overlay created, but drawing callback not implemented without cairo-rs"
            );
        }

        // Set up probe to extract video dimensions
        let frame_data_clone = frame_data.clone();

        sink_pad.add_probe(gst::PadProbeType::BUFFER, move |pad, info| {
            if let Some(_buffer) = info.buffer() {
                // Extract caps to get video dimensions
                if let Some(caps) = pad.current_caps() {
                    if let Some(structure) = caps.structure(0) {
                        let width = structure.get::<i32>("width").unwrap_or(1920) as u32;
                        let height = structure.get::<i32>("height").unwrap_or(1080) as u32;

                        if let Ok(mut data) = frame_data_clone.write() {
                            data.width = width;
                            data.height = height;
                        }
                    }
                }
            }
            gst::PadProbeReturn::Ok
        });

        log::info!(
            "Standard renderer created with {} overlay",
            if use_cairo { "Cairo" } else { "text" }
        );

        Ok(Self {
            bin,
            overlay_element,
            metrics,
            config,
            metadata_bridge: None,
            frame_data,
            use_cairo,
        })
    }
}

impl BoundingBoxRenderer for StandardRenderer {
    fn initialize(&mut self, config: &RenderingConfig) -> Result<()> {
        *self.config.lock().unwrap() = config.clone();

        // Configure text overlay if not using Cairo
        if !self.use_cairo && config.enable_labels {
            let font_desc = format!(
                "{} {}",
                config.font_config.family, config.font_config.size as i32
            );
            self.overlay_element.set_property("font-desc", &font_desc);
        }

        log::debug!("Standard renderer initialized with config");
        Ok(())
    }

    fn render_frame(&mut self, objects: &[ObjectMeta], timestamp: gst::ClockTime) -> Result<()> {
        // Update frame data
        if let Ok(mut data) = self.frame_data.write() {
            data.objects = objects.to_vec();
            data.timestamp = Some(timestamp);
        }

        // Update metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.objects_rendered += objects.len() as u64;
        }

        // If using text overlay, update text with object info
        if !self.use_cairo {
            let text = format_objects_as_text(objects);
            self.overlay_element.set_property("text", &text);
        }

        log::trace!(
            "Rendering {} objects at timestamp {}",
            objects.len(),
            timestamp
        );
        Ok(())
    }

    fn update_config(&mut self, config: &RenderingConfig) -> Result<()> {
        self.initialize(config)
    }

    fn get_element(&self) -> &gst::Element {
        self.bin.upcast_ref()
    }

    fn connect_metadata_source(&mut self, bridge: Arc<Mutex<MetadataBridge>>) -> Result<()> {
        self.metadata_bridge = Some(bridge.clone());

        // Set up probe to get objects from bridge
        let sink_pad = self
            .bin
            .static_pad("sink")
            .ok_or_else(|| DeepStreamError::PadNotFound {
                element: "standard-renderer".to_string(),
                pad: "sink".to_string(),
            })?;

        let frame_data_clone = self.frame_data.clone();

        sink_pad.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
            if let Some(_buffer) = info.buffer() {
                // Get current objects from bridge
                if let Ok(bridge_guard) = bridge.lock() {
                    if let Some((objects, timestamp)) = bridge_guard.get_current_objects() {
                        // Update frame data
                        if let Ok(mut data) = frame_data_clone.write() {
                            data.objects = objects;
                            data.timestamp = Some(timestamp);
                        }
                    }
                }
            }
            gst::PadProbeReturn::Ok
        });

        log::info!("Standard renderer connected to metadata source");
        Ok(())
    }

    fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }

    fn clear(&mut self) -> Result<()> {
        if let Ok(mut data) = self.frame_data.write() {
            data.objects.clear();
        }

        if !self.use_cairo {
            self.overlay_element.set_property("text", "");
        }

        if let Some(ref bridge) = self.metadata_bridge {
            bridge.lock().unwrap().clear();
        }

        log::trace!("Standard renderer cleared");
        Ok(())
    }
}

/// Draw bounding boxes using Cairo (stub without cairo-rs)
#[allow(unused)]
fn draw_bounding_boxes(
    _cr: &(), // Placeholder for cairo::Context
    frame_data: &FrameData,
    config: &RenderingConfig,
) {
    if !config.enable_bbox || frame_data.objects.is_empty() {
        return;
    }

    let width = frame_data.width as f64;
    let height = frame_data.height as f64;

    // Stub implementation without cairo-rs
    log::trace!("Would draw {} bounding boxes", frame_data.objects.len());
}

/// Draw a rounded rectangle (stub without cairo-rs)
#[allow(unused)]
fn draw_rounded_rectangle(
    _cr: &(), // Placeholder for cairo::Context
    _x: f64,
    _y: f64,
    _width: f64,
    _height: f64,
    _radius: f64,
) {
    // Stub implementation
}

/// Draw object label (stub without cairo-rs)
#[allow(unused)]
fn draw_label(
    _cr: &(), // Placeholder for cairo::Context
    obj: &ObjectMeta,
    _x: f64,
    _y: f64,
    _w: f64,
    _h: f64,
    config: &RenderingConfig,
) {
    // Stub implementation - just format the label
    let mut label = obj.obj_label.clone();

    if config.enable_tracking_id && obj.is_tracked() {
        label = format!("{} #{}", label, obj.object_id);
    }

    if config.enable_confidence {
        label = format!("{} {:.1}%", label, obj.confidence * 100.0);
    }

    log::trace!("Would draw label: {}", label);
}

/// Format objects as text for text overlay fallback
fn format_objects_as_text(objects: &[ObjectMeta]) -> String {
    if objects.is_empty() {
        return String::new();
    }

    let mut text = format!("Detected {} objects:\n", objects.len());

    for (i, obj) in objects.iter().enumerate().take(5) {
        let bbox = obj.bbox();
        text.push_str(&format!(
            "{}: {} ({:.0},{:.0}) {:.1}%\n",
            i + 1,
            obj.obj_label,
            bbox.left,
            bbox.top,
            obj.confidence * 100.0
        ));
    }

    if objects.len() > 5 {
        text.push_str(&format!("... and {} more", objects.len() - 5));
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_renderer_creation() {
        gst::init().unwrap();

        let renderer = StandardRenderer::new(Some("test-std-renderer")).unwrap();
        assert_eq!(renderer.bin.name(), "test-std-renderer");
    }

    #[test]
    fn test_format_objects_as_text() {
        let mut objects = Vec::new();

        for i in 0..3 {
            let mut obj = ObjectMeta::new(i as u64);
            obj.set_class(0, &format!("object_{}", i));
            obj.confidence = 0.85 + i as f32 * 0.05;
            obj.rect_params = crate::metadata::object::BoundingBox::new(
                10.0 * i as f32,
                20.0 * i as f32,
                50.0,
                60.0,
            );
            objects.push(obj);
        }

        let text = format_objects_as_text(&objects);
        assert!(text.contains("Detected 3 objects"));
        assert!(text.contains("object_0"));
        assert!(text.contains("85.0%"));
    }
}
