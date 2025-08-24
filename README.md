# ds-rs

A Rust port of NVIDIA's DeepStream runtime source addition/deletion reference application, demonstrating dynamic video source management in AI-powered video analytics pipelines.

## Recent Updates

### 2025-08-24: Fixed Video Playback Freezing Issue
- ✅ Resolved critical H264 framerate negotiation bug that caused video freezing
- Added videorate and capsfilter elements to normalize framerates to 30fps
- Videos now play smoothly across various formats without stalling
- Pipeline flow optimized: uridecodebin → videorate → capsfilter → compositor

### 2025-08-24: Enhanced Logging with Timestamps
- Added timestamps to all state change log messages for better debugging and monitoring
- Implemented consistent timestamp formatting across the codebase using Unix epoch seconds
- State changes now show precise timing: `[1735056789.123] Source 1 state change SUCCESS`
- Improved visibility into pipeline state transitions and source management operations

## Features

- **Cross-Platform Support**: Run on NVIDIA hardware with DeepStream or any system with standard GStreamer
- **Hardware Abstraction**: Automatic backend detection and fallback mechanisms
- **Type-Safe GStreamer Bindings**: Leverages official gstreamer-rs for robust pipeline management
- **Dynamic Source Management**: Add and remove video sources at runtime without pipeline interruption
- **CPU Object Detection**: Custom GStreamer plugin with ONNX Runtime support for YOLOv3-v12
- **Configuration System**: Support for DeepStream configuration files and TOML-based settings
- **Pipeline Builder**: Fluent API for constructing complex GStreamer pipelines
- **Comprehensive Test Infrastructure**: Self-contained testing with RTSP server and video generation

## Architecture

### Backend System

The project features a flexible backend system that automatically detects and uses the best available video processing backend:

1. **DeepStream Backend**: Full NVIDIA hardware acceleration with AI inference
2. **Standard Backend**: Software-based processing using standard GStreamer elements
3. **Mock Backend**: Testing and development without any hardware dependencies

### Project Structure

```
ds-rs/
├── crates/
│   ├── ds-rs/              # Main library and application
│   │   ├── src/
│   │   │   ├── backend/    # Backend implementations and detection
│   │   │   │   └── cpu_vision/ # CPU-based object detection and tracking
│   │   │   ├── config/     # Configuration parsing and management
│   │   │   ├── elements/   # GStreamer element abstractions
│   │   │   ├── inference/  # AI inference processing
│   │   │   ├── metadata/   # DeepStream metadata extraction
│   │   │   ├── messages/   # Message handling and EOS tracking
│   │   │   ├── pipeline/   # Pipeline builder and state management
│   │   │   ├── source/     # Dynamic source management
│   │   │   ├── tracking/   # Object tracking and trajectories
│   │   │   ├── app/        # Main application implementation
│   │   │   ├── error.rs    # Error handling
│   │   │   ├── platform.rs # Platform detection (Jetson/x86)
│   │   │   ├── lib.rs      # Library entry point
│   │   │   └── main.rs     # Application entry point
│   │   ├── examples/       # Usage examples (4 demos)
│   │   └── tests/          # Integration tests
│   ├── cpuinfer/           # GStreamer CPU inference plugin
│   │   ├── src/
│   │   │   ├── detector.rs # ONNX detector implementation
│   │   │   └── cpudetector/ # GStreamer element implementation
│   ├── source-videos/      # Test video generation and RTSP server
│   │   ├── src/
│   │   │   ├── config.rs   # Video source configuration
│   │   │   ├── file.rs     # File generation (MP4, MKV, WebM)
│   │   │   ├── manager.rs  # Source management
│   │   │   ├── patterns.rs # 25+ test patterns (SMPTE, ball, etc.)
│   │   │   ├── pipeline/   # Pipeline builders
│   │   │   ├── rtsp/       # RTSP server implementation
│   │   │   └── main.rs     # CLI application
│   │   └── tests/
│   └── dsl/                # DeepStream Services Library (future)
├── scripts/                # Test orchestration scripts
├── TODO.md                 # Current task tracking
├── BUGS.md                 # Known issues and bug tracking
├── codebase-review-report.md # Code quality assessment
└── CLAUDE.md               # AI assistant guidance
```

## Installation

### Prerequisites

#### For NVIDIA DeepStream Support
- NVIDIA DeepStream SDK 6.0+
- CUDA 10.2 (Jetson) or 11.4+ (x86)
- TensorRT 8.0+
- GStreamer 1.14+

#### For Standard GStreamer Support
- GStreamer 1.14+ with base, good, bad, and rtspserver plugins
- Rust 1.70+ (edition 2024 compatible)

#### For CPU Object Detection (Optional)
- ONNX Runtime 1.16.3 (automatically downloaded by build)
- YOLO models (.onnx format) from Ultralytics or official repos

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/ds-rs.git
cd ds-rs

# Build all workspace members
cargo build --release

# Build with CPU vision features (includes ONNX support)
cargo build --release --features cpu_vision,nalgebra

# Run tests (200+ total tests)
cargo test

# Build with specific GStreamer version
cargo build --features gst_v1_24
```

### Platform-Specific Builds

```bash
# For Jetson platforms
CUDA_VER=10.2 cargo build --release

# For x86 with CUDA
CUDA_VER=11.4 cargo build --release

# For systems without NVIDIA hardware (uses standard GStreamer)
cargo build --release
```

## CPU Inference Plugin

The project includes a custom GStreamer plugin (`cpuinfer`) for CPU-based object detection:

### Features
- **ONNX Runtime Support**: YOLOv3-v12 models with automatic version detection
- **Multiple Backends**: ONNX (default), OpenCV DNN, or mock detection
- **Float32 Support**: Full precision inference (Float16 models currently unsupported due to lifetime constraints)
- **Passthrough Architecture**: Identity element behavior with signal emission

### Building the Plugin

```bash
# Build with ONNX support (default)
cd crates/cpuinfer
cargo build --release

# Build without ONNX (lightweight mock mode)
cargo build --release --no-default-features

# Install the plugin (Windows example)
copy target\release\gstcpuinfer.dll %GSTREAMER_1_0_ROOT_X86_64%\lib\gstreamer-1.0\

# Linux/macOS
cp target/release/libgstcpuinfer.so /usr/lib/x86_64-linux-gnu/gstreamer-1.0/
```

### Using the Plugin

```bash
# Basic pipeline with CPU detection
gst-launch-1.0 filesrc location=video.mp4 ! decodebin ! videoconvert ! \
  cpudetector model-path=models/yolov5n.onnx ! videoconvert ! autovideosink

# Inspect the plugin
gst-inspect-1.0 cpudetector
```

## Usage

### Main Runtime Demo Application

```bash
# Run the main application (from crates/ds-rs directory)
cargo run --release --bin ds-app -- <video_uri>

# Example with file source
cargo run --release --bin ds-app -- file:///path/to/video.mp4

# Example with RTSP stream
cargo run --release --bin ds-app -- rtsp://camera.local/stream

# Run with debug output and timestamps
RUST_LOG=debug cargo run --release --bin ds-app -- <video_uri>

# Force specific backend
FORCE_BACKEND=standard cargo run --release --bin ds-app -- <video_uri>

# Show help
cargo run --release --bin ds-app -- --help
```

The demo application will:
1. Start with the provided video source
2. Automatically add a new source every 10 seconds (up to 4 sources)
3. After reaching maximum sources, randomly remove sources every 10 seconds
4. Continue until all sources are removed or interrupted with Ctrl+C
5. Show timestamped state changes for debugging

### Example Applications

```bash
# Cross-platform demo with automatic backend detection
cargo run --example cross_platform

# Runtime source management demo
cargo run --example runtime_demo

# Detection application with metadata extraction
cargo run --example detection_app -- file:///path/to/video.mp4

# CPU detection demo (requires ONNX model)
cargo run --example cpu_detection_demo --features cpu_vision,nalgebra
```

### Test Video Generation and RTSP Server

The `source-videos` crate provides comprehensive test infrastructure:

```bash
# Navigate to source-videos crate
cd crates/source-videos

# Start RTSP server with test patterns
cargo run --release -- rtsp

# Generate test video files
cargo run --release -- file --output test.mp4 --pattern ball --duration 10

# Start interactive mode
cargo run --release -- interactive

# List available patterns (25+ options)
cargo run --release -- --help
```

Available test patterns include:
- **Static**: SMPTE, EBU color bars, checkerboard, gradient
- **Animated**: Ball (bouncing), circular, pinwheel
- **Noise**: White noise, random, blink
- **Solid colors**: Red, green, blue, white, black

### Library Usage

```rust
use ds_rs::{init, BackendManager, PlatformInfo, timestamp};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the library
    init()?;
    
    // Get timestamp for logging
    println!("[{:.3}] Starting application", timestamp());
    
    // Detect platform capabilities
    let platform = PlatformInfo::detect()?;
    println!("Platform: {:?}", platform.platform);
    println!("Has NVIDIA Hardware: {}", platform.has_nvidia_hardware());
    
    // Create backend manager (auto-detects best backend)
    let manager = BackendManager::new()?;
    println!("Selected Backend: {}", manager.backend_type().name());
    
    // Check backend capabilities
    let caps = manager.capabilities();
    println!("Supports AI Inference: {}", caps.supports_inference);
    println!("Supports Object Tracking: {}", caps.supports_tracking);
    
    Ok(())
}
```

### Dynamic Source Management

```rust
use ds_rs::{Pipeline, SourceController, timestamp};
use std::sync::Arc;

// Create pipeline and source controller
let pipeline = Arc::new(Pipeline::new("my-pipeline")?);
let streammux = factory.create_stream_mux(Some("mux"))?;
let controller = SourceController::new(pipeline, streammux);

// Add sources dynamically at runtime
let source1 = controller.add_source("file:///path/to/video1.mp4")?;
println!("[{:.3}] Added source: {}", timestamp(), source1);

let source2 = controller.add_source("rtsp://camera.local/stream")?;
println!("[{:.3}] Added source: {}", timestamp(), source2);

// Remove sources without stopping pipeline
controller.remove_source(source1)?;
println!("[{:.3}] Removed source: {}", timestamp(), source1);

// List active sources
for (id, uri, state) in controller.list_active_sources()? {
    println!("[{:.3}] Source {}: {} [{:?}]", timestamp(), id, uri, state);
}
```

## Configuration

The library supports both TOML configuration files and DeepStream's native configuration format:

### TOML Configuration

```toml
[pipeline]
enable = true
width = 1920
height = 1080
batch_size = 4
gpu_id = 0

[sources]
enable = true
uri = "file:///path/to/video.mp4"
num_sources = 1

[sink]
enable = true
sync = false
sink_type = "egl"  # or "file", "fake", "rtsp"
```

### DeepStream Configuration

The library can parse standard DeepStream configuration files:
- `dstest_pgie_config.txt` - Primary inference engine config
- `dstest_tracker_config.txt` - Object tracker config
- Additional inference and metadata configs

## Development Status

### Implemented ✅
- **Core Infrastructure**: Error handling, platform detection, module structure
- **Hardware Abstraction**: Three-tier backend system with auto-detection
- **Pipeline Management**: Builder pattern, state management, bus handling
- **Source Control APIs**: Dynamic source addition/removal at runtime
- **Main Application**: Full demo with timestamped logging
- **CPU Vision Backend**: ONNX-based object detection for YOLOv3-v12
- **Test Infrastructure**: Complete test video generation with RTSP server
- **Enhanced Logging**: Timestamp support for all state changes
- **Configuration System**: TOML and DeepStream format parsing
- **Test Suite**: 200+ tests across all modules

### Current Limitations
- **Float16 Models**: Not supported due to ONNX Runtime lifetime constraints
- **Mock Backend**: Cannot test uridecodebin-based source management (10 tests fail as expected)
- **DeepStream Metadata**: Mock implementations for non-NVIDIA systems
- **Code Quality**: Some unwrap() calls need proper error handling

### Planned Enhancements 📋
- Real-time bounding box rendering on video output
- Multi-stream detection pipeline (4+ concurrent streams)
- Detection data export (MQTT, RabbitMQ, databases)
- Enhanced tracking algorithms (Kalman filter, SORT)
- WebSocket control API for remote management

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output and timestamps
RUST_LOG=debug cargo test -- --nocapture

# Run specific test suite
cargo test --test backend_tests
cargo test --test pipeline_tests
cargo test --test source_management
cargo test --test cpu_backend_tests

# Run tests for specific crate
cd crates/source-videos && cargo test
cd crates/cpuinfer && cargo test

# Note: Some source_management tests fail with Mock backend (expected behavior)
```

### Test Coverage
- **Total Tests**: 200+ across all modules
- **Core Library Tests**: 83 passing
- **CPU Inference Tests**: 10+ passing
- **Source Videos Tests**: 24+ passing
- **Known Limitations**: Mock backend cannot test uridecodebin (10 expected failures)

## Environment Variables

- `CUDA_VER` - Specify CUDA version (e.g., "10.2", "11.4")
- `GPU_ID` - Select GPU device (default: 0)
- `GST_PLUGIN_PATH` - Additional GStreamer plugin paths
- `DS_SDK_ROOT` - DeepStream SDK installation path
- `RUST_LOG` - Set logging level (error, warn, info, debug, trace)
- `FORCE_BACKEND` - Force specific backend (mock, standard, deepstream)

## Troubleshooting

### Common Issues

1. **"DeepStream elements not found"**
   - Ensure DeepStream SDK is installed
   - Set `GST_PLUGIN_PATH` to include DeepStream plugins
   - Verify with: `gst-inspect-1.0 | grep nv`

2. **"ONNX Runtime DLL not found" (Windows)**
   - This warning is normal during build
   - DLLs are downloaded automatically at runtime
   - Ensure internet connection for first run

3. **"Float16 models not currently supported"**
   - Use Float32 YOLO models instead
   - This is a known limitation with current ONNX Runtime integration

4. **Source management test failures**
   - Mock backend doesn't support uridecodebin
   - Use Standard backend for full source management testing
   - This is expected behavior

5. **RTSP server issues**
   - Ensure gstreamer1.0-rtsp-server is installed
   - Check firewall settings for port 8554
   - Verify with: `gst-inspect-1.0 | grep rtsp`

## Contributing

See [TODO.md](TODO.md) for current tasks and priorities. When contributing:

1. Check BUGS.md for known issues
2. Create a feature branch
3. Update TODO.md to mark items in-progress
4. Write tests for new functionality
5. Update documentation as needed
6. Mark complete in TODO.md when merged

## Related Projects

- **gstreamer-rs**: Rust bindings for GStreamer
- **ONNX Runtime**: Machine learning inference acceleration
- **Original C implementation**: [NVIDIA-AI-IOT/deepstream_reference_apps](https://github.com/NVIDIA-AI-IOT/deepstream_reference_apps)

## License

This project is a port of NVIDIA's DeepStream reference applications. Please refer to NVIDIA's licensing terms for DeepStream SDK usage.

## Acknowledgments

- Original C implementation: [NVIDIA-AI-IOT/deepstream_reference_apps](https://github.com/NVIDIA-AI-IOT/deepstream_reference_apps)
- Built with [gstreamer-rs](https://github.com/GStreamer/gstreamer-rs)
- ONNX Runtime for CPU inference capabilities

## Project Status

**Version**: 0.1.0 (Active Development)

This is an active port of NVIDIA's DeepStream reference application to Rust. Core functionality is complete with dynamic source management, cross-platform backend abstraction, and CPU-based object detection working.

**Recent Achievement**: Successfully added timestamp logging to all state changes for improved debugging and monitoring.

**Current Focus**: CPU vision backend with ONNX Runtime integration for object detection. Float32 models are fully supported, with YOLOv3-v12 automatic version detection.