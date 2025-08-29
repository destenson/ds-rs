use super::{Backend, BackendCapabilities, BackendType};
use crate::error::{DeepStreamError, Result};
use crate::platform::PlatformInfo;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::collections::HashMap;

pub struct DeepStreamBackend {
    capabilities: BackendCapabilities,
    platform: PlatformInfo,
}

impl DeepStreamBackend {
    fn create_capabilities() -> BackendCapabilities {
        BackendCapabilities {
            supports_inference: true,
            supports_tracking: true,
            supports_osd: true,
            supports_batching: true,
            supports_hardware_decode: true,
            max_batch_size: 30,
            available_elements: vec![
                "nvstreammux".to_string(),
                "nvinfer".to_string(),
                "nvtracker".to_string(),
                "nvdsosd".to_string(),
                "nvtiler".to_string(),
                "nvvideoconvert".to_string(),
                "nvv4l2decoder".to_string(),
                "nveglglessink".to_string(),
            ],
        }
    }

    fn create_element(element_type: &str, name: Option<&str>) -> Result<gst::Element> {
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
}

impl Backend for DeepStreamBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::DeepStream
    }

    fn capabilities(&self) -> &BackendCapabilities {
        &self.capabilities
    }

    fn is_available() -> bool {
        super::detector::check_deepstream_availability()
    }

    fn new(platform: &PlatformInfo) -> Result<Box<dyn Backend>> {
        if !Self::is_available() {
            return Err(DeepStreamError::BackendNotAvailable {
                backend: "DeepStream".to_string(),
            });
        }

        Ok(Box::new(Self {
            capabilities: Self::create_capabilities(),
            platform: platform.clone(),
        }))
    }

    fn create_stream_mux(&self, name: Option<&str>) -> Result<gst::Element> {
        let mux = Self::create_element("nvstreammux", name)?;

        // Set platform-specific properties
        mux.set_property("batch-size", 30u32);
        mux.set_property("width", 1920i32);
        mux.set_property("height", 1080i32);
        mux.set_property("batched-push-timeout", self.platform.get_batch_timeout());
        mux.set_property("gpu-id", self.platform.get_gpu_id());
        mux.set_property("live-source", 1i32);

        Ok(mux)
    }

    fn create_inference(&self, name: Option<&str>, config_path: &str) -> Result<gst::Element> {
        let nvinfer = Self::create_element("nvinfer", name)?;

        nvinfer.set_property("config-file-path", config_path);
        nvinfer.set_property("gpu-id", self.platform.get_gpu_id());

        Ok(nvinfer)
    }

    fn create_tracker(&self, name: Option<&str>) -> Result<gst::Element> {
        let tracker = Self::create_element("nvtracker", name)?;

        tracker.set_property("gpu-id", self.platform.get_gpu_id());
        tracker.set_property(
            "ll-lib-file",
            "/opt/nvidia/deepstream/deepstream/lib/libnvds_nvmultiobjecttracker.so",
        );
        tracker.set_property("ll-config-file", "tracker_config.yml");
        tracker.set_property("tracker-width", 640i32);
        tracker.set_property("tracker-height", 480i32);

        Ok(tracker)
    }

    fn create_tiler(&self, name: Option<&str>) -> Result<gst::Element> {
        let tiler = Self::create_element("nvtiler", name)?;

        tiler.set_property("width", 1920u32);
        tiler.set_property("height", 1080u32);
        tiler.set_property("rows", 2u32);
        tiler.set_property("columns", 2u32);
        tiler.set_property("gpu-id", self.platform.get_gpu_id());

        Ok(tiler)
    }

    fn create_osd(&self, name: Option<&str>) -> Result<gst::Element> {
        let osd = Self::create_element("nvdsosd", name)?;

        osd.set_property("process-mode", 0i32); // GPU_MODE
        osd.set_property("gpu-id", self.platform.get_gpu_id());
        osd.set_property("display-text", 1i32);
        osd.set_property("display-bbox", 1i32);
        osd.set_property("display-mask", 0i32);

        Ok(osd)
    }

    fn create_video_convert(&self, name: Option<&str>) -> Result<gst::Element> {
        let convert = Self::create_element("nvvideoconvert", name)?;

        convert.set_property("gpu-id", self.platform.get_gpu_id());
        convert.set_property("nvbuf-memory-type", 0i32); // NVBUF_MEM_DEFAULT

        Ok(convert)
    }

    fn create_video_sink(&self, name: Option<&str>) -> Result<gst::Element> {
        // Use platform-appropriate sink
        let sink_type = if self.platform.is_jetson() {
            "nveglglessink"
        } else if cfg!(target_os = "windows") {
            "d3dvideosink"
        } else {
            "nveglglessink"
        };

        let sink = gst::ElementFactory::make(sink_type)
            .name(name.unwrap_or("video-sink"))
            .build()
            .or_else(|_| {
                // Fallback to autovideosink if native sink fails
                gst::ElementFactory::make("autovideosink")
                    .name(name.unwrap_or("video-sink"))
                    .build()
            })
            .map_err(|_| DeepStreamError::ElementCreation {
                element: sink_type.to_string(),
            })?;

        sink.set_property("sync", false);

        Ok(sink)
    }

    fn create_decoder(&self, name: Option<&str>) -> Result<gst::Element> {
        let decoder_type = if self.platform.is_jetson() {
            "nvv4l2decoder"
        } else {
            "nvdec"
        };

        let decoder = Self::create_element(decoder_type, name)?;

        if self.platform.is_jetson() {
            decoder.set_property("enable-max-performance", true);
            decoder.set_property("drop-frame-interval", 0u32);
            decoder.set_property("num-extra-surfaces", 0u32);
        } else {
            decoder.set_property("gpu-id", self.platform.get_gpu_id());
        }

        Ok(decoder)
    }

    fn configure_element(
        &self,
        element: &gst::Element,
        config: &HashMap<String, String>,
    ) -> Result<()> {
        for (key, value) in config {
            // Parse and set properties based on type
            if let Ok(int_val) = value.parse::<i32>() {
                element.set_property_from_str(key, &int_val.to_string());
            } else if let Ok(uint_val) = value.parse::<u32>() {
                element.set_property_from_str(key, &uint_val.to_string());
            } else if let Ok(bool_val) = value.parse::<bool>() {
                element.set_property_from_str(key, &bool_val.to_string());
            } else {
                element.set_property_from_str(key, value);
            }
        }
        Ok(())
    }

    fn get_element_mapping(&self, deepstream_element: &str) -> Option<&str> {
        // DeepStream backend uses native elements, no mapping needed
        match deepstream_element {
            "nvstreammux" => Some("nvstreammux"),
            "nvinfer" => Some("nvinfer"),
            "nvtracker" => Some("nvtracker"),
            "nvdsosd" => Some("nvdsosd"),
            "nvtiler" => Some("nvtiler"),
            "nvvideoconvert" => Some("nvvideoconvert"),
            _ => None,
        }
    }
}
