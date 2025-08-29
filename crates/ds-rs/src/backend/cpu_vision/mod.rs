pub mod cpudetector;
pub mod elements;
pub mod metadata;
#[cfg(feature = "nalgebra")]
pub mod tracker;

// Re-export detector types from cpuinfer crate
pub use gstcpuinfer::detector::{
    Detection, DetectorConfig, DetectorError, OnnxDetector, YoloVersion,
};

use crate::error::Result;

/// CPU Vision Backend for object detection and tracking
/// Uses ONNX Runtime for inference and pure Rust tracking algorithms
pub struct CpuVisionBackend {
    detector: Option<OnnxDetector>,
    #[cfg(feature = "nalgebra")]
    tracker: tracker::CentroidTracker,
}

impl CpuVisionBackend {
    pub fn new() -> Result<Self> {
        Ok(Self {
            detector: None,
            #[cfg(feature = "nalgebra")]
            tracker: tracker::CentroidTracker::new(50.0, 30),
        })
    }

    pub fn load_model(&mut self, model_path: &str) -> Result<()> {
        self.detector = Some(OnnxDetector::new(model_path)?);
        Ok(())
    }

    pub fn detector(&self) -> Option<&OnnxDetector> {
        self.detector.as_ref()
    }

    #[cfg(feature = "nalgebra")]
    pub fn tracker_mut(&mut self) -> &mut tracker::CentroidTracker {
        &mut self.tracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_vision_backend_creation() {
        let backend = CpuVisionBackend::new().unwrap();
        assert!(backend.detector().is_none());
    }
}
