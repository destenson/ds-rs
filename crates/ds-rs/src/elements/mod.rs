pub mod abstracted;
pub mod factory;

use crate::error::Result;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeepStreamElementType {
    StreamMux,
    Inference,
    Tracker,
    Tiler,
    Osd,
    VideoConvert,
    VideoSink,
    Decoder,
}

impl DeepStreamElementType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StreamMux => "nvstreammux",
            Self::Inference => "nvinfer",
            Self::Tracker => "nvtracker",
            Self::Tiler => "nvtiler",
            Self::Osd => "nvdsosd",
            Self::VideoConvert => "nvvideoconvert",
            Self::VideoSink => "nveglglessink",
            Self::Decoder => "nvv4l2decoder",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "nvstreammux" => Some(Self::StreamMux),
            "nvinfer" => Some(Self::Inference),
            "nvtracker" => Some(Self::Tracker),
            "nvtiler" => Some(Self::Tiler),
            "nvdsosd" => Some(Self::Osd),
            "nvvideoconvert" => Some(Self::VideoConvert),
            "nveglglessink" | "nv3dsink" => Some(Self::VideoSink),
            "nvv4l2decoder" | "nvdec" => Some(Self::Decoder),
            _ => None,
        }
    }
}

pub trait DeepStreamElement {
    fn element_type(&self) -> DeepStreamElementType;

    fn inner(&self) -> &gst::Element;

    fn inner_mut(&mut self) -> &mut gst::Element;

    fn set_property_from_str(&self, name: &str, value: &str) -> Result<()> {
        self.inner().set_property_from_str(name, value);
        Ok(())
    }

    fn link(&self, dest: &impl DeepStreamElement) -> Result<()> {
        self.inner().link(dest.inner()).map_err(|_| {
            crate::error::DeepStreamError::PadLinking(format!(
                "Failed to link {} to {}",
                self.element_type().name(),
                dest.element_type().name()
            ))
        })
    }

    fn set_state(&self, state: gst::State) -> Result<gst::StateChangeSuccess> {
        self.inner().set_state(state).map_err(|_| {
            crate::error::DeepStreamError::StateChange(format!(
                "Failed to set {} to {:?} state",
                self.element_type().name(),
                state
            ))
        })
    }
}

pub struct ElementBuilder {
    element_type: DeepStreamElementType,
    name: Option<String>,
    properties: HashMap<String, String>,
}

impl ElementBuilder {
    pub fn new(element_type: DeepStreamElementType) -> Self {
        Self {
            element_type,
            name: None,
            properties: HashMap::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn property(mut self, key: impl Into<String>, value: impl ToString) -> Self {
        self.properties.insert(key.into(), value.to_string());
        self
    }

    pub fn build_with_backend(self, backend: &dyn crate::backend::Backend) -> Result<gst::Element> {
        let element = match self.element_type {
            DeepStreamElementType::StreamMux => backend.create_stream_mux(self.name.as_deref())?,
            DeepStreamElementType::Inference => backend.create_inference(
                self.name.as_deref(),
                self.properties
                    .get("config-file-path")
                    .map(|s| s.as_str())
                    .unwrap_or(""),
            )?,
            DeepStreamElementType::Tracker => backend.create_tracker(self.name.as_deref())?,
            DeepStreamElementType::Tiler => backend.create_tiler(self.name.as_deref())?,
            DeepStreamElementType::Osd => backend.create_osd(self.name.as_deref())?,
            DeepStreamElementType::VideoConvert => {
                backend.create_video_convert(self.name.as_deref())?
            }
            DeepStreamElementType::VideoSink => backend.create_video_sink(self.name.as_deref())?,
            DeepStreamElementType::Decoder => backend.create_decoder(self.name.as_deref())?,
        };

        // Apply additional properties
        if !self.properties.is_empty() {
            backend.configure_element(&element, &self.properties)?;
        }

        Ok(element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_type_conversions() {
        assert_eq!(
            DeepStreamElementType::from_name("nvstreammux"),
            Some(DeepStreamElementType::StreamMux)
        );

        assert_eq!(DeepStreamElementType::StreamMux.name(), "nvstreammux");

        assert_eq!(DeepStreamElementType::from_name("invalid"), None);
    }

    #[test]
    fn test_element_builder() {
        let builder = ElementBuilder::new(DeepStreamElementType::StreamMux)
            .name("my-mux")
            .property("batch-size", "30")
            .property("width", "1920");

        assert_eq!(builder.name, Some("my-mux".to_string()));
        assert_eq!(
            builder.properties.get("batch-size"),
            Some(&"30".to_string())
        );
        assert_eq!(builder.properties.get("width"), Some(&"1920".to_string()));
    }
}
