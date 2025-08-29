use super::{Backend, BackendType};
use crate::error::{DeepStreamError, Result};
use crate::platform::PlatformInfo;
use gstreamer as gst;
use gstreamer::prelude::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static DETECTION_CACHE: Lazy<Mutex<Option<BackendType>>> = Lazy::new(|| Mutex::new(None));

const DEEPSTREAM_ELEMENTS: &[&str] = &[
    "nvstreammux",
    "nvinfer",
    "nvtracker",
    "nvdsosd",
    "nvtiler",
    "nvvideoconvert",
];

const STANDARD_ELEMENTS: &[&str] = &[
    "compositor",
    "queue",
    "videoconvert",
    "textoverlay",
    "videoscale",
    "identity",
];

pub fn detect_available_backends() -> Vec<BackendType> {
    let mut available = Vec::new();

    // Initialize GStreamer if not already done
    let _ = gst::init();

    // Check for DeepStream backend
    if check_deepstream_availability() {
        available.push(BackendType::DeepStream);
    }

    // Check for standard GStreamer backend
    if check_standard_availability() {
        available.push(BackendType::Standard);
    }

    // Mock backend is always available
    available.push(BackendType::Mock);

    available
}

pub fn detect_and_create_backend(platform: &PlatformInfo) -> Result<Box<dyn Backend>> {
    // Check cache first
    if let Ok(cache) = DETECTION_CACHE.lock() {
        if let Some(cached_type) = *cache {
            log::debug!("Using cached backend type: {:?}", cached_type);
            return create_backend(cached_type, platform);
        }
    }

    // Initialize GStreamer if not already done
    gst::init().map_err(|e| DeepStreamError::GStreamer(e.into()))?;

    // Detect optimal backend
    let backend_type = if platform.has_nvidia_hardware() && check_deepstream_availability() {
        log::info!("DeepStream elements detected, using DeepStream backend");
        BackendType::DeepStream
    } else if check_standard_availability() {
        log::info!("Standard GStreamer elements detected, using standard backend");
        BackendType::Standard
    } else {
        log::warn!("No suitable GStreamer elements found, using mock backend");
        BackendType::Mock
    };

    // Cache the detection result
    if let Ok(mut cache) = DETECTION_CACHE.lock() {
        *cache = Some(backend_type);
    }

    create_backend(backend_type, platform)
}

fn create_backend(backend_type: BackendType, platform: &PlatformInfo) -> Result<Box<dyn Backend>> {
    match backend_type {
        BackendType::DeepStream => super::deepstream::DeepStreamBackend::new(platform),
        BackendType::Standard => super::standard::StandardBackend::new(platform),
        BackendType::Mock => super::mock::MockBackend::new(platform),
    }
}

pub fn check_deepstream_availability() -> bool {
    for element in DEEPSTREAM_ELEMENTS {
        if !check_element_availability(element) {
            log::debug!("DeepStream element '{}' not found", element);
            return false;
        }
    }
    log::debug!("All DeepStream elements found");
    true
}

fn check_standard_availability() -> bool {
    for element in STANDARD_ELEMENTS {
        if !check_element_availability(element) {
            log::debug!("Standard element '{}' not found", element);
            return false;
        }
    }
    log::debug!("All standard GStreamer elements found");
    true
}

pub fn check_element_availability(element_name: &str) -> bool {
    gst::ElementFactory::find(element_name).is_some()
}

pub fn list_available_elements() -> Vec<String> {
    let registry = gst::Registry::get();
    let mut elements = Vec::new();

    for feature in registry.features_by_plugin("nvcodec") {
        if let Ok(factory) = feature.downcast::<gst::ElementFactory>() {
            elements.push(factory.name().to_string());
        }
    }

    for feature in registry.features_by_plugin("nvinfer") {
        if let Ok(factory) = feature.downcast::<gst::ElementFactory>() {
            elements.push(factory.name().to_string());
        }
    }

    for feature in registry.features_by_plugin("nvstreammux") {
        if let Ok(factory) = feature.downcast::<gst::ElementFactory>() {
            elements.push(factory.name().to_string());
        }
    }

    elements
}

pub fn get_element_properties(element_name: &str) -> HashMap<String, String> {
    let mut properties = HashMap::new();

    if let Some(factory) = gst::ElementFactory::find(element_name) {
        if let Ok(element) = factory.create().build() {
            // Get element class for property listing
            let class = element.element_class();
            for property in class.list_properties() {
                let name = property.name().to_string();
                let type_name = property.value_type().name().to_string();
                properties.insert(name, type_name);
            }
        }
    }

    properties
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_detection() {
        let _ = gst::init();
        let backends = detect_available_backends();

        // At least mock backend should be available
        assert!(!backends.is_empty());
        assert!(backends.contains(&BackendType::Mock));

        println!("Available backends: {:?}", backends);
    }

    #[test]
    fn test_element_availability() {
        let _ = gst::init();

        // Standard elements that should be available
        assert!(check_element_availability("queue"));
        assert!(check_element_availability("identity"));

        // DeepStream elements might not be available
        let has_deepstream = check_element_availability("nvstreammux");
        println!("DeepStream available: {}", has_deepstream);
    }

    #[test]
    fn test_cached_detection() {
        let _ = gst::init();
        let platform = PlatformInfo::detect().unwrap();

        // First detection
        let _ = detect_and_create_backend(&platform);

        // Second detection should use cache
        let result = detect_and_create_backend(&platform);
        assert!(result.is_ok());
    }
}
