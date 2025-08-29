# PRP-20: CPU-Based Vision Backend for Object Detection and Tracking

**Status**: PARTIAL - CPU vision backend structure created, detector element implemented

## Summary
Implement a fully functional CPU-based vision backend as an alternative to NVIDIA-specific elements, enabling object detection and tracking on systems with limited GPU resources. This backend will leverage existing Rust GStreamer plugins, OpenCV DNN module, and lightweight tracking algorithms to provide real-time video analytics on integrated graphics and CPU-only systems.

## Background
The current ds-rs implementation has three backends:
- **DeepStream**: Requires NVIDIA hardware
- **Standard**: Uses fakesink for inference and identity for tracking (non-functional)
- **Mock**: For testing only

The Standard backend needs to be upgraded from placeholder implementations to actual CPU-based computer vision processing, enabling real object detection and tracking without NVIDIA dependencies.

## Goals
1. Replace placeholder implementations in Standard backend with functional CV processing
2. Integrate lightweight object detection models (MobileNet SSD, YOLOv5 Nano)
3. Implement CPU-optimized tracking algorithms (SORT, Centroid, Kalman)
4. Leverage integrated graphics compute capabilities where available
5. Maintain compatibility with existing backend abstraction

## Non-Goals
- Achieving NVIDIA-level performance (this is for resource-constrained systems)
- Supporting all DeepStream features (focus on core detection/tracking)
- Implementing training or model conversion pipelines

## Detailed Design

### Architecture Overview
The CPU Vision Backend will be implemented as an enhancement to the existing Standard backend, introducing new modules:

```
crates/ds-rs/src/backend/
├── standard.rs          (enhanced)
├── cpu_vision/
│   ├── mod.rs
│   ├── detector.rs      (OpenCV DNN wrapper)
│   ├── tracker.rs       (SORT/Centroid implementation)
│   └── elements.rs      (GStreamer element creation)
```

### Phase 1: Detection Infrastructure

#### OpenCV Integration
- Add opencv-rust dependency with DNN module support
- Create OpenCVDetector wrapper implementing detection trait
- Support ONNX model loading for cross-platform compatibility
- Implement batching for multi-stream efficiency

#### Model Support
Priority order for implementation:
1. **YOLOv5 Nano** - Smallest YOLO variant, good accuracy/speed balance
2. **MobileNet SSD** - Optimized for mobile/embedded, 300x300 input
3. **YOLOv8n** - Latest nano variant if performance allows

#### Detection Pipeline
1. Frame preprocessing (resize, normalization)
2. Blob creation for neural network input
3. Forward pass through DNN
4. Post-processing (NMS, confidence filtering)
5. Bounding box extraction

### Phase 2: Tracking Implementation

#### Tracking Algorithms
Implement tiered tracking based on available compute:

1. **Centroid Tracker** (lowest compute)
   - Simple Euclidean distance between frame centroids
   - Suitable for stable camera, slow-moving objects
   
2. **Kalman Filter + Hungarian** (SORT)
   - Position and velocity prediction
   - Hungarian algorithm for data association
   - Good balance of speed and accuracy
   
3. **Appearance Features** (DeepSORT-lite)
   - Lightweight feature extractor
   - Cosine similarity for re-identification
   - Handles occlusions better

#### Tracking Pipeline
1. Receive detections from detector
2. Predict object positions (Kalman)
3. Associate detections with tracks (Hungarian)
4. Update track states
5. Handle track lifecycle (creation/deletion)

### Phase 3: GStreamer Integration

#### Custom Elements
Create new GStreamer elements:

1. **cpudetector** - Wraps OpenCV DNN detection
   - Properties: model-path, confidence-threshold, nms-threshold
   - Input: video/x-raw
   - Output: video/x-raw with metadata
   
2. **cputracker** - Implements tracking algorithms
   - Properties: tracker-type (centroid/sort/deepsort-lite)
   - Input: video/x-raw with detection metadata
   - Output: video/x-raw with track metadata

3. **cpuosd** - Optimized overlay for bounding boxes
   - Properties: show-labels, show-confidence, show-track-id
   - Leverages cairo or skia for drawing

#### Integration with Existing Backend
Modify standard.rs to use new elements:

```rust
// Current (non-functional)
fn create_inference(&self, name: Option<&str>, _config_path: &str) -> Result<gst::Element> {
    // Returns fakesink
}

// Enhanced (functional)
fn create_inference(&self, name: Option<&str>, config_path: &str) -> Result<gst::Element> {
    // Returns cpudetector element with model from config_path
}
```

### Phase 4: Optimization

#### CPU Optimizations
- SIMD operations for preprocessing
- Multi-threading for batch processing
- Frame skipping under high load
- ROI-based processing for efficiency

#### Integrated Graphics Utilization
- OpenGL compute shaders for preprocessing
- Hardware video decode acceleration
- Zero-copy frame handling where possible

#### Memory Management
- Object pooling for detection results
- Circular buffers for tracking history
- Lazy initialization of models

## Implementation Plan

### Prerequisites
1. Add dependencies to Cargo.toml:
   - opencv = { version = "0.92", features = ["dnn"] }
   - nalgebra = "0.33" (for Kalman filter math)
   - hungarian = "1.1" (for data association)
   
2. Ensure OpenCV 4.x installed with DNN module

### Task Sequence

1. **Setup OpenCV Integration** (backend/cpu_vision/detector.rs)
   - Create OpenCVDetector struct
   - Implement model loading (ONNX)
   - Add preprocessing pipeline
   - Test with YOLOv5 Nano model

2. **Implement Centroid Tracker** (backend/cpu_vision/tracker.rs)
   - Create CentroidTracker struct
   - Implement Euclidean distance matching
   - Add track lifecycle management
   - Write unit tests

3. **Create cpudetector Element** (backend/cpu_vision/elements.rs)
   - Define GStreamer element boilerplate
   - Wire OpenCVDetector to element
   - Add properties for configuration
   - Implement caps negotiation

4. **Implement SORT Tracker** (backend/cpu_vision/tracker.rs)
   - Add Kalman filter implementation
   - Integrate Hungarian algorithm
   - Add motion prediction
   - Benchmark against Centroid

5. **Create cputracker Element** (backend/cpu_vision/elements.rs)
   - Define element with tracker selection
   - Handle detection metadata input
   - Output track metadata
   - Test with multiple trackers

6. **Update Standard Backend** (backend/standard.rs)
   - Replace fakesink with cpudetector
   - Replace identity with cputracker
   - Update element mappings
   - Maintain backward compatibility

7. **Optimize Performance**
   - Profile bottlenecks
   - Add frame skipping logic
   - Implement batch processing
   - Tune for integrated graphics

8. **Integration Testing**
   - Test with existing pipelines
   - Verify metadata compatibility
   - Benchmark CPU usage
   - Document performance limits

## Testing Strategy

### Unit Tests
- Model loading and inference
- Tracker algorithm correctness
- Metadata serialization
- Element property handling

### Integration Tests
- End-to-end pipeline with test videos
- Multi-stream performance
- Backend switching compatibility
- Memory leak detection

### Performance Tests
- FPS benchmarks per model/tracker combo
- CPU usage monitoring
- Memory consumption tracking
- Latency measurements

## Validation Gates

```bash
# Unit tests
cargo test --package ds-rs --lib backend::cpu_vision

# Integration tests  
cargo test --package ds-rs --test cpu_backend_tests

# Benchmark
cargo bench --package ds-rs --bench cpu_vision_bench

# Memory check
valgrind --leak-check=full target/debug/ds-app --backend standard
```

## Resources and References

### Documentation
- OpenCV DNN Tutorial: https://docs.opencv.org/4.x/d2/d58/tutorial_table_of_content_dnn.html
- YOLOv5 Export to ONNX: https://github.com/ultralytics/yolov5/issues/251
- Rust OpenCV Bindings: https://github.com/twistedfall/opencv-rust
- GStreamer Plugin Writing: https://gstreamer.freedesktop.org/documentation/plugin-development/

### Existing Code References
- Current backend abstraction: `crates/ds-rs/src/backend/mod.rs`
- Standard backend to enhance: `crates/ds-rs/src/backend/standard.rs`
- HSV detector example: `C:\Users\deste\repos\gst-plugins-rs\video\hsv\src\hsvdetector\`
- Color detect example: `C:\Users\deste\repos\gst-plugins-rs\video\videofx\src\colordetect\`

### Model Resources
- YOLOv5 Nano weights: https://github.com/ultralytics/yolov5/releases
- MobileNet SSD: https://github.com/chuanqi305/MobileNet-SSD
- COCO dataset labels: Already in codebase at `inference/mod.rs:176`

### Related Rust Crates
- object-detection-opencv-rust: https://crates.io/crates/object-detection-opencv-rust
- kalman-rust: https://crates.io/crates/kalman-rust
- DeepSort Rust: https://github.com/andreytkachenko/deep-sort

## Performance Targets

### Minimum Requirements (Single Stream)
- 15 FPS on integrated graphics
- < 50% CPU usage on quad-core
- < 500MB RAM per stream
- < 100ms detection latency

### Stretch Goals (Multi-Stream)
- 4 concurrent streams at 10 FPS each
- Dynamic quality adjustment
- Hardware decode acceleration
- GPU compute shader preprocessing

## Common Pitfalls to Avoid

1. **Model Format Issues**: Ensure ONNX opset 12 for OpenCV compatibility
2. **Memory Leaks**: Careful with OpenCV Mat lifecycle in Rust
3. **Thread Safety**: GStreamer elements must be thread-safe
4. **Metadata Format**: Maintain compatibility with existing DeepStream metadata structure
5. **Performance Regression**: Always benchmark against current Mock backend

## Success Criteria

1. Standard backend performs actual object detection (not fakesink)
2. At least one tracking algorithm functional (Centroid minimum)
3. Achieves 15+ FPS on test videos with integrated graphics
4. Passes all existing backend tests
5. Documentation includes performance characteristics

## Notes

- Start with YOLOv5 Nano as it has best size/accuracy tradeoff
- Centroid tracker is sufficient for initial implementation
- Consider using gst-plugins-rs patterns for element creation
- Integrated graphics compute shaders can significantly boost preprocessing
- Frame skipping is acceptable under high load

## Confidence Score: 8/10

High confidence due to:
- Clear existing backend abstraction to build upon
- Well-documented OpenCV DNN module
- Existing Rust GStreamer plugin examples
- Proven algorithms (YOLO, SORT) with Rust implementations

Points deducted for:
- OpenCV-Rust binding complexity
- Potential performance tuning iterations needed
