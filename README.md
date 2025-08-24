# ds-rs

A Rust port of NVIDIA's DeepStream runtime source addition/deletion reference application, demonstrating dynamic video source management in AI-powered video analytics pipelines.

## Features

- **Cross-Platform Support**: Run on NVIDIA hardware with DeepStream or any system with standard GStreamer
- **Hardware Abstraction**: Automatic backend detection and fallback mechanisms
- **Type-Safe GStreamer Bindings**: Leverages official gstreamer-rs for robust pipeline management
- **Dynamic Source Management**: Add and remove video sources at runtime without pipeline interruption
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
│   │   ├── examples/       # Usage examples
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
├── PRPs/                   # Project planning documents (15 total)
├── vendor/                 # Reference C implementation
├── TODO.md                 # Current task tracking
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

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/ds-rs.git
cd ds-rs

# Build all workspace members
cargo build --release

# Build specific crate
cd crates/ds-rs
cargo build --release

# Run tests (95+ total tests)
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
- **Optional Backends**: ONNX (default), OpenCV DNN, or mock detection
- **Float16/Float32 Support**: Automatic tensor type conversion
- **Passthrough Architecture**: Identity element behavior with signal emission

### Building the Plugin

```bash
# Build with ONNX support (default)
cd crates/cpuinfer
cargo build --release

# Build without ONNX (lightweight)
cargo build --release --no-default-features

# Install the plugin (Windows example)
copy target\release\gstcpuinfer.dll %GSTREAMER_1_0_ROOT_X86_64%\lib\gstreamer-1.0\
```

### Using the Plugin

```bash
# Basic pipeline with CPU detection
gst-launch-1.0 filesrc location=video.mp4 ! decodebin ! videoconvert ! \
  cpudetector model-path=models/yolov5n.onnx ! videoconvert ! autovideosink
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

# Run with debug output
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

### Cross-Platform Example

```bash
# Run with automatic backend detection
cargo run --example cross_platform

# Force specific backend
cargo run --example cross_platform mock
cargo run --example cross_platform standard
cargo run --example cross_platform deepstream  # Requires NVIDIA hardware
```

### Library Usage

```rust
use ds_rs::{init, BackendManager, PlatformInfo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the library
    init()?;
    
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

### Creating Elements with Backend Abstraction

```rust
use ds_rs::elements::factory::ElementFactory;
use std::sync::Arc;

// Create element factory with automatic backend
let manager = Arc::new(BackendManager::new()?);
let factory = ElementFactory::new(manager);

// Create elements (automatically uses appropriate backend)
let mux = factory.create_stream_mux(Some("muxer"))?;
let convert = factory.create_video_convert(Some("converter"))?;
let sink = factory.create_video_sink(Some("display"))?;
```

### Dynamic Source Management

```rust
use ds_rs::{Pipeline, SourceController};
use std::sync::Arc;

// Create pipeline and source controller
let pipeline = Arc::new(Pipeline::new("my-pipeline")?);
let streammux = factory.create_stream_mux(Some("mux"))?;
let controller = SourceController::new(pipeline, streammux);

// Add sources dynamically at runtime
let source1 = controller.add_source("file:///path/to/video1.mp4")?;
let source2 = controller.add_source("rtsp://camera.local/stream")?;

// Remove sources without stopping pipeline
controller.remove_source(source1)?;

// List active sources
for (id, uri, state) in controller.list_active_sources()? {
    println!("Source {}: {} [{:?}]", id, uri, state);
}

// Enable automatic removal on EOS
let mut controller = SourceController::new(pipeline, streammux);
controller.enable_auto_remove_on_eos(true);
```

### Building Pipelines with Fluent API

```rust
use ds_rs::{Pipeline, PipelineBuilder, BackendType};
use gstreamer_rs::prelude::*;

// Build a pipeline using the fluent API
let pipeline = Pipeline::builder("my-pipeline")
    .backend(BackendType::Standard)  // or DeepStream, Mock
    .add_test_source("source")
    .add_queue("queue")
    .add_element("convert", "videoconvert")
    .add_auto_sink("sink")
    .link_many(vec!["source", "queue", "convert", "sink"])
    .build()?;

// Control pipeline state
pipeline.play()?;
// ... do processing ...
pipeline.stop()?;

// Set properties using gstreamer-rs
let source = pipeline.by_name("source").unwrap();
source.set_property_from_str("pattern", "smpte");  // Enum property
source.set_property("num-buffers", 100i32);        // Regular property
```

### Metadata Extraction and AI Inference

```rust
use ds_rs::{MetadataExtractor, ObjectTracker, InferenceProcessor};
use gstreamer::prelude::*;

// Create metadata extractor
let extractor = MetadataExtractor::new();
let tracker = ObjectTracker::new(100, 30, 50);

// Add probe to extract metadata from buffers
let pad = element.static_pad("sink").unwrap();
pad.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
    if let Some(buffer) = info.buffer() {
        // Extract batch metadata
        if let Ok(batch_meta) = extractor.extract_batch_meta(buffer) {
            // Process each frame
            for frame in batch_meta.frames() {
                println!("Frame from source {}: {} objects detected", 
                    frame.source_id, frame.num_objects());
                
                // Process detected objects
                for object in frame.objects() {
                    println!("  {} at ({:.0},{:.0}) confidence: {:.2}",
                        object.class_name(),
                        object.rect_params.left,
                        object.rect_params.top,
                        object.confidence
                    );
                    
                    // Track object
                    if object.is_tracked() {
                        tracker.update_track(object.object_id, &object, timestamp)?;
                    }
                }
            }
        }
    }
    gst::PadProbeReturn::Ok
});
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
- **Core Infrastructure** (PRP-01): Error handling, platform detection, module structure
- **Hardware Abstraction** (PRP-06): Three-tier backend system with auto-detection
- **Pipeline Management** (PRP-02): Builder pattern, state management, bus handling
- **Source Control APIs** (PRP-03): Dynamic source addition/removal at runtime
- **Main Application** (PRP-05): Full demo matching C reference implementation
- **DeepStream Metadata** (PRP-04): AI inference result extraction and tracking
- **Dynamic Video Sources** (PRP-07): Complete test infrastructure with RTSP server
- **Code Quality Improvements** (PRP-08): Error handling enhancements
- **CPU Vision Backend** (PRP-20): Foundation for CPU-based object detection and tracking
- **Configuration System**: TOML and DeepStream format parsing
- **Test Suite**: 95+ tests with 88.8% pass rate

### Planned Enhancements 📋

#### Computer Vision & Object Detection (PRPs 10-13)
- Ball Detection Integration with OpenCV
- Real-time Bounding Box Rendering
- Multi-Stream Detection Pipeline (4+ concurrent streams)
- Detection Data Export (MQTT, RabbitMQ, databases)

#### Backend Integration & Element Discovery (PRPs 14-15)
- Enhanced backend integration (PRP-14)
- Simplified GStreamer element discovery (PRP-15)
- Leveraging gstreamer-rs existing capabilities
- Compile-time element discovery for better backend detection

#### Test Orchestration (PRP-09)
- Cross-platform test orchestration scripts
- End-to-end testing with configurable scenarios
- CI/CD integration with GitHub Actions

### Known Issues 🐛

1. **Source Management Tests with Mock Backend**
   - 10 tests fail when using Mock backend
   - Reason: Mock backend doesn't support uridecodebin
   - Workaround: Use Standard backend for full testing

2. **Code Quality**
   - 237 unwrap() calls need error handling improvements
   - 2 panic!() calls in test code to be replaced
   - Some GStreamer property type issues in source-videos

3. **Build Configuration**
   - Workspace manifest uses resolver "3" and edition "2024"
   - Some unused workspace manifest keys

## Testing

### Automated Test Orchestration

The project includes comprehensive test orchestration scripts for automated end-to-end testing:

```bash
# Validate environment (check all dependencies)
python scripts/validate-environment.py

# Run all tests with orchestration
python scripts/test-orchestrator.py --scenario all

# Run specific test scenarios
python scripts/test-orchestrator.py --scenario unit        # Unit tests only
python scripts/test-orchestrator.py --scenario integration  # Integration with RTSP
python scripts/test-orchestrator.py --scenario e2e         # End-to-end tests
python scripts/test-orchestrator.py --scenario quick       # Quick smoke tests

# List available test scenarios
python scripts/test-orchestrator.py --list

# Platform-specific orchestrators
./scripts/test-orchestrator.sh unit      # Linux/macOS
.\scripts\test-orchestrator.ps1 -Scenario unit  # Windows PowerShell

# Run with verbose output
python scripts/test-orchestrator.py --scenario integration --verbose
```

#### Test Scenarios

- **quick**: Fast smoke tests (format, clippy, build check)
- **unit**: Unit tests for all crates
- **integration**: Integration tests with RTSP server
- **e2e**: End-to-end tests with real video sources
- **backend-mock**: Tests using Mock backend
- **backend-standard**: Tests using Standard GStreamer backend
- **backend-deepstream**: Tests using NVIDIA DeepStream (requires hardware)
- **all**: Run all applicable test scenarios

The test orchestrator automatically:
- Manages RTSP server lifecycle
- Generates test video files
- Handles backend selection
- Provides detailed logging and reporting
- Cleans up resources on completion

### Manual Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test suite
cargo test --test backend_tests
cargo test --test pipeline_tests
cargo test --test source_management

# Run specific test
cargo test test_video_source_creation

# Run tests for source-videos crate
cd crates/source-videos
cargo test

# Note: Some source_management tests may fail with Mock backend
# This is expected - use Standard backend for full testing
```

### Test Coverage
- **Total Tests**: 107+ across all modules
- **Pass Rate**: 88.8% (95/107 passing)
- **Known Failures**: 10 Mock backend limitations, 2 GStreamer property issues

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

2. **"CUDA version mismatch"**
   - Set correct `CUDA_VER` environment variable
   - Ensure CUDA toolkit matches DeepStream requirements

3. **"Pipeline state change failed"**
   - Check element compatibility
   - Verify all required configuration files are present
   - Enable debug logging: `RUST_LOG=debug`

4. **Source management test failures**
   - Mock backend doesn't support uridecodebin
   - Use Standard backend for full source management testing
   - This is expected behavior

5. **RTSP server issues**
   - Ensure gstreamer1.0-rtsp-server is installed
   - Check firewall settings for port 8554
   - Verify with: `gst-inspect-1.0 | grep rtsp`

## CI/CD

The project uses GitHub Actions for continuous integration and testing:

### Automated Workflows

- **Test Orchestration**: Runs on push and pull requests to main branches
  - Quick checks (format, clippy, build)
  - Unit tests on multiple platforms (Linux, Windows, macOS)
  - Integration tests with RTSP server
  - End-to-end tests with video sources
  - Backend-specific testing

### Running CI Locally

```bash
# Install act (GitHub Actions runner)
# https://github.com/nektos/act

# Run CI workflow locally
act -j unit-tests
act -j integration-tests

# Run with specific event
act pull_request
```

## Contributing

See [PRPs/](PRPs/) directory for project planning documents. Currently 23 PRPs available:
- 9 completed (PRPs 01-08, 14-16: Core infrastructure, test orchestration)
- 3 in progress (PRPs 20-22: CPU Vision Backend)
- 11 ready for implementation (Computer vision, detection, tracking)

When contributing:
1. Create a feature branch
2. Update TODO.md to mark items in-progress
3. Write tests for new functionality
4. Update documentation as needed
5. Mark complete in TODO.md when merged

## Related Projects

- **gstreamer-rs**: Located in `../gstreamer-rs`, provides excellent Rust bindings for GStreamer
- **Original C implementation**: [NVIDIA-AI-IOT/deepstream_reference_apps](https://github.com/NVIDIA-AI-IOT/deepstream_reference_apps)

## License

This project is a port of NVIDIA's DeepStream reference applications. Please refer to NVIDIA's licensing terms for DeepStream SDK usage.

## Acknowledgments

- Original C implementation: [NVIDIA-AI-IOT/deepstream_reference_apps](https://github.com/NVIDIA-AI-IOT/deepstream_reference_apps)
- Built with [gstreamer-rs](https://github.com/GStreamer/gstreamer-rs)

## Project Status

This is an active port of NVIDIA's DeepStream reference application to Rust. The core functionality is complete with all 7 initial PRPs implemented:

- ✅ Dynamic source management working
- ✅ Cross-platform backend abstraction operational
- ✅ AI metadata extraction functional
- ✅ Main demo application complete
- ✅ Comprehensive test infrastructure with RTSP server
- ✅ 95+ tests with 88.8% pass rate

**Current Focus**: Completed CPU Vision Backend foundation (PRP-20). Next priorities include full ONNX Runtime integration (PRP-21), enhanced tracking algorithms (PRP-22), and production readiness improvements.

**Version**: 0.1.0 (Pre-release)