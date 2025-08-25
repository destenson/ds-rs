# PRP-37: Fix RTSP File Serving Architecture

## Problem Statement

The current implementation incorrectly mixes local playback and RTSP serving concepts. When serving video files through RTSP, the `VideoSourceManager` creates `VideoSource` instances (like `FileVideoSource`) and calls `start()` on them, which creates local playback pipelines with `autovideosink`. This causes port binding conflicts and prevents proper RTSP streaming.

The architecture should work as follows:
- **For RTSP serving**: Only use `VideoSourceConfig` objects passed to `RtspServerBuilder`
- **For local playback**: Create and manage `VideoSource` instances  
- **Never mix the two**: RTSP server handles all pipeline creation through media factories

## Root Cause Analysis

### Current (Broken) Flow
1. `main.rs` creates `VideoSourceConfig` objects for files
2. Passes configs to `RtspServerBuilder.add_source()`
3. `RtspServerBuilder.build()` creates `RtspServer` 
4. `RtspServer.add_source()` creates media factories correctly
5. **BUT**: `VideoSourceManager` is also creating `VideoSource` instances and calling `start()`
6. This creates local playback pipelines that conflict with RTSP

### Evidence
- In `crates/source-videos/src/manager.rs:51`: `source.start()?` is called when adding sources
- In `crates/source-videos/src/file_source.rs:52-116`: `create_pipeline()` creates local playback pipeline with `autovideosink`
- Error message: "Only one usage of each socket address" indicates multiple servers/pipelines

## Solution Design

### Architectural Principles
1. **Separation of Concerns**: RTSP serving and local playback are separate use cases
2. **Config vs Runtime**: `VideoSourceConfig` is configuration data; `VideoSource` is a runtime object
3. **Single Responsibility**: `RtspServer` owns all RTSP serving; `VideoSourceManager` owns local playback

### Implementation Strategy

#### Phase 1: Clarify VideoSourceManager Purpose
The `VideoSourceManager` should be for local playback management only. For RTSP serving, we only need configs.

#### Phase 2: Fix the Serve Command Flow
The serve command in `main.rs` should:
1. Create `VideoSourceConfig` objects from directory/files
2. Pass configs to `RtspServerBuilder`
3. Build and start the RTSP server
4. NOT create any `VideoSource` instances
5. NOT use `VideoSourceManager` for RTSP serving

#### Phase 3: Separate File Watching for RTSP
File watching for RTSP should:
1. Monitor directories for changes
2. Create/update `VideoSourceConfig` objects
3. Add/remove sources from the running RTSP server
4. NOT create `VideoSource` instances

## Tasks

### 1. Remove VideoSourceManager from RTSP Serving Path
- File: `crates/source-videos/src/main.rs`
- In `serve_command` function:
  - Remove any VideoSourceManager usage
  - Only work with VideoSourceConfig objects
  - Pass configs directly to RtspServerBuilder

### 2. Create RTSP-specific File Watching Integration
- File: `crates/source-videos/src/rtsp/mod.rs`
- Add method to RtspServer: `update_source(mount_point: &str, config: VideoSourceConfig)`
- Add method: `handle_file_event(event: FileSystemEvent)`
- This keeps file watching but routes it through RTSP server, not VideoSourceManager

### 3. Fix File Path Handling in Media Factory
- File: `crates/source-videos/src/rtsp/factory.rs`
- Already fixed: Windows path conversion with `path.replace('\\', "/")`
- Verify this works correctly

### 4. Update Integration Points
- File: `crates/source-videos/src/lib.rs`
- Ensure `SourceVideos` struct doesn't mix concerns
- RTSP server should work independently of VideoSourceManager

### 5. Add Tests for RTSP File Serving
- File: `crates/source-videos/tests/rtsp_file_serving_test.rs` (new)
- Test that file sources can be served through RTSP
- Test that no local pipelines are created
- Test file watching with RTSP updates

## Implementation Order

1. First, fix `main.rs` serve_command to not use VideoSourceManager
2. Test that basic RTSP file serving works
3. Add RTSP-specific file watching
4. Test file watching with RTSP
5. Add comprehensive tests

## Validation Gates

```bash
# Build and check
cd crates/source-videos
cargo build --release

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Manual test - start server with files
cargo run -- serve -d D:/files/large/vid -r

# In another terminal, test with GStreamer tools or VLC
# Should be able to connect to rtsp://localhost:8554/<mount_point>
```

## Success Criteria

1. Video files can be served through RTSP without errors
2. No "port already in use" errors
3. VLC/GStreamer can connect and play the streams
4. File watching updates RTSP server without creating local pipelines
5. Clean separation between RTSP serving and local playback code

## References

### Internal Code Patterns
- `../gstreamer-rs/examples/src/bin/rtsp-server.rs` - Shows proper RTSP server setup
- `crates/source-videos/src/rtsp/factory.rs` - Already has correct media factory creation
- `crates/source-videos/src/rtsp/mod.rs` - RtspServer implementation

### Key Concepts
- RTSP MediaFactory: Creates pipelines on-demand for each client
- Launch string: GStreamer pipeline description for media factory
- Mount points: URL paths where streams are available
- Shared vs non-shared: Whether clients share the same pipeline instance

## Notes

- The existing RTSP server code is correct - it properly creates media factories
- The bug is in the inappropriate use of VideoSourceManager for RTSP serving
- File watching should update the RTSP server directly, not go through VideoSourceManager
- Test patterns work because they don't create VideoSource instances

## Confidence Score: 8/10

The problem is well understood and the solution is clear. The existing RTSP infrastructure is correct, we just need to remove the inappropriate VideoSourceManager usage from the RTSP serving path. The main risk is ensuring file watching continues to work correctly after the refactoring.