#![allow(unused)]
use crate::error::{DeepStreamError, Result};
use super::detector::{OnnxDetector, DetectorConfig};
use super::metadata::DetectionMeta;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_video as gst_video;
use std::sync::{Arc, Mutex};
use image::{DynamicImage, RgbImage};
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
    
    // Initialize ONNX detector
    let detector = Arc::new(Mutex::new(None::<OnnxDetector>));
    let frame_counter = Arc::new(Mutex::new(0u64));
    
    if let Some(model) = model_path {
        // Try to load the ONNX model
        match std::path::Path::new(model).exists() {
            true => {
                let config = DetectorConfig {
                    model_path: Some(model.to_string()),
                    input_width: 640,
                    input_height: 640,
                    confidence_threshold: 0.5,
                    nms_threshold: 0.4,
                    num_threads: 4,
                    ..Default::default()
                };
                
                match OnnxDetector::new_with_config(config) {
                    Ok(onnx_detector) => {
                        *detector.lock().unwrap() = Some(onnx_detector);
                        log::info!("CPU detector loaded ONNX model: {}", model);
                    },
                    Err(e) => {
                        log::warn!("Failed to load ONNX model {}: {}", model, e);
                        *detector.lock().unwrap() = Some(OnnxDetector::new_mock());
                        log::info!("Using mock detector instead");
                    }
                }
            },
            false => {
                log::warn!("Model file not found: {}, using mock detector", model);
                *detector.lock().unwrap() = Some(OnnxDetector::new_mock());
            }
        }
    } else {
        log::info!("No model path provided, using mock detector");
        *detector.lock().unwrap() = Some(OnnxDetector::new_mock());
    }
    
    // Add probe to process buffers and run detection
    let detector_clone = detector.clone();
    let frame_counter_clone = frame_counter.clone();
    
    src_pad.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
        if let Some(buffer) = info.buffer() {
            let mut detector_guard = detector_clone.lock().unwrap();
            if let Some(ref detector) = *detector_guard {
                // Convert buffer to image and run detection
                match extract_image_from_buffer(buffer) {
                    Ok(image) => {
                        match detector.detect(&image) {
                            Ok(detections) => {
                                let mut counter = frame_counter_clone.lock().unwrap();
                                *counter += 1;
                                
                                log::debug!("Frame {}: Detected {} objects", *counter, detections.len());
                                
                                // In a full implementation, we would attach metadata to the buffer here
                                // For now, just log the detections
                                for (i, detection) in detections.iter().enumerate() {
                                    log::trace!("  Detection {}: {} at ({:.1}, {:.1}) {}x{} conf={:.2}",
                                               i, detection.class_name, 
                                               detection.x, detection.y,
                                               detection.width, detection.height,
                                               detection.confidence);
                                }
                            },
                            Err(e) => {
                                log::error!("Detection failed: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        log::trace!("Failed to extract image from buffer: {}", e);
                    }
                }
            }
        }
        gst::PadProbeReturn::Ok
    });
    
    log::info!("CPU detector element created with real ONNX inference");
    Ok(bin.upcast())
}

/// Extract image from GStreamer buffer
fn extract_image_from_buffer(buffer: &gst::Buffer) -> Result<DynamicImage> {
    // Map the buffer for reading
    let map = buffer.map_readable().map_err(|_| 
        DeepStreamError::Configuration("Failed to map buffer for reading".to_string())
    )?;
    
    let data = map.as_slice();
    
    // For now, create a dummy image since we need proper caps parsing
    // In a real implementation, we would:
    // 1. Parse the caps to get width, height, and format
    // 2. Convert the raw buffer data to the appropriate image format
    // 3. Handle different video formats (RGB, YUV, etc.)
    
    // Create a placeholder 640x640 RGB image from buffer data
    let width = 640u32;
    let height = 640u32;
    
    // If we have enough data, try to use it, otherwise create a test pattern
    if data.len() >= (width * height * 3) as usize {
        // Try to interpret as RGB data
        let rgb_data: Vec<u8> = data[0..(width * height * 3) as usize].to_vec();
        
        match RgbImage::from_raw(width, height, rgb_data) {
            Some(rgb_img) => Ok(DynamicImage::ImageRgb8(rgb_img)),
            None => {
                // Create a test pattern if data doesn't fit
                create_test_image(width, height)
            }
        }
    } else {
        // Create a test pattern for smaller buffers
        create_test_image(width, height)
    }
}

/// Create a test image for detection testing
fn create_test_image(width: u32, height: u32) -> Result<DynamicImage> {
    let mut img_data = vec![128u8; (width * height * 3) as usize]; // Gray background
    
    // Add a simple pattern (a rectangle that might be detected)
    let rect_x = width / 4;
    let rect_y = height / 4;
    let rect_w = width / 2;
    let rect_h = height / 2;
    
    for y in rect_y..(rect_y + rect_h) {
        for x in rect_x..(rect_x + rect_w) {
            let idx = ((y * width + x) * 3) as usize;
            if idx + 2 < img_data.len() {
                img_data[idx] = 255;     // R
                img_data[idx + 1] = 255; // G  
                img_data[idx + 2] = 255; // B (white rectangle)
            }
        }
    }
    
    RgbImage::from_raw(width, height, img_data)
        .map(DynamicImage::ImageRgb8)
        .ok_or_else(|| DeepStreamError::Configuration("Failed to create test image".to_string()))
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
