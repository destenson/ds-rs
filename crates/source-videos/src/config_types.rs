use crate::error::{Result, SourceVideoError};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoSourceConfig {
    #[serde(default = "default_name")]
    pub name: String,

    #[serde(flatten)]
    pub source_type: VideoSourceType,

    #[serde(default = "default_resolution")]
    pub resolution: Resolution,

    #[serde(default = "default_framerate")]
    pub framerate: Framerate,

    #[serde(default = "default_format")]
    pub format: VideoFormat,

    #[serde(default)]
    pub duration: Option<u64>,

    #[serde(default)]
    pub num_buffers: Option<i32>,

    #[serde(default = "default_is_live")]
    pub is_live: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VideoSourceType {
    TestPattern {
        #[serde(default = "default_pattern")]
        pattern: String,
    },
    File {
        path: String,
        #[serde(default = "default_file_format")]
        container: FileContainer,
    },
    Rtsp {
        #[serde(default = "default_mount_point")]
        mount_point: String,
        #[serde(default = "default_rtsp_port")]
        port: u16,
    },
    Directory {
        #[serde(flatten)]
        config: DirectoryConfig,
    },
    FileList {
        #[serde(flatten)]
        config: FileListConfig,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Framerate {
    pub numerator: i32,
    pub denominator: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoFormat {
    I420,
    NV12,
    RGB,
    RGBA,
    BGRx,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileContainer {
    Mp4,
    Mkv,
    Avi,
    WebM,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub path: String,

    #[serde(default = "default_recursive")]
    pub recursive: bool,

    #[serde(default)]
    pub filters: Option<FilterConfig>,

    #[serde(default = "default_lazy_loading")]
    pub lazy_loading: bool,

    #[serde(default)]
    pub mount_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileListConfig {
    pub files: Vec<String>,

    #[serde(default)]
    pub mount_prefix: Option<String>,

    #[serde(default = "default_lazy_loading")]
    pub lazy_loading: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterConfig {
    #[serde(default)]
    pub include: Vec<String>,

    #[serde(default)]
    pub exclude: Vec<String>,

    #[serde(default)]
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchConfig {
    #[serde(default = "default_watch_enabled")]
    pub enabled: bool,

    #[serde(default = "default_auto_repeat")]
    pub auto_repeat: bool,

    #[serde(default = "default_reload_on_change")]
    pub reload_on_change: bool,

    #[serde(default = "default_debounce_duration")]
    pub debounce_duration_ms: u64,

    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    #[serde(default)]
    pub max_loops: Option<u32>,

    #[serde(default = "default_seamless_loop")]
    pub seamless_loop: bool,

    #[serde(default = "default_gap_duration")]
    pub gap_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtspServerConfig {
    #[serde(default = "default_rtsp_port")]
    pub port: u16,

    #[serde(default = "default_rtsp_address")]
    pub address: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    #[serde(default)]
    pub authentication: Option<BasicAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: RtspServerConfig,

    #[serde(default)]
    pub sources: Vec<VideoSourceConfig>,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub output_dir: Option<String>,
}

impl VideoSourceConfig {
    pub fn test_pattern(name: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source_type: VideoSourceType::TestPattern {
                pattern: pattern.into(),
            },
            resolution: default_resolution(),
            framerate: default_framerate(),
            format: default_format(),
            duration: None,
            num_buffers: None,
            is_live: true,
        }
    }

    pub fn file(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source_type: VideoSourceType::File {
                path: path.into(),
                container: default_file_format(),
            },
            resolution: default_resolution(),
            framerate: default_framerate(),
            format: default_format(),
            duration: Some(10),
            num_buffers: None,
            is_live: false,
        }
    }

    pub fn rtsp(name: impl Into<String>, mount_point: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source_type: VideoSourceType::Rtsp {
                mount_point: mount_point.into(),
                port: default_rtsp_port(),
            },
            resolution: default_resolution(),
            framerate: default_framerate(),
            format: default_format(),
            duration: None,
            num_buffers: None,
            is_live: true,
        }
    }

    pub fn get_uri(&self) -> String {
        match &self.source_type {
            VideoSourceType::TestPattern { .. } => {
                format!("videotestsrc:///{}", self.name)
            }
            VideoSourceType::File { path, .. } => {
                format!("file:///{}", path.replace('\\', "/"))
            }
            VideoSourceType::Rtsp { mount_point, port } => {
                format!("rtsp://localhost:{}/{}", port, mount_point)
            }
            VideoSourceType::Directory { config } => {
                format!("directory:///{}", config.path.replace('\\', "/"))
            }
            VideoSourceType::FileList { config } => {
                format!("filelist:///[{}]", config.files.len())
            }
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(Into::into)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| SourceVideoError::config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for RtspServerConfig {
    fn default() -> Self {
        Self {
            port: default_rtsp_port(),
            address: default_rtsp_address(),
            max_connections: default_max_connections(),
            authentication: None,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: RtspServerConfig::default(),
            sources: vec![
                VideoSourceConfig::test_pattern("test-1", "smpte"),
                VideoSourceConfig::test_pattern("test-2", "ball"),
            ],
            log_level: default_log_level(),
            output_dir: None,
        }
    }
}

impl VideoFormat {
    pub fn to_caps_string(&self) -> &str {
        match self {
            VideoFormat::I420 => "I420",
            VideoFormat::NV12 => "NV12",
            VideoFormat::RGB => "RGB",
            VideoFormat::RGBA => "RGBA",
            VideoFormat::BGRx => "BGRx",
        }
    }
}

impl FileContainer {
    pub fn muxer_name(&self) -> &str {
        match self {
            FileContainer::Mp4 => "mp4mux",
            FileContainer::Mkv => "matroskamux",
            FileContainer::Avi => "avimux",
            FileContainer::WebM => "webmmux",
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            FileContainer::Mp4 => "mp4",
            FileContainer::Mkv => "mkv",
            FileContainer::Avi => "avi",
            FileContainer::WebM => "webm",
        }
    }
}

fn default_name() -> String {
    format!("source-{}", uuid::Uuid::new_v4())
}

fn default_resolution() -> Resolution {
    Resolution {
        width: 1920,
        height: 1080,
    }
}

fn default_framerate() -> Framerate {
    Framerate {
        numerator: 30,
        denominator: 1,
    }
}

fn default_format() -> VideoFormat {
    VideoFormat::I420
}

fn default_pattern() -> String {
    "smpte".to_string()
}

fn default_file_format() -> FileContainer {
    FileContainer::Mp4
}

fn default_mount_point() -> String {
    "test".to_string()
}

fn default_rtsp_port() -> u16 {
    8554
}

fn default_rtsp_address() -> String {
    "0.0.0.0".to_string()
}

fn default_max_connections() -> u32 {
    100
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_is_live() -> bool {
    true
}

fn default_recursive() -> bool {
    false
}

fn default_lazy_loading() -> bool {
    true
}

fn default_watch_enabled() -> bool {
    false
}

fn default_auto_repeat() -> bool {
    false
}

fn default_reload_on_change() -> bool {
    true
}

fn default_debounce_duration() -> u64 {
    500
}

fn default_seamless_loop() -> bool {
    true
}

fn default_gap_duration() -> u64 {
    100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.server.port, config.server.port);
    }

    #[test]
    fn test_source_uri_generation() {
        let pattern_source = VideoSourceConfig::test_pattern("test", "smpte");
        assert_eq!(pattern_source.get_uri(), "videotestsrc:///test");

        let file_source = VideoSourceConfig::file("video", "/tmp/test.mp4");
        assert_eq!(file_source.get_uri(), "file:////tmp/test.mp4");

        let rtsp_source = VideoSourceConfig::rtsp("stream", "test1");
        assert_eq!(rtsp_source.get_uri(), "rtsp://localhost:8554/test1");
    }
}
