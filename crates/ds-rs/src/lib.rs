
pub mod error;
pub mod platform;
pub mod backend;
pub mod elements;
pub mod config;
pub mod pipeline;
pub mod source;
pub mod app;
pub mod metadata;
pub mod inference;
pub mod tracking;
pub mod messages;

#[cfg(target_os = "windows")]
pub mod dll_validator;

pub use error::{DeepStreamError, Result};
pub use platform::{Platform, PlatformInfo};
pub use backend::{Backend, BackendType, BackendCapabilities, BackendManager};
pub use elements::{DeepStreamElement, DeepStreamElementType, ElementBuilder};
pub use config::ApplicationConfig;
pub use pipeline::{Pipeline, PipelineBuilder, PipelineState, StateManager, BusWatcher, MessageHandler};
pub use source::{
    SourceId, SourceState, SourceInfo, SourceManager, VideoSource,
    SourceAddition, SourceRemoval, SourceEvent, SourceEventHandler,
    SourceSynchronizer, SourceController
};
pub use metadata::{
    MetadataExtractor, MetadataError, MetadataStats,
    BatchMeta, FrameMeta, ObjectMeta, BoundingBox, ClassificationMeta
};
pub use inference::{
    InferenceProcessor, DetectionResult, ClassificationResult, LabelMap,
    InferenceConfig, ModelConfig
};
pub use tracking::{
    ObjectTracker, TrackStatus, TrackerState, Trajectory, TrackingStats
};
pub use messages::{
    DSMessageHandler, DSMessageType, StreamEosTracker
};

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
    
    // Register custom elements
    register_elements()?;
    
    // Initialize logging if not already done
    let _ = log::set_logger(&SimpleLogger);
    log::set_max_level(log::LevelFilter::Info);
    
    Ok(())
}

fn register_elements() -> Result<()> {
    use gstreamer as gst;
    
    // Create a temporary plugin for registering elements
    let plugin = match gst::Plugin::load_by_name("coreelements") {
        Ok(p) => p,
        Err(_) => return Err(DeepStreamError::Configuration("Failed to load coreelements plugin".to_string()))
    };
    
    // Register CPU detector element
    backend::cpu_vision::cpudetector::register(&plugin)
        .map_err(|e| DeepStreamError::Configuration(format!("Failed to register cpudetector element: {}", e)))?;
    
    log::info!("Successfully registered custom CPU detector element");
    
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
