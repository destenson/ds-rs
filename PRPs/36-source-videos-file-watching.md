# PRP: File Watching and Auto-Reload for Source-Videos

## Executive Summary

Implement file system watching capabilities to automatically detect changes in video source directories, enabling dynamic addition/removal of streams as files are added, modified, or deleted. Includes auto-repeat functionality for continuous playback and hot-reload of modified files.

## Problem Statement

### Current State
- Static source configuration requiring restart for changes
- No awareness of filesystem changes after startup
- Config watching exists but only for configuration files
- No automatic handling of completed video playback
- Manual intervention needed for source updates

### Desired State
- Automatic detection of new video files in watched directories
- Dynamic stream creation when files are added
- Graceful stream removal when files are deleted
- Auto-repeat/loop functionality for continuous playback
- Hot-reload when video files are modified
- Event-driven architecture for file system changes

### Business Value
Enables continuous testing scenarios, simplifies content management workflows, supports live production environments where video content changes dynamically, and reduces operational overhead.

## Requirements

### Functional Requirements

1. **Directory Watching**: Monitor directories for file changes
2. **File Addition**: Automatically serve new video files
3. **File Deletion**: Remove streams for deleted files
4. **File Modification**: Reload modified video files
5. **Auto-Repeat**: Loop video playback continuously
6. **Selective Watching**: Choose which directories/files to watch
7. **Event Notifications**: Emit events for file system changes

### Non-Functional Requirements

1. **Low Latency**: Detect changes within 1-2 seconds
2. **Efficiency**: Minimal CPU usage during idle watching
3. **Reliability**: Handle rapid file system changes gracefully
4. **Scalability**: Watch multiple directories simultaneously
5. **Resilience**: Recover from temporary file access issues

### Context and Research

Existing infrastructure:
- `ConfigWatcher` in `src/config/watcher.rs` uses notify crate
- Event system in `src/runtime/events.rs` for configuration changes
- VideoSourceManager supports dynamic source addition/removal

The notify crate provides efficient cross-platform file watching with:
- inotify on Linux
- FSEvents on macOS  
- ReadDirectoryChangesW on Windows

### Documentation & References

```yaml
- file: crates/source-videos/src/config/watcher.rs
  why: Existing file watching patterns using notify crate

- file: crates/source-videos/src/runtime/events.rs
  why: Event system for propagating changes

- file: crates/source-videos/src/manager.rs
  why: Dynamic source management interface

- url: https://docs.rs/notify/latest/notify/
  why: File watching crate documentation

- url: https://gstreamer.freedesktop.org/documentation/design/segments.html
  why: Looping/seeking for auto-repeat functionality

- file: crates/ds-rs/src/source/controller.rs
  why: Reference for dynamic source management patterns
```

### List of tasks to be completed

```yaml
Task 1:
CREATE src/watch/mod.rs:
  - DEFINE FileWatcher trait for abstraction
  - IMPLEMENT DirectoryWatcher using notify crate
  - ADD FileWatcher for individual files
  - INCLUDE event debouncing for rapid changes
  - CREATE WatcherManager to coordinate multiple watchers

Task 2:
CREATE src/watch/events.rs:
  - DEFINE FileSystemEvent enum (Created, Modified, Deleted, Renamed)
  - IMPLEMENT FileEventHandler trait
  - ADD event filtering and routing
  - CREATE event aggregation for batch changes
  - INCLUDE metadata with events (path, size, timestamp)

Task 3:
EXTEND src/runtime/events.rs:
  - ADD FileSystemChange variant to ConfigurationEvent
  - INCLUDE source mapping for file events
  - IMPLEMENT event correlation (file -> stream)
  - ADD batch event support

Task 4:
CREATE src/auto_repeat.rs:
  - IMPLEMENT LoopingVideoSource wrapper
  - ADD segment seeking for seamless looping
  - HANDLE EOS events for restart
  - INCLUDE configurable loop count/duration
  - SUPPORT gapless playback

Task 5:
MODIFY src/manager.rs:
  - INTEGRATE FileWatcher for watched sources
  - ADD auto-reload on file modification
  - IMPLEMENT source lifecycle based on file events
  - HANDLE temporary file unavailability
  - INCLUDE source state persistence

Task 6:
EXTEND src/config_types.rs:
  - ADD WatchConfig for file watching options
  - INCLUDE auto_repeat settings
  - ADD reload_on_change flag
  - DEFINE debounce duration settings
  - SUPPORT watch exclusion patterns

Task 7:
UPDATE src/file_source.rs:
  - ADD file modification detection
  - IMPLEMENT hot-reload capability
  - HANDLE file lock/access issues
  - INCLUDE retry logic for temporary failures
  - SUPPORT atomic file replacement

Task 8:
EXTEND CLI in src/main.rs:
  - ADD --watch/-w flag for directory watching
  - ADD --auto-repeat/-l flag for looping
  - ADD --reload-on-change flag
  - INCLUDE --watch-interval for poll frequency
  - SUPPORT --ignore patterns for exclusions

Task 9:
CREATE tests/file_watching_tests.rs:
  - TEST file addition detection
  - VERIFY file deletion handling
  - TEST modification reload
  - VALIDATE auto-repeat functionality
  - TEST rapid change handling

Task 10:
ADD examples/watched_directory.rs:
  - DEMONSTRATE directory watching setup
  - SHOW auto-repeat configuration
  - ILLUSTRATE event handling
  - EXAMPLE of hot-reload behavior
```

### Out of Scope
- File content analysis for change detection (use modification time)
- Versioning or history of file changes
- Distributed file watching across network shares
- Complex file synchronization logic

## Success Criteria

- [ ] New files in watched directories create streams within 2 seconds
- [ ] Deleted files remove corresponding streams gracefully
- [ ] Modified files trigger reload without stream interruption
- [ ] Auto-repeat provides seamless looping
- [ ] Multiple directories can be watched simultaneously
- [ ] Rapid file changes are debounced appropriately
- [ ] System remains responsive under heavy file activity
- [ ] Events are properly emitted for all file changes

## Dependencies

### Technical Dependencies
- notify crate (already in use) for file watching
- tokio for async event handling
- GStreamer segment events for looping

### Knowledge Dependencies
- Platform-specific file watching mechanisms
- GStreamer segment and seeking for seamless loops
- File system event debouncing strategies

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| File watching overhead | Low | Medium | Use native OS mechanisms via notify |
| Race conditions with rapid changes | Medium | Medium | Implement proper debouncing |
| File lock conflicts | Medium | Low | Retry logic with exponential backoff |
| Memory leaks from watchers | Low | High | Proper cleanup on drop |

## Architecture Decisions

### Decision: File Watching Strategy
**Options Considered:**
1. Polling-based checking
2. OS-native file watching (notify crate)
3. Hybrid with initial scan + watching

**Decision:** Option 3 - Hybrid approach

**Rationale:** Ensures no files are missed during startup while maintaining efficiency

### Decision: Auto-Repeat Implementation
**Options Considered:**
1. Pipeline restart on EOS
2. Segment seeking for gapless loop
3. Multiple pipeline alternation

**Decision:** Option 2 - Segment seeking

**Rationale:** Provides smoothest playback without interruption

### Decision: Change Debouncing
**Options Considered:**
1. Fixed delay debouncing
2. Adaptive debouncing based on activity
3. Event coalescing with time window

**Decision:** Option 3 - Event coalescing

**Rationale:** Handles both single changes and batch operations efficiently

## Validation Strategy

### Validation Commands
```bash
# Test file watching
cargo test file_watching

# Run with directory watching
cargo run -- serve -d /videos --watch --auto-repeat

# Test hot reload
touch /videos/test.mp4 # Should trigger reload

# Verify continuous playback
cargo run --example watched_directory

# Test with multiple watched directories
cargo run -- serve -d /videos1 -d /videos2 --watch
```

## Future Considerations

- Integration with inotify-tools for advanced filtering
- Distributed watching across network filesystems
- Smart caching of frequently accessed files
- Predictive preloading based on access patterns
- WebSocket notifications for file change events

## References

- notify crate documentation: https://docs.rs/notify/
- GStreamer seeking and segments documentation
- File system monitoring best practices

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-01-25
- **Status**: Ready for Implementation
- **Confidence Level**: 9/10 - Well-understood problem with existing patterns