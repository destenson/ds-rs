#![allow(unused)]
use super::metadata::DetectionMeta;
#[cfg(feature = "nalgebra")]
use super::tracker::CentroidTracker;
use crate::error::{DeepStreamError, Result};
use crate::rendering::metadata_bridge::MetadataBridge;
#[cfg(feature = "cairo-rs")]
use cairo;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_video as gst_video;
use image::{DynamicImage, RgbImage};
use std::sync::{Arc, Mutex};

/// Create a CPU detector element that performs object detection
/// This creates a bin containing the cpuinfer element from the cpuinfer plugin
pub fn create_cpu_detector(name: Option<&str>, model_path: Option<&str>) -> Result<gst::Element> {
    let bin = gst::Bin::builder()
        .name(name.unwrap_or("cpu-detector"))
        .build();

    // Create cpuinfer element from the cpuinfer plugin
    let detector = gst::ElementFactory::make("cpuinfer")
        .name("detector")
        .build()
        .map_err(|_| DeepStreamError::ElementCreation {
            element: "cpuinfer".to_string(),
        })?;

    // Configure the detector with model path if provided
    if let Some(path) = model_path {
        detector.set_property("model-path", path);
    }

    // Create queue for buffering - make it leaky to prevent blocking
    let queue = gst::ElementFactory::make("queue")
        .name("detector-queue")
        .property("max-size-buffers", 1u32) // Small buffer
        .property("max-size-bytes", 0u32) // No byte limit
        .property("max-size-time", 0u64) // No time limit
        .property_from_str("leaky", "upstream") // Leak downstream (drop old buffers)
        .build()
        .map_err(|_| DeepStreamError::ElementCreation {
            element: "queue".to_string(),
        })?;

    bin.add_many([&queue, &detector])?;
    queue.link(&detector)?;

    // Create ghost pads
    let sink_pad = queue.static_pad("sink").unwrap();
    let src_pad = detector.static_pad("src").unwrap();

    bin.add_pad(&gst::GhostPad::with_target(&sink_pad)?)?;
    bin.add_pad(&gst::GhostPad::with_target(&src_pad)?)?;

    log::info!("CPU detector element created using cpuinfer plugin");
    Ok(bin.upcast())
}

/// Extract image from GStreamer buffer
fn extract_image_from_buffer(buffer: &gst::Buffer) -> Result<DynamicImage> {
    // Map the buffer for reading
    let map = buffer.map_readable().map_err(|_| {
        DeepStreamError::Configuration("Failed to map buffer for reading".to_string())
    })?;

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
                img_data[idx] = 255; // R
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
pub fn create_cpu_osd(
    name: Option<&str>,
    metadata_bridge: Option<Arc<Mutex<MetadataBridge>>>,
) -> Result<gst::Element> {
    let bin = gst::Bin::builder().name(name.unwrap_or("cpu-osd")).build();

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
        log::info!("CPU OSD using Cairo for rendering");

        if let Some(bridge) = metadata_bridge {
            #[cfg(feature = "cairo-rs")]
            {
                // Track video dimensions for coordinate transformation
                let video_info = Arc::new(Mutex::new(None::<(u32, u32)>));
                let video_info_clone = video_info.clone();

                // Connect caps-changed signal to track video dimensions
                overlay.connect("caps-changed", false, move |args| {
                    if let Ok(caps) = args[1].get::<gst::Caps>() {
                        if let Ok(video_info_from_caps) = gst_video::VideoInfo::from_caps(&caps) {
                            let width = video_info_from_caps.width();
                            let height = video_info_from_caps.height();
                            *video_info_clone.lock().unwrap() = Some((width, height));
                            log::debug!("Cairo overlay video dimensions: {}x{}", width, height);
                        }
                    }
                    None
                });

                // Connect draw signal for rendering bounding boxes
                #[cfg(feature = "cairo-rs")]
                overlay.connect("draw", false, move |args| {
                    // Get the cairo context and timestamp
                    let cr = args[1].get::<cairo::Context>().ok()?;
                    let timestamp = args[2].get::<gst::ClockTime>().ok()?;

                    // Get current video dimensions
                    let (width, height) = match *video_info.lock().unwrap() {
                        Some(dims) => dims,
                        None => return None, // Skip if we don't know dimensions yet
                    };

                    // Get detections from metadata bridge
                    let detections = bridge.lock().unwrap().get_frame_metadata(timestamp);

                    if let Some(objects) = detections {
                        if !objects.is_empty() {
                            log::debug!(
                                "Drawing {} detections at timestamp {:?}",
                                objects.len(),
                                timestamp
                            );
                        }

                        // Set drawing properties
                        cr.set_line_width(3.0);

                        for obj in objects {
                            // Get detection bounding box
                            let detection_bbox = &obj.detector_bbox_info;
                            let confidence = obj.confidence;
                            let class_name = obj.class_name();
                            let class_id = obj.class_id;

                            // Log detection details for debugging
                            log::trace!(
                                "Drawing box for {}: ({:.1}, {:.1}) {}x{} conf={:.2}",
                                class_name,
                                detection_bbox.left,
                                detection_bbox.top,
                                detection_bbox.width,
                                detection_bbox.height,
                                confidence
                            );

                            // Convert normalized coordinates to pixel coordinates if needed
                            // Detection coordinates might be in pixels already (0-width, 0-height)
                            // or normalized (0-1). Check and convert if needed.
                            let (x, y, w, h) = if detection_bbox.left <= 1.0
                                && detection_bbox.top <= 1.0
                                && detection_bbox.width <= 1.0
                                && detection_bbox.height <= 1.0
                            {
                                // Normalized coordinates - convert to pixels
                                (
                                    detection_bbox.left * width as f32,
                                    detection_bbox.top * height as f32,
                                    detection_bbox.width * width as f32,
                                    detection_bbox.height * height as f32,
                                )
                            } else {
                                // Already in pixels
                                (
                                    detection_bbox.left,
                                    detection_bbox.top,
                                    detection_bbox.width,
                                    detection_bbox.height,
                                )
                            };

                            // Set color based on class or confidence
                            // Use different colors for different classes
                            let (r, g, b) = match class_id % 6 {
                                0 => (1.0, 0.0, 0.0), // Red
                                1 => (0.0, 1.0, 0.0), // Green
                                2 => (0.0, 0.0, 1.0), // Blue
                                3 => (1.0, 1.0, 0.0), // Yellow
                                4 => (1.0, 0.0, 1.0), // Magenta
                                _ => (0.0, 1.0, 1.0), // Cyan
                            };

                            // Set color with alpha based on confidence
                            cr.set_source_rgba(r, g, b, 0.8);

                            // Draw the bounding box
                            cr.rectangle(x as f64, y as f64, w as f64, h as f64);
                            cr.stroke().unwrap_or_default();

                            // Draw the label background
                            let label = format!("{}: {:.0}%", class_name, confidence * 100.0);
                            let label_height = 20.0;
                            let label_padding = 4.0;

                            // Create text extents to measure label size
                            cr.set_font_size(14.0);
                            // Get text extents or use default width
                            let text_width = cr
                                .text_extents(&label)
                                .map(|te| te.width())
                                .unwrap_or(100.0);
                            let label_width = text_width + label_padding * 2.0;

                            // Draw label background
                            cr.set_source_rgba(r, g, b, 0.9);
                            cr.rectangle(
                                x as f64,
                                (y as f64) - label_height,
                                label_width,
                                label_height,
                            );
                            cr.fill().unwrap_or_default();

                            // Draw label text
                            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0); // White text
                            cr.move_to((x as f64) + label_padding, (y as f64) - label_padding);
                            cr.show_text(&label).unwrap_or_default();
                        }
                    }

                    None
                });
            }
            #[cfg(not(feature = "cairo-rs"))]
            {
                log::warn!("Cairo rendering disabled - cairo-rs feature not enabled");
            }
        } else {
            log::warn!("CPU OSD created without metadata bridge - no detections will be rendered");
        }
    } else {
        log::info!("CPU OSD using text overlay fallback");
    }

    Ok(bin.upcast())
}

/// Connect a metadata bridge to an existing CPU OSD element
/// This allows connecting the bridge after the element has been created
pub fn connect_metadata_bridge_to_cpu_osd(
    osd_bin: &gst::Element,
    metadata_bridge: Arc<Mutex<MetadataBridge>>,
) -> Result<()> {
    // Check if this is actually a bin
    let bin = osd_bin
        .downcast_ref::<gst::Bin>()
        .ok_or_else(|| DeepStreamError::Configuration("OSD element is not a bin".to_string()))?;

    // Get the cairooverlay element from the bin
    let overlay = bin.by_name("osd-overlay").ok_or_else(|| {
        DeepStreamError::Configuration("Could not find osd-overlay element in bin".to_string())
    })?;

    // Only proceed if it's actually a cairooverlay element
    if overlay.type_().name() != "GstCairoOverlay" {
        log::warn!("OSD overlay is not Cairo-based, cannot connect metadata bridge");
        return Ok(());
    }

    log::info!("Connecting metadata bridge to CPU OSD Cairo overlay");

    #[cfg(feature = "cairo-rs")]
    {
        // Track video dimensions for coordinate transformation
        let video_info = Arc::new(Mutex::new(None::<(u32, u32)>));
        let video_info_clone = video_info.clone();

        // Connect caps-changed signal to track video dimensions
        overlay.connect("caps-changed", false, move |args| {
            if let Ok(caps) = args[1].get::<gst::Caps>() {
                if let Ok(video_info_from_caps) = gst_video::VideoInfo::from_caps(&caps) {
                    let width = video_info_from_caps.width();
                    let height = video_info_from_caps.height();
                    *video_info_clone.lock().unwrap() = Some((width, height));
                    log::debug!("Cairo overlay video dimensions: {}x{}", width, height);
                }
            }
            None
        });

        // Connect draw signal for rendering bounding boxes
        #[cfg(feature = "cairo-rs")]
        overlay.connect("draw", false, move |args| {
            // Get the cairo context and timestamp
            let cr = args[1].get::<cairo::Context>().ok()?;
            let timestamp = args[2].get::<gst::ClockTime>().ok()?;

            // Get current video dimensions
            let (width, height) = match *video_info.lock().unwrap() {
                Some(dims) => dims,
                None => return None, // Skip if we don't know dimensions yet
            };

            // Get detections from metadata bridge
            let detections = metadata_bridge
                .lock()
                .unwrap()
                .get_frame_metadata(timestamp);

            if let Some(objects) = detections {
                if !objects.is_empty() {
                    log::info!(
                        "Drawing {} detections at timestamp {:?}",
                        objects.len(),
                        timestamp
                    );
                }

                // Set drawing properties
                cr.set_line_width(3.0);

                for obj in objects {
                    // Get detection bounding box
                    let detection_bbox = &obj.detector_bbox_info;
                    let confidence = obj.confidence;
                    let class_name = obj.class_name();
                    let class_id = obj.class_id;

                    // Log detection details for debugging
                    log::debug!(
                        "Drawing box for {}: ({:.1}, {:.1}) {}x{} conf={:.2}",
                        class_name,
                        detection_bbox.left,
                        detection_bbox.top,
                        detection_bbox.width,
                        detection_bbox.height,
                        confidence
                    );

                    // Convert normalized coordinates to pixel coordinates if needed
                    // Detection coordinates might be in pixels already (0-width, 0-height)
                    // or normalized (0-1). Check and convert if needed.
                    let (x, y, w, h) = if detection_bbox.left <= 1.0
                        && detection_bbox.top <= 1.0
                        && detection_bbox.width <= 1.0
                        && detection_bbox.height <= 1.0
                    {
                        // Normalized coordinates - convert to pixels
                        (
                            detection_bbox.left * width as f32,
                            detection_bbox.top * height as f32,
                            detection_bbox.width * width as f32,
                            detection_bbox.height * height as f32,
                        )
                    } else {
                        // Already in pixels
                        (
                            detection_bbox.left,
                            detection_bbox.top,
                            detection_bbox.width,
                            detection_bbox.height,
                        )
                    };

                    // Set color based on class or confidence
                    // Use different colors for different classes
                    let (r, g, b) = match class_id % 6 {
                        0 => (1.0, 0.0, 0.0), // Red
                        1 => (0.0, 1.0, 0.0), // Green
                        2 => (0.0, 0.0, 1.0), // Blue
                        3 => (1.0, 1.0, 0.0), // Yellow
                        4 => (1.0, 0.0, 1.0), // Magenta
                        _ => (0.0, 1.0, 1.0), // Cyan
                    };

                    // Set color with alpha based on confidence
                    cr.set_source_rgba(r, g, b, 0.8);

                    // Draw the bounding box
                    cr.rectangle(x as f64, y as f64, w as f64, h as f64);
                    cr.stroke().unwrap_or_default();

                    // Draw the label background
                    let label = format!("{}: {:.0}%", class_name, confidence * 100.0);
                    let label_height = 20.0;
                    let label_padding = 4.0;

                    // Create text extents to measure label size
                    cr.set_font_size(14.0);
                    // Get text extents or use default width
                    let text_width = cr
                        .text_extents(&label)
                        .map(|te| te.width())
                        .unwrap_or(100.0);
                    let label_width = text_width + label_padding * 2.0;

                    // Draw label background
                    cr.set_source_rgba(r, g, b, 0.9);
                    cr.rectangle(
                        x as f64,
                        (y as f64) - label_height,
                        label_width,
                        label_height,
                    );
                    cr.fill().unwrap_or_default();

                    // Draw label text
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0); // White text
                    cr.move_to((x as f64) + label_padding, (y as f64) - label_padding);
                    cr.show_text(&label).unwrap_or_default();
                }
            }

            None
        });
    }

    #[cfg(not(feature = "cairo-rs"))]
    {
        log::warn!("BOUNDING BOX RENDERING DISABLED - cairo-rs feature not enabled!");
        log::warn!("To enable bounding box visualization, rebuild with:");
        log::warn!("  cargo run --features cairo-rs --example ball_tracking_visualization ...");
        log::warn!("Or add 'cairo-rs' to default features in Cargo.toml");
    }

    Ok(())
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
    // Note: For complete pipeline, metadata_bridge should be passed from pipeline builder
    let osd = create_cpu_osd(Some("osd"), None)?;

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

        let osd = create_cpu_osd(Some("test-osd"), None).unwrap();
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
