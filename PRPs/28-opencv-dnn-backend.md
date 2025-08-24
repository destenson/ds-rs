# PRP-28: OpenCV DNN Backend Implementation

## Executive Summary

Implement OpenCV DNN as an alternative detection backend to ONNX Runtime, leveraging OpenCV's optimized CPU inference capabilities and broad model format support. This provides users with a well-integrated option when already using OpenCV for image processing.

## Problem Statement

### Current State
The system only supports ONNX Runtime for object detection. Users who already have OpenCV in their pipeline face redundant dependencies, and some optimized OpenCV models cannot be used directly.

### Desired State
OpenCV DNN backend available as a compile-time or runtime option, supporting various model formats (ONNX, Caffe, TensorFlow, Darknet) with OpenCV's CPU optimizations, especially beneficial on Intel processors.

### Business Value
- Reduces dependencies for OpenCV-heavy applications
- Leverages Intel CPU optimizations (OpenVINO backend)
- Supports legacy model formats (Caffe, Darknet weights)
- Provides fallback option if ONNX Runtime unavailable

## Requirements

### Functional Requirements

1. **OpenCV DNN Integration**: Implement detector using OpenCV's dnn module
2. **Multi-Format Support**: Load models in ONNX, Caffe, TensorFlow, and Darknet formats
3. **Preprocessing Pipeline**: Use OpenCV's blobFromImage for preprocessing
4. **Backend Selection**: Support OpenCV inference backends (CPU, OpenCL, OpenVINO)
5. **Compatibility**: Maintain identical Detection output format

### Non-Functional Requirements

1. **Performance**: Within 20% of ONNX Runtime performance on CPU
2. **Memory**: Efficient memory usage with OpenCV Mat operations
3. **Thread Safety**: Safe concurrent inference support
4. **Error Handling**: Graceful fallback when models unsupported

### Context and Research

OpenCV DNN provides highly optimized inference on Intel CPUs through OpenVINO backend. While it may be slower than ONNX Runtime in some cases, it excels when the entire pipeline uses OpenCV, avoiding data conversion overhead.

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- file: ../opencv/modules/dnn/samples/object_detection.cpp
  why: OpenCV DNN object detection example patterns

- url: https://docs.opencv.org/4.x/d2/d58/tutorial_table_of_content_dnn.html
  why: OpenCV DNN module documentation and tutorials

- url: https://learnopencv.com/deep-learning-with-opencvs-dnn-module-a-definitive-guide/
  why: Comprehensive guide on OpenCV DNN usage patterns

- file: crates/cpuinfer/Cargo.toml
  why: Check opencv-dnn feature flag already defined

- file: ../darknet-rust/src/lib.rs
  why: Reference for Darknet weight loading patterns

- url: https://github.com/opencv/opencv/wiki/Deep-Learning-in-OpenCV
  why: Supported layers and model compatibility guide
```

### List of tasks to be completed to fulfill the PRP in the order they should be completed

```yaml
Task 1:
CREATE crates/cpuinfer/src/opencv_detector.rs:
  - MIRROR pattern from: crates/cpuinfer/src/detector.rs
  - DEFINE OpenCVDetector struct with opencv::dnn::Net
  - IMPLEMENT model loading for multiple formats
  - USE opencv::dnn::blobFromImage for preprocessing

Task 2:
IMPLEMENT trait Detector for OpenCVDetector:
  - IMPLEMENT detect() method using Net::forward()
  - CONVERT opencv::Mat outputs to Detection structs
  - HANDLE different output formats (YOLO, SSD, etc.)
  - APPLY NMS using opencv::dnn::NMSBoxes

Task 3:
CREATE crates/cpuinfer/src/opencv_backends.rs:
  - ENUMERATE available OpenCV backends (CPU, OpenCL, OpenVINO)
  - IMPLEMENT backend selection logic
  - ADD performance hints configuration
  - VERIFY backend availability at runtime

Task 4:
MODIFY crates/cpuinfer/src/detector_factory.rs:
  - ADD OpenCV backend case
  - CHECK opencv feature flag
  - FALLBACK to ONNX if OpenCV unavailable
  - PARSE OpenCV-specific configuration

Task 5:
CREATE tests/opencv_detector_tests.rs:
  - TEST model loading for each format
  - VERIFY detection accuracy matches ONNX
  - BENCHMARK performance vs ONNX Runtime
  - TEST thread safety with concurrent inference

Task 6:
UPDATE documentation:
  - DOCUMENT OpenCV installation requirements
  - ADD examples for each model format
  - EXPLAIN backend selection criteria
  - PROVIDE performance tuning guide
```

### Out of Scope
- Training models with OpenCV
- Video processing with OpenCV (use GStreamer)
- GPU acceleration (focus on CPU optimization)
- Custom layer implementation

## Success Criteria

- [ ] OpenCV detector passes all existing tests
- [ ] Supports at least ONNX and Darknet formats
- [ ] Performance within 20% of ONNX Runtime
- [ ] Seamless switching between backends
- [ ] Documentation includes migration guide

## Dependencies

### Technical Dependencies
- OpenCV 4.5+ with dnn module
- opencv-rust crate with dnn features
- Existing Detector trait (from PRP-27)

### Knowledge Dependencies
- OpenCV DNN API understanding
- Model format specifications
- OpenCV build configuration

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| OpenCV version incompatibilities | Medium | High | Support multiple OpenCV versions, document requirements |
| Performance regression | Medium | Medium | Benchmark early, optimize blob creation |
| Missing layer support | Low | Medium | Document supported architectures, provide fallback |
| Build complexity | High | Low | Provide pre-built binaries, clear build instructions |

## Architecture Decisions

### Decision: OpenCV Mat vs ndarray
**Options Considered:**
1. Use OpenCV Mat throughout
2. Convert to ndarray for consistency
3. Hybrid approach based on operation

**Decision:** Use OpenCV Mat internally, convert only at boundaries

**Rationale:** Avoids conversion overhead within OpenCV operations while maintaining consistent external interface.

## Validation Strategy

- **Unit Testing**: Test each model format loading
- **Integration Testing**: Verify with GStreamer pipeline
- **Performance Testing**: Benchmark against ONNX Runtime
- **Compatibility Testing**: Test on different OpenCV versions

## Future Considerations

- OpenVINO model optimizer integration
- INT8 quantization support
- Model caching for faster loading
- Custom layer plugin support

## References

- OpenCV DNN Module Documentation
- OpenVINO Toolkit Documentation
- Darknet to OpenCV conversion guides

---

## PRP Metadata

- **Author**: AI Assistant
- **Created**: 2025-08-24
- **Last Modified**: 2025-08-24
- **Status**: Draft
- **Confidence Level**: 8/10 - OpenCV integration well-documented, some uncertainty on performance parity