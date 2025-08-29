//! Object tracking and trajectory management

use crate::metadata::{BoundingBox, ObjectMeta};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

/// Errors that can occur during tracking operations
#[derive(Debug, Error)]
pub enum TrackingError {
    #[error("Tracker not initialized")]
    NotInitialized,

    #[error("Invalid track ID: {0}")]
    InvalidTrackId(u64),

    #[error("Tracking failed: {0}")]
    TrackingFailed(String),
}

pub type Result<T> = std::result::Result<T, TrackingError>;

/// Tracker state
#[derive(Debug, Clone, PartialEq)]
pub enum TrackerState {
    /// New track just created
    New,

    /// Track is being actively tracked
    Tracking,

    /// Track temporarily lost but may recover
    Lost,

    /// Track removed/terminated
    Removed,
}

/// Track status information
#[derive(Debug, Clone)]
pub struct TrackStatus {
    /// Unique track ID
    pub track_id: u64,

    /// Current state
    pub state: TrackerState,

    /// Age of track (frames since creation)
    pub age: u32,

    /// Number of frames since last update
    pub time_since_update: u32,

    /// Number of consecutive hits
    pub hits: u32,

    /// Number of consecutive misses
    pub misses: u32,

    /// Track confidence
    pub confidence: f32,
}

impl TrackStatus {
    /// Create new track status
    pub fn new(track_id: u64) -> Self {
        Self {
            track_id,
            state: TrackerState::New,
            age: 0,
            time_since_update: 0,
            hits: 0,
            misses: 0,
            confidence: 0.0,
        }
    }

    /// Update track with a hit
    pub fn update_hit(&mut self, confidence: f32) {
        self.hits += 1;
        self.misses = 0;
        self.time_since_update = 0;
        self.confidence = confidence;
        self.state = TrackerState::Tracking;
    }

    /// Update track with a miss
    pub fn update_miss(&mut self) {
        self.misses += 1;
        self.hits = 0;
        self.time_since_update += 1;

        if self.misses > 5 {
            self.state = TrackerState::Lost;
        }
    }

    /// Check if track should be removed
    pub fn should_remove(&self, max_age: u32) -> bool {
        self.state == TrackerState::Lost && self.time_since_update > max_age
    }
}

/// Object trajectory over time
#[derive(Debug, Clone)]
pub struct Trajectory {
    /// Track ID
    pub track_id: u64,

    /// Historical positions (limited to max_history)
    positions: VecDeque<(f32, f32)>,

    /// Historical bounding boxes
    bboxes: VecDeque<BoundingBox>,

    /// Timestamps for each position
    timestamps: VecDeque<u64>,

    /// Maximum history to keep
    max_history: usize,
}

impl Trajectory {
    /// Create new trajectory
    pub fn new(track_id: u64, max_history: usize) -> Self {
        Self {
            track_id,
            positions: VecDeque::with_capacity(max_history),
            bboxes: VecDeque::with_capacity(max_history),
            timestamps: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Add a position to the trajectory
    pub fn add_position(&mut self, bbox: &BoundingBox, timestamp: u64) {
        let center = bbox.center();

        self.positions.push_back(center);
        self.bboxes.push_back(bbox.clone());
        self.timestamps.push_back(timestamp);

        // Limit history
        while self.positions.len() > self.max_history {
            self.positions.pop_front();
            self.bboxes.pop_front();
            self.timestamps.pop_front();
        }
    }

    /// Get the current position
    pub fn current_position(&self) -> Option<(f32, f32)> {
        self.positions.back().copied()
    }

    /// Get the current bounding box
    pub fn current_bbox(&self) -> Option<&BoundingBox> {
        self.bboxes.back()
    }

    /// Calculate velocity (pixels per second)
    pub fn velocity(&self) -> Option<(f32, f32)> {
        if self.positions.len() < 2 {
            return None;
        }

        let n = self.positions.len();
        let (x1, y1) = self.positions[n - 2];
        let (x2, y2) = self.positions[n - 1];
        let dt = (self.timestamps[n - 1] - self.timestamps[n - 2]) as f32 / 1_000_000_000.0; // ns to s

        if dt > 0.0 {
            Some(((x2 - x1) / dt, (y2 - y1) / dt))
        } else {
            None
        }
    }

    /// Calculate total distance traveled
    pub fn total_distance(&self) -> f32 {
        if self.positions.len() < 2 {
            return 0.0;
        }

        let mut distance = 0.0;
        for i in 1..self.positions.len() {
            let (x1, y1) = self.positions[i - 1];
            let (x2, y2) = self.positions[i];
            distance += ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        }

        distance
    }

    /// Get trajectory history
    pub fn history(&self) -> Vec<(f32, f32)> {
        self.positions.iter().copied().collect()
    }
}

/// Object tracker managing multiple tracks
pub struct ObjectTracker {
    /// Active tracks
    tracks: HashMap<u64, TrackStatus>,

    /// Track trajectories
    trajectories: HashMap<u64, Trajectory>,

    /// Next available track ID
    next_track_id: u64,

    /// Maximum tracks to maintain
    max_tracks: usize,

    /// Maximum age before removing lost tracks
    max_age: u32,

    /// Maximum trajectory history
    max_history: usize,
}

impl ObjectTracker {
    /// Create new object tracker
    pub fn new(max_tracks: usize, max_age: u32, max_history: usize) -> Self {
        Self {
            tracks: HashMap::new(),
            trajectories: HashMap::new(),
            next_track_id: 1,
            max_tracks,
            max_age,
            max_history,
        }
    }

    /// Create new track
    pub fn create_track(&mut self, object: &ObjectMeta) -> u64 {
        let track_id = self.next_track_id;
        self.next_track_id += 1;

        let mut status = TrackStatus::new(track_id);
        status.update_hit(object.confidence);

        let mut trajectory = Trajectory::new(track_id, self.max_history);
        trajectory.add_position(&object.rect_params, 0);

        self.tracks.insert(track_id, status);
        self.trajectories.insert(track_id, trajectory);

        track_id
    }

    /// Update existing track
    pub fn update_track(
        &mut self,
        track_id: u64,
        object: &ObjectMeta,
        timestamp: u64,
    ) -> Result<()> {
        let status = self
            .tracks
            .get_mut(&track_id)
            .ok_or(TrackingError::InvalidTrackId(track_id))?;

        status.update_hit(object.tracker_confidence);
        status.age += 1;

        if let Some(trajectory) = self.trajectories.get_mut(&track_id) {
            trajectory.add_position(&object.rect_params, timestamp);
        }

        Ok(())
    }

    /// Mark track as missed
    pub fn mark_missed(&mut self, track_id: u64) -> Result<()> {
        let status = self
            .tracks
            .get_mut(&track_id)
            .ok_or(TrackingError::InvalidTrackId(track_id))?;

        status.update_miss();
        status.age += 1;

        Ok(())
    }

    /// Remove track
    pub fn remove_track(&mut self, track_id: u64) -> Result<()> {
        self.tracks
            .remove(&track_id)
            .ok_or(TrackingError::InvalidTrackId(track_id))?;

        self.trajectories.remove(&track_id);

        Ok(())
    }

    /// Clean up old/lost tracks
    pub fn cleanup_tracks(&mut self) {
        let mut to_remove = Vec::new();

        for (track_id, status) in &self.tracks {
            if status.should_remove(self.max_age) {
                to_remove.push(*track_id);
            }
        }

        for track_id in to_remove {
            self.remove_track(track_id).ok();
        }

        // Limit total tracks
        if self.tracks.len() > self.max_tracks {
            // Remove oldest tracks
            let mut tracks_by_age: Vec<_> = self
                .tracks
                .iter()
                .map(|(id, status)| (*id, status.age))
                .collect();

            tracks_by_age.sort_by_key(|&(_, age)| std::cmp::Reverse(age));

            for (track_id, _) in tracks_by_age.iter().skip(self.max_tracks) {
                self.remove_track(*track_id).ok();
            }
        }
    }

    /// Get track status
    pub fn get_track_status(&self, track_id: u64) -> Option<&TrackStatus> {
        self.tracks.get(&track_id)
    }

    /// Get track trajectory
    pub fn get_trajectory(&self, track_id: u64) -> Option<&Trajectory> {
        self.trajectories.get(&track_id)
    }

    /// Get all active tracks
    pub fn active_tracks(&self) -> Vec<u64> {
        self.tracks
            .iter()
            .filter(|(_, status)| status.state == TrackerState::Tracking)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get tracking statistics
    pub fn get_stats(&self) -> TrackingStats {
        let mut stats = TrackingStats::default();

        for status in self.tracks.values() {
            stats.total_tracks += 1;

            match status.state {
                TrackerState::New => stats.new_tracks += 1,
                TrackerState::Tracking => stats.active_tracks += 1,
                TrackerState::Lost => stats.lost_tracks += 1,
                TrackerState::Removed => {}
            }
        }

        stats
    }
}

/// Tracking statistics
#[derive(Debug, Clone, Default)]
pub struct TrackingStats {
    pub total_tracks: usize,
    pub active_tracks: usize,
    pub new_tracks: usize,
    pub lost_tracks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_status() {
        let mut status = TrackStatus::new(1);
        assert_eq!(status.state, TrackerState::New);

        status.update_hit(0.95);
        assert_eq!(status.state, TrackerState::Tracking);
        assert_eq!(status.hits, 1);
        assert_eq!(status.misses, 0);

        for _ in 0..6 {
            status.update_miss();
        }
        assert_eq!(status.state, TrackerState::Lost);
    }

    #[test]
    fn test_trajectory() {
        let mut trajectory = Trajectory::new(1, 10);

        let bbox1 = BoundingBox::new(100.0, 100.0, 50.0, 50.0);
        let bbox2 = BoundingBox::new(110.0, 105.0, 50.0, 50.0);

        trajectory.add_position(&bbox1, 1_000_000_000);
        trajectory.add_position(&bbox2, 2_000_000_000);

        assert_eq!(trajectory.history().len(), 2);

        let velocity = trajectory.velocity();
        assert!(velocity.is_some());

        let distance = trajectory.total_distance();
        assert!(distance > 0.0);
    }

    #[test]
    fn test_object_tracker() {
        let mut tracker = ObjectTracker::new(100, 30, 50);

        let mut obj = ObjectMeta::new(1);
        obj.confidence = 0.9;
        obj.rect_params = BoundingBox::new(100.0, 100.0, 50.0, 50.0);

        let track_id = tracker.create_track(&obj);
        assert_eq!(track_id, 1);

        obj.rect_params = BoundingBox::new(105.0, 102.0, 50.0, 50.0);
        assert!(tracker.update_track(track_id, &obj, 1_000_000_000).is_ok());

        let status = tracker.get_track_status(track_id);
        assert!(status.is_some());
        assert_eq!(status.unwrap().hits, 2);

        let trajectory = tracker.get_trajectory(track_id);
        assert!(trajectory.is_some());

        let stats = tracker.get_stats();
        assert_eq!(stats.total_tracks, 1);
        assert_eq!(stats.active_tracks, 1);
    }
}
