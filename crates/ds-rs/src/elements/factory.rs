use super::{DeepStreamElementType, ElementBuilder};
use crate::backend::{Backend, BackendManager};
use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::Arc;

pub struct ElementFactory {
    backend_manager: Arc<BackendManager>,
}

impl ElementFactory {
    pub fn new(backend_manager: Arc<BackendManager>) -> Self {
        Self { backend_manager }
    }

    pub fn with_default_backend() -> Result<Self> {
        Ok(Self {
            backend_manager: Arc::new(BackendManager::new()?),
        })
    }

    pub fn backend(&self) -> &dyn Backend {
        self.backend_manager.backend()
    }

    pub fn create(&self, element_type: DeepStreamElementType) -> ElementBuilder {
        ElementBuilder::new(element_type)
    }

    pub fn create_element(
        &self,
        element_type: DeepStreamElementType,
        name: Option<&str>,
    ) -> Result<gst::Element> {
        let mut builder = self.create(element_type);

        if let Some(n) = name {
            builder = builder.name(n);
        }

        builder.build_with_backend(self.backend())
    }

    pub fn create_stream_mux(&self, name: Option<&str>) -> Result<gst::Element> {
        self.create_element(DeepStreamElementType::StreamMux, name)
    }

    pub fn create_inference(&self, name: Option<&str>, config_path: &str) -> Result<gst::Element> {
        self.create(DeepStreamElementType::Inference)
            .name(name.unwrap_or("inference"))
            .property("config-file-path", config_path)
            .build_with_backend(self.backend())
    }

    pub fn create_tracker(&self, name: Option<&str>) -> Result<gst::Element> {
        self.create_element(DeepStreamElementType::Tracker, name)
    }

    pub fn create_tiler(&self, name: Option<&str>) -> Result<gst::Element> {
        self.create_element(DeepStreamElementType::Tiler, name)
    }

    pub fn create_osd(&self, name: Option<&str>) -> Result<gst::Element> {
        self.create_element(DeepStreamElementType::Osd, name)
    }

    pub fn create_video_convert(&self, name: Option<&str>) -> Result<gst::Element> {
        self.create_element(DeepStreamElementType::VideoConvert, name)
    }

    pub fn create_video_sink(&self, name: Option<&str>) -> Result<gst::Element> {
        self.create_element(DeepStreamElementType::VideoSink, name)
    }

    pub fn create_decoder(&self, name: Option<&str>) -> Result<gst::Element> {
        self.create_element(DeepStreamElementType::Decoder, name)
    }

    pub fn create_standard_element(
        &self,
        element_type: &str,
        name: Option<&str>,
    ) -> Result<gst::Element> {
        let mut builder = gst::ElementFactory::make(element_type);

        if let Some(n) = name {
            builder = builder.name(n);
        }

        builder
            .build()
            .map_err(|_| DeepStreamError::ElementCreation {
                element: element_type.to_string(),
            })
    }

    pub fn create_uri_decode_bin(&self, uri: &str, name: Option<&str>) -> Result<gst::Element> {
        let uridecodebin = self.create_standard_element("uridecodebin", name)?;
        uridecodebin.set_property("uri", uri);
        Ok(uridecodebin)
    }

    pub fn create_queue(&self, name: Option<&str>) -> Result<gst::Element> {
        self.create_standard_element("queue", name)
    }

    pub fn create_caps_filter(&self, caps: &gst::Caps, name: Option<&str>) -> Result<gst::Element> {
        let capsfilter = self.create_standard_element("capsfilter", name)?;
        capsfilter.set_property("caps", caps);
        Ok(capsfilter)
    }

    pub fn validate_element_availability(&self, element_type: DeepStreamElementType) -> bool {
        let backend = self.backend();
        let capabilities = backend.capabilities();

        match element_type {
            DeepStreamElementType::Inference => capabilities.supports_inference,
            DeepStreamElementType::Tracker => capabilities.supports_tracking,
            DeepStreamElementType::Osd => capabilities.supports_osd,
            DeepStreamElementType::StreamMux => capabilities.supports_batching,
            _ => true,
        }
    }

    pub fn get_backend_mapping(&self, deepstream_element: &str) -> Option<&str> {
        self.backend().get_element_mapping(deepstream_element)
    }
}

pub struct PipelineElements {
    pub pipeline: gst::Pipeline,
    pub stream_mux: gst::Element,
    pub pgie: Option<gst::Element>,
    pub tracker: Option<gst::Element>,
    pub tiler: Option<gst::Element>,
    pub video_convert: gst::Element,
    pub osd: gst::Element,
    pub sink: gst::Element,
}

impl PipelineElements {
    pub fn create_base_pipeline(factory: &ElementFactory, name: &str) -> Result<Self> {
        let pipeline = gst::Pipeline::builder().name(name).build();

        let stream_mux = factory.create_stream_mux(Some("stream-mux"))?;
        let video_convert = factory.create_video_convert(Some("video-convert"))?;
        let osd = factory.create_osd(Some("on-screen-display"))?;
        let sink = factory.create_video_sink(Some("video-sink"))?;

        pipeline.add_many([&stream_mux, &video_convert, &osd, &sink])?;

        Ok(Self {
            pipeline,
            stream_mux,
            pgie: None,
            tracker: None,
            tiler: None,
            video_convert,
            osd,
            sink,
        })
    }

    pub fn add_inference(&mut self, factory: &ElementFactory, config_path: &str) -> Result<()> {
        let pgie = factory.create_inference(Some("primary-inference"), config_path)?;
        self.pipeline.add(&pgie)?;
        self.pgie = Some(pgie);
        Ok(())
    }

    pub fn add_tracker(&mut self, factory: &ElementFactory) -> Result<()> {
        let tracker = factory.create_tracker(Some("object-tracker"))?;
        self.pipeline.add(&tracker)?;
        self.tracker = Some(tracker);
        Ok(())
    }

    pub fn add_tiler(&mut self, factory: &ElementFactory) -> Result<()> {
        let tiler = factory.create_tiler(Some("tiler"))?;
        self.pipeline.add(&tiler)?;
        self.tiler = Some(tiler);
        Ok(())
    }

    pub fn link_pipeline(&self) -> Result<()> {
        // Build link chain based on available elements
        let mut link_chain = vec![&self.stream_mux];

        if let Some(ref pgie) = self.pgie {
            link_chain.push(pgie);
        }

        if let Some(ref tracker) = self.tracker {
            link_chain.push(tracker);
        }

        if let Some(ref tiler) = self.tiler {
            link_chain.push(tiler);
        }

        link_chain.push(&self.video_convert);
        link_chain.push(&self.osd);
        link_chain.push(&self.sink);

        // Link all elements in sequence
        for i in 0..link_chain.len() - 1 {
            link_chain[i].link(link_chain[i + 1]).map_err(|_| {
                DeepStreamError::PadLinking(format!(
                    "Failed to link pipeline elements at index {}",
                    i
                ))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendType;

    #[test]
    fn test_factory_creation() {
        let _ = gst::init();
        let factory = ElementFactory::with_default_backend();
        assert!(factory.is_ok());
    }

    #[test]
    fn test_element_creation() {
        let _ = gst::init();
        let backend_manager = BackendManager::with_backend(BackendType::Mock).unwrap();
        let factory = ElementFactory::new(Arc::new(backend_manager));

        // Test creating various elements
        assert!(factory.create_stream_mux(Some("test-mux")).is_ok());
        assert!(factory.create_video_convert(None).is_ok());
        assert!(factory.create_osd(None).is_ok());
        assert!(factory.create_video_sink(None).is_ok());
    }

    #[test]
    fn test_pipeline_elements() {
        let _ = gst::init();
        let backend_manager = BackendManager::with_backend(BackendType::Mock).unwrap();
        let factory = ElementFactory::new(Arc::new(backend_manager));

        let pipeline = PipelineElements::create_base_pipeline(&factory, "test-pipeline");
        assert!(pipeline.is_ok());

        let mut pipeline = pipeline.unwrap();
        assert!(pipeline.add_inference(&factory, "test.txt").is_ok());
        assert!(pipeline.add_tracker(&factory).is_ok());
        assert!(pipeline.link_pipeline().is_ok());
    }
}
