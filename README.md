# ds-rs

A Rust port of NVIDIA's DeepStream runtime source addition/deletion reference application, demonstrating dynamic video source management in AI-powered video analytics pipelines error recovery and fault tolerance.

## Recent Updates

### 2025-01-25: Directory and File List Support Complete (PRP-35)
- ✅ **COMPLETED: Full directory serving** - Serve all video files from directories as RTSP streams
- ✅ **ADDED: Recursive directory traversal** - Process nested video collections with configurable depth
- ✅ **IMPLEMENTED: File list support** - Accept explicit lists of files via CLI or config
- ✅ **CREATED: Auto-detection of formats** - uridecodebin automatically handles mp4, mkv, avi, webm
- ✅ **BUILT: Mount point generation** - Predictable RTSP URLs from file paths with URL encoding
- ✅ **ADDED: Filter system** - Include/exclude patterns and extension filtering
- ✅ **CREATED: CLI integration** - Complete --directory, --files, --recursive, --include/--exclude flags

### 2025-08-24: Fault-Tolerant Source Controller Integration (PRP-12 Integration)
- ✅ **INTEGRATED: Error recovery with SourceController** - Automatic reconnection for failed sources
- ✅ **ADDED: FaultTolerantSourceController wrapper** - Simple, robust fault tolerance without complexity
- ✅ **CREATED: Automatic error handling** - Sources automatically retry on transient failures
- ✅ **IMPLEMENTED: Circuit breaker integration** - Prevents cascade failures across sources
- ✅ **ADDED: Per-source recovery tracking** - Independent recovery policies for each stream
- ✅ **CREATED: fault_tolerant_multi_stream example** - Demonstrates production-ready multi-stream recovery

### 2025-08-24: Network Simulation for Error Recovery Testing (PRP-19)
- ✅ **IMPLEMENTED: Network simulation capabilities** - Test error recovery with realistic network conditions
- ✅ **ADDED: Packet loss simulation** - Random and burst patterns with configurable rates
- ✅ **CREATED: Connection drop simulation** - Test reconnection and recovery mechanisms
- ✅ **BUILT: Network profiles** - Predefined conditions (3G, 4G, WiFi, Satellite, etc.)
- ✅ **INTEGRATED: GStreamer pipeline integration** - Transparent insertion of simulation elements
- ✅ **ADDED: RTSP server integration** - Simulate network issues in video streaming
- ✅ **CREATED: Examples and tests** - Comprehensive testing infrastructure for error recovery

### 2025-08-24: Enhanced Error Recovery and Fault Tolerance (PRP-34)
- ✅ **IMPLEMENTED: Comprehensive error recovery system** - Production-ready fault tolerance mechanisms
- ✅ **ADDED: Exponential backoff with jitter** - Smart retry strategies prevent thundering herd
- ✅ **CREATED: Circuit breaker pattern** - Prevents cascade failures with automatic recovery testing
- ✅ **INTEGRATED: Health monitoring** - Proactive detection of degraded sources with frame rate tracking
- ✅ **BUILT: Error classification system** - Distinguishes transient vs permanent failures for appropriate recovery
- ✅ **ADDED: Stream isolation** - Error boundaries prevent single source failures from affecting pipeline
- ✅ **IMPLEMENTED: Recovery statistics** - Track success rates, downtime, and recovery patterns
- ✅ **CREATED: fault_tolerant_pipeline example** - Demonstrates all recovery features in action

### 2025-08-24: Completed Main Application Demo (PRP-05)
- ✅ **IMPLEMENTED: Timer-based source management** - Automatically adds sources every 10 seconds
- ✅ **ADDED: Source deletion timer** - Removes sources randomly after reaching MAX_NUM_SOURCES
- ✅ **INTEGRATED: GLib timers** - Matching C reference implementation behavior
- ✅ **COMPLETED: Full runtime demo** - Application now demonstrates dynamic source addition/deletion

### 2025-08-24: Fixed Video Playback State Management (PRP-03)
- ✅ **FIXED: Pipeline now properly reaches PLAYING state** - Video window appears and displays content
- ✅ **Corrected initialization order** - Sources are now added before pipeline transitions to PAUSED
- ✅ **Implemented proper async state handling** - Pipeline waits for state changes to complete
- ✅ **Fixed dynamic element synchronization** - Using sync_state_with_parent() for proper clock inheritance
- ✅ **Added comprehensive state validation** - Detailed logging of all element states for debugging

### 2025-08-24: Real-time Bounding Box Rendering (PRP-11)
- ✅ **Implemented real-time bounding box visualization** - Dynamic rendering of detection results
- ✅ **Created cross-backend rendering system** - Works with DeepStream (nvdsosd) and Standard (Cairo/text overlay)
- ✅ **Added metadata bridge** - Connects inference results to OSD rendering pipeline
- ✅ **Enhanced pipeline builder** - Dynamic OSD configuration with rendering presets
- ✅ **Created ball tracking example** - Demonstrates real-time object tracking with visual feedback

### 2025-08-24: Critical Bug Fixes and Improvements
- ✅ **Fixed f16/f32 array conversion issue** in cpuinfer - Resolved lifetime issues with ONNX tensor arrays
- ✅ **Fixed Application test compilation errors** - Updated tests to reflect correct API methods
- ✅ **Fixed video playback freezing** - Added videorate and capsfilter for framerate normalization
- ✅ **Enhanced timestamp logging** - All state changes now show precise timing for debugging

### 2025-08-23: Major Milestone Achievements
- ✅ Fixed application shutdown handling with proper GLib signal integration
- ✅ Resolved ONNX Runtime API compatibility (v1.16.3)
- ✅ Fixed ONNX Runtime DLL loading on Windows
- ✅ Added comprehensive DLL validation module

## Features

- **Cross-Platform Support**: Run on NVIDIA hardware with DeepStream or any system with standard GStreamer
- **Hardware Abstraction**: Automatic backend detection and fallback mechanisms
- **Type-Safe GStreamer Bindings**: Leverages official gstreamer-rs for robust pipeline management
- **Dynamic Source Management**: Add and remove video sources at runtime without pipeline interruption
- **CPU Object Detection**: Custom GStreamer plugin with ONNX Runtime support for YOLOv3-v12
- **Real-time Bounding Box Rendering**: Visual feedback showing detected objects with configurable styles
- **Production-Grade Error Recovery**: Exponential backoff, circuit breakers, and health monitoring
- **Stream Isolation**: Error boundaries prevent cascade failures across sources
- **Configuration System**: Support for DeepStream configuration files and TOML-based settings
- **Pipeline Builder**: Fluent API for constructing complex GStreamer pipelines with dynamic OSD
- **Comprehensive Test Infrastructure**: Self-contained testing with RTSP server and video generation

## Architecture

### Backend System

The project features a flexible backend system that automatically detects and uses the best available video processing backend:

1. **DeepStream Backend**: Full NVIDIA hardware acceleration with AI inference
2. **Standard Backend**: Software-based processing using standard GStreamer elements with CPU vision
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
│   │   │   ├── dll_validator.rs # Windows DLL validation
│   │   │   ├── error.rs    # Error handling
│   │   │   ├── platform.rs # Platform detection (Jetson/x86)
│   │   │   ├── lib.rs      # Library entry point
│   │   │   └── main.rs     # Application entry point
│   │   ├── models/         # YOLO ONNX models
│   │   ├── examples/       # Usage examples (4 demos)
│   │   └── tests/          # Integration tests (6 test suites)
│   ├── cpuinfer/           # GStreamer CPU inference plugin
│   │   ├── src/
│   │   │   ├── detector.rs # ONNX detector with f16/f32 support
│   │   │   └── cpudetector/ # GStreamer element implementation
│   ├── source-videos/      # Test video generation and RTSP server
│   │   ├── src/
│   │   │   ├── config/     # Configuration management
│   │   │   ├── file.rs     # File generation (MP4, MKV, WebM)
│   │   │   ├── manager.rs  # Source management
│   │   │   ├── patterns.rs # 25+ test patterns (SMPTE, ball, etc.)
│   │   │   ├── pipeline/   # Pipeline builders
│   │   │   ├── rtsp/       # RTSP server implementation
│   │   │   └── runtime/    # Runtime configuration
│   │   └── tests/
│   └── dsl/                # DeepStream Services Library (future)
├── scripts/                # Test orchestration scripts
├── TODO.md                 # Current task tracking
├── BUGS.md                 # Known issues and bug tracking
├── CLAUDE.md               # AI assistant guidance
└── codebase-review-report.md # Code quality assessment
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

#### Windows-Specific Requirements
- **Visual C++ Redistributables**: Required for ONNX Runtime
  - Download from: https://aka.ms/vs/17/release/vc_redist.x64.exe
  - Both MSVCP140.dll and VCRUNTIME140.dll must be present in System32
- **DLL Setup**: The build script automatically copies ONNX Runtime DLLs to the correct locations

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/ds-rs.git
cd ds-rs

# Build all workspace members
cargo build --release

# Build with CPU vision features (includes ONNX support)
cargo build --release --features cpu_vision,nalgebra,half,ort

# Run tests (137 passing, 10 expected failures with Mock backend)
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
- **Float16/Float32 Support**: Full support for both half-precision (f16) and full precision (f32) models
- **Automatic Type Conversion**: Handles f16/f32 conversion based on model requirements
- **Passthrough Architecture**: Identity element behavior with signal emission

### Building the Plugin

```bash
# Build with ONNX support (default)
cd crates/cpuinfer
cargo build --release --features half,ort

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
cargo run --example cpu_detection_demo --features cpu_vision,nalgebra,half

# Fault-tolerant pipeline with error recovery
cargo run --example fault_tolerant_pipeline

# Multi-stream fault tolerance with automatic recovery (NEW)
cargo run --example fault_tolerant_multi_stream
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
- **Float16 Support**: Full f16/f32 conversion and model compatibility
- **Test Suite**: 137+ passing tests across all modules

### Current Limitations
- **Mock Backend**: Cannot test uridecodebin-based source management (10 tests fail as expected)
- **DeepStream Metadata**: Mock implementations for non-NVIDIA systems
- **Code Quality**: Some unwrap() calls need proper error handling (100+ occurrences)
- **Placeholder Implementations**: 15+ locations with "for now" comments

### Planned Enhancements 📋
- Real-time bounding box rendering on video output
- Multi-stream detection pipeline (4+ concurrent streams)
- Detection data export (MQTT, RabbitMQ, databases)
- Enhanced tracking algorithms (Kalman filter, SORT)
- WebSocket control API for remote management
- Additional inference backends (TensorFlow Lite, OpenVINO, TensorRT)

## Testing

### Running Tests

```bash
# Run all tests (use -j 1 to avoid memory issues on Windows)
cargo test --features cpu_vision,nalgebra,half,ort -j 1

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
- **Total Tests**: 147 tests across all modules
- **Passing Tests**: 137 tests
- **Expected Failures**: 10 source_management tests with Mock backend
- **Test Suites**:
  - Core Library Tests: 90 passing
  - CPU Inference Tests: 10 passing  
  - Backend Tests: 9 passing
  - CPU Backend Tests: 10 passing
  - Pipeline Tests: 13 passing
  - Shutdown Tests: 2 passing
  - Source Management: 3 passing, 10 expected failures with Mock

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
   - DLLs are downloaded automatically by the ort crate
   - The build script copies them to target directories

### Windows-Specific Issues

1. **Error 0xc000007b when running examples**
   - This indicates a DLL loading issue
   - **Solution 1**: Ensure Visual C++ Redistributables are installed
     - Download: https://aka.ms/vs/17/release/vc_redist.x64.exe
   - **Solution 2**: Run `cargo clean && cargo build --features ort`
     - This triggers the build script to copy DLLs properly
   - **Solution 3**: Set ORT_DYLIB_PATH to ONNX Runtime location
     - `set ORT_DYLIB_PATH=C:\path\to\onnxruntime.dll`

2. **Memory allocation failures during build**
   - Use `-j 1` flag to limit parallel compilation
   - Example: `cargo test -j 1`
   - This prevents excessive memory usage on Windows

3. **DLLs not found in examples directory**
   - The build.rs script should copy DLLs automatically
   - Check build output for "Successfully copied" messages
   - Manual fix: Copy from `target\debug\` to `target\debug\examples\`

4. **Model precision mismatch**
   - Both Float16 and Float32 models are now supported
   - The detector automatically handles conversion between f16 and f32 as needed
   - Arrays are properly managed with correct lifetimes

5. **Source management test failures**
   - Mock backend doesn't support uridecodebin
   - Use Standard backend for full source management testing
   - This is expected behavior documented in CLAUDE.md

6. **RTSP server issues**
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

### Recent Contributors
- Fixed critical f16/f32 conversion issues
- Resolved application test compilation errors
- Enhanced Windows DLL handling
- Improved test stability

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

**Recent Achievements** (2025-08-24):
- ✅ Fixed critical f16/f32 array lifetime issues in ONNX inference
- ✅ Resolved Application test API mismatches
- ✅ Enhanced Windows build with automatic DLL management
- ✅ Stabilized test suite with 137 passing tests

**Current Focus**: Improving test coverage and resolving placeholder implementations. The CPU vision backend with ONNX Runtime is fully functional for YOLOv3-v12 models with automatic version detection and f16/f32 support.
