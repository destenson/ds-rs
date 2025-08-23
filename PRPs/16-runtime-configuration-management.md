# PRP: Runtime Configuration Management & File Monitoring

## Executive Summary

Transform the source-videos application from a static, launch-time configured system to a dynamic, runtime-configurable service that monitors configuration files for changes and applies updates without restart. This foundational enhancement enables live configuration updates, graceful source management, and sets the stage for external control interfaces.

## Problem Statement

### Current State
- Configuration loaded once at startup from TOML file
- No runtime modification of source properties
- Manual restart required for any configuration changes
- No automatic configuration file monitoring
- Sources can be added/removed but not modified
- Resolution, framerate, and format fixed at creation time

### Desired State
- Configuration file monitored for changes with automatic reload
- Runtime modification of source properties without restart
- Atomic configuration updates with validation
- Graceful handling of invalid configuration changes
- Event notifications for configuration changes
- Support for partial configuration updates

### Business Value
Enables zero-downtime configuration updates, reduces operational complexity, improves system flexibility for testing and production environments, and provides foundation for advanced control interfaces.

## Requirements

### Functional Requirements

1. **File Monitoring**: Watch configuration file for changes with debouncing
2. **Configuration Validation**: Pre-validate changes before applying
3. **Atomic Updates**: Apply configuration changes atomically or rollback
4. **Source Lifecycle Management**: Handle source additions, removals, and modifications
5. **Property Updates**: Support runtime changes to resolution, framerate, format, etc.
6. **Event System**: Notify components of configuration changes
7. **Diff Detection**: Identify what changed between configurations

### Non-Functional Requirements

1. **Performance**: Configuration reload < 100ms
2. **Reliability**: Never crash on invalid configuration
3. **Atomicity**: All-or-nothing configuration updates
4. **Observability**: Log all configuration changes
5. **Compatibility**: Maintain backward compatibility

### Context and Research

The notify crate provides cross-platform filesystem notifications and is used by rust-analyzer, cargo-watch, and other production tools. For async integration, tokio::sync::watch provides perfect broadcast semantics for configuration updates. The pattern of SIGHUP for configuration reload is well-established in Unix daemons.

Current implementation in crates/source-videos already has basic add/remove capabilities but lacks modification support. The VideoSourceManager maintains sources in Arc<RwLock<HashMap>> which supports concurrent access patterns needed for runtime updates.

### Documentation & References

```yaml
- url: https://github.com/notify-rs/notify
  why: Cross-platform filesystem notification library

- url: https://docs.rs/tokio/latest/tokio/sync/struct.watch.html
  why: Broadcast channel perfect for configuration updates

- url: https://vorner.github.io/2019/08/11/runtime-configuration-reloading.html
  why: Best practices for runtime configuration reloading

- file: crates/source-videos/src/manager.rs
  why: Current VideoSourceManager implementation to extend

- file: crates/source-videos/src/config.rs
  why: Configuration structures to enhance

- file: crates/source-videos/configuration/default.toml
  why: Default configuration format to maintain compatibility

- url: https://github.com/justinrubek/async-watcher
  why: Debounced file watching for Tokio
```

### List of tasks to be completed

```yaml
Task 1:
CREATE crates/source-videos/src/config/watcher.rs:
  - IMPLEMENT ConfigWatcher struct
  - USE notify crate for file system events
  - ADD debouncing with configurable delay (default 500ms)
  - CONVERT notify events to tokio channel
  - HANDLE file creation, modification, deletion
  - SUPPORT both polling and native OS watchers

Task 2:
CREATE crates/source-videos/src/config/loader.rs:
  - IMPLEMENT ConfigLoader trait
  - PARSE TOML configuration files
  - VALIDATE configuration against schema
  - DETECT configuration differences
  - PROVIDE atomic load operations
  - HANDLE partial configuration files

Task 3:
CREATE crates/source-videos/src/config/validator.rs:
  - IMPLEMENT ConfigValidator struct
  - CHECK resolution constraints (min/max)
  - VERIFY framerate validity
  - VALIDATE format compatibility
  - ENSURE source name uniqueness
  - TEST mount point conflicts for RTSP

Task 4:
CREATE crates/source-videos/src/runtime/mod.rs:
  - IMPLEMENT RuntimeManager struct
  - COORDINATE configuration updates
  - MANAGE source lifecycle transitions
  - HANDLE rollback on failure
  - EMIT configuration change events
  - MAINTAIN configuration history

Task 5:
UPDATE crates/source-videos/src/manager.rs:
  - ADD update_source() method
  - IMPLEMENT modify_source_config()
  - SUPPORT atomic batch updates
  - ADD configuration versioning
  - ENHANCE error recovery
  - PROVIDE source state snapshots

Task 6:
CREATE crates/source-videos/src/runtime/events.rs:
  - DEFINE ConfigurationEvent enum
  - IMPLEMENT event bus using tokio::sync::broadcast
  - ADD event filtering and routing
  - PROVIDE event persistence option
  - SUPPORT event replay for debugging

Task 7:
CREATE crates/source-videos/src/runtime/differ.rs:
  - IMPLEMENT ConfigDiffer struct
  - DETECT added sources
  - IDENTIFY removed sources
  - FIND modified properties
  - GENERATE change plan
  - OPTIMIZE minimal update path

Task 8:
UPDATE crates/source-videos/src/source.rs:
  - ADD update_config() method to VideoSource trait
  - IMPLEMENT property setters for runtime changes
  - HANDLE pipeline state during updates
  - SUPPORT graceful format transitions
  - MAINTAIN backward compatibility

Task 9:
CREATE crates/source-videos/src/runtime/applicator.rs:
  - IMPLEMENT ChangeApplicator struct
  - APPLY changes in correct order
  - HANDLE dependencies between sources
  - IMPLEMENT rollback mechanism
  - LOG all operations
  - MEASURE update performance

Task 10:
ADD signal handling:
  - IMPLEMENT SIGHUP handler for Unix
  - ADD Windows event handling
  - TRIGGER configuration reload
  - SUPPORT graceful shutdown
  - INTEGRATE with tokio runtime

Task 11:
CREATE integration tests:
  - TEST configuration file updates
  - VERIFY atomic updates
  - TEST rollback scenarios
  - BENCHMARK reload performance
  - VALIDATE event delivery

Task 12:
UPDATE documentation:
  - DOCUMENT configuration reload behavior
  - ADD examples for runtime updates
  - DESCRIBE event system
  - PROVIDE migration guide
```

### Out of Scope
- External control API (covered in PRP-17)
- WebSocket interface (covered in PRP-17)
- Dynamic codec changes
- Stream format conversion
- Configuration UI

## Success Criteria

- [ ] Configuration changes detected within 1 second
- [ ] Updates applied without stream interruption
- [ ] Invalid configurations rejected without affecting running system
- [ ] All configuration fields support runtime updates
- [ ] Event notifications delivered to all subscribers
- [ ] Zero downtime during configuration reload

## Dependencies

### Technical Dependencies
- notify crate for file watching
- tokio for async runtime
- serde for configuration parsing
- GStreamer pipeline state management

### Knowledge Dependencies
- File system event handling
- Atomic update patterns
- GStreamer element property changes
- Event-driven architecture

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| File system event storms | Medium | Low | Implement debouncing |
| Invalid configuration crashes | Low | High | Validation before apply |
| Race conditions during update | Medium | Medium | Atomic operations with locks |
| Memory leaks from watchers | Low | Medium | Proper cleanup handlers |

## Architecture Decisions

### Decision: File Watching Strategy
**Options Considered:**
1. Polling-based checking
2. OS native file system events
3. Hybrid approach with fallback

**Decision:** Option 3 - Hybrid with fallback

**Rationale:** Best compatibility across platforms, fallback for network filesystems

### Decision: Configuration Update Model
**Options Considered:**
1. Incremental property updates
2. Full configuration replacement
3. Diff-based patching

**Decision:** Option 3 - Diff-based patching

**Rationale:** Minimal disruption, clear change tracking, supports rollback

## Validation Strategy

- **Unit Tests**: Test each component in isolation
- **Integration Tests**: End-to-end configuration updates
- **Stress Tests**: Rapid configuration changes
- **Performance Tests**: Measure reload latency

## Future Considerations

- Configuration versioning and history
- Distributed configuration synchronization
- A/B testing support
- Feature flags integration
- Configuration templates

## References

- notify-rs Documentation
- Tokio Synchronization Primitives
- Runtime Configuration Best Practices
- GStreamer Dynamic Pipeline Modifications

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Complete
- **Confidence Level**: 8 - Well-researched with clear implementation path using proven patterns