use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
#[cfg(feature = "nalgebra")]
use super::tracker::CentroidTracker;

/// Create a CPU detector element that performs object detection
/// This creates a bin containing an identity element with a probe for detection
pub fn create_cpu_detector(name: Option<&str>, model_path: Option<&str>) -> Result<gst::Element> {
    let bin = gst::Bin::builder()
        .name(name.unwrap_or("cpu-detector"))
        .build();
    
    // Create identity element to pass through video
    let identity = gst::ElementFactory::make("identity")
        .name("detector-identity")
        .build()
        .map_err(|_| DeepStreamError::ElementCreation {
            element: "identity".to_string(),
        })?;
    
    // Create queue for buffering - make it leaky to prevent blocking
    let queue = gst::ElementFactory::make("queue")
        .name("detector-queue")
        .property("max-size-buffers", 1u32)         // Small buffer
        .property("max-size-bytes", 0u32)           // No byte limit
        .property("max-size-time", 0u64)            // No time limit
        .property_from_str("leaky", "upstream")   // Leak downstream (drop old buffers)
        .build()
        .map_err(|_| DeepStreamError::ElementCreation {
            element: "queue".to_string(),
        })?;
    
    bin.add_many([&queue, &identity])?;
    queue.link(&identity)?;
    
    // Create ghost pads
    let sink_pad = queue.static_pad("sink").unwrap();
    let src_pad = identity.static_pad("src").unwrap();
    
    bin.add_pad(&gst::GhostPad::with_target(&sink_pad)?)?;
    bin.add_pad(&gst::GhostPad::with_target(&src_pad)?)?;
    
    // Store detector state (would be initialized with model in real implementation)
    if let Some(model) = model_path {
        // In a real implementation, we would:
        // 1. Load the ONNX model
        // 2. Add a probe to the identity element
        // 3. Run detection on each buffer
        // 4. Attach metadata to buffers
        
        log::info!("CPU detector initialized with model: {}", model);
    } else {
        log::warn!("CPU detector created without model - detection disabled");
    }
    
    Ok(bin.upcast())
}

/// Create a CPU tracker element that tracks detected objects
#[cfg(feature = "nalgebra")]
pub fn create_cpu_tracker(name: Option<&str>) -> Result<gst::Element> {
    let bin = gst::Bin::builder()
        .name(name.unwrap_or("cpu-tracker"))
        .build();
    
    // Create identity element to pass through video
    let identity = gst::ElementFactory::make("identity")
        .name("tracker-identity")
        .build()
        .map_err(|_| DeepStreamError::ElementCreation {
            element: "identity".to_string(),
        })?;
    
    bin.add(&identity)?;
    
    // Create ghost pads
    let sink_pad = identity.static_pad("sink").unwrap();
    let src_pad = identity.static_pad("src").unwrap();
    
    bin.add_pad(&gst::GhostPad::with_target(&sink_pad)?)?;
    bin.add_pad(&gst::GhostPad::with_target(&src_pad)?)?;
    
    // Initialize tracker
    let tracker = Arc::new(Mutex::new(CentroidTracker::new(50.0, 30)));
    
    // Add probe to process detection metadata and perform tracking
    let _tracker_clone = tracker.clone();
    src_pad.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
        if let Some(_buffer) = info.buffer() {
            // In a real implementation, we would:
            // 1. Extract detection metadata from buffer
            // 2. Update tracker with detections
            // 3. Attach tracking metadata to buffer
            
            // For now, just pass through
            log::trace!("CPU tracker processing buffer");
        }
        gst::PadProbeReturn::Ok
    });
    
    log::info!("CPU tracker initialized with Centroid algorithm");
    
    Ok(bin.upcast())
}

/// Create a CPU tracker element that tracks detected objects (fallback without nalgebra)
#[cfg(not(feature = "nalgebra"))]
pub fn create_cpu_tracker(name: Option<&str>) -> Result<gst::Element> {
    // Return a simple passthrough identity element when nalgebra is not available
    let identity = gst::ElementFactory::make("identity")
        .name(name.unwrap_or("cpu-tracker-passthrough"))
        .build()
        .map_err(|_| DeepStreamError::ElementCreation {
            element: "identity".to_string(),
        })?;
    
    log::warn!("CPU tracker not available: nalgebra feature not enabled. Using passthrough.");
    
    Ok(identity)
}

/// Create a CPU OSD (On-Screen Display) element for drawing bounding boxes
pub fn create_cpu_osd(name: Option<&str>) -> Result<gst::Element> {
    let bin = gst::Bin::builder()
        .name(name.unwrap_or("cpu-osd"))
        .build();
    
    // Create videoconvert for format conversion
    let convert_in = gst::ElementFactory::make("videoconvert")
        .name("osd-convert-in")
        .build()
        .map_err(|_| DeepStreamError::ElementCreation {
            element: "videoconvert".to_string(),
        })?;
    
    // Create cairooverlay for drawing
    let overlay = gst::ElementFactory::make("cairooverlay")
        .name("osd-overlay")
        .build()
        .or_else(|_| {
            // Fallback to textoverlay if cairooverlay not available
            gst::ElementFactory::make("textoverlay")
                .name("osd-textoverlay")
                .property("text", "CPU Vision Backend")
                .property_from_str("valignment", "top")
                .property_from_str("halignment", "left")
                .build()
        })
        .map_err(|_| DeepStreamError::ElementCreation {
            element: "overlay".to_string(),
        })?;
    
    // Create videoconvert for output
    let convert_out = gst::ElementFactory::make("videoconvert")
        .name("osd-convert-out")
        .build()
        .map_err(|_| DeepStreamError::ElementCreation {
            element: "videoconvert".to_string(),
        })?;
    
    bin.add_many([&convert_in, &overlay, &convert_out])?;
    convert_in.link(&overlay)?;
    overlay.link(&convert_out)?;
    
    // Create ghost pads
    let sink_pad = convert_in.static_pad("sink").unwrap();
    let src_pad = convert_out.static_pad("src").unwrap();
    
    bin.add_pad(&gst::GhostPad::with_target(&sink_pad)?)?;
    bin.add_pad(&gst::GhostPad::with_target(&src_pad)?)?;
    
    // If using cairooverlay, set up drawing callback
    if overlay.type_().name() == "GstCairoOverlay" {
        // In a real implementation, we would set up the draw signal
        // to draw bounding boxes from metadata
        log::info!("CPU OSD using Cairo for rendering");
    } else {
        log::info!("CPU OSD using text overlay fallback");
    }
    
    Ok(bin.upcast())
}

/// Create a complete CPU vision pipeline bin
pub fn create_cpu_vision_pipeline(
    name: Option<&str>,
    model_path: Option<&str>,
) -> Result<gst::Element> {
    let bin = gst::Bin::builder()
        .name(name.unwrap_or("cpu-vision"))
        .build();
    
    // Create elements
    let detector = create_cpu_detector(Some("detector"), model_path)?;
    let tracker = create_cpu_tracker(Some("tracker"))?;
    let osd = create_cpu_osd(Some("osd"))?;
    
    // Add to bin
    bin.add_many([&detector, &tracker, &osd])?;
    
    // Link elements
    detector.link(&tracker)?;
    tracker.link(&osd)?;
    
    // Create ghost pads
    let sink_pad = detector.static_pad("sink").unwrap();
    let src_pad = osd.static_pad("src").unwrap();
    
    bin.add_pad(&gst::GhostPad::with_target(&sink_pad)?)?;
    bin.add_pad(&gst::GhostPad::with_target(&src_pad)?)?;
    
    log::info!("Created complete CPU vision pipeline");
    
    Ok(bin.upcast())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_cpu_detector() {
        gst::init().unwrap();
        
        let detector = create_cpu_detector(Some("test-detector"), None).unwrap();
        assert_eq!(detector.name(), "test-detector");
        assert!(detector.static_pad("sink").is_some());
        assert!(detector.static_pad("src").is_some());
    }
    
    #[test]
    fn test_create_cpu_tracker() {
        gst::init().unwrap();
        
        let tracker = create_cpu_tracker(Some("test-tracker")).unwrap();
        assert_eq!(tracker.name(), "test-tracker");
        assert!(tracker.static_pad("sink").is_some());
        assert!(tracker.static_pad("src").is_some());
    }
    
    #[test]
    fn test_create_cpu_osd() {
        gst::init().unwrap();
        
        let osd = create_cpu_osd(Some("test-osd")).unwrap();
        assert_eq!(osd.name(), "test-osd");
        assert!(osd.static_pad("sink").is_some());
        assert!(osd.static_pad("src").is_some());
    }
    
    #[test]
    fn test_create_cpu_vision_pipeline() {
        gst::init().unwrap();
        
        let pipeline = create_cpu_vision_pipeline(Some("test-vision"), None).unwrap();
        assert_eq!(pipeline.name(), "test-vision");
        assert!(pipeline.static_pad("sink").is_some());
        assert!(pipeline.static_pad("src").is_some());
    }
}
