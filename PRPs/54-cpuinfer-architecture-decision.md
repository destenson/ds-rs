# PRP-54: CpuInfer Architecture Decision - Dual-Use Crate vs Interface Library

**Priority**: Critical
**Confidence Level**: 8/10

## Problem Statement

The cpuinfer crate was recently converted from a library dependency to a GStreamer plugin (gstcpuinfer), but ds-rs still has multiple references to cpuinfer-specific types throughout the codebase. The build is currently broken with 14 compilation errors related to unresolved `gstcpuinfer` imports. We need to decide on the architectural approach:

1. **Dual-use crate**: Make cpuinfer work as both a GStreamer plugin (cdylib) and a Rust library (rlib)
2. **Separate interface library**: Create a separate crate for shared types that both ds-rs and cpuinfer can depend on
3. **Remove cpuinfer types**: Eliminate all cpuinfer-specific types from ds-rs and use generic interfaces

## Current State Analysis

### Broken Dependencies
The following files have unresolved imports of `gstcpuinfer`:
- `crates/ds-rs/src/backend/cpu_vision/mod.rs` - Re-exports detector types
- `crates/ds-rs/src/error/mod.rs` - Uses DetectorError
- `crates/ds-rs/src/backend/cpu_vision/tracker.rs` - Uses Detection type
- `crates/ds-rs/src/backend/cpu_vision/metadata.rs` - Uses Detection type
- `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs` - Uses OnnxDetector and DetectorConfig
- `crates/ds-rs/src/multistream/pipeline_pool.rs` - Uses detector types
- `crates/ds-rs/src/multistream/config.rs` - Uses DetectorConfig
- `crates/ds-rs/src/rendering/metadata_bridge.rs` - Uses Detection type

### CpuInfer Plugin Structure
- Currently configured as: `crate-type = ["cdylib", "rlib"]` in `crates/cpuinfer/Cargo.toml`
- Has public exports: `detector`, `config` modules and conditionally exports `ort`
- Implements GStreamer plugin registration in `lib.rs`
- Contains detector implementation with types: `OnnxDetector`, `DetectorConfig`, `Detection`, `YoloVersion`, `DetectorError`

### Architecture Patterns in Codebase
- Backend abstraction system already exists (DeepStream, Standard, Mock backends)
- Similar dual-use pattern seen in gst-plugins-rs (e.g., gtk4 plugin)
- Inference configuration system in `crates/ds-rs/src/inference/` with its own config types

## Recommendation: Dual-Use Crate Approach

### Rationale
1. **Minimal refactoring**: The cpuinfer Cargo.toml already specifies both `cdylib` and `rlib`
2. **Precedent exists**: gst-plugins-rs uses this pattern successfully (gtk4, webp, etc.)
3. **Maintains compatibility**: Both plugin and library usage patterns preserved
4. **Single source of truth**: Detector types remain in one place
5. **Simpler dependency graph**: No additional interface crate needed

### Alternative Considerations

**Interface Library Approach**:
- Would require creating new crate, moving types, updating both cpuinfer and ds-rs
- Adds complexity without clear benefit
- More packages to maintain and version

**Type Removal Approach**:
- Would require significant refactoring of cpu_vision backend
- Loss of type safety and detector-specific functionality
- Goes against existing architecture patterns

## Implementation Blueprint

### Phase 1: Fix Immediate Build Issues
1. Add cpuinfer as workspace dependency with path reference
2. Update ds-rs Cargo.toml to include cpuinfer as optional dependency
3. Ensure cpuinfer exports are properly accessible when used as library

### Phase 2: Refine Architecture
1. Review and consolidate detector types between cpuinfer and ds-rs inference module
2. Ensure clean separation between plugin functionality and library API
3. Update backend abstraction to properly handle cpuinfer as both plugin and library

### Phase 3: Testing and Documentation
1. Test cpuinfer as GStreamer plugin via gst-inspect-1.0
2. Test ds-rs compilation with cpuinfer as library dependency
3. Verify runtime behavior in both usage modes
4. Document dual-use pattern in CLAUDE.md

## Files to Reference

### Primary Files to Modify
- `crates/ds-rs/Cargo.toml` - Add cpuinfer dependency
- `Cargo.toml` (workspace root) - Ensure cpuinfer in workspace members
- `crates/cpuinfer/src/lib.rs` - Verify proper exports for library usage

### Pattern References
- `../gst-plugins-rs/video/gtk4/Cargo.toml` - Example of dual-use crate configuration
- `crates/ds-rs/src/backend/` - Backend abstraction patterns to follow
- `crates/ds-rs/src/inference/config.rs` - Existing inference configuration types

## Tasks

1. Add cpuinfer as dependency to ds-rs with correct configuration
2. Verify cpuinfer module exports are public and accessible
3. Test build with cargo check on both crates
4. Run integration tests for cpu_vision backend
5. Test GStreamer plugin registration and loading
6. Update documentation about dual-use architecture

## Validation Gates

```bash
# Build validation
cd crates/cpuinfer && cargo build --release
cd crates/ds-rs && cargo check --all-features

# Plugin validation  
gst-inspect-1.0 cpuinfer

# Test validation
cargo test --all-features --workspace

# Lint validation
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings
```

## Risk Mitigation

- **Binary size increase**: Monitor .so/.dll size with both crate types
- **Version conflicts**: Use workspace versioning to ensure consistency
- **Plugin loading issues**: Test GST_PLUGIN_PATH configuration thoroughly
- **Type duplication**: Audit for redundant types between cpuinfer and ds-rs inference module

## Success Criteria

1. ds-rs builds successfully with cpuinfer types available
2. cpuinfer works as GStreamer plugin (gst-inspect-1.0 shows it)
3. No regression in existing tests
4. Both usage patterns documented and tested

## External Documentation

- Rust dual crate types: https://doc.rust-lang.org/reference/linkage.html
- GStreamer Rust plugin development: https://github.com/gstreamer/gstreamer-rs
- Example dual-use plugins: https://github.com/gstreamer/gst-plugins-rs

## Notes

The dual-use approach aligns with existing patterns in the GStreamer Rust ecosystem and requires minimal changes to restore functionality. This approach maintains the flexibility for cpuinfer to be used both as a dynamically loaded GStreamer plugin and as a compile-time Rust dependency, which is valuable for testing and direct integration scenarios.