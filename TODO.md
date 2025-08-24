# TODO List

Last Updated: 2025-08-24 (Complete codebase scan)

## Critical Priority 🔴

### Active Critical Issues
✅ **ALL CRITICAL ISSUES RESOLVED** - Application is fully functional!

### Recently Fixed (2025-08-24)
- [x] **Fixed video playback state management** ✅ (PRP-03 COMPLETED)
  - Pipeline now properly reaches PLAYING state
  - Video window appears and displays content correctly
  - Solution: Fixed initialization order, proper async state handling, sync_state_with_parent()
  
- [x] **Fixed application shutdown issue** ✅ (PRP-25 COMPLETED)
  - Application shuts down properly on Ctrl+C
  - Solution: GLib's unix_signal_add + MainLoop.run() pattern
  
- [x] **Fixed video playback freezing** ✅
  - Added videorate and capsfilter for framerate normalization
  - Videos play smoothly without freezing

## High Priority 🟡

### Code Quality & Production Readiness
- [ ] **Replace unwrap() calls in production code** (146 occurrences across 32 files)
  - Highest priority files:
    - `backend/cpu_vision/elements.rs`: 22 instances
    - `backend/cpu_vision/cpudetector/imp.rs`: 16 instances  
    - `source/mod.rs`: 9 instances
    - `config/mod.rs`: 8 instances
    - `source/events.rs`: 8 instances
    - `source/video_source.rs`: 8 instances
  - Replace with proper error handling using `?` operator

- [ ] **Complete TODO comments in code** (8 occurrences)
  - `Cargo.toml:3-4`: Use workspace version and edition
  - `Cargo.toml:52`, `source-videos/Cargo.toml:20`: Remove tokio dependency
  - `tests/cpu_backend_tests.rs:343`: Test with actual ONNX model file
  - `rendering/deepstream_renderer.rs:190,222`: Implement DeepStream metadata processing
  - `backend/cpu_vision/cpudetector/imp.rs:154`: Attach custom metadata to buffer

- [ ] **Handle unimplemented!() calls**
  - `backend/cpu_vision/cpudetector/imp.rs`: 2 occurrences in match statements
  - Replace with proper error handling

- [ ] **Clean up unused parameters** (33 occurrences with underscore prefix)
  - Review and either use or document why they're unused
  - Many are legitimate in trait implementations

### DeepStream Integration (PRP-04)
- [ ] **Implement NvDsMeta extraction with FFI bindings**
  - Required for actual object detection metadata
  - Currently returns mock data
  
- [ ] **Complete DeepStream renderer metadata processing**
  - `rendering/deepstream_renderer.rs:190,222`: Implement actual processing
  - Create and attach NvDsObjectMeta

### Testing Infrastructure
- [ ] **Fix ignored test**
  - `tests/main_app_test.rs:23`: Test marked as ignored due to runtime requirements
  - Enable or create mock version

- [ ] **Add ONNX model integration tests**
  - `tests/cpu_backend_tests.rs:343`: Test with real YOLO models
  - Download and validate YOLOv5/v8/v12 models

## Medium Priority 🟢

### Build Configuration
- [ ] **Fix workspace configuration**
  - `Cargo.toml:3-4`: Use workspace version/edition instead of hardcoded values
  - Currently: "0.1.0" and "2024"

- [ ] **Review and remove tokio dependency**
  - `Cargo.toml:52`, `source-videos/Cargo.toml:20`: Both have TODO comments
  - Async is ok, but full tokio may not be needed

### DSL Library
- [ ] **Implement dsl crate**
  - `dsl/src/lib.rs`: Contains single `todo!()` macro
  - DeepStream Services Library implementation pending

### CPU Vision Backend Enhancements
- [ ] **Complete metadata attachment in CPU detector**
  - `backend/cpu_vision/cpudetector/imp.rs:154`: Attach detection results to buffer
  - Currently passes through without metadata

### Documentation
- [ ] Add inline documentation for all public APIs
- [ ] Create architecture diagrams
- [ ] Write migration guide from C to Rust
- [ ] Document metadata extraction architecture

## Low Priority 🔵

### Code Cleanup
- [ ] Review 33 unused parameters (underscore-prefixed)
- [ ] Add missing match arms instead of unimplemented!()
- [ ] Standardize error handling patterns

### Performance Optimizations
- [ ] Profile and optimize hot paths
- [ ] Reduce allocations in frame processing
- [ ] Implement zero-copy buffer passing where possible

### Future Enhancements
- [ ] Add native RTSP server support
- [ ] Implement custom inference post-processing
- [ ] Create Docker container for deployment
- [ ] Add WebRTC sink support
- [ ] Implement cloud inference backend

## Completed PRPs ✅
- PRP-01: Core Infrastructure
- PRP-02: GStreamer Pipeline
- PRP-03: Fix Video Playback State Management ✅ (2025-08-24)
- PRP-06: Hardware Abstraction
- PRP-07: Dynamic Video Sources
- PRP-08: Code Quality
- PRP-09: Test Orchestration Scripts
- PRP-11: Real-time Bounding Box Rendering ✅ (2025-08-24)
- PRP-14: Backend Integration
- PRP-15: Element Discovery
- PRP-16: Runtime Configuration Management
- PRP-25: Fix Shutdown Window Race Condition

## In Progress PRPs 🔄
- PRP-04: DeepStream Integration (metadata extraction needed)
- PRP-20: CPU Vision Backend (detector/tracker stubs exist)
- PRP-21: CPU Detection Module (stub implementation)
- PRP-22: CPU Tracking Module (stub implementation)

## Not Started PRPs ⏳
- PRP-05: Main Application (demo incomplete)
- PRP-10: Ball Detection Integration
- PRP-12: Multistream Detection Pipeline
- PRP-13: Detection Data Export/Streaming
- PRP-17: Control API WebSocket
- PRP-18: Dynamic Source Properties
- PRP-19: Network Simulation
- PRP-23: GST Plugins Integration
- PRP-27: Multi-Backend Detector Trait Architecture
- PRP-28: OpenCV DNN Backend
- PRP-29: TensorFlow Lite Backend
- PRP-30: Darknet Native Backend
- PRP-31: Advanced Tracking Algorithms

## Statistics 📊

### Code Quality Metrics
- **unwrap() calls**: 146 occurrences across 32 files
- **TODO comments**: 8 found
- **todo!() macros**: 1 (in dsl crate)
- **unimplemented!()**: 2 occurrences
- **Unused parameters**: 33 underscore-prefixed variables
- **Ignored tests**: 1 test requiring runtime

### Project Status
- **Critical Bugs**: 0 (ALL RESOLVED ✅)
- **Build Status**: ✅ SUCCESS
- **Test Status**: 13/13 pipeline tests passing
- **PRP Progress**: 13/31 complete (42%), 4/31 in progress (13%), 14/31 not started (45%)

### Recent Achievements (Last 24 Hours)
- Fixed video playback state management (PRP-03)
- Implemented real-time bounding box rendering (PRP-11)
- Fixed f16/f32 array conversion issues
- Fixed application test compilation errors
- Enhanced timestamp logging throughout

## Priority Focus

### Immediate Next Steps
1. **Code Quality**: Replace 146 unwrap() calls with proper error handling
2. **DeepStream FFI**: Implement metadata extraction for real object detection
3. **Testing**: Add ONNX model integration tests with real YOLO models
4. **Build Config**: Fix workspace configuration issues

### Technical Debt
- Mock implementations need replacement with real functionality
- Unused parameters need review and cleanup
- Error handling needs standardization

## Contributing

When working on any TODO item:
1. Create a feature branch
2. Update this TODO.md to mark item as in-progress
3. Write tests for new functionality
4. Update documentation as needed
5. Mark complete in TODO.md when merged

---

**Status: Application fully functional - No critical bugs! 🎉**
**Focus: Code quality improvements and feature completion**