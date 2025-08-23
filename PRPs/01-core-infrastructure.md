# PRP: Core Infrastructure Setup for DeepStream Rust Port

## Executive Summary

Establish the foundational infrastructure for porting the NVIDIA DeepStream runtime source add/delete application from C to Rust. This PRP sets up the project structure, dependencies, and basic FFI bindings necessary for subsequent development phases.

## Problem Statement

### Current State
- Existing C implementation in `vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete`
- Workspace structure exists with basic Cargo.toml and gstreamer dependency
- No DeepStream-specific Rust bindings or infrastructure

### Desired State
- Complete Rust project structure with proper module organization
- Essential DeepStream FFI bindings generated and integrated
- Build system configured for NVIDIA GPU/Jetson environments
- Foundation ready for implementing GStreamer pipeline functionality

### Business Value
Provides a modern, memory-safe implementation with Rust's safety guarantees while maintaining compatibility with NVIDIA's DeepStream SDK for AI video analytics applications.

## Requirements

### Functional Requirements

1. **Project Structure**: Establish modular Rust project architecture supporting DeepStream components
2. **FFI Bindings**: Generate and integrate essential DeepStream SDK bindings
3. **Build Configuration**: Support both x86 and Jetson platforms with appropriate CUDA versions
4. **Type Safety**: Create safe Rust wrappers for core DeepStream types
5. **Error Handling**: Implement comprehensive error handling framework

### Non-Functional Requirements

1. **Performance**: Zero-cost abstractions maintaining C-level performance
2. **Compatibility**: Support DeepStream SDK 6.0+ and GStreamer 1.14+
3. **Platform Support**: Build on both x86_64 (CUDA 11.4+) and Jetson (CUDA 10.2+)
4. **Maintainability**: Clear module boundaries and documentation

### Context and Research
Based on analysis of the C implementation, the application uses GStreamer, NVIDIA DeepStream SDK components (nvstreammux, nvinfer, nvtracker, nvdsosd), and CUDA runtime APIs for dynamic source management.

### Documentation & References
```yaml
- url: https://github.com/aosoft/nvidia-deepstream-rs
  why: Existing DeepStream Rust bindings to build upon

- url: https://github.com/GStreamer/gstreamer-rs
  why: Official GStreamer Rust bindings documentation

- url: https://docs.nvidia.com/metropolis/deepstream/dev-guide/
  why: DeepStream SDK developer guide for API reference

- file: vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete\deepstream_test_rt_src_add_del.c
  why: Original C implementation to reference

- file: vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete\Makefile
  why: Build configuration and library dependencies

- url: https://github.com/rust-lang/rust-bindgen
  why: Tool for generating FFI bindings from C headers
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
MODIFY Cargo.toml:
  - ADD nvidia-deepstream-rs dependency with git source
  - ADD gstreamer-app, gstreamer-video dependencies
  - ADD build-dependencies for bindgen
  - ADD platform-specific features for x86/jetson

Task 2:
CREATE build.rs:
  - DETECT platform (x86 vs Jetson) via CUDA version
  - SET library paths for DeepStream SDK
  - CONFIGURE bindgen for required headers
  - LINK required libraries (nvdsgst_meta, nvds_meta, cudart)

Task 3:
CREATE src/ffi/mod.rs:
  - GENERATE bindings for nvdsmeta.h
  - GENERATE bindings for gst-nvmessage.h
  - WRAP unsafe functions with safe interfaces

Task 4:
CREATE src/error.rs:
  - DEFINE custom error types for DeepStream operations
  - IMPLEMENT From traits for GStreamer errors
  - ADD error context and debugging helpers

Task 5:
CREATE src/config/mod.rs:
  - DEFINE configuration structures
  - PARSE inference config files (pgie, sgie, tracker)
  - VALIDATE configuration parameters

Task 6:
MODIFY src/lib.rs:
  - EXPORT public modules
  - SETUP module hierarchy
  - ADD feature flags for optional components
```

### Out of Scope
- Actual pipeline implementation (covered in next PRP)
- Runtime source manipulation logic
- UI/visualization components
- Model inference configuration tuning

## Success Criteria

- [ ] Project builds successfully on both x86 and Jetson platforms
- [ ] FFI bindings compile without warnings
- [ ] Basic DeepStream types accessible from Rust
- [ ] Error handling framework in place
- [ ] Build system detects and configures for target platform

## Dependencies

### Technical Dependencies
- NVIDIA DeepStream SDK 6.0+
- GStreamer 1.14+
- CUDA Toolkit (10.2 for Jetson, 11.4+ for x86)
- Rust 1.70+ with cargo

### Knowledge Dependencies
- Understanding of FFI and unsafe Rust
- DeepStream SDK architecture
- GStreamer fundamentals

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Incomplete DeepStream bindings | Medium | High | Use nvidia-deepstream-rs as base, generate additional as needed |
| Platform-specific build issues | Medium | Medium | Implement comprehensive build.rs detection logic |
| Version compatibility issues | Low | High | Pin specific DeepStream/GStreamer versions |

## Architecture Decisions

### Decision: FFI Binding Strategy
**Options Considered:**
1. Manual FFI bindings
2. Use nvidia-deepstream-rs + custom bindgen
3. Pure bindgen generation

**Decision:** Option 2 - Leverage existing work and extend as needed

**Rationale:** Balances development speed with control over critical bindings

### Decision: Error Handling Approach
**Options Considered:**
1. Direct Result<T, E> propagation
2. Custom error types with context
3. anyhow/thiserror crates

**Decision:** Option 2 with thiserror for derive macros

**Rationale:** Provides rich error context while maintaining type safety

## Validation Strategy

- **Build Testing**: Verify compilation on both target platforms
- **FFI Testing**: Validate binding correctness with simple DeepStream calls
- **Integration Testing**: Basic GStreamer pipeline creation

## Future Considerations

- Adding more DeepStream plugin bindings as needed
- Potential contribution back to nvidia-deepstream-rs
- Performance profiling and optimization opportunities

## References

- DeepStream SDK Documentation: https://docs.nvidia.com/metropolis/deepstream/
- GStreamer Application Development Manual
- Rust FFI Omnibus

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-22
- **Last Modified**: 2025-08-22
- **Status**: Draft
- **Confidence Level**: 8 - Strong foundation based on existing bindings and clear C reference
