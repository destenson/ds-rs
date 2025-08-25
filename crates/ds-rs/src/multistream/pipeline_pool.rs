//! Pool of detection pipelines for concurrent processing

use crate::backend::cpu_vision::detector::{OnnxDetector, DetectorConfig, Detection};
use crate::source::SourceId;
use crate::error::Result;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// State of a detection pipeline
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineState {
    Idle,
    Processing,
    Error(String),
}

/// A single detection pipeline for processing video frames
pub struct DetectionPipeline {
    pub id: usize,
    pub detector: Arc<Mutex<OnnxDetector>>,
    pub state: Arc<RwLock<PipelineState>>,
    pub assigned_source: Option<SourceId>,
    pub last_used: Instant,
    pub frames_processed: u64,
    pub detections_total: u64,
}

impl DetectionPipeline {
    /// Create a new detection pipeline
    pub fn new(id: usize, detector_config: DetectorConfig) -> Result<Self> {
        // Use the existing constructor that takes a model path
        let detector = if let Some(model_path) = &detector_config.model_path {
            OnnxDetector::new(model_path)?
        } else {
            // Create a mock detector if no model path
            OnnxDetector::new("mock_model.onnx")?
        };
        
        Ok(Self {
            id,
            detector: Arc::new(Mutex::new(detector)),
            state: Arc::new(RwLock::new(PipelineState::Idle)),
            assigned_source: None,
            last_used: Instant::now(),
            frames_processed: 0,
            detections_total: 0,
        })
    }
    
    /// Process a frame through the detection pipeline
    pub fn process_frame(&mut self, _frame_data: &[u8], _width: u32, _height: u32) -> Result<Vec<Detection>> {
        *self.state.write().unwrap() = PipelineState::Processing;
        
        // Stub implementation - in real implementation would perform detection
        let detections = vec![];
        
        // Update statistics
        self.frames_processed += 1;
        self.detections_total += detections.len() as u64;
        self.last_used = Instant::now();
        
        *self.state.write().unwrap() = PipelineState::Idle;
        
        Ok(detections)
    }
    
    /// Reset the pipeline for reuse
    pub fn reset(&mut self) {
        self.assigned_source = None;
        self.frames_processed = 0;
        self.detections_total = 0;
        *self.state.write().unwrap() = PipelineState::Idle;
    }
    
    /// Check if pipeline is available
    pub fn is_available(&self) -> bool {
        self.assigned_source.is_none() && 
        *self.state.read().unwrap() == PipelineState::Idle
    }
}

/// Pool of detection pipelines with lifecycle management
pub struct PipelinePool {
    pipelines: Arc<RwLock<Vec<Arc<Mutex<DetectionPipeline>>>>>,
    available_pipelines: Arc<Mutex<VecDeque<usize>>>,
    source_to_pipeline: Arc<RwLock<HashMap<SourceId, usize>>>,
    max_pipelines: usize,
    detector_config: DetectorConfig,
}

impl PipelinePool {
    /// Create a new pipeline pool
    pub fn new(max_pipelines: usize) -> Self {
        let mut pipelines = Vec::new();
        let mut available = VecDeque::new();
        
        // Pre-create initial pipelines
        let initial_count = (max_pipelines / 2).max(1);
        for i in 0..initial_count {
            if let Ok(pipeline) = DetectionPipeline::new(i, DetectorConfig::default()) {
                pipelines.push(Arc::new(Mutex::new(pipeline)));
                available.push_back(i);
            }
        }
        
        Self {
            pipelines: Arc::new(RwLock::new(pipelines)),
            available_pipelines: Arc::new(Mutex::new(available)),
            source_to_pipeline: Arc::new(RwLock::new(HashMap::new())),
            max_pipelines,
            detector_config: DetectorConfig::default(),
        }
    }
    
    /// Set custom detector configuration
    pub fn set_detector_config(&mut self, config: DetectorConfig) {
        self.detector_config = config;
    }
    
    /// Allocate a pipeline for a source
    pub fn allocate_pipeline(&self, source_id: SourceId) -> Result<usize> {
        // Check if already allocated
        if let Some(&pipeline_id) = self.source_to_pipeline.read().unwrap().get(&source_id) {
            return Ok(pipeline_id);
        }
        
        // Try to get an available pipeline
        let mut available = self.available_pipelines.lock().unwrap();
        
        if let Some(pipeline_id) = available.pop_front() {
            // Use existing pipeline
            let pipelines = self.pipelines.read().unwrap();
            if let Some(pipeline) = pipelines.get(pipeline_id) {
                let mut p = pipeline.lock().unwrap();
                p.assigned_source = Some(source_id);
                p.reset();
            }
            
            self.source_to_pipeline.write().unwrap().insert(source_id, pipeline_id);
            return Ok(pipeline_id);
        }
        
        // Create new pipeline if under limit
        let mut pipelines = self.pipelines.write().unwrap();
        if pipelines.len() < self.max_pipelines {
            let pipeline_id = pipelines.len();
            let mut pipeline = DetectionPipeline::new(pipeline_id, self.detector_config.clone())?;
            pipeline.assigned_source = Some(source_id);
            
            pipelines.push(Arc::new(Mutex::new(pipeline)));
            self.source_to_pipeline.write().unwrap().insert(source_id, pipeline_id);
            
            Ok(pipeline_id)
        } else {
            Err(crate::DeepStreamError::ResourceLimit(
                format!("Pipeline pool exhausted, max {} pipelines", self.max_pipelines)
            ).into())
        }
    }
    
    /// Release a pipeline back to the pool
    pub fn release_pipeline(&self, pipeline_id: usize) -> Result<()> {
        let pipelines = self.pipelines.read().unwrap();
        
        if let Some(pipeline) = pipelines.get(pipeline_id) {
            let mut p = pipeline.lock().unwrap();
            
            // Remove source mapping
            if let Some(source_id) = p.assigned_source {
                self.source_to_pipeline.write().unwrap().remove(&source_id);
            }
            
            // Reset and mark as available
            p.reset();
            self.available_pipelines.lock().unwrap().push_back(pipeline_id);
        }
        
        Ok(())
    }
    
    /// Get a pipeline by ID
    pub fn get_pipeline(&self, pipeline_id: usize) -> Option<Arc<Mutex<DetectionPipeline>>> {
        self.pipelines.read().unwrap().get(pipeline_id).cloned()
    }
    
    /// Get pipeline for a specific source
    pub fn get_pipeline_for_source(&self, source_id: SourceId) -> Option<Arc<Mutex<DetectionPipeline>>> {
        if let Some(&pipeline_id) = self.source_to_pipeline.read().unwrap().get(&source_id) {
            self.get_pipeline(pipeline_id)
        } else {
            None
        }
    }
    
    /// Clean up idle pipelines
    pub fn cleanup_idle_pipelines(&self, idle_threshold: Duration) -> usize {
        let mut cleaned = 0;
        let now = Instant::now();
        let pipelines = self.pipelines.read().unwrap();
        
        for pipeline in pipelines.iter() {
            let p = pipeline.lock().unwrap();
            if p.is_available() && now.duration_since(p.last_used) > idle_threshold {
                // Pipeline has been idle too long, could clean up resources
                // For now, just count them
                cleaned += 1;
            }
        }
        
        cleaned
    }
    
    /// Get pool statistics
    pub fn get_stats(&self) -> PipelinePoolStats {
        let pipelines = self.pipelines.read().unwrap();
        let available_count = self.available_pipelines.lock().unwrap().len();
        let total_count = pipelines.len();
        let active_count = total_count - available_count;
        
        let mut total_frames = 0u64;
        let mut total_detections = 0u64;
        
        for pipeline in pipelines.iter() {
            let p = pipeline.lock().unwrap();
            total_frames += p.frames_processed;
            total_detections += p.detections_total;
        }
        
        PipelinePoolStats {
            total_pipelines: total_count,
            active_pipelines: active_count,
            available_pipelines: available_count,
            total_frames_processed: total_frames,
            total_detections: total_detections,
        }
    }
}

/// Statistics for the pipeline pool
#[derive(Debug, Clone)]
pub struct PipelinePoolStats {
    pub total_pipelines: usize,
    pub active_pipelines: usize,
    pub available_pipelines: usize,
    pub total_frames_processed: u64,
    pub total_detections: u64,
}

