# ds-rs

A Rust port of NVIDIA's DeepStream runtime source addition/deletion reference application, demonstrating dynamic video source management in AI-powered video analytics pipelines.

## Features

- **Cross-Platform Support**: Run on NVIDIA hardware with DeepStream or any system with standard GStreamer
- **Hardware Abstraction**: Automatic backend detection and fallback mechanisms
- **Type-Safe GStreamer Bindings**: Leverages official gstreamer-rs for robust pipeline management
- **Dynamic Source Management**: Add and remove video sources at runtime (in development)
- **Configuration System**: Support for DeepStream configuration files and TOML-based settings

## Architecture

### Backend System

The project features a flexible backend system that automatically detects and uses the best available video processing backend:

1. **DeepStream Backend**: Full NVIDIA hardware acceleration with AI inference
2. **Standard Backend**: Software-based processing using standard GStreamer elements
3. **Mock Backend**: Testing and development without any hardware dependencies

### Project Structure

```
ds-rs/
├── src/
│   ├── backend/        # Backend implementations and detection
│   ├── config/         # Configuration parsing and management
│   ├── elements/       # GStreamer element abstractions
│   ├── error.rs        # Error handling
│   ├── platform.rs     # Platform detection (Jetson/x86)
│   └── main.rs         # Application entry point
├── examples/           # Usage examples
├── tests/              # Integration tests
├── PRPs/               # Project planning documents
└── vendor/             # Reference C implementation
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
cd ds-rs

# Build the project
cargo build --release

# Run tests
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
# Run the main application
cargo run --release

# Run with debug output
RUST_LOG=debug cargo run --release
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
- Core infrastructure and error handling
- Platform detection (Jetson/x86/Unknown)
- Hardware abstraction layer with three backends
- Element factory with backend-aware creation
- Configuration system (TOML and DeepStream formats)
- Basic pipeline element creation
- Comprehensive test suite (29 tests)

### In Progress 🚧
- Pipeline builder pattern (PRP-02)
- Runtime source addition/deletion (PRP-03)
- DeepStream metadata extraction (PRP-04)
- Complete demo application (PRP-05)

### Planned 📋
- Integration tests with video files
- Performance benchmarking
- CI/CD pipeline
- Documentation improvements
- Additional examples

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_backend_detection

# Run backend tests
cargo test --test backend_tests
```

## Environment Variables

- `CUDA_VER` - Specify CUDA version (e.g., "10.2", "11.4")
- `GPU_ID` - Select GPU device (default: 0)
- `GST_PLUGIN_PATH` - Additional GStreamer plugin paths
- `DS_SDK_ROOT` - DeepStream SDK installation path
- `RUST_LOG` - Set logging level (error, warn, info, debug, trace)

## Contributing

See [PRPs/](PRPs/) directory for project planning documents and contribution guidelines.

## License

This project is a port of NVIDIA's DeepStream reference applications. Please refer to NVIDIA's licensing terms for DeepStream SDK usage.

## Acknowledgments

- Original C implementation: [NVIDIA-AI-IOT/deepstream_reference_apps](https://github.com/NVIDIA-AI-IOT/deepstream_reference_apps)
- Built with [gstreamer-rs](https://github.com/GStreamer/gstreamer-rs)

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

4. **Cross-platform example fails**
   - Known issue with compositor property types (fix in progress)
   - Use mock backend for testing: `cargo run --example cross_platform mock`

## Project Status

This is an active port of NVIDIA's DeepStream reference application to Rust. The core infrastructure is complete, with pipeline management and dynamic source control features under development. The project emphasizes cross-platform compatibility, allowing development and testing without specialized hardware.