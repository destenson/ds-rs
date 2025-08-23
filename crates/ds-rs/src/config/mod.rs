use crate::error::{DeepStreamError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    #[serde(rename = "property")]
    pub properties: HashMap<String, PropertyValue>,
    
    #[serde(rename = "primary-gie")]
    pub primary_gie: Option<GieConfig>,
    
    #[serde(rename = "secondary-gie")]
    pub secondary_gies: Option<Vec<GieConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl PropertyValue {
    pub fn as_string(&self) -> String {
        match self {
            PropertyValue::String(s) => s.clone(),
            PropertyValue::Integer(i) => i.to_string(),
            PropertyValue::Float(f) => f.to_string(),
            PropertyValue::Boolean(b) => b.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GieConfig {
    pub enable: bool,
    pub gpu_id: u32,
    pub batch_size: u32,
    
    #[serde(rename = "gie-unique-id")]
    pub unique_id: u32,
    
    #[serde(rename = "model-engine-file")]
    pub model_engine_file: Option<String>,
    
    #[serde(rename = "config-file")]
    pub config_file: Option<String>,
    
    pub interval: u32,
    pub bbox_border_color: Option<String>,
    pub bbox_bg_color: Option<String>,
    
    #[serde(rename = "nvbuf-memory-type")]
    pub nvbuf_memory_type: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerConfig {
    pub enable: bool,
    pub tracker_width: u32,
    pub tracker_height: u32,
    pub gpu_id: u32,
    
    #[serde(rename = "ll-lib-file")]
    pub ll_lib_file: String,
    
    #[serde(rename = "ll-config-file")]
    pub ll_config_file: String,
    
    #[serde(rename = "enable-batch-process")]
    pub enable_batch_process: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub pipeline: PipelineConfig,
    pub sources: Vec<SourceConfig>,
    pub sink: SinkConfig,
    pub osd: Option<OsdConfig>,
    pub tiler: Option<TilerConfig>,
    pub inference: Option<InferenceConfig>,
    pub tracker: Option<TrackerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub enable: bool,
    pub width: u32,
    pub height: u32,
    pub batch_size: u32,
    pub batched_push_timeout: u32,
    pub gpu_id: u32,
    pub live_source: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub enable: bool,
    pub uri: String,
    pub num_sources: u32,
    pub gpu_id: u32,
    pub cudadec_mem_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkConfig {
    pub enable: bool,
    pub sync: bool,
    pub source_id: u32,
    pub gpu_id: u32,
    pub nvbuf_memory_type: i32,
    pub sink_type: SinkType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SinkType {
    #[serde(rename = "egl")]
    Egl,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "fake")]
    Fake,
    #[serde(rename = "rtsp")]
    Rtsp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsdConfig {
    pub enable: bool,
    pub gpu_id: u32,
    pub border_width: u32,
    pub text_size: u32,
    pub text_color: String,
    pub text_bg_color: String,
    pub font: String,
    pub show_clock: bool,
    pub clock_x_offset: u32,
    pub clock_y_offset: u32,
    pub clock_text_size: u32,
    pub clock_color: String,
    pub nvbuf_memory_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilerConfig {
    pub enable: bool,
    pub rows: u32,
    pub columns: u32,
    pub width: u32,
    pub height: u32,
    pub gpu_id: u32,
    pub nvbuf_memory_type: i32,
}

impl ApplicationConfig {
    pub fn from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: ApplicationConfig = toml::from_str(&contents)?;
        Ok(config)
    }
    
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| DeepStreamError::Configuration(e.to_string()))?;
        fs::write(path, contents)?;
        Ok(())
    }
    
    pub fn default() -> Self {
        ApplicationConfig {
            pipeline: PipelineConfig {
                enable: true,
                width: 1920,
                height: 1080,
                batch_size: 1,
                batched_push_timeout: 40000,
                gpu_id: 0,
                live_source: true,
            },
            sources: vec![SourceConfig {
                enable: true,
                uri: String::from("file:///opt/nvidia/deepstream/deepstream/samples/streams/sample_720p.mp4"),
                num_sources: 1,
                gpu_id: 0,
                cudadec_mem_type: 0,
            }],
            sink: SinkConfig {
                enable: true,
                sync: false,
                source_id: 0,
                gpu_id: 0,
                nvbuf_memory_type: 0,
                sink_type: SinkType::Egl,
            },
            osd: Some(OsdConfig {
                enable: true,
                gpu_id: 0,
                border_width: 3,
                text_size: 15,
                text_color: String::from("1;1;1;1"),
                text_bg_color: String::from("0.3;0.3;0.3;1"),
                font: String::from("Serif"),
                show_clock: false,
                clock_x_offset: 800,
                clock_y_offset: 820,
                clock_text_size: 12,
                clock_color: String::from("1;0;0;1"),
                nvbuf_memory_type: 0,
            }),
            tiler: None,
            inference: None,
            tracker: None,
        }
    }
}

pub fn parse_deepstream_config_file(path: &Path) -> Result<HashMap<String, String>> {
    let contents = fs::read_to_string(path)?;
    let mut config = HashMap::new();
    
    let mut current_section = String::new();
    
    for line in contents.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse section headers
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].to_string();
            continue;
        }
        
        // Parse key-value pairs
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();
            
            // Store with section prefix if we're in a section
            if !current_section.is_empty() {
                config.insert(
                    format!("{}:{}", current_section, key),
                    value.to_string()
                );
            } else {
                config.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = ApplicationConfig::default();
        assert!(config.pipeline.enable);
        assert_eq!(config.pipeline.width, 1920);
        assert_eq!(config.pipeline.height, 1080);
        assert_eq!(config.sources.len(), 1);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = ApplicationConfig::default();
        let toml_str = toml::to_string(&config);
        assert!(toml_str.is_ok());
        
        let parsed: std::result::Result<ApplicationConfig, _> = toml::from_str(&toml_str.unwrap());
        assert!(parsed.is_ok());
    }
    
    #[test]
    fn test_parse_deepstream_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[property]").unwrap();
        writeln!(temp_file, "gpu-id=0").unwrap();
        writeln!(temp_file, "net-scale-factor=0.0039215697906911373").unwrap();
        writeln!(temp_file, "[class-attrs-all]").unwrap();
        writeln!(temp_file, "threshold=0.2").unwrap();
        
        let config = parse_deepstream_config_file(temp_file.path()).unwrap();
        assert_eq!(config.get("property:gpu-id"), Some(&"0".to_string()));
        assert_eq!(config.get("class-attrs-all:threshold"), Some(&"0.2".to_string()));
    }
}