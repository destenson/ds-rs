use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceVideoError {
    #[error("GStreamer error: {0}")]
    GStreamer(#[from] gstreamer::glib::Error),
    
    #[error("GStreamer boolean error: {0}")]
    GStreamerBool(#[from] gstreamer::glib::BoolError),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Server error: {0}")]
    Server(String),
    
    #[error("Resource error: {0}")]
    Resource(String),
    
    #[error("Pipeline error: {0}")]
    Pipeline(String),
    
    #[error("Source not found: {0}")]
    SourceNotFound(String),
    
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    
    #[error("File error: {0}")]
    File(#[from] std::io::Error),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    #[error("State change error: {0}")]
    StateChange(String),
    
    #[error("Element creation failed: {0}")]
    ElementCreation(String),
    
    #[error("Linking failed: {0} -> {1}")]
    LinkingFailed(String, String),
    
    #[error("RTSP mount point error: {0}")]
    RtspMountPoint(String),
    
    #[error("Timeout error: operation timed out after {0} seconds")]
    Timeout(u64),
}

pub type Result<T> = std::result::Result<T, SourceVideoError>;

impl SourceVideoError {
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }
    
    pub fn server<S: Into<String>>(msg: S) -> Self {
        Self::Server(msg.into())
    }
    
    pub fn resource<S: Into<String>>(msg: S) -> Self {
        Self::Resource(msg.into())
    }
    
    pub fn pipeline<S: Into<String>>(msg: S) -> Self {
        Self::Pipeline(msg.into())
    }
    
    pub fn element<S: Into<String>>(element: S) -> Self {
        Self::ElementCreation(element.into())
    }
    
    pub fn linking<S: Into<String>>(src: S, sink: S) -> Self {
        Self::LinkingFailed(src.into(), sink.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = SourceVideoError::config("invalid resolution");
        assert_eq!(err.to_string(), "Configuration error: invalid resolution");
        
        let err = SourceVideoError::linking("videotestsrc", "fakesink");
        assert_eq!(err.to_string(), "Linking failed: videotestsrc -> fakesink");
    }
}