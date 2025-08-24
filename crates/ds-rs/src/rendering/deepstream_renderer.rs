#![allow(unused)]
//! DeepStream-specific bounding box renderer using nvdsosd

use super::{BoundingBoxRenderer, PerformanceMetrics, RenderingConfig};
use crate::error::{DeepStreamError, Result};
use crate::metadata::object::ObjectMeta;
use crate::rendering::metadata_bridge::MetadataBridge;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// DeepStream renderer using nvdsosd for GPU-accelerated rendering
pub struct DeepStreamRenderer {
    element: gst::Element,
    metrics: Arc<Mutex<PerformanceMetrics>>,
    config: Arc<Mutex<RenderingConfig>>,
    metadata_bridge: Option<Arc<Mutex<MetadataBridge>>>,
}

impl DeepStreamRenderer {
    /// Create a new DeepStream renderer
    pub fn new(name: Option<&str>) -> Result<Self> {
        // Create nvdsosd element
        let element = gst::ElementFactory::make("nvdsosd")
            .name(name.unwrap_or("deepstream-renderer"))
            .property("process-mode", 0i32)  // GPU_MODE
            .property("display-text", 1i32)
            .property("display-bbox", 1i32)
            .property("display-mask", 0i32)
            .build()
            .map_err(|_| DeepStreamError::ElementCreation {
                element: "nvdsosd".to_string(),
            })?;
        
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
        let config = Arc::new(Mutex::new(RenderingConfig::default()));
        
        // Set up probe on sink pad to intercept metadata
        let sink_pad = element.static_pad("sink")
            .ok_or_else(|| DeepStreamError::PadNotFound {
                element: "nvdsosd".to_string(),
                pad: "sink".to_string(),
            })?;
        
        let metrics_clone = metrics.clone();
        let config_clone = config.clone();
        
        sink_pad.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
            if let Some(buffer) = info.buffer() {
                let start = Instant::now();
                
                // Process metadata on the buffer
                if let Err(e) = process_buffer_metadata(buffer, &config_clone, &metrics_clone) {
                    log::error!("Failed to process buffer metadata: {}", e);
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
            }
            gst::PadProbeReturn::Ok
        });
        
        log::info!("DeepStream renderer created with nvdsosd");
        
        Ok(Self {
            element,
            metrics,
            config,
            metadata_bridge: None,
        })
    }
}

impl BoundingBoxRenderer for DeepStreamRenderer {
    fn initialize(&mut self, config: &RenderingConfig) -> Result<()> {
        *self.config.lock().unwrap() = config.clone();
        
        // Configure nvdsosd properties based on config
        self.element.set_property("display-text", config.enable_labels as i32);
        self.element.set_property("display-bbox", config.enable_bbox as i32);
        
        // Set font if text is enabled
        if config.enable_labels {
            let font_desc = format!("{} {}", 
                config.font_config.family,
                config.font_config.size as i32
            );
            self.element.set_property("font-desc", &font_desc);
        }
        
        log::debug!("DeepStream renderer initialized with config");
        Ok(())
    }
    
    fn render_frame(&mut self, objects: &[ObjectMeta], timestamp: gst::ClockTime) -> Result<()> {
        // In DeepStream, rendering happens through metadata attached to buffers
        // This method would typically be called from a probe or metadata extractor
        
        if let Some(ref bridge) = self.metadata_bridge {
            bridge.lock().unwrap().update_objects(objects.to_vec(), timestamp);
        }
        
        // Update object count in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.objects_rendered += objects.len() as u64;
        }
        
        log::trace!("Queued {} objects for rendering at timestamp {}", 
                   objects.len(), timestamp);
        Ok(())
    }
    
    fn update_config(&mut self, config: &RenderingConfig) -> Result<()> {
        self.initialize(config)
    }
    
    fn get_element(&self) -> &gst::Element {
        &self.element
    }
    
    fn connect_metadata_source(&mut self, bridge: Arc<Mutex<MetadataBridge>>) -> Result<()> {
        self.metadata_bridge = Some(bridge.clone());
        
        // Set up src pad probe to inject metadata
        let src_pad = self.element.static_pad("src")
            .ok_or_else(|| DeepStreamError::PadNotFound {
                element: "nvdsosd".to_string(),
                pad: "src".to_string(),
            })?;
        
        let config_clone = self.config.clone();
        
        src_pad.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
            if let Some(buffer) = info.buffer_mut() {
                // Get current objects from bridge
                if let Ok(bridge_guard) = bridge.lock() {
                    if let Some((objects, _timestamp)) = bridge_guard.get_current_objects() {
                        // Inject DeepStream metadata
                        if let Err(e) = inject_deepstream_metadata(buffer, &objects, &config_clone) {
                            log::error!("Failed to inject metadata: {}", e);
                        }
                    }
                }
            }
            gst::PadProbeReturn::Ok
        });
        
        log::info!("DeepStream renderer connected to metadata source");
        Ok(())
    }
    
    fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    fn clear(&mut self) -> Result<()> {
        if let Some(ref bridge) = self.metadata_bridge {
            bridge.lock().unwrap().clear();
        }
        log::trace!("DeepStream renderer cleared");
        Ok(())
    }
}

/// Process metadata on a buffer for rendering
fn process_buffer_metadata(
    buffer: &gst::Buffer,
    config: &Arc<Mutex<RenderingConfig>>,
    metrics: &Arc<Mutex<PerformanceMetrics>>,
) -> Result<()> {
    // In a real DeepStream implementation, we would:
    // 1. Extract NvDsBatchMeta from the buffer
    // 2. Iterate through frame metadata
    // 3. Update display metadata for each object
    // 4. Apply rendering configuration
    
    // For now, this is a placeholder that logs the operation
    log::trace!("Processing buffer metadata for DeepStream rendering");
    
    // TODO: Implement actual DeepStream metadata processing
    // This requires DeepStream SDK FFI bindings
    
    Ok(())
}

/// Inject DeepStream metadata into a buffer
fn inject_deepstream_metadata(
    buffer: &mut gst::Buffer,
    objects: &[ObjectMeta],
    config: &Arc<Mutex<RenderingConfig>>,
) -> Result<()> {
    let config_guard = config.lock().unwrap();
    
    // In a real implementation, we would:
    // 1. Get or create NvDsBatchMeta
    // 2. Add NvDsFrameMeta for the frame
    // 3. Add NvDsObjectMeta for each detection
    // 4. Set display properties (colors, thickness, etc.)
    
    log::trace!("Injecting {} objects into DeepStream metadata", objects.len());
    
    for (i, obj) in objects.iter().enumerate() {
        let bbox = obj.bbox();
        let style = config_guard.get_style_for_class(&obj.obj_label);
        
        log::trace!(
            "Object {}: {} at ({:.1}, {:.1}) {}x{} - color: {:?}",
            i, obj.obj_label, bbox.left, bbox.top, bbox.width, bbox.height,
            style.color
        );
        
        // TODO: Create and attach actual NvDsObjectMeta
        // This requires DeepStream SDK FFI bindings
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deepstream_renderer_creation() {
        gst::init().unwrap();
        
        // This will fail if nvdsosd is not available (non-DeepStream systems)
        match DeepStreamRenderer::new(Some("test-ds-renderer")) {
            Ok(renderer) => {
                assert_eq!(renderer.element.name(), "test-ds-renderer");
            }
            Err(e) => {
                // Expected on non-DeepStream systems
                log::info!("DeepStream renderer not available: {}", e);
            }
        }
    }
}
