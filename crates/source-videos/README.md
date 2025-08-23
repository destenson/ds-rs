# Source Videos

Dynamic video source generation infrastructure for testing ds-rs DeepStream applications.

## Overview

The `source-videos` crate provides a comprehensive solution for generating test video sources that can be used to test dynamic source management capabilities without requiring actual video files or camera feeds. It supports multiple source types including test patterns, file generation, and RTSP streaming.

## Features

- **Test Pattern Generation**: 25+ built-in test patterns (SMPTE, ball animation, noise, etc.)
- **RTSP Server**: Embedded RTSP server serving multiple test streams simultaneously
- **File Generation**: Generate test video files in common formats (MP4, MKV, WebM)
- **Dynamic Source Management**: Runtime addition/removal of video sources
- **CLI Application**: User-friendly command-line interface
- **Configuration Support**: TOML-based configuration files
- **Cross-Platform**: Works on Windows, Linux, and macOS

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
source-videos = { path = "crates/source-videos" }
```

### Basic Usage

```rust
use source_videos::{SourceVideos, VideoSourceConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize GStreamer
    source_videos::init()?;
    
    // Create a source videos instance
    let mut sv = SourceVideos::new()?;
    
    // Add test patterns
    sv.add_test_pattern("test1", "smpte")?;
    sv.add_test_pattern("test2", "ball")?;
    
    // Start RTSP server
    sv.start_rtsp_server(8554)?;
    
    // List available streams
    for url in sv.get_rtsp_urls() {
        println!("Stream: {}", url);
    }
    
    Ok(())
}
```

### CLI Usage

```bash
# List available test patterns
cargo run -- list

# Start RTSP server with default test patterns
cargo run -- serve --port 8554

# Start server with specific patterns
cargo run -- serve --patterns smpte,ball,snow

# Generate a test video file
cargo run -- generate --pattern smpte --duration 10 --output test.mp4

# Interactive mode
cargo run -- interactive

# Run comprehensive test suite
cargo run -- test --port 8554
```

## Test Patterns

The crate supports 25+ test patterns based on GStreamer's `videotestsrc`:

### Standard Patterns
- `smpte` - SMPTE 100% color bars (broadcast standard)
- `smpte75` - SMPTE 75% color bars
- `snow` - Random noise pattern
- `black`, `white`, `red`, `green`, `blue` - Solid colors

### Animated Patterns
- `ball` - Moving ball animation (great for motion tracking)
- `bar` - Horizontal bar moving vertically
- `blink` - Blinking black/white pattern

### Testing Patterns
- `checkers1`, `checkers2`, `checkers4`, `checkers8` - Checkerboard patterns
- `zone-plate` - Zone plate for frequency response testing
- `gradient` - Color gradient for testing color depth

Use `cargo run -- list` to see all available patterns with descriptions.

## Configuration

Create a `config.toml` file:

```toml
log_level = "info"

[server]
port = 8554
address = "0.0.0.0"
max_connections = 100

[[sources]]
name = "test1"
type = "test_pattern"
pattern = "smpte"

[sources.resolution]
width = 1920
height = 1080

[sources.framerate]
numerator = 30
denominator = 1

format = "i420"
is_live = true
```

Then run with configuration:

```bash
cargo run -- serve --config config.toml
```

## Runtime Configuration Management

The crate now supports dynamic configuration updates without restart:

### File Monitoring

Automatically detect and apply configuration changes:

```rust
use source_videos::{RuntimeManager, config::ConfigWatcher};

// Set up file watcher
let mut watcher = ConfigWatcher::new("config.toml")?
    .with_debounce(Duration::from_millis(500));
watcher.start().await?;

// Handle configuration changes
while let Some(event) = watcher.recv().await {
    match event {
        ConfigEvent::Modified(path) => {
            let new_config = AppConfig::from_file(&path)?;
            runtime.apply_config(new_config).await?;
        }
        _ => {}
    }
}
```

### Runtime Updates

Apply configuration changes dynamically:

```rust
use source_videos::{RuntimeManager, VideoSourceManager};
use std::sync::Arc;

let manager = Arc::new(VideoSourceManager::new());
let runtime = RuntimeManager::new(manager, initial_config);

// Subscribe to events
let mut events = runtime.subscribe_events();

// Apply new configuration
let new_config = AppConfig::from_file("updated.toml")?;
runtime.apply_config(new_config).await?;

// Update individual sources
runtime.update_source("test1", new_source_config).await?;

// Rollback if needed
runtime.rollback().await?;
```

### Signal Handling

On Unix systems, reload configuration with SIGHUP:

```bash
# Send reload signal
kill -HUP <pid>
```

In code:

```rust
use source_videos::runtime::signal_handler::setup_signal_handlers;

let mut signals = setup_signal_handlers().await?;
while let Some(signal) = signals.recv().await {
    match signal {
        SignalEvent::Reload => {
            // Reload configuration
            let config = AppConfig::from_file("config.toml")?;
            runtime.apply_config(config).await?;
        }
        SignalEvent::Shutdown => break,
    }
}
```

### Configuration Validation

All configuration changes are validated before applying:

- Resolution constraints: 160x120 to 7680x4320 (8K)
- Framerate limits: 1-120 fps
- Source name uniqueness
- RTSP mount point conflicts
- Pattern validity checking

Invalid configurations are rejected without affecting the running system.

## API Reference

### VideoSourceManager

Thread-safe registry for managing video sources:

```rust
use source_videos::{VideoSourceManager, VideoSourceConfig};

let manager = VideoSourceManager::new();

// Add a source
let config = VideoSourceConfig::test_pattern("my-test", "smpte");
let id = manager.add_source(config)?;

// Control source
manager.pause_source("my-test")?;
manager.resume_source("my-test")?;
manager.remove_source("my-test")?;

// List all sources
let sources = manager.list_sources();
```

### RTSP Server

Embedded RTSP server for streaming test patterns:

```rust
use source_videos::RtspServerBuilder;

let server = RtspServerBuilder::new()
    .port(8554)
    .add_test_pattern("test1", "smpte")
    .add_test_pattern("test2", "ball")
    .build()?;

server.start()?;
println!("RTSP streams available at:");
for mount in server.list_sources() {
    println!("  {}", server.get_url(&mount));
}
```

### File Generation

Generate test video files:

```rust
use source_videos::generate_test_file;

// Generate a 10-second SMPTE pattern video
generate_test_file("smpte", 10, "test.mp4")?;
```

## Integration with ds-rs

The crate is designed to work seamlessly with the ds-rs DeepStream application:

```rust
use source_videos::SourceVideos;

// Start test infrastructure
let mut sv = SourceVideos::new()?;
sv.add_test_pattern("input1", "smpte")?;
sv.start_rtsp_server(8554)?;

// Use with ds-rs
let uri = "rtsp://localhost:8554/input1";
// Pass this URI to ds-rs SourceController
```

## Requirements

- GStreamer 1.14+ with base, good, and bad plugins
- Rust 1.70+
- Platform-specific GStreamer development packages

### Ubuntu/Debian
```bash
sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
                     libgstreamer-plugins-good1.0-dev libgstreamer-plugins-bad1.0-dev
```

### Windows
Install GStreamer from the official website or use vcpkg.

### macOS
```bash
brew install gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad
```

## Testing

Run the test suite:

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_test

# Test with real RTSP clients
cargo run -- serve --port 8554 &
ffplay rtsp://localhost:8554/test1
```

## Troubleshooting

### GStreamer Plugin Not Found
```
Error: element "videotestsrc" not found
```
Install GStreamer plugins:
- Ubuntu: `sudo apt install gstreamer1.0-plugins-base`
- Windows: Ensure GStreamer bin directory is in PATH
- macOS: `brew install gst-plugins-base`

### RTSP Connection Issues
- Check firewall settings for the RTSP port (default 8554)
- Ensure GStreamer RTSP server plugins are installed
- Try different ports if 8554 is in use

### Performance Issues
- Reduce resolution or framerate in configuration
- Use simpler patterns like "black" instead of "snow"
- Limit the number of concurrent streams

## Examples

See the `examples/` directory for:
- `basic_rtsp_server.rs` - Simple RTSP server setup
- `generate_test_files.rs` - Batch file generation
- `ds_rs_integration.rs` - Integration with ds-rs

## Architecture

The crate follows a modular architecture:

- **Config** - TOML configuration parsing and validation
- **Patterns** - Test pattern definitions and utilities
- **Pipeline** - GStreamer pipeline construction and management
- **Source** - Video source trait and implementations
- **Manager** - Thread-safe source registry and lifecycle management
- **RTSP** - Embedded RTSP server with media factories
- **File** - Video file generation with encoding support

## License

This project is part of the ds-rs codebase and follows the same licensing terms.

## Contributing

1. Ensure all tests pass: `cargo test`
2. Format code: `cargo fmt`
3. Run clippy: `cargo clippy`
4. Add appropriate tests for new features

## Performance Notes

- The crate can handle 10+ concurrent video streams on modern hardware
- Memory usage scales with the number of active sources
- RTSP server performance depends on network bandwidth and client capabilities
- File generation performance varies by codec and resolution settings