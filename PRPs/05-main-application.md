# PRP: Main Application and Runtime Demonstration

## Executive Summary

Implement the main application that demonstrates the complete runtime source addition/deletion functionality, providing a CLI interface and orchestrating all components developed in previous PRPs. This final integration showcases the full DeepStream capabilities in Rust.

## Problem Statement

### Current State
- Individual components implemented (PRPs 01-04)
- No unified application entry point
- Missing demonstration of runtime capabilities
- No CLI interface for testing

### Desired State
- Complete application mirroring C implementation functionality
- CLI with URI input support
- Automated source addition/deletion demonstration
- Comprehensive logging and monitoring

### Business Value
Provides a production-ready demonstration of dynamic video analytics that can serve as a foundation for real-world applications in surveillance, streaming, and monitoring systems.

## Requirements

### Functional Requirements

1. **CLI Interface**: Accept video URIs as command-line arguments
2. **Pipeline Orchestration**: Initialize and manage complete DeepStream pipeline
3. **Automated Demo**: Periodically add sources up to MAX_NUM_SOURCES, then remove
4. **Event Loop**: Manage GStreamer main loop and event processing
5. **Graceful Shutdown**: Clean resource cleanup on exit
6. **Error Recovery**: Handle failures without crashing
7. **Status Reporting**: Display pipeline and source status

### Non-Functional Requirements

1. **Usability**: Simple command-line interface
2. **Robustness**: Handle various input formats (file, RTSP, HTTP)
3. **Observability**: Comprehensive logging
4. **Performance**: Match or exceed C implementation

### Context and Research
The C implementation creates a pipeline with one initial source, uses timers to add sources every 10 seconds up to MAX_NUM_SOURCES, then removes sources periodically until one remains or EOS is reached.

### Documentation & References
```yaml
- file: vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete\deepstream_test_rt_src_add_del.c
  why: Main function implementation (lines 465-668)

- file: vendor\NVIDIA-AI-IOT--deepstream_reference_apps\runtime_source_add_delete\README.md
  why: Usage instructions and expected behavior

- url: https://docs.rs/clap/latest/clap/
  why: CLI argument parsing in Rust

- url: https://docs.rs/tokio/latest/tokio/
  why: Async runtime for event handling

- url: https://docs.rs/tracing/latest/tracing/
  why: Structured logging framework
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
CREATE src/main.rs:
  - SETUP CLI argument parser
  - INITIALIZE GStreamer
  - CREATE application context
  - RUN main event loop
  - HANDLE shutdown signals

Task 2:
CREATE src/app/mod.rs:
  - DEFINE Application struct
  - MANAGE pipeline lifecycle
  - COORDINATE source manager
  - HANDLE global state

Task 3:
CREATE src/app/config.rs:
  - DEFINE configuration constants
  - LOAD environment variables
  - VALIDATE settings
  - PROVIDE defaults

Task 4:
CREATE src/app/runner.rs:
  - IMPLEMENT main run loop
  - SETUP initial pipeline
  - START source addition timer
  - HANDLE state transitions
  - MANAGE source deletion

Task 5:
CREATE src/app/timers.rs:
  - IMPLEMENT add_sources timer
  - IMPLEMENT delete_sources timer
  - HANDLE timer cancellation
  - COORDINATE with source manager

Task 6:
CREATE src/app/monitor.rs:
  - TRACK pipeline statistics
  - MONITOR source states
  - LOG performance metrics
  - DETECT anomalies

Task 7:
CREATE src/bin/ds-runtime-demo.rs:
  - PARSE command line arguments
  - VALIDATE input URIs
  - INITIALIZE application
  - RUN demonstration
  - REPORT results

Task 8:
CREATE examples/multi_source.rs:
  - DEMONSTRATE multiple sources
  - SHOW different URI types
  - DISPLAY detection results
  - HANDLE various scenarios

Task 9:
MODIFY Cargo.toml:
  - ADD binary target
  - INCLUDE CLI dependencies
  - SETUP example compilation
  - ADD development dependencies

Task 10:
CREATE tests/integration.rs:
  - TEST full pipeline flow
  - VERIFY source addition/deletion
  - CHECK memory usage
  - VALIDATE output
```

### Out of Scope
- GUI interface
- Configuration file support (beyond inference configs)
- Network streaming output
- Recording capabilities

## Success Criteria

- [ ] Application accepts URI and runs pipeline
- [ ] Sources automatically added every 10 seconds
- [ ] Sources removed after reaching maximum
- [ ] Clean shutdown on EOS or interrupt
- [ ] No memory leaks during extended runs
- [ ] Matches C implementation behavior

## Dependencies

### Technical Dependencies
- All previous PRPs (01-04) completed
- clap for CLI parsing
- tokio for async runtime
- tracing for logging

### Knowledge Dependencies
- Application lifecycle management
- Signal handling in Rust
- Async programming patterns

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Integration issues between components | Medium | High | Comprehensive integration testing |
| Timer synchronization problems | Low | Medium | Use tokio's timer primitives |
| Resource cleanup failures | Medium | High | Implement Drop traits properly |
| Platform-specific behaviors | Medium | Medium | Abstract platform differences |

## Architecture Decisions

### Decision: Async Runtime
**Options Considered:**
1. Synchronous with threads
2. Tokio async runtime
3. async-std

**Decision:** Option 2 - Tokio for consistency with ecosystem

**Rationale:** Best ecosystem support and GStreamer integration

### Decision: Logging Framework
**Options Considered:**
1. println debugging
2. log crate
3. tracing with structured logging

**Decision:** Option 3 - Tracing for production readiness

**Rationale:** Provides structured logging and async support

## Validation Strategy

- **Functional Testing**: Verify all features work as specified
- **Performance Testing**: Compare with C implementation
- **Stress Testing**: Extended runs with multiple sources
- **Memory Testing**: Check for leaks with valgrind
- **Platform Testing**: Verify on both x86 and Jetson

## Future Considerations

- Web API for remote control
- Prometheus metrics export
- Configuration file support
- Docker containerization
- Kubernetes deployment manifests

## References

- Original C implementation
- GStreamer Application Development Manual
- Rust CLI Book
- Tokio Tutorial

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-22
- **Last Modified**: 2025-08-22
- **Status**: Draft
- **Confidence Level**: 9 - Clear requirements with straightforward integration
