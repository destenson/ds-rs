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
            
            overlay_element.connect("draw", false, move |values| {
                let _element = values[0].get::<gst::Element>().unwrap();
                let cr = values[1].get::<cairo::Context>().unwrap();
                let _timestamp = values[2].get::<gst::ClockTime>().unwrap();
                let _duration = values[3].get::<gst::ClockTime>().unwrap();
                
                let start = Instant::now();
                
                // Draw bounding boxes
                if let Ok(data) = frame_data_clone.read() {
                    if let Ok(cfg) = config_clone.lock() {
                        draw_bounding_boxes(&cr, &data, &cfg);
                    }
                }
                
                // Update metrics
                let elapsed = start.elapsed().as_millis() as f64;
                if let Ok(mut m) = metrics_clone.lock() {
                    m.frames_rendered += 1;
                    m.avg_render_time_ms = 
                        (m.avg_render_time_ms * (m.frames_rendered - 1) as f64 + elapsed) 
                        / m.frames_rendered as f64;
                    if elapsed > m.peak_render_time_ms {
                        m.peak_render_time_ms = elapsed;
                    }
                }
                
                None
            });
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
        
        log::info!("Standard renderer created with {} overlay", 
                   if use_cairo { "Cairo" } else { "text" });
        
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
            let font_desc = format!("{} {}", 
                config.font_config.family,
                config.font_config.size as i32
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
        
        log::trace!("Rendering {} objects at timestamp {}", 
                   objects.len(), timestamp);
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
        let sink_pad = self.bin.static_pad("sink")
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

/// Draw bounding boxes using Cairo
fn draw_bounding_boxes(
    cr: &cairo::Context,
    frame_data: &FrameData,
    config: &RenderingConfig,
) {
    if !config.enable_bbox || frame_data.objects.is_empty() {
        return;
    }
    
    let width = frame_data.width as f64;
    let height = frame_data.height as f64;
    
    // Limit number of objects to render
    let max_objects = config.performance.max_objects_per_frame.min(frame_data.objects.len());
    
    for obj in &frame_data.objects[..max_objects] {
        let bbox = obj.bbox();
        let style = config.get_style_for_class(&obj.obj_label);
        
        // Convert normalized coordinates to pixels if needed
        let (x, y, w, h) = if bbox.left <= 1.0 && bbox.top <= 1.0 {
            // Normalized coordinates
            (
                bbox.left as f64 * width,
                bbox.top as f64 * height,
                bbox.width as f64 * width,
                bbox.height as f64 * height,
            )
        } else {
            // Already in pixels
            (
                bbox.left as f64,
                bbox.top as f64,
                bbox.width as f64,
                bbox.height as f64,
            )
        };
        
        // Set color
        let (r, g, b) = style.color.to_normalized();
        cr.set_source_rgba(r, g, b, style.alpha as f64);
        cr.set_line_width(style.thickness as f64);
        
        // Draw rectangle
        if style.corner_radius > 0.0 {
            // Rounded rectangle
            draw_rounded_rectangle(cr, x, y, w, h, style.corner_radius as f64);
        } else {
            // Regular rectangle
            cr.rectangle(x, y, w, h);
        }
        
        // Fill if needed
        if style.filled {
            let (fr, fg, fb) = style.fill_color.to_normalized();
            cr.set_source_rgba(fr, fg, fb, style.fill_alpha as f64);
            cr.fill_preserve();
            cr.set_source_rgba(r, g, b, style.alpha as f64);
        }
        
        cr.stroke();
        
        // Draw label if enabled
        if config.enable_labels {
            draw_label(cr, obj, x, y, w, h, config);
        }
    }
}

/// Draw a rounded rectangle
fn draw_rounded_rectangle(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) {
    let degrees = std::f64::consts::PI / 180.0;
    
    cr.new_sub_path();
    cr.arc(x + width - radius, y + radius, radius, -90.0 * degrees, 0.0 * degrees);
    cr.arc(x + width - radius, y + height - radius, radius, 0.0 * degrees, 90.0 * degrees);
    cr.arc(x + radius, y + height - radius, radius, 90.0 * degrees, 180.0 * degrees);
    cr.arc(x + radius, y + radius, radius, 180.0 * degrees, 270.0 * degrees);
    cr.close_path();
}

/// Draw object label
fn draw_label(
    cr: &cairo::Context,
    obj: &ObjectMeta,
    x: f64,
    y: f64,
    _w: f64,
    _h: f64,
    config: &RenderingConfig,
) {
    let mut label = obj.obj_label.clone();
    
    if config.enable_tracking_id && obj.is_tracked() {
        label = format!("{} #{}", label, obj.object_id);
    }
    
    if config.enable_confidence {
        label = format!("{} {:.1}%", label, obj.confidence * 100.0);
    }
    
    // Set font
    cr.select_font_face(
        &config.font_config.family,
        if config.font_config.italic { cairo::FontSlant::Italic } else { cairo::FontSlant::Normal },
        if config.font_config.bold { cairo::FontWeight::Bold } else { cairo::FontWeight::Normal },
    );
    cr.set_font_size(config.font_config.size as f64);
    
    // Calculate text position based on config
    let (text_x, text_y) = match config.font_config.position {
        super::config::LabelPosition::TopLeft => (x + 5.0, y - 5.0),
        super::config::LabelPosition::Above => (x + 5.0, y - 20.0),
        super::config::LabelPosition::Below => (x + 5.0, y + 20.0),
        _ => (x + 5.0, y - 5.0),
    };
    
    // Draw background if configured
    if config.font_config.background_alpha > 0.0 {
        let extents = cr.text_extents(&label);
        let (br, bg, bb) = config.font_config.background_color.to_normalized();
        cr.set_source_rgba(br, bg, bb, config.font_config.background_alpha as f64);
        cr.rectangle(
            text_x - 2.0,
            text_y - extents.height - 2.0,
            extents.width + 4.0,
            extents.height + 4.0,
        );
        cr.fill();
    }
    
    // Draw text
    let (tr, tg, tb) = config.font_config.color.to_normalized();
    cr.set_source_rgba(tr, tg, tb, 1.0);
    cr.move_to(text_x, text_y);
    cr.show_text(&label);
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