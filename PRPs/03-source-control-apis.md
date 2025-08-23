# PRP: Runtime Source Addition/Deletion Control APIs

## Executive Summary

Implement the dynamic source management system that allows adding and removing video sources at runtime without stopping the pipeline. This PRP provides the control APIs and synchronization mechanisms for safe manipulation of sources during active video processing.

## Problem Statement

### Current State
- Pipeline infrastructure established (PRP-02)
- No dynamic source management capabilities
- C implementation uses timers and manual state management for source control

### Desired State
- Thread-safe APIs for adding/removing sources at runtime
- Automatic pad management and element lifecycle control
- Robust synchronization preventing data loss or corruption
- Event-driven architecture for source state changes

### Business Value
Enables real-time adaptation to changing video source requirements, critical for surveillance systems, live streaming platforms, and dynamic monitoring applications.

## Requirements

### Functional Requirements

1. **Source Addition**: Add new video sources to running pipeline
2. **Source Removal**: Safely remove sources without data loss
3. **Source Tracking**: Maintain source registry with unique IDs
4. **State Synchronization**: Coordinate source states with pipeline state
5. **Pad Management**: Handle dynamic pad creation/deletion
6. **EOS Handling**: Process per-source end-of-stream events
7. **Resource Cleanup**: Proper cleanup of removed sources

### Non-Functional Requirements

1. **Thread Safety**: Concurrent source operations without race conditions
2. **Zero Downtime**: Add/remove sources without pipeline interruption
3. **Memory Safety**: No leaks during repeated add/remove cycles
4. **Scalability**: Support up to MAX_NUM_SOURCES (configurable)

### Context and Research
The C implementation uses uridecodebin elements with pad-added callbacks, manages source state arrays, and implements timer-based source manipulation. Sources are added/removed randomly to demonstrate dynamic capabilities.

### Documentation & References
```yaml
- url: https://gstreamer.freedesktop.org/documentation/tutorials/basic/dynamic-pipelines.html
  why: Dynamic pad handling and runtime element manipulation

- file: vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete\deepstream_test_rt_src_add_del.c
  why: Reference implementation of add_sources (lines 263-321) and delete_sources (lines 228-261)

- url: https://stackoverflow.com/questions/75036823/dynamically-add-rtmp-source-to-running-pipeline-using-compositor
  why: Real-world example of dynamic source addition in Rust

- url: https://docs.nvidia.com/metropolis/deepstream/dev-guide/text/DS_plugin_gst-nvstreammux.html
  why: nvstreammux sink pad request and release procedures

- url: https://gstreamer.freedesktop.org/documentation/application-development/advanced/pipeline-manipulation.html
  why: Proper element state management and synchronization
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
CREATE src/source/mod.rs:
  - DEFINE SourceManager struct
  - IMPLEMENT source registry with HashMap
  - ADD mutex for thread-safe access
  - TRACK source states and metadata

Task 2:
CREATE src/source/video_source.rs:
  - DEFINE VideoSource struct
  - WRAP uridecodebin element
  - IMPLEMENT pad-added signal handler
  - MANAGE source lifecycle states

Task 3:
CREATE src/source/manager.rs:
  - IMPLEMENT add_source method
  - VALIDATE URI and source parameters
  - CREATE uridecodebin dynamically
  - CONNECT to nvstreammux request pad
  - SYNC element state with pipeline

Task 4:
CREATE src/source/removal.rs:
  - IMPLEMENT remove_source method
  - SEND EOS event to source
  - WAIT for EOS propagation
  - RELEASE nvstreammux pad
  - CLEANUP element from pipeline

Task 5:
CREATE src/source/events.rs:
  - DEFINE source event types
  - IMPLEMENT event channel system
  - HANDLE pad-added events
  - PROCESS per-source EOS events
  - EMIT source state changes

Task 6:
CREATE src/source/synchronization.rs:
  - IMPLEMENT state synchronization
  - ADD probe mechanisms for safe removal
  - HANDLE IDLE/BLOCK probes
  - ENSURE data flushing before removal

Task 7:
CREATE src/source/controller.rs:
  - IMPLEMENT high-level control API
  - ADD source by URI or config
  - REMOVE source by ID
  - QUERY source status
  - LIST active sources

Task 8:
CREATE tests/source_management.rs:
  - TEST concurrent add/remove operations
  - VERIFY no memory leaks
  - VALIDATE state consistency
  - CHECK EOS handling
```

### Out of Scope
- Source content analysis or filtering
- Advanced scheduling algorithms
- Source quality adaptation
- Network source reconnection logic

## Success Criteria

- [ ] Sources can be added to running pipeline
- [ ] Sources can be removed without stopping pipeline
- [ ] No data corruption during source manipulation
- [ ] Thread-safe concurrent operations
- [ ] Proper resource cleanup verified
- [ ] EOS events handled per-source

## Dependencies

### Technical Dependencies
- Completed PRP-01 and PRP-02
- GStreamer dynamic pipeline capabilities
- nvstreammux request pad support

### Knowledge Dependencies
- GStreamer pad probing mechanisms
- Thread synchronization patterns
- State machine design

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Race conditions during concurrent ops | Medium | High | Implement fine-grained locking |
| Memory leaks from improper cleanup | Medium | High | Use RAII patterns and testing |
| Deadlocks during state changes | Low | High | Add timeouts and deadlock detection |
| Data loss during removal | Medium | Medium | Implement proper EOS flushing |

## Architecture Decisions

### Decision: Synchronization Strategy
**Options Considered:**
1. Global lock for all operations
2. Per-source locks with careful ordering
3. Lock-free data structures

**Decision:** Option 2 - Per-source locks with ordering protocol

**Rationale:** Balances performance with safety, avoiding global bottlenecks

### Decision: Event Communication
**Options Considered:**
1. Callbacks/closures
2. Channel-based events
3. Direct polling

**Decision:** Option 2 - Channel-based event system

**Rationale:** Provides decoupling and natural async handling in Rust

### Decision: Source Identification
**Options Considered:**
1. Sequential integer IDs
2. UUIDs
3. User-provided strings

**Decision:** Option 1 with internal mapping to user labels

**Rationale:** Maintains compatibility with C implementation while allowing flexibility

## Validation Strategy

- **Stress Testing**: Rapid add/remove cycles
- **Concurrency Testing**: Parallel source operations
- **Memory Testing**: Valgrind/sanitizers for leak detection
- **State Testing**: Verify state machine transitions
- **Integration Testing**: Full pipeline with dynamic sources

## Future Considerations

- Source priority and scheduling
- Automatic source recovery/reconnection
- Source health monitoring
- Dynamic source transformation pipelines
- Source grouping and synchronization

## References

- GStreamer Dynamic Pipelines Documentation
- NVIDIA DeepStream User Guide
- Rust Concurrency Patterns

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-22
- **Last Modified**: 2025-08-22
- **Status**: Draft
- **Confidence Level**: 7 - Complex synchronization but well-documented patterns
