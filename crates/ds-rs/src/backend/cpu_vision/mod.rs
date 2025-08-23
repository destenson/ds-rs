pub mod detector;
pub mod tracker;
pub mod elements;

use crate::error::Result;

/// CPU Vision Backend for object detection and tracking
/// Uses ONNX Runtime for inference and pure Rust tracking algorithms
pub struct CpuVisionBackend {
    detector: Option<detector::OnnxDetector>,
    tracker: tracker::CentroidTracker,
}

impl CpuVisionBackend {
    pub fn new() -> Result<Self> {
        Ok(Self {
            detector: None,
            tracker: tracker::CentroidTracker::new(50.0, 30),
        })
    }
    
    pub fn load_model(&mut self, model_path: &str) -> Result<()> {
        self.detector = Some(detector::OnnxDetector::new(model_path)?);
        Ok(())
    }
    
    pub fn detector(&self) -> Option<&detector::OnnxDetector> {
        self.detector.as_ref()
    }
    
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