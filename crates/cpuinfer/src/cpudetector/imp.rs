use gstreamer::glib;
use gstreamer::prelude::*;
use gstreamer::subclass::prelude::*;
use gstreamer_base::subclass::prelude::*;
use gstreamer_video as gst_video;
use gstreamer_video::VideoFrameExt;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::detector::{OnnxDetector, DetectorConfig};
use image::DynamicImage;

static CAT: Lazy<gstreamer::DebugCategory> = Lazy::new(|| {
    gstreamer::DebugCategory::new(
        "cpudetector",
        gstreamer::DebugColorFlags::empty(),
        Some("CPU-based object detector using ONNX"),
    )
});

const DEFAULT_MODEL_PATH: &str = "yolov5n.onnx";
const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.5;
const DEFAULT_NMS_THRESHOLD: f64 = 0.4;
const DEFAULT_INPUT_WIDTH: u32 = 640;
const DEFAULT_INPUT_HEIGHT: u32 = 640;
const DEFAULT_PROCESS_EVERY_N_FRAMES: u32 = 1;
const DEFAULT_BATCH_SIZE: u32 = 2;
const DEFAULT_UNIQUE_ID: u32 = 0;
const DEFAULT_PROCESS_MODE: u32 = 1;  // Primary mode
const DEFAULT_OUTPUT_TENSOR_META: bool = false;

#[derive(Debug, Clone)]
struct Settings {
    model_path: String,
    config_file_path: Option<String>,  // nvinfer compatibility
    confidence_threshold: f64,
    nms_threshold: f64,
    input_width: u32,
    input_height: u32,
    process_every_n_frames: u32,
    batch_size: u32,  // nvinfer compatibility
    unique_id: u32,  // nvinfer compatibility
    process_mode: u32,  // nvinfer compatibility (1=primary, 2=secondary)
    output_tensor_meta: bool,  // nvinfer compatibility
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            model_path: DEFAULT_MODEL_PATH.to_string(),
            config_file_path: None,
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
            nms_threshold: DEFAULT_NMS_THRESHOLD,
            input_width: DEFAULT_INPUT_WIDTH,
            input_height: DEFAULT_INPUT_HEIGHT,
            process_every_n_frames: DEFAULT_PROCESS_EVERY_N_FRAMES,
            batch_size: DEFAULT_BATCH_SIZE,
            unique_id: DEFAULT_UNIQUE_ID,
            process_mode: DEFAULT_PROCESS_MODE,
            output_tensor_meta: DEFAULT_OUTPUT_TENSOR_META,
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
    fn initialize_detector(&self, settings: &Settings) -> Result<OnnxDetector, String> {
        let config = DetectorConfig {
            model_path: Some(settings.model_path.clone()),
            input_width: settings.input_width,
            input_height: settings.input_height,
            confidence_threshold: settings.confidence_threshold as f32,
            nms_threshold: settings.nms_threshold as f32,
            num_threads: 4,
            ..Default::default()
        };
        
        OnnxDetector::new_with_config(config)
            .map_err(|e| format!("Failed to create detector: {}", e))
    }
    
    fn ensure_detector_loaded(&self) {
        let settings = self.settings.lock().unwrap().clone();
        let mut detector_guard = self.detector.lock().unwrap();
        
        if detector_guard.is_none() {
            match self.initialize_detector(&settings) {
                Ok(detector) => {
                    gstreamer::info!(CAT, imp = self, "Loaded ONNX detector from: {}", settings.model_path);
                    *detector_guard = Some(detector);
                },
                #[cfg(test)]
                Err(e) => {
                    gstreamer::warning!(CAT, imp = self, "Failed to load detector: {}, using mock", e);
                    *detector_guard = Some(OnnxDetector::new_mock());
                }
                #[cfg(not(test))]
                Err(e) => {
                    gstreamer::warning!(CAT, imp = self, "Failed to load detector: {}", e);
                }
            }
        }
    }
    
    fn frame_to_image(&self, frame: &gst_video::VideoFrameRef<&gstreamer::BufferRef>) -> Option<DynamicImage> {
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
                gstreamer::warning!(CAT, imp = self, "Unsupported video format: {:?}", format);
                None
            }
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for CpuDetector {
    const NAME: &'static str = "GstCpuDetector";
    type Type = super::CpuDetector;
    type ParentType = gstreamer_base::BaseTransform;
}

impl ObjectImpl for CpuDetector {
    fn signals() -> &'static [glib::subclass::Signal] {
        static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
            vec![
                glib::subclass::Signal::builder("inference-done")
                    .param_types([
                        u64::static_type(),    // frame number
                        u32::static_type(),    // detection count
                    ])
                    .build(),
            ]
        });
        
        SIGNALS.as_ref()
    }
    
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
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
                // nvinfer-compatible properties
                glib::ParamSpecString::builder("config-file-path")
                    .nick("Config File Path")
                    .blurb("Path to configuration file (nvinfer compatibility)")
                    .mutable_ready()
                    .build(),
                glib::ParamSpecUInt::builder("batch-size")
                    .nick("Batch Size")
                    .blurb("Number of frames to batch for processing")
                    .minimum(1)
                    .maximum(32)
                    .default_value(DEFAULT_BATCH_SIZE)
                    .mutable_ready()
                    .build(),
                glib::ParamSpecUInt::builder("unique-id")
                    .nick("Unique ID")
                    .blurb("Unique identifier for this detector instance")
                    .minimum(0)
                    .maximum(u32::MAX)
                    .default_value(DEFAULT_UNIQUE_ID)
                    .mutable_ready()
                    .build(),
                glib::ParamSpecUInt::builder("process-mode")
                    .nick("Process Mode")
                    .blurb("Process mode: 1=Primary (full frame), 2=Secondary (crops)")
                    .minimum(1)
                    .maximum(2)
                    .default_value(DEFAULT_PROCESS_MODE)
                    .mutable_ready()
                    .build(),
                glib::ParamSpecBoolean::builder("output-tensor-meta")
                    .nick("Output Tensor Meta")
                    .blurb("Output raw tensor metadata")
                    .default_value(DEFAULT_OUTPUT_TENSOR_META)
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
                gstreamer::info!(CAT, imp = self, "Setting model path to: {}", model_path);
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
            "config-file-path" => {
                let config_path: Option<String> = value.get().ok();
                settings.config_file_path = config_path.clone();
                if let Some(path) = config_path {
                    gstreamer::info!(CAT, imp = self, "Loading config file: {}", path);
                    
                    // Parse config file and update settings
                    match crate::config::parse_config_file(&path) {
                        Ok(config) => {
                            // Apply settings from config file
                            if let Some(onnx_file) = config.onnx_file {
                                settings.model_path = onnx_file;
                                gstreamer::info!(CAT, imp = self, "Model path from config: {}", settings.model_path);
                            }
                            settings.batch_size = config.batch_size;
                            settings.unique_id = config.unique_id;
                            settings.process_mode = config.process_mode;
                            settings.confidence_threshold = config.pre_cluster_threshold as f64;
                            settings.nms_threshold = config.nms_iou_threshold as f64;
                            
                            // Reset detector to reload with new settings
                            *self.detector.lock().unwrap() = None;
                            
                            gstreamer::info!(CAT, imp = self, "Config loaded successfully");
                        }
                        Err(e) => {
                            gstreamer::error!(CAT, imp = self, "Failed to parse config file: {}", e);
                        }
                    }
                }
            },
            "batch-size" => {
                settings.batch_size = value.get().expect("type checked upstream");
                gstreamer::info!(CAT, imp = self, "Batch size set to: {}", settings.batch_size);
            },
            "unique-id" => {
                settings.unique_id = value.get().expect("type checked upstream");
            },
            "process-mode" => {
                settings.process_mode = value.get().expect("type checked upstream");
                gstreamer::info!(CAT, imp = self, "Process mode set to: {} ({})", 
                    settings.process_mode,
                    if settings.process_mode == 1 { "primary" } else { "secondary" }
                );
            },
            "output-tensor-meta" => {
                settings.output_tensor_meta = value.get().expect("type checked upstream");
            },
            _ => {
                gstreamer::warning!(CAT, imp = self, "Unknown property '{}' in set_property", pspec.name());
            }
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
            "config-file-path" => settings.config_file_path.to_value(),
            "batch-size" => settings.batch_size.to_value(),
            "unique-id" => settings.unique_id.to_value(),
            "process-mode" => settings.process_mode.to_value(),
            "output-tensor-meta" => settings.output_tensor_meta.to_value(),
            _ => {
                gstreamer::warning!(CAT, imp = self, "Unknown property '{}' in property getter", pspec.name());
                // Return a default value to avoid crashes
                glib::Value::from(&0u32)
            }
        }
    }
}

impl GstObjectImpl for CpuDetector {}

impl ElementImpl for CpuDetector {
    fn metadata() -> Option<&'static gstreamer::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gstreamer::subclass::ElementMetadata> = Lazy::new(|| {
            gstreamer::subclass::ElementMetadata::new(
                "CPU Object Detector",
                "Filter/Analyzer/Video",
                "Detects objects using ONNX models on CPU with passthrough behavior",
                "DeepStream Rust Team <dev@example.com>",
            )
        });
        
        Some(&*ELEMENT_METADATA)
    }
    
    fn pad_templates() -> &'static [gstreamer::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gstreamer::PadTemplate>> = Lazy::new(|| {
            let caps = gst_video::VideoCapsBuilder::new()
                .format_list([
                    gst_video::VideoFormat::Rgb,
                    gst_video::VideoFormat::Bgr,
                    gst_video::VideoFormat::Rgba,
                    gst_video::VideoFormat::Bgra,
                ])
                .build();
            
            let src_pad_template = gstreamer::PadTemplate::new(
                "src",
                gstreamer::PadDirection::Src,
                gstreamer::PadPresence::Always,
                &caps,
            )
            .unwrap();
            
            let sink_pad_template = gstreamer::PadTemplate::new(
                "sink",
                gstreamer::PadDirection::Sink,
                gstreamer::PadPresence::Always,
                &caps,
            )
            .unwrap();
            
            vec![src_pad_template, sink_pad_template]
        });
        
        PAD_TEMPLATES.as_ref()
    }
}

impl BaseTransformImpl for CpuDetector {
    const MODE: gstreamer_base::subclass::BaseTransformMode = 
        gstreamer_base::subclass::BaseTransformMode::AlwaysInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = true;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = true;
    
    fn start(&self) -> Result<(), gstreamer::ErrorMessage> {
        self.ensure_detector_loaded();
        Ok(())
    }
    
    fn transform_ip(&self, buf: &mut gstreamer::BufferRef) -> Result<gstreamer::FlowSuccess, gstreamer::FlowError> {
        let mut frame_count = self.frame_count.lock().unwrap();
        *frame_count += 1;
        
        let settings = self.settings.lock().unwrap().clone();
        
        // Skip processing if not on the right frame interval
        if *frame_count % (settings.process_every_n_frames as u64) != 0 {
            return Ok(gstreamer::FlowSuccess::Ok);
        }
        
        // Get video info from sink pad caps
        let element = self.obj();
        let sink_pad = element.static_pad("sink").unwrap();
        let caps = sink_pad.current_caps().ok_or(gstreamer::FlowError::NotNegotiated)?;
        let info = gst_video::VideoInfo::from_caps(&caps)
            .map_err(|_| gstreamer::FlowError::NotSupported)?;
        
        // Map buffer for reading (we don't modify the video data)
        let _map = buf.map_readable().map_err(|_| gstreamer::FlowError::Error)?;
        let frame = gst_video::VideoFrameRef::from_buffer_ref_readable(buf, &info)
            .map_err(|_| gstreamer::FlowError::Error)?;
        
        // Convert frame to image for detection
        if let Some(image) = self.frame_to_image(&frame) {
            if let Some(ref detector) = *self.detector.lock().unwrap() {
                match detector.detect(&image) {
                    Ok(detections) => {
                        let detection_count = detections.len() as u32;
                        
                        gstreamer::trace!(CAT, imp = self, 
                                   "Frame {}: Detected {} objects", *frame_count, detection_count);
                        
                        // Emit signal with detection results
                        element.emit_by_name::<()>(
                            "inference-done",
                            &[&(*frame_count as u64), &detection_count],
                        );
                        
                        // Log detections for debugging
                        for detection in &detections {
                            gstreamer::trace!(CAT, imp = self,
                                       "Detection: {} at ({:.1}, {:.1}) {}x{} conf={:.2}",
                                       detection.class_name,
                                       detection.x, detection.y,
                                       detection.width, detection.height,
                                       detection.confidence);
                        }
                    },
                    Err(e) => {
                        gstreamer::warning!(CAT, imp = self, "Detection failed: {}", e);
                    }
                }
            }
        }
        
        // Buffer passes through unchanged (identity behavior)
        Ok(gstreamer::FlowSuccess::Ok)
    }
}
