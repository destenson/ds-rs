# TODO List

Last Updated: 2025-08-25 (Comprehensive Codebase Scan)

## Recent Achievements ✅

### Latest Completions (2025-08-25)
- **COMPLETED**: PRP-33 CPU OSD Cairo Draw Implementation - Ball tracking visualization now working!
- **COMPLETED**: PRP-09 Test Orchestration Scripts
- **COMPLETED**: PRP-08 Code Quality & Production Readiness
- **COMPLETED**: Code Refactoring and Duplication Elimination
- **COMPLETED**: Critical Runtime Panic Fixes
- **COMPLETED**: PRP-02 Float16 Model Support
- **COMPLETED**: PRP-38 Advanced CLI Options for Source-Videos
- **COMPLETED**: PRP-39 Enhanced REPL Mode
- **COMPLETED**: PRP-41 Source-Videos Control API for Automation

## Critical Priority TODOs 🔴

### 1. Excessive unwrap() Usage
**Status**: CRITICAL - 753 unwrap() calls across 86 files
- **Impact**: Any call could cause production panic
- **Priority**: Must address before production deployment
- **Target**: Replace 200 critical unwrap() calls per week

### 2. Remove Global State in Error Classification
**Location**: `crates/ds-rs/src/error/classification.rs:309`
- GET RID OF THIS GLOBAL & dependency on lazy_static
- **Impact**: Architecture smell, testing difficulties

### 3. DSL Crate Implementation
**Location**: `crates/dsl/src/lib.rs:10`
- TODO: Implement actual DSL tests when DSL functionality is added
- Currently just a placeholder test
- **Impact**: Missing core functionality

## High Priority TODOs 🟠

### 4. YOLOv11 Format Support
**Location**: `crates/cpuinfer/src/detector.rs:390`
- Currently treating v11 same as v8 with logging
- Comment: "For now, treat similar to v8 but log the difference"
- **Impact**: May not fully support latest YOLO models

### 5. DeepStream Metadata Processing
**Locations**: 
- `crates/ds-rs/src/rendering/deepstream_renderer.rs:190` - TODO: Implement actual DeepStream metadata processing
- `crates/ds-rs/src/rendering/deepstream_renderer.rs:222` - TODO: Create and attach actual NvDsObjectMeta
- **Impact**: Critical for hardware acceleration
- **Blocked by**: Need DeepStream FFI bindings (PRP-04)

### 6. Custom Metadata Attachment
**Location**: `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs:174`
- TODO: Attach custom metadata to buffer
- Currently just logging detections
- **Impact**: Metadata flow incomplete

### 7. Remove Tokio Dependency
**Locations**:
- `crates/ds-rs/Cargo.toml:56` - TODO comment
- `crates/source-videos/Cargo.toml:33` - TODO comment
- Comment: "we should not use tokio (async is ok though)"
- **Impact**: Reduce dependencies, simpler runtime

## Medium Priority TODOs 🟡

### 8. Mock Backend Conditional Compilation
**Location**: `crates/ds-rs/src/backend/mock.rs:48`
- TODO: only include this for testing #[cfg(test)]
- **Impact**: Smaller production binaries

### 9. Progressive/Lazy Loading
**Locations**:
- `crates/source-videos/src/manager.rs:319` - TODO: Implement progressive loading
- `crates/source-videos/src/manager.rs:366` - TODO: Implement lazy loading
- **Impact**: Performance with large video catalogs

### 10. Real ONNX Model Testing
**Location**: `crates/ds-rs/tests/cpu_backend_tests.rs:343`
- TODO: When a real ONNX model is available, test with proper model
- **Impact**: Better test coverage

### 11. Placeholder Implementations ("for now" patterns)
**Key locations with temporary implementations**:
- `crates/cpuinfer/src/detector.rs:390` - YOLOv11 treated as v8
- `crates/source-videos/src/api/auth.rs:91` - Skipping auth check
- `crates/source-videos/src/directory.rs:63` - Synchronous scanning instead of async
- `crates/source-videos/src/main.rs:1481` - Simple console detach on Windows
- `crates/source-videos/src/main.rs:1666` - Skipping duration filtering
- `crates/source-videos/src/manager.rs:318` - Adding all at once instead of progressive
- `crates/source-videos/src/runtime/applicator.rs:72,81,88` - Logging changes instead of applying
- Multiple API route handlers returning placeholder responses

## Low Priority TODOs 🔵

### 12. Source Config Modification
**Location**: `crates/source-videos/src/manager.rs:224`
- Function `modify_source_config` has placeholder implementation
- Comment: "For now, this is a placeholder for future enhancement"
- **Impact**: Limited runtime configuration changes

### 13. Unix Socket Server
**Location**: `crates/source-videos/src/main.rs:1072`
- TODO: Implement Unix socket server for runtime control
- **Impact**: Better operational control

### 14. Actual Metrics Collection
**Location**: `crates/source-videos/src/main.rs:1279`
- TODO: Get actual metrics (currently returns 0)
- **Impact**: Monitoring capabilities

### 15. GStreamer Metadata Extraction
**Location**: `crates/source-videos/src/file_utils.rs:128`
- TODO: Implement actual metadata extraction using GStreamer discoverer
- **Impact**: Better file information

### 16. REPL Placeholder Commands
**Location**: `crates/source-videos/src/repl/commands.rs:502-520`
- Multiple commands using placeholder_command! macro
- Includes modify command and others
- **Impact**: Limited REPL functionality

## Technical Debt 🔧

### Code Quality Metrics
- **unwrap() Usage**: 753 occurrences (critical production risk)
- **unimplemented!()**: ✅ 0 in property handlers (all fixed)
- **todo!()**: 0 active runtime panics (all fixed)
- **TODO/FIXME comments**: 13 requiring implementation
- **"for now" patterns**: 30+ temporary implementations
- **Placeholder/stub implementations**: 10+ locations

### Unused Variables (_prefix)
Multiple locations with unused parameters indicating incomplete implementations:
- Signal handlers: `_bus`, `_src`, `_pad`, `_info` parameters
- Property handlers: `_id` in set_property/property methods
- Test variables: `_pattern`, `_source_id`, `_bin`, etc.
- Config watcher: `_watcher` field in ConfigWatcher struct

## Project Statistics 📊

### Implementation Status
- **PRPs Created**: 33 total
- **PRPs Completed**: 30/33 (90.9% completion)
- **Crates Building**: 4/4 ✅
- **Tests Passing**: 
  - ds-rs: 121/121 ✅
  - cpuinfer: 8/10 (80%)
  - source-videos: 81/82 (98.8%)

### Current Capabilities
✅ **Working Features**:
- Dynamic source management
- RTSP streaming server
- CPU-based object detection with visualization
- Network simulation
- REST API for automation
- Advanced CLI modes
- Enhanced REPL
- Ball tracking with bounding boxes

⚠️ **Incomplete Features**:
- DeepStream hardware acceleration (needs FFI)
- Metadata flow incomplete
- DSL crate not implemented
- Some REPL commands are placeholders

## Next Sprint Focus 🎯

### Week 1-2: Critical Fixes
1. **URGENT**: Start unwrap() replacement campaign (target 150 calls)
2. **HIGH**: Remove global state in error classification
3. **HIGH**: Complete metadata attachment in CPU detector

### Week 3-4: Core Improvements
4. **MEDIUM**: Implement DSL crate functionality
5. **MEDIUM**: Full YOLOv11 format support
6. **MEDIUM**: Implement progressive/lazy loading
7. **MEDIUM**: Remove tokio dependency

### Week 5-6: Feature Completion
8. **LOW**: Complete REPL placeholder commands
9. **LOW**: Unix socket server implementation
10. **LOW**: Actual metrics collection
11. **LOW**: GStreamer metadata extraction

## Production Readiness Assessment 🏭

### ✅ Ready
- Core streaming functionality
- Error recovery
- API automation
- CLI interface
- CPU object detection with visualization

### 🚨 Blockers
1. **753 unwrap() calls** - Panic risk
2. **Incomplete metadata flow** - Detection data not propagated
3. **30+ placeholder implementations** - Temporary code
4. **DSL crate not implemented** - Missing core functionality

### 📈 Recommendation
**Priority Order**:
1. Replace unwrap() calls - Production stability
2. Complete metadata flow - Core functionality
3. Implement DSL crate - Core feature
4. Remove placeholders - Code quality

The codebase has strong foundations with ball tracking visualization now working. Focus should shift to production stability (unwrap() replacement) and completing core features (metadata flow, DSL crate).

## Development Guidelines 📝

When working on any TODO:
1. Check for existing PRP documentation
2. Update TODO.md marking as in-progress
3. Write tests before implementation
4. Update documentation
5. Mark complete when merged

## Notes

- Ball tracking visualization fixed with PRP-33! ✅
- Metadata flow completion would enable many downstream features
- Consider creating batch PRP for unwrap() replacement campaign
- Most "for now" implementations are in non-critical paths
- DSL crate needs design and implementation plan