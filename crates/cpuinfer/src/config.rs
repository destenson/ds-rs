#![allow(unused)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration structure compatible with nvinfer config files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferConfig {
    // [property] section
    pub onnx_file: Option<String>,
    pub model_engine_file: Option<String>,
    pub labelfile_path: Option<String>,
    pub batch_size: u32,
    pub process_mode: u32,
    pub num_detected_classes: u32,
    pub interval: u32,
    pub unique_id: u32,
    pub network_mode: u32,
    pub cluster_mode: u32,
    pub maintain_aspect_ratio: u32,
    pub symmetric_padding: u32,
    pub gpu_id: u32,

    // [class-attrs-all] section
    pub pre_cluster_threshold: f32,
    pub nms_iou_threshold: f32,
    pub topk: u32,
}

impl Default for InferConfig {
    fn default() -> Self {
        InferConfig {
            onnx_file: None,
            model_engine_file: None,
            labelfile_path: None,
            batch_size: 1,
            process_mode: 1,
            num_detected_classes: 80,
            interval: 0,
            unique_id: 0,
            network_mode: 0, // FP32
            cluster_mode: 2, // NMS
            maintain_aspect_ratio: 1,
            symmetric_padding: 1,
            gpu_id: 0,
            pre_cluster_threshold: 0.4,
            nms_iou_threshold: 0.5,
            topk: 300,
        }
    }
}

/// Parse a nvinfer-style configuration file
pub fn parse_config_file(path: &str) -> Result<InferConfig, String> {
    if !Path::new(path).exists() {
        return Err(format!("Configuration file not found: {}", path));
    }

    let contents =
        fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

    parse_config_string(&contents)
}

/// Parse configuration from a string
pub fn parse_config_string(contents: &str) -> Result<InferConfig, String> {
    let mut config = InferConfig::default();
    let mut current_section = String::new();

    for line in contents.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Check for section headers
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_lowercase();
            continue;
        }

        // Parse key-value pairs
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            match current_section.as_str() {
                "property" => {
                    match key {
                        "onnx-file" => config.onnx_file = Some(value.to_string()),
                        "model-engine-file" => config.model_engine_file = Some(value.to_string()),
                        "labelfile-path" => config.labelfile_path = Some(value.to_string()),
                        "batch-size" => config.batch_size = value.parse().unwrap_or(1),
                        "process-mode" => config.process_mode = value.parse().unwrap_or(1),
                        "num-detected-classes" => {
                            config.num_detected_classes = value.parse().unwrap_or(80)
                        }
                        "interval" => config.interval = value.parse().unwrap_or(0),
                        "unique-id" => config.unique_id = value.parse().unwrap_or(0),
                        "network-mode" => config.network_mode = value.parse().unwrap_or(0),
                        "cluster-mode" => config.cluster_mode = value.parse().unwrap_or(2),
                        "maintain-aspect-ratio" => {
                            config.maintain_aspect_ratio = value.parse().unwrap_or(1)
                        }
                        "symmetric-padding" => {
                            config.symmetric_padding = value.parse().unwrap_or(1)
                        }
                        "gpu-id" => config.gpu_id = value.parse().unwrap_or(0),
                        _ => {} // Ignore unknown properties
                    }
                }
                "class-attrs-all" => {
                    match key {
                        "pre-cluster-threshold" => {
                            config.pre_cluster_threshold = value.parse().unwrap_or(0.4)
                        }
                        "nms-iou-threshold" => {
                            config.nms_iou_threshold = value.parse().unwrap_or(0.5)
                        }
                        "topk" => config.topk = value.parse().unwrap_or(300),
                        _ => {} // Ignore unknown properties
                    }
                }
                _ => {} // Ignore unknown sections
            }
        }
    }

    // Validate the configuration
    validate_config(&config)?;

    Ok(config)
}

/// Validate configuration values
pub fn validate_config(config: &InferConfig) -> Result<(), String> {
    // Must have either ONNX file or model engine file
    if config.onnx_file.is_none() && config.model_engine_file.is_none() {
        return Err("Configuration must specify either onnx-file or model-engine-file".to_string());
    }

    // Check batch size
    if config.batch_size == 0 || config.batch_size > 32 {
        return Err(format!(
            "Invalid batch-size: {} (must be 1-32)",
            config.batch_size
        ));
    }

    // Check process mode
    if config.process_mode != 1 && config.process_mode != 2 {
        return Err(format!(
            "Invalid process-mode: {} (must be 1 or 2)",
            config.process_mode
        ));
    }

    // Check thresholds
    if config.pre_cluster_threshold < 0.0 || config.pre_cluster_threshold > 1.0 {
        return Err(format!(
            "Invalid pre-cluster-threshold: {} (must be 0.0-1.0)",
            config.pre_cluster_threshold
        ));
    }

    if config.nms_iou_threshold < 0.0 || config.nms_iou_threshold > 1.0 {
        return Err(format!(
            "Invalid nms-iou-threshold: {} (must be 0.0-1.0)",
            config.nms_iou_threshold
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_string() {
        let config_str = r#"
[property]
onnx-file=model.onnx
batch-size=4
num-detected-classes=80
process-mode=1

[class-attrs-all]
pre-cluster-threshold=0.4
nms-iou-threshold=0.5
topk=200
"#;

        let config = parse_config_string(config_str).unwrap();
        assert_eq!(config.onnx_file, Some("model.onnx".to_string()));
        assert_eq!(config.batch_size, 4);
        assert_eq!(config.num_detected_classes, 80);
        assert_eq!(config.process_mode, 1);
        assert_eq!(config.pre_cluster_threshold, 0.4);
        assert_eq!(config.nms_iou_threshold, 0.5);
        assert_eq!(config.topk, 200);
    }

    #[test]
    fn test_validate_config() {
        let mut config = InferConfig::default();

        // Should fail without model file
        assert!(validate_config(&config).is_err());

        // Should pass with ONNX file
        config.onnx_file = Some("model.onnx".to_string());
        assert!(validate_config(&config).is_ok());

        // Should fail with invalid batch size
        config.batch_size = 0;
        assert!(validate_config(&config).is_err());

        config.batch_size = 33;
        assert!(validate_config(&config).is_err());

        // Should fail with invalid process mode
        config.batch_size = 1;
        config.process_mode = 3;
        assert!(validate_config(&config).is_err());
    }
}
