# PRP: DeepStream Metadata Handling and Message Processing

## Executive Summary

Implement metadata extraction and processing for DeepStream elements, handling inference results and DeepStream-specific messages. This PRP focuses on the minimal FFI required for metadata structures while using DeepStream elements through the standard GStreamer API.

## Problem Statement

### Current State
- DeepStream elements accessible through gstreamer-rs (PRP-01 to PRP-03)
- No metadata extraction from GstBuffer
- Missing inference result handling
- No DeepStream-specific message processing

### Desired State
- Minimal FFI for NvDsBatchMeta structure access
- Inference result extraction from metadata
- Stream-specific EOS handling through gst-nvmessage
- Safe wrappers for metadata traversal

### Business Value
Enables AI-powered video analytics with object detection, tracking, and classification capabilities essential for intelligent video processing applications.

## Requirements

### Functional Requirements

1. **Metadata Access**: Read and write NvDsBatchMeta structures
2. **Object Detection**: Process inference results from nvinfer
3. **Object Tracking**: Handle tracker metadata and IDs
4. **Stream Messages**: Process DeepStream-specific GStreamer messages
5. **Configuration Loading**: Parse and apply DeepStream config files
6. **Display Metadata**: Manage OSD display information
7. **Surface Access**: Handle NvBufSurface for direct frame access

### Non-Functional Requirements

1. **Performance**: Minimal overhead for metadata processing
2. **Safety**: Safe wrappers for unsafe metadata operations
3. **Compatibility**: Support all DeepStream 6.0+ metadata formats
4. **Extensibility**: Allow custom metadata processing

### Context and Research
The C implementation uses gstnvdsmeta.h for metadata access, processes inference results for vehicle/person detection, and handles stream-specific EOS through custom GStreamer messages.

### Documentation & References
```yaml
- url: https://docs.nvidia.com/metropolis/deepstream/dev-guide/text/DS_plugin_metadata.html
  why: DeepStream metadata structures and access patterns

- url: https://github.com/rust-lang/rust-bindgen
  why: Generate minimal FFI bindings for nvdsmeta.h

- file: vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete\deepstream_test_rt_src_add_del.c
  why: Metadata handling example (lines 358-375)

- url: https://docs.nvidia.com/metropolis/deepstream/dev-guide/text/DS_plugin_gst-nvinfer.html
  why: Inference metadata output format

- url: https://docs.nvidia.com/metropolis/deepstream/dev-guide/text/DS_plugin_gst-nvtracker.html
  why: Tracker metadata and object IDs

- file: vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete\dstest_pgie_config.txt
  why: Inference configuration format
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
CREATE src/ffi/nvdsmeta.rs:
  - USE bindgen for minimal nvdsmeta.h bindings
  - FOCUS on NvDsBatchMeta, NvDsFrameMeta, NvDsObjectMeta
  - INCLUDE gst_buffer_get_nvds_batch_meta function
  - KEEP bindings minimal and focused

Task 2:
CREATE src/metadata/mod.rs:
  - DEFINE safe wrappers around FFI types
  - IMPLEMENT metadata extraction from GstBuffer
  - ADD iterator patterns for traversing metadata
  - ENSURE lifetime safety with PhantomData

Task 3:
CREATE src/metadata/object.rs:
  - WRAP NvDsObjectMeta structure
  - EXTRACT detection results (bbox, confidence)
  - ACCESS classifier metadata
  - HANDLE tracker IDs and states

Task 4:
CREATE src/metadata/frame.rs:
  - WRAP NvDsFrameMeta structure
  - ACCESS frame-level information
  - ITERATE object metadata
  - HANDLE user metadata

Task 5:
CREATE src/inference/mod.rs:
  - DEFINE inference result structures
  - PARSE nvinfer output metadata
  - MAP class IDs to labels
  - HANDLE multi-class detection

Task 6:
CREATE src/inference/config.rs:
  - PARSE inference config files
  - EXTRACT model parameters
  - VALIDATE configuration
  - APPLY to nvinfer elements

Task 7:
CREATE src/tracking/mod.rs:
  - PROCESS tracker metadata
  - MAINTAIN object ID mapping
  - HANDLE tracker state updates
  - IMPLEMENT trajectory tracking

Task 8:
CREATE src/messages/mod.rs:
  - HANDLE gst_nvmessage_is_stream_eos
  - PARSE stream-specific EOS
  - PROCESS custom DeepStream messages
  - EMIT high-level events

Task 9:
CREATE build.rs:
  - CONFIGURE bindgen for nvdsmeta.h
  - SET include paths for DeepStream headers
  - GENERATE only required structures
  - HANDLE platform-specific paths

Task 10:
CREATE examples/detection_app.rs:
  - DEMONSTRATE full pipeline
  - SHOW metadata processing
  - DISPLAY detection results
  - HANDLE dynamic sources
```

### Out of Scope
- Full DeepStream SDK FFI bindings (only metadata needed)
- Custom inference model training
- Advanced visualization beyond OSD
- CUDA kernel implementation
- NvBufSurface direct manipulation

## Success Criteria

- [ ] Metadata safely accessible from Rust
- [ ] Inference results correctly extracted
- [ ] Object tracking IDs maintained
- [ ] Stream-specific EOS handled
- [ ] Config files properly parsed
- [ ] Example application runs successfully

## Dependencies

### Technical Dependencies
- Completed PRP-01, PRP-02, PRP-03
- bindgen for minimal FFI generation
- DeepStream SDK headers (nvdsmeta.h)

### Knowledge Dependencies
- DeepStream metadata architecture
- FFI safety in Rust
- GStreamer buffer metadata

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Unsafe metadata access crashes | Medium | High | Extensive safety wrappers and validation |
| Bindgen output complexity | Low | Medium | Generate only required structures |
| Metadata format changes | Low | High | Version detection and compatibility layer |
| FFI overhead | Low | Low | Minimal bindings, direct access patterns |

## Architecture Decisions

### Decision: Metadata Safety Strategy
**Options Considered:**
1. Direct unsafe access
2. Safe wrappers with runtime checks
3. Compile-time safety with lifetimes

**Decision:** Option 3 - Lifetime-based safety where possible

**Rationale:** Maximizes safety without runtime overhead

### Decision: FFI Scope
**Options Considered:**
1. Full DeepStream SDK bindings
2. Minimal metadata-only bindings
3. No FFI - metadata via GObject properties

**Decision:** Option 2 - Minimal metadata-only bindings

**Rationale:** Metadata structures require direct access, everything else works through GStreamer

## Validation Strategy

- **Metadata Testing**: Verify correct metadata extraction
- **Inference Testing**: Validate detection results
- **Tracking Testing**: Ensure ID persistence
- **Message Testing**: Confirm stream EOS handling
- **Performance Testing**: Benchmark metadata processing

## Future Considerations

- Custom metadata types
- Advanced analytics plugins
- Multi-GPU support
- Cloud metadata streaming
- Analytics result storage

## References

- DeepStream SDK Programming Guide
- NvDsMetadata API Reference
- GStreamer Metadata Documentation

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-22
- **Last Modified**: 2025-08-22
- **Status**: Draft
- **Confidence Level**: 8 - Reduced complexity with minimal FFI scope, clear documentation available
