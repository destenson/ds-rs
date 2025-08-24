//! Rendering configuration for bounding box visualization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for rendering bounding boxes and labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    /// Enable bounding box rendering
    pub enable_bbox: bool,
    
    /// Enable text labels
    pub enable_labels: bool,
    
    /// Enable confidence scores
    pub enable_confidence: bool,
    
    /// Enable tracking IDs
    pub enable_tracking_id: bool,
    
    /// Default bounding box appearance
    pub default_bbox_style: BoundingBoxStyle,
    
    /// Class-specific styles (overrides default)
    pub class_styles: HashMap<String, BoundingBoxStyle>,
    
    /// Font configuration for labels
    pub font_config: FontConfig,
    
    /// Performance settings
    pub performance: PerformanceConfig,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        let mut class_styles = HashMap::new();
        
        // Default styles for common object classes
        class_styles.insert(
            "person".to_string(),
            BoundingBoxStyle {
                color: Color::rgb(0, 255, 0),  // Green
                thickness: 2.0,
                ..Default::default()
            },
        );
        
        class_styles.insert(
            "vehicle".to_string(),
            BoundingBoxStyle {
                color: Color::rgb(255, 0, 0),  // Red
                thickness: 2.0,
                ..Default::default()
            },
        );
        
        class_styles.insert(
            "ball".to_string(),
            BoundingBoxStyle {
                color: Color::rgb(255, 255, 0),  // Yellow
                thickness: 3.0,
                ..Default::default()
            },
        );
        
        Self {
            enable_bbox: true,
            enable_labels: true,
            enable_confidence: true,
            enable_tracking_id: false,
            default_bbox_style: BoundingBoxStyle::default(),
            class_styles,
            font_config: FontConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Style configuration for bounding boxes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBoxStyle {
    /// Border color
    pub color: Color,
    
    /// Line thickness in pixels
    pub thickness: f32,
    
    /// Transparency (0.0 = transparent, 1.0 = opaque)
    pub alpha: f32,
    
    /// Corner radius for rounded rectangles (0 = sharp corners)
    pub corner_radius: f32,
    
    /// Draw filled rectangle instead of outline
    pub filled: bool,
    
    /// Fill color (if filled is true)
    pub fill_color: Color,
    
    /// Fill transparency
    pub fill_alpha: f32,
}

impl Default for BoundingBoxStyle {
    fn default() -> Self {
        Self {
            color: Color::rgb(0, 255, 255),  // Cyan
            thickness: 2.0,
            alpha: 1.0,
            corner_radius: 0.0,
            filled: false,
            fill_color: Color::rgb(0, 0, 0),
            fill_alpha: 0.3,
        }
    }
}

/// Color representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a color from RGB values
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    /// Convert to normalized float values
    pub fn to_normalized(&self) -> (f64, f64, f64) {
        (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        )
    }
    
    /// Create from normalized float values
    pub fn from_normalized(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }
}

/// Font configuration for text rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Font family name
    pub family: String,
    
    /// Font size in points
    pub size: f32,
    
    /// Bold text
    pub bold: bool,
    
    /// Italic text
    pub italic: bool,
    
    /// Text color
    pub color: Color,
    
    /// Background color for labels
    pub background_color: Color,
    
    /// Background transparency
    pub background_alpha: f32,
    
    /// Text position relative to bounding box
    pub position: LabelPosition,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "Sans".to_string(),
            size: 12.0,
            bold: false,
            italic: false,
            color: Color::rgb(255, 255, 255),  // White
            background_color: Color::rgb(0, 0, 0),  // Black
            background_alpha: 0.5,
            position: LabelPosition::TopLeft,
        }
    }
}

/// Label position relative to bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LabelPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Center,
    Above,
    Below,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum objects to render per frame
    pub max_objects_per_frame: usize,
    
    /// Skip rendering if frame rate drops below this value
    pub min_fps_threshold: f32,
    
    /// Enable GPU acceleration where available
    pub use_gpu_acceleration: bool,
    
    /// Enable multi-threading for rendering
    pub use_multithreading: bool,
    
    /// Cache rendered elements between frames
    pub enable_caching: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_objects_per_frame: 100,
            min_fps_threshold: 15.0,
            use_gpu_acceleration: true,
            use_multithreading: true,
            enable_caching: true,
        }
    }
}

impl RenderingConfig {
    /// Create a configuration optimized for ball tracking
    pub fn for_ball_tracking() -> Self {
        let mut config = Self::default();
        
        // Customize for ball tracking
        config.enable_tracking_id = true;
        config.enable_confidence = false;
        
        // Yellow boxes for balls with thicker lines
        config.class_styles.insert(
            "ball".to_string(),
            BoundingBoxStyle {
                color: Color::rgb(255, 255, 0),
                thickness: 4.0,
                alpha: 1.0,
                corner_radius: 5.0,  // Slightly rounded for balls
                ..Default::default()
            },
        );
        
        // Smaller font for ball labels
        config.font_config.size = 10.0;
        config.font_config.position = LabelPosition::Above;
        
        config
    }
    
    /// Create a minimal configuration for performance
    pub fn minimal() -> Self {
        Self {
            enable_bbox: true,
            enable_labels: false,
            enable_confidence: false,
            enable_tracking_id: false,
            default_bbox_style: BoundingBoxStyle {
                thickness: 1.0,
                ..Default::default()
            },
            class_styles: HashMap::new(),
            font_config: FontConfig::default(),
            performance: PerformanceConfig {
                max_objects_per_frame: 50,
                use_gpu_acceleration: false,
                use_multithreading: false,
                enable_caching: false,
                ..Default::default()
            },
        }
    }
    
    /// Get style for a specific class
    pub fn get_style_for_class(&self, class_name: &str) -> &BoundingBoxStyle {
        self.class_styles.get(class_name)
            .unwrap_or(&self.default_bbox_style)
    }
}