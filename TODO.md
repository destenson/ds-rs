# TODO

Last Updated: 2025-08-27 (Post cpuinfer completion scan)

## Recent Achievements ✅

### Latest Completions (2025-08-27)
- **COMPLETED**: PRP-54 CpuInfer Architecture Decision - Dual-Use Crate
  - ✅ Fixed critical build failure (14 compilation errors resolved)
  - ✅ Implemented dual-use crate approach (cdylib + rlib)
  - ✅ Added cpuinfer as dependency to ds-rs Cargo.toml
  - ✅ Library name is `gstcpuinfer` (package name is `cpuinfer`)
  - ✅ All imports working correctly with gstcpuinfer:: prefix
  - ✅ Build now compiles successfully (cargo check passes)
  - ✅ 119 of 121 tests passing (2 failures are GStreamer plugin-related)
  - ✅ cpuinfer works as both plugin and library dependency
- **COMPLETED**: PRPs 51-53 for cpuinfer GStreamer plugin
  - ✅ Plugin now discovered by gst-inspect-1.0 as "cpuinfer"
  - ✅ Fixed plugin registration and removed duplicate from ds-rs
  - ✅ Added nvinfer-compatible properties (all visible in gst-inspect)
  - ✅ Created INI config file parser
  - ✅ Installation scripts for Windows/Linux
  - ✅ Properties/caps properly showing in gst-inspect
  - ✅ Implemented transform_caps() for proper caps negotiation
  - ✅ Element renamed from "cpudetector" to "cpuinfer" for nvinfer compatibility
  - ✅ Fixed transform_ip_passthrough for proper passthrough mode operation
  - ✅ Plugin runs successfully in GStreamer pipelines without crashing
- **COMPLETED**: Enhanced PRP-50 with dependency reduction focus (463 → <50 deps goal)
- **COMPLETED**: Created Debtmap workflow for code quality analysis
- **COMPLETED**: Added .debtmap.yml configuration with Rust-specific rules

### Previous Completions
- **COMPLETED**: PRP-44 Fix False Detection Bug - Coordinates fixed, confidence thresholds adjusted!
- **COMPLETED**: Generated PRPs for cpuinfer GStreamer plugin (PRPs 51-53)
- **COMPLETED**: PRP-50 Refactor to Specialized Crates plan created
- **COMPLETED**: PRP-43 Network Congestion Simulation Enhancement
- **COMPLETED**: PRP-33 CPU OSD Cairo Draw Implementation

## Critical Priority 🔴

### 1. Remove Global State & lazy_static Dependency
**Location**: `crates/ds-rs/src/error/classification.rs:309`
```rust
// TODO: GET RID OF THIS GLOBAL & dependency on lazy_static
```
- **Impact**: Architecture smell, testing difficulties, thread safety issues
- **Solution**: Use dependency injection or context-based error classification


### 2. Replace Tokio Dependency
**Locations**: 
- `crates/ds-rs/Cargo.toml:55`
- `crates/source-videos/Cargo.toml:33`
```toml
tokio = { version = "1.47.1", features = ["full"] } # TODO: we should not use tokio (async is ok though)
```
- **Impact**: Heavy dependency (~200 deps), slower builds
- **Solution**: Consider `smol` or remove async where not needed

### 3. Remove .unwrap() Usage
**Multiple locations** - Search with: `grep -r "\.unwrap()" crates/`
- **Impact**: Can cause panics in production
- **Solution**: Replace with proper error handling using `?` or `expect()` with meaningful messages
- **Note**: Debtmap rule configured to catch these

## High Priority 🟠

### 4. Mock Backend Conditional Compilation
**Location**: `crates/ds-rs/src/backend/mock.rs:48`
```rust
// TODO: only include this for testing #[cfg(test)]
```
- **Impact**: Unnecessary code in production builds
- **Solution**: Add `#[cfg(test)]` attribute

### 5. DeepStream Metadata Processing
**Locations**: 
- `crates/ds-rs/src/rendering/deepstream_renderer.rs:190,222`
- `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs:186`
```rust
// TODO: Implement actual DeepStream metadata processing
// TODO: Create and attach actual NvDsObjectMeta
// TODO: Attach custom metadata to buffer
```
- **Impact**: Cannot use hardware acceleration properly
- **Blocked by**: Need DeepStream SDK FFI bindings

### 6. Real ONNX Model Testing
**Locations**: 
- `crates/ds-rs/tests/cpu_backend_tests.rs:336,352`
```rust
// TODO: When a real ONNX model is available, test with an actual model
```
- **Impact**: Limited test coverage for inference
- **Solution**: Add test models to repository

## Medium Priority 🟡

### 7. Source Videos Features

#### Progressive Loading
**Location**: `crates/source-videos/src/manager.rs:319`
```rust
// TODO: Implement progressive loading
```

#### Lazy Loading
**Location**: `crates/source-videos/src/manager.rs:366`
```rust
// TODO: Implement lazy loading
```

#### Unix Socket Control
**Location**: `crates/source-videos/src/main.rs:1124`
```rust
// TODO: Implement Unix socket server for runtime control
```

#### Actual Metrics
**Location**: `crates/source-videos/src/main.rs:1331`
```rust
0, // TODO: Get actual metrics
```

### 8. Video Metadata Extraction
**Location**: `crates/source-videos/src/file_utils.rs:128`
```rust
// TODO: Implement actual metadata extraction using GStreamer discoverer
```
- **Impact**: Returns placeholder metadata
- **Solution**: Use GStreamer discoverer API

### 9. DSL Implementation
**Location**: `crates/dsl/src/lib.rs:10`
```rust
// TODO: Implement actual DSL tests when DSL functionality is added
```
- **Impact**: DSL crate is placeholder only

## Low Priority 🔵

### 10. Unused Parameters Cleanup
**Multiple locations with `_` prefixed parameters**:
- Probe callbacks: `_pad`, `_info`, `_bus`
- Config loaders: `_path` parameters
- Simulation functions: `_pipeline`
- Processing functions in multistream

Notable files:
- `crates/ds-rs/src/source/video_source.rs`
- `crates/ds-rs/examples/fault_tolerant_pipeline.rs`
- `crates/ds-rs/src/multistream/`

### 11. Incomplete API Implementations
**Marked with "for now" comments** (30+ locations):
- `source-videos/src/api/routes/sources.rs:126` - Update not fully implemented
- `source-videos/src/api/routes/server.rs:110,130,147` - Simplified responses
- `source-videos/src/api/routes/config.rs:28` - Config update placeholder
- Various authentication checks skipped

## Technical Debt & Architecture 🏗️

### Dependency Reduction (PRP-50/60)
**Current State**: 463 dependencies
**Goal**: <50 for basic usage

Proposed crate structure:
- **Core** (0 deps): `ds-core`
- **Error** (1 dep): `ds-error` 
- **GStreamer** (5-10 deps): `ds-gstreamer`, `ds-backend`
- **Optional Heavy**:
  - `cpu-inference` (~150 deps with ONNX)
  - `image-utils` (~100 deps)
  - `video-server` (~200 deps with axum/tokio)
  - `cli-utils` (~20 deps)

### Code Quality Metrics (Latest Scan)
- **TODO comments**: 11 explicit (down from 13)
- **"for now" patterns**: 21 temporary implementations  
- **"actual" references**: 12 needing real implementations
- **Unused parameters**: 35+ with `_` prefix
- **NOTE comments**: 15 documentation/clarification notes

## Next Actions 🎯

### Week 1: Critical Fixes
1. Remove global error classifier
2. Start tokio replacement investigation
3. Fix highest priority unwrap() calls

### Week 2: Testing & Quality
4. Add real ONNX models for testing
5. Implement mock backend conditional compilation
6. Run Debtmap analysis and fix high-severity issues

### Week 3: Feature Completion
7. Implement video metadata extraction
8. Add progressive/lazy loading
9. Complete Unix socket control

### Long-term: Architecture
10. Execute PRP-50 crate modularization
11. Complete DeepStream FFI bindings
12. Implement all metadata propagation

## CI/CD & Tooling

### New Tools Added
- **Debtmap**: Code quality analysis workflow
- **Coverage**: cargo-llvm-cov for accurate coverage
- **Dependencies**: Fixed missing libgstrtspserver-1.0-dev

### Build Issues to Address
- Long build times due to 463 dependencies
- Need feature gates for optional functionality
- Consider workspace optimization

## Guidelines 📝

When addressing TODOs:
1. Remove TODO comment when implementing
2. Write tests for the implementation
3. Update this file to reflect completion
4. Check for related TODOs
5. Run `cargo clippy` and fix warnings
6. Run Debtmap locally before committing
7. Ensure no new global state introduced

## Priority Definitions
- **Critical**: Security risk, panics, or blocks major functionality
- **High**: Significant technical debt or missing core features
- **Medium**: Feature completeness and performance
- **Low**: Code cleanup, nice-to-have features
