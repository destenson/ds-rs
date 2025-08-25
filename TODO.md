# TODO List

Last Updated: 2025-08-25 (Post-PRP-33 Ball Tracking Visualization Fix)

## Recent Achievements ✅

### Latest Fix: Ball Tracking Visualization (2025-08-25)
- **IDENTIFIED**: Root cause of missing bounding boxes in ball tracking
- **PRP-33 CREATED**: CPU OSD Bounding Box Implementation plan
- **Issue**: CPU detector emits signals but Cairo draw callback not implemented
- **Solution Path**: Connect cairooverlay draw signal to render detection boxes

### Previous Completions (2025-08-25)
- **COMPLETED**: PRP-09 Test Orchestration Scripts
- **COMPLETED**: PRP-08 Code Quality & Production Readiness
- **COMPLETED**: Code Refactoring and Duplication Elimination
- **COMPLETED**: Critical Runtime Panic Fixes
- **COMPLETED**: PRP-02 Float16 Model Support
- **COMPLETED**: PRP-38 Advanced CLI Options for Source-Videos
- **COMPLETED**: PRP-39 Enhanced REPL Mode
- **COMPLETED**: PRP-41 Source-Videos Control API for Automation

## Critical Priority TODOs 🔴

### 1. CPU OSD Cairo Draw Implementation - NEW
**Status**: CRITICAL - Ball tracking visualization broken
**Location**: `crates/ds-rs/src/backend/cpu_vision/elements.rs:314`
- Comment: "In a real implementation, we would set up the draw signal"
- **Impact**: No bounding boxes displayed despite successful detection
- **PRP**: PRP-33 created with implementation plan
- **Fix**: Connect cairooverlay draw signal, implement Cairo drawing callback

### 2. Excessive unwrap() Usage
**Status**: CRITICAL - 753 unwrap() calls across 86 files
- **Impact**: Any call could cause production panic
- **Priority**: Must address before production deployment
- **Target**: Replace 200 critical unwrap() calls per week

### 3. Remove Global State in Error Classification
**Location**: `crates/ds-rs/src/error/classification.rs:309`
- GET RID OF THIS GLOBAL & dependency on lazy_static
- **Impact**: Architecture smell, testing difficulties

## High Priority TODOs 🟠

### 4. DeepStream Metadata Processing
**Locations**: 
- `crates/ds-rs/src/rendering/deepstream_renderer.rs:190` - TODO: Implement actual DeepStream metadata processing
- `crates/ds-rs/src/rendering/deepstream_renderer.rs:222` - TODO: Create and attach actual NvDsObjectMeta
- **Impact**: Critical for hardware acceleration
- **Blocked by**: Need DeepStream FFI bindings (PRP-04)

### 5. Custom Metadata Attachment
**Location**: `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs:174`
- TODO: Attach custom metadata to buffer
- Currently just logging detections
- **Impact**: Metadata flow incomplete

### 6. Remove Tokio Dependency
**Locations**:
- `crates/ds-rs/Cargo.toml:56` - TODO comment
- `crates/source-videos/Cargo.toml:33` - TODO comment
- Comment: "we should not use tokio (async is ok though)"
- **Impact**: Reduce dependencies, simpler runtime

## Medium Priority TODOs 🟡

### 7. Mock Backend Conditional Compilation
**Location**: `crates/ds-rs/src/backend/mock.rs:48`
- TODO: only include this for testing #[cfg(test)]
- **Impact**: Smaller production binaries

### 8. Progressive/Lazy Loading
**Locations**:
- `crates/source-videos/src/manager.rs:319` - TODO: Implement progressive loading
- `crates/source-videos/src/manager.rs:366` - TODO: Implement lazy loading
- **Impact**: Performance with large video catalogs

### 9. Real ONNX Model Testing
**Location**: `crates/ds-rs/tests/cpu_backend_tests.rs:343`
- TODO: When a real ONNX model is available, test with proper model
- **Impact**: Better test coverage

### 10. Placeholder Implementations ("for now" patterns)
**Key locations with temporary implementations**:
- `crates/ds-rs/src/backend/cpu_vision/elements.rs:109` - Logging instead of metadata attachment
- `crates/ds-rs/src/rendering/standard_renderer.rs:104` - Skipping signal connection
- `crates/ds-rs/src/inference/mod.rs:175` - Default label map
- `crates/ds-rs/src/platform.rs:149` - Common capabilities instead of detection
- `crates/source-videos/src/file_utils.rs:128` - TODO: Actual metadata extraction

## Low Priority TODOs 🔵

### 11. Unix Socket Server
**Location**: `crates/source-videos/src/main.rs:1072`
- TODO: Implement Unix socket server for runtime control
- **Impact**: Better operational control

### 12. Actual Metrics Collection
**Location**: `crates/source-videos/src/main.rs:1279`
- TODO: Get actual metrics (currently returns 0)
- **Impact**: Monitoring capabilities

### 13. GStreamer Metadata Extraction
**Location**: `crates/source-videos/src/file_utils.rs:128`
- TODO: Implement actual metadata extraction using GStreamer discoverer
- **Impact**: Better file information

## Technical Debt 🔧

### Code Quality Metrics
- **unwrap() Usage**: 753 occurrences (critical production risk)
- **unimplemented!()**: ✅ 0 in property handlers (all fixed)
- **todo!()**: 0 active runtime panics (all fixed)
- **TODO comments**: 11 requiring implementation
- **"for now" patterns**: 25+ temporary implementations

### Unused Variables (_prefix)
Multiple locations with unused parameters indicating incomplete implementations:
- Signal handlers with unused parameters
- Callback functions with placeholders
- Mock implementations

## Project Statistics 📊

### Implementation Status
- **PRPs Created**: 33 total (latest: PRP-33 for ball tracking)
- **PRPs Completed**: 29/33 (87.9% completion)
- **Crates Building**: 4/4 ✅
- **Tests Passing**: 
  - ds-rs: 121/121 ✅
  - cpuinfer: 8/10 (80%)
  - source-videos: 81/82 (98.8%)

### Current Capabilities
✅ **Working Features**:
- Dynamic source management
- RTSP streaming server
- CPU-based object detection
- Network simulation
- REST API for automation
- Advanced CLI modes
- Enhanced REPL

⚠️ **Broken Features**:
- Ball tracking visualization (no bounding boxes)
- DeepStream hardware acceleration (needs FFI)
- Metadata flow incomplete

## Next Sprint Focus 🎯

### Week 1-2: Critical Fixes
1. **URGENT**: Implement PRP-33 (CPU OSD Cairo drawing)
2. **HIGH**: Start unwrap() replacement (target 150 calls)
3. **HIGH**: Complete metadata attachment in CPU detector

### Week 3-4: Core Improvements
4. **MEDIUM**: Remove global state in error classification
5. **MEDIUM**: Implement progressive/lazy loading
6. **MEDIUM**: Remove tokio dependency

### Week 5-6: Feature Completion
7. **LOW**: Unix socket server implementation
8. **LOW**: Actual metrics collection
9. **LOW**: GStreamer metadata extraction

## Production Readiness Assessment 🏭

### ✅ Ready
- Core streaming functionality
- Error recovery
- API automation
- CLI interface

### 🚨 Blockers
1. **Ball tracking visualization broken** - PRP-33 needed
2. **753 unwrap() calls** - Panic risk
3. **Incomplete metadata flow** - Detection data not propagated
4. **25+ placeholder implementations** - Temporary code

### 📈 Recommendation
**Priority Order**:
1. Fix ball tracking (PRP-33) - User-visible issue
2. Complete metadata flow - Core functionality
3. Replace unwrap() calls - Production stability
4. Remove placeholders - Code quality

The codebase has strong foundations but needs the Cairo drawing implementation for ball tracking and systematic error handling improvements before production deployment.

## Development Guidelines 📝

When working on any TODO:
1. Check for existing PRP documentation
2. Update TODO.md marking as in-progress
3. Write tests before implementation
4. Update documentation
5. Mark complete when merged

## Notes

- Ball tracking fix (PRP-33) is highest priority - user-facing feature broken
- Metadata flow completion would enable many downstream features
- Consider creating batch PRP for unwrap() replacement campaign
- Most "for now" implementations are in non-critical paths