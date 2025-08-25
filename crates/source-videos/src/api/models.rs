use serde::{Deserialize, Serialize};
use crate::{VideoSourceConfig, VideoSourceType, SourceInfo, SourceState, TestPattern};
use crate::config_types::{Resolution, Framerate, VideoFormat, FileContainer};
use std::collections::HashMap;

// Source Management Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSourceRequest {
    pub name: String,
    #[serde(flatten)]
    pub source_type: SourceTypeRequest,
    #[serde(default)]
    pub resolution: Option<Resolution>,
    #[serde(default)]
    pub framerate: Option<Framerate>,
    #[serde(default)]
    pub format: Option<VideoFormat>,
    #[serde(default)]
    pub duration: Option<u64>,
    #[serde(default)]
    pub is_live: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceTypeRequest {
    TestPattern { pattern: String },
    File { path: String, container: Option<FileContainer> },
    Rtsp { mount_point: String, port: Option<u16> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceResponse {
    pub id: String,
    pub name: String,
    pub uri: String,
    pub state: String,
    pub source_type: String,
    pub created_at: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl From<SourceInfo> for SourceResponse {
    fn from(info: SourceInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            uri: info.uri,
            state: format!("{:?}", info.state),
            source_type: "unknown".to_string(),
            created_at: None,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSourceRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub resolution: Option<Resolution>,
    #[serde(default)]
    pub framerate: Option<Framerate>,
    #[serde(default)]
    pub format: Option<VideoFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub operation: BatchOperationType,
    pub source: AddSourceRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchOperationType {
    Add,
    Remove,
    Update,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationRequest {
    pub operations: Vec<BatchOperation>,
    #[serde(default)]
    pub atomic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResponse {
    pub results: Vec<BatchResult>,
    pub success_count: usize,
    pub failure_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub operation: String,
    pub source_name: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

// Server Control Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartServerRequest {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default)]
    pub sources: Vec<AddSourceRequest>,
    #[serde(default)]
    pub network_profile: Option<String>,
}

fn default_port() -> u16 { 8554 }
fn default_address() -> String { "0.0.0.0".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub running: bool,
    pub port: Option<u16>,
    pub address: Option<String>,
    pub source_count: usize,
    pub uptime_seconds: Option<u64>,
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfoResponse {
    pub version: String,
    pub capabilities: Vec<String>,
    pub supported_formats: Vec<String>,
    pub max_sources: Option<usize>,
}

// Configuration Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub server: ApiServerConfig,
    pub sources: Vec<VideoSourceConfig>,
    pub network: Option<NetworkConfig>,
    pub watch: Option<WatchConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServerConfig {
    pub port: u16,
    pub address: String,
    pub max_connections: Option<usize>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub profile: Option<String>,
    pub packet_loss: Option<f32>,
    pub latency_ms: Option<u32>,
    pub bandwidth_kbps: Option<u32>,
    pub jitter_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    pub enabled: bool,
    pub directories: Vec<String>,
    pub recursive: bool,
    pub debounce_ms: u64,
    pub auto_reload: bool,
}

// Network Simulation Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProfileResponse {
    pub name: String,
    pub description: String,
    pub packet_loss: f32,
    pub latency_ms: u32,
    pub bandwidth_kbps: u32,
    pub jitter_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyNetworkProfileRequest {
    pub profile: String,
    #[serde(default)]
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNetworkConditionsRequest {
    pub packet_loss: Option<f32>,
    pub latency_ms: Option<u32>,
    pub bandwidth_kbps: Option<u32>,
    pub jitter_ms: Option<u32>,
    #[serde(default)]
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatusResponse {
    pub active: bool,
    pub profile: Option<String>,
    pub conditions: NetworkConditionsResponse,
    pub per_source: Option<HashMap<String, NetworkConditionsResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConditionsResponse {
    pub packet_loss: f32,
    pub latency_ms: u32,
    pub bandwidth_kbps: u32,
    pub jitter_ms: u32,
    pub connection_dropped: bool,
}

// Operations Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateVideoRequest {
    pub pattern: String,
    pub duration: u64,
    pub output: String,
    #[serde(default = "default_resolution")]
    pub resolution: Resolution,
    #[serde(default = "default_framerate")]
    pub framerate: Framerate,
}

fn default_resolution() -> Resolution {
    Resolution { width: 1920, height: 1080 }
}

fn default_framerate() -> Framerate {
    Framerate { numerator: 30, denominator: 1 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanDirectoryRequest {
    pub path: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub add_to_server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanDirectoryResponse {
    pub found_count: usize,
    pub added_count: usize,
    pub sources: Vec<SourceResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternResponse {
    pub name: String,
    pub description: String,
    pub animated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWatchingRequest {
    pub directory: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub auto_reload: bool,
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,
}

fn default_debounce() -> u64 { 500 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchStatusResponse {
    pub active: bool,
    pub watchers: Vec<WatcherInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherInfo {
    pub id: String,
    pub directory: String,
    pub recursive: bool,
    pub events_received: u64,
}

// Health Check Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessResponse {
    pub alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub components: HashMap<String, ComponentStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub healthy: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub source_count: u64,
    pub active_connections: u64,
    pub total_requests: u64,
    pub error_count: u64,
    pub uptime_seconds: u64,
    pub cpu_usage: Option<f32>,
    pub memory_usage_mb: Option<u64>,
}

// Generic Response Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    pub status: u16,
}