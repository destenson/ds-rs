# PRP-22: CPU-Based Object Tracking Module

## Summary
Implement lightweight object tracking algorithms for the Standard backend, replacing the identity element placeholder with functional tracking using Centroid, Kalman filter, and SORT algorithms optimized for CPU execution.

## Background
The Standard backend currently uses an identity element (passthrough) for tracking, providing no actual tracking capability. This PRP implements real multi-object tracking using CPU-efficient algorithms, building on the detection capabilities from PRP-21.

## Goals
1. Implement tiered tracking algorithms (Centroid, Kalman, SORT)
2. Create reusable tracking infrastructure for Standard backend
3. Handle multi-object tracking with ID management
4. Provide configurable algorithm selection
5. Maintain compatibility with existing pipeline

## Non-Goals
- Deep learning based tracking (DeepSORT with heavy features)
- GPU acceleration
- 3D tracking or pose estimation
- Re-implementation of existing DeepStream tracker features

## Detailed Design

### Module Structure
```
crates/ds-rs/src/backend/cpu_vision/
├── tracker.rs           # Core tracking logic
├── centroid.rs         # Centroid tracker implementation
├── kalman.rs           # Kalman filter
├── sort.rs             # SORT algorithm
└── association.rs      # Data association utilities
```

### Tracking Algorithms

#### 1. Centroid Tracker (Simplest)
Basic position-based tracking:

- Compute centroids of bounding boxes
- Track via Euclidean distance
- Simple ID assignment logic
- Best for: Static cameras, slow motion
- Performance: 100+ FPS capability

#### 2. Kalman Filter Tracker
Motion prediction tracking:

- State: [x, y, w, h, dx, dy, dw, dh]
- Predict next position
- Update with measurements
- Better occlusion handling
- Performance: 50+ FPS capability

#### 3. SORT (Simple Online Realtime Tracking)
Complete tracking system:

- Kalman filter for state estimation
- Hungarian algorithm for assignment
- IoU-based association
- Track lifecycle management
- Performance: 30+ FPS capability

### Core Components

#### Track Management
```rust
pub struct Track {
    pub id: u32,
    pub bbox: BoundingBox,
    pub confidence: f32,
    pub class_id: u32,
    pub age: u32,
    pub hits: u32,
    pub time_since_update: u32,
    state: TrackState,
}

enum TrackState {
    Tentative,   // New track
    Confirmed,   // Established track
    Deleted,     // To be removed
}
```

#### Association Metrics
- **IoU (Intersection over Union)**: Spatial overlap
- **Euclidean Distance**: Centroid distance
- **Mahalanobis Distance**: Statistical distance

#### Track Lifecycle
1. **Creation**: New detection without match
2. **Update**: Detection matched to track
3. **Prediction**: No detection match
4. **Deletion**: Too many missed frames

### Integration Architecture

#### Tracker Trait
```rust
pub trait Tracker: Send + Sync {
    fn update(&mut self, detections: Vec<Detection>) -> Vec<Track>;
    fn predict(&mut self);
    fn get_tracks(&self) -> &[Track];
    fn reset(&mut self);
}
```

#### Factory Pattern
```rust
pub enum TrackerType {
    Centroid,
    Kalman,
    Sort,
}

pub fn create_tracker(tracker_type: TrackerType) -> Box<dyn Tracker> {
    match tracker_type {
        TrackerType::Centroid => Box::new(CentroidTracker::new()),
        TrackerType::Kalman => Box::new(KalmanTracker::new()),
        TrackerType::Sort => Box::new(SortTracker::new()),
    }
}
```

### GStreamer Integration

#### Element Properties
- `tracker-type`: centroid|kalman|sort
- `max-age`: Maximum frames without detection
- `min-hits`: Minimum detections to confirm track
- `iou-threshold`: Association threshold

#### Metadata Flow
1. Receive detection metadata from detector
2. Run tracking algorithm
3. Attach track metadata to buffer
4. Include track ID, age, trajectory

## Implementation Plan

### Phase 1: Centroid Tracker
1. Implement centroid calculation
2. Add distance-based matching
3. Create ID management system
4. Write comprehensive tests

### Phase 2: Kalman Filter
1. Port Kalman filter equations
2. Implement state prediction
3. Add measurement updates
4. Validate with synthetic data

### Phase 3: Hungarian Algorithm
1. Add hungarian crate dependency
2. Create cost matrix generation
3. Implement assignment logic
4. Test with multiple objects

### Phase 4: SORT Integration
1. Combine Kalman + Hungarian
2. Add IoU calculation
3. Implement track management
4. Benchmark performance

### Phase 5: GStreamer Element
1. Create tracker element
2. Handle metadata I/O
3. Add configuration properties
4. Test in pipeline

## Testing Strategy

### Unit Tests
- Centroid distance calculations
- Kalman filter predictions
- Hungarian assignments
- Track lifecycle management

### Integration Tests
- Multi-object scenarios
- Occlusion handling
- ID consistency
- Performance under load

### Test Scenarios
1. Linear motion tracking
2. Crossing paths
3. Object entry/exit
4. Temporary occlusions
5. Dense object scenes

## Validation Gates

```bash
# Format and lint
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings

# Unit tests
cargo test cpu_vision::tracker --all-features

# Tracking accuracy test
cargo run --example tracking_benchmark

# Integration test
cargo test --test tracking_integration
```

## Resources

### Mathematical References
- Kalman Filter: https://www.kalmanfilter.net/
- Hungarian Algorithm: https://en.wikipedia.org/wiki/Hungarian_algorithm
- SORT Paper: https://arxiv.org/abs/1602.00763

### Rust Implementations
- kalman-rust: https://crates.io/crates/kalman-rust
- hungarian: https://crates.io/crates/hungarian
- DeepSort Rust: https://github.com/andreytkachenko/deep-sort

### Code References
- Current tracker stub: `crates/ds-rs/src/backend/standard.rs:107-114`
- Track metadata types: `crates/ds-rs/src/metadata/`
- Source management patterns: `crates/ds-rs/src/source/`

### Related Projects
- py-motmetrics: https://github.com/cheind/py-motmetrics
- Simple Online Tracking: https://github.com/abewley/sort

## Performance Targets

### Minimum Requirements
- Centroid: 100+ FPS
- Kalman: 50+ FPS  
- SORT: 30+ FPS
- < 100MB memory for 100 tracks
- < 10ms processing per frame

### Quality Metrics
- MOTA (Multiple Object Tracking Accuracy) > 60%
- ID switches < 10% of tracks
- Track fragmentation < 15%

## Algorithm Selection Guide

### Use Centroid When:
- Static or slow-moving camera
- Objects maintain consistent speed
- Real-time performance critical
- Memory very constrained

### Use Kalman When:
- Predictable motion patterns
- Need to handle brief occlusions
- Moderate compute available

### Use SORT When:
- Complex multi-object scenes
- Need best tracking quality
- Can afford 30ms per frame
- Varying object speeds

## Risk Mitigation

### Performance Issues
- Start with Centroid for baseline
- Add algorithm switching at runtime
- Implement frame skipping option

### ID Switch Problems
- Tune association thresholds
- Add appearance features later
- Implement track smoothing

### Memory Growth
- Limit maximum tracks
- Aggressive track pruning
- Circular buffer for history

## Success Criteria

1. All three trackers implemented and tested
2. 30+ FPS achieved with SORT on 10 objects
3. Correct track ID persistence across frames
4. Integration with detection pipeline working
5. Configurable algorithm selection

## Dependencies

### Requires
- PRP-21 (CPU Detection Module) completed
- Detection metadata format defined

### External Dependencies
- nalgebra = "0.33" (matrix operations)
- hungarian = "1.1" (assignment)
- kalman-rust = "0.1" (optional)

## Notes

- Start with Centroid for MVP
- SORT provides best quality/performance balance
- Consider adding simple appearance features later
- Batch processing can improve performance
- Track smoothing improves visual quality

## Future Enhancements

1. **Appearance Features**: Simple color histogram
2. **Multi-class Tracking**: Class-specific parameters
3. **Track Smoothing**: Trajectory interpolation
4. **Predictive Caching**: Anticipate object positions
5. **Track Visualization**: Debug overlay support

## Confidence Score: 9/10

High confidence because:
- Well-understood algorithms
- Existing Rust implementations available
- Clear performance/quality tradeoffs
- Incremental implementation path

Minor concern:
- Tuning parameters for optimal performance