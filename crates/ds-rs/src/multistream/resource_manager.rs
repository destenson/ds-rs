//! Resource management and monitoring for multi-stream processing

use crate::source::SourceId;
use crate::error::Result;
use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use sysinfo::System;

/// Resource limits configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f32,
    /// Maximum memory usage in MB
    pub max_memory_mb: f32,
    /// Maximum number of concurrent streams
    pub max_streams: usize,
    /// Enable adaptive throttling
    pub adaptive_throttling: bool,
    /// Memory per stream in MB
    pub memory_per_stream_mb: f32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 80.0,
            max_memory_mb: 2048.0,
            max_streams: 8,
            adaptive_throttling: true,
            memory_per_stream_mb: 200.0,
        }
    }
}

/// Current resource usage
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_percentage: f32,
    pub memory_mb: f32,
    pub active_streams: usize,
    pub timestamp: Instant,
}

/// Historical resource tracking
#[derive(Debug)]
struct ResourceHistory {
    cpu_history: Vec<(Instant, f32)>,
    memory_history: Vec<(Instant, f32)>,
    max_history_size: usize,
}

impl ResourceHistory {
    fn new() -> Self {
        Self {
            cpu_history: Vec::new(),
            memory_history: Vec::new(),
            max_history_size: 100,
        }
    }
    
    fn add_sample(&mut self, cpu: f32, memory: f32) {
        let now = Instant::now();
        
        self.cpu_history.push((now, cpu));
        self.memory_history.push((now, memory));
        
        // Trim old samples
        if self.cpu_history.len() > self.max_history_size {
            self.cpu_history.remove(0);
        }
        if self.memory_history.len() > self.max_history_size {
            self.memory_history.remove(0);
        }
    }
    
    fn get_average_cpu(&self, duration: Duration) -> f32 {
        let cutoff = Instant::now() - duration;
        let recent: Vec<f32> = self.cpu_history.iter()
            .filter(|(t, _)| *t > cutoff)
            .map(|(_, v)| *v)
            .collect();
        
        if recent.is_empty() {
            0.0
        } else {
            recent.iter().sum::<f32>() / recent.len() as f32
        }
    }
    
    fn get_average_memory(&self, duration: Duration) -> f32 {
        let cutoff = Instant::now() - duration;
        let recent: Vec<f32> = self.memory_history.iter()
            .filter(|(t, _)| *t > cutoff)
            .map(|(_, v)| *v)
            .collect();
        
        if recent.is_empty() {
            0.0
        } else {
            recent.iter().sum::<f32>() / recent.len() as f32
        }
    }
}

/// Manages resource allocation and monitoring
pub struct ResourceManager {
    limits: ResourceLimits,
    current_usage: Arc<RwLock<ResourceUsage>>,
    stream_resources: Arc<RwLock<HashMap<SourceId, StreamResources>>>,
    history: Arc<Mutex<ResourceHistory>>,
    system: Arc<Mutex<System>>,
    throttle_state: Arc<RwLock<ThrottleState>>,
}

/// Resources allocated to a specific stream
#[derive(Debug, Clone)]
struct StreamResources {
    memory_mb: f32,
    cpu_shares: f32,
    allocated_at: Instant,
}

/// Throttling state for adaptive resource management
#[derive(Debug, Clone)]
struct ThrottleState {
    is_throttled: bool,
    throttle_level: f32, // 0.0 to 1.0
    last_adjustment: Instant,
}

impl ResourceManager {
    pub fn new(limits: ResourceLimits) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            limits,
            current_usage: Arc::new(RwLock::new(ResourceUsage {
                cpu_percentage: 0.0,
                memory_mb: 0.0,
                active_streams: 0,
                timestamp: Instant::now(),
            })),
            stream_resources: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(Mutex::new(ResourceHistory::new())),
            system: Arc::new(Mutex::new(system)),
            throttle_state: Arc::new(RwLock::new(ThrottleState {
                is_throttled: false,
                throttle_level: 0.0,
                last_adjustment: Instant::now(),
            })),
        }
    }
    
    /// Check if we can add a new stream based on resources
    pub fn can_add_stream(&self) -> Result<bool> {
        let usage = self.current_usage.read().unwrap();
        
        // Check stream count limit
        if usage.active_streams >= self.limits.max_streams {
            return Ok(false);
        }
        
        // Check memory availability
        let projected_memory = usage.memory_mb + self.limits.memory_per_stream_mb;
        if projected_memory > self.limits.max_memory_mb {
            return Ok(false);
        }
        
        // Check CPU headroom
        if usage.cpu_percentage > self.limits.max_cpu_percent - 10.0 {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Allocate resources for a new stream
    pub fn stream_added(&self, source_id: SourceId) -> Result<()> {
        let resources = StreamResources {
            memory_mb: self.limits.memory_per_stream_mb,
            cpu_shares: 1.0 / self.limits.max_streams as f32,
            allocated_at: Instant::now(),
        };
        
        self.stream_resources.write().unwrap().insert(source_id, resources);
        
        let mut usage = self.current_usage.write().unwrap();
        usage.active_streams += 1;
        
        Ok(())
    }
    
    /// Release resources from a removed stream
    pub fn stream_removed(&self, source_id: SourceId) -> Result<()> {
        self.stream_resources.write().unwrap().remove(&source_id);
        
        let mut usage = self.current_usage.write().unwrap();
        usage.active_streams = usage.active_streams.saturating_sub(1);
        
        Ok(())
    }
    
    /// Update current resource usage
    pub fn update_usage(&self) -> Result<()> {
        let mut system = self.system.lock().unwrap();
        system.refresh_cpu_usage();
        system.refresh_memory();
        
        // Calculate CPU usage (average across all CPUs)
        let cpu_usage = system.cpus().iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>() / system.cpus().len() as f32;
        
        // Calculate memory usage
        let used_memory = system.used_memory() as f32 / 1024.0 / 1024.0; // Convert to MB
        
        // Update current usage
        let mut usage = self.current_usage.write().unwrap();
        usage.cpu_percentage = cpu_usage;
        usage.memory_mb = used_memory;
        usage.timestamp = Instant::now();
        
        // Add to history
        self.history.lock().unwrap().add_sample(cpu_usage, used_memory);
        
        // Check for throttling needs
        if self.limits.adaptive_throttling {
            self.update_throttle_state(cpu_usage, used_memory)?;
        }
        
        Ok(())
    }
    
    /// Update throttle state based on resource usage
    fn update_throttle_state(&self, cpu: f32, memory: f32) -> Result<()> {
        let mut state = self.throttle_state.write().unwrap();
        
        // Don't adjust too frequently
        if state.last_adjustment.elapsed() < Duration::from_secs(2) {
            return Ok(());
        }
        
        let cpu_pressure = cpu / self.limits.max_cpu_percent;
        let memory_pressure = memory / self.limits.max_memory_mb;
        let pressure = cpu_pressure.max(memory_pressure);
        
        if pressure > 0.9 {
            // High pressure - increase throttling
            state.is_throttled = true;
            state.throttle_level = (state.throttle_level + 0.1).min(1.0);
        } else if pressure < 0.6 {
            // Low pressure - reduce throttling
            state.throttle_level = (state.throttle_level - 0.1).max(0.0);
            if state.throttle_level == 0.0 {
                state.is_throttled = false;
            }
        }
        
        state.last_adjustment = Instant::now();
        
        Ok(())
    }
    
    /// Get current resource usage
    pub fn get_current_usage(&self) -> Result<ResourceUsage> {
        Ok(self.current_usage.read().unwrap().clone())
    }
    
    /// Get throttle recommendations
    pub fn get_throttle_recommendation(&self) -> ThrottleRecommendation {
        let state = self.throttle_state.read().unwrap();
        
        if state.is_throttled {
            ThrottleRecommendation {
                should_throttle: true,
                quality_factor: 1.0 - state.throttle_level * 0.5, // Reduce quality by up to 50%
                frame_skip: (state.throttle_level * 3.0) as usize, // Skip up to 3 frames
            }
        } else {
            ThrottleRecommendation {
                should_throttle: false,
                quality_factor: 1.0,
                frame_skip: 0,
            }
        }
    }
    
    /// Get resource statistics over a time window
    pub fn get_stats(&self, window: Duration) -> ResourceStats {
        let history = self.history.lock().unwrap();
        let usage = self.current_usage.read().unwrap();
        
        ResourceStats {
            current_cpu: usage.cpu_percentage,
            average_cpu: history.get_average_cpu(window),
            current_memory: usage.memory_mb,
            average_memory: history.get_average_memory(window),
            active_streams: usage.active_streams,
            max_streams: self.limits.max_streams,
        }
    }
    
    /// Predict if we can handle additional load
    pub fn predict_capacity(&self, additional_streams: usize) -> CapacityPrediction {
        let usage = self.current_usage.read().unwrap();
        
        let projected_streams = usage.active_streams + additional_streams;
        let projected_memory = usage.memory_mb + (additional_streams as f32 * self.limits.memory_per_stream_mb);
        let projected_cpu = usage.cpu_percentage * (1.0 + (additional_streams as f32 * 0.1));
        
        CapacityPrediction {
            can_handle: projected_streams <= self.limits.max_streams &&
                       projected_memory <= self.limits.max_memory_mb &&
                       projected_cpu <= self.limits.max_cpu_percent,
            projected_cpu,
            projected_memory,
            projected_streams,
        }
    }
}

/// Throttle recommendation based on resource usage
#[derive(Debug, Clone)]
pub struct ThrottleRecommendation {
    pub should_throttle: bool,
    pub quality_factor: f32,
    pub frame_skip: usize,
}

/// Resource statistics over a time window
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub current_cpu: f32,
    pub average_cpu: f32,
    pub current_memory: f32,
    pub average_memory: f32,
    pub active_streams: usize,
    pub max_streams: usize,
}

/// Capacity prediction for additional streams
#[derive(Debug, Clone)]
pub struct CapacityPrediction {
    pub can_handle: bool,
    pub projected_cpu: f32,
    pub projected_memory: f32,
    pub projected_streams: usize,
}