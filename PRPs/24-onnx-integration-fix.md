# PRP-24: ONNX Runtime Integration Fix and Best Practices

## Executive Summary

Fix the ONNX Runtime (ort) integration in the ds-rs project to use the correct API for version 1.16.3. The current implementation attempts to use deprecated or non-existent API methods (`OrtOwnedTensor::from_shape_vec` and `Value::as_slice`). This PRP provides a comprehensive approach to properly integrate ONNX models for CPU-based object detection while minimizing type conversions and using native ort types directly.

## Problem Statement

### Current State
- ONNX model loading is implemented but uses incorrect API methods
- `OrtOwnedTensor::from_shape_vec` doesn't exist in ort 1.16.3
- `Value::as_slice()` method is not available
- Compilation fails when ort feature is enabled
- Unnecessary type conversions between different tensor representations

### Desired State
- Working ONNX inference using ort 1.16.3's correct API
- Minimal type conversions - use ort native types directly
- Efficient tensor creation and data extraction
- Support for common object detection models (YOLO, SSD)
- Clear error handling and debugging capabilities

### Business Value
Enables CPU-based computer vision on 90%+ of systems without NVIDIA hardware, providing object detection and tracking capabilities for the Standard backend.

## Requirements

### Functional Requirements

1. **Correct Tensor Creation**: Use ort 1.16.3's proper tensor creation methods
2. **Efficient Data Handling**: Minimize copies and conversions
3. **Model Compatibility**: Support YOLO and similar detection models
4. **Performance**: Achieve 15+ FPS on standard hardware for 640x640 inference
5. **Error Handling**: Proper error messages for debugging model issues

### Non-Functional Requirements

1. **Type Safety**: Use ort's native types without unnecessary conversions
2. **Memory Efficiency**: Avoid unnecessary allocations and copies
3. **Maintainability**: Clear code with proper documentation
4. **Testability**: Unit tests for tensor operations

### Context and Research

Based on the actual ort v1.16.3 source code in `../pykeio--ort`, the correct API patterns are:

**Current (incorrect) approach:**
- Uses `OrtOwnedTensor::from_shape_vec` (doesn't exist in public API)
- Attempts `Value::as_slice()` (not available)
- Tries to create tensors without allocator

**Correct v1.16.3 approach (from examples/gpt.rs):**
- Use `Value::from_array(session.allocator(), &array)` - requires session allocator
- Use `outputs[0].try_extract::<f32>()` to get `OrtOwnedTensor<f32, _>`
- Access data via `.view()` method on `OrtOwnedTensor`
- Work with `CowArray` for efficient memory management

### Documentation & References

```yaml
- file: ../pykeio--ort/src/value.rs
  why: Actual Value::from_array implementation showing allocator requirement

- file: ../pykeio--ort/examples/gpt.rs
  why: Working example showing correct tensor creation and extraction patterns

- file: ../pykeio--ort/src/tensor/ort_owned_tensor.rs
  why: Understanding OrtOwnedTensor and view() method

- file: crates/ds-rs/src/backend/cpu_vision/detector.rs
  why: Current implementation that needs fixing

- file: crates/ds-rs/Cargo.toml
  why: Feature flags and dependency versions

- url: https://docs.rs/ndarray/latest/ndarray/type.CowArray.html
  why: Understanding CowArray for efficient memory management

- url: https://github.com/microsoft/onnxruntime/blob/main/docs/python/inference/api_summary.md
  why: ONNX Runtime concepts applicable to Rust bindings
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
RESEARCH ort 1.16.3 API:
  - STUDY Value creation methods in docs.rs/ort/1.16.3
  - FIND correct tensor creation pattern for v1.16.3
  - IDENTIFY extraction methods for outputs
  - UNDERSTAND memory management and allocator usage
  - VERIFY ndarray integration if available

Task 2:
FIX tensor creation in detector.rs:
  - STORE session.allocator() reference in OnnxDetector struct
  - CONVERT Vec<f32> to CowArray<f32, IxDyn> using ndarray
  - CALL Value::from_array(allocator, &cow_array) for tensor creation
  - ENSURE shape matches model expectations (1, 3, height, width for YOLO)
  - USE CowArray::from(array.into_dyn()) pattern for shape conversion

Task 3:
FIX output extraction in detector.rs:
  - REPLACE as_slice() with try_extract::<f32>() returning OrtOwnedTensor
  - CALL .view() on OrtOwnedTensor to get ArrayView
  - CONVERT ArrayView to Vec or slice as needed for postprocessing
  - HANDLE try_extract errors with proper error mapping
  - PRESERVE tensor shape information for correct interpretation

Task 4:
UPDATE preprocessing pipeline:
  - ENSURE image data is in correct format (CHW vs HWC)
  - NORMALIZE pixel values correctly (0-1 or specific model requirements)
  - USE efficient resizing with correct interpolation
  - MINIMIZE allocations during preprocessing

Task 5:
UPDATE postprocessing pipeline:
  - PARSE YOLO output format correctly (depends on model version)
  - IMPLEMENT proper NMS with configurable thresholds
  - MAP predictions back to original image coordinates
  - HANDLE different model output formats (YOLO, SSD, etc.)

Task 6:
ADD model configuration:
  - CREATE config structure for model parameters
  - SUPPORT different input/output shapes
  - ALLOW runtime configuration of thresholds
  - DOCUMENT expected model format

Task 7:
IMPLEMENT proper error handling:
  - ADD detailed error messages for debugging
  - VALIDATE model inputs/outputs
  - CHECK tensor shapes and types
  - PROVIDE helpful error context

Task 8:
ADD unit tests:
  - TEST tensor creation with different shapes
  - TEST extraction methods
  - TEST preprocessing/postprocessing
  - MOCK inference for testing without models
  - VERIFY memory efficiency

Task 9:
ADD integration tests:
  - TEST with actual YOLO model if available
  - BENCHMARK inference performance
  - VALIDATE detection results
  - TEST error cases

Task 10:
UPDATE documentation:
  - DOCUMENT API usage patterns
  - ADD examples for common use cases
  - EXPLAIN model requirements
  - PROVIDE troubleshooting guide
```

### Out of Scope
- Training or fine-tuning models
- Model conversion from other formats
- GPU acceleration (separate PRP if needed)
- Video codec handling

## Key Implementation Notes

### Critical API Differences from Documentation
1. **Allocator is mandatory** - Value::from_array requires session.allocator() as first parameter
2. **Use CowArray not raw Vec** - Convert data to CowArray before Value::from_array
3. **Extract returns OrtOwnedTensor** - Call .view() to access underlying ArrayView
4. **Shape must be IxDyn** - Use .into_dyn() on arrays before passing to Value::from_array

### Common Pitfalls to Avoid
- Don't try to use OrtOwnedTensor::from_shape_vec - it's not public API
- Don't look for as_slice() method - use try_extract() then .view()
- Don't forget the allocator parameter in Value::from_array
- Don't pass raw Vec to Value::from_array - convert to CowArray first

## Success Criteria

- [ ] ONNX inference compiles and runs with ort feature enabled
- [ ] Correct tensor creation using Value::from_array with allocator
- [ ] Proper output extraction using try_extract() and view()
- [ ] Minimal type conversions - uses ort native types
- [ ] 15+ FPS inference on 640x640 images (CPU)
- [ ] Unit tests pass for all tensor operations
- [ ] Integration test with real YOLO model succeeds
- [ ] No unnecessary memory allocations or copies
- [ ] Clear error messages for debugging

## Dependencies

### Technical Dependencies
- ort crate version 1.16.3 (already in Cargo.toml)
- ndarray for efficient array operations (optional feature)
- image crate for preprocessing (already present)
- ONNX model files for testing (YOLO v5/v8 nano recommended)

### Knowledge Dependencies
- Understanding of ONNX Runtime memory management
- YOLO output format and postprocessing requirements
- Image preprocessing standards for object detection
- Non-maximum suppression algorithms

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| API documentation incomplete | Medium | High | Study source code and examples directly |
| Performance below target | Medium | Medium | Profile and optimize hot paths, consider smaller models |
| Model compatibility issues | Low | High | Test with multiple model formats, provide clear requirements |
| Memory leaks | Low | High | Use proper RAII patterns, test with valgrind |

## Architecture Decisions

### Decision: Use Native ort Types
**Options Considered:**
1. Convert everything to/from custom types
2. Use ort types directly throughout
3. Hybrid approach with minimal conversions

**Decision:** Option 2 - Use ort types directly

**Rationale:** Minimizes conversions, reduces memory overhead, better performance

### Decision: Memory Management Strategy
**Options Considered:**
1. Copy all data for safety
2. Use zero-copy where possible
3. Let ort manage all allocations

**Decision:** Option 2 - Zero-copy where possible with ort allocator

**Rationale:** Best performance while maintaining safety through ort's memory management

### Decision: Session and Allocator Storage
**Options Considered:**
1. Create new session for each inference
2. Store session in OnnxDetector struct
3. Use Arc for shared session across threads

**Decision:** Option 2 - Store session with allocator in struct

**Rationale:** Avoids repeated initialization, allocator needed for Value creation

## Validation Strategy

### Validation Commands
```bash
# Build with ort feature
cargo build --features ort

# Run unit tests
cargo test --features ort backend::cpu_vision::detector::tests

# Run integration test with model
cargo test --features ort --test cpu_backend_tests

# Check for memory leaks (Linux)
valgrind --leak-check=full cargo test --features ort

# Benchmark inference
cargo bench --features ort inference_benchmark
```

### Performance Validation
```bash
# Measure FPS with test video
cargo run --release --features ort --example detection_app -- test_video.mp4

# Profile CPU usage
perf record -g cargo run --release --features ort --bin ds-app
perf report
```

## Future Considerations

- GPU acceleration support (CUDA, DirectML)
- Model quantization for faster inference
- Batch inference for multiple frames
- Dynamic model loading without recompilation
- Support for other model formats (TensorFlow, PyTorch)
- Model zoo integration for easy model downloads

## References

- ONNX Runtime documentation: https://onnxruntime.ai/docs/
- ort crate documentation: https://docs.rs/ort/1.16.3/
- YOLO model formats: https://github.com/ultralytics/yolov5
- Object detection benchmarks: https://paperswithcode.com/task/object-detection

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Ready for Implementation
- **Confidence Level**: 8 - Clear requirements with specific API patterns to follow, good documentation available