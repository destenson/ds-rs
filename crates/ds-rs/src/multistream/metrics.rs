#![allow(unused)]

//! Metrics collection and monitoring for multi-stream processing

use crate::source::SourceId;
use std::sync::{Arc, RwLock, Mutex};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::Write;

/// Metrics for a single stream
#[derive(Debug, Clone)]
pub struct StreamMetrics {
    pub source_id: SourceId,
    pub start_time: Instant,
    pub last_update: Instant,
    pub frames_processed: u64,
    pub frames_dropped: u64,
    pub detections_count: u64,
    pub average_fps: f32,
    pub current_fps: f32,
    pub processing_time_ms: f32,
    pub detection_latency_ms: f32,
    pub error_count: u32,
    pub recovery_count: u32,
}

impl StreamMetrics {
    fn new(source_id: SourceId) -> Self {
        let now = Instant::now();
        Self {
            source_id,
            start_time: now,
            last_update: now,
            frames_processed: 0,
            frames_dropped: 0,
            detections_count: 0,
            average_fps: 0.0,
            current_fps: 0.0,
            processing_time_ms: 0.0,
            detection_latency_ms: 0.0,
            error_count: 0,
            recovery_count: 0,
        }
    }
    
    fn update_fps(&mut self) {
        let elapsed = self.last_update.elapsed().as_secs_f32();
        if elapsed > 0.0 {
            self.current_fps = 1.0 / elapsed;
        }
        
        let total_elapsed = self.start_time.elapsed().as_secs_f32();
        if total_elapsed > 0.0 {
            self.average_fps = self.frames_processed as f32 / total_elapsed;
        }
        
        self.last_update = Instant::now();
    }
}

/// Time-series data point
#[derive(Debug, Clone)]
struct MetricDataPoint {
    timestamp: Instant,
    value: f32,
}

/// Time-series metrics storage
#[derive(Debug)]
struct TimeSeries {
    data: VecDeque<MetricDataPoint>,
    max_points: usize,
}

impl TimeSeries {
    fn new(max_points: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(max_points),
            max_points,
        }
    }
    
    fn add_point(&mut self, value: f32) {
        let point = MetricDataPoint {
            timestamp: Instant::now(),
            value,
        };
        
        self.data.push_back(point);
        
        if self.data.len() > self.max_points {
            self.data.pop_front();
        }
    }
    
    fn get_average(&self, window: Duration) -> Option<f32> {
        let cutoff = Instant::now() - window;
        let recent: Vec<f32> = self.data.iter()
            .filter(|p| p.timestamp > cutoff)
            .map(|p| p.value)
            .collect();
        
        if recent.is_empty() {
            None
        } else {
            Some(recent.iter().sum::<f32>() / recent.len() as f32)
        }
    }
    
    fn get_max(&self, window: Duration) -> Option<f32> {
        let cutoff = Instant::now() - window;
        self.data.iter()
            .filter(|p| p.timestamp > cutoff)
            .map(|p| p.value)
            .fold(None, |max, val| {
                Some(max.map_or(val, |m: f32| m.max(val)))
            })
    }
    
    fn get_min(&self, window: Duration) -> Option<f32> {
        let cutoff = Instant::now() - window;
        self.data.iter()
            .filter(|p| p.timestamp > cutoff)
            .map(|p| p.value)
            .fold(None, |min, val| {
                Some(min.map_or(val, |m: f32| m.min(val)))
            })
    }
}

/// Collects and aggregates metrics for all streams
pub struct MetricsCollector {
    stream_metrics: Arc<RwLock<HashMap<SourceId, StreamMetrics>>>,
    time_series: Arc<Mutex<HashMap<String, TimeSeries>>>,
    export_file: Option<Arc<Mutex<File>>>,
    collection_interval: Duration,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            stream_metrics: Arc::new(RwLock::new(HashMap::new())),
            time_series: Arc::new(Mutex::new(HashMap::new())),
            export_file: None,
            collection_interval: Duration::from_secs(1),
        }
    }
    
    /// Enable metrics export to file
    pub fn enable_export(&mut self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        self.export_file = Some(Arc::new(Mutex::new(file)));
        Ok(())
    }
    
    /// Start collecting metrics for a stream
    pub fn start_stream_metrics(&self, source_id: SourceId) {
        let mut metrics = self.stream_metrics.write().unwrap();
        metrics.insert(source_id, StreamMetrics::new(source_id));
    }
    
    /// Stop collecting metrics for a stream
    pub fn stop_stream_metrics(&self, source_id: SourceId) {
        self.stream_metrics.write().unwrap().remove(&source_id);
    }
    
    /// Update stream with new frame
    pub fn update_stream(&self, source_id: SourceId) {
        let mut metrics = self.stream_metrics.write().unwrap();
        if let Some(m) = metrics.get_mut(&source_id) {
            m.frames_processed += 1;
            m.update_fps();
        }
    }
    
    /// Record detection results
    pub fn record_detection(&self, source_id: SourceId, count: usize, latency_ms: f32) {
        let mut metrics = self.stream_metrics.write().unwrap();
        if let Some(m) = metrics.get_mut(&source_id) {
            m.detections_count += count as u64;
            m.detection_latency_ms = latency_ms;
        }
        
        // Add to time series
        let mut series = self.time_series.lock().unwrap();
        let key = format!("detection_count_{}", source_id);
        series.entry(key)
            .or_insert_with(|| TimeSeries::new(1000))
            .add_point(count as f32);
    }
    
    /// Record dropped frame
    pub fn record_dropped_frame(&self, source_id: SourceId) {
        let mut metrics = self.stream_metrics.write().unwrap();
        if let Some(m) = metrics.get_mut(&source_id) {
            m.frames_dropped += 1;
        }
    }
    
    /// Record stream error
    pub fn record_error(&self, source_id: SourceId) {
        let mut metrics = self.stream_metrics.write().unwrap();
        if let Some(m) = metrics.get_mut(&source_id) {
            m.error_count += 1;
        }
    }
    
    /// Record stream recovery
    pub fn record_recovery(&self, source_id: SourceId) {
        let mut metrics = self.stream_metrics.write().unwrap();
        if let Some(m) = metrics.get_mut(&source_id) {
            m.recovery_count += 1;
        }
    }
    
    /// Get metrics for a specific stream
    pub fn get_stream_metrics(&self, source_id: SourceId) -> Option<StreamMetrics> {
        self.stream_metrics.read().unwrap().get(&source_id).cloned()
    }
    
    /// Get all stream metrics
    pub fn get_all_metrics(&self) -> Vec<StreamMetrics> {
        self.stream_metrics.read().unwrap().values().cloned().collect()
    }
    
    /// Get aggregated statistics
    pub fn get_aggregate_stats(&self) -> AggregateStats {
        let metrics = self.stream_metrics.read().unwrap();
        
        let total_frames: u64 = metrics.values().map(|m| m.frames_processed).sum();
        let total_dropped: u64 = metrics.values().map(|m| m.frames_dropped).sum();
        let total_detections: u64 = metrics.values().map(|m| m.detections_count).sum();
        let total_errors: u32 = metrics.values().map(|m| m.error_count).sum();
        
        let avg_fps = if !metrics.is_empty() {
            metrics.values().map(|m| m.average_fps).sum::<f32>() / metrics.len() as f32
        } else {
            0.0
        };
        
        let avg_latency = if !metrics.is_empty() {
            metrics.values().map(|m| m.detection_latency_ms).sum::<f32>() / metrics.len() as f32
        } else {
            0.0
        };
        
        AggregateStats {
            active_streams: metrics.len(),
            total_frames_processed: total_frames,
            total_frames_dropped: total_dropped,
            total_detections: total_detections,
            total_errors,
            average_fps: avg_fps,
            average_latency_ms: avg_latency,
            drop_rate: if total_frames > 0 {
                total_dropped as f32 / total_frames as f32
            } else {
                0.0
            },
        }
    }
    
    /// Export current metrics to file
    pub fn export_metrics(&self) -> std::io::Result<()> {
        if let Some(file) = &self.export_file {
            let stats = self.get_aggregate_stats();
            let mut file = file.lock().unwrap();
            
            writeln!(file, "Timestamp: {:?}", Instant::now())?;
            writeln!(file, "Active Streams: {}", stats.active_streams)?;
            writeln!(file, "Total Frames: {}", stats.total_frames_processed)?;
            writeln!(file, "Average FPS: {:.2}", stats.average_fps)?;
            writeln!(file, "Average Latency: {:.2}ms", stats.average_latency_ms)?;
            writeln!(file, "Drop Rate: {:.2}%", stats.drop_rate * 100.0)?;
            writeln!(file, "---")?;
            
            file.flush()?;
        }
        Ok(())
    }
    
    /// Generate performance report
    pub fn generate_report(&self, window: Duration) -> PerformanceReport {
        let stats = self.get_aggregate_stats();
        let series = self.time_series.lock().unwrap();
        
        // Calculate percentiles and trends from time series
        let fps_series = series.get("fps_aggregate");
        let latency_series = series.get("latency_aggregate");
        
        // Clone stats for recommendations
        let stats_clone = stats.clone();
        
        PerformanceReport {
            timestamp: Instant::now(),
            window,
            aggregate_stats: stats,
            fps_trend: fps_series.and_then(|s| s.get_average(window)),
            latency_trend: latency_series.and_then(|s| s.get_average(window)),
            recommendations: self.generate_recommendations(&stats_clone),
        }
    }
    
    /// Generate optimization recommendations
    fn generate_recommendations(&self, stats: &AggregateStats) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if stats.drop_rate > 0.1 {
            recommendations.push("High frame drop rate detected. Consider reducing stream quality or count.".to_string());
        }
        
        if stats.average_fps < 15.0 && stats.active_streams > 0 {
            recommendations.push("Low average FPS. System may be overloaded.".to_string());
        }
        
        if stats.average_latency_ms > 100.0 {
            recommendations.push("High detection latency. Consider optimizing detector configuration.".to_string());
        }
        
        if stats.total_errors > 10 {
            recommendations.push("Multiple errors detected. Check stream connectivity and resources.".to_string());
        }
        
        recommendations
    }
}

/// Aggregated statistics across all streams
#[derive(Debug, Clone)]
pub struct AggregateStats {
    pub active_streams: usize,
    pub total_frames_processed: u64,
    pub total_frames_dropped: u64,
    pub total_detections: u64,
    pub total_errors: u32,
    pub average_fps: f32,
    pub average_latency_ms: f32,
    pub drop_rate: f32,
}

/// Performance report with trends and recommendations
#[derive(Debug)]
pub struct PerformanceReport {
    pub timestamp: Instant,
    pub window: Duration,
    pub aggregate_stats: AggregateStats,
    pub fps_trend: Option<f32>,
    pub latency_trend: Option<f32>,
    pub recommendations: Vec<String>,
}
