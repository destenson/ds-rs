# GStreamer CPU Inference Plugin

A GStreamer plugin providing CPU-based object detection using ONNX models, with fallback support for OpenCV DNN and mock detection.

## Features

- **ONNX Runtime Support** - High-performance inference using ONNX models (YOLOv3-v12)
- **OpenCV DNN Alternative** - Fallback option when ONNX Runtime isn't available
- **Mock Detection Mode** - Testing without actual models
- **Float16/Float32 Support** - Automatic tensor type conversion
- **Passthrough Architecture** - Identity element behavior with inference metadata
- **GStreamer Signals** - Real-time detection notifications

## Building

### With ONNX support (default):
```bash
cargo build --release
```

### Without ONNX (lightweight build):
```bash
cargo build --release --no-default-features
```

### With OpenCV DNN backend:
```bash
cargo build --release --no-default-features --features opencv-dnn
```

## Installation

After building, copy the library to your GStreamer plugins directory:

```bash
# Windows
copy target\release\gstcpuinfer.dll %GSTREAMER_1_0_ROOT_X86_64%\lib\gstreamer-1.0\

# Linux
cp target/release/libgstcpuinfer.so /usr/lib/x86_64-linux-gnu/gstreamer-1.0/

# macOS
cp target/release/libgstcpuinfer.dylib /usr/local/lib/gstreamer-1.0/
```

## Usage

### Basic Pipeline
```bash
gst-launch-1.0 filesrc location=video.mp4 ! decodebin ! videoconvert ! \
  cpudetector model-path=yolov5n.onnx ! videoconvert ! autovideosink
```

### With Properties
```bash
gst-launch-1.0 videotestsrc ! videoconvert ! \
  cpudetector \
    model-path=models/yolov5n.onnx \
    confidence-threshold=0.5 \
    nms-threshold=0.4 \
    process-every-n-frames=2 ! \
  videoconvert ! autovideosink
```

## Element Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| model-path | string | "yolov5n.onnx" | Path to ONNX model file |
| confidence-threshold | double | 0.5 | Minimum confidence for detections (0.0-1.0) |
| nms-threshold | double | 0.4 | Non-maximum suppression threshold (0.0-1.0) |
| input-width | uint | 640 | Model input width |
| input-height | uint | 640 | Model input height |
| process-every-n-frames | uint | 1 | Process every Nth frame (1 = every frame) |

## Signals

### inference-done
Emitted when inference completes on a frame.

**Parameters:**
- `frame_number` (u64): Frame number
- `detection_count` (u32): Number of objects detected

## Supported Models

### ONNX Models
- YOLOv3, YOLOv4, YOLOv5 (all variants)
- YOLOv6, YOLOv7, YOLOv8
- YOLOv9, YOLOv10 (NMS-free)
- YOLOv11, YOLOv12
- YOLO-RD

The detector automatically detects the YOLO version based on output tensor shape.

### Model Formats
- **Input**: RGB/BGR/RGBA/BGRA video frames
- **Model Input**: 640x640 (configurable)
- **Output**: Detection bounding boxes with class labels

## Architecture

The plugin implements a GStreamer BaseTransform element in passthrough mode:
1. Video buffers pass through unchanged
2. Inference runs on frame data
3. Detection results are emitted via signals
4. Optional metadata attachment for downstream processing

## Feature Flags

- `onnx` (default) - ONNX Runtime support
- `opencv-dnn` - OpenCV DNN backend
- `static` - Static linking
- `capi` - C API bindings

## Development

### Running Tests
```bash
cargo test --all-features
```

### Debug Output
```bash
GST_DEBUG=cpudetector:5 gst-launch-1.0 ...
```

## Comparison with Alternatives

Unlike the official `onnxinference` element (GStreamer 1.24+), this plugin:
- Works with older GStreamer versions (1.20+)
- Provides automatic YOLO version detection
- Includes mock detection for testing
- Supports multiple backends (ONNX, OpenCV)
- Emits detection signals directly

## License

MIT/Apache-2.0 (same as parent project)

## Credits

Part of the ds-rs DeepStream Rust port project.