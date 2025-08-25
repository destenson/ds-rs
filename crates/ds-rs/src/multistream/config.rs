//! Configuration for multi-stream processing

use super::ResourceLimits;
use super::StreamPriority;
use crate::backend::cpu_vision::detector::DetectorConfig;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Multi-stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiStreamConfig {
    /// Maximum number of concurrent streams
    pub max_concurrent_streams: usize,
    
    /// Resource limits for the system
    pub resource_limits: ResourceLimits,
    
    /// Detection configuration
    pub detector_config: DetectorConfig,
    
    /// Load balancing configuration
    pub load_balancing: LoadBalancingConfig,
    
    /// Quality control settings
    pub quality_control: QualityControlConfig,
    
    /// Recovery configuration for failed streams
    pub recovery_config: StreamRecoveryConfig,
    
    /// Metrics collection settings
    pub metrics_config: MetricsConfig,
    
    /// Number of worker threads for async processing
    pub worker_threads: usize,
    
    /// Enable debug logging
    pub debug_mode: bool,
}

impl Default for MultiStreamConfig {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 8,
            resource_limits: ResourceLimits::default(),
            detector_config: DetectorConfig::default(),
            load_balancing: LoadBalancingConfig::default(),
            quality_control: QualityControlConfig::default(),
            recovery_config: StreamRecoveryConfig::default(),
            metrics_config: MetricsConfig::default(),
            worker_threads: 4,
            debug_mode: false,
        }
    }
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Strategy for load balancing
    pub strategy: LoadBalancingStrategy,
    
    /// Rebalance interval
    pub rebalance_interval: Duration,
    
    /// Enable dynamic rebalancing
    pub dynamic_rebalancing: bool,
    
    /// Load threshold for triggering rebalancing
    pub rebalance_threshold: f32,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::RoundRobin,
            rebalance_interval: Duration::from_secs(30),
            dynamic_rebalancing: true,
            rebalance_threshold: 0.2, // 20% load difference
        }
    }
}

/// Load balancing strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LoadBalancingStrategy {
    /// Round-robin assignment
    RoundRobin,
    /// Assign to least loaded pipeline
    LeastLoaded,
    /// Priority-based assignment
    PriorityBased,
    /// Content-aware assignment
    ContentAware,
}

/// Quality control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControlConfig {
    /// Enable adaptive quality control
    pub adaptive_quality: bool,
    
    /// Minimum FPS to maintain
    pub min_fps: f32,
    
    /// Target FPS for normal operation
    pub target_fps: f32,
    
    /// Maximum FPS (for throttling)
    pub max_fps: f32,
    
    /// Quality adjustment interval
    pub adjustment_interval: Duration,
    
    /// Frame skip threshold (CPU usage %)
    pub frame_skip_threshold: f32,
    
    /// Quality reduction factor when under pressure
    pub quality_reduction_factor: f32,
}

impl Default for QualityControlConfig {
    fn default() -> Self {
        Self {
            adaptive_quality: true,
            min_fps: 10.0,
            target_fps: 30.0,
            max_fps: 60.0,
            adjustment_interval: Duration::from_secs(5),
            frame_skip_threshold: 75.0,
            quality_reduction_factor: 0.8,
        }
    }
}

/// Stream recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRecoveryConfig {
    /// Enable automatic recovery
    pub auto_recovery: bool,
    
    /// Maximum recovery attempts
    pub max_recovery_attempts: usize,
    
    /// Recovery backoff interval
    pub recovery_backoff: Duration,
    
    /// Recovery timeout
    pub recovery_timeout: Duration,
    
    /// Restart entire pipeline on critical failures
    pub restart_on_critical: bool,
}

impl Default for StreamRecoveryConfig {
    fn default() -> Self {
        Self {
            auto_recovery: true,
            max_recovery_attempts: 3,
            recovery_backoff: Duration::from_secs(5),
            recovery_timeout: Duration::from_secs(30),
            restart_on_critical: true,
        }
    }
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    
    /// Collection interval
    pub collection_interval: Duration,
    
    /// History retention period
    pub retention_period: Duration,
    
    /// Export metrics to file
    pub export_to_file: bool,
    
    /// Metrics export path
    pub export_path: Option<String>,
    
    /// Enable performance profiling
    pub profiling_enabled: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(1),
            retention_period: Duration::from_secs(3600), // 1 hour
            export_to_file: false,
            export_path: None,
            profiling_enabled: false,
        }
    }
}

/// Per-stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Stream URI
    pub uri: String,
    
    /// Stream priority
    pub priority: StreamPriority,
    
    /// Custom detector config for this stream
    pub detector_config: Option<DetectorConfig>,
    
    /// Target FPS for this stream
    pub target_fps: Option<f32>,
    
    /// Enable detection for this stream
    pub detection_enabled: bool,
    
    /// Enable rendering for this stream
    pub rendering_enabled: bool,
    
    /// Custom recovery settings
    pub recovery_override: Option<StreamRecoveryConfig>,
}

impl StreamConfig {
    pub fn new(uri: String) -> Self {
        Self {
            uri,
            priority: StreamPriority::Normal,
            detector_config: None,
            target_fps: None,
            detection_enabled: true,
            rendering_enabled: true,
            recovery_override: None,
        }
    }
    
    pub fn with_priority(mut self, priority: StreamPriority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_target_fps(mut self, fps: f32) -> Self {
        self.target_fps = Some(fps);
        self
    }
    
    pub fn with_detection(mut self, enabled: bool) -> Self {
        self.detection_enabled = enabled;
        self
    }
}

/// Builder for MultiStreamConfig
pub struct MultiStreamConfigBuilder {
    config: MultiStreamConfig,
}

impl MultiStreamConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: MultiStreamConfig::default(),
        }
    }
    
    pub fn max_streams(mut self, max: usize) -> Self {
        self.config.max_concurrent_streams = max;
        self
    }
    
    pub fn resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.config.resource_limits = limits;
        self
    }
    
    pub fn detector_config(mut self, config: DetectorConfig) -> Self {
        self.config.detector_config = config;
        self
    }
    
    pub fn load_balancing_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.config.load_balancing.strategy = strategy;
        self
    }
    
    pub fn worker_threads(mut self, threads: usize) -> Self {
        self.config.worker_threads = threads;
        self
    }
    
    pub fn debug_mode(mut self, enabled: bool) -> Self {
        self.config.debug_mode = enabled;
        self
    }
    
    pub fn build(self) -> MultiStreamConfig {
        self.config
    }
}

impl Default for MultiStreamConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}