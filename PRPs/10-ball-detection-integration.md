# PRP: Ball Detection Integration using Computer Vision

## Executive Summary

Integrate computer vision capabilities into the ds-rs inference pipeline to detect bouncing balls in video streams. This PRP establishes the foundation for object detection by implementing OpenCV-based circle detection that works with the existing inference infrastructure, specifically targeting the bouncing ball test pattern available in source-videos.

## Problem Statement

### Current State
- source-videos provides a bouncing ball test pattern (available at rtsp://0.0.0.0:8554/test2)
- ds-rs has comprehensive inference infrastructure (DetectionResult, ObjectMeta, BoundingBox)
- No actual computer vision detection capabilities integrated
- Mock metadata generation used for testing inference pipeline
- Existing inference processor expects external model outputs

### Desired State
- Real computer vision detection of bouncing balls in video streams
- Integration with existing DetectionResult and ObjectMeta structures
- Support for configurable detection parameters (sensitivity, color thresholds)
- Real-time performance suitable for video stream processing
- Proper integration with existing backend abstraction system

### Business Value
Enables practical demonstration of AI-powered video analytics with visible, trackable objects, providing a foundation for more complex object detection scenarios and validating the complete inference-to-visualization pipeline.

## Requirements

### Functional Requirements

1. **Circle Detection**: Implement Hough circle transform for detecting circular objects (balls)
2. **Color-based Filtering**: Add color thresholding to improve ball detection accuracy
3. **Inference Integration**: Connect detection results to existing DetectionResult structures
4. **Backend Compatibility**: Work with all three backend types (Mock, Standard, DeepStream)
5. **Configuration System**: Provide tunable parameters for detection sensitivity
6. **Performance Optimization**: Achieve real-time detection rates (>15 FPS)
7. **Error Handling**: Graceful handling of detection failures or edge cases

### Non-Functional Requirements

1. **Performance**: Real-time processing with minimal latency impact
2. **Accuracy**: Reliable detection of balls in various positions and lighting
3. **Configurability**: Adjustable detection parameters without code changes
4. **Memory Efficiency**: Minimal memory overhead for CV processing
5. **Thread Safety**: Safe integration with existing multi-threaded pipeline

### Context and Research

The source-videos crate provides TestPattern::Ball (pattern 18) described as "Moving ball animation" specifically designed for "Testing motion detection and tracking". This pattern generates a bouncing ball that moves predictably, making it ideal for detection algorithm development.

OpenCV provides HoughCircles() function using the HOUGH_GRADIENT method for efficient circle detection. Combined with color thresholding in HSV space, this approach is proven effective for ball detection with real-time performance.

The ds-rs inference infrastructure already supports DetectionResult with ObjectMeta containing BoundingBox coordinates, confidence scores, and class IDs - exactly what's needed for computer vision integration.

### Documentation & References
```yaml
- file: crates/source-videos/src/patterns.rs
  why: TestPattern::Ball implementation and GStreamer pattern mapping

- file: crates/ds-rs/src/inference/mod.rs
  why: Existing DetectionResult and InferenceProcessor integration patterns

- file: crates/ds-rs/src/metadata/object.rs
  why: ObjectMeta and BoundingBox structures for detection results

- url: https://docs.opencv.org/3.4/d4/d70/tutorial_hough_circle.html
  why: OpenCV Hough Circle Transform documentation and parameters

- url: https://docs.rs/opencv/latest/opencv/imgproc/
  why: Rust OpenCV bindings for image processing functions

- file: crates/ds-rs/src/backend/mod.rs
  why: Backend abstraction patterns for cross-platform compatibility

- url: https://pyimagesearch.com/2015/09/14/ball-tracking-with-opencv/
  why: Proven ball detection techniques using color thresholding and circle detection

- file: crates/ds-rs/examples/detection_app.rs
  why: Existing pattern for processing inference results and metadata extraction
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
ADD opencv dependency to Cargo.toml:
  - ADD opencv crate to workspace dependencies
  - CONFIGURE feature flags for required OpenCV modules
  - HANDLE platform-specific OpenCV installation requirements
  - UPDATE build documentation for OpenCV dependencies

Task 2:
CREATE src/vision/mod.rs:
  - DEFINE ComputerVision trait for detection algorithms
  - IMPLEMENT OpenCVDetector struct
  - ADD configuration structures for detection parameters
  - INCLUDE error handling for CV operations

Task 3:
CREATE src/vision/ball_detector.rs:
  - IMPLEMENT BallDetector using Hough circle detection
  - ADD color-based filtering in HSV space
  - CONFIGURE detection parameters (min/max radius, sensitivity)
  - OPTIMIZE for real-time performance

Task 4:
CREATE src/vision/config.rs:
  - DEFINE BallDetectionConfig with tunable parameters
  - ADD HSV color range specifications
  - INCLUDE circle detection thresholds
  - SUPPORT loading configuration from files

Task 5:
MODIFY src/inference/mod.rs:
  - EXTEND InferenceProcessor to support computer vision models
  - ADD integration with BallDetector
  - CONVERT CV detection results to DetectionResult format
  - MAINTAIN compatibility with existing inference patterns

Task 6:
CREATE src/backend/cv_integration.rs:
  - IMPLEMENT CV processing in GStreamer pipeline context
  - ADD buffer format conversion (GStreamer to OpenCV Mat)
  - HANDLE different backend requirements
  - ENSURE thread-safe buffer processing

Task 7:
MODIFY src/elements/factory.rs:
  - ADD computer vision element creation
  - INTEGRATE CV detection with pipeline elements
  - HANDLE CV processing as inference alternative
  - SUPPORT backend-specific CV implementations

Task 8:
CREATE examples/ball_detection_demo.rs:
  - DEMONSTRATE ball detection with source-videos RTSP stream
  - SHOW integration with existing pipeline infrastructure
  - INCLUDE configuration examples
  - VALIDATE real-time performance

Task 9:
UPDATE tests/cv_integration_tests.rs:
  - ADD tests for ball detection accuracy
  - VALIDATE integration with existing inference system
  - TEST configuration parameter effects
  - INCLUDE performance benchmarks

Task 10:
UPDATE configuration files:
  - ADD ball detection configuration section
  - INCLUDE example parameters for different scenarios
  - DOCUMENT parameter tuning guidelines
  - PROVIDE troubleshooting guidance
```

### Out of Scope
- Advanced machine learning models (YOLO, CNN-based detection)
- Real-time training or adaptive detection parameters
- Multi-object detection beyond balls
- Integration with external inference services

## Success Criteria

- [ ] Detects bouncing ball in source-videos test pattern with >90% accuracy
- [ ] Integrates seamlessly with existing DetectionResult infrastructure
- [ ] Achieves real-time performance (>15 FPS) on standard hardware
- [ ] Works with all three backend types (Mock, Standard, DeepStream)
- [ ] Provides configurable detection parameters
- [ ] Handles edge cases gracefully (no ball present, multiple balls)
- [ ] Example application successfully processes rtsp://0.0.0.0:8554/test2 stream

## Dependencies

### Technical Dependencies
- OpenCV 4.x library with Rust bindings
- Existing ds-rs inference infrastructure
- source-videos RTSP server for testing
- GStreamer buffer format conversion capabilities

### Knowledge Dependencies
- OpenCV Hough circle detection algorithms
- HSV color space conversion and thresholding
- GStreamer buffer format handling
- Rust OpenCV bindings API

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| OpenCV installation complexity | High | Medium | Provide detailed setup documentation, consider containerization |
| Performance bottlenecks in CV processing | Medium | High | Implement performance profiling, optimize critical paths |
| GStreamer buffer format incompatibilities | Medium | Medium | Research buffer formats, implement robust conversion |
| Detection accuracy variations | Medium | Medium | Provide tunable parameters, extensive testing with different scenarios |

## Architecture Decisions

### Decision: OpenCV Integration Approach
**Options Considered:**
1. Direct OpenCV Rust bindings integration
2. Custom GStreamer plugin with OpenCV
3. External process for CV with IPC communication

**Decision:** Option 1 - Direct OpenCV Rust bindings

**Rationale:** Provides best performance and integration with existing Rust codebase while maintaining type safety

### Decision: Detection Algorithm Choice
**Options Considered:**
1. Hough circle detection only
2. Color thresholding + circle detection
3. Template matching approach

**Decision:** Option 2 - Combined color and circle detection

**Rationale:** Provides best accuracy for ball detection while maintaining real-time performance

## Validation Strategy

### Validation Commands
```bash
# Build with OpenCV dependencies
cargo build --features opencv

# Run ball detection tests
cargo test --features opencv ball_detection

# Test with source-videos RTSP stream
cargo run --example ball_detection_demo rtsp://127.0.0.1:8554/test2

# Benchmark detection performance
cargo bench --features opencv ball_detection_bench
```

## Future Considerations

- Integration with tracking algorithms for motion prediction
- Support for multiple ball colors and sizes
- Extension to other geometric shapes detection
- Machine learning model integration for improved accuracy
- Real-time parameter adjustment based on detection performance

## References

- OpenCV Circle Detection Documentation
- Rust OpenCV Bindings API Reference
- GStreamer Buffer Format Specifications
- Computer Vision Ball Tracking Techniques

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Draft
- **Confidence Level**: 7 - OpenCV integration requires careful setup but detection algorithms are well-established