# PRP-21: CPU-Based Object Detection Module

## Summary
Implement OpenCV DNN-based object detection for the Standard backend, replacing the current fakesink placeholder with functional CPU-optimized detection using lightweight models like YOLOv5 Nano and MobileNet SSD.

## Background
The Standard backend currently returns a fakesink element for inference, providing no actual detection capability. This PRP focuses specifically on implementing real object detection using OpenCV's DNN module, which provides optimized CPU inference for various model formats.

## Goals
1. Integrate OpenCV DNN module with Rust bindings
2. Support ONNX model format for cross-platform compatibility  
3. Implement YOLOv5 Nano as primary lightweight model
4. Create reusable detection infrastructure for Standard backend
5. Maintain GStreamer element interface compatibility

## Non-Goals
- Tracking implementation (separate PRP)
- Training or model conversion
- GPU acceleration (CPU-only focus)
- Supporting all model architectures

## Detailed Design

### Module Structure
```
crates/ds-rs/src/backend/cpu_vision/
├── mod.rs
├── detector.rs         # Core detection logic
├── models.rs          # Model loading and management
└── postprocess.rs     # Detection post-processing
```

### Core Components

#### 1. OpenCV Integration Layer
Create safe Rust wrappers around OpenCV DNN operations:

- Model loading from ONNX files
- Blob creation from frames
- Forward pass execution
- Result extraction

#### 2. Model Management
Support multiple model types with unified interface:

- **YOLOv5 Nano**: Primary model, 1.9M parameters
- **MobileNet SSD**: Backup option, 300x300 input
- Model configuration from files
- Automatic format detection

#### 3. Detection Pipeline
Frame processing workflow:

1. Receive video frame (GStreamer buffer)
2. Convert to OpenCV Mat
3. Preprocess (resize, normalize)
4. Create blob for network input
5. Run forward pass
6. Post-process outputs (NMS, filtering)
7. Convert to metadata format

#### 4. Post-Processing
Model-specific output handling:

- YOLOv5: Extract boxes from [1, 25200, 85] output
- MobileNet: Parse [1, 1, 100, 7] detection format
- Apply confidence thresholding
- Non-maximum suppression
- Coordinate transformation

### Integration Points

#### Standard Backend Enhancement
Modify `backend/standard.rs`:

```rust
fn create_inference(&self, name: Option<&str>, config_path: &str) -> Result<gst::Element> {
    // Load model configuration
    let config = DetectionConfig::from_file(config_path)?;
    
    // Create detection element
    let detector = CpuDetector::new(name, config)?;
    
    Ok(detector.to_gst_element())
}
```

#### Metadata Compatibility
Ensure output matches existing metadata structures:
- Use existing BoundingBox types
- Maintain label mapping format
- Compatible confidence scoring

## Implementation Plan

### Step 1: OpenCV Setup
1. Add opencv dependency with DNN feature
2. Verify OpenCV 4.x installation
3. Create basic DNN wrapper module
4. Test model loading capability

### Step 2: YOLOv5 Integration
1. Download YOLOv5n.onnx model
2. Implement YOLO preprocessing
3. Add YOLO post-processing  
4. Verify detection outputs

### Step 3: Detection Module
1. Create Detector trait
2. Implement YOLOv5Detector
3. Add configuration parsing
4. Write unit tests

### Step 4: GStreamer Integration
1. Create detection element wrapper
2. Handle buffer conversion
3. Attach metadata to buffers
4. Test in pipeline

### Step 5: Optimization
1. Profile performance bottlenecks
2. Add frame resize optimization
3. Implement batch processing
4. Tune confidence thresholds

## Testing Requirements

### Unit Tests
- Model loading from ONNX
- Preprocessing correctness
- Post-processing accuracy
- Metadata generation

### Integration Tests
- Pipeline with test video
- Detection accuracy validation
- Performance benchmarks
- Memory leak checks

### Test Data
- Use existing test videos from source-videos crate
- COCO validation images for accuracy
- Synthetic test patterns

## Validation Gates

```bash
# Lint and format
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings

# Unit tests
cargo test cpu_vision::detector --all-features

# Integration test
cargo test --test detection_integration

# Performance test
cargo run --example detection_benchmark
```

## Resources

### Model Files
- YOLOv5n ONNX: https://github.com/ultralytics/yolov5/releases/download/v7.0/yolov5n.onnx
- Export guide: https://github.com/ultralytics/yolov5/issues/251

### Documentation
- OpenCV DNN: https://docs.opencv.org/4.x/d6/d0f/group__dnn.html
- Rust bindings: https://docs.rs/opencv/latest/opencv/dnn/index.html
- ONNX format: https://onnx.ai/

### Code References
- Standard backend: `crates/ds-rs/src/backend/standard.rs:95-104`
- Inference config: `crates/ds-rs/src/inference/config.rs`
- Label maps: `crates/ds-rs/src/inference/mod.rs:173-176`

### Example Implementations
- object-detection-opencv-rust crate: https://github.com/LdDl/object-detection-opencv-rust
- YOLOv5 Rust: https://github.com/bencevans/rust-opencv-yolov5

## Performance Targets

### Required
- 20+ FPS on single stream (640x480)
- < 200ms initial model load
- < 300MB memory usage
- < 40% CPU on quad-core

### Stretch Goals
- 30+ FPS with optimization
- Batch processing support
- Multi-threaded inference
- SIMD preprocessing

## Risk Mitigation

### OpenCV Compatibility
- Test with OpenCV 4.5+ and 4.9
- Fallback to pure Rust implementation if needed
- Document OpenCV installation clearly

### Model Format Issues
- Verify ONNX opset compatibility
- Provide model conversion scripts
- Include pre-converted models

### Performance Concerns
- Start with lower resolution (416x416)
- Implement frame skipping
- Add configurable quality levels

## Success Criteria

1. YOLOv5 Nano detection working at 20+ FPS
2. Correct bounding box generation
3. Standard backend tests pass with new detector
4. Memory usage within targets
5. Documentation with benchmarks

## Dependencies

This PRP depends on:
- OpenCV 4.x with DNN module installed
- ONNX model files available
- Existing backend abstraction

This PRP blocks:
- PRP-22 (CPU Tracking Module)
- Full Standard backend functionality

## Notes

- Focus on YOLOv5 Nano first, add other models later
- OpenCV DNN is optimized for Intel CPUs
- Consider OpenVINO backend for Intel integrated graphics
- Frame preprocessing is often the bottleneck

## Confidence Score: 9/10

High confidence because:
- OpenCV DNN is mature and well-documented
- YOLOv5 ONNX export is straightforward
- Clear integration points in existing code
- Focused scope (detection only)

Minor concerns:
- OpenCV Rust bindings learning curve