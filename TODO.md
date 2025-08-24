# TODO List

Last Updated: 2025-08-24 (Updated after comprehensive scan of current state)

## Critical Priority 🔴

### Critical Bugs (from BUGS.md) - UPDATED
- [x] **Fix application shutdown issue** ✅ (2025-08-23 - PRP-25 COMPLETED)
  - FIXED: Application now shuts down properly on Ctrl+C
  - FIXED: No more repeated "shutting down..." messages  
  - FIXED: Race conditions eliminated with GLib MainLoop integration
  - Solution: GLib's unix_signal_add + MainLoop.run() pattern
  
- [ ] **Fix video playback freezing**
  - Video gets stuck after first/last frame
  - H264 parser warning: "VUI framerate 15360.0 exceeds allowed maximum 32.8"
  - Caps/framerate handling needs investigation

### CPU Vision Backend Implementation
- [x] **Fix ONNX Runtime API compatibility issues** ✅ (2025-08-23 - PRP-24)
  - Successfully fixed all ONNX Runtime v1.16.3 API issues
  - Supports YOLOv3-v12 and YOLO-RD with automatic version detection
  - Model files available from Ultralytics and official YOLO repos
- [ ] **Complete ONNX detector integration tests**
  - `crates/ds-rs/tests/cpu_backend_tests.rs:342`: TODO comment - test with actual ONNX model file
  - Need to download and test with real YOLOv5/v8/v12 models
  - Validate inference performance matches expected FPS targets

### DeepStream Integration (PRP-04)
- [ ] **Implement NvDsMeta extraction with FFI bindings**
  - `metadata/mod.rs:61,72`: Currently returns mock metadata ("For now" comment)
  - Need to implement actual `gst_buffer_get_nvds_batch_meta` FFI binding
  - Related: Known limitation from CLAUDE.md

- [ ] **Implement stream-specific EOS messages**
  - `messages/mod.rs:174,181`: Returns mock implementation ("For now" comments)
  - `pipeline/bus.rs:216`: Requires FFI for `gst_nvmessage_is_stream_eos` ("For now" comment)
  - Need `gst_nvmessage_parse_stream_eos` binding

### Code Quality & Production Readiness  
- [ ] Replace `unwrap()` calls in production code (100 occurrences across 27 files in ds-rs/src)
  - **Highest priority files**: 
    - `backend/cpu_vision/elements.rs`: 16 instances
    - `source/mod.rs`: 9 instances
    - `config/mod.rs`: 8 instances  
    - `source/events.rs`: 8 instances
    - `source/video_source.rs`: 6 instances
    - `backend/mock.rs`: 4 instances
- [x] Fix build warnings ✅ (2025-08-23)
  - `backend/cpu_vision/detector.rs`: Fixed unused imports
  - `backend/cpu_vision/elements.rs`: Removed unused Arc, Mutex imports
- [ ] Clean up unused parameters (25+ underscore-prefixed variables found)
  - Callback handlers: `_bus`, `_pad`, `_info` in multiple probe callbacks
  - Function parameters: `_path`, `_msg`, `_decodebin` indicating incomplete implementations
  - Test/mock code appropriately uses underscore prefix

### Test Issues
- [x] **Fix ONNX detector compilation** ✅ (2025-08-23)
  - Compilation now succeeds with ort feature enabled
  - Tests pass without ort feature using mock detector
- [ ] **Add ONNX integration tests with real models**
  - `crates/ds-rs/tests/cpu_backend_tests.rs:342`: TODO comment - test with actual ONNX model file
  - Download actual YOLO models for testing
  - Validate inference accuracy and performance
  - Test all supported YOLO versions (v3-v12)
- [ ] **Fix source-videos file generation test**
  - `integration_test.rs`: test_file_generation times out after 11 seconds
  
### Placeholder Implementation Resolution
- [ ] **Complete metadata attachment in CPU detector**
  - `backend/cpu_vision/cpudetector/imp.rs:167-168`: TODO comment about attaching custom metadata to buffer
  - Currently just passes through without attaching detection metadata
- [ ] **Replace stub implementations with actual functionality**
  - `metadata/mod.rs:60`: "In a real implementation, this would call gst_buffer_get_nvds_batch_meta"
  - `metadata/mod.rs:125`: `_batch_meta` unused - extraction not implemented
  - `messages/mod.rs:174`: "In real DeepStream, this would call gst_nvmessage_is_stream_eos"
  - `messages/mod.rs:181`: "In real DeepStream, this would call gst_nvmessage_parse_stream_eos"
  - `inference/mod.rs:174-175`: "In a real implementation, this would parse a label file" ("for now" comment)
  - `inference/config.rs:227-228`: `from_deepstream_config` takes `_path` - returns mock config ("for now" comment)
  - `backend/cpu_vision/elements.rs:108-109`: Detection metadata attachment placeholder ("For now" comment)
  - `backend/cpu_vision/elements.rs:145-146`: Image creation placeholder ("For now" comment) 
  - `backend/cpu_vision/elements.rs:235`: CPU tracker currently just passes through ("For now" comment)
  - `cpuinfer/detector.rs:417`: YOLOv10 NMS-free design handling placeholder ("For now" comment)
  - `pipeline/bus.rs:214`: `is_stream_eos` takes `_msg` - returns None (mock)
  - `source/controller.rs:154`: `_event_handler` cloned but not used
  - `backend/standard.rs`: Tiler comment says "actual tiling is done by the compositor"
  - `platform.rs:149`: GPU capabilities returns common capabilities ("for now" comment)

### Build Configuration
- [ ] **Fix workspace configuration**
  - `crates/ds-rs/Cargo.toml:3`: `# TODO: use the workspace version`
  - `crates/ds-rs/Cargo.toml:4`: `# TODO: use the workspace edition`
  - Currently hardcoded as "0.1.0" and "2024"
- [ ] **Review tokio dependency usage**
  - `crates/ds-rs/Cargo.toml:48`: TODO comment - should not use tokio (async is ok though)
  - `crates/source-videos/Cargo.toml:20`: Similar tokio dependency with TODO comment
  - Consider replacing with async-std or futures-lite where possible

## High Priority 🟡

### Core Functionality 
- [ ] **Complete main demo application (PRP-05)**
  - `tests/main_app_test.rs:23`: Test marked as ignored due to runtime requirements
  - Full application matching C reference implementation needed
  - Related: Known limitation from CLAUDE.md

- [ ] **Implement DeepStream config file parsing**
  - `inference/config.rs:228`: `from_deepstream_config()` returns mock ("for now" comment)
  - Need to parse actual .txt config format

- [ ] **Implement label map file loading**
  - `inference/mod.rs:175`: `load_from_file()` returns default map ("for now" comment)
  - Parse actual label files

### Testing & CI/CD Infrastructure  
- [ ] Fix Source Management tests with Mock backend limitations
  - 10 tests fail because Mock backend doesn't support uridecodebin
  - Consider creating dedicated test backend or skip tests conditionally
- [ ] Set up GitHub Actions CI/CD pipeline
- [ ] Add integration tests using source-videos test infrastructure
- [ ] Implement stress testing for concurrent source operations
- [ ] Add memory leak detection tests

### Configuration & Build Issues
- [ ] Fix deprecated rand API usage in timer implementations
  - Update to modern thread_rng() patterns

## Medium Priority 🟢

### Platform Improvements
- [ ] **Implement GPU capabilities detection**
  - `platform.rs:149`: Currently returns common capabilities ("for now" comment)
  - Need nvidia-smi or CUDA API calls for actual detection

- [ ] **Runtime configuration enhancements**
  - `source-videos/src/manager.rs:215`: `modify_source_config()` is placeholder ("for now" comment)
  - `source-videos/src/runtime/applicator.rs:72,81,88`: Partial implementation ("for now" comments)

### Code Cleanup
- [ ] **Remove/implement unused parameter functions** (40+ occurrences)
  - Examples: `detection_app.rs:74,180,268` - Unused inference processor and callback parameters
  - Tests: `pipeline_tests.rs:177` - Unused bus callback parameter
  - Source videos: `file.rs:50,136`, `config/watcher.rs:13,34,66,108`, `rtsp/mod.rs:92`, `rtsp/factory.rs:68,109`
  - Source management: `video_source.rs:66,181,234`, `synchronization.rs:22,127,138`, `manager.rs:46,210`, `controller.rs:154`
  - CPU Vision: `backend/cpu_vision/elements.rs:86-88` - Unused tracker and buffer parameters
  - Inference: `inference/config.rs:226`, `inference/mod.rs:173`
  - Metadata: `metadata/mod.rs:123,125` - Unused extraction results
  - Message handling: `pipeline/bus.rs:46,214,223,325,378`, `messages/mod.rs:193,317`
  - Pipeline: `state.rs:196,291,292`
  - Timers: `app/timers.rs:35` - Unused source ID
  - Various handler parameters with `_` prefix indicating incomplete implementations

### Testing & Examples
- [ ] Create test RTSP source for better integration testing
- [ ] Add integration tests with actual video files
- [ ] Create example for each backend type
- [ ] Add memory leak tests (valgrind)
- [ ] Implement stress tests for rapid source changes

### Documentation
- [ ] Add inline documentation for all public APIs
- [ ] Create architecture diagrams
- [ ] Write migration guide from C to Rust
- [ ] Document backend selection logic
- [ ] Add troubleshooting guide for common issues
- [ ] Document metadata extraction architecture

## Low Priority 🔵

### Dependency Preparation
- [ ] **Enable commented dependencies when needed** (`crates/ds-rs/Cargo.toml`)
  - Line 36: ort (ONNX Runtime) - needed for CPU detector
  - Line 38: imageproc - needed for image processing  
  - Line 40: ndarray - needed for tensor operations

### CI/CD & Infrastructure
- [ ] Set up GitHub Actions workflow
- [ ] Add multi-platform CI testing (Linux, Windows, macOS)
- [ ] Configure automated releases
- [ ] Add code coverage reporting
- [ ] Set up dependency updates (dependabot)
- [ ] Add benchmark suite

### Future Enhancements
- [ ] Add native RTSP server support
- [ ] Implement custom inference post-processing
- [ ] Add performance profiling tools
- [ ] Create Docker container for easy deployment
- [ ] Add WebRTC sink support
- [ ] Implement cloud inference backend
- [ ] Add Kubernetes deployment manifests

### Performance Optimizations
- [ ] Profile and optimize hot paths
- [ ] Reduce allocations in frame processing
- [ ] Implement zero-copy buffer passing where possible
- [ ] Add frame skipping for overloaded systems
- [ ] Optimize backend detection caching

## New Feature Development 🚀

### Computer Vision & Object Detection (PRPs 10-13)
- [ ] **Ball Detection Integration (PRP-10)**
  - Implement OpenCV-based circle detection for bouncing balls
  - Add HSV color thresholding for improved accuracy
  - Integrate with existing DetectionResult infrastructure
  - Target: Real-time performance (>15 FPS)
  
- [ ] **Real-time Bounding Box Rendering (PRP-11)**
  - Connect detection results to OSD pipeline
  - Implement dynamic bounding box visualization
  - Add configurable visual appearance (colors, thickness)
  - Ensure frame-synchronized rendering
  
- [ ] **Multi-Stream Detection Pipeline (PRP-12)**
  - Scale to 4+ concurrent RTSP streams
  - Implement resource scheduling and load balancing
  - Add stream isolation and error handling
  - Target: >15 FPS per stream with detection
  
- [ ] **Detection Data Export (PRP-13)**
  - Export detection data to MQTT, RabbitMQ
  - Add database persistence (PostgreSQL, MongoDB)
  - Implement file-based export with rotation
  - Support configurable serialization formats

### Test Orchestration (PRP-09) ✅
- [x] **Integration Test Scripts** - COMPLETED
  - Created PowerShell orchestration scripts for Windows
  - Implemented Python cross-platform test runner
  - Added shell scripts for Linux/macOS
  - Setup automated RTSP server management
  
- [x] **End-to-End Testing** - COMPLETED
  - Configured test scenarios in TOML
  - Implemented backend-specific test suites
  - Added GitHub Actions CI/CD workflow
  - Created environment validation script

## Completed PRPs ✅
- PRP-01: Core Infrastructure
- PRP-02: GStreamer Pipeline
- PRP-03: Source Control APIs
- PRP-06: Hardware Abstraction
- PRP-07: Dynamic Video Sources
- PRP-08: Code Quality
- PRP-09: Test Orchestration Scripts ✅ (2025-08-23)
- PRP-14: Backend Integration
- PRP-15: Element Discovery
- PRP-16: Runtime Configuration Management
- PRP-25: Fix Shutdown Window Race Condition ✅ (2025-08-23)

## In Progress PRPs 🔄
- PRP-20: CPU Vision Backend (partial - detector/tracker stubs exist)
- PRP-21: CPU Detection Module (stub implementation)
- PRP-22: CPU Tracking Module (stub implementation)

## Not Started PRPs ⏳
- PRP-04: DeepStream Integration (metadata extraction needed)
- PRP-05: Main Application (demo incomplete)
- PRP-10: Ball Detection Integration
- PRP-11: Realtime Bounding Box Rendering
- PRP-12: Multistream Detection Pipeline
- PRP-13: Detection Data Export/Streaming
- PRP-17: Control API WebSocket
- PRP-18: Dynamic Source Properties
- PRP-19: Network Simulation
- PRP-23: GST Plugins Integration
- PRP-27: Multi-Backend Detector Trait Architecture (NEW)
- PRP-28: OpenCV DNN Backend (NEW)
- PRP-29: TensorFlow Lite Backend (NEW)
- PRP-30: Darknet Native Backend (NEW)
- PRP-31: Advanced Tracking Algorithms (NEW)

## Recently Completed ✅

### Latest Completions (2025-08-23)
- [x] **Test Orchestration Scripts (PRP-09) ✅**
  - Created Python/PowerShell/Shell test orchestrators for all platforms
  - Implemented 8 test scenarios with TOML configuration
  - Added automated RTSP server management and test file generation
  - Created GitHub Actions CI/CD workflow
  - Added environment validation script
- [x] **CPU Vision Backend Foundation (PRP-20 partial)**
  - Created cpu_vision module structure with detector, tracker, elements
  - Implemented ONNX model loading (needs API compatibility fixes)
  - Centroid tracker with trajectory history implemented
  - GStreamer element wrappers for detection/tracking/OSD created
- [x] **Fixed RTSP Server Issues in source-videos**
  - Fixed GLib main loop integration for RTSP server
  - Corrected test pattern configuration issues
  - Resolved property type mismatches
- [x] **Complete Dynamic Video Sources Test Infrastructure (PRP-07)**
  - Full source-videos crate with 1,200+ lines of code and 24 tests
  - RTSP server serving multiple concurrent test streams  
  - 25+ test patterns including animated sequences
  - File generation in MP4, MKV, WebM formats
  - CLI application with interactive mode
  - Thread-safe VideoSourceManager with configuration support
- [x] **All PRPs Successfully Implemented (01-07)**
  - Core Infrastructure, Pipeline Management, Source Control APIs
  - DeepStream Integration with metadata extraction
  - Main Application Demo with runtime source management
  - Hardware Abstraction with three-backend system
  - Complete test infrastructure for self-contained testing

### Previous Major Completions  
- [x] DeepStream Metadata Integration (PRP-04) - Full AI inference pipeline
- [x] Main Application Demo (PRP-05) - Production-ready CLI interface
- [x] Source Control APIs (PRP-03) - Dynamic source management
- [x] Pipeline Management (PRP-02) - Robust GStreamer integration  
- [x] Hardware Abstraction (PRP-06) - Cross-platform backend system

## Known Issues 🐛

1. **Critical: Application Shutdown** - NEW
   - Application doesn't respond to Ctrl+C or window close
   - Only displays "Received interrupt signal, shutting down..." repeatedly
   - Status: CRITICAL BUG

2. **Critical: Video Playback Freezing** - NEW
   - Video stuck on first/last frame
   - H264 parser framerate warning
   - Status: CRITICAL BUG

3. **Source Management Tests with Mock Backend**
   - 10 tests fail when using Mock backend
   - Reason: Mock backend doesn't support uridecodebin
   - Status: Expected behavior - use Standard backend for full testing

4. **Build Warnings**
   - Unused imports after linter added `#![allow(unused)]`
   - Non-snake-case field `bInferDone` (kept for DeepStream compatibility)
   - Unused workspace manifest keys

5. **DeepStream Backend** 
   - Not tested on actual NVIDIA hardware
   - Need validation on Jetson and x86 with GPU

6. **Memory Management**
   - Need to verify no leaks, particularly around dynamic source addition/removal

## Statistics 📊

- **Total TODO items**: ~55 active items
- **Critical Bugs**: 2 ACTIVE (shutdown, video freeze) - MUST FIX
- **Code Quality Issues**: 
  - **unwrap() calls**: 100+ occurrences across 27 files in ds-rs/src
  - **TODO comments**: 4 found (3 in Cargo.toml files, 1 in cpu_backend_tests.rs)
  - **todo!() macros**: Not found in current scan
  - **"For now" comments**: 11 occurrences indicating temporary implementations
  - **"Real implementation" comments**: 9+ occurrences indicating stubs
  - **"actual" comments**: 2+ occurrences (test comments, tiling)  
  - **"later/temporary" comments**: Multiple references to incomplete work
  - **Unused parameters**: 40+ underscore-prefixed variables indicating incomplete implementations
  - **Mock backend**: Extensive mock implementation for testing without hardware
- **Test Coverage**: 67 tests total, some fail on Mock backend (expected)
- **Codebase Size**: ~15,000+ lines across all crates
- **Build Status**: ✅ SUCCESS with ort,ndarray features enabled
- **YOLO Support**: v3-v12 + YOLO-RD with automatic version detection
- **PRP Progress**: 11/31 complete (35%), 3/31 in progress (10%), 17/31 not started (55%)

## New Feature Development 🚀

### CPU Vision Backend (PRPs 20-23) - IN PROGRESS
- [x] **Implement CPU-Based Vision Backend (PRP-20)**
  - ✅ Created cpu_vision module structure
  - ✅ Implemented placeholder ONNX detector with mock detection output
  - ✅ Implemented Centroid tracker with trajectory history
  - ✅ Created GStreamer element wrappers for detection/tracking/OSD
  - ✅ Integrated with Standard backend to replace fakesink/identity
  - 🔄 Full ONNX Runtime integration needed (see PRP-21)
  
- [ ] **CPU Object Detection Module (PRP-21)**
  - Integrate YOLOv5 Nano (1.9M parameters)
  - Support MobileNet SSD as backup
  - ONNX model loading and inference
  - Target: 20+ FPS single stream
  
- [ ] **CPU Object Tracking Module (PRP-22)**
  - Implement Centroid tracker (100+ FPS)
  - Add Kalman filter tracker (50+ FPS)
  - Implement SORT algorithm (30+ FPS)
  - Configurable algorithm selection
  
- [ ] **GStreamer Plugin Integration (PRP-23)**
  - Integrate hsvdetector for color-based detection
  - Use colordetect for specific object tracking
  - Leverage videocompare for motion detection
  - Build hybrid detection pipelines

## Notes

### Key Technical Debt
- **CRITICAL BUGS**: Shutdown and video playback issues make app unusable
- **DeepStream FFI Bindings**: Critical metadata and message handling functions need implementation
- **Mock/Placeholder Implementations**: 13+ functions return simplified data marked with "for now" comments
  - `inference/mod.rs:175`: Label map loading
  - `inference/config.rs:228`: DeepStream config parsing
  - `platform.rs:149`: GPU capabilities detection
  - `metadata/mod.rs:61`: Batch metadata extraction
  - `messages/mod.rs:175,182`: Stream EOS handling
  - `pipeline/bus.rs:217`: Stream-specific EOS detection
  - `source-videos/src/manager.rs:215`: modify_source_config is placeholder
  - `source-videos/src/runtime/applicator.rs:72,81,88`: Partial implementation for config changes
  - `backend/cpu_vision/elements.rs:94`: Pass-through implementation in tracker
  - `backend/standard.rs`: Tiler using identity element
- **Unused Parameters**: 40+ underscore-prefixed variables indicate incomplete implementations
- **Test Limitations**: Mock backend cannot test uridecodebin-based functionality

### Priority Focus
- **IMMEDIATE**: Fix shutdown and video playback critical bugs
- **Critical**: DeepStream FFI integration for metadata extraction and stream EOS handling
- **High**: Complete main demo app and config file parsing
- **New**: CPU Vision Backend implementation for non-NVIDIA systems
- **Code Quality**: Replace 100 unwrap() calls for production readiness

### Development Patterns Found
- Mock implementations consistently marked with "for now" comments (13+ occurrences)
- Unused parameters prefixed with underscore (_) to avoid warnings (40+ occurrences)
- Placeholder logic returning simplified or hardcoded values
- Standard backend uses fakesink/identity as placeholders
- Test files marked with "placeholder" or "actual runtime" comments indicate incomplete testing

## Contributing

When working on any TODO item:
1. **CHECK BUGS.md FIRST** for critical issues
2. Create a feature branch
3. Update this TODO.md to mark item as in-progress
4. Write tests for new functionality
5. Update documentation as needed
6. Mark complete in TODO.md when merged

---

**Status: CRITICAL BUGS - Shutdown and video playback issues need immediate attention**

### New PRPs Added (2025-08-24)
- **PRP-27: Multi-Backend Detector Trait Architecture** - Foundation for pluggable detection backends
- **PRP-28: OpenCV DNN Backend** - Leverage OpenCV's optimized CPU inference
- **PRP-29: TensorFlow Lite Backend** - Optimized for mobile/edge devices
- **PRP-30: Darknet Native Backend** - Direct support for original YOLO implementations
- **PRP-31: Advanced Tracking Algorithms** - SORT, Deep SORT, and ByteTrack implementations

### Recent Findings (2025-08-24 Update)
- **CRITICAL**: 2 active bugs make application unusable (shutdown, video freeze)
- 4 TODO comments: 3 in Cargo.toml files (workspace config, tokio usage), 1 in tests about ONNX models
- No FIXME/HACK/XXX/NOTE comments found in active code
- No todo!() macros found in current scan
- 15+ "for now" placeholder implementations across codebase
- Multiple "actual", "real implementation" and "temporary" references indicating incomplete functionality
- 40+ unused parameters (underscore-prefixed) indicating incomplete implementations
- Multiple references to "later", "temporary", "actual" indicating pending work
- Test coverage: 67 tests pass, Mock backend limitations expected with uridecodebin
