pub mod builder;

use crate::config::{FileContainer, VideoSourceConfig, VideoSourceType};
use crate::error::{Result, SourceVideoError};
use crate::patterns::TestPattern;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::Arc;

pub trait PipelineFactory: Send + Sync {
    fn create_pipeline(&self, config: &VideoSourceConfig) -> Result<gst::Pipeline>;
    fn get_name(&self) -> &str;
}

pub struct TestPatternPipeline;
pub struct FileSinkPipeline;
pub struct RtspSourcePipeline;

impl TestPatternPipeline {
    pub fn new() -> Arc<dyn PipelineFactory> {
        Arc::new(Self)
    }

    fn create_test_src(&self, name: Option<&str>, pattern: &str) -> Result<gst::Element> {
        let src = gst::ElementFactory::make("videotestsrc")
            .name(name.unwrap_or("testsrc"))
            .build()
            .map_err(|_| SourceVideoError::element("videotestsrc"))?;

        let _pattern = TestPattern::from_str(pattern)?; // Validate pattern exists
        src.set_property_from_str("pattern", pattern);

        Ok(src)
    }
}

impl PipelineFactory for TestPatternPipeline {
    fn create_pipeline(&self, config: &VideoSourceConfig) -> Result<gst::Pipeline> {
        let pipeline = gst::Pipeline::builder()
            .name(&format!("test-pattern-{}", config.name))
            .build();

        if let VideoSourceType::TestPattern { pattern } = &config.source_type {
            let src = self.create_test_src(Some("source"), pattern)?;

            src.set_property("is-live", config.is_live);

            if let Some(num_buffers) = config.num_buffers {
                src.set_property("num-buffers", num_buffers);
            }

            let capsfilter = gst::ElementFactory::make("capsfilter")
                .name("filter")
                .build()
                .map_err(|_| SourceVideoError::element("capsfilter"))?;

            let caps = gst::Caps::builder("video/x-raw")
                .field("width", config.resolution.width as i32)
                .field("height", config.resolution.height as i32)
                .field(
                    "framerate",
                    gst::Fraction::new(config.framerate.numerator, config.framerate.denominator),
                )
                .field("format", config.format.to_caps_string())
                .build();

            capsfilter.set_property("caps", &caps);

            let sink = gst::ElementFactory::make("fakesink")
                .name("sink")
                .build()
                .map_err(|_| SourceVideoError::element("fakesink"))?;

            pipeline
                .add_many([&src, &capsfilter, &sink])
                .map_err(|_| SourceVideoError::pipeline("Failed to add elements"))?;

            gst::Element::link_many([&src, &capsfilter, &sink])
                .map_err(|_| SourceVideoError::pipeline("Failed to link elements"))?;

            Ok(pipeline)
        } else {
            Err(SourceVideoError::config(
                "Invalid config for test pattern pipeline",
            ))
        }
    }

    fn get_name(&self) -> &str {
        "TestPatternPipeline"
    }
}

impl FileSinkPipeline {
    pub fn new() -> Arc<dyn PipelineFactory> {
        Arc::new(Self)
    }

    fn create_encoder(&self, format: &FileContainer) -> Result<gst::Element> {
        let encoder_name = match format {
            FileContainer::Mp4 | FileContainer::Mkv | FileContainer::Avi => "x264enc",
            FileContainer::WebM => "vp8enc",
        };

        gst::ElementFactory::make(encoder_name)
            .name("encoder")
            .build()
            .map_err(|_| SourceVideoError::element(encoder_name))
    }
}

impl PipelineFactory for FileSinkPipeline {
    fn create_pipeline(&self, config: &VideoSourceConfig) -> Result<gst::Pipeline> {
        let pipeline = gst::Pipeline::builder()
            .name(&format!("file-sink-{}", config.name))
            .build();

        if let VideoSourceType::File { path, container } = &config.source_type {
            let src = gst::ElementFactory::make("videotestsrc")
                .name("source")
                .build()
                .map_err(|_| SourceVideoError::element("videotestsrc"))?;

            src.set_property("is-live", false);

            if let Some(num_buffers) = config.num_buffers {
                src.set_property("num-buffers", num_buffers);
            } else if let Some(duration) = config.duration {
                let num_buffers = duration * config.framerate.numerator as u64
                    / config.framerate.denominator as u64;
                src.set_property("num-buffers", num_buffers as i32);
            }

            let videoconvert = gst::ElementFactory::make("videoconvert")
                .name("convert")
                .build()
                .map_err(|_| SourceVideoError::element("videoconvert"))?;

            let encoder = self.create_encoder(container)?;

            let muxer = gst::ElementFactory::make(container.muxer_name())
                .name("muxer")
                .build()
                .map_err(|_| SourceVideoError::element(container.muxer_name()))?;

            let filesink = gst::ElementFactory::make("filesink")
                .name("sink")
                .property("location", path)
                .build()
                .map_err(|_| SourceVideoError::element("filesink"))?;

            pipeline
                .add_many([&src, &videoconvert, &encoder, &muxer, &filesink])
                .map_err(|_| SourceVideoError::pipeline("Failed to add elements"))?;

            gst::Element::link_many([&src, &videoconvert, &encoder])
                .map_err(|_| SourceVideoError::pipeline("Failed to link encoding chain"))?;

            encoder
                .link(&muxer)
                .map_err(|_| SourceVideoError::linking("encoder", "muxer"))?;

            muxer
                .link(&filesink)
                .map_err(|_| SourceVideoError::linking("muxer", "filesink"))?;

            Ok(pipeline)
        } else {
            Err(SourceVideoError::config(
                "Invalid config for file sink pipeline",
            ))
        }
    }

    fn get_name(&self) -> &str {
        "FileSinkPipeline"
    }
}

impl RtspSourcePipeline {
    pub fn new() -> Arc<dyn PipelineFactory> {
        Arc::new(Self)
    }
}

impl PipelineFactory for RtspSourcePipeline {
    fn create_pipeline(&self, config: &VideoSourceConfig) -> Result<gst::Pipeline> {
        let pipeline = gst::Pipeline::builder()
            .name(&format!("rtsp-source-{}", config.name))
            .build();

        if let VideoSourceType::Rtsp { .. } = &config.source_type {
            let src = gst::ElementFactory::make("videotestsrc")
                .name("source")
                .property("is-live", true)
                .build()
                .map_err(|_| SourceVideoError::element("videotestsrc"))?;

            let videoconvert = gst::ElementFactory::make("videoconvert")
                .name("convert")
                .build()
                .map_err(|_| SourceVideoError::element("videoconvert"))?;

            let encoder = gst::ElementFactory::make("x264enc")
                .name("encoder")
                .property("tune", "zerolatency")
                .property("speed-preset", "ultrafast")
                .build()
                .map_err(|_| SourceVideoError::element("x264enc"))?;

            let payloader = gst::ElementFactory::make("rtph264pay")
                .name("pay")
                .property("config-interval", 1i32)
                .build()
                .map_err(|_| SourceVideoError::element("rtph264pay"))?;

            let sink = gst::ElementFactory::make("fakesink")
                .name("sink")
                .build()
                .map_err(|_| SourceVideoError::element("fakesink"))?;

            pipeline
                .add_many([&src, &videoconvert, &encoder, &payloader, &sink])
                .map_err(|_| SourceVideoError::pipeline("Failed to add elements"))?;

            gst::Element::link_many([&src, &videoconvert, &encoder, &payloader, &sink])
                .map_err(|_| SourceVideoError::pipeline("Failed to link elements"))?;

            Ok(pipeline)
        } else {
            Err(SourceVideoError::config(
                "Invalid config for RTSP source pipeline",
            ))
        }
    }

    fn get_name(&self) -> &str {
        "RtspSourcePipeline"
    }
}

pub fn create_factory(config: &VideoSourceConfig) -> Arc<dyn PipelineFactory> {
    match &config.source_type {
        VideoSourceType::TestPattern { .. } => TestPatternPipeline::new(),
        VideoSourceType::File { .. } => FileSinkPipeline::new(),
        VideoSourceType::Rtsp { .. } => RtspSourcePipeline::new(),
        VideoSourceType::Directory { .. } => {
            // Directory sources are expanded to individual file sources,
            // so this should not be reached in normal operation
            FileSinkPipeline::new()
        }
        VideoSourceType::FileList { .. } => {
            // FileList sources are expanded to individual file sources,
            // so this should not be reached in normal operation
            FileSinkPipeline::new()
        }
    }
}
