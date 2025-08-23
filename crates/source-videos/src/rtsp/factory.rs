use crate::config::VideoSourceConfig;
use crate::error::{Result, SourceVideoError};
use crate::patterns::TestPattern;
use gstreamer_rtsp_server as rtsp_server;
use gstreamer_rtsp_server::prelude::*;

pub struct MediaFactoryBuilder {
    launch_string: Option<String>,
    shared: bool,
    eos_shutdown: bool,
    latency: u32,
}

impl MediaFactoryBuilder {
    pub fn new() -> Self {
        Self {
            launch_string: None,
            shared: true,
            eos_shutdown: false,
            latency: 200,
        }
    }
    
    pub fn from_config(mut self, config: &VideoSourceConfig) -> Result<Self> {
        let launch = self.create_launch_string(config)?;
        self.launch_string = Some(launch);
        Ok(self)
    }
    
    pub fn launch_string(mut self, launch: impl Into<String>) -> Self {
        self.launch_string = Some(launch.into());
        self
    }
    
    pub fn shared(mut self, shared: bool) -> Self {
        self.shared = shared;
        self
    }
    
    pub fn eos_shutdown(mut self, shutdown: bool) -> Self {
        self.eos_shutdown = shutdown;
        self
    }
    
    pub fn latency(mut self, latency: u32) -> Self {
        self.latency = latency;
        self
    }
    
    pub fn build(self) -> Result<rtsp_server::RTSPMediaFactory> {
        let launch = self.launch_string
            .ok_or_else(|| SourceVideoError::config("No launch string provided"))?;
        
        let factory = rtsp_server::RTSPMediaFactory::new();
        factory.set_launch(&launch);
        factory.set_shared(self.shared);
        factory.set_eos_shutdown(self.eos_shutdown);
        factory.set_latency(self.latency);
        
        factory.set_property("enable-rtcp", true);
        
        Ok(factory)
    }
    
    fn create_launch_string(&self, config: &VideoSourceConfig) -> Result<String> {
        let pattern = if let crate::config::VideoSourceType::TestPattern { pattern } = &config.source_type {
            TestPattern::from_str(pattern)?
        } else {
            TestPattern::Smpte
        };
        
        let launch = format!(
            "( videotestsrc pattern={} is-live=true ! \
             video/x-raw,width={},height={},framerate={}/{},format={} ! \
             videoconvert ! \
             x264enc tune=zerolatency speed-preset=ultrafast bitrate=2000 ! \
             rtph264pay name=pay0 pt=96 config-interval=1 )",
            pattern.to_gst_pattern(),
            config.resolution.width,
            config.resolution.height,
            config.framerate.numerator,
            config.framerate.denominator,
            config.format.to_caps_string()
        );
        
        Ok(launch)
    }
}

pub fn create_test_pattern_factory(pattern: &str) -> Result<rtsp_server::RTSPMediaFactory> {
    let pattern = TestPattern::from_str(pattern)?;
    
    let launch = format!(
        "( videotestsrc pattern={} is-live=true ! \
         video/x-raw,width=1920,height=1080,framerate=30/1 ! \
         videoconvert ! \
         x264enc tune=zerolatency speed-preset=ultrafast ! \
         rtph264pay name=pay0 pt=96 config-interval=1 )",
        pattern.to_gst_pattern()
    );
    
    MediaFactoryBuilder::new()
        .launch_string(launch)
        .shared(true)
        .build()
}

pub fn create_file_source_factory(file_path: &str) -> Result<rtsp_server::RTSPMediaFactory> {
    let launch = format!(
        "( filesrc location={} ! \
         decodebin ! \
         videoconvert ! \
         x264enc tune=zerolatency speed-preset=ultrafast ! \
         rtph264pay name=pay0 pt=96 config-interval=1 )",
        file_path
    );
    
    MediaFactoryBuilder::new()
        .launch_string(launch)
        .shared(false)
        .eos_shutdown(true)
        .build()
}

pub fn create_custom_factory(launch_string: &str) -> Result<rtsp_server::RTSPMediaFactory> {
    MediaFactoryBuilder::new()
        .launch_string(launch_string)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_media_factory_builder() {
        gstreamer::init().unwrap();
        
        let factory = MediaFactoryBuilder::new()
            .launch_string("( videotestsrc ! fakesink )")
            .shared(false)
            .eos_shutdown(true)
            .latency(100)
            .build();
        
        assert!(factory.is_ok());
    }
    
    #[test]
    fn test_test_pattern_factory() {
        gstreamer::init().unwrap();
        
        let factory = create_test_pattern_factory("smpte");
        assert!(factory.is_ok());
        
        let factory = create_test_pattern_factory("ball");
        assert!(factory.is_ok());
        
        let factory = create_test_pattern_factory("invalid");
        assert!(factory.is_err());
    }
}