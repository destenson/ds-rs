# PRP-23: Integration of gst-plugins-rs Vision Elements

## Summary
Integrate existing Rust GStreamer plugins from gst-plugins-rs repository to enhance the CPU Vision backend with additional computer vision capabilities, including HSV-based detection, color detection, and video effects that can support object detection and tracking workflows.

## Background
The gst-plugins-rs repository contains numerous vision-related plugins written in Rust that could enhance ds-rs capabilities:
- **hsvdetector/hsvfilter**: HSV color space detection and filtering
- **colordetect**: Color detection in video streams
- **videocompare**: Frame comparison capabilities
- **videofx/border**: Video effects and border detection

These plugins can provide additional detection mechanisms and preprocessing capabilities for the CPU Vision backend.

## Goals
1. Integrate hsvdetector for color-based object detection
2. Utilize colordetect for specific color tracking
3. Leverage videocompare for motion detection
4. Create preprocessing pipeline with videofx elements
5. Build hybrid detection combining multiple techniques

## Non-Goals
- Reimplementing existing plugins
- Modifying gst-plugins-rs source code
- Creating new GStreamer plugins from scratch

## Detailed Design

### Integration Architecture

#### Plugin Discovery and Loading
```
crates/ds-rs/src/backend/cpu_vision/
├── plugins/
│   ├── mod.rs
│   ├── hsv_detector.rs    # HSV detector wrapper
│   ├── color_detect.rs    # Color detection wrapper  
│   └── preprocessor.rs    # Preprocessing pipeline
```

#### Element Registry
Extend the Standard backend to know about gst-plugins-rs elements:

```rust
pub struct PluginRegistry {
    available_plugins: HashMap<String, PluginInfo>,
    loaded: HashSet<String>,
}

impl PluginRegistry {
    pub fn discover_plugins(&mut self) {
        // Check for gst-plugins-rs elements
        self.check_element("hsvdetector");
        self.check_element("colordetect");
        self.check_element("videocompare");
    }
}
```

### Use Cases

#### 1. HSV-Based Ball Detection
Perfect for the bouncing ball detection use case:

- Configure HSV ranges for ball color
- Detect colored regions
- Extract bounding boxes
- Feed to tracker

Pipeline example:
```
videosrc ! hsvdetector ! cputracker ! overlay
```

#### 2. Color-Based Object Tracking
Track specific colored objects:

- Define target colors
- Use colordetect for initial detection
- Combine with shape analysis
- Track colored regions

#### 3. Motion Detection Pipeline
Detect moving objects using frame comparison:

- Use videocompare for frame differencing
- Identify motion regions
- Trigger detection on motion
- Reduce computation on static scenes

#### 4. Preprocessing Enhancement
Improve detection quality:

- Use videofx for image enhancement
- Apply filters before detection
- Border detection for region of interest
- Color space conversions

### Integration Patterns

#### Pattern 1: Parallel Detection
Run multiple detection methods in parallel:

```
                ┌─> hsvdetector ─┐
videosrc ─> tee ├─> colordetect ─┼─> aggregator ─> tracker
                └─> cpudetector ─┘
```

#### Pattern 2: Cascaded Detection
Use one detector to guide another:

```
videosrc ─> hsvdetector ─> roi_extract ─> cpudetector ─> tracker
```

#### Pattern 3: Selective Processing
Process only when motion detected:

```
videosrc ─> videocompare ─> motion_gate ─> cpudetector ─> tracker
```

### Configuration System

#### Unified Configuration
Extend detection config to include plugin settings:

```rust
pub struct VisionConfig {
    pub detection: DetectionConfig,
    pub plugins: PluginConfig,
}

pub struct PluginConfig {
    pub use_hsv: bool,
    pub hsv_ranges: Vec<HsvRange>,
    pub use_color_detect: bool,
    pub target_colors: Vec<Color>,
    pub use_motion_gate: bool,
    pub motion_threshold: f32,
}
```

#### Dynamic Pipeline Building
Build pipelines based on configuration:

```rust
impl PipelineBuilder {
    pub fn build_vision_pipeline(&self, config: &VisionConfig) -> Pipeline {
        let mut pipeline = Pipeline::new();
        
        if config.plugins.use_hsv {
            pipeline.add_element("hsvdetector");
        }
        
        if config.plugins.use_motion_gate {
            pipeline.add_element("videocompare");
        }
        
        // Add main detector
        pipeline.add_element("cpudetector");
        
        pipeline
    }
}
```

## Implementation Plan

### Phase 1: Plugin Discovery
1. Add gst-plugins-rs path to GST_PLUGIN_PATH
2. Implement plugin availability checking
3. Create plugin registry system
4. Document required plugins

### Phase 2: HSV Detector Integration
1. Create HSV detector wrapper
2. Add configuration for HSV ranges
3. Convert HSV results to detection format
4. Test with colored object videos

### Phase 3: Color Detection Integration
1. Wrap colordetect element
2. Add color target configuration
3. Implement color-to-bbox conversion
4. Create color tracking example

### Phase 4: Motion Detection
1. Integrate videocompare element
2. Create motion gating logic
3. Add motion threshold config
4. Benchmark performance improvement

### Phase 5: Hybrid Pipeline
1. Implement parallel detection pattern
2. Create detection aggregator
3. Add confidence fusion logic
4. Build complete example

## Testing Strategy

### Unit Tests
- Plugin availability detection
- Configuration parsing
- Element creation
- Metadata conversion

### Integration Tests
- HSV detection accuracy
- Color tracking consistency
- Motion detection sensitivity
- Pipeline performance

### Test Scenarios
1. Colored ball tracking (HSV)
2. Specific object color tracking
3. Motion-triggered detection
4. Multi-method fusion

## Validation Gates

```bash
# Ensure plugins available
GST_PLUGIN_PATH=$GST_PLUGIN_PATH:../gst-plugins-rs/target/debug

# Check plugin loading
gst-inspect-1.0 hsvdetector
gst-inspect-1.0 colordetect

# Run integration tests
cargo test --test plugin_integration

# Benchmark hybrid pipeline
cargo run --example hybrid_detection
```

## Resources

### Plugin Documentation
- gst-plugins-rs repo: `C:\Users\deste\repos\gst-plugins-rs`
- HSV detector: `video/hsv/src/hsvdetector/`
- Color detect: `video/videofx/src/colordetect/`
- Video compare: `video/videofx/src/videocompare/`

### Example Code
- HSV filter example: Check gst-plugins-rs examples
- Plugin loading: GStreamer plugin documentation

### References
- HSV color space: https://en.wikipedia.org/wiki/HSL_and_HSV
- Color detection algorithms: OpenCV documentation
- Motion detection: Frame differencing techniques

## Performance Considerations

### Advantages
- HSV detection is very fast (100+ FPS)
- Color detection has minimal overhead
- Motion gating reduces unnecessary processing
- All plugins are Rust-native (no FFI overhead)

### Trade-offs
- Multiple plugins increase pipeline complexity
- Color-based detection sensitive to lighting
- Motion detection adds frame latency
- Memory usage increases with parallel paths

## Configuration Examples

### Ball Detection Config
```toml
[plugins]
use_hsv = true
hsv_ranges = [
    { h_min = 20, h_max = 40, s_min = 100, s_max = 255, v_min = 100, v_max = 255 }
]

[detection]
enable_neural_net = false  # Use HSV only
```

### Motion-Triggered Detection
```toml
[plugins]
use_motion_gate = true
motion_threshold = 0.1

[detection]
enable_neural_net = true
model = "yolov5n.onnx"
```

### Hybrid Detection
```toml
[plugins]
use_hsv = true
use_color_detect = true
use_motion_gate = true

[detection]
enable_neural_net = true
fusion_strategy = "weighted_average"
```

## Success Criteria

1. HSV detector integrated and working
2. At least two plugins successfully wrapped
3. Hybrid pipeline achieving 30+ FPS
4. Configuration system supporting all plugins
5. Documentation with examples

## Dependencies

### Required
- gst-plugins-rs built and in GST_PLUGIN_PATH
- PRP-21 and PRP-22 for base detection/tracking

### Runtime Dependencies
- GStreamer 1.20+
- gst-plugins-rs plugins loaded

## Risk Mitigation

### Plugin Availability
- Check at runtime and fallback gracefully
- Document plugin installation clearly
- Provide pre-built binaries

### Version Compatibility
- Test with multiple GStreamer versions
- Lock gst-plugins-rs version
- Document version requirements

## Future Enhancements

1. **Custom Fusion Algorithms**: Weighted combination of detections
2. **Adaptive Thresholds**: Auto-adjust based on scene
3. **Plugin Chaining**: Dynamic pipeline reconfiguration
4. **Performance Profiling**: Per-plugin metrics
5. **Cloud Processing**: Offload to remote plugins

## Notes

- Start with HSV detector for ball tracking demo
- Color detection useful for specific object tracking
- Motion gating can significantly reduce CPU usage
- Consider plugin overhead in pipeline design
- Document lighting requirements for color-based detection

## Confidence Score: 9/10

Very high confidence because:
- Plugins already exist and are tested
- Clear integration points
- No need to modify external code
- Incremental implementation possible

Minor concern:
- Plugin version compatibility management