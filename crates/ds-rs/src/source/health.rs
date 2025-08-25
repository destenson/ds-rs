use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use gstreamer as gst;
use gst::prelude::*;
use crate::error::Result;
use super::SourceId;

/// Health status of a source
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// Source is healthy and operating normally
    Healthy,
    /// Source is experiencing minor issues
    Degraded { reason: String },
    /// Source is unhealthy and may need intervention
    Unhealthy { reason: String },
    /// Source health is unknown (not enough data)
    Unknown,
}

/// Health metrics for a source
#[derive(Debug, Clone)]
pub struct HealthMetrics {
    /// Current frame rate
    pub frame_rate: f64,
    /// Average frame rate over window
    pub avg_frame_rate: f64,
    /// Number of buffer underruns
    pub buffer_underruns: usize,
    /// Network latency for RTSP sources (in ms)
    pub network_latency_ms: Option<f64>,
    /// Last frame timestamp
    pub last_frame_time: Option<Instant>,
    /// Total frames processed
    pub total_frames: usize,
    /// Dropped frames count
    pub dropped_frames: usize,
    /// Time since last health check
    pub time_since_last_check: Duration,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            frame_rate: 0.0,
            avg_frame_rate: 0.0,
            buffer_underruns: 0,
            network_latency_ms: None,
            last_frame_time: None,
            total_frames: 0,
            dropped_frames: 0,
            time_since_last_check: Duration::from_secs(0),
        }
    }
}

/// Configuration for health monitoring
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Minimum acceptable frame rate
    pub min_frame_rate: f64,
    /// Maximum acceptable buffer underruns
    pub max_buffer_underruns: usize,
    /// Maximum acceptable network latency (ms)
    pub max_network_latency_ms: f64,
    /// Time window for averaging metrics (seconds)
    pub window_size_secs: u64,
    /// Health check interval
    pub check_interval: Duration,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: usize,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            min_frame_rate: 10.0,
            max_buffer_underruns: 5,
            max_network_latency_ms: 500.0,
            window_size_secs: 10,
            check_interval: Duration::from_secs(5),
            failure_threshold: 3,
        }
    }
}

/// Trait for monitoring source health
pub trait HealthMonitor: Send + Sync {
    /// Perform a health check
    fn check_health(&self) -> HealthStatus;
    
    /// Get current health metrics
    fn get_metrics(&self) -> HealthMetrics;
    
    /// Update metrics with new frame data
    fn update_frame_metrics(&self, timestamp: Instant);
    
    /// Report a buffer underrun
    fn report_underrun(&self);
    
    /// Report network latency
    fn report_latency(&self, latency_ms: f64);
    
    /// Reset health metrics
    fn reset_metrics(&self);
}

/// Frame rate calculator with sliding window
struct FrameRateCalculator {
    timestamps: VecDeque<Instant>,
    window_size: Duration,
}

impl FrameRateCalculator {
    fn new(window_size: Duration) -> Self {
        Self {
            timestamps: VecDeque::new(),
            window_size,
        }
    }

    fn add_frame(&mut self, timestamp: Instant) {
        self.timestamps.push_back(timestamp);
        
        // Remove old timestamps outside the window
        let cutoff = timestamp - self.window_size;
        while let Some(front) = self.timestamps.front() {
            if *front < cutoff {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
    }

    fn get_rate(&self) -> f64 {
        if self.timestamps.len() < 2 {
            return 0.0;
        }

        let duration = *self.timestamps.back().unwrap() - *self.timestamps.front().unwrap();
        if duration.as_secs_f64() > 0.0 {
            (self.timestamps.len() - 1) as f64 / duration.as_secs_f64()
        } else {
            0.0
        }
    }
}

/// Default implementation of health monitoring
pub struct SourceHealthMonitor {
    source_id: SourceId,
    config: HealthConfig,
    metrics: Arc<Mutex<HealthMetrics>>,
    frame_calculator: Arc<Mutex<FrameRateCalculator>>,
    consecutive_failures: Arc<Mutex<usize>>,
    last_check: Arc<Mutex<Instant>>,
}

impl SourceHealthMonitor {
    pub fn new(source_id: SourceId, config: HealthConfig) -> Self {
        let window = Duration::from_secs(config.window_size_secs);
        Self {
            source_id,
            config,
            metrics: Arc::new(Mutex::new(HealthMetrics::default())),
            frame_calculator: Arc::new(Mutex::new(FrameRateCalculator::new(window))),
            consecutive_failures: Arc::new(Mutex::new(0)),
            last_check: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Install a pad probe to monitor buffer flow
    pub fn install_probe(&self, pad: &gst::Pad) -> Result<()> {
        let metrics = self.metrics.clone();
        let calculator = self.frame_calculator.clone();
        
        pad.add_probe(gst::PadProbeType::BUFFER, move |_, _| {
            let now = Instant::now();
            
            // Update frame metrics
            let mut calc = calculator.lock().unwrap();
            calc.add_frame(now);
            let frame_rate = calc.get_rate();
            drop(calc);
            
            let mut m = metrics.lock().unwrap();
            m.total_frames += 1;
            m.last_frame_time = Some(now);
            m.frame_rate = frame_rate;
            
            gst::PadProbeReturn::Ok
        });
        
        Ok(())
    }
}

impl HealthMonitor for SourceHealthMonitor {
    fn check_health(&self) -> HealthStatus {
        let metrics = self.metrics.lock().unwrap();
        let mut failures = self.consecutive_failures.lock().unwrap();
        let mut last_check = self.last_check.lock().unwrap();
        
        let now = Instant::now();
        let time_since_check = now - *last_check;
        *last_check = now;
        
        // Check frame rate
        if metrics.avg_frame_rate < self.config.min_frame_rate && metrics.total_frames > 10 {
            *failures += 1;
            if *failures >= self.config.failure_threshold {
                return HealthStatus::Unhealthy {
                    reason: format!("Frame rate too low: {:.1} fps", metrics.avg_frame_rate),
                };
            } else {
                return HealthStatus::Degraded {
                    reason: format!("Frame rate degraded: {:.1} fps", metrics.avg_frame_rate),
                };
            }
        }
        
        // Check buffer underruns
        if metrics.buffer_underruns > self.config.max_buffer_underruns {
            *failures += 1;
            return HealthStatus::Unhealthy {
                reason: format!("Too many buffer underruns: {}", metrics.buffer_underruns),
            };
        }
        
        // Check network latency
        if let Some(latency) = metrics.network_latency_ms {
            if latency > self.config.max_network_latency_ms {
                *failures += 1;
                return HealthStatus::Degraded {
                    reason: format!("High network latency: {:.1}ms", latency),
                };
            }
        }
        
        // Check if we're receiving frames
        if let Some(last_frame) = metrics.last_frame_time {
            let time_since_frame = now - last_frame;
            if time_since_frame > Duration::from_secs(5) {
                *failures += 1;
                return HealthStatus::Unhealthy {
                    reason: format!("No frames for {} seconds", time_since_frame.as_secs()),
                };
            }
        }
        
        // Reset consecutive failures on healthy check
        *failures = 0;
        HealthStatus::Healthy
    }

    fn get_metrics(&self) -> HealthMetrics {
        let metrics = self.metrics.lock().unwrap();
        let calculator = self.frame_calculator.lock().unwrap();
        
        HealthMetrics {
            frame_rate: metrics.frame_rate,
            avg_frame_rate: calculator.get_rate(),
            buffer_underruns: metrics.buffer_underruns,
            network_latency_ms: metrics.network_latency_ms,
            last_frame_time: metrics.last_frame_time,
            total_frames: metrics.total_frames,
            dropped_frames: metrics.dropped_frames,
            time_since_last_check: metrics.time_since_last_check,
        }
    }

    fn update_frame_metrics(&self, timestamp: Instant) {
        let mut calculator = self.frame_calculator.lock().unwrap();
        calculator.add_frame(timestamp);
        
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_frames += 1;
        metrics.last_frame_time = Some(timestamp);
        metrics.frame_rate = calculator.get_rate();
    }

    fn report_underrun(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.buffer_underruns += 1;
    }

    fn report_latency(&self, latency_ms: f64) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.network_latency_ms = Some(latency_ms);
    }

    fn reset_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = HealthMetrics::default();
        
        let mut calculator = self.frame_calculator.lock().unwrap();
        calculator.timestamps.clear();
        
        let mut failures = self.consecutive_failures.lock().unwrap();
        *failures = 0;
    }
}

/// Aggregates health status across multiple sources
pub struct HealthAggregator {
    monitors: Arc<Mutex<Vec<Box<dyn HealthMonitor>>>>,
}

impl HealthAggregator {
    pub fn new() -> Self {
        Self {
            monitors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_monitor(&self, monitor: Box<dyn HealthMonitor>) {
        let mut monitors = self.monitors.lock().unwrap();
        monitors.push(monitor);
    }

    pub fn get_overall_health(&self) -> HealthStatus {
        let monitors = self.monitors.lock().unwrap();
        
        if monitors.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut unhealthy_count = 0;
        let mut degraded_count = 0;
        let mut reasons = Vec::new();

        for monitor in monitors.iter() {
            match monitor.check_health() {
                HealthStatus::Unhealthy { reason } => {
                    unhealthy_count += 1;
                    reasons.push(reason);
                }
                HealthStatus::Degraded { reason } => {
                    degraded_count += 1;
                    reasons.push(reason);
                }
                _ => {}
            }
        }

        if unhealthy_count > 0 {
            HealthStatus::Unhealthy {
                reason: reasons.join("; "),
            }
        } else if degraded_count > 0 {
            HealthStatus::Degraded {
                reason: reasons.join("; "),
            }
        } else {
            HealthStatus::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_rate_calculation() {
        let mut calc = FrameRateCalculator::new(Duration::from_secs(1));
        
        // Add frames at 30 fps
        let start = Instant::now();
        for i in 0..30 {
            let timestamp = start + Duration::from_millis(i * 33); // ~30fps
            calc.add_frame(timestamp);
        }
        
        let rate = calc.get_rate();
        assert!(rate > 28.0 && rate < 32.0, "Expected ~30 fps, got {}", rate);
    }

    #[test]
    fn test_health_status_transitions() {
        let config = HealthConfig {
            min_frame_rate: 20.0,
            failure_threshold: 2,
            ..Default::default()
        };
        
        let monitor = SourceHealthMonitor::new(SourceId(0), config);
        
        // Initially unknown/healthy
        let status = monitor.check_health();
        assert!(matches!(status, HealthStatus::Healthy));
        
        // Simulate low frame rate
        let mut metrics = monitor.metrics.lock().unwrap();
        metrics.avg_frame_rate = 10.0;
        metrics.total_frames = 100;
        drop(metrics);
        
        // First check should be degraded
        let status = monitor.check_health();
        assert!(matches!(status, HealthStatus::Degraded { .. }));
    }

    #[test]
    fn test_buffer_underrun_detection() {
        let config = HealthConfig {
            max_buffer_underruns: 3,
            ..Default::default()
        };
        
        let monitor = SourceHealthMonitor::new(SourceId(0), config);
        
        // Report underruns
        for _ in 0..4 {
            monitor.report_underrun();
        }
        
        let status = monitor.check_health();
        assert!(matches!(status, HealthStatus::Unhealthy { .. }));
    }

    #[test]
    fn test_metrics_reset() {
        let monitor = SourceHealthMonitor::new(SourceId(0), HealthConfig::default());
        
        // Add some metrics
        monitor.update_frame_metrics(Instant::now());
        monitor.report_underrun();
        monitor.report_latency(100.0);
        
        // Reset
        monitor.reset_metrics();
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_frames, 0);
        assert_eq!(metrics.buffer_underruns, 0);
        assert!(metrics.network_latency_ms.is_none());
    }
}