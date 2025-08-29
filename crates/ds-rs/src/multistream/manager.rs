#![allow(unused)]

//! Multi-stream manager for coordinating multiple detection pipelines

use super::{
    MetricsCollector, MultiStreamConfig, MultiStreamStateManager, PipelinePool, ResourceManager,
    StreamCoordinator, StreamState,
};
use crate::error::Result;
use crate::pipeline::Pipeline;
use crate::source::{FaultTolerantSourceController, SourceId};
use gstreamer as gst;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Manages multiple concurrent detection pipelines with fault tolerance
pub struct MultiStreamManager {
    /// Fault-tolerant source controller for stream management
    source_controller: Arc<FaultTolerantSourceController>,
    /// Pool of detection pipelines
    pipeline_pool: Arc<PipelinePool>,
    /// Stream coordination and load balancing
    coordinator: Arc<StreamCoordinator>,
    /// Resource management and monitoring
    resource_manager: Arc<ResourceManager>,
    /// Stream state tracking
    state_manager: Arc<MultiStreamStateManager>,
    /// Metrics collection
    metrics_collector: Arc<MetricsCollector>,
    /// Configuration
    config: MultiStreamConfig,
    /// Async runtime for concurrent processing
    runtime: Arc<Runtime>,
    /// Mapping of source IDs to pipeline IDs
    source_to_pipeline: Arc<Mutex<HashMap<SourceId, usize>>>,
}

impl MultiStreamManager {
    /// Create a new multi-stream manager
    pub fn new(
        pipeline: Arc<Pipeline>,
        streammux: gst::Element,
        config: MultiStreamConfig,
    ) -> Result<Self> {
        // Create fault-tolerant source controller
        let source_controller = Arc::new(FaultTolerantSourceController::new(
            pipeline.clone(),
            streammux,
        ));

        // Initialize components
        let pipeline_pool = Arc::new(PipelinePool::new(config.max_concurrent_streams));
        let coordinator = Arc::new(StreamCoordinator::new());
        let resource_manager = Arc::new(ResourceManager::new(config.resource_limits.clone()));
        let state_manager = Arc::new(MultiStreamStateManager::new());
        let metrics_collector = Arc::new(MetricsCollector::new());

        // Create async runtime for concurrent processing
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(config.worker_threads)
                .enable_all()
                .build()
                .map_err(|e| crate::DeepStreamError::Configuration(e.to_string()))?,
        );

        Ok(Self {
            source_controller,
            pipeline_pool,
            coordinator,
            resource_manager,
            state_manager,
            metrics_collector,
            config,
            runtime,
            source_to_pipeline: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Add a new stream with detection processing
    pub fn add_stream(&self, uri: &str) -> Result<SourceId> {
        // Check resource availability
        if !self.resource_manager.can_add_stream()? {
            return Err(crate::DeepStreamError::ResourceLimit(
                "Resource limits exceeded, cannot add new stream".to_string(),
            )
            .into());
        }

        // Add source through fault-tolerant controller
        let source_id = self.source_controller.add_source(uri)?;

        // Allocate a detection pipeline from the pool
        let pipeline_id = self.pipeline_pool.allocate_pipeline(source_id)?;

        // Track the mapping
        self.source_to_pipeline
            .lock()
            .unwrap()
            .insert(source_id, pipeline_id);

        // Register with state manager
        self.state_manager
            .add_stream(source_id, uri.to_string(), pipeline_id)?;

        // Set up detection processing for this stream
        self.setup_detection_processing(source_id, pipeline_id)?;

        // Start metrics collection for this stream
        self.metrics_collector.start_stream_metrics(source_id);

        // Notify coordinator
        self.coordinator.register_stream(source_id, pipeline_id)?;

        // Update resource tracking
        self.resource_manager.stream_added(source_id)?;

        Ok(source_id)
    }

    /// Remove a stream and clean up resources
    pub fn remove_stream(&self, source_id: SourceId) -> Result<()> {
        // Stop detection processing
        if let Some(&pipeline_id) = self.source_to_pipeline.lock().unwrap().get(&source_id) {
            self.pipeline_pool.release_pipeline(pipeline_id)?;
        }

        // Remove from source controller
        self.source_controller.remove_source(source_id)?;

        // Clean up state
        self.state_manager.remove_stream(source_id)?;
        self.source_to_pipeline.lock().unwrap().remove(&source_id);

        // Stop metrics collection
        self.metrics_collector.stop_stream_metrics(source_id);

        // Notify coordinator
        self.coordinator.unregister_stream(source_id)?;

        // Update resource tracking
        self.resource_manager.stream_removed(source_id)?;

        Ok(())
    }

    /// Add multiple streams concurrently
    pub fn add_streams_batch(&self, uris: &[String]) -> Result<Vec<SourceId>> {
        let mut source_ids = Vec::new();

        for uri in uris {
            match self.add_stream(uri) {
                Ok(id) => source_ids.push(id),
                Err(e) => {
                    eprintln!("Failed to add stream {}: {:?}", uri, e);
                    // Continue with other streams
                }
            }
        }

        Ok(source_ids)
    }

    /// Get the current state of all streams
    pub fn get_all_stream_states(&self) -> Vec<StreamState> {
        self.state_manager.get_all_streams()
    }

    /// Get metrics for a specific stream
    pub fn get_stream_metrics(&self, source_id: SourceId) -> Option<super::StreamMetrics> {
        self.metrics_collector.get_stream_metrics(source_id)
    }

    /// Get global multi-stream statistics
    pub fn get_stats(&self) -> super::MultiStreamStats {
        let mut stats = self.state_manager.get_stats();

        // Add resource usage
        if let Ok(usage) = self.resource_manager.get_current_usage() {
            stats.cpu_usage = usage.cpu_percentage;
            stats.memory_usage_mb = usage.memory_mb;
        }

        stats
    }

    /// Start monitoring all streams
    pub fn start_monitoring(&self) -> Result<()> {
        let state_manager = self.state_manager.clone();
        let resource_manager = self.resource_manager.clone();
        let metrics_collector = self.metrics_collector.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(5));

                // Update resource usage
                if let Err(e) = resource_manager.update_usage() {
                    eprintln!("Failed to update resource usage: {:?}", e);
                }

                // Collect metrics for all active streams
                for stream in state_manager.get_all_streams() {
                    if stream.is_active {
                        metrics_collector.update_stream(stream.source_id);
                    }
                }

                // Print summary
                let stats = state_manager.get_stats();
                println!(
                    "Active streams: {}, Avg FPS: {:.1}, Total detections: {}",
                    stats.active_streams, stats.average_fps, stats.total_detections
                );
            }
        });

        Ok(())
    }

    /// Set up detection processing for a stream
    fn setup_detection_processing(&self, source_id: SourceId, _pipeline_id: usize) -> Result<()> {
        let state_manager = self.state_manager.clone();
        let runtime = self.runtime.clone();

        // Spawn async task for detection processing
        runtime.spawn(async move {
            loop {
                // Get frames from the source
                // Process through detection pipeline
                // Update metrics

                // For now, simulate processing
                tokio::time::sleep(Duration::from_millis(33)).await; // ~30 FPS

                // Update metrics (simulated)
                let fps = 30.0;
                let detections = 2; // Simulated detection count

                if let Err(e) = state_manager.update_stream_metrics(source_id, fps, detections) {
                    eprintln!("Failed to update metrics for stream {}: {:?}", source_id, e);
                    break;
                }

                // Check if stream is still active
                if let Some(state) = state_manager.get_stream_state(source_id) {
                    if !state.is_active {
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        Ok(())
    }

    /// Apply adaptive quality control based on resources
    pub fn apply_adaptive_quality(&self) -> Result<()> {
        let usage = self.resource_manager.get_current_usage()?;

        if usage.cpu_percentage > 80.0 {
            // Reduce quality for all streams
            println!(
                "High CPU usage ({}%), reducing stream quality",
                usage.cpu_percentage
            );
            self.coordinator.apply_quality_reduction(0.8)?;
        } else if usage.cpu_percentage < 50.0 {
            // Can increase quality
            println!(
                "Low CPU usage ({}%), increasing stream quality",
                usage.cpu_percentage
            );
            self.coordinator.apply_quality_increase(1.2)?;
        }

        Ok(())
    }

    /// Restart a failed stream
    pub fn restart_stream(&self, source_id: SourceId) -> Result<()> {
        self.source_controller.restart_source(source_id)
    }
}
