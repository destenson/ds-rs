# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust port of NVIDIA's DeepStream runtime source addition/deletion reference application. The project demonstrates dynamic video source management in AI-powered video analytics pipelines using GStreamer and DeepStream SDK.

## Architecture

### Project Structure
- **PRPs/** - Project Requirement and Planning documents defining implementation phases
- **src/** - Main library code for DeepStream Rust bindings
- **crates/dsl** - DeepStream Library Services crate (for future use)
- **vendor/** - Original C implementation reference

### Implementation Phases (PRPs)
1. **Core Infrastructure** - FFI bindings, build system, error handling
2. **Hardware Abstraction** - Runtime detection and fallback to standard GStreamer
3. **GStreamer Pipeline** - Pipeline management with abstracted elements
4. **Source Control APIs** - Runtime source addition/deletion
5. **DeepStream Integration** - Metadata handling and inference processing
6. **Main Application** - CLI and demonstration runner

### Key Dependencies
- `gstreamer = "0.24.1"` - Official GStreamer Rust bindings (includes DeepStream element access)
- NVIDIA DeepStream SDK 6.0+ (installed as GStreamer plugins)
- CUDA 10.2 (Jetson) or 11.4+ (x86)
- DeepStream elements accessed via standard GStreamer API - no custom FFI needed

## Build Commands

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Run with all GStreamer features
cargo build --features gst_v1_24

# Build for Jetson (CUDA 10.2)
CUDA_VER=10.2 cargo build --release

# Build for x86 (CUDA 11.4)
CUDA_VER=11.4 cargo build --release

# Run the main application (once implemented)
cargo run --release -- <video_uri>
# Example: cargo run --release -- file:///path/to/video.mp4
```

## Development Commands

```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Run a specific test
cargo test test_name

# Build documentation
cargo doc --open

# Clean build artifacts
cargo clean
```

## DeepStream Integration Notes

### Environment Setup
```bash
# Required environment variable for DeepStream SDK
export DS_SDK_ROOT=/opt/nvidia/deepstream/deepstream

# GStreamer plugin path for DeepStream elements
export GST_PLUGIN_PATH=$DS_SDK_ROOT/lib/gstreamer-1.0:$GST_PLUGIN_PATH

# Library paths
export LD_LIBRARY_PATH=$DS_SDK_ROOT/lib:$LD_LIBRARY_PATH
```

### Platform Detection
The build system should detect platform via CUDA version:
- Jetson: CUDA 10.2, uses integrated GPU features
- x86: CUDA 11.4+, requires explicit GPU ID configuration

### Critical DeepStream Components (GStreamer Elements)
- **nvstreammux**: Batches streams from multiple sources
- **nvinfer**: Runs TensorRT inference (pgie/sgie)
- **nvtracker**: Object tracking across frames
- **nvdsosd**: On-screen display for bounding boxes
- **nvtiler**: Composites multiple streams into grid

All these are standard GStreamer elements created via:
```rust
let element = gst::ElementFactory::make("nvstreammux")
    .property("batch-size", 1)
    .build()?;
```

## C Reference Implementation

The original C implementation in `vendor/NVIDIA-AI-IOT--deepstream_reference_apps/runtime_source_add_delete/` demonstrates:
- Pipeline: uridecodebin -> nvstreammux -> nvinfer -> nvtracker -> nvtiler -> nvvideoconvert -> nvdsosd -> displaysink
- Adds sources every 10 seconds up to MAX_NUM_SOURCES (4)
- Then removes sources periodically
- Handles per-stream EOS events

Key functions to reference:
- `create_uridecode_bin()` - Creates source elements
- `add_sources()` - Runtime source addition
- `delete_sources()` - Runtime source removal
- `cb_newpad()` - Dynamic pad handling

## Hardware Abstraction and Cross-Platform Support

The application includes a hardware abstraction layer (PRP-06) that enables it to run on systems without NVIDIA hardware:

### Backend Detection
```rust
// Runtime detection of available backends
let backend = detect_available_backends();
match backend {
    BackendType::DeepStream => // Use NVIDIA elements
    BackendType::Standard => // Use standard GStreamer elements
    BackendType::Mock => // Use mock elements for testing
}
```

### Element Mapping
- **nvstreammux** → compositor + queue
- **nvinfer** → fakesink or appsink with mock inference
- **nvtracker** → identity element
- **nvdsosd** → textoverlay + videobox
- **nvvideoconvert** → videoconvert

This enables development and testing on any system with GStreamer installed.

## DeepStream Element Usage

When NVIDIA hardware is available, DeepStream functionality is accessed through GStreamer elements:
```rust
// Create DeepStream elements using gstreamer-rs
let streammux = gst::ElementFactory::make("nvstreammux")
    .property("batch-size", 30)
    .property("width", 1920)
    .property("height", 1080)
    .build()?;

let pgie = gst::ElementFactory::make("nvinfer")
    .property("config-file-path", "dstest_pgie_config.txt")
    .build()?;
```

Note: Metadata extraction (line 25 reference) may require minimal FFI for `nvdsmeta.h` structures.

## Testing Strategy

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test pipeline construction and state changes
3. **Memory Tests**: Use valgrind to check for leaks
   ```bash
   valgrind --leak-check=full target/release/ds-runtime-demo <uri>
   ```
4. **Stress Tests**: Rapid source add/remove cycles

## Common Issues and Solutions

### Issue: DeepStream elements not found
**Solution**: Ensure DeepStream SDK is installed and `GST_PLUGIN_PATH` includes DeepStream plugins:
```bash
export GST_PLUGIN_PATH=$DS_SDK_ROOT/lib/gstreamer-1.0:$GST_PLUGIN_PATH
```

### Issue: CUDA version mismatch
**Solution**: Set `CUDA_VER` environment variable before building

### Issue: Pipeline state change failures
**Solution**: Check element compatibility and ensure all config files are present

## Configuration Files

The application requires DeepStream config files in the working directory:
- `dstest_pgie_config.txt` - Primary inference engine config
- `dstest_sgie[1-3]_config.txt` - Secondary inference configs
- `dstest_tracker_config.txt` - Tracker configuration
- `tracker_config.yml` - Low-level tracker config

These files configure model paths, inference parameters, and tracking algorithms.

## Performance Considerations

- Use `nvstreammux` batching for optimal GPU utilization
- Set appropriate `batch-size` based on GPU memory
- Enable `nvv4l2decoder` hardware acceleration on Jetson
- Use `enable-max-performance` for Jetson platforms

## Debug Tools

```bash
# Enable GStreamer debug output
export GST_DEBUG=3

# Generate pipeline graph
export GST_DEBUG_DUMP_DOT_DIR=/tmp
# Graphs will be generated in /tmp as .dot files

# List available GStreamer elements
gst-inspect-1.0 | grep nv

# Inspect specific element
gst-inspect-1.0 nvstreammux
```
