# PRP-51: Fix cpuinfer Plugin Build and Registration for GStreamer

## Problem Statement

The cpuinfer crate is intended to be a GStreamer plugin that provides CPU-based inference as an alternative to NVIDIA's nvinfer element. However, it's not currently building as a proper GStreamer plugin that can be discovered by `gst-inspect-1.0` and used with `gst-launch-1.0`. The plugin needs proper build configuration, registration, and installation to be usable as a standard GStreamer element.

## Research & References

### GStreamer Plugin Architecture
- https://gstreamer.freedesktop.org/documentation/gstreamer/gstplugin.html - Official plugin documentation
- https://gstreamer.freedesktop.org/documentation/tools/gst-inspect.html - gst-inspect tool documentation
- https://gstreamer.freedesktop.org/documentation/gstreamer/running.html - Runtime environment documentation

### Reference Implementations
- `../gst-plugins-rs/tutorial/` - Tutorial plugin showing standard structure
- `../gst-plugins-rs/*/src/lib.rs` - Multiple examples of proper plugin registration
- `../gstreamer-rs/examples/` - GStreamer Rust bindings examples

### Plugin Discovery Mechanisms
- System paths: `/usr/lib/x86_64-linux-gnu/gstreamer-1.0/` (Linux), `C:\gstreamer\1.0\x86_64\lib\gstreamer-1.0\` (Windows)
- Environment variable: `GST_PLUGIN_PATH` for additional paths
- Plugin cache: `~/.cache/gstreamer-1.0/` stores plugin registry

## Current State Analysis

### What's Already There
- `crates/cpuinfer/src/lib.rs` has basic plugin structure with `gst::plugin_define!`
- `crates/cpuinfer/build.rs` uses `gst_plugin_version_helper::info()`
- `crates/cpuinfer/src/cpudetector/` has the element implementation
- Cargo.toml has correct crate-type: `["cdylib", "rlib"]`

### What's Missing/Wrong
1. Plugin name in lib.rs doesn't match library name pattern
2. Element registration happens in wrong place (ds-rs crate instead of plugin init)
3. No proper installation mechanism for the plugin
4. Missing proper plugin metadata

## Implementation Blueprint

### Phase 1: Fix Plugin Structure

#### 1.1 Update lib.rs Plugin Definition
**File**: `crates/cpuinfer/src/lib.rs`
**Changes Needed**:
- Ensure plugin_init properly registers the cpudetector element
- Plugin name should match library name pattern (gstcpuinfer)
- Remove duplicate registration from ds-rs crate

#### 1.2 Fix Element Registration
**File**: `crates/cpuinfer/src/cpudetector/mod.rs`
**Pattern to Follow**: Same as `../gst-plugins-rs/tutorial/src/rgb2gray/mod.rs`
- Create proper glib wrapper
- Register element with correct name (cpudetector or cpuinfer)
- Set appropriate rank (SECONDARY or NONE)

#### 1.3 Remove Duplicate Registration
**File**: `crates/ds-rs/src/lib.rs`
**Changes Needed**:
- Remove the `register_elements()` function that tries to register cpudetector
- Remove the temporary plugin loading hack

### Phase 2: Build Configuration

#### 2.1 Update Cargo.toml
**File**: `crates/cpuinfer/Cargo.toml`
**Changes Needed**:
- Ensure library name follows GStreamer convention: `name = "gstcpuinfer"`
- Keep crate-type as `["cdylib", "rlib"]`
- Add proper metadata fields

#### 2.2 Verify build.rs
**File**: `crates/cpuinfer/build.rs`
**Current State**: Already correct with `gst_plugin_version_helper::info()`

### Phase 3: Installation and Discovery

#### 3.1 Create Installation Script (Windows)
**File**: `crates/cpuinfer/install.ps1`
**Purpose**: Copy built plugin to GStreamer plugin directory
**Logic**:
- Find GStreamer installation (check common paths or use GST_PLUGIN_PATH)
- Copy the built .dll from target/release to plugin directory
- Clear plugin cache if exists

#### 3.2 Create Installation Script (Linux)
**File**: `crates/cpuinfer/install.sh`
**Purpose**: Install plugin to system or user directory
**Logic**:
- Check for system GStreamer path or use GST_PLUGIN_PATH
- Copy .so file to appropriate location
- Run ldconfig if installing system-wide
- Clear plugin cache

#### 3.3 Development Environment Setup
**File**: `crates/cpuinfer/dev-setup.md`
**Contents**: Instructions for developers
- How to set GST_PLUGIN_PATH for development
- How to verify plugin is discovered with gst-inspect-1.0
- Example gst-launch-1.0 pipelines for testing

## Testing Strategy

### Verification Steps
1. Build the plugin: `cargo build --release -p cpuinfer`
2. Set plugin path: `export GST_PLUGIN_PATH=target/release` (Linux) or `set GST_PLUGIN_PATH=target\release` (Windows)
3. Check plugin discovery: `gst-inspect-1.0 cpuinfer`
4. List elements: `gst-inspect-1.0 cpudetector`
5. Test pipeline: `gst-launch-1.0 videotestsrc ! cpudetector ! fakesink`

### Troubleshooting Commands
- Check for blacklisting: `gst-inspect-1.0 -b`
- Clear cache: `rm -rf ~/.cache/gstreamer-1.0/`
- Check dependencies: `ldd target/release/libgstcpuinfer.so` (Linux)
- Verbose loading: `GST_DEBUG=GST_PLUGIN_LOADING:7 gst-inspect-1.0`

## Element Naming Convention

Based on nvinfer compatibility requirements:
- Plugin name: `cpuinfer`
- Element name: `cpuinfer` (to be drop-in replacement for nvinfer)
- Library name: `libgstcpuinfer.so` (Linux) / `gstcpuinfer.dll` (Windows)

## Validation Gates

```bash
# Build the plugin
cargo build --release -p cpuinfer

# Check plugin file exists
ls target/release/libgstcpuinfer.* || ls target/release/gstcpuinfer.*

# Set plugin path for testing
export GST_PLUGIN_PATH=$(pwd)/target/release

# Verify plugin is discovered
gst-inspect-1.0 cpuinfer | grep "Plugin Details"

# Verify element is available
gst-inspect-1.0 cpudetector | grep "Factory Details"

# Test basic pipeline
gst-launch-1.0 videotestsrc num-buffers=10 ! cpudetector ! fakesink

# Check for errors in plugin loading
GST_DEBUG=GST_PLUGIN_LOADING:7 gst-inspect-1.0 cpuinfer 2>&1 | grep -i error

# Verify no blacklisting
gst-inspect-1.0 -b | grep cpuinfer && echo "ERROR: Plugin is blacklisted" || echo "Plugin not blacklisted"
```

## Success Criteria

- Plugin builds without errors
- `gst-inspect-1.0` finds the cpuinfer plugin
- `gst-inspect-1.0 cpudetector` shows element details
- Simple pipeline with cpudetector runs without errors
- No blacklisting or loading errors
- Plugin works on both Linux and Windows

## Common Pitfalls to Avoid

1. **Wrong library naming**: GStreamer expects specific naming patterns
2. **Missing dependencies**: ONNX Runtime DLLs must be available
3. **Cache issues**: Old plugin cache can prevent discovery
4. **Path separators**: Use `:` on Linux, `;` on Windows for GST_PLUGIN_PATH
5. **Rank too high**: Don't use PRIMARY rank unless replacing default elements

## Risk Assessment

- **Medium Risk**: Breaking existing ds-rs integration
- **Mitigation**: Test ds-rs examples after changes
- **Low Risk**: Plugin naming conflicts
- **Mitigation**: Use unique cpuinfer name

## Estimated Effort

- Code changes: 2-3 hours
- Testing and debugging: 2-3 hours
- Documentation: 1 hour
- Total: 5-7 hours

## References from Codebase

- Pattern to follow: `../gst-plugins-rs/tutorial/src/lib.rs`
- Element registration: `../gst-plugins-rs/tutorial/src/rgb2gray/mod.rs`
- Build configuration: `../gst-plugins-rs/tutorial/Cargo.toml`
- Plugin examples: Any plugin in `../gst-plugins-rs/*/src/lib.rs`

## Confidence Score: 7/10

The implementation path is clear with good reference examples. Main complexity is ensuring proper installation and discovery across different platforms. The score would be higher with more specific knowledge of the target deployment environment.