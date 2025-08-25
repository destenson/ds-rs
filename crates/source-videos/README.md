# Source Videos

Dynamic video source generation infrastructure for testing ds-rs DeepStream applications.

## Overview

The `source-videos` crate provides a comprehensive solution for generating test video sources that can be used to test dynamic source management capabilities without requiring actual video files or camera feeds. It supports multiple source types including test patterns, file generation, and RTSP streaming.

## Features

### Core Functionality
- **Test Pattern Generation**: 25+ built-in test patterns (SMPTE, ball animation, noise, etc.)
- **RTSP Server**: Embedded RTSP server serving multiple test streams simultaneously
- **File Generation**: Generate test video files in common formats (MP4, MKV, WebM)
- **Dynamic Source Management**: Runtime addition/removal of video sources
- **Configuration Support**: TOML-based configuration files
- **Cross-Platform**: Works on Windows, Linux, and macOS

### Advanced CLI Features (NEW in PRP-38)
- **Multiple Serving Modes**: Files, playlist, monitoring, and network simulation
- **Advanced File Serving**: Enhanced `serve-files` command with sophisticated filtering
- **Playlist Mode**: Sequential, random, or shuffle playback with repeat options
- **Directory Monitoring**: Real-time file system watching with metrics
- **Network Simulation Integration**: Test resilience with various network profiles
- **Shell Completions**: Auto-completion for Bash, Zsh, Fish, and PowerShell
- **Advanced Filtering**: Include/exclude patterns, format filters, duration filters, date filters
- **Daemon Mode**: Background operation with PID file management
- **Multiple Output Formats**: Text, JSON, and CSV output for automation
- **Dry Run Mode**: Preview operations without starting servers

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

### Command Line Usage

#### Basic Commands
```bash
# List available test patterns
source-videos list

# Traditional serve with patterns
source-videos serve --patterns smpte,ball,snow

# Generate test video file
source-videos generate --pattern smpte --duration 10 --output test.mp4

# Interactive REPL mode
source-videos interactive

# Run comprehensive test suite
source-videos test --port 8554
```

#### Advanced Commands (PRP-38)

##### serve-files: Enhanced File Serving
```bash
# Basic directory serving
source-videos serve-files -d /path/to/videos

# Advanced filtering and controls
source-videos serve-files -d /path/to/videos \
  --recursive \
  --include "*.mp4" "*.mkv" \
  --exclude "*test*" "*backup*" \
  --format mp4 \
  --min-duration 60 \
  --max-duration 3600 \
  --modified-since 2024-01-01 \
  --max-streams 10

# Production features
source-videos serve-files -d /path/to/videos \
  --daemon \
  --pid-file /var/run/source-videos.pid \
  --status-interval 30 \
  --metrics \
  --output-format json

# Preview without starting
source-videos serve-files -d /path/to/videos --dry-run
```

##### playlist: Sequential Video Playback
```bash
# Sequential playlist
source-videos playlist -d /path/to/videos --playlist-mode sequential

# Random playlist with repeat
source-videos playlist -d /path/to/videos \
  --playlist-mode shuffle \
  --playlist-repeat all \
  --transition-duration 2.0 \
  --crossfade

# From playlist file
source-videos playlist \
  --playlist-file /path/to/playlist.m3u \
  --playlist-mode random
```

##### monitor: Real-time Directory Monitoring
```bash
# Basic monitoring
source-videos monitor -d /path/to/videos --recursive

# With metrics and structured output
source-videos monitor -d /path/to/videos \
  --metrics \
  --output-format json \
  --watch-interval 500 \
  --list-streams
```

##### simulate: Network Condition Testing
```bash
# Test with network profiles
source-videos simulate --network-profile 3g -d /path/to/videos
source-videos simulate --network-profile satellite --patterns ball,smpte

# Custom network conditions
source-videos simulate --network-profile custom \
  --packet-loss 5.0 --latency 250 --bandwidth 2000

# Duration-limited testing with metrics
source-videos simulate --network-profile poor \
  --duration 300 --metrics
```

##### completions: Shell Integration
```bash
# Generate bash completions
source-videos completions bash > /etc/bash_completion.d/source-videos

# Other shells
source-videos completions zsh > ~/.zsh/completions/_source-videos
source-videos completions fish > ~/.config/fish/completions/source-videos.fish
source-videos completions powershell > source-videos.ps1
```

#### Global Options
```bash
# Verbose logging (stackable)
source-videos -v serve-files -d /videos    # info level
source-videos -vv serve-files -d /videos   # debug level
source-videos -vvv serve-files -d /videos  # trace level

# Configuration file
source-videos -c config.toml serve -d /videos

# Quiet mode
source-videos serve-files -d /videos --quiet
```

#### REST API Integration
```bash
# Enable API with any command
source-videos serve-files -d /videos --api --api-port 3000
source-videos playlist -d /videos --api

# API endpoints available at http://localhost:3000/api/
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

Embedded RTSP server for streaming test patterns and video files:

```rust
use source_videos::{RtspServerBuilder, DirectoryScanner, DirectoryConfig};

// Basic server with test patterns
let server = RtspServerBuilder::new()
    .port(8554)
    .add_test_pattern("test1", "smpte")
    .add_test_pattern("test2", "ball")
    .build()?;

server.start()?;

// NEW: Server with directory scanning
let dir_config = DirectoryConfig {
    path: "/path/to/videos".to_string(),
    recursive: true,
    filters: None,
    lazy_loading: false,
    mount_prefix: Some("videos".to_string()),
};

let mut scanner = DirectoryScanner::new(dir_config);
let source_configs = scanner.scan()?;

let mut server_builder = RtspServerBuilder::new().port(8554);
for config in source_configs {
    server_builder = server_builder.add_source(config);
}

let server = server_builder.build()?;
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

### Directory and File Serving (NEW!)

Serve video files from directories or explicit file lists:

```rust
use source_videos::{DirectoryScanner, DirectoryConfig, FilterConfig, VideoSourceManager};

// Scan directory for video files
let dir_config = DirectoryConfig {
    path: "/path/to/videos".to_string(),
    recursive: true,
    filters: Some(FilterConfig {
        include: vec!["*.mp4".to_string(), "*.avi".to_string()],
        exclude: vec!["*.tmp".to_string(), "backup_*".to_string()],
        extensions: vec!["mp4".to_string(), "avi".to_string(), "mkv".to_string()],
    }),
    lazy_loading: false,
    mount_prefix: Some("videos".to_string()),
};

let mut scanner = DirectoryScanner::new(dir_config);
let source_configs = scanner.scan()?;

println!("Found {} video files", source_configs.len());

// Add to source manager
let manager = VideoSourceManager::new();
let added = manager.add_sources_batch(source_configs)?;
println!("Successfully added {} sources", added.len());

// Or serve via RTSP
let mut server_builder = RtspServerBuilder::new().port(8554);
for config in scanner.scan()? {
    server_builder = server_builder.add_source(config);
}
let server = server_builder.build()?;
```

### Understanding Mount Points

Mount points are the path part of RTSP URLs that identify specific video streams:

- **Pattern**: `rtsp://localhost:8554/pattern_smpte` → mount point is `pattern_smpte`  
- **File**: `rtsp://localhost:8554/videos/movie` → mount point is `videos/movie`  
- **Directory**: `/videos/action/movie.mp4` → becomes mount point `videos/action/movie`

Each video file gets a unique mount point so clients can access streams independently.

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

### Real-world Usage Examples

#### Video Collection Server
```bash
# Serve large video collection with smart filtering
source-videos serve-files -d /media/videos \
  --recursive \
  --include "*.mp4" "*.mkv" "*.avi" \
  --exclude "*sample*" "*trailer*" "*temp*" \
  --max-streams 20 \
  --daemon \
  --metrics
```

#### Playlist Testing
```bash
# Create shuffled playlist of high-quality videos
source-videos playlist -d /media/hd-content \
  --playlist-mode shuffle \
  --playlist-repeat all \
  --include "*1080p*.mp4" "*4k*.mkv" \
  --transition-duration 1.5
```

#### Development Monitoring
```bash
# Monitor video development directory with JSON output
source-videos monitor -d /dev/video-assets \
  --recursive \
  --output-format json \
  --metrics \
  --list-streams > video-changes.json
```

#### Advanced Network Testing (PRP-43)
```bash
# Test with GStreamer netsim element for realistic simulation
source-videos serve -d /test-videos \
  --network-profile noisy-radio  # Simulates 15% loss, duplicates, reordering

# Dynamic network scenarios with time-based changes
source-videos serve -d /test-videos \
  --network-scenario drone-urban  # Simulates urban drone flight with building obstruction

# Available scenarios:
# - degrading: Network quality degrades over time
# - flaky: Periodic connection issues
# - intermittent-satellite: Satellite with periodic drops
# - noisy-radio: High interference radio link
# - drone-urban: Urban drone flight with multipath
# - drone-mountain: Mountain terrain with masking
# - congestion: Peak hour network congestion

# Custom network conditions with netsim properties
source-videos serve -d /test-videos \
  --packet-loss 5.0 \
  --latency 100 \
  --jitter 20 \
  --bandwidth 1000  # Now uses netsim for bandwidth throttling
```

### Code Examples

See the `examples/` directory for:
- `directory_server.rs` - Serve video files from a directory
- `batch_file_server.rs` - Serve specific video files from a list
- `mixed_sources.rs` - Combine test patterns, directory files, and explicit files
- `drone_network_demo.rs` - Network simulation with drone profiles
- `watched_directory.rs` - File system monitoring with auto-reload
- Basic RTSP server setups and integration patterns

## Architecture

The crate follows a modular architecture:

### Core Modules
- **Config** - TOML configuration parsing and validation
- **Patterns** - Test pattern definitions and utilities
- **Pipeline** - GStreamer pipeline construction and management
- **Source** - Video source trait and implementations
- **Manager** - Thread-safe source registry and lifecycle management
- **RTSP** - Embedded RTSP server with media factories
- **File** - Video file generation with encoding support

### Advanced Features (PRP-38)
- **CLI** - Advanced command-line interface with multiple modes
- **Directory** - Directory scanning and video file discovery
- **FileUtils** - Video file detection and mount point generation
- **FileSource** - Direct file-based video source implementation
- **Watch** - File system monitoring and auto-reload functionality
- **Network** - Advanced network simulation with GStreamer netsim element
  - Packet loss, duplication, reordering simulation
  - Dynamic scenarios with time-based progression
  - Bandwidth throttling with token bucket algorithm
  - Support for drone, satellite, and mobile network profiles
- **API** - REST API for remote control and automation
- **Runtime** - Dynamic configuration and signal handling

### CLI Command Architecture
- **serve-files** - Enhanced file serving with advanced filtering
- **playlist** - Sequential/random playback with repeat modes
- **monitor** - Real-time directory monitoring with metrics
- **simulate** - Network condition testing with profiles
- **completions** - Shell auto-completion generation

## License

This project is part of the ds-rs codebase and follows the same licensing terms.

## Contributing

1. Ensure all tests pass: `cargo test`
2. Test new CLI features: `cargo run -- --help`
3. Add appropriate tests for new features
4. Run clippy: `cargo clippy`
5. Format code: `cargo fmt`
6. Test shell completions: `source-videos completions bash | head -20`
7. Validate against PRP requirements in `PRPs/38-source-videos-cli-enhancements.md`

## Performance Notes

- The crate can handle 10+ concurrent video streams on modern hardware
- Memory usage scales with the number of active sources
- RTSP server performance depends on network bandwidth and client capabilities
- File generation performance varies by codec and resolution settings
