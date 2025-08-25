#![allow(unused)]

//! Stream coordination for timing, synchronization and load balancing

use crate::source::SourceId;
use crate::error::Result;
use std::sync::{Arc, RwLock, Mutex};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::time::{Duration, Instant};

/// Priority level for stream processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum StreamPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Stream scheduling information
#[derive(Debug, Clone)]
pub struct StreamSchedule {
    pub source_id: SourceId,
    pub pipeline_id: usize,
    pub priority: StreamPriority,
    pub next_process_time: Instant,
    pub processing_interval: Duration,
    pub quality_factor: f32, // 0.1 to 1.0, affects frame skip
}

impl PartialEq for StreamSchedule {
    fn eq(&self, other: &Self) -> bool {
        self.source_id == other.source_id
    }
}

impl Eq for StreamSchedule {}

impl PartialOrd for StreamSchedule {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StreamSchedule {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier time
        other.priority.cmp(&self.priority)
            .then_with(|| self.next_process_time.cmp(&other.next_process_time))
    }
}

/// Coordinates timing and load balancing across streams
pub struct StreamCoordinator {
    schedules: Arc<RwLock<HashMap<SourceId, StreamSchedule>>>,
    processing_queue: Arc<Mutex<BinaryHeap<StreamSchedule>>>,
    load_balancer: Arc<LoadBalancer>,
    sync_manager: Arc<SyncManager>,
}

impl StreamCoordinator {
    pub fn new() -> Self {
        Self {
            schedules: Arc::new(RwLock::new(HashMap::new())),
            processing_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            load_balancer: Arc::new(LoadBalancer::new()),
            sync_manager: Arc::new(SyncManager::new()),
        }
    }
    
    /// Register a new stream for coordination
    pub fn register_stream(&self, source_id: SourceId, pipeline_id: usize) -> Result<()> {
        let schedule = StreamSchedule {
            source_id,
            pipeline_id,
            priority: StreamPriority::Normal,
            next_process_time: Instant::now(),
            processing_interval: Duration::from_millis(33), // ~30 FPS default
            quality_factor: 1.0,
        };
        
        self.schedules.write().unwrap().insert(source_id, schedule.clone());
        self.processing_queue.lock().unwrap().push(schedule);
        self.load_balancer.add_stream(source_id, pipeline_id);
        
        Ok(())
    }
    
    /// Unregister a stream
    pub fn unregister_stream(&self, source_id: SourceId) -> Result<()> {
        self.schedules.write().unwrap().remove(&source_id);
        self.load_balancer.remove_stream(source_id);
        
        // Remove from processing queue
        let mut queue = self.processing_queue.lock().unwrap();
        let filtered: Vec<_> = queue.drain().filter(|s| s.source_id != source_id).collect();
        for schedule in filtered {
            queue.push(schedule);
        }
        
        Ok(())
    }
    
    /// Set priority for a stream
    pub fn set_stream_priority(&self, source_id: SourceId, priority: StreamPriority) -> Result<()> {
        // Update in schedules map
        if let Some(schedule) = self.schedules.write().unwrap().get_mut(&source_id) {
            schedule.priority = priority;
        }
        
        // Also need to update in the processing queue
        let mut queue = self.processing_queue.lock().unwrap();
        let mut updated_schedules: Vec<_> = queue.drain().collect();
        for schedule in &mut updated_schedules {
            if schedule.source_id == source_id {
                schedule.priority = priority;
            }
        }
        for schedule in updated_schedules {
            queue.push(schedule);
        }
        
        Ok(())
    }
    
    /// Get the next stream to process
    pub fn get_next_stream(&self) -> Option<StreamSchedule> {
        let mut queue = self.processing_queue.lock().unwrap();
        
        if let Some(mut schedule) = queue.pop() {
            // Update next process time
            schedule.next_process_time = Instant::now() + schedule.processing_interval;
            
            // Re-add to queue for next iteration
            queue.push(schedule.clone());
            
            Some(schedule)
        } else {
            None
        }
    }
    
    /// Apply quality reduction to all streams
    pub fn apply_quality_reduction(&self, factor: f32) -> Result<()> {
        let mut schedules = self.schedules.write().unwrap();
        
        for schedule in schedules.values_mut() {
            schedule.quality_factor = (schedule.quality_factor * factor).max(0.1);
            // Increase processing interval to reduce load
            let new_interval_ms = (schedule.processing_interval.as_millis() as f32 / factor) as u64;
            schedule.processing_interval = Duration::from_millis(new_interval_ms.min(100)); // Cap at 10 FPS
        }
        
        Ok(())
    }
    
    /// Apply quality increase to all streams
    pub fn apply_quality_increase(&self, factor: f32) -> Result<()> {
        let mut schedules = self.schedules.write().unwrap();
        
        for schedule in schedules.values_mut() {
            schedule.quality_factor = (schedule.quality_factor * factor).min(1.0);
            // Decrease processing interval to increase frame rate
            let new_interval_ms = (schedule.processing_interval.as_millis() as f32 / factor) as u64;
            schedule.processing_interval = Duration::from_millis(new_interval_ms.max(16)); // Cap at 60 FPS
        }
        
        Ok(())
    }
    
    /// Synchronize stream processing
    pub fn synchronize_streams(&self, source_ids: &[SourceId]) -> Result<()> {
        self.sync_manager.create_sync_group(source_ids)
    }
    
    /// Get load balancing statistics
    pub fn get_load_stats(&self) -> LoadStats {
        self.load_balancer.get_stats()
    }
}

/// Load balancer for distributing work across pipelines
struct LoadBalancer {
    pipeline_loads: Arc<RwLock<HashMap<usize, f32>>>,
    stream_assignments: Arc<RwLock<HashMap<SourceId, usize>>>,
}

impl LoadBalancer {
    fn new() -> Self {
        Self {
            pipeline_loads: Arc::new(RwLock::new(HashMap::new())),
            stream_assignments: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    fn add_stream(&self, source_id: SourceId, pipeline_id: usize) {
        self.stream_assignments.write().unwrap().insert(source_id, pipeline_id);
        
        let mut loads = self.pipeline_loads.write().unwrap();
        *loads.entry(pipeline_id).or_insert(0.0) += 1.0;
    }
    
    fn remove_stream(&self, source_id: SourceId) {
        if let Some(pipeline_id) = self.stream_assignments.write().unwrap().remove(&source_id) {
            let mut loads = self.pipeline_loads.write().unwrap();
            if let Some(load) = loads.get_mut(&pipeline_id) {
                *load = (*load - 1.0).max(0.0);
            }
        }
    }
    
    fn get_least_loaded_pipeline(&self) -> Option<usize> {
        self.pipeline_loads.read().unwrap()
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(id, _)| *id)
    }
    
    fn get_stats(&self) -> LoadStats {
        let loads = self.pipeline_loads.read().unwrap();
        let total_load: f32 = loads.values().sum();
        let avg_load = if !loads.is_empty() {
            total_load / loads.len() as f32
        } else {
            0.0
        };
        
        LoadStats {
            total_pipelines: loads.len(),
            average_load: avg_load,
            max_load: loads.values().cloned().fold(0.0, f32::max),
            min_load: loads.values().cloned().fold(f32::MAX, f32::min),
        }
    }
}

/// Synchronization manager for coordinated stream processing
struct SyncManager {
    sync_groups: Arc<RwLock<Vec<Vec<SourceId>>>>,
}

impl SyncManager {
    fn new() -> Self {
        Self {
            sync_groups: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    fn create_sync_group(&self, source_ids: &[SourceId]) -> Result<()> {
        self.sync_groups.write().unwrap().push(source_ids.to_vec());
        Ok(())
    }
    
    fn get_sync_group(&self, source_id: SourceId) -> Option<Vec<SourceId>> {
        self.sync_groups.read().unwrap()
            .iter()
            .find(|group| group.contains(&source_id))
            .cloned()
    }
}

/// Load balancing statistics
#[derive(Debug, Clone)]
pub struct LoadStats {
    pub total_pipelines: usize,
    pub average_load: f32,
    pub max_load: f32,
    pub min_load: f32,
}
