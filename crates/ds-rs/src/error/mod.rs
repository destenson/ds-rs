pub mod classification;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeepStreamError {
    #[error("GStreamer error: {0}")]
    GStreamer(#[from] gstreamer::glib::Error),
    
    #[error("GStreamer boolean error: {0}")]
    GStreamerBool(#[from] gstreamer::glib::BoolError),
    
    #[error("Element creation failed: {element}")]
    ElementCreation { element: String },
    
    #[error("Element not found: {element}")]
    ElementNotFound { element: String },
    
    #[error("Backend not available: {backend}")]
    BackendNotAvailable { backend: String },
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Platform detection failed: {0}")]
    PlatformDetection(String),
    
    #[error("Pipeline error: {0}")]
    Pipeline(String),
    
    #[error("Property setting failed for element {element}: {property}")]
    PropertySetting { element: String, property: String },
    
    #[error("State change failed: {0}")]
    StateChange(String),
    
    #[error("Pad linking failed: {0}")]
    PadLinking(String),
    
    #[error("Pad not found: {element}::{pad}")]
    PadNotFound { element: String, pad: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Not initialized: {0}")]
    NotInitialized(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Initialization failed: {reason}")]
    InitializationFailed { reason: String },
    
    #[error("Processing failed: {reason}")]
    ProcessingFailed { reason: String },
    
    #[error("Resource limit: {0}")]
    ResourceLimit(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, DeepStreamError>;

pub trait ResultExt<T> {
    fn context(self, msg: &str) -> Result<T>;
}

impl<T> ResultExt<T> for std::result::Result<T, DeepStreamError> {
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|e| DeepStreamError::Unknown(format!("{}: {}", msg, e)))
    }
}

impl<T> ResultExt<T> for Option<T> {
    fn context(self, msg: &str) -> Result<T> {
        self.ok_or_else(|| DeepStreamError::Unknown(msg.to_string()))
    }
}

pub use classification::{
    classify, is_retryable, ErrorCategory, ErrorClassification, ErrorClassifier,
    ErrorPersistence, ErrorSeverity, RecoveryAction,
};