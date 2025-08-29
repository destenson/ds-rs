//! Multi-stream detection pipeline architecture
//!
//! This module provides scalable multi-stream processing with concurrent detection,
//! fault tolerance, and resource management.

pub mod config;
pub mod manager;
pub mod metrics;
pub mod pipeline_pool;
pub mod resource_manager;
pub mod stream_coordinator;

pub use config::{MultiStreamConfig, MultiStreamConfigBuilder};
pub use manager::MultiStreamManager;
pub use metrics::{MetricsCollector, StreamMetrics};
pub use pipeline_pool::{DetectionPipeline, PipelinePool};
pub use resource_manager::{ResourceLimits, ResourceManager};
pub use stream_coordinator::{StreamCoordinator, StreamPriority};

use crate::error::Result;
use crate::source::SourceId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Stream state for tracking individual stream processing
#[derive(Debug, Clone)]
pub struct StreamState {
    pub source_id: SourceId,
    pub uri: String,
    pub pipeline_id: usize,
    pub is_active: bool,
    pub fps: f32,
    pub frames_processed: u64,
    pub detections_count: u64,
    pub last_error: Option<String>,
}

/// Multi-stream event types
#[derive(Debug, Clone)]
pub enum MultiStreamEvent {
    StreamAdded { source_id: SourceId, uri: String },
    StreamRemoved { source_id: SourceId },
    DetectionProcessed { source_id: SourceId, count: usize },
    StreamError { source_id: SourceId, error: String },
    ResourceThresholdReached { cpu_usage: f32, memory_usage: f32 },
}

/// Multi-stream statistics
#[derive(Debug, Default)]
pub struct MultiStreamStats {
    pub active_streams: usize,
    pub total_streams_processed: usize,
    pub total_frames_processed: u64,
    pub total_detections: u64,
    pub average_fps: f32,
    pub cpu_usage: f32,
    pub memory_usage_mb: f32,
}

/// Manager for multi-stream state
pub struct MultiStreamStateManager {
    streams: Arc<RwLock<HashMap<SourceId, StreamState>>>,
    stats: Arc<RwLock<MultiStreamStats>>,
}

impl MultiStreamStateManager {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(MultiStreamStats::default())),
        }
    }

    pub fn add_stream(&self, source_id: SourceId, uri: String, pipeline_id: usize) -> Result<()> {
        let mut streams = self.streams.write().unwrap();
        streams.insert(
            source_id,
            StreamState {
                source_id,
                uri,
                pipeline_id,
                is_active: true,
                fps: 0.0,
                frames_processed: 0,
                detections_count: 0,
                last_error: None,
            },
        );

        let mut stats = self.stats.write().unwrap();
        stats.active_streams = streams.len();
        stats.total_streams_processed += 1;

        Ok(())
    }

    pub fn remove_stream(&self, source_id: SourceId) -> Result<()> {
        let mut streams = self.streams.write().unwrap();
        streams.remove(&source_id);

        let mut stats = self.stats.write().unwrap();
        stats.active_streams = streams.len();

        Ok(())
    }

    pub fn update_stream_metrics(
        &self,
        source_id: SourceId,
        fps: f32,
        detections: usize,
    ) -> Result<()> {
        let mut streams = self.streams.write().unwrap();
        if let Some(stream) = streams.get_mut(&source_id) {
            stream.fps = fps;
            stream.frames_processed += 1;
            stream.detections_count += detections as u64;
        }

        // Update global stats
        let mut stats = self.stats.write().unwrap();
        stats.total_frames_processed += 1;
        stats.total_detections += detections as u64;

        // Calculate average FPS
        let active_streams = streams.values().filter(|s| s.is_active).collect::<Vec<_>>();
        if !active_streams.is_empty() {
            stats.average_fps =
                active_streams.iter().map(|s| s.fps).sum::<f32>() / active_streams.len() as f32;
        }

        Ok(())
    }

    pub fn get_stream_state(&self, source_id: SourceId) -> Option<StreamState> {
        self.streams.read().unwrap().get(&source_id).cloned()
    }

    pub fn get_all_streams(&self) -> Vec<StreamState> {
        self.streams.read().unwrap().values().cloned().collect()
    }

    pub fn get_stats(&self) -> MultiStreamStats {
        let stats = self.stats.read().unwrap();
        MultiStreamStats {
            active_streams: stats.active_streams,
            total_streams_processed: stats.total_streams_processed,
            total_frames_processed: stats.total_frames_processed,
            total_detections: stats.total_detections,
            average_fps: stats.average_fps,
            cpu_usage: stats.cpu_usage,
            memory_usage_mb: stats.memory_usage_mb,
        }
    }
}
