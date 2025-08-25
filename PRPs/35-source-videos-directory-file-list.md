# PRP: Directory and File List Support for Source-Videos CLI

## Executive Summary

Expand the source-videos CLI to serve video files from directories and file lists, supporting recursive directory traversal, multiple file formats, and dynamic source discovery. This enables batch processing of existing video assets and simplifies testing scenarios with real video files.

## Problem Statement

### Current State
- source-videos can only generate test patterns or serve single predefined sources
- No support for serving existing video files from the filesystem
- Cannot process directories of video files
- No batch file serving capabilities
- Limited to programmatically defined sources

### Desired State
- Serve all video files from a directory as RTSP streams
- Support recursive directory traversal for nested video collections
- Accept file lists (via config or command line) for specific file serving
- Auto-detect video formats and configure appropriate pipelines
- Dynamic mount point generation based on file paths
- Support for mixed sources (files + patterns) in single server

### Business Value
Enables realistic testing with actual video content, simplifies batch processing workflows, and provides a bridge between file-based video assets and streaming infrastructure testing.

## Requirements

### Functional Requirements

1. **Directory Serving**: Process all video files in specified directories
2. **Recursive Traversal**: Option to include subdirectories
3. **File List Support**: Accept explicit lists of files to serve
4. **Format Detection**: Auto-detect video container formats (mp4, mkv, avi, etc.)
5. **Mount Point Generation**: Create RTSP endpoints based on file paths
6. **Mixed Sources**: Combine file sources with test patterns
7. **Filter Support**: Include/exclude patterns for file selection

### Non-Functional Requirements

1. **Performance**: Lazy loading of file sources to minimize startup time
2. **Scalability**: Handle directories with 100+ video files
3. **Memory Efficiency**: Stream files without loading entire content
4. **Error Handling**: Graceful handling of invalid/corrupt video files
5. **Compatibility**: Support common video formats (H.264, H.265, VP8, VP9)

### Context and Research

The existing codebase has:
- `FileGenerator` in `src/file.rs` for creating video files
- `VideoSourceConfig` with File variant in `src/config_types.rs`
- Basic file watching in `src/config/watcher.rs` (for config files)
- RTSP server infrastructure in `src/rtsp/`

Reference implementations:
- GStreamer's `multifilesrc` element for sequential file processing
- The `notify` crate already used for config watching can monitor directories
- Standard library's `std::fs::read_dir` and `walkdir` crate for directory traversal

### Documentation & References

```yaml
- file: crates/source-videos/src/file.rs
  why: Existing file handling patterns to extend

- file: crates/source-videos/src/config_types.rs
  why: VideoSourceConfig structure needs directory/list variants

- file: crates/source-videos/src/rtsp/mod.rs
  why: RTSP server integration for file sources

- file: crates/source-videos/src/manager.rs
  why: VideoSourceManager needs batch source addition

- url: https://gstreamer.freedesktop.org/documentation/coreelements/filesrc.html
  why: GStreamer filesrc element for serving files

- url: https://docs.rs/walkdir/latest/walkdir/
  why: Recursive directory traversal patterns

- url: https://gstreamer.freedesktop.org/documentation/playback/uridecodebin.html
  why: Auto-detection of file formats using uridecodebin
```

### List of tasks to be completed

```yaml
Task 1:
EXTEND src/config_types.rs:
  - ADD Directory variant to VideoSourceType enum
  - ADD FileList variant for explicit file lists
  - INCLUDE options for recursion, filters, format hints
  - ADD DirectoryConfig and FileListConfig structs

Task 2:
CREATE src/directory.rs:
  - IMPLEMENT DirectoryScanner for discovering video files
  - ADD recursive traversal with walkdir crate
  - INCLUDE format detection based on extensions
  - CREATE filter system for include/exclude patterns
  - GENERATE appropriate VideoSourceConfig for each file

Task 3:
CREATE src/file_source.rs:
  - IMPLEMENT FileVideoSource wrapping filesrc + decodebin
  - ADD format auto-detection using uridecodebin
  - HANDLE various container formats (mp4, mkv, avi, webm)
  - INCLUDE error recovery for corrupt files
  - SUPPORT seeking and looping for continuous playback

Task 4:
MODIFY src/manager.rs:
  - ADD batch source registration from directories
  - IMPLEMENT lazy loading of file sources
  - HANDLE dynamic source addition as files discovered
  - INCLUDE source deduplication for overlapping paths

Task 5:
EXTEND src/rtsp/mod.rs:
  - GENERATE mount points from file paths
  - HANDLE special characters in filenames
  - SUPPORT hierarchical mount points for nested directories
  - ADD metadata endpoints for file information

Task 6:
UPDATE src/main.rs CLI:
  - ADD --directory/-d flag for directory serving
  - ADD --recursive/-r flag for recursive traversal
  - ADD --files/-f flag for explicit file list
  - ADD --include/--exclude patterns for filtering
  - SUPPORT multiple directories/files in single command

Task 7:
CREATE src/file_utils.rs:
  - IMPLEMENT video file detection utilities
  - ADD MIME type detection for format hints
  - CREATE path to mount point conversion
  - INCLUDE file metadata extraction (duration, resolution)

Task 8:
ADD tests/directory_serving_tests.rs:
  - TEST directory scanning with various structures
  - VALIDATE recursive traversal
  - TEST filter patterns
  - VERIFY mount point generation
  - TEST mixed source scenarios

Task 9:
UPDATE examples/:
  - CREATE directory_server.rs example
  - ADD batch_file_server.rs for file lists
  - SHOW mixed source configurations
  - DEMONSTRATE filter usage

Task 10:
UPDATE documentation:
  - DOCUMENT new CLI options
  - ADD configuration examples for directories
  - EXPLAIN mount point generation rules
  - INCLUDE troubleshooting for common file issues
```

### Out of Scope
- Transcoding of incompatible formats (use original encoding)
- Database/index of served files (keep it stateless)
- Thumbnail generation or preview features
- File modification/upload capabilities

## Success Criteria

- [ ] Can serve all video files from a directory as RTSP streams
- [ ] Recursive directory traversal works correctly
- [ ] File lists can be specified via config or CLI
- [ ] Auto-detection of video formats works for common containers
- [ ] Mount points are generated predictably from file paths
- [ ] Mixed sources (files + patterns) work together
- [ ] Filter patterns correctly include/exclude files
- [ ] Performance is acceptable with 100+ files
- [ ] Invalid files are handled gracefully

## Dependencies

### Technical Dependencies
- walkdir crate for efficient directory traversal
- mime_guess or similar for format detection
- Existing GStreamer pipeline infrastructure
- notify crate (already present) for potential file watching

### Knowledge Dependencies
- GStreamer's uridecodebin for format auto-detection
- RTSP URL encoding standards for special characters
- Common video container formats and their detection

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Large directories causing slow startup | Medium | Medium | Implement lazy loading and pagination |
| Incompatible video formats | High | Low | Graceful fallback, clear error messages |
| File path to URL conversion issues | Medium | Medium | Robust escaping and validation |
| Memory usage with many sources | Low | High | Use streaming, not preloading |

## Architecture Decisions

### Decision: Directory Traversal Strategy
**Options Considered:**
1. Eager loading of all files at startup
2. Lazy discovery as streams requested
3. Background scanning with progressive loading

**Decision:** Option 3 - Background scanning with progressive loading

**Rationale:** Provides quick startup while ensuring all files are eventually available

### Decision: File Format Detection
**Options Considered:**
1. Extension-based detection only
2. Magic number/header inspection
3. GStreamer's uridecodebin auto-detection

**Decision:** Option 3 - GStreamer's uridecodebin

**Rationale:** Most reliable and handles edge cases automatically

## Validation Strategy

### Validation Commands
```bash
# Build with file serving features
cargo build --features file-serving

# Test directory serving
cargo test directory_serving

# Run directory server example
cargo run --example directory_server -- -d /path/to/videos -r

# Test with mixed sources
cargo run -- serve -d /videos --patterns smpte,ball

# Verify RTSP streams
ffplay rtsp://localhost:8554/videos/sample.mp4
```

## Future Considerations

- Watch directories for new files (covered in next PRP)
- Playlist generation from directories
- Metadata extraction and serving
- Thumbnail generation for file browser
- Integration with cloud storage backends

## References

- GStreamer filesrc and uridecodebin documentation
- RTSP URL encoding RFC 3986
- Common video container format specifications

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-01-25
- **Status**: Ready for Implementation
- **Confidence Level**: 8/10 - Clear requirements with existing patterns to follow