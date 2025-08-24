# PRP-27: Multi-Backend Detector Trait Architecture

## Executive Summary

Create a trait-based architecture for object detection that allows pluggable backends (ONNX, OpenCV DNN, TFLite, Darknet) while maintaining a consistent interface. This enables users to choose the optimal detection backend for their specific hardware and use case.

## Problem Statement

### Current State
The codebase directly implements `OnnxDetector` as a concrete struct without a trait abstraction. This tightly couples the detection logic to ONNX Runtime, making it difficult to add alternative detection backends or switch between them at runtime.

### Desired State
A flexible detector trait system where different backends (ONNX, OpenCV, TFLite, Darknet) can be plugged in transparently, selected via configuration or feature flags, with consistent detection results across all backends.

### Business Value
- Users can select the optimal backend for their hardware (CPU, GPU, mobile, edge devices)
- Developers can easily add new detection frameworks without modifying existing code
- Applications can switch backends at runtime based on available resources
- Better performance optimization for specific deployment scenarios

## Requirements

### Functional Requirements

1. **Detector Trait Definition**: Define a common trait that all detection backends must implement
2. **Backend Selection**: Allow runtime and compile-time backend selection
3. **Consistent Results**: Ensure all backends produce compatible Detection structs
4. **Configuration Support**: Allow backend-specific configuration while maintaining common interface
5. **Error Handling**: Unified error handling across all backends

### Non-Functional Requirements

1. **Performance**: Trait abstraction should add minimal overhead (<1% performance impact)
2. **Memory**: No additional memory allocations beyond what backends require
3. **Compatibility**: Work with existing GStreamer element architecture
4. **Extensibility**: Easy to add new backends without breaking existing code

### Context and Research

Based on research, ONNX Runtime provides the best general CPU performance, but TFLite excels on mobile devices, OpenCV DNN integrates well with existing OpenCV pipelines, and Darknet provides native YOLO support. The trait architecture allows optimal backend selection per use case.

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- file: crates/cpuinfer/src/detector.rs
  why: Current OnnxDetector implementation to understand existing patterns

- file: crates/ds-rs/src/backend/cpu_vision/mod.rs
  why: Current module structure that needs trait integration

- url: https://github.com/PaulKlinger/ioutrack
  why: Example of Rust trait-based tracking architecture

- url: https://doc.rust-lang.org/book/ch10-02-traits.html
  why: Rust trait patterns and best practices

- file: crates/ds-rs/src/backend/mod.rs
  why: Backend abstraction patterns already in use
```

### List of tasks to be completed to fulfill the PRP in the order they should be completed

```yaml
Task 1:
CREATE crates/cpuinfer/src/detector_trait.rs:
  - DEFINE Detector trait with detect(), set_confidence_threshold(), set_nms_threshold()
  - DEFINE DetectorBackend enum (Onnx, OpenCV, TFLite, Darknet)
  - DEFINE BoxedDetector type alias for Box<dyn Detector>

Task 2:
MODIFY crates/cpuinfer/src/detector.rs:
  - FIND struct OnnxDetector
  - ADD impl Detector for OnnxDetector
  - PRESERVE all existing methods
  - ENSURE trait methods delegate to existing implementation

Task 3:
CREATE crates/cpuinfer/src/detector_factory.rs:
  - MIRROR pattern from: crates/ds-rs/src/elements/factory.rs
  - CREATE DetectorFactory struct
  - IMPLEMENT create_detector() method with backend selection
  - ADD configuration parsing for backend selection

Task 4:
MODIFY crates/cpuinfer/src/lib.rs:
  - ADD pub mod detector_trait
  - ADD pub mod detector_factory
  - EXPORT Detector trait and DetectorFactory

Task 5:
MODIFY crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs:
  - FIND detector: Arc<Mutex<Option<OnnxDetector>>>
  - REPLACE with detector: Arc<Mutex<Option<Box<dyn Detector>>>>
  - UPDATE initialize_detector() to use DetectorFactory

Task 6:
CREATE tests/detector_trait_tests.rs:
  - TEST trait implementation for OnnxDetector
  - TEST factory creates correct backend
  - TEST backends are interchangeable
  - VERIFY performance overhead is minimal
```

### Out of Scope
- Implementing the actual alternative backends (covered in separate PRPs)
- Model conversion between formats
- GPU acceleration specifics
- Training or fine-tuning models

## Success Criteria

- [ ] Detector trait compiles and passes all existing tests
- [ ] OnnxDetector works identically through trait interface
- [ ] Factory can create detectors based on configuration
- [ ] Performance regression less than 1%
- [ ] Documentation clearly explains trait usage

## Dependencies

### Technical Dependencies
- Rust trait system
- Existing detector.rs implementation
- GStreamer element architecture

### Knowledge Dependencies
- Understanding of Rust traits and dynamic dispatch
- Current detector implementation details
- Backend-specific requirements

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Performance overhead from dynamic dispatch | Medium | Medium | Use static dispatch where possible, benchmark thoroughly |
| Breaking existing code | Low | High | Implement trait for existing OnnxDetector first |
| Backend incompatibilities | Medium | Medium | Define strict trait contract with tests |

## Architecture Decisions

### Decision: Dynamic vs Static Dispatch
**Options Considered:**
1. Dynamic dispatch with Box<dyn Detector>
2. Static dispatch with generics
3. Enum-based dispatch

**Decision:** Dynamic dispatch for flexibility, with option for static dispatch in performance-critical paths

**Rationale:** Dynamic dispatch allows runtime backend selection and plugin architecture, while specific implementations can use static dispatch internally.

## Validation Strategy

- **Unit Testing**: Test each trait method independently
- **Integration Testing**: Verify backends work with GStreamer element
- **Performance Testing**: Benchmark trait overhead
- **Compatibility Testing**: Ensure all backends produce compatible results

## Future Considerations

- Plugin architecture for loading backends from external libraries
- Backend-specific optimizations and hardware acceleration
- Model format conversion utilities
- Backend capability discovery

## References

- Rust trait design patterns
- GStreamer plugin architecture
- Dynamic library loading in Rust

---

## PRP Metadata

- **Author**: AI Assistant
- **Created**: 2025-08-24
- **Last Modified**: 2025-08-24
- **Status**: Draft
- **Confidence Level**: 9/10 - Clear trait pattern with existing backend abstraction examples in codebase