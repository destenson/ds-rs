//! Inference configuration parsing and management

use super::{InferenceError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main inference configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InferenceConfig {
    /// Primary inference configuration
    pub primary_gie: Option<ModelConfig>,

    /// Secondary inference configurations
    pub secondary_gies: Vec<ModelConfig>,

    /// Global inference settings
    pub global: GlobalConfig,
}

/// Global inference settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalConfig {
    /// GPU ID to use
    pub gpu_id: u32,

    /// Enable TensorRT optimization
    pub enable_tensorrt: bool,

    /// Batch size for inference
    pub batch_size: u32,

    /// Inference interval (process every N frames)
    pub interval: u32,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            gpu_id: 0,
            enable_tensorrt: true,
            batch_size: 1,
            interval: 0,
        }
    }
}

/// Model-specific configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelConfig {
    /// Unique component ID
    pub unique_id: i32,

    /// Model name/identifier
    pub name: String,

    /// Model file paths
    pub model_paths: ModelPaths,

    /// Input configuration
    pub input: InputConfig,

    /// Output configuration
    pub output: OutputConfig,

    /// Processing parameters
    pub processing: ProcessingConfig,

    /// Custom properties
    pub properties: HashMap<String, String>,
}

/// Model file paths
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelPaths {
    /// Path to model file (ONNX, UFF, or Caffe)
    pub model_file: Option<PathBuf>,

    /// Path to proto file (for Caffe)
    pub proto_file: Option<PathBuf>,

    /// Path to TensorRT engine file
    pub engine_file: Option<PathBuf>,

    /// Path to label file
    pub label_file: Option<PathBuf>,

    /// Path to custom library
    pub custom_lib: Option<PathBuf>,
}

/// Input configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputConfig {
    /// Input width
    pub width: u32,

    /// Input height
    pub height: u32,

    /// Number of channels
    pub channels: u32,

    /// Input format (RGB, BGR, GRAY)
    pub format: String,

    /// Preprocessing parameters
    pub preprocess: PreprocessConfig,
}

/// Preprocessing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PreprocessConfig {
    /// Mean values for normalization
    pub mean: Vec<f32>,

    /// Scale factor
    pub scale_factor: f32,

    /// Offsets for each channel
    pub offsets: Vec<f32>,
}

impl Default for PreprocessConfig {
    fn default() -> Self {
        Self {
            mean: vec![0.0, 0.0, 0.0],
            scale_factor: 1.0,
            offsets: vec![0.0, 0.0, 0.0],
        }
    }
}

/// Output configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputConfig {
    /// Number of classes
    pub num_classes: u32,

    /// Output tensor names
    pub output_tensor_names: Vec<String>,

    /// Post-processing type (DBSCAN, NMS, etc.)
    pub post_process: String,

    /// Detection parameters
    pub detection: DetectionConfig,
}

/// Detection-specific configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DetectionConfig {
    /// Confidence threshold
    pub threshold: f32,

    /// NMS IoU threshold
    pub nms_iou_threshold: f32,

    /// Minimum box size
    pub min_box_size: f32,

    /// Maximum detections per frame
    pub max_detections: u32,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            nms_iou_threshold: 0.5,
            min_box_size: 10.0,
            max_detections: 100,
        }
    }
}

/// Processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessingConfig {
    /// Process on specific classes only
    pub operate_on_classes: Vec<i32>,

    /// Process on specific unique IDs
    pub operate_on_gie_ids: Vec<i32>,

    /// Enable classifier async mode
    pub classifier_async_mode: bool,

    /// Classifier threshold
    pub classifier_threshold: f32,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            operate_on_classes: Vec::new(),
            operate_on_gie_ids: Vec::new(),
            classifier_async_mode: false,
            classifier_threshold: 0.5,
        }
    }
}

impl InferenceConfig {
    /// Create default inference configuration
    pub fn default() -> Self {
        Self {
            primary_gie: None,
            secondary_gies: Vec::new(),
            global: GlobalConfig::default(),
        }
    }

    /// Load configuration from TOML file
    pub fn from_toml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| InferenceError::ConfigError(format!("Failed to read config: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| InferenceError::ConfigError(format!("Failed to parse TOML: {}", e)))
    }

    /// Parse DeepStream config file format
    pub fn from_deepstream_config<P: AsRef<Path>>(_path: P) -> Result<ModelConfig> {
        // This would parse the DeepStream .txt config format
        // For now, return a mock configuration
        Ok(ModelConfig::default_primary())
    }

    /// Save configuration to TOML file
    pub fn to_toml<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| InferenceError::ConfigError(format!("Failed to serialize: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| InferenceError::ConfigError(format!("Failed to write config: {}", e)))
    }
}

impl ModelConfig {
    /// Create default primary detector configuration
    pub fn default_primary() -> Self {
        Self {
            unique_id: 1,
            name: "primary-detector".to_string(),
            model_paths: ModelPaths {
                model_file: Some(PathBuf::from("model.onnx")),
                proto_file: None,
                engine_file: Some(PathBuf::from("model.engine")),
                label_file: Some(PathBuf::from("labels.txt")),
                custom_lib: None,
            },
            input: InputConfig {
                width: 640,
                height: 480,
                channels: 3,
                format: "RGB".to_string(),
                preprocess: PreprocessConfig::default(),
            },
            output: OutputConfig {
                num_classes: 80,
                output_tensor_names: vec!["detection_out".to_string()],
                post_process: "NMS".to_string(),
                detection: DetectionConfig::default(),
            },
            processing: ProcessingConfig::default(),
            properties: HashMap::new(),
        }
    }

    /// Create default secondary classifier configuration
    pub fn default_secondary() -> Self {
        let mut config = Self::default_primary();
        config.unique_id = 2;
        config.name = "secondary-classifier".to_string();
        config.processing.operate_on_classes = vec![0, 1]; // Operate on vehicles and persons
        config
    }

    /// Apply configuration to GStreamer element
    pub fn apply_to_element(&self, element: &gst::Element) -> Result<()> {
        // Set properties on the element
        if let Some(engine_file) = &self.model_paths.engine_file {
            element.set_property("model-engine-file", engine_file.to_str().unwrap());
        }

        element.set_property("batch-size", self.input.width);
        element.set_property("unique-id", self.unique_id);

        // Set other properties...
        for (key, value) in &self.properties {
            element.set_property_from_str(key, value);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configs() {
        let global = GlobalConfig::default();
        assert_eq!(global.gpu_id, 0);
        assert_eq!(global.batch_size, 1);

        let detection = DetectionConfig::default();
        assert_eq!(detection.threshold, 0.5);

        let model = ModelConfig::default_primary();
        assert_eq!(model.unique_id, 1);
        assert_eq!(model.input.width, 640);
    }

    #[test]
    fn test_config_serialization() {
        let config = InferenceConfig::default();
        let serialized = toml::to_string(&config);
        assert!(serialized.is_ok());
    }
}
