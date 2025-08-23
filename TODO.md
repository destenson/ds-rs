# TODO List

## Critical Priority 🔴

### Dynamic Video Sources Test Infrastructure (PRP-07)
- [ ] Create test video generation crate (`source-videos`)
- [ ] Implement RTSP server for test streams
- [ ] Add configurable test pattern generation
- [ ] Support multiple concurrent video streams
- [ ] Enable self-contained testing without external files
- **Impact**: Required for comprehensive testing and CI/CD
- **Files**: Need to expand `crates/source-videos/`

## High Priority 🟡

### Real DeepStream FFI Implementation
- [ ] Implement actual FFI bindings when DeepStream SDK available
- [ ] Replace mock metadata extraction with real `gst_buffer_get_nvds_batch_meta`
- [ ] Add proper NvDsMeta structure bindings
- [ ] Implement `gst_nvmessage_is_stream_eos` and related functions
- **Current state**: Using simulated metadata for development
- **Files**: `crates/ds-rs/src/metadata/mod.rs:60`, `messages/mod.rs:175,182`

### Code Quality Improvements
- [ ] Fix workspace Cargo.toml configuration
  - `crates/ds-rs/Cargo.toml:3`: Use workspace version instead of hardcoded "0.1.0"
  - `crates/ds-rs/Cargo.toml:4`: Use workspace edition instead of hardcoded "2024"
- [ ] Fix deprecated rand API usage
  - `crates/ds-rs/src/source/controller.rs:287-288`: Update to modern rand API
  - `crates/ds-rs/src/app/timers.rs:106-107`: Update to modern rand API
- [ ] Replace `unwrap()` calls in non-test code (83 occurrences across 23 files)
  - Highest priority files:
    - `config/mod.rs`: 8 instances
    - `source/mod.rs`: 9 instances  
    - `backend/mock.rs`: 6 instances
    - `source/video_source.rs`: 6 instances
    - `pipeline/mod.rs`: 6 instances
    - `app/timers.rs`: 5 instances
    - `app/mod.rs`: 5 instances

## Medium Priority 🟢

### Testing & Examples
- [ ] Create test RTSP source for better integration testing
- [ ] Add integration tests with actual video files
- [ ] Create example for each backend type
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
- [ ] Document metadata extraction architecture

### Placeholder Implementations to Replace
- [ ] Replace mock metadata extraction (`metadata/mod.rs:61`)
  - Currently returns mock data, needs real DeepStream integration
- [ ] Fix stream EOS detection (`messages/mod.rs:175`)
  - Currently using simplified check
- [ ] Implement proper stream ID parsing (`messages/mod.rs:182`)
  - Currently returns mock stream ID
- [ ] Load actual label files (`inference/mod.rs:175`)
  - Currently returns default label map
- [ ] Parse DeepStream config files (`inference/config.rs:228`)
  - Currently returns mock configuration
- [ ] Implement actual CPU inference (`backend/standard.rs:17,108`)
  - Currently using identity element as passthrough

## Low Priority 🔵

### CI/CD & Infrastructure
- [ ] Set up GitHub Actions workflow
- [ ] Add multi-platform CI testing (Linux, Windows, macOS)
- [ ] Configure automated releases
- [ ] Add code coverage reporting
- [ ] Set up dependency updates (dependabot)
- [ ] Add benchmark suite

### Incomplete Implementations
- [ ] Replace compute capability detection (`platform.rs:149`)
  - Currently returns hardcoded values based on platform
  - Should query actual GPU capabilities via nvidia-smi or CUDA API
- [ ] Complete DSL crate implementation (`crates/dsl/src/lib.rs:8`)
  - Currently has `todo!()` placeholder
  - Needs actual DeepStream Services Library functionality
- [ ] Implement simplified processing in inference (`inference/mod.rs:214-215`)
  - Currently has simplified tensor output parsing

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

### Latest Completions (2025-08-23)
- [x] Implement DeepStream Metadata Integration (PRP-04)
  - Complete metadata extraction system with BatchMeta, FrameMeta, ObjectMeta
  - Inference result processing with detection and classification support
  - Object tracking with trajectory management
  - DeepStream message handling including stream-specific EOS
  - Comprehensive test coverage (38 new tests)
  - Example detection application demonstrating metadata extraction
- [x] Implement Main Application Demo (PRP-05)
  - Full runtime demo matching C reference behavior
  - CLI interface with timer-based source management
  - Graceful shutdown with signal handling

### Previous Completions
- [x] Implement Source Control APIs (PRP-03)
- [x] Implement Pipeline Management (PRP-02)
- [x] Implement core infrastructure (PRP-01)
- [x] Implement hardware abstraction layer (PRP-06)

## Known Issues 🐛

1. **Source Management Tests with Mock Backend**
   - 10 tests fail when using Mock backend
   - Reason: Mock backend doesn't support uridecodebin
   - Status: Expected behavior - use Standard backend for full testing

2. **Build Warnings**
   - Unused imports after linter added `#![allow(unused)]`
   - Non-snake-case field `bInferDone` (kept for DeepStream compatibility)
   - Unused workspace manifest keys

3. **DeepStream Backend** 
   - Not tested on actual NVIDIA hardware
   - Need validation on Jetson and x86 with GPU

4. **Memory Management**
   - Need to verify no leaks, particularly around dynamic source addition/removal

## Statistics 📊

- **Total TODO items**: 52 (8 critical/high, 15 medium, 29 low)
- **Code with `unwrap()`**: 83 occurrences across 23 files
- **Placeholder implementations**: 7 locations marked with "for now" comments
- **Test coverage**: 97/107 tests passing (90.7%)
- **Lines of code**: ~12,000 (excluding tests)

## Notes

- Priority based on blocking dependencies and user impact
- PRP-07 (Dynamic Video Sources) is next critical priority for testing infrastructure
- Real DeepStream FFI needed when SDK is available
- Code quality improvements should be addressed before v1.0 release

## Contributing

When working on any TODO item:
1. Create a feature branch
2. Update this TODO.md to mark item as in-progress
3. Write tests for new functionality
4. Update documentation as needed
5. Mark complete in TODO.md when merged

Last Updated: 2025-08-23