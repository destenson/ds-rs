# TODO List

Last Updated: 2025-08-25 (Comprehensive Codebase Scan)

## Recent Achievements ✅

### Latest Completions (2025-08-25)
- **COMPLETED**: PRP-43 Network Congestion Simulation Enhancement - Full netsim integration!
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

### 1. False Detection Bug in CPU Detector
**Location**: `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs`
**Evidence**: BUGS.md lines 1-150
- **CRITICAL BUG**: Detector producing 324 false detections per frame
- **Symptoms**: 
  - All detections at position (0.0, 0.0) 
  - Extremely high confidence scores (404814.00)
  - Invalid object classes (dog, kite, broccoli, cell phone in ball tracking)
  - Tiny or invalid bounding box sizes
- **Impact**: Makes tracking completely unusable, erratic behavior
- **Root cause**: Likely improper YOLO post-processing or confidence threshold

### 2. Remove Global State in Error Classification
**Location**: `crates/ds-rs/src/error/classification.rs:309`
- TODO: GET RID OF THIS GLOBAL & dependency on lazy_static
- **Impact**: Architecture smell, testing difficulties, thread safety issues
- **Priority**: CRITICAL - Global state is an anti-pattern

### 3. DeepStream Metadata Processing Implementation
**Locations**: 
- `crates/ds-rs/src/rendering/deepstream_renderer.rs:190` - TODO: Implement actual DeepStream metadata processing
- `crates/ds-rs/src/rendering/deepstream_renderer.rs:222` - TODO: Create and attach actual NvDsObjectMeta
- **Impact**: Critical for hardware acceleration and production deployment
- **Blocked by**: Need DeepStream FFI bindings (PRP-04)

### 4. Custom Metadata Attachment in CPU Detector
**Location**: `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs:187`
- TODO: Attach custom metadata to buffer
- Currently just logging detections without propagating metadata
- **Impact**: Metadata flow incomplete, downstream processing cannot access detection results

## High Priority TODOs 🟠

### 5. Remove Tokio Dependency
**Location**: `crates/ds-rs/Cargo.toml:56`
- TODO: we should not use tokio (async is ok though)
- **Impact**: Reduce dependencies, simpler runtime model
- **Note**: Async/await is acceptable, but full tokio runtime is unnecessary

### 6. Mock Backend Conditional Compilation
**Location**: `crates/ds-rs/src/backend/mock.rs:48`
- TODO: only include this for testing #[cfg(test)]
- **Impact**: Smaller production binaries, clearer test boundaries

### 7. Real ONNX Model Testing
**Location**: `crates/ds-rs/tests/cpu_backend_tests.rs:343`
- TODO: When a real ONNX model is available, test with actual model
- Currently using mock mode for testing
- **Impact**: Better test coverage, validation of real inference

## Medium Priority TODOs 🟡

### 7. Load Model Metadata from File
**Location**: `crates/ds-rs/src/inference/mod.rs:173`
- Function `load_from_file` returns default map with comment "For now, return a default map"
- **Impact**: Cannot load actual model metadata configurations

### 8. Parse DeepStream Config Files
**Location**: `crates/ds-rs/src/inference/config.rs:226`
- Function `from_deepstream_config` returns mock configuration with comment "For now, return a mock configuration"
- **Impact**: Cannot use existing DeepStream configuration files

### 9. Metadata Extraction Implementation
**Location**: `crates/ds-rs/src/metadata/mod.rs:61,94`
- Currently creating mock metadata for testing
- Returns error message "For now, return a clear error message"
- **Impact**: No real metadata extraction from buffers

### 10. Stream ID from Messages
**Location**: `crates/ds-rs/src/messages/mod.rs:182`
- Returns mock stream ID with comment "For now, return mock stream ID"
- **Impact**: Cannot track individual streams in multi-stream pipelines

## Low Priority TODOs 🔵

### 11. Setup Detection Processing
**Location**: `crates/ds-rs/src/multistream/manager.rs:219,230`
- Function `setup_detection_processing` not implemented
- Currently simulating processing with comment "For now, simulate processing"
- **Impact**: Detection pipeline setup incomplete

### 12. Process Frame Implementation
**Location**: `crates/ds-rs/src/multistream/pipeline_pool.rs:52`
- Function `process_frame` not implemented
- **Impact**: Cannot process individual frames in pipeline pool

### 13. Cairo Rendering Placeholders
**Location**: `crates/ds-rs/src/rendering/standard_renderer.rs:249,267,280`
- Multiple functions with placeholder cairo::Context
- Functions not implemented: draw_detection_box, draw_rounded_rectangle, draw_text
- **Impact**: Limited rendering capabilities without Cairo

## Technical Debt 🔧

### Unused Parameters (_prefix)
Multiple locations with unused parameters indicating incomplete implementations:
- **Event Handlers**: 27 instances
  - `crates/ds-rs/src/source/controller.rs:154` - _event_handler
  - `crates/ds-rs/src/source/video_source.rs:321,333,486` - _info, _pad, _decodebin
  - `crates/ds-rs/src/source/synchronization.rs:19,126,137` - _pipeline, _pad, _info
  - `crates/ds-rs/src/metadata/mod.rs:136,138` - _pad, _batch_meta
  - `crates/ds-rs/src/messages/mod.rs:193,317` - _element_msg, _stream_id
  - `crates/ds-rs/src/pipeline/state.rs:196,291,292` - _pending, _pending, _success
  - `crates/ds-rs/src/pipeline/bus.rs:46,214,223,325,378` - _bus, _msg handlers
  - `crates/ds-rs/src/rendering/*.rs` - Multiple _buffer, _pad, _timestamp instances
  - `crates/ds-rs/src/backend/cpu_vision/elements.rs:97,230,231,232` - _tracker_clone, _pad, _buffer
  - `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs:186,277` - _buf, _id

### "For now" Temporary Implementations
**30+ locations with temporary code:**
- Model loading and configuration parsing
- Metadata extraction and processing
- Stream message handling
- Detection processing setup
- Frame processing in pipeline pool
- Cairo rendering functions

## Code Quality Metrics 📊

### Current State
- **TODO comments**: 10 explicit TODOs requiring implementation
- **FIXME comments**: 0 active
- **NOTE comments**: 3 informational notes
- **Unused parameters**: 50+ indicating incomplete implementations
- **"For now" patterns**: 30+ temporary implementations
- **Placeholder implementations**: Multiple API endpoints and functions

### Test Coverage
- `ds-rs`: 121/121 tests passing ✅
- Some tests using mock mode instead of real implementations
- Need real ONNX model for comprehensive testing

## Next Sprint Focus 🎯

### Week 1: Critical Fixes
1. **Remove global state** in error classification (lazy_static removal)
2. **Implement metadata attachment** in CPU detector
3. **Add conditional compilation** for mock backend

### Week 2: Core Functionality
4. **Implement model loading** from files
5. **Parse DeepStream configs** properly
6. **Complete metadata extraction** from buffers

### Week 3: Integration & Testing
7. **Add real ONNX model** testing
8. **Complete detection processing** setup
9. **Implement frame processing** in pipeline pool

## Production Readiness Assessment 🏭

### ✅ Ready
- Core pipeline management
- Dynamic source management
- CPU-based detection with visualization
- Cross-platform backend abstraction

### 🚨 Blockers
1. **Global state** in error handling
2. **Incomplete metadata flow** - detections not propagated
3. **Mock implementations** for critical functions
4. **No real model loading** capability

### 📈 Recommendation
Focus on removing global state and completing the metadata flow. These are the primary blockers for production deployment. The temporary implementations can be addressed incrementally after core functionality is solid.

## Development Guidelines 📝

When working on any TODO:
1. Remove the TODO comment when implementing
2. Write tests for the implementation
3. Update this file to reflect completion
4. Check for related TODOs that might be affected
5. Ensure no new global state is introduced
