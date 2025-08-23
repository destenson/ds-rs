# PRP: Core Infrastructure Setup for DeepStream Rust Port

## Executive Summary

Establish the foundational infrastructure for porting the NVIDIA DeepStream runtime source add/delete application from C to Rust. This PRP sets up the project structure, dependencies, and DeepStream GStreamer element integration necessary for subsequent development phases.

## Problem Statement

### Current State
- Existing C implementation in `vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete`
- Workspace structure exists with basic Cargo.toml and gstreamer dependency
- No DeepStream element usage implemented yet

### Desired State
- Complete Rust project structure with proper module organization
- DeepStream GStreamer elements accessible through gstreamer-rs
- Configuration system for element properties
- Foundation ready for implementing GStreamer pipeline functionality

### Business Value
Provides a modern, memory-safe implementation with Rust's safety guarantees while maintaining compatibility with NVIDIA's DeepStream SDK for AI video analytics applications.

## Requirements

### Functional Requirements

1. **Project Structure**: Establish modular Rust project architecture supporting DeepStream components
2. **Element Creation**: Use DeepStream GStreamer elements through gstreamer-rs
3. **Configuration Parsing**: Load and apply DeepStream element configuration files
4. **Platform Detection**: Support both x86 and Jetson platforms
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
- url: https://github.com/GStreamer/gstreamer-rs
  why: Official GStreamer Rust bindings for element creation

- url: https://docs.nvidia.com/metropolis/deepstream/dev-guide/
  why: DeepStream element properties and configuration

- file: vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete\deepstream_test_rt_src_add_del.c
  why: Original C implementation showing element usage

- file: vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete\*.txt
  why: DeepStream element configuration files to parse
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
MODIFY Cargo.toml:
  - ADD gstreamer-app, gstreamer-video dependencies
  - ADD serde, toml for config file parsing
  - ADD thiserror for error handling
  - ADD platform-specific features for x86/jetson

Task 2:
CREATE src/elements/mod.rs:
  - DEFINE DeepStreamElement trait
  - CREATE factory functions for DeepStream elements
  - IMPLEMENT element property setters
  - HANDLE element creation errors

Task 3:
CREATE src/elements/factory.rs:
  - WRAP gst::ElementFactory::make for DeepStream elements
  - VALIDATE element availability
  - PROVIDE typed element creation (nvstreammux, nvinfer, etc.)

Task 4:
CREATE src/error.rs:
  - DEFINE custom error types for DeepStream operations
  - IMPLEMENT From traits for GStreamer errors
  - ADD error context and debugging helpers

Task 5:
CREATE src/config/mod.rs:
  - DEFINE configuration structures
  - PARSE inference config files (pgie, sgie, tracker)
  - APPLY properties to GStreamer elements
  - VALIDATE configuration parameters

Task 6:
CREATE src/platform.rs:
  - DETECT Jetson vs x86 platform
  - SET appropriate element properties
  - HANDLE platform-specific variations

Task 7:
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

- [ ] Project builds successfully with gstreamer dependencies
- [ ] DeepStream elements can be created through gstreamer-rs
- [ ] Configuration files can be parsed and applied
- [ ] Error handling framework in place
- [ ] Platform detection works correctly

## Dependencies

### Technical Dependencies
- NVIDIA DeepStream SDK 6.0+ (installed with GStreamer plugins)
- GStreamer 1.14+
- gstreamer-rs crate
- Rust 1.70+ with cargo

### Knowledge Dependencies
- GStreamer element creation and configuration
- DeepStream element properties
- Configuration file formats

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| DeepStream elements not available | Low | High | Check GST_PLUGIN_PATH, validate installation |
| Configuration parsing errors | Medium | Medium | Validate configs, provide clear error messages |
| Platform-specific property differences | Medium | Low | Abstract platform variations in dedicated module |

## Architecture Decisions

### Decision: DeepStream Integration Strategy
**Options Considered:**
1. Create FFI bindings for DeepStream SDK
2. Use DeepStream as GStreamer elements through gstreamer-rs
3. Hybrid approach with custom bindings for metadata

**Decision:** Option 2 - Use GStreamer element interface

**Rationale:** Simplest approach, leverages existing gstreamer-rs functionality

### Decision: Error Handling Approach
**Options Considered:**
1. Direct Result<T, E> propagation
2. Custom error types with context
3. anyhow/thiserror crates

**Decision:** Option 2 with thiserror for derive macros

**Rationale:** Provides rich error context while maintaining type safety

## Validation Strategy

- **Element Testing**: Verify DeepStream elements can be created
- **Configuration Testing**: Validate config file parsing
- **Integration Testing**: Create simple pipeline with DeepStream elements

## Future Considerations

- Creating helper crate for DeepStream element configuration
- Adding more element wrapper types
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
- **Confidence Level**: 9 - Simplified approach using standard GStreamer API, well-documented path
