# PRP: Dynamic Video Sources Test Infrastructure

## Executive Summary

Create a robust test infrastructure crate (`source-videos`) that provides dynamic video sources for testing the ds-rs DeepStream runtime. This crate will generate multiple types of test video streams (file-based, RTSP, raw pipelines) that can be dynamically created and destroyed, enabling comprehensive testing of the source management capabilities without requiring actual video files or camera feeds.

## Problem Statement

### Current State
- The ds-rs application requires video sources (URIs) to demonstrate runtime source management
- Tests currently rely on hardcoded file paths that may not exist
- No easy way to generate multiple test video streams for stress testing
- RTSP testing requires external RTSP servers or cameras
- No infrastructure for generating controlled test patterns with specific characteristics

### Desired State
- Self-contained test video generation infrastructure
- Multiple video source types (files, RTSP streams, test patterns)
- Dynamic creation and destruction of sources on demand
- Configurable video properties (resolution, framerate, pattern, duration)
- RTSP server that can serve multiple test streams simultaneously
- Integration with ds-rs for automated testing

### Business Value
Enables reliable, repeatable testing of dynamic source management without external dependencies, accelerating development and ensuring robust validation of the runtime source control features.

## Requirements

### Functional Requirements

1. **Test Video Generation**: Create various test video patterns using GStreamer's videotestsrc
2. **RTSP Server**: Embedded RTSP server serving multiple test streams
3. **File Generation**: Generate test video files in common formats (MP4, MKV)
4. **Pipeline Factory**: Build GStreamer pipelines dynamically based on configuration
5. **Source Lifecycle**: Start, stop, and restart video sources on demand
6. **Status Monitoring**: Track active sources and their states
7. **Integration API**: Simple API for ds-rs tests to request/release sources

### Non-Functional Requirements

1. **Performance**: Support at least 10 concurrent video streams
2. **Memory Efficiency**: Clean resource management, no leaks
3. **Configurability**: TOML/JSON configuration for source properties
4. **Portability**: Work on Windows, Linux, and macOS
5. **Zero Dependencies**: Beyond GStreamer and standard Rust crates

### Context and Research

The crate will leverage GStreamer's test source capabilities (videotestsrc) and RTSP server bindings. Based on the existing ds-rs patterns, it should follow similar architectural approaches for pipeline management and backend abstraction.

### Documentation & References
```yaml
- url: https://gstreamer.freedesktop.org/documentation/videotestsrc/
  why: Complete videotestsrc element documentation with all patterns and properties

- url: https://gstreamer.freedesktop.org/documentation/tutorials/basic/dynamic-pipelines.html
  why: Dynamic pipeline creation patterns essential for runtime source management

- url: https://docs.rs/gstreamer-rtsp-server/latest/gstreamer_rtsp_server/
  why: Rust bindings for GStreamer RTSP server implementation

- url: https://github.com/GStreamer/gst-rtsp-server/blob/master/examples/test-appsrc.c
  why: Reference implementation for RTSP server with appsrc

- url: https://github.com/GStreamer/gst-rtsp-server/blob/master/examples/test-launch.c
  why: Simple RTSP server example with launch syntax

- file: crates/ds-rs/src/pipeline/builder.rs
  why: Follow existing pipeline builder patterns from ds-rs

- file: crates/ds-rs/src/source/video_source.rs
  why: Understand how ds-rs consumes video sources

- file: crates/ds-rs/tests/pipeline_tests.rs
  why: See existing test patterns using videotestsrc
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
UPDATE crates/source-videos/Cargo.toml:
  - ADD gstreamer = "0.24.1" dependency
  - ADD gstreamer-app = "0.24.0" dependency
  - ADD gstreamer-rtsp = "0.24.0" dependency
  - ADD gstreamer-rtsp-server = "0.24.0" dependency
  - ADD tokio with full features
  - ADD serde with derive feature
  - ADD toml for configuration
  - ADD thiserror for error handling
  - ADD clap with derive for CLI
  - ADD log and env_logger for logging

Task 2:
CREATE src/lib.rs:
  - DEFINE public API module structure
  - EXPORT VideoSourceManager as main interface
  - EXPORT VideoSourceConfig for configuration
  - EXPORT VideoSourceType enum (TestPattern, File, Rtsp)
  - EXPORT Error types

Task 3:
CREATE src/error.rs:
  - DEFINE custom error types using thiserror
  - ADD GStreamerError wrapper
  - ADD ConfigurationError for invalid configs
  - ADD ServerError for RTSP server issues
  - ADD ResourceError for resource exhaustion

Task 4:
CREATE src/config.rs:
  - DEFINE VideoSourceConfig struct with serde
  - ADD resolution settings (width, height)
  - ADD framerate configuration
  - ADD video format specification
  - ADD test pattern selection
  - ADD duration/num-buffers options
  - IMPLEMENT config file loading from TOML

Task 5:
CREATE src/pipeline/mod.rs:
  - DEFINE PipelineFactory trait
  - IMPLEMENT TestPatternPipeline
  - IMPLEMENT FileSinkPipeline
  - IMPLEMENT RtspSourcePipeline
  - ADD pipeline state management
  - ADD dynamic property configuration

Task 6:
CREATE src/pipeline/builder.rs:
  - CREATE fluent pipeline builder API
  - ADD element creation helpers
  - ADD linking utilities
  - ADD caps filter support
  - ADD property setting methods
  - FOLLOW ds-rs PipelineBuilder patterns

Task 7:
CREATE src/patterns.rs:
  - DEFINE TestPattern enum matching videotestsrc patterns
  - ADD pattern descriptions and use cases
  - CREATE pattern rotation utilities
  - ADD custom pattern generation via appsrc
  - IMPLEMENT pattern-specific configurations

Task 8:
CREATE src/rtsp/mod.rs:
  - DEFINE RtspServer wrapper
  - IMPLEMENT server lifecycle management
  - ADD mount point management
  - CREATE media factory for test sources
  - ADD authentication support (optional)
  - IMPLEMENT port configuration

Task 9:
CREATE src/rtsp/factory.rs:
  - IMPLEMENT RTSPMediaFactory wrapper
  - CREATE launch string generation
  - ADD dynamic source configuration
  - IMPLEMENT client connection handling
  - ADD stream-specific settings

Task 10:
CREATE src/manager.rs:
  - IMPLEMENT VideoSourceManager struct
  - ADD source registry with unique IDs
  - CREATE add_source method
  - CREATE remove_source method
  - ADD list_sources method
  - IMPLEMENT source state tracking
  - ADD concurrent access protection

Task 11:
CREATE src/source.rs:
  - DEFINE VideoSource trait
  - IMPLEMENT TestPatternSource
  - IMPLEMENT FileSource
  - IMPLEMENT RtspSource
  - ADD source lifecycle methods
  - ADD URI generation methods

Task 12:
CREATE src/file.rs:
  - IMPLEMENT file generation pipelines
  - ADD MP4 muxing support
  - ADD MKV muxing support
  - CREATE temporary file management
  - ADD configurable encoders
  - IMPLEMENT duration control

Task 13:
UPDATE src/main.rs:
  - CREATE CLI application using clap
  - ADD subcommands (serve, generate, list)
  - IMPLEMENT RTSP server mode
  - ADD file generation mode
  - CREATE interactive mode
  - ADD status monitoring

Task 14:
CREATE tests/integration_test.rs:
  - TEST concurrent source creation
  - TEST RTSP server with multiple streams
  - TEST file generation
  - TEST resource cleanup
  - TEST configuration loading
  - VERIFY no memory leaks

Task 15:
CREATE examples/basic_rtsp_server.rs:
  - DEMONSTRATE RTSP server setup
  - SHOW multiple stream mounting
  - INCLUDE client connection example
  - ADD graceful shutdown

Task 16:
CREATE examples/generate_test_files.rs:
  - SHOW file generation API
  - DEMONSTRATE different patterns
  - INCLUDE batch generation
  - ADD progress reporting

Task 17:
CREATE examples/ds_rs_integration.rs:
  - DEMONSTRATE integration with ds-rs
  - SHOW dynamic source provisioning
  - INCLUDE cleanup patterns
  - ADD error handling

Task 18:
CREATE configuration/default.toml:
  - PROVIDE default configurations
  - INCLUDE pattern presets
  - ADD RTSP server defaults
  - SPECIFY file generation settings

Task 19:
CREATE README.md:
  - DOCUMENT installation steps
  - EXPLAIN architecture
  - PROVIDE usage examples
  - ADD troubleshooting guide
  - INCLUDE API reference
```

### Out of Scope
- Actual camera device integration
- Network camera discovery (ONVIF)
- Video transcoding between formats
- Advanced video effects or filters
- Production RTSP server features (authentication, RTCP, etc.)

## Success Criteria

- [ ] Can generate at least 10 concurrent test video sources
- [ ] RTSP server serves multiple streams accessible via standard players
- [ ] Generated files playable in common video players
- [ ] Clean shutdown with no resource leaks
- [ ] Integration tests pass with ds-rs
- [ ] Memory usage stays constant during extended runs

## Dependencies

### Technical Dependencies
- GStreamer 1.14+ with base, good, and bad plugins
- gstreamer-rtsp-server for RTSP functionality
- Existing ds-rs patterns for consistency

### Knowledge Dependencies
- GStreamer pipeline construction
- RTSP protocol basics
- Video encoding fundamentals
- Rust async patterns with tokio

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| GStreamer plugin availability varies by platform | High | Medium | Detect available plugins at runtime, provide fallbacks |
| RTSP server port conflicts | Medium | Low | Make ports configurable, auto-detect free ports |
| Memory leaks in long-running streams | Medium | High | Implement proper cleanup, add leak detection tests |
| Platform-specific pipeline differences | Medium | Medium | Abstract pipeline creation, test on all platforms |

## Architecture Decisions

### Decision: Pipeline Management Strategy
**Options Considered:**
1. Static pipeline templates
2. Dynamic pipeline construction
3. Hybrid with presets and customization

**Decision:** Option 3 - Hybrid approach with presets and customization

**Rationale:** Provides ease of use for common cases while allowing flexibility

### Decision: RTSP Server Architecture
**Options Considered:**
1. Single server, multiple mount points
2. Multiple servers on different ports
3. Server pool with load balancing

**Decision:** Option 1 - Single server with multiple mount points

**Rationale:** Simplest to manage, sufficient for testing needs

### Decision: Source Identification
**Options Considered:**
1. Sequential integer IDs
2. UUIDs
3. User-provided names with fallback to IDs

**Decision:** Option 3 - Names with ID fallback

**Rationale:** User-friendly while maintaining uniqueness

## Validation Strategy

### Validation Commands
```bash
# Build and check
cargo build --release
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features -- --nocapture

# Test RTSP server
cargo run -- serve --port 8554 &
ffplay rtsp://localhost:8554/test-1
ffplay rtsp://localhost:8554/test-2

# Test file generation
cargo run -- generate --pattern smpte --duration 10 --output test.mp4
ffplay test.mp4

# Integration test with ds-rs
cd ../ds-rs
cargo run --bin ds-app -- rtsp://localhost:8554/test-1

# Memory leak check (Linux)
valgrind --leak-check=full cargo run -- serve --duration 60

# Load test
for i in {1..10}; do
  ffplay rtsp://localhost:8554/test-$i &
done
```

## Future Considerations

- WebRTC source support for browser-based testing
- HLS/DASH streaming support
- Synthetic motion patterns for tracking tests
- Audio test signal generation
- Network simulation (latency, packet loss)
- Docker container with pre-configured sources

## References

- GStreamer Documentation: https://gstreamer.freedesktop.org/
- RTSP RFC: https://tools.ietf.org/html/rfc7826
- GStreamer Rust Bindings: https://gitlab.freedesktop.org/gstreamer/gstreamer-rs

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Ready
- **Confidence Level**: 8 - Well-defined scope with clear patterns to follow from ds-rs