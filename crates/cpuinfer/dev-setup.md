# cpuinfer Plugin Development Setup

## Quick Start

### Building the Plugin

```bash
# Build the plugin with ONNX support
cd crates/cpuinfer
cargo build --release

# The plugin will be at: target/release/libgstcpuinfer.so (Linux) or gstcpuinfer.dll (Windows)
```

### Development Testing (Without Installation)

For development, you don't need to install the plugin. Instead, set the `GST_PLUGIN_PATH`:

#### Linux/macOS
```bash
# From the ds-rs root directory
export GST_PLUGIN_PATH=$PWD/target/release:$GST_PLUGIN_PATH

# Verify the plugin is found
gst-inspect-1.0 cpuinfer
```

#### Windows (PowerShell)
```powershell
# From the ds-rs root directory
$env:GST_PLUGIN_PATH = "$PWD\target\release;$env:GST_PLUGIN_PATH"

# Verify the plugin is found
gst-inspect-1.0 cpuinfer
```

#### Windows (Command Prompt)
```cmd
# From the ds-rs root directory
set GST_PLUGIN_PATH=%CD%\target\release;%GST_PLUGIN_PATH%

# Verify the plugin is found
gst-inspect-1.0 cpuinfer
```

## Testing the Plugin

### Basic Pipeline Test
```bash
# Test with videotestsrc
gst-launch-1.0 videotestsrc num-buffers=100 ! cpudetector ! fakesink

# Test with model path
gst-launch-1.0 videotestsrc num-buffers=100 ! \
    cpudetector model-path=models/yolov5n.onnx ! \
    fakesink
```

### Test with Video File
```bash
# Process a video file
gst-launch-1.0 filesrc location=test.mp4 ! \
    decodebin ! videoconvert ! \
    cpudetector model-path=models/yolov5n.onnx ! \
    videoconvert ! autovideosink
```

### Test nvinfer Compatibility
```bash
# Test with config file (nvinfer-style)
cat > test_config.txt << EOF
[property]
onnx-file=models/yolov5n.onnx
batch-size=1
num-detected-classes=80
process-mode=1

[class-attrs-all]
pre-cluster-threshold=0.5
nms-iou-threshold=0.4
EOF

gst-launch-1.0 videotestsrc ! \
    cpudetector config-file-path=test_config.txt ! \
    fakesink
```

### Debugging

Enable debug output to see what the plugin is doing:

```bash
# Basic debug output
GST_DEBUG=cpudetector:5 gst-launch-1.0 videotestsrc ! cpudetector ! fakesink

# Full plugin loading debug
GST_DEBUG=GST_PLUGIN_LOADING:7 gst-inspect-1.0 cpuinfer

# Check if plugin is blacklisted
gst-inspect-1.0 -b
```

## Property Testing

Test all the nvinfer-compatible properties:

```bash
# Test batch processing
gst-launch-1.0 videotestsrc ! \
    cpudetector batch-size=4 ! \
    fakesink

# Test unique ID
gst-launch-1.0 videotestsrc ! \
    cpudetector unique-id=42 ! \
    fakesink

# Test process mode (1=primary, 2=secondary)
gst-launch-1.0 videotestsrc ! \
    cpudetector process-mode=1 ! \
    fakesink

# Test confidence threshold
gst-launch-1.0 videotestsrc ! \
    cpudetector confidence-threshold=0.7 ! \
    fakesink
```

## Troubleshooting

### Plugin Not Found
```bash
# Clear the cache
rm -rf ~/.cache/gstreamer-1.0/  # Linux
rm -rf ~/Library/Caches/gstreamer-1.0/  # macOS
rmdir /s %APPDATA%\gstreamer-1.0  # Windows

# Check plugin path
echo $GST_PLUGIN_PATH  # Linux/macOS
echo %GST_PLUGIN_PATH%  # Windows

# Verify the plugin file exists
ls target/release/*cpuinfer*  # Linux/macOS
dir target\release\*cpuinfer*  # Windows
```

### ONNX Runtime Issues
```bash
# Check for ONNX Runtime
ldd target/release/libgstcpuinfer.so | grep onnx  # Linux
otool -L target/release/libgstcpuinfer.dylib | grep onnx  # macOS

# On Windows, check with Dependency Walker or:
dumpbin /DEPENDENTS target\release\gstcpuinfer.dll
```

### Model Loading Issues
```bash
# Test with debug output
GST_DEBUG=cpudetector:5 gst-launch-1.0 videotestsrc ! \
    cpudetector model-path=/full/path/to/model.onnx ! \
    fakesink
```

## Integration with ds-rs

The cpuinfer plugin is automatically used by ds-rs when using the Standard backend:

```bash
# From ds-rs directory
cargo run --example cross_platform

# The CPU detector will be loaded automatically
```

## Running Tests

```bash
# Run plugin tests
cargo test -p cpuinfer

# Run with ONNX tests (requires model files)
cargo test -p cpuinfer --features ort
```

## Creating a Development Environment Script

Create a `dev-env.sh` (Linux/macOS) or `dev-env.bat` (Windows) in your project root:

### Linux/macOS (`dev-env.sh`)
```bash
#!/bin/bash
export GST_PLUGIN_PATH="$PWD/target/release:$GST_PLUGIN_PATH"
export RUST_LOG=debug
export GST_DEBUG=2

echo "Development environment set up!"
echo "GST_PLUGIN_PATH includes: $PWD/target/release"
echo "You can now run gst-launch-1.0 with cpudetector"
```

### Windows (`dev-env.bat`)
```batch
@echo off
set GST_PLUGIN_PATH=%CD%\target\release;%GST_PLUGIN_PATH%
set RUST_LOG=debug
set GST_DEBUG=2

echo Development environment set up!
echo GST_PLUGIN_PATH includes: %CD%\target\release
echo You can now run gst-launch-1.0 with cpudetector
```

## Performance Testing

```bash
# Measure processing time
gst-launch-1.0 filesrc location=test.mp4 ! \
    decodebin ! videoconvert ! \
    cpudetector model-path=models/yolov5n.onnx ! \
    fpsdisplaysink video-sink=fakesink text-overlay=false -v

# Process every Nth frame for better performance
gst-launch-1.0 filesrc location=test.mp4 ! \
    decodebin ! videoconvert ! \
    cpudetector process-every-n-frames=5 ! \
    videoconvert ! autovideosink
```