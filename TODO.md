# TODO List

Last Updated: 2025-01-25 (Post PRP-35 Completion)

## Recent Achievements ✅
- **COMPLETED**: PRP-35 Directory and File List Support (2025-01-25)
  - ✅ DirectoryScanner with recursive traversal
  - ✅ FileVideoSource with uridecodebin auto-detection
  - ✅ Filter system for include/exclude patterns  
  - ✅ Mount point generation from file paths
  - ✅ CLI support for directory and file list serving
  - ✅ 13/13 directory serving tests passing
  - ✅ Examples: directory_server.rs and batch_file_server.rs

- **COMPLETED**: PRP-12 Multi-stream Detection Pipeline (2025-01-25)
  - ✅ MultiStreamManager with fault tolerance integration
  - ✅ Pipeline pool for concurrent processing
  - ✅ Stream coordinator with priority scheduling
  - ✅ Resource manager with CPU/memory monitoring
  - ✅ Comprehensive metrics collection
  - ✅ 12/12 multistream tests passing

- **COMPLETED**: Enhanced Error Recovery (PRP-34)
- **COMPLETED**: Network Simulation (PRP-19)
- **COMPLETED**: Source Management Fixes (PRP-33)

## Critical Priority TODOs 🔴

### 1. Remove Global State in Error Classification
**Location**: `src/error/classification.rs:309`
- GET RID OF THIS GLOBAL & dependency on lazy_static
- Replace with proper dependency injection
- **Impact**: Architecture smell, testing difficulties

### 2. DeepStream Metadata Processing
**Location**: `src/rendering/deepstream_renderer.rs:190,222`
- Implement actual DeepStream metadata processing
- Create and attach actual NvDsObjectMeta
- **Impact**: Critical for hardware acceleration features
- **Blocked by**: Need DeepStream FFI bindings (PRP-04)

### 3. Fix Unimplemented Property Handlers
**Locations**: 
- `cpuinfer/src/cpudetector/imp.rs:258,272`
- `src/backend/cpu_vision/cpudetector/imp.rs:274,288`
- Complete property getter/setter implementations
- **Impact**: Runtime errors when properties accessed

## High Priority TODOs 🟠

### 4. Source-Videos CLI Expansion
**PRPs Created**: 35-40
- [x] **PRP-35**: Directory and file list support - **COMPLETED**
- [ ] **PRP-36**: File watching and auto-reload
- [ ] **PRP-37**: Enhanced configuration system
- [ ] **PRP-38**: Advanced CLI options
- [ ] **PRP-39**: REPL mode enhancements
- [ ] **PRP-40**: Network simulation integration
- **Impact**: Major feature expansion for testing infrastructure

### 5. Float16 Model Support
**Issue**: YOLO f16 models fail due to lifetime issues
- Workaround exists (use f32 models)
- **Location**: ONNX integration
- **PRP**: PRP-02 planned

### 6. DeepStream FFI Bindings
**PRP**: PRP-04
- Extract NvDsMeta from hardware inference
- Enable hardware acceleration features
- **Impact**: Full DeepStream capabilities

## Medium Priority TODOs 🟡

### 7. Remove Tokio Dependency
**Locations**:
- `Cargo.toml:53` - ds-rs crate
- `source-videos/Cargo.toml:20`
- Comment: "we should not use tokio (async is ok though)"
- **Impact**: Reduce dependencies, simpler runtime

### 8. Mock Backend Conditional Compilation
**Location**: `src/backend/mock.rs:48`
- Only include mock backend for testing with #[cfg(test)]
- **Impact**: Smaller production binaries

### 9. Real Metadata Extraction
**Location**: `src/metadata/mod.rs:92`
- Replace todo!() with actual metadata extraction logic
- Currently returns mock data
- **Impact**: Production readiness

### 10. Custom Metadata on Buffers
**Location**: `src/backend/cpu_vision/cpudetector/imp.rs:154`
- Attach detection metadata to GStreamer buffers
- **Impact**: Better pipeline integration

### 11. Progressive Loading Implementation
**Location**: `source-videos/src/manager.rs:306`
- Implement progressive loading for large directories
- Currently placeholder comment
- **Impact**: Performance with large video catalogs

### 12. Lazy Loading Implementation  
**Location**: `source-videos/src/manager.rs:353`
- Implement lazy loading for better memory usage
- Currently placeholder comment
- **Impact**: Memory efficiency

## Low Priority TODOs 🔵

### 13. DSL Crate Implementation
**Location**: `dsl/src/lib.rs:9`
- DeepStream Services Library implementation
- Single todo!() in test
- **Impact**: High-level API

### 14. Test with Real ONNX Model
**Location**: `tests/cpu_backend_tests.rs:343`
- When real ONNX model available, add proper tests
- **Impact**: Test coverage

### 15. Directory Scanning Optimization
**Location**: `source-videos/src/directory.rs:63`
- Replace synchronous scanning with async implementation
- Currently "for now" comment
- **Impact**: Performance for large directories

### 16. Export/Streaming Integration
**PRP**: PRP-13
- MQTT/Kafka integration for detection results
- **Impact**: Production deployment features

### 17. Control API
**PRP**: PRP-17
- WebSocket/REST interface for remote control
- **Impact**: Production monitoring/control

## Technical Debt 🔧

### Code Quality Issues
- **TODO/FIXME comments**: 9 active items in source code (excluding git logs)
- **"For now" comments**: 9 occurrences indicating temporary solutions  
- **Placeholder implementations**: 4 critical unimplemented property handlers
- **Metadata stubs**: 2 locations with actual processing needed
- **Tokio usage**: 2 locations marked for removal per architecture decisions

### Test Failures
- **CPU Detector Tests**: 2 failures due to missing ONNX model
- **Ball Tracking Example**: Compilation errors from API mismatches

## Project Statistics 📊

### Test Coverage
- **Overall**: 207/209 tests passing (99% pass rate)
- **ds-rs**: 124/126 passing (98.4%)
- **source-videos**: 83/83 passing (100%)
- **multistream**: 12/12 passing (100%) - NEW!

### Implementation Status
- **PRPs Completed**: 20/40 (including PRP-35)
- **Working Examples**: 7/8 (including directory/file serving)
- **Crates Building**: 4/4

## Next Sprint Focus 🎯

1. **Immediate**: Start PRP-36 (File Watching and Auto-reload)
2. **Week 1-2**: File system monitoring with inotify/FSEvents (PRP-36)
3. **Week 3-4**: Enhanced config system + CLI improvements (PRP-37, 38) 
4. **Week 5-6**: REPL mode + network sim integration (PRP-39, 40)

## Development Guidelines 📝

When working on any TODO item:
1. Check for existing PRP documentation
2. Update this TODO.md to mark item as in-progress
3. Write tests before implementation
4. Update documentation
5. Mark complete in TODO.md when merged

## Notes

- Playlist functionality marked as lower priority per user feedback
- Focus on core file serving and testing infrastructure
- Multi-stream implementation provides foundation for advanced features
- Network simulation framework ready for integration with source-videos
