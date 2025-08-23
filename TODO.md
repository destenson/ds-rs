# TODO List

## Critical Priority 🔴

### Pipeline Management (PRP-02)
- [x] Create `src/pipeline/` module structure
- [x] Implement `PipelineBuilder` with fluent API
- [x] Add pipeline state management (NULL → READY → PAUSED → PLAYING)
- [x] Implement bus message handling
- [x] Add EOS event handling
- [x] Create pipeline element linking logic
- [x] Add error recovery mechanisms
- **Files**: Created `src/pipeline/mod.rs`, `src/pipeline/builder.rs`, `src/pipeline/state.rs`, `src/pipeline/bus.rs`
- **Status**: ✅ COMPLETED

### Source Control APIs (PRP-03)
- [ ] Create `src/source/` module structure
- [ ] Implement runtime source addition (`add_source`)
- [ ] Implement runtime source removal (`remove_source`)
- [ ] Add source bin creation with uridecodebin
- [ ] Implement pad-added signal handling
- [ ] Add source registry management
- [ ] Create thread-safe source operations
- **Files**: Need to create `src/source/mod.rs`, `src/source/manager.rs`
- **Blocking**: Dynamic source management feature

## High Priority 🟡

### DeepStream Integration (PRP-04)
- [ ] Implement metadata extraction from buffers
- [ ] Add NvDsMeta FFI bindings (minimal set)
- [ ] Create object detection metadata parsing
- [ ] Add classification metadata support
- [ ] Implement stream-specific message handling
- [ ] Add inference result callbacks
- **Files**: Need to create `src/metadata/`, potentially `src/ffi/nvdsmeta.rs`
- **Impact**: Cannot access AI inference results

### Main Application Demo (PRP-05)
- [ ] Create full demo matching C reference behavior
- [ ] Add CLI argument parsing
- [ ] Implement source addition timer (every 10 seconds)
- [ ] Add source removal logic after MAX_NUM_SOURCES
- [ ] Create configuration file loading
- [ ] Add signal handling (SIGINT)
- **Files**: Update `src/main.rs`
- **Impact**: No working demonstration of the port

## Medium Priority 🟢

### Code Quality Improvements
- [ ] Replace `unwrap()` calls in non-test code (25 instances)
  - `src/elements/factory.rs`: 3 instances
  - `src/config/mod.rs`: 8 instances
  - `src/platform.rs`: 2 instances
  - `src/backend/standard.rs`: 2 instances
  - `src/elements/abstracted.rs`: 3 instances
  - `src/backend/detector.rs`: 1 instance
  - `src/backend/mock.rs`: 6 instances
- [ ] Remove dead code warnings
  - Remove unused `platform` field in `StandardBackend` (src/backend/standard.rs:10)
  - Remove unused `platform` field in `MockBackend` (src/backend/mock.rs:10)
- [ ] Fix workspace Cargo.toml warnings
  - Use workspace.version in main Cargo.toml:12
  - Use workspace.edition in main Cargo.toml:13

### Testing & Examples
- [ ] Add integration tests with actual video files
- [ ] Create example for each backend type
- [ ] Add pipeline state transition tests
- [ ] Create source addition/removal example
- [ ] Add memory leak tests (valgrind)
- [ ] Implement stress tests for rapid source changes

### Documentation
- [x] ~~Create README.md~~ ✅ Completed
- [ ] Add inline documentation for all public APIs
- [ ] Create architecture diagrams
- [ ] Write migration guide from C to Rust
- [ ] Document backend selection logic
- [ ] Add troubleshooting guide for common issues

## Low Priority 🔵

### CI/CD & Infrastructure
- [ ] Set up GitHub Actions workflow
- [ ] Add multi-platform CI testing (Linux, Windows)
- [ ] Configure automated releases
- [ ] Add code coverage reporting
- [ ] Set up dependency updates (dependabot)
- [ ] Add benchmark suite

### Temporary/Incomplete Implementations
- [ ] Replace compute capability detection placeholder (`src/platform.rs:132`)
  - Currently returns hardcoded values based on platform
  - Should query actual GPU capabilities via nvidia-smi or CUDA API
- [ ] Implement actual CPU-based inference in StandardBackend
  - Currently using fakesink as placeholder (`src/backend/standard.rs:106`)
  - Could integrate ONNX Runtime or TensorFlow Lite
- [ ] Complete DSL crate implementation (`crates/dsl/src/lib.rs:8`)
  - Currently has `todo!()` placeholder
  - Needs actual DeepStream Services Library functionality

### Future Enhancements
- [ ] Add RTSP source support
- [ ] Implement custom inference post-processing
- [ ] Add performance profiling tools
- [ ] Create Docker container for easy deployment
- [ ] Add WebRTC sink support
- [ ] Implement cloud inference backend
- [ ] Add Kubernetes deployment manifests
- [ ] Add actual CPU inference support (ONNX, TensorFlow Lite)
- [ ] Implement GPU capability detection via nvidia-ml or CUDA APIs

### Performance Optimizations
- [ ] Profile and optimize hot paths
- [ ] Reduce allocations in frame processing
- [ ] Implement zero-copy buffer passing where possible
- [ ] Add frame skipping for overloaded systems
- [ ] Optimize backend detection caching

## Recently Completed ✅

- [x] Fix compositor background property bug in StandardBackend
- [x] Fix text overlay alignment properties 
- [x] Create comprehensive README.md
- [x] Implement core infrastructure (PRP-01)
- [x] Implement hardware abstraction layer (PRP-06)
- [x] Create backend detection system
- [x] Add configuration parsing system

## Known Issues 🐛

1. **Cross-platform example** - ✅ FIXED
   - ~~Runtime panic with compositor background property~~
   - ~~File: `src/backend/standard.rs:80`~~

2. **DeepStream Backend** - Not tested on actual NVIDIA hardware
   - Need validation on Jetson and x86 with GPU

3. **Memory Management** - Need to verify no leaks
   - Particularly around dynamic source addition/removal

## Notes

- Priority based on blocking dependencies and user impact
- PRP-02 (Pipeline) and PRP-03 (Sources) are most critical for basic functionality
- Code quality improvements can be done incrementally
- CI/CD can wait until core functionality is complete

## Contributing

When working on any TODO item:
1. Create a feature branch
2. Update this TODO.md to mark item as in-progress
3. Write tests for new functionality
4. Update documentation as needed
5. Mark complete in TODO.md when merged

Last Updated: 2024-01-23