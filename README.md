# ds-rs

A Rust port of NVIDIA's DeepStream runtime source addition/deletion reference application, demonstrating dynamic video source management in AI-powered video analytics pipelines.

## Features

- **Cross-Platform Support**: Run on NVIDIA hardware with DeepStream or any system with standard GStreamer
- **Hardware Abstraction**: Automatic backend detection and fallback mechanisms
- **Type-Safe GStreamer Bindings**: Leverages official gstreamer-rs for robust pipeline management
- **Dynamic Source Management**: Add and remove video sources at runtime without pipeline interruption
- **Configuration System**: Support for DeepStream configuration files and TOML-based settings
- **Pipeline Builder**: Fluent API for constructing complex GStreamer pipelines

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
│   ├── ds-rs/          # Main library and application
│   │   ├── src/
│   │   │   ├── backend/        # Backend implementations and detection
│   │   │   ├── config/         # Configuration parsing and management
│   │   │   ├── elements/       # GStreamer element abstractions
│   │   │   ├── pipeline/       # Pipeline builder and state management
│   │   │   ├── source/         # Dynamic source management
│   │   │   ├── error.rs        # Error handling
│   │   │   ├── platform.rs     # Platform detection (Jetson/x86)
│   │   │   ├── lib.rs          # Library entry point
│   │   │   └── main.rs         # Application entry point
│   │   ├── examples/           # Usage examples
│   │   └── tests/              # Integration tests
│   └── dsl/            # DeepStream Services Library (future)
├── PRPs/               # Project planning documents
├── vendor/             # Reference C implementation
└── CLAUDE.md           # AI assistant guidance
```

## Installation

### Prerequisites

#### For NVIDIA DeepStream Support
- NVIDIA DeepStream SDK 6.0+
- CUDA 10.2 (Jetson) or 11.4+ (x86)
- TensorRT 8.0+
- GStreamer 1.14+

#### For Standard GStreamer Support
- GStreamer 1.14+ with base, good, and bad plugins
- Rust 1.70+

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/ds-rs.git
cd ds-rs/crates/ds-rs

# Build the project
cargo build --release

# Run tests (currently 67 tests)
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

## Usage

### Basic Application

```bash
# Run the main application (from crates/ds-rs directory)
cargo run --release --bin ds-app

# Run with debug output
RUST_LOG=debug cargo run --release --bin ds-app
```

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

// Set enum properties using strings
let pipeline = PipelineBuilder::new("test")
    .add_element("source", "videotestsrc")
    .set_property_from_str("source", "pattern", "smpte")  // Enum property
    .set_property("source", "num-buffers", 100i32)        // Regular property
    .build()?;
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
  - Thread-safe source registry with unique IDs
  - VideoSource wrapper for uridecodebin elements
  - Pad-added signal handling for dynamic linking
  - Per-source EOS tracking and event system
  - High-level SourceController API
- **Configuration System**: TOML and DeepStream format parsing
- **Test Suite**: 67 tests across all modules

### In Progress 🚧
- **Main Application** (PRP-05): Full demo matching C reference
- **DeepStream Metadata** (PRP-04): AI inference result extraction

### Planned 📋
- Integration tests with actual video files
- Test RTSP source for better integration testing
- Performance benchmarking
- CI/CD pipeline with GitHub Actions
- Documentation improvements
- Additional examples

## Testing

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

# Note: Some source_management tests may fail with Mock backend
# This is expected - use Standard backend for full testing
```

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

## Contributing

See [PRPs/](PRPs/) directory for project planning documents and contribution guidelines.

When contributing:
1. Create a feature branch
2. Update TODO.md to mark items in-progress
3. Write tests for new functionality
4. Update documentation as needed
5. Mark complete in TODO.md when merged

## License

This project is a port of NVIDIA's DeepStream reference applications. Please refer to NVIDIA's licensing terms for DeepStream SDK usage.

## Acknowledgments

- Original C implementation: [NVIDIA-AI-IOT/deepstream_reference_apps](https://github.com/NVIDIA-AI-IOT/deepstream_reference_apps)
- Built with [gstreamer-rs](https://github.com/GStreamer/gstreamer-rs)

## Project Status

This is an active port of NVIDIA's DeepStream reference application to Rust. The core infrastructure, pipeline management, and dynamic source control are complete. The project emphasizes cross-platform compatibility, allowing development and testing without specialized hardware.

**Current Focus**: Implementing the main application demo (PRP-05) to showcase the complete functionality of dynamic source management in video analytics pipelines.