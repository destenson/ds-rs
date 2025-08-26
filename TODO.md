# TODO List

Last Updated: 2025-08-25 (Comprehensive Codebase Scan)

## Recent Achievements ✅

### Latest Completions (2025-08-25)
- **COMPLETED**: PRP-44 Fix False Detection Bug - Coordinates fixed, confidence thresholds adjusted!
- **COMPLETED**: Generated PRPs for cpuinfer GStreamer plugin (PRPs 51-53)
- **COMPLETED**: PRP-50 Refactor to Specialized Crates plan created

### Previous Completions (2025-08-25)
- **COMPLETED**: PRP-43 Network Congestion Simulation Enhancement
- **COMPLETED**: PRP-33 CPU OSD Cairo Draw Implementation
- **COMPLETED**: PRP-09 Test Orchestration Scripts
- **COMPLETED**: PRP-08 Code Quality & Production Readiness
- **COMPLETED**: Code Refactoring and Duplication Elimination

## Critical Priority TODOs 🔴

### 1. Remove Global State in Error Classification
**Location**: `crates/ds-rs/src/error/classification.rs:309`
```rust
// TODO: GET RID OF THIS GLOBAL & dependency on lazy_static
```
- **Impact**: Architecture smell, testing difficulties, thread safety issues
- **Priority**: CRITICAL - Global state is an anti-pattern

### 2. cpuinfer GStreamer Plugin Registration
**Locations**: Multiple PRPs created
- **PRP-51**: Fix plugin build and registration
- **PRP-52**: Implement nvinfer-compatible properties
- **PRP-53**: Plugin installation and system integration
- **Impact**: cpuinfer not usable as standard GStreamer element
- **Status**: PRPs ready for implementation

### 3. DeepStream Metadata Processing Implementation
**Locations**: 
- `crates/ds-rs/src/rendering/deepstream_renderer.rs:190` - TODO: Implement actual DeepStream metadata processing
- `crates/ds-rs/src/rendering/deepstream_renderer.rs:222` - TODO: Create and attach actual NvDsObjectMeta
- `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs:186-187` - TODO: Attach custom metadata to buffer
- **Impact**: Critical for hardware acceleration and production deployment
- **Blocked by**: Need DeepStream FFI bindings

## High Priority TODOs 🟠

### 4. Mock Backend Conditional Compilation
**Location**: `crates/ds-rs/src/backend/mock.rs:48`
```rust
// TODO: only include this for testing #[cfg(test)]
```
- **Impact**: Smaller production binaries, clearer test boundaries

### 5. Source-Videos Progressive Loading
**Location**: `crates/source-videos/src/manager.rs:319`
```rust
// TODO: Implement progressive loading
```
**Location**: `crates/source-videos/src/manager.rs:366`
```rust
// TODO: Implement lazy loading
```
- **Impact**: Better performance with large video collections

### 6. Unix Socket Server for Runtime Control
**Location**: `crates/source-videos/src/main.rs:1124`
```rust
// TODO: Implement Unix socket server for runtime control
```
- **Impact**: Better IPC for control operations

### 7. Actual Metadata Extraction
**Location**: `crates/source-videos/src/file_utils.rs:128`
```rust
// TODO: Implement actual metadata extraction using GStreamer discoverer
```
- **Impact**: Cannot extract real video metadata

## Medium Priority TODOs 🟡

### 8. Real ONNX Model Testing
**Locations**: 
- `crates/ds-rs/tests/cpu_backend_tests.rs:336` - TODO: When a real ONNX model is available, test with an actual model
- `crates/ds-rs/tests/cpu_backend_tests.rs:352` - TODO: When a real ONNX model is available, test with...
- **Current state**: Tests verify detector fails without model (correct behavior)
- **Impact**: Better test coverage, validation of real inference

### 9. DSL Implementation
**Location**: `crates/dsl/src/lib.rs:10`
```rust
// TODO: Implement actual DSL tests when DSL functionality is added
```
- **Impact**: DSL crate is placeholder only

### 10. Get Actual Metrics
**Location**: `crates/source-videos/src/main.rs:1331`
```rust
0, // TODO: Get actual metrics
```
- **Impact**: Dashboard shows 0 for active streams metric

## Low Priority TODOs 🔵

### 11. Element Registration Cleanup
**Location**: `crates/ds-rs/src/lib.rs:89`
- Comment about temporary plugin for registering elements
- Should be removed once cpuinfer is proper plugin (PRP-51)

### 12. Various "For now" Temporary Implementations
**30+ locations with temporary code marked "for now":**
- Model loading and configuration parsing
- Metadata extraction and processing  
- Stream message handling
- Detection processing setup
- Frame processing in pipeline pool
- Various placeholder implementations

## Technical Debt 🔧

### Architecture Refactoring
- **PRP-50**: Refactor into 15+ specialized crates
  - Foundation: ds-core, ds-error, ds-platform
  - GStreamer: ds-gstreamer, ds-backend, ds-elements
  - Processing: ds-source, ds-metadata, ds-inference, ds-tracking, ds-health
  - Features: ds-rendering, ds-multistream
  - Application: ds-config, ds-app
- **Impact**: Better modularity, reusability, maintainability

### Code Quality Metrics 📊
- **TODO comments**: 10 explicit TODOs
- **"For now" patterns**: 30+ temporary implementations
- **"actual" references**: 15+ places needing real implementations
- **NOTE comments**: 8 informational notes
- **Test coverage**: 121/121 tests passing ✅

## Next Sprint Focus 🎯

### Week 1: Plugin Infrastructure
1. **Implement PRP-51**: Fix cpuinfer plugin build and registration
2. **Remove global state** in error classification
3. **Add conditional compilation** for mock backend

### Week 2: Plugin Compatibility  
4. **Implement PRP-52**: Add nvinfer-compatible properties to cpuinfer
5. **Implement metadata attachment** in CPU detector
6. **Add configuration file parsing**

### Week 3: Installation & Testing
7. **Implement PRP-53**: Plugin installation scripts
8. **Add real ONNX model** testing
9. **Complete progressive loading** for source-videos

## Production Readiness Assessment 🏭

### ✅ Ready
- Core pipeline management
- Dynamic source management  
- CPU-based detection (after PRP-44 fix)
- Cross-platform backend abstraction
- Ball tracking visualization working

### 🚨 Blockers
1. **Global state** in error handling
2. **cpuinfer not a proper plugin** - can't use with gst-launch
3. **Incomplete metadata flow** - detections not propagated via GStreamer metadata
4. **No DeepStream metadata** implementation

### 📈 Recommendation
Priority order:
1. Complete cpuinfer plugin (PRPs 51-53) - enables standard GStreamer usage
2. Remove global state - architectural cleanup
3. Implement metadata propagation - complete the data flow
4. Refactor to specialized crates (PRP-50) - long-term maintainability

## Development Guidelines 📝

When working on any TODO:
1. Remove the TODO comment when implementing
2. Write tests for the implementation
3. Update this file to reflect completion
4. Check for related TODOs that might be affected
5. Ensure no new global state is introduced
6. Follow patterns from reference implementations in ../gst-plugins-rs
