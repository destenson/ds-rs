# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust port of NVIDIA's DeepStream runtime source addition/deletion reference application. The project demonstrates dynamic video source management in AI-powered video analytics pipelines using GStreamer and DeepStream SDK.

## High-Level Architecture

### Backend Abstraction System
The codebase implements a three-tier backend system to enable cross-platform compatibility:
- **DeepStream Backend** (`crates/ds-rs/src/backend/deepstream.rs`): Uses NVIDIA hardware acceleration
- **Standard Backend** (`crates/ds-rs/src/backend/standard.rs`): Falls back to standard GStreamer elements
- **Mock Backend** (`crates/ds-rs/src/backend/mock.rs`): For testing without any real hardware

Backend selection is automatic via `BackendManager` which probes for available capabilities at runtime. This abstraction is critical for development on non-NVIDIA systems.

### Dynamic Source Management Architecture
The source management system (`crates/ds-rs/src/source/`) enables runtime addition/removal of video sources:
- **SourceManager**: Thread-safe registry using `Arc<RwLock<HashMap>>`
- **VideoSource**: Wraps uridecodebin with pad-added signal handling
- **SourceController**: High-level API coordinating manager, events, and synchronization
- **Event System**: Channel-based architecture for async source state changes

Sources link dynamically to nvstreammux (or compositor in standard backend) without pipeline interruption.

### Pipeline State Management
The pipeline module (`crates/ds-rs/src/pipeline/`) provides:
- **PipelineBuilder**: Fluent API for constructing pipelines
- **StateManager**: Validates and manages GST state transitions with recovery
- **BusWatcher**: Message handling with callback registration

## Build and Test Commands

```bash
# Main crate is in crates/ds-rs/
cd crates/ds-rs

# Build the project
cargo build --release

# Run all tests (currently 67 tests)
cargo test

# Run specific test file
cargo test --test backend_tests
cargo test --test pipeline_tests  
cargo test --test source_management

# Run single test by name
cargo test test_video_source_creation -- --exact

# Run with GStreamer debug output
GST_DEBUG=3 cargo test

# Check without building
cargo check

# Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
cargo doc --open

# Run the main application
cargo run --release --bin ds-app

# Run cross-platform example
cargo run --example cross_platform
```

## Platform-Specific Builds

```bash
# For Jetson (CUDA 10.2)
CUDA_VER=10.2 cargo build --release

# For x86 with CUDA 11.4+
CUDA_VER=11.4 cargo build --release

# For non-NVIDIA systems (auto-selects standard backend)
cargo build --release
```

## Critical Implementation Details

### Element Creation Pattern
All elements are created through the backend abstraction:
```rust
// NEVER create elements directly
// BAD: gst::ElementFactory::make("nvstreammux")

// GOOD: Use ElementFactory with backend
let factory = ElementFactory::new(backend_manager);
let mux = factory.create_stream_mux(Some("mux"))?;
```

### Source Addition Flow
1. `SourceController::add_source()` generates unique SourceId
2. Creates `VideoSource` with uridecodebin
3. Connects pad-added signal for dynamic linking
4. Adds to pipeline and sets to PLAYING state
5. Links to streammux on pad-added callback
6. Updates registry and emits SourceAdded event

### Property Setting Gotchas
- Use `set_property_from_str()` for enum-like string properties
- Standard properties use regular `set_property()`
- Mock backend validates but doesn't apply properties

### State Synchronization
When adding sources to running pipeline:
1. Source state must sync with pipeline state
2. Use `SourceSynchronizer::sync_source_with_pipeline()`
3. Handle ASYNC state changes with timeout

## Test Failures to Expect

The source_management tests will fail 10 tests when using Mock backend because uridecodebin requires actual GStreamer plugins. This is expected - these tests pass with Standard or DeepStream backends.

## Configuration Files Required

For DeepStream backend, these configs must be in working directory:
- `dstest_pgie_config.txt` - Primary inference config
- `dstest_sgie[1-3]_config.txt` - Secondary inference configs  
- `dstest_tracker_config.txt` - Tracker config
- `tracker_config.yml` - Low-level tracker settings

## Common Development Patterns

### Adding New Backend Elements
1. Add variant to `DeepStreamElementType` enum
2. Implement creation in all three backends
3. Add to `ElementFactory::create_*` method
4. Update element mapping documentation

### Implementing New Source Features
1. Add trait method to `SourceAddition` or `SourceRemoval`
2. Implement in `SourceManager`
3. Expose through `SourceController` API
4. Add event variant if state change is involved
5. Write test in `source_management.rs`

### Debugging Pipeline Issues
```bash
# Enable detailed GStreamer logging
GST_DEBUG=3 cargo run

# Generate pipeline graphs
GST_DEBUG_DUMP_DOT_DIR=/tmp cargo run
# Then convert: dot -Tpng /tmp/*.dot -o pipeline.png

# Check element availability
gst-inspect-1.0 | grep nv  # For DeepStream elements
```

## C Reference Implementation

Original implementation: `vendor/NVIDIA-AI-IOT--deepstream_reference_apps/runtime_source_add_delete/`

Key differences from C version:
- Uses Rust ownership instead of manual memory management
- Channel-based events instead of callbacks
- Backend abstraction not present in original
- Thread-safe by default with Arc/RwLock

## Known Limitations

1. **Metadata extraction** - DeepStream metadata (NvDsMeta) extraction not yet implemented (PRP-04)
2. **Main demo** - Full application matching C reference not complete (PRP-05)
3. **Mock backend** - Cannot test uridecodebin-based source management
4. **Workspace warnings** - Cargo.toml has unused workspace.edition/version keys

## CRITICAL: Check BUGS.md Before Working

**ALWAYS read `BUGS.md` at the start of any work session** to understand current critical bugs that need fixing. This file contains:
- Active bugs that break core functionality
- Specific error messages and symptoms
- Reproduction steps

When fixing bugs:
- DO NOT break existing functionality
- DO NOT introduce new bugs  
- Test thoroughly after each fix
- Reference implementations in ../NVIDIA-AI-IOT--deepstream_reference_apps/ for correct patterns

## Environment Variables

```bash
# DeepStream SDK location
export DS_SDK_ROOT=/opt/nvidia/deepstream/deepstream

# Add DeepStream plugins to GStreamer
export GST_PLUGIN_PATH=$DS_SDK_ROOT/lib/gstreamer-1.0:$GST_PLUGIN_PATH

# Library paths
export LD_LIBRARY_PATH=$DS_SDK_ROOT/lib:$LD_LIBRARY_PATH

# Force specific backend (optional)
export FORCE_BACKEND=mock  # or standard, deepstream

# Set CUDA version for platform detection
export CUDA_VER=10.2  # Jetson
export CUDA_VER=11.4  # x86 with GPU
```

## Gstreamer commands

Learn about gstreamer elements using `gst-inspect-1.0 <element_name>`

For example:
```
# list all elements
gst-inspect-1.0

# get info on videotestsrc
gst-inspect-1.0 videotestsrc

# get info on uridecodebin
gst-inspect-1.0 uridecodebin
```

`gstreamer-rs` is the Rust bindings for GStreamer. See https://crates.io/crates/gstreamer for documentation and examples.

## CRITICAL: Reference Local Source Code

**NEVER make assumptions about library code. ALWAYS check the actual implementation in these local repositories:**

### Available Source Repositories
The following repositories are available locally for reference:

- **`../gstreamer-rs/`** - GStreamer Rust bindings source code
  - Check actual API signatures, trait implementations, and usage patterns
  - Reference: `gstreamer/src/`, `gstreamer-app/src/`, `gstreamer-video/src/`
  
- **`../gst-plugins-rs/`** - GStreamer plugins written in Rust
  - Example implementations of custom elements and plugins
  - Reference for creating new GStreamer elements in Rust
  
- **`../MultimediaTechLab--YOLO/`** - YOLO implementation examples
  - Integration patterns for object detection in video pipelines
  
- **`../pykeio--ort/`** - ONNX Runtime Rust bindings
  - For ML inference integration patterns
  
- **`../NVIDIA-AI-IOT--deepstream_reference_apps/`** - Original DeepStream C/C++ implementations
  - The C reference for this port is in `runtime_source_add_delete/`
  - Check for correct DeepStream API usage and patterns
  
- **`../prominenceai--deepstream-services-library/`** - DeepStream Services Library
  - High-level DeepStream abstractions and utilities
  - Reference for DeepStream best practices

### How to Use These References

Before implementing any functionality:
1. **Search the relevant repository** for existing implementations or similar patterns
2. **Read the actual source code** to understand correct API usage
3. **Check examples** in the repositories for working code patterns
4. **Verify function signatures** and trait requirements from the source

**IMPORTANT: Always use tools (Grep, Glob, Read) instead of bash commands for searching:**

Example workflow:
```
# When working with GStreamer elements, check actual implementation:
Use Grep tool: pattern="ElementFactory" path="../gstreamer-rs/"
Use Grep tool: pattern="pad_added" path="../gstreamer-rs/examples/"

# When implementing DeepStream features, reference the C code:
Use Grep tool: pattern="nvstreammux" path="../NVIDIA-AI-IOT--deepstream_reference_apps/"
Use Grep tool: pattern="source_bin" path="../prominenceai--deepstream-services-library/"

# For custom plugin patterns:
Use LS tool: path="../gst-plugins-rs/video/"
Use Grep tool: pattern="impl ElementImpl" path="../gst-plugins-rs/"
```

**DO NOT:**
- Guess API signatures or method names
- Assume how a library works without checking
- Create wrapper types that might already exist
- Implement functionality that's already available in these libraries

**ALWAYS:**
- Reference the actual source code before implementing
- Use existing patterns from the reference repositories
- Check for helper functions or utilities that already exist
- Validate against working examples in the repositories
