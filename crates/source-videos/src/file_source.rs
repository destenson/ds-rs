use crate::config_types::VideoSourceConfig;
use crate::error::{Result, SourceVideoError};
use crate::source::{VideoSource, SourceState};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::glib;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct FileVideoSource {
    id: String,
    name: String,
    file_path: String,
    pipeline: Option<gst::Pipeline>,
    state: Arc<Mutex<SourceState>>,
    mount_point: Option<String>,
    loop_playback: bool,
}

impl FileVideoSource {
    pub fn new(file_path: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            file_path: file_path.into(),
            pipeline: None,
            state: Arc::new(Mutex::new(SourceState::Created)),
            mount_point: None,
            loop_playback: false,
        }
    }
    
    pub fn from_config(config: &VideoSourceConfig) -> Result<Self> {
        if let crate::config_types::VideoSourceType::File { path, .. } = &config.source_type {
            let mut source = Self::new(path.clone(), config.name.clone());
            source.loop_playback = config.duration.is_none(); // Loop if no duration specified
            Ok(source)
        } else {
            Err(SourceVideoError::config("Not a file source config"))
        }
    }
    
    pub fn set_mount_point(&mut self, mount_point: String) {
        self.mount_point = Some(mount_point);
    }
    
    pub fn set_loop_playback(&mut self, loop_playback: bool) {
        self.loop_playback = loop_playback;
    }
    
    fn create_pipeline(&mut self) -> Result<()> {
        if self.pipeline.is_some() {
            return Ok(());
        }
        
        let path = Path::new(&self.file_path);
        if !path.exists() {
            return Err(SourceVideoError::FileNotFound(self.file_path.clone()));
        }
        
        let pipeline_name = format!("file-source-{}", self.id);
        let pipeline = gst::Pipeline::with_name(&pipeline_name);
        
        // Create elements
        let filesrc = gst::ElementFactory::make("filesrc")
            .name(&format!("{}-filesrc", self.name))
            .property("location", &self.file_path)
            .build()
            .map_err(|_| SourceVideoError::pipeline("Failed to create filesrc element"))?;
        
        // Use uridecodebin for automatic format detection and decoding
        let uri = format!("file:///{}", self.file_path.replace('\\', "/"));
        let decodebin = gst::ElementFactory::make("uridecodebin")
            .name(&format!("{}-decodebin", self.name))
            .property("uri", &uri)
            .build()
            .map_err(|_| SourceVideoError::pipeline("Failed to create uridecodebin element"))?;
        
        // Create video and audio sinks
        let video_convert = gst::ElementFactory::make("videoconvert")
            .name(&format!("{}-videoconvert", self.name))
            .build()
            .map_err(|_| SourceVideoError::pipeline("Failed to create videoconvert element"))?;
        
        let video_sink = gst::ElementFactory::make("autovideosink")
            .name(&format!("{}-videosink", self.name))
            .build()
            .map_err(|_| SourceVideoError::pipeline("Failed to create autovideosink element"))?;
        
        let audio_convert = gst::ElementFactory::make("audioconvert")
            .name(&format!("{}-audioconvert", self.name))
            .build()
            .map_err(|_| SourceVideoError::pipeline("Failed to create audioconvert element"))?;
        
        let audio_sink = gst::ElementFactory::make("autoaudiosink")
            .name(&format!("{}-audiosink", self.name))
            .build()
            .map_err(|_| SourceVideoError::pipeline("Failed to create autoaudiosink element"))?;
        
        // Add elements to pipeline
        pipeline.add_many([
            &decodebin,
            &video_convert,
            &video_sink,
            &audio_convert,
            &audio_sink,
        ])
        .map_err(|_| SourceVideoError::pipeline("Failed to add elements to pipeline"))?;
        
        // Link static elements
        video_convert.link(&video_sink)
            .map_err(|_| SourceVideoError::pipeline("Failed to link video elements"))?;
        
        audio_convert.link(&audio_sink)
            .map_err(|_| SourceVideoError::pipeline("Failed to link audio elements"))?;
        
        // Connect pad-added signal for dynamic linking
        let video_convert_weak = video_convert.downgrade();
        let audio_convert_weak = audio_convert.downgrade();
        let pipeline_weak = pipeline.downgrade();
        
        decodebin.connect_pad_added(move |_src, src_pad| {
            let pipeline = match pipeline_weak.upgrade() {
                Some(p) => p,
                None => return,
            };
            
            let pad_caps = src_pad.current_caps().unwrap_or_else(|| src_pad.query_caps(None));
            let pad_struct = pad_caps.structure(0).unwrap();
            let pad_name = pad_struct.name();
            
            if pad_name.starts_with("video/") {
                if let Some(video_convert) = video_convert_weak.upgrade() {
                    let sink_pad = video_convert.static_pad("sink").unwrap();
                    if !sink_pad.is_linked() {
                        if let Err(e) = src_pad.link(&sink_pad) {
                            log::error!("Failed to link video pad: {:?}", e);
                        } else {
                            log::info!("Linked video pad for {}", pad_name);
                        }
                    }
                }
            } else if pad_name.starts_with("audio/") {
                if let Some(audio_convert) = audio_convert_weak.upgrade() {
                    let sink_pad = audio_convert.static_pad("sink").unwrap();
                    if !sink_pad.is_linked() {
                        if let Err(e) = src_pad.link(&sink_pad) {
                            log::error!("Failed to link audio pad: {:?}", e);
                        } else {
                            log::info!("Linked audio pad for {}", pad_name);
                        }
                    }
                }
            }
        });
        
        // Handle loop playback if enabled
        if self.loop_playback {
            let pipeline_weak = pipeline.downgrade();
            let bus = pipeline.bus().unwrap();
            
            bus.add_watch_local(move |_, msg| {
                use gst::MessageView;
                
                match msg.view() {
                    MessageView::Eos(_) => {
                        if let Some(pipeline) = pipeline_weak.upgrade() {
                            log::info!("Restarting playback for loop");
                            let _ = pipeline.seek_simple(
                                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                                gst::ClockTime::ZERO,
                            );
                        }
                    }
                    _ => {}
                }
                
                glib::ControlFlow::Continue
            })
            .map_err(|_| SourceVideoError::pipeline("Failed to add bus watch"))?;
        }
        
        self.pipeline = Some(pipeline);
        Ok(())
    }
    
    fn set_state(&self, state: SourceState) {
        if let Ok(mut s) = self.state.lock() {
            *s = state;
        }
    }
}

impl VideoSource for FileVideoSource {
    fn get_id(&self) -> &str {
        &self.id
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn get_uri(&self) -> String {
        if let Some(mount) = &self.mount_point {
            format!("rtsp://localhost:8554/{}", mount)
        } else {
            format!("file:///{}", self.file_path.replace('\\', "/"))
        }
    }
    
    fn get_state(&self) -> SourceState {
        self.state.lock()
            .map(|s| s.clone())
            .unwrap_or(SourceState::Error("Failed to lock state".to_string()))
    }
    
    fn start(&mut self) -> Result<()> {
        self.create_pipeline()?;
        
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Playing)
                .map_err(|_| SourceVideoError::StateChange("Failed to set playing state".to_string()))?;
            
            self.set_state(SourceState::Playing);
            log::info!("Started file source: {} ({})", self.name, self.file_path);
            Ok(())
        } else {
            Err(SourceVideoError::pipeline("Pipeline not created"))
        }
    }
    
    fn stop(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null)
                .map_err(|_| SourceVideoError::StateChange("Failed to set null state".to_string()))?;
            
            self.set_state(SourceState::Stopped);
            log::info!("Stopped file source: {}", self.name);
        }
        
        self.pipeline = None;
        Ok(())
    }
    
    fn pause(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Paused)
                .map_err(|_| SourceVideoError::StateChange("Failed to set paused state".to_string()))?;
            
            self.set_state(SourceState::Paused);
            log::info!("Paused file source: {}", self.name);
            Ok(())
        } else {
            Err(SourceVideoError::pipeline("Pipeline not created"))
        }
    }
    
    fn resume(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Playing)
                .map_err(|_| SourceVideoError::StateChange("Failed to resume playing".to_string()))?;
            
            self.set_state(SourceState::Playing);
            log::info!("Resumed file source: {}", self.name);
            Ok(())
        } else {
            Err(SourceVideoError::pipeline("Pipeline not created"))
        }
    }
    
    fn get_pipeline(&self) -> Option<&gst::Pipeline> {
        self.pipeline.as_ref()
    }
}

impl FileVideoSource {
    pub fn reload(&mut self) -> Result<()> {
        log::info!("Reloading file source: {} ({})", self.name, self.file_path);
        
        // Get current state
        let current_state = self.get_state();
        
        // Stop the current pipeline
        if self.pipeline.is_some() {
            self.stop()?;
        }
        
        // Check if file still exists
        let path = Path::new(&self.file_path);
        if !path.exists() {
            return Err(SourceVideoError::FileNotFound(self.file_path.clone()));
        }
        
        // Recreate and restart based on previous state
        match current_state {
            SourceState::Playing => {
                self.start()?;
            }
            SourceState::Paused => {
                self.start()?;
                self.pause()?;
            }
            _ => {
                // If it was stopped or in error, just recreate pipeline
                self.create_pipeline()?;
            }
        }
        
        log::info!("Successfully reloaded file source: {}", self.name);
        Ok(())
    }
    
    pub fn supports_hot_reload(&self) -> bool {
        true
    }
    
    pub fn get_file_path(&self) -> &str {
        &self.file_path
    }
    
    pub fn update_file_path(&mut self, new_path: impl Into<String>) -> Result<()> {
        let new_path = new_path.into();
        let path = Path::new(&new_path);
        
        if !path.exists() {
            return Err(SourceVideoError::FileNotFound(new_path));
        }
        
        self.file_path = new_path;
        self.reload()
    }
}

/// Factory for creating file-based video sources
pub struct FileSourceFactory;

impl FileSourceFactory {
    pub fn create_from_path(path: &Path, name: Option<String>) -> Result<FileVideoSource> {
        if !path.exists() {
            return Err(SourceVideoError::FileNotFound(path.display().to_string()));
        }
        
        let name = name.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file-source")
                .to_string()
        });
        
        let source = FileVideoSource::new(path.display().to_string(), name);
        Ok(source)
    }
    
    pub fn create_from_config(config: &VideoSourceConfig) -> Result<FileVideoSource> {
        FileVideoSource::from_config(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_file_video_source_creation() {
        let source = FileVideoSource::new("/tmp/video.mp4", "test-source");
        assert_eq!(source.get_name(), "test-source");
        assert_eq!(source.get_state(), SourceState::Created);
        assert!(source.get_uri().starts_with("file:///"));
    }
    
    #[test]
    fn test_file_source_with_mount_point() {
        let mut source = FileVideoSource::new("/tmp/video.mp4", "test-source");
        source.set_mount_point("videos/test".to_string());
        assert_eq!(source.get_uri(), "rtsp://localhost:8554/videos/test");
    }
    
    #[test]
    fn test_file_source_factory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.mp4");
        fs::write(&file_path, b"dummy video data").unwrap();
        
        let source = FileSourceFactory::create_from_path(&file_path, Some("test".to_string()));
        assert!(source.is_ok());
        
        let source = source.unwrap();
        assert_eq!(source.get_name(), "test");
    }
    
    #[test]
    fn test_file_not_found_error() {
        let result = FileSourceFactory::create_from_path(
            Path::new("/nonexistent/video.mp4"),
            None
        );
        assert!(result.is_err());
    }
}