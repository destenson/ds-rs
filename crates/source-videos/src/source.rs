use crate::config_types::{VideoSourceConfig, VideoSourceType};
use crate::error::{Result, SourceVideoError};
use crate::pipeline::{self, PipelineFactory};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceState {
    Created,
    Playing,
    Paused,
    Stopped,
    Error(String),
}

impl std::fmt::Display for SourceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceState::Created => write!(f, "CREATED"),
            SourceState::Playing => write!(f, "PLAYING"),
            SourceState::Paused => write!(f, "PAUSED"),
            SourceState::Stopped => write!(f, "STOPPED"),
            SourceState::Error(msg) => write!(f, "ERROR: {}", msg),
        }
    }
}

pub trait VideoSource: Send + Sync {
    fn get_id(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_uri(&self) -> String;
    fn get_state(&self) -> SourceState;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
    fn get_pipeline(&self) -> Option<&gst::Pipeline>;
}

pub struct BaseVideoSource {
    id: String,
    name: String,
    config: VideoSourceConfig,
    pipeline: Option<gst::Pipeline>,
    state: Arc<Mutex<SourceState>>,
    factory: Arc<dyn PipelineFactory>,
}

impl BaseVideoSource {
    fn new(config: VideoSourceConfig, factory: Arc<dyn PipelineFactory>) -> Self {
        let id = Uuid::new_v4().to_string();
        let name = config.name.clone();

        Self {
            id,
            name,
            config,
            pipeline: None,
            state: Arc::new(Mutex::new(SourceState::Created)),
            factory,
        }
    }

    fn create_pipeline(&mut self) -> Result<()> {
        if self.pipeline.is_some() {
            return Ok(());
        }

        let pipeline = self.factory.create_pipeline(&self.config)?;
        self.pipeline = Some(pipeline);
        Ok(())
    }

    fn set_state(&self, state: SourceState) {
        if let Ok(mut s) = self.state.lock() {
            *s = state;
        }
    }
}

impl VideoSource for BaseVideoSource {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_uri(&self) -> String {
        self.config.get_uri()
    }

    fn get_state(&self) -> SourceState {
        self.state
            .lock()
            .map(|s| s.clone())
            .unwrap_or(SourceState::Error("Failed to lock state".to_string()))
    }

    fn start(&mut self) -> Result<()> {
        self.create_pipeline()?;

        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Playing).map_err(|_| {
                SourceVideoError::StateChange("Failed to set playing state".to_string())
            })?;

            self.set_state(SourceState::Playing);
            Ok(())
        } else {
            Err(SourceVideoError::pipeline("Pipeline not created"))
        }
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null).map_err(|_| {
                SourceVideoError::StateChange("Failed to set null state".to_string())
            })?;

            self.set_state(SourceState::Stopped);
        }

        self.pipeline = None;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Paused).map_err(|_| {
                SourceVideoError::StateChange("Failed to set paused state".to_string())
            })?;

            self.set_state(SourceState::Paused);
            Ok(())
        } else {
            Err(SourceVideoError::pipeline("Pipeline not created"))
        }
    }

    fn resume(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Playing).map_err(|_| {
                SourceVideoError::StateChange("Failed to resume playing".to_string())
            })?;

            self.set_state(SourceState::Playing);
            Ok(())
        } else {
            Err(SourceVideoError::pipeline("Pipeline not created"))
        }
    }

    fn get_pipeline(&self) -> Option<&gst::Pipeline> {
        self.pipeline.as_ref()
    }
}

pub struct TestPatternSource {
    base: BaseVideoSource,
}

impl TestPatternSource {
    pub fn new(config: VideoSourceConfig) -> Self {
        let factory = pipeline::TestPatternPipeline::new();
        Self {
            base: BaseVideoSource::new(config, factory),
        }
    }
}

impl VideoSource for TestPatternSource {
    fn get_id(&self) -> &str {
        self.base.get_id()
    }

    fn get_name(&self) -> &str {
        self.base.get_name()
    }

    fn get_uri(&self) -> String {
        self.base.get_uri()
    }

    fn get_state(&self) -> SourceState {
        self.base.get_state()
    }

    fn start(&mut self) -> Result<()> {
        self.base.start()
    }

    fn stop(&mut self) -> Result<()> {
        self.base.stop()
    }

    fn pause(&mut self) -> Result<()> {
        self.base.pause()
    }

    fn resume(&mut self) -> Result<()> {
        self.base.resume()
    }

    fn get_pipeline(&self) -> Option<&gst::Pipeline> {
        self.base.get_pipeline()
    }
}

pub struct FileSource {
    base: BaseVideoSource,
}

impl FileSource {
    pub fn new(config: VideoSourceConfig) -> Self {
        let factory = pipeline::FileSinkPipeline::new();
        Self {
            base: BaseVideoSource::new(config, factory),
        }
    }
}

impl VideoSource for FileSource {
    fn get_id(&self) -> &str {
        self.base.get_id()
    }

    fn get_name(&self) -> &str {
        self.base.get_name()
    }

    fn get_uri(&self) -> String {
        self.base.get_uri()
    }

    fn get_state(&self) -> SourceState {
        self.base.get_state()
    }

    fn start(&mut self) -> Result<()> {
        self.base.start()
    }

    fn stop(&mut self) -> Result<()> {
        self.base.stop()
    }

    fn pause(&mut self) -> Result<()> {
        self.base.pause()
    }

    fn resume(&mut self) -> Result<()> {
        self.base.resume()
    }

    fn get_pipeline(&self) -> Option<&gst::Pipeline> {
        self.base.get_pipeline()
    }
}

pub struct RtspSource {
    base: BaseVideoSource,
}

impl RtspSource {
    pub fn new(config: VideoSourceConfig) -> Self {
        let factory = pipeline::RtspSourcePipeline::new();
        Self {
            base: BaseVideoSource::new(config, factory),
        }
    }
}

impl VideoSource for RtspSource {
    fn get_id(&self) -> &str {
        self.base.get_id()
    }

    fn get_name(&self) -> &str {
        self.base.get_name()
    }

    fn get_uri(&self) -> String {
        self.base.get_uri()
    }

    fn get_state(&self) -> SourceState {
        self.base.get_state()
    }

    fn start(&mut self) -> Result<()> {
        self.base.start()
    }

    fn stop(&mut self) -> Result<()> {
        self.base.stop()
    }

    fn pause(&mut self) -> Result<()> {
        self.base.pause()
    }

    fn resume(&mut self) -> Result<()> {
        self.base.resume()
    }

    fn get_pipeline(&self) -> Option<&gst::Pipeline> {
        self.base.get_pipeline()
    }
}

/// A source that always returns errors, used for unexpanded directory/file list sources
struct ErrorSource {
    config: VideoSourceConfig,
    error_message: String,
    id: String,
}

impl ErrorSource {
    fn new(config: VideoSourceConfig, error_message: String) -> Self {
        let id = format!("error-{}", uuid::Uuid::new_v4());
        Self {
            config,
            error_message,
            id,
        }
    }
}

impl VideoSource for ErrorSource {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        &self.config.name
    }

    fn get_uri(&self) -> String {
        format!("error://{}", self.error_message)
    }

    fn get_state(&self) -> SourceState {
        SourceState::Error(self.error_message.clone())
    }

    fn start(&mut self) -> Result<()> {
        Err(SourceVideoError::config(&self.error_message))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(()) // Allow stop to succeed
    }

    fn pause(&mut self) -> Result<()> {
        Err(SourceVideoError::config(&self.error_message))
    }

    fn resume(&mut self) -> Result<()> {
        Err(SourceVideoError::config(&self.error_message))
    }

    fn get_pipeline(&self) -> Option<&gst::Pipeline> {
        None
    }
}

pub fn create_source(config: VideoSourceConfig) -> Box<dyn VideoSource> {
    match &config.source_type {
        VideoSourceType::TestPattern { .. } => Box::new(TestPatternSource::new(config)),
        VideoSourceType::File { .. } => Box::new(FileSource::new(config)),
        VideoSourceType::Rtsp { .. } => Box::new(RtspSource::new(config)),
        VideoSourceType::Directory { .. } => {
            // Directory sources should be expanded to individual file sources before this point
            // Return an error source instead of panicking
            eprintln!("WARNING: Directory sources should be expanded before creating video source");
            Box::new(ErrorSource::new(
                config,
                "Directory sources must be expanded to individual file sources before creation"
                    .to_string(),
            ))
        }
        VideoSourceType::FileList { .. } => {
            // FileList sources should be expanded to individual file sources before this point
            // Return an error source instead of panicking
            eprintln!("WARNING: FileList sources should be expanded before creating video source");
            Box::new(ErrorSource::new(
                config,
                "FileList sources must be expanded to individual file sources before creation"
                    .to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_creation() {
        gst::init().unwrap();

        let config = VideoSourceConfig::test_pattern("test", "smpte");
        let source = create_source(config);

        assert_eq!(source.get_name(), "test");
        assert_eq!(source.get_uri(), "videotestsrc:///test");
        assert_eq!(source.get_state(), SourceState::Created);
    }

    #[test]
    fn test_source_lifecycle() {
        gst::init().unwrap();

        let config = VideoSourceConfig::test_pattern("lifecycle-test", "ball");
        let mut source = create_source(config);

        assert_eq!(source.get_state(), SourceState::Created);

        source.start().unwrap();
        assert_eq!(source.get_state(), SourceState::Playing);

        source.pause().unwrap();
        assert_eq!(source.get_state(), SourceState::Paused);

        source.resume().unwrap();
        assert_eq!(source.get_state(), SourceState::Playing);

        source.stop().unwrap();
        assert_eq!(source.get_state(), SourceState::Stopped);
    }
}
