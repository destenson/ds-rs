pub mod app;
pub mod backend;
pub mod config;
pub mod elements;
pub mod error;
pub mod inference;
pub mod messages;
pub mod metadata;
pub mod multistream;
pub mod pipeline;
pub mod platform;
pub mod rendering;
pub mod source;
pub mod tracking;

#[cfg(target_os = "windows")]
pub mod dll_validator;

pub use backend::{Backend, BackendCapabilities, BackendManager, BackendType};
pub use config::ApplicationConfig;
pub use elements::factory::ElementFactory;
pub use elements::{DeepStreamElement, DeepStreamElementType, ElementBuilder};
pub use error::{DeepStreamError, ErrorClassification, ErrorClassifier, Result, is_retryable};
pub use inference::{
    ClassificationResult, DetectionResult, InferenceConfig, InferenceProcessor, LabelMap,
    ModelConfig,
};
pub use messages::{DSMessageHandler, DSMessageType, StreamEosTracker};
pub use metadata::{
    BatchMeta, BoundingBox, ClassificationMeta, FrameMeta, MetadataError, MetadataExtractor,
    MetadataStats, ObjectMeta,
};
pub use multistream::{
    DetectionPipeline, MetricsCollector, MultiStreamConfig, MultiStreamConfigBuilder,
    MultiStreamEvent, MultiStreamManager, MultiStreamStats, PipelinePool, ResourceLimits,
    ResourceManager, StreamCoordinator, StreamMetrics, StreamPriority,
};
pub use pipeline::{
    BusWatcher, MessageHandler, Pipeline, PipelineBuilder, PipelineState, StateManager,
};
pub use platform::{Platform, PlatformInfo};
pub use rendering::{
    BoundingBoxRenderer, MetadataBridge, PerformanceMetrics, RendererFactory, RenderingConfig,
};
pub use source::{
    CircuitBreaker,
    CircuitBreakerConfig,
    CircuitBreakerManager,
    CircuitState,
    ErrorBoundary,
    FaultTolerantSourceController,
    HealthConfig,
    HealthMonitor,
    HealthStatus,
    IsolatedSource,
    IsolationManager,
    IsolationPolicy,
    // Recovery and fault tolerance exports
    RecoveryConfig,
    RecoveryManager,
    RecoveryState,
    RecoveryStats,
    SourceAddition,
    SourceController,
    SourceEvent,
    SourceEventHandler,
    SourceHealthMonitor,
    SourceId,
    SourceInfo,
    SourceManager,
    SourceRemoval,
    SourceState,
    SourceSynchronizer,
    VideoSource,
};
pub use tracking::{ObjectTracker, TrackStatus, TrackerState, TrackingStats, Trajectory};

/// Get current timestamp in seconds since Unix epoch
/// Used for consistent timestamp formatting in log messages
#[inline]
pub fn timestamp() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

pub fn init() -> Result<()> {
    gstreamer::init().map_err(|e| DeepStreamError::GStreamer(e.into()))?;

    // Initialize logging if not already done
    let _ = log::set_logger(&SimpleLogger);
    log::set_max_level(log::LevelFilter::Info);

    Ok(())
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_platform_detection() {
        let _ = init();
        let platform = PlatformInfo::detect();
        assert!(platform.is_ok());
    }

    #[test]
    fn test_backend_manager_creation() {
        let _ = init();
        let manager = BackendManager::new();
        assert!(manager.is_ok());
    }
}
