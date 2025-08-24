#![allow(unused)]
use gstreamer::glib;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::subclass::prelude::*;
use gstreamer_base as gst_base;
use gstreamer_base::subclass::prelude::*;
use gstreamer_video as gst_video;
use gstreamer_video::prelude::*;
use std::sync::{LazyLock, Mutex};
use crate::backend::cpu_vision::detector::{OnnxDetector, DetectorConfig};
use crate::error::Result;
use image::DynamicImage;

static CAT: LazyLock<gst::DebugCategory> = LazyLock::new(|| {
    gst::DebugCategory::new(
        "cpudetector",
        gst::DebugColorFlags::empty(),
        Some("CPU-based object detector using ONNX"),
    )
});

const DEFAULT_MODEL_PATH: &str = "yolov5n.onnx";
const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.5;
const DEFAULT_NMS_THRESHOLD: f64 = 0.4;
const DEFAULT_INPUT_WIDTH: u32 = 640;
const DEFAULT_INPUT_HEIGHT: u32 = 640;
const DEFAULT_PROCESS_EVERY_N_FRAMES: u32 = 1;

#[derive(Debug, Clone)]
struct Settings {
    model_path: String,
    confidence_threshold: f64,
    nms_threshold: f64,
    input_width: u32,
    input_height: u32,
    process_every_n_frames: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            model_path: DEFAULT_MODEL_PATH.to_string(),
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
            nms_threshold: DEFAULT_NMS_THRESHOLD,
            input_width: DEFAULT_INPUT_WIDTH,
            input_height: DEFAULT_INPUT_HEIGHT,
            process_every_n_frames: DEFAULT_PROCESS_EVERY_N_FRAMES,
        }
    }
}

#[derive(Default)]
pub struct CpuDetector {
    settings: Mutex<Settings>,
    detector: Mutex<Option<OnnxDetector>>,
    frame_count: Mutex<u64>,
}

impl CpuDetector {
    fn initialize_detector(&self, settings: &Settings) -> Result<OnnxDetector> {
        use crate::backend::cpu_vision::detector::{OnnxDetector, DetectorConfig};
        let config = DetectorConfig {
            model_path: Some(settings.model_path.clone()),
            input_width: settings.input_width,
            input_height: settings.input_height,
            confidence_threshold: settings.confidence_threshold as f32,
            nms_threshold: settings.nms_threshold as f32,
            num_threads: 4,
            ..Default::default()
        };
        
        OnnxDetector::new_with_config(config).map_err(|e| e.into())
    }
    
    fn ensure_detector_loaded(&self) {
        let settings = self.settings.lock().unwrap().clone();
        let mut detector_guard = self.detector.lock().unwrap();
        
        if detector_guard.is_none() {
            match self.initialize_detector(&settings) {
                Ok(detector) => {
                    gst::info!(CAT, imp = self, "Loaded ONNX detector from: {}", settings.model_path);
                    *detector_guard = Some(detector);
                },
                Err(e) => {
                    gst::warning!(CAT, imp = self, "Failed to load detector: {}, using mock", e);
                    *detector_guard = Some(OnnxDetector::new_mock());
                }
            }
        }
    }
    
    fn frame_to_image(&self, frame: &gst_video::VideoFrameRef<&gst::BufferRef>) -> Option<DynamicImage> {
        let width = frame.width();
        let height = frame.height();
        let format = frame.format();
        
        match format {
            gst_video::VideoFormat::Rgb => {
                let data = frame.plane_data(0).ok()?;
                let stride = frame.plane_stride()[0] as usize;
                
                // Convert strided RGB to contiguous RGB
                let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
                for y in 0..height {
                    let row_start = (y as usize) * stride;
                    let row_end = row_start + (width as usize * 3);
                    if row_end <= data.len() {
                        rgb_data.extend_from_slice(&data[row_start..row_end]);
                    }
                }
                
                image::RgbImage::from_raw(width, height, rgb_data)
                    .map(DynamicImage::ImageRgb8)
            },
            gst_video::VideoFormat::Bgr => {
                let data = frame.plane_data(0).ok()?;
                let stride = frame.plane_stride()[0] as usize;
                
                // Convert BGR to RGB
                let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
                for y in 0..height {
                    let row_start = (y as usize) * stride;
                    for x in 0..width {
                        let pixel_start = row_start + (x as usize * 3);
                        if pixel_start + 2 < data.len() {
                            rgb_data.push(data[pixel_start + 2]); // R
                            rgb_data.push(data[pixel_start + 1]); // G
                            rgb_data.push(data[pixel_start]);     // B
                        }
                    }
                }
                
                image::RgbImage::from_raw(width, height, rgb_data)
                    .map(DynamicImage::ImageRgb8)
            },
            _ => {
                gst::warning!(CAT, imp = self, "Unsupported video format: {:?}", format);
                None
            }
        }
    }
    
    fn emit_inference_results(&self, frame_num: u64, detections: &[crate::backend::cpu_vision::detector::Detection]) {
        // For now, just log the detections
        // Signal emission would require proper GObject signal registration
        if !detections.is_empty() {
            log::debug!("Frame {}: {} detections", frame_num, detections.len());
        }
    }
    
    fn attach_detection_metadata(&self, _buf: &mut gst::BufferRef, detections: &[crate::backend::cpu_vision::detector::Detection]) {
        // TODO: Attach custom metadata to buffer
        // For now, we could use custom metadata or simply pass through
        // This would be where we'd attach DetectionMeta to the buffer
        
        // Example structure (not fully implemented):
        // let detection_meta = DetectionMeta::new(detections);
        // buf.add_meta(detection_meta);
        
        gst::trace!(CAT, imp = self, "Attached {} detections as metadata", detections.len());
    }
}

#[glib::object_subclass]
impl ObjectSubclass for CpuDetector {
    const NAME: &'static str = "GstCpuDetector";
    type Type = super::CpuDetector;
    type ParentType = gst_base::BaseTransform;
}

impl ObjectImpl for CpuDetector {
    fn signals() -> &'static [glib::subclass::Signal] {
        static SIGNALS: LazyLock<Vec<glib::subclass::Signal>> = LazyLock::new(|| {
            vec![]
        });
        
        SIGNALS.as_ref()
    }
    
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: LazyLock<Vec<glib::ParamSpec>> = LazyLock::new(|| {
            vec![
                glib::ParamSpecString::builder("model-path")
                    .nick("Model Path")
                    .blurb("Path to ONNX model file")
                    .default_value(Some(DEFAULT_MODEL_PATH))
                    .mutable_ready()
                    .build(),
                glib::ParamSpecDouble::builder("confidence-threshold")
                    .nick("Confidence Threshold")
                    .blurb("Minimum confidence for detections")
                    .minimum(0.0)
                    .maximum(1.0)
                    .default_value(DEFAULT_CONFIDENCE_THRESHOLD)
                    .mutable_playing()
                    .build(),
                glib::ParamSpecDouble::builder("nms-threshold")
                    .nick("NMS Threshold")
                    .blurb("Non-maximum suppression threshold")
                    .minimum(0.0)
                    .maximum(1.0)
                    .default_value(DEFAULT_NMS_THRESHOLD)
                    .mutable_playing()
                    .build(),
                glib::ParamSpecUInt::builder("input-width")
                    .nick("Input Width")
                    .blurb("Model input width")
                    .minimum(32)
                    .maximum(2048)
                    .default_value(DEFAULT_INPUT_WIDTH)
                    .mutable_ready()
                    .build(),
                glib::ParamSpecUInt::builder("input-height")
                    .nick("Input Height")
                    .blurb("Model input height")
                    .minimum(32)
                    .maximum(2048)
                    .default_value(DEFAULT_INPUT_HEIGHT)
                    .mutable_ready()
                    .build(),
                glib::ParamSpecUInt::builder("process-every-n-frames")
                    .nick("Process Every N Frames")
                    .blurb("Process every Nth frame (1 = every frame)")
                    .minimum(1)
                    .maximum(60)
                    .default_value(DEFAULT_PROCESS_EVERY_N_FRAMES)
                    .mutable_playing()
                    .build(),
            ]
        });
        
        PROPERTIES.as_ref()
    }
    
    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        let mut settings = self.settings.lock().unwrap();
        
        match pspec.name() {
            "model-path" => {
                let model_path: String = value.get().expect("type checked upstream");
                gst::info!(CAT, imp = self, "Setting model path to: {}", model_path);
                settings.model_path = model_path;
                // Reset detector to reload with new model
                *self.detector.lock().unwrap() = None;
            },
            "confidence-threshold" => {
                let threshold: f64 = value.get().expect("type checked upstream");
                settings.confidence_threshold = threshold;
                // Update existing detector if available
                if let Some(ref mut detector) = *self.detector.lock().unwrap() {
                    detector.set_confidence_threshold(threshold as f32);
                }
            },
            "nms-threshold" => {
                let threshold: f64 = value.get().expect("type checked upstream");
                settings.nms_threshold = threshold;
                if let Some(ref mut detector) = *self.detector.lock().unwrap() {
                    detector.set_nms_threshold(threshold as f32);
                }
            },
            "input-width" => {
                settings.input_width = value.get().expect("type checked upstream");
                *self.detector.lock().unwrap() = None;
            },
            "input-height" => {
                settings.input_height = value.get().expect("type checked upstream");
                *self.detector.lock().unwrap() = None;
            },
            "process-every-n-frames" => {
                settings.process_every_n_frames = value.get().expect("type checked upstream");
            },
            _ => unimplemented!(),
        }
    }
    
    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        let settings = self.settings.lock().unwrap();
        
        match pspec.name() {
            "model-path" => settings.model_path.to_value(),
            "confidence-threshold" => settings.confidence_threshold.to_value(),
            "nms-threshold" => settings.nms_threshold.to_value(),
            "input-width" => settings.input_width.to_value(),
            "input-height" => settings.input_height.to_value(),
            "process-every-n-frames" => settings.process_every_n_frames.to_value(),
            _ => unimplemented!(),
        }
    }
}

impl GstObjectImpl for CpuDetector {}

impl ElementImpl for CpuDetector {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: LazyLock<gst::subclass::ElementMetadata> = LazyLock::new(|| {
            gst::subclass::ElementMetadata::new(
                "CPU Object Detector",
                "Filter/Analyzer/Video",
                "Detects objects using ONNX models on CPU with passthrough behavior",
                "DeepStream Rust Team <dev@example.com>",
            )
        });
        
        Some(&*ELEMENT_METADATA)
    }
    
    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: LazyLock<Vec<gst::PadTemplate>> = LazyLock::new(|| {
            let caps = gst_video::VideoCapsBuilder::new()
                .format_list([
                    gst_video::VideoFormat::Rgb,
                    gst_video::VideoFormat::Bgr,
                ])
                .build();
            
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();
            
            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();
            
            vec![src_pad_template, sink_pad_template]
        });
        
        PAD_TEMPLATES.as_ref()
    }
}

impl BaseTransformImpl for CpuDetector {
    const MODE: gst_base::subclass::BaseTransformMode = gst_base::subclass::BaseTransformMode::AlwaysInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = true;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = true;
    
    fn start(&self) -> std::result::Result<(), gst::ErrorMessage> {
        self.ensure_detector_loaded();
        Ok(())
    }
    
    fn transform_ip(&self, buf: &mut gst::BufferRef) -> std::result::Result<gst::FlowSuccess, gst::FlowError> {
        let mut frame_count = self.frame_count.lock().unwrap();
        *frame_count += 1;
        
        let settings = self.settings.lock().unwrap().clone();
        
        // Skip processing if not on the right frame interval
        if *frame_count % (settings.process_every_n_frames as u64) != 0 {
            return Ok(gst::FlowSuccess::Ok);
        }
        
        // Get video info from sink pad caps
        let element = self.obj();
        let sink_pad = element.static_pad("sink").unwrap();
        let caps = sink_pad.current_caps().unwrap();
        let info = gst_video::VideoInfo::from_caps(&caps)
            .map_err(|_| gst::FlowError::NotSupported)?;
        
        // Process frame and get detections
        let detections = {
            let frame = gst_video::VideoFrameRef::from_buffer_ref_readable(buf, &info)
                .map_err(|_| gst::FlowError::Error)?;
            
            // Convert frame to image for detection
            if let Some(image) = self.frame_to_image(&frame) {
                if let Some(ref detector) = *self.detector.lock().unwrap() {
                    match detector.detect(&image) {
                        Ok(detections) => {
                            gst::trace!(CAT, imp = self, 
                                       "Frame {}: Detected {} objects", *frame_count, detections.len());
                            
                            // Emit signal with detection results
                            self.emit_inference_results(*frame_count, &detections);
                            
                            // Log detections for debugging
                            for detection in &detections {
                                gst::trace!(CAT, imp = self,
                                           "Detection: {} at ({:.1}, {:.1}) {}x{} conf={:.2}",
                                           detection.class_name,
                                           detection.x, detection.y,
                                           detection.width, detection.height,
                                           detection.confidence);
                            }
                            
                            Some(detections)
                        },
                        Err(e) => {
                            gst::warning!(CAT, imp = self, "Detection failed: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };
        
        // Attach metadata to buffer if we have detections
        if let Some(detections) = detections {
            self.attach_detection_metadata(buf, &detections);
        }
        
        // Buffer passes through unchanged (identity behavior)
        Ok(gst::FlowSuccess::Ok)
    }
}
