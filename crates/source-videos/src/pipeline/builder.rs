use crate::error::{Result, SourceVideoError};
use gstreamer as gst;
use gstreamer::prelude::*;

pub struct PipelineBuilder {
    pipeline: gst::Pipeline,
    elements: Vec<gst::Element>,
}

impl PipelineBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        let pipeline = gst::Pipeline::builder().name(name.into()).build();

        Self {
            pipeline,
            elements: Vec::new(),
        }
    }

    pub fn add_element(mut self, element: gst::Element) -> Result<Self> {
        self.pipeline.add(&element).map_err(|_| {
            SourceVideoError::pipeline(format!("Failed to add element: {:?}", element.name()))
        })?;
        self.elements.push(element);
        Ok(self)
    }

    pub fn add_many(mut self, elements: Vec<gst::Element>) -> Result<Self> {
        for element in elements {
            self.pipeline.add(&element).map_err(|_| {
                SourceVideoError::pipeline(format!("Failed to add element: {:?}", element.name()))
            })?;
            self.elements.push(element);
        }
        Ok(self)
    }

    pub fn link_all(self) -> Result<Self> {
        if self.elements.len() < 2 {
            return Ok(self);
        }

        for window in self.elements.windows(2) {
            window[0].link(&window[1]).map_err(|_| {
                SourceVideoError::linking(
                    window[0].name().to_string(),
                    window[1].name().to_string(),
                )
            })?;
        }

        Ok(self)
    }

    pub fn link_elements(self, src: &gst::Element, sink: &gst::Element) -> Result<Self> {
        src.link(sink).map_err(|_| {
            SourceVideoError::linking(src.name().to_string(), sink.name().to_string())
        })?;
        Ok(self)
    }

    pub fn link_filtered(
        self,
        src: &gst::Element,
        sink: &gst::Element,
        caps: &gst::Caps,
    ) -> Result<Self> {
        src.link_filtered(sink, caps).map_err(|_| {
            SourceVideoError::linking(
                format!("{} (with caps)", src.name()),
                sink.name().to_string(),
            )
        })?;
        Ok(self)
    }

    pub fn set_property<V: Into<gst::glib::Value>>(
        self,
        element_name: &str,
        property: &str,
        value: V,
    ) -> Result<Self> {
        let element = self.pipeline.by_name(element_name).ok_or_else(|| {
            SourceVideoError::pipeline(format!("Element '{}' not found in pipeline", element_name))
        })?;

        element.set_property(property, value);
        Ok(self)
    }

    pub fn create_element(&self, factory_name: &str, name: Option<&str>) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make(factory_name);

        if let Some(n) = name {
            builder = builder.name(n);
        }

        builder
            .build()
            .map_err(|_| SourceVideoError::element(factory_name))
    }

    pub fn create_caps(&self, media_type: &str) -> gst::caps::Builder<gst::caps::NoFeature> {
        gst::Caps::builder(media_type)
    }

    pub fn build(self) -> gst::Pipeline {
        self.pipeline
    }

    pub fn get_element(&self, name: &str) -> Option<gst::Element> {
        self.pipeline.by_name(name)
    }

    pub fn pipeline(&self) -> &gst::Pipeline {
        &self.pipeline
    }
}

pub struct ElementBuilder;

impl ElementBuilder {
    pub fn videotestsrc(name: Option<&str>) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make("videotestsrc");
        if let Some(n) = name {
            builder = builder.name(n);
        }
        builder
            .build()
            .map_err(|_| SourceVideoError::element("videotestsrc"))
    }

    pub fn capsfilter(name: Option<&str>, caps: &gst::Caps) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make("capsfilter");
        if let Some(n) = name {
            builder = builder.name(n);
        }
        let element = builder
            .build()
            .map_err(|_| SourceVideoError::element("capsfilter"))?;

        element.set_property("caps", caps);
        Ok(element)
    }

    pub fn videoconvert(name: Option<&str>) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make("videoconvert");
        if let Some(n) = name {
            builder = builder.name(n);
        }
        builder
            .build()
            .map_err(|_| SourceVideoError::element("videoconvert"))
    }

    pub fn x264enc(name: Option<&str>) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make("x264enc");
        if let Some(n) = name {
            builder = builder.name(n);
        }
        builder
            .build()
            .map_err(|_| SourceVideoError::element("x264enc"))
    }

    pub fn rtph264pay(name: Option<&str>) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make("rtph264pay");
        if let Some(n) = name {
            builder = builder.name(n);
        }
        builder
            .build()
            .map_err(|_| SourceVideoError::element("rtph264pay"))
    }

    pub fn filesink(name: Option<&str>, location: &str) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make("filesink");
        if let Some(n) = name {
            builder = builder.name(n);
        }
        let element = builder
            .property("location", location)
            .build()
            .map_err(|_| SourceVideoError::element("filesink"))?;

        Ok(element)
    }

    pub fn fakesink(name: Option<&str>) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make("fakesink");
        if let Some(n) = name {
            builder = builder.name(n);
        }
        builder
            .build()
            .map_err(|_| SourceVideoError::element("fakesink"))
    }
}

pub struct CapsBuilder;

impl CapsBuilder {
    pub fn video_raw(width: u32, height: u32, framerate: (i32, i32), format: &str) -> gst::Caps {
        gst::Caps::builder("video/x-raw")
            .field("width", width as i32)
            .field("height", height as i32)
            .field("framerate", gst::Fraction::new(framerate.0, framerate.1))
            .field("format", format)
            .build()
    }

    pub fn h264_rtp() -> gst::Caps {
        gst::Caps::builder("application/x-rtp")
            .field("media", "video")
            .field("encoding-name", "H264")
            .field("payload", 96i32)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_builder() {
        gst::init().unwrap();

        let pipeline = PipelineBuilder::new("test-pipeline").build();

        assert_eq!(pipeline.name(), "test-pipeline");
    }

    #[test]
    fn test_element_builder() {
        gst::init().unwrap();

        let src = ElementBuilder::videotestsrc(Some("test-src")).unwrap();
        assert_eq!(src.name(), "test-src");

        let caps = CapsBuilder::video_raw(1920, 1080, (30, 1), "I420");
        let filter = ElementBuilder::capsfilter(Some("filter"), &caps).unwrap();
        assert_eq!(filter.name(), "filter");
    }
}
