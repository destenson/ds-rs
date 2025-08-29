#![allow(unused)]
//! Real-time bounding box rendering and visualization module
//!
//! This module provides cross-backend rendering capabilities for displaying
//! detection results as bounding boxes overlaid on video streams.

use crate::backend::BackendType;
use crate::error::{DeepStreamError, Result};
use crate::metadata::object::{BoundingBox, ObjectMeta};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};

pub mod config;
pub mod deepstream_renderer;
pub mod metadata_bridge;
pub mod standard_renderer;

pub use config::RenderingConfig;
pub use metadata_bridge::MetadataBridge;

/// Trait for cross-backend bounding box rendering
pub trait BoundingBoxRenderer: Send + Sync {
    /// Initialize the renderer with the given configuration
    fn initialize(&mut self, config: &RenderingConfig) -> Result<()>;

    /// Render bounding boxes for a single frame
    fn render_frame(&mut self, objects: &[ObjectMeta], timestamp: gst::ClockTime) -> Result<()>;

    /// Update rendering configuration at runtime
    fn update_config(&mut self, config: &RenderingConfig) -> Result<()>;

    /// Get the GStreamer element for this renderer
    fn get_element(&self) -> &gst::Element;

    /// Connect metadata source to renderer
    fn connect_metadata_source(&mut self, bridge: Arc<Mutex<MetadataBridge>>) -> Result<()>;

    /// Get performance metrics
    fn get_performance_metrics(&self) -> PerformanceMetrics;

    /// Clear all rendered overlays
    fn clear(&mut self) -> Result<()>;
}

/// Performance metrics for rendering
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Average render time per frame in milliseconds
    pub avg_render_time_ms: f64,
    /// Peak render time in milliseconds
    pub peak_render_time_ms: f64,
    /// Number of frames rendered
    pub frames_rendered: u64,
    /// Number of objects rendered
    pub objects_rendered: u64,
    /// Frames dropped due to performance
    pub frames_dropped: u64,
}

/// Factory for creating backend-specific renderers
pub struct RendererFactory;

impl RendererFactory {
    /// Create a renderer for the specified backend
    pub fn create_renderer(
        backend: BackendType,
        name: Option<&str>,
    ) -> Result<Box<dyn BoundingBoxRenderer>> {
        match backend {
            BackendType::DeepStream => {
                log::info!("Creating DeepStream bounding box renderer");
                Ok(Box::new(deepstream_renderer::DeepStreamRenderer::new(
                    name,
                )?))
            }
            BackendType::Standard => {
                log::info!("Creating Standard backend bounding box renderer");
                Ok(Box::new(standard_renderer::StandardRenderer::new(name)?))
            }
            BackendType::Mock => {
                log::info!("Creating Mock bounding box renderer");
                Ok(Box::new(MockRenderer::new(name)?))
            }
        }
    }

    /// Create a renderer with custom configuration
    pub fn create_renderer_with_config(
        backend: BackendType,
        name: Option<&str>,
        config: RenderingConfig,
    ) -> Result<Box<dyn BoundingBoxRenderer>> {
        let mut renderer = Self::create_renderer(backend, name)?;
        renderer.initialize(&config)?;
        Ok(renderer)
    }
}

/// Mock renderer for testing
struct MockRenderer {
    element: gst::Element,
    metrics: PerformanceMetrics,
    config: RenderingConfig,
}

impl MockRenderer {
    fn new(name: Option<&str>) -> Result<Self> {
        let element = gst::ElementFactory::make("identity")
            .name(name.unwrap_or("mock-renderer"))
            .build()
            .map_err(|_| DeepStreamError::ElementCreation {
                element: "identity".to_string(),
            })?;

        Ok(Self {
            element,
            metrics: PerformanceMetrics::default(),
            config: RenderingConfig::default(),
        })
    }
}

impl BoundingBoxRenderer for MockRenderer {
    fn initialize(&mut self, config: &RenderingConfig) -> Result<()> {
        self.config = config.clone();
        log::debug!("Mock renderer initialized with config: {:?}", config);
        Ok(())
    }

    fn render_frame(&mut self, objects: &[ObjectMeta], _timestamp: gst::ClockTime) -> Result<()> {
        self.metrics.frames_rendered += 1;
        self.metrics.objects_rendered += objects.len() as u64;
        log::trace!("Mock rendering {} objects", objects.len());
        Ok(())
    }

    fn update_config(&mut self, config: &RenderingConfig) -> Result<()> {
        self.config = config.clone();
        Ok(())
    }

    fn get_element(&self) -> &gst::Element {
        &self.element
    }

    fn connect_metadata_source(&mut self, _bridge: Arc<Mutex<MetadataBridge>>) -> Result<()> {
        log::debug!("Mock renderer connected to metadata source");
        Ok(())
    }

    fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.metrics.clone()
    }

    fn clear(&mut self) -> Result<()> {
        log::trace!("Mock renderer cleared");
        Ok(())
    }
}

/// Rendering utilities
pub mod utils {
    use super::*;

    /// Convert normalized coordinates to pixel coordinates
    pub fn normalize_to_pixels(bbox: &BoundingBox, width: u32, height: u32) -> BoundingBox {
        BoundingBox {
            left: bbox.left * width as f32,
            top: bbox.top * height as f32,
            width: bbox.width * width as f32,
            height: bbox.height * height as f32,
        }
    }

    /// Convert pixel coordinates to normalized coordinates
    pub fn pixels_to_normalized(bbox: &BoundingBox, width: u32, height: u32) -> BoundingBox {
        BoundingBox {
            left: bbox.left / width as f32,
            top: bbox.top / height as f32,
            width: bbox.width / width as f32,
            height: bbox.height / height as f32,
        }
    }

    /// Clamp bounding box to frame boundaries
    pub fn clamp_to_frame(bbox: &BoundingBox, width: u32, height: u32) -> BoundingBox {
        let left = bbox.left.max(0.0).min(width as f32);
        let top = bbox.top.max(0.0).min(height as f32);
        let right = (bbox.left + bbox.width).min(width as f32);
        let bottom = (bbox.top + bbox.height).min(height as f32);

        BoundingBox {
            left,
            top,
            width: (right - left).max(0.0),
            height: (bottom - top).max(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_renderer_creation() {
        gst::init().unwrap();

        let renderer = MockRenderer::new(Some("test-mock")).unwrap();
        assert_eq!(renderer.element.name(), "test-mock");
    }

    #[test]
    fn test_renderer_factory() {
        gst::init().unwrap();

        let renderer =
            RendererFactory::create_renderer(BackendType::Mock, Some("factory-test")).unwrap();

        assert!(renderer.get_element().name() == "factory-test");
    }

    #[test]
    fn test_coordinate_conversion() {
        let normalized = BoundingBox::new(0.5, 0.5, 0.25, 0.25);
        let pixels = utils::normalize_to_pixels(&normalized, 1920, 1080);

        assert_eq!(pixels.left, 960.0);
        assert_eq!(pixels.top, 540.0);
        assert_eq!(pixels.width, 480.0);
        assert_eq!(pixels.height, 270.0);

        let back = utils::pixels_to_normalized(&pixels, 1920, 1080);
        assert!((back.left - normalized.left).abs() < 0.001);
    }

    #[test]
    fn test_bbox_clamping() {
        let bbox = BoundingBox::new(-10.0, -10.0, 2000.0, 1200.0);
        let clamped = utils::clamp_to_frame(&bbox, 1920, 1080);

        assert_eq!(clamped.left, 0.0);
        assert_eq!(clamped.top, 0.0);
        assert_eq!(clamped.width, 1920.0);
        assert_eq!(clamped.height, 1080.0);
    }
}
