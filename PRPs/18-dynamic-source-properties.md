# PRP: Dynamic Source Properties & Multi-Resolution Support

**Status**: NOT STARTED (as of 2025-08-27) - No multi-resolution or dynamic property support

## Executive Summary

Enhance source-videos to support dynamic property changes and multiple simultaneous resolutions per source. This enables adaptive streaming scenarios, multi-client support with different quality requirements, and runtime optimization without service disruption.

## Problem Statement

### Current State
- Single fixed resolution per source
- Properties locked at source creation
- No adaptive quality support
- Cannot serve multiple resolutions simultaneously
- Manual source recreation for property changes
- No dynamic bitrate adjustment

### Desired State
- Multiple resolution outputs per source
- Runtime property modifications
- Adaptive streaming support
- Automatic transcoding pipelines
- Dynamic quality adjustment
- Client-specific stream selection

### Business Value
Supports diverse client requirements, enables adaptive streaming for varying network conditions, reduces resource usage through shared pipelines, and provides production-ready multi-resolution streaming.

## Requirements

### Functional Requirements

1. **Multi-Resolution Pipeline**: Single source, multiple outputs
2. **Dynamic Scaling**: Add/remove resolutions at runtime
3. **Property Updates**: Change framerate, bitrate, codec settings
4. **Quality Profiles**: Predefined resolution/quality combinations
5. **Automatic Transcoding**: On-demand format conversion
6. **Client Negotiation**: Serve appropriate resolution per client
7. **Resource Optimization**: Share decode, process once

### Non-Functional Requirements

1. **Performance**: Minimal overhead for multi-resolution
2. **Efficiency**: Shared pipeline components
3. **Scalability**: Support 10+ resolutions per source
4. **Compatibility**: Work with existing RTSP/file outputs
5. **Quality**: Maintain video quality during transitions

### Context and Research

GStreamer's tee element enables branching pipelines for multiple outputs. The videoscale and videorate elements handle resolution and framerate conversion. Queue elements provide buffering and thread decoupling. The compositor and videomixer elements can combine multiple streams.

Modern streaming protocols like HLS and DASH expect multiple quality levels. The current pipeline architecture in source-videos can be extended with tee branches. GStreamer properties can be modified at runtime using set_property on running pipelines.

### Documentation & References

```yaml
- file: crates/source-videos/src/pipeline/builder.rs
  why: Current pipeline construction to extend

- file: crates/source-videos/src/source.rs
  why: VideoSource trait to enhance

- url: https://gstreamer.freedesktop.org/documentation/coreelements/tee.html
  why: Tee element for pipeline branching

- url: https://gstreamer.freedesktop.org/documentation/videoscale/
  why: Video scaling documentation

- url: https://gstreamer.freedesktop.org/documentation/tutorials/basic/multithreading-and-pad-availability.html
  why: Dynamic pipeline modification patterns

- file: ../gstreamer-rs/gstreamer/src/element.rs
  why: Runtime property modification APIs

- url: https://gstreamer.freedesktop.org/documentation/additional/design/probes.html
  why: Pad probes for format negotiation
```

### List of tasks to be completed

```yaml
Task 1:
CREATE crates/source-videos/src/pipeline/multi_resolution.rs:
  - DESIGN MultiResolutionPipeline struct
  - IMPLEMENT tee-based branching
  - ADD queue buffering per branch
  - CREATE resolution scaling chains
  - HANDLE pad request/release
  - MANAGE branch lifecycle

Task 2:
CREATE crates/source-videos/src/resolution/mod.rs:
  - DEFINE ResolutionProfile struct
  - IMPLEMENT standard profiles (1080p, 720p, 480p, etc.)
  - ADD aspect ratio handling
  - CREATE quality presets
  - SUPPORT custom resolutions
  - HANDLE format conversions

Task 3:
CREATE crates/source-videos/src/resolution/manager.rs:
  - IMPLEMENT ResolutionManager
  - TRACK active resolutions per source
  - HANDLE resolution addition/removal
  - MANAGE pipeline modifications
  - COORDINATE format negotiation
  - OPTIMIZE shared components

Task 4:
UPDATE crates/source-videos/src/source.rs:
  - EXTEND VideoSource trait for multi-resolution
  - ADD add_resolution() method
  - ADD remove_resolution() method
  - ADD update_resolution() method
  - IMPLEMENT get_available_resolutions()
  - SUPPORT resolution enumeration

Task 5:
CREATE crates/source-videos/src/pipeline/branch.rs:
  - IMPLEMENT PipelineBranch struct
  - HANDLE videoscale element
  - MANAGE videorate element
  - ADD capsfilter for format
  - IMPLEMENT queue management
  - SUPPORT branch isolation

Task 6:
CREATE crates/source-videos/src/pipeline/dynamic.rs:
  - IMPLEMENT DynamicPipelineModifier
  - HANDLE pipeline state during modifications
  - USE pad probes for safe changes
  - IMPLEMENT atomic modifications
  - ADD rollback support
  - ENSURE data flow continuity

Task 7:
UPDATE crates/source-videos/src/config.rs:
  - ADD ResolutionConfig type
  - EXTEND VideoSourceConfig for multiple resolutions
  - ADD quality profiles configuration
  - SUPPORT resolution priorities
  - IMPLEMENT validation rules
  - ADD bandwidth targets

Task 8:
CREATE crates/source-videos/src/transcoding/mod.rs:
  - IMPLEMENT TranscodingPipeline
  - SUPPORT format conversion
  - ADD codec selection logic
  - HANDLE colorspace conversion
  - IMPLEMENT bitrate control
  - ADD quality settings

Task 9:
CREATE crates/source-videos/src/negotiation/mod.rs:
  - IMPLEMENT ClientNegotiator
  - PARSE client capabilities
  - SELECT optimal resolution
  - HANDLE RTSP DESCRIBE
  - SUPPORT SDP generation
  - ADD bandwidth estimation

Task 10:
UPDATE crates/source-videos/src/rtsp/factory.rs:
  - MODIFY for multi-resolution support
  - IMPLEMENT resolution selection
  - ADD SDP variants
  - HANDLE client negotiation
  - SUPPORT quality parameters
  - ADD stream switching

Task 11:
CREATE crates/source-videos/src/optimization/mod.rs:
  - IMPLEMENT PipelineOptimizer
  - DETECT shared components
  - MERGE common processing
  - REDUCE memory copies
  - OPTIMIZE thread usage
  - MONITOR resource usage

Task 12:
ADD property update system:
  - IMPLEMENT property change queue
  - BATCH property updates
  - HANDLE timing constraints
  - VALIDATE property ranges
  - SUPPORT incremental changes
  - ADD property animations

Task 13:
CREATE quality adaptation:
  - MONITOR client bandwidth
  - IMPLEMENT quality switching
  - ADD ABR algorithms
  - HANDLE network congestion
  - SUPPORT manual override
  - LOG quality changes

Task 14:
CREATE integration tests:
  - TEST multi-resolution pipelines
  - VERIFY dynamic modifications
  - TEST quality switching
  - BENCHMARK performance
  - VALIDATE resource usage

Task 15:
UPDATE documentation:
  - DOCUMENT multi-resolution setup
  - ADD configuration examples
  - DESCRIBE quality profiles
  - PROVIDE migration guide
  - ADD performance tuning
```

### Out of Scope
- Video encoding hardware acceleration
- ML-based quality optimization
- Perceptual quality metrics
- CDN integration
- DRM support

## Success Criteria

- [ ] Support 5+ simultaneous resolutions per source
- [ ] Resolution changes without stream interruption
- [ ] Property updates applied within 100ms
- [ ] Resource usage scales sub-linearly with resolutions
- [ ] Client-appropriate resolution selection working
- [ ] Quality switching smooth and fast

## Dependencies

### Technical Dependencies
- GStreamer 1.14+ for modern pipeline features
- gstreamer-rs for property manipulation
- Hardware resources for transcoding

### Knowledge Dependencies
- GStreamer pipeline design patterns
- Video transcoding principles
- Adaptive streaming protocols
- Resource optimization techniques

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| CPU overload from transcoding | High | High | Hardware acceleration, limits |
| Memory usage explosion | Medium | High | Careful buffer management |
| Pipeline deadlocks | Low | Critical | Thorough testing, timeouts |
| Quality degradation | Medium | Medium | Quality monitoring metrics |

## Architecture Decisions

### Decision: Pipeline Architecture
**Options Considered:**
1. Separate pipeline per resolution
2. Single pipeline with tee branching
3. Hybrid with shared decode

**Decision:** Option 3 - Hybrid with shared decode

**Rationale:** Best balance of efficiency and flexibility, reduces decode overhead

### Decision: Resolution Management
**Options Considered:**
1. Static configuration only
2. Fully dynamic
3. Profile-based with overrides

**Decision:** Option 3 - Profile-based with overrides

**Rationale:** Good defaults with flexibility, easier configuration management

## Validation Strategy

- **Unit Tests**: Test pipeline components
- **Integration Tests**: Multi-resolution scenarios
- **Performance Tests**: Resource usage monitoring
- **Quality Tests**: Video quality validation
- **Stress Tests**: Maximum resolution count

## Future Considerations

- Hardware acceleration integration
- AI-based quality optimization
- Distributed transcoding
- Edge caching integration
- VR/360 video support

## References

- GStreamer Pipeline Design
- Adaptive Streaming Specifications
- Video Transcoding Best Practices
- GStreamer Dynamic Pipelines Guide

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-27
- **Status**: Complete
- **Confidence Level**: 7 - Complex implementation but well-understood patterns
