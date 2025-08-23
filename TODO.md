# TODO List

## Critical Priority 🔴

### Main Application Demo (PRP-05) - ✅ COMPLETE
- [x] Create full demo matching C reference behavior
- [x] Add CLI argument parsing with clap
- [x] Implement source addition timer (every 10 seconds)
- [x] Add source removal logic after MAX_NUM_SOURCES
- [x] Add signal handling (SIGINT) with ctrlc crate
- **Files**: Completed `crates/ds-rs/src/main.rs` and `crates/ds-rs/src/app/`
- **Impact**: Now have working demonstration of runtime source management
- **Status**: Successfully demonstrates dynamic source addition/removal capabilities

## High Priority 🟡

### DeepStream Integration (PRP-04) - ✅ COMPLETE
- [x] Implement metadata extraction from buffers
- [x] Add NvDsMeta FFI bindings (minimal set) - Simulated for Mock backend
- [x] Create object detection metadata parsing
- [x] Add classification metadata support
- [x] Implement stream-specific message handling
- [x] Add inference result callbacks
- **Files**: Created `crates/ds-rs/src/metadata/`, `inference/`, `tracking/`, `messages/`
- **Impact**: AI inference results now accessible through metadata extraction

### Code Quality Improvements
- [ ] Fix workspace Cargo.toml configuration
  - `crates/ds-rs/Cargo.toml:3`: Use workspace version instead of hardcoded "0.1.0"
  - `crates/ds-rs/Cargo.toml:4`: Use workspace edition instead of hardcoded "2024"
- [ ] Fix deprecated rand API usage
  - `crates/ds-rs/src/source/controller.rs:287`: Replace `rand::thread_rng()` with `rand::rng()`
  - `crates/ds-rs/src/source/controller.rs:288`: Replace `gen_range()` with `random_range()`
- [ ] Remove dead code warnings
  - `crates/ds-rs/src/source/mod.rs:54`: Remove unused field `next_id` in SourceManager

## Medium Priority 🟢

### Testing & Examples
- [ ] Create test RTSP source for better integration testing
  - Would allow testing source management without relying on file URIs
  - Could use GStreamer test sources with RTSP server
- [ ] Add integration tests with actual video files
- [ ] Create example for each backend type
- [ ] Create dedicated source addition/removal example
- [ ] Add memory leak tests (valgrind)
- [ ] Implement stress tests for rapid source changes
- [ ] Fix source_management test failures with Mock backend
  - 10 tests fail because Mock backend doesn't support uridecodebin
  - Consider creating mock uridecodebin or skip tests for Mock backend

### Documentation
- [ ] Add inline documentation for all public APIs
- [ ] Create architecture diagrams
- [ ] Write migration guide from C to Rust
- [ ] Document backend selection logic
- [ ] Add troubleshooting guide for common issues
- [ ] Document source management architecture

### Code Cleanup
- [ ] Replace `unwrap()` calls in non-test code (estimated 25+ instances)
  - `crates/ds-rs/src/elements/factory.rs`: 3 instances
  - `crates/ds-rs/src/config/mod.rs`: 8 instances
  - `crates/ds-rs/src/platform.rs`: 2 instances
  - `crates/ds-rs/src/backend/standard.rs`: 2 instances
  - `crates/ds-rs/src/elements/abstracted.rs`: 3 instances
  - `crates/ds-rs/src/backend/detector.rs`: 1 instance
  - `crates/ds-rs/src/backend/mock.rs`: 6 instances

## Low Priority 🔵

### CI/CD & Infrastructure
- [ ] Set up GitHub Actions workflow
- [ ] Add multi-platform CI testing (Linux, Windows)
- [ ] Configure automated releases
- [ ] Add code coverage reporting
- [ ] Set up dependency updates (dependabot)
- [ ] Add benchmark suite

### Incomplete Implementations
- [ ] Replace compute capability detection placeholder (`crates/ds-rs/src/platform.rs:132`)
  - Currently returns hardcoded values based on platform
  - Should query actual GPU capabilities via nvidia-smi or CUDA API
- [ ] Implement actual CPU-based inference in StandardBackend
  - Currently using fakesink as placeholder (`crates/ds-rs/src/backend/standard.rs:106`)
  - Could integrate ONNX Runtime or TensorFlow Lite
- [ ] Complete DSL crate implementation (`crates/dsl/src/lib.rs:8`)
  - Currently has `todo!()` placeholder
  - Needs actual DeepStream Services Library functionality

### Future Enhancements
- [ ] Add native RTSP server support
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

### Latest Completions
- [x] Implement DeepStream Metadata Integration (PRP-04) - AI inference support
  - Complete metadata extraction system with BatchMeta, FrameMeta, ObjectMeta
  - Inference result processing with detection and classification support
  - Object tracking with trajectory management
  - DeepStream message handling including stream-specific EOS
  - Comprehensive test coverage (25+ new tests)
  - Example detection application demonstrating metadata extraction
- [x] Implement Main Application Demo (PRP-05) - Runtime demonstration
  - CLI interface with clap for argument parsing
  - Automatic source addition every 10 seconds
  - Source removal after MAX_NUM_SOURCES reached
  - Graceful shutdown with signal handling
  - Backend-aware property configuration
  - Integration tests for application components
- [x] Implement Source Control APIs (PRP-03) - Dynamic source management
  - Thread-safe source registry
  - VideoSource wrapper for uridecodebin
  - Pad-added signal handling
  - Per-source EOS tracking
  - High-level SourceController API
- [x] Updated README.md to reflect current project state
- [x] Created comprehensive CLAUDE.md for AI assistant guidance

### Previous Completions
- [x] Implement Pipeline Management (PRP-02) - Complete pipeline module
- [x] Implement core infrastructure (PRP-01)
- [x] Implement hardware abstraction layer (PRP-06)
- [x] Fix compositor background property bug in StandardBackend
- [x] Fix text overlay alignment properties
- [x] Create backend detection system
- [x] Add configuration parsing system

## Known Issues 🐛

1. **Source Management Tests with Mock Backend**
   - 10 tests fail when using Mock backend
   - Reason: Mock backend doesn't support uridecodebin
   - Status: Expected behavior - use Standard backend for full testing

2. **DeepStream Backend** - Not tested on actual NVIDIA hardware
   - Need validation on Jetson and x86 with GPU

3. **Memory Management** - Need to verify no leaks
   - Particularly around dynamic source addition/removal

4. **Workspace Configuration**
   - Cargo.toml has TODOs for using workspace version/edition

## Notes

- Priority based on blocking dependencies and user impact
- PRP-05 (Main Application) is now critical priority to demonstrate the complete functionality
- Code quality improvements should be addressed before v1.0 release
- CI/CD can wait until core functionality is complete

## Contributing

When working on any TODO item:
1. Create a feature branch
2. Update this TODO.md to mark item as in-progress
3. Write tests for new functionality
4. Update documentation as needed
5. Mark complete in TODO.md when merged

Last Updated: 2025-08-23