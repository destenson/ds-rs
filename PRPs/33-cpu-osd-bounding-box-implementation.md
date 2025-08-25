# PRP-33: CPU OSD Bounding Box Implementation for Ball Tracking Visualization

**Generated**: 2025-08-25  
**Status**: Ready for Implementation  
**Priority**: Critical  
**Confidence**: 9/10

## Problem Statement

The ball tracking visualization pipeline detects objects but fails to display bounding boxes on screen. The CPU detector successfully emits detection signals with object coordinates, and the metadata bridge receives this data, but the CPU OSD element lacks the actual drawing implementation to render bounding boxes.

### Current State
- CPU detector (`cpudetector`) emits `inference-results` signal with JSON-serialized detections
- Pipeline builder connects signals to MetadataBridge which stores detection data
- CPU OSD uses either `cairooverlay` or `textoverlay` as fallback
- No draw signal handler implementation for `cairooverlay` (line 314 in `elements.rs`: "In a real implementation...")
- Result: Detection data exists but no visual bounding boxes appear

### Desired State
- Real-time bounding box rendering around detected balls
- Boxes drawn using Cairo graphics with configurable appearance
- Smooth performance without frame drops
- Clear visual feedback showing detection confidence and class labels

### Business Value
Enables visual validation of detection accuracy, provides immediate feedback for debugging AI models, and creates compelling demonstrations of the complete computer vision pipeline.

## Context and Research

### Architecture Overview
The rendering pipeline follows this data flow:
1. **Detection**: CPU detector processes frames and generates Detection objects
2. **Signal Emission**: Detector emits `inference-results` signal with JSON data
3. **Bridge Update**: MetadataBridge receives and stores detection metadata
4. **OSD Rendering**: OSD element should draw boxes but implementation missing

### GStreamer Overlay Patterns
Two primary approaches for overlays in GStreamer:

1. **cairooverlay**: Direct drawing via draw signal
   - Receives Cairo context for each frame
   - Draws directly on video surface
   - Used in current CPU OSD implementation

2. **overlaycomposition**: Composition-based approach
   - Creates separate overlay buffer
   - Composes with video frame
   - More flexible for complex overlays

### Signal Handling in Rust
The gstreamer-rs bindings provide `connect_closure` for signal handling:
- Signals pass arguments as closure parameters
- Cairo context provided for drawing operations
- Thread safety handled via Arc<Mutex<>> pattern

## Requirements

### Functional Requirements
1. **Connect draw signal**: Wire cairooverlay draw signal to rendering callback
2. **Retrieve detection data**: Access MetadataBridge data for current frame
3. **Draw bounding boxes**: Render rectangles at detection coordinates
4. **Display labels**: Show class names and confidence scores
5. **Handle multiple objects**: Support rendering multiple detections per frame
6. **Coordinate transformation**: Convert normalized to pixel coordinates

### Non-Functional Requirements
1. **Performance**: Maintain 30+ FPS with overlay rendering
2. **Thread safety**: Proper synchronization with Arc<Mutex<>>
3. **Error handling**: Graceful degradation if drawing fails
4. **Memory efficiency**: Avoid allocations in draw callback

## Documentation & References

### Core Files to Modify
- `crates/ds-rs/src/backend/cpu_vision/elements.rs` - CPU OSD implementation
- `crates/ds-rs/src/pipeline/builder.rs` - Signal connection logic
- `crates/ds-rs/src/rendering/metadata_bridge.rs` - Detection data access

### Reference Implementations
- `../gstreamer-rs/examples/src/bin/overlay-composition.rs` - Cairo overlay pattern in Rust
- `../gstreamer-rs/examples/src/bin/pango-cairo.rs` - Text rendering with Cairo
- `https://github.com/GStreamer/gst-plugins-good/blob/master/tests/examples/cairo/cairo_overlay.c` - C example

### Documentation
- GStreamer cairooverlay: `https://gstreamer.freedesktop.org/documentation/cairo/index.html`
- Cairo graphics: `https://www.cairographics.org/manual/`
- gstreamer-rs signals: Check `Element::connect_closure` in local gstreamer-rs source

## Implementation Blueprint

### Phase 1: Signal Connection
```pseudocode
create_cpu_osd():
    IF cairooverlay created successfully:
        // Create shared context for draw callback
        metadata_bridge = Arc<Mutex<MetadataBridge>>
        
        // Connect draw signal
        overlay.connect_closure("draw", false, closure(
            |element, cr, timestamp, duration| -> draw_bounding_boxes()
        ))
```

### Phase 2: Draw Callback Implementation
```pseudocode
draw_bounding_boxes(cr: cairo::Context, timestamp: ClockTime):
    // Get detection data from bridge
    detections = metadata_bridge.get_frame_metadata(timestamp)
    
    // Configure drawing style
    cr.set_source_rgba(0.0, 1.0, 0.0, 0.8)  // Green boxes
    cr.set_line_width(3.0)
    
    // Draw each detection
    FOR detection IN detections:
        // Draw rectangle
        cr.rectangle(detection.x, detection.y, detection.width, detection.height)
        cr.stroke()
        
        // Draw label
        cr.move_to(detection.x, detection.y - 5)
        cr.show_text(format!("{}: {:.2}", detection.class_name, detection.confidence))
```

### Phase 3: Metadata Bridge Integration
```pseudocode
// Modify builder.rs to pass metadata_bridge to CPU OSD
configure_osd_element():
    IF backend == Standard AND element is CPU OSD:
        // Get internal cairooverlay element from bin
        overlay = bin.get_by_name("osd-overlay")
        
        // Connect metadata bridge to overlay
        setup_cairo_draw_handler(overlay, metadata_bridge)
```

## Tasks to Complete

### Task 1: Implement Cairo Draw Handler
**File**: `crates/ds-rs/src/backend/cpu_vision/elements.rs`
- Add metadata_bridge parameter to `create_cpu_osd`
- Connect cairooverlay draw signal
- Implement draw callback with Cairo rectangle operations
- Handle coordinate scaling from normalized to pixel space

### Task 2: Update Pipeline Builder Integration
**File**: `crates/ds-rs/src/pipeline/builder.rs`
- Pass metadata_bridge to CPU OSD creation
- Ensure proper Arc<Mutex<>> wrapping for thread safety
- Add backend-specific configuration checks

### Task 3: Enhance Metadata Bridge Access
**File**: `crates/ds-rs/src/rendering/metadata_bridge.rs`
- Add efficient frame lookup by timestamp
- Implement caching for recent frames
- Add coordinate transformation utilities

### Task 4: Add Rendering Configuration
**File**: `crates/ds-rs/src/rendering/config.rs`
- Add CPU OSD specific rendering options
- Configure box colors, line width, font settings
- Support per-class color mapping

### Task 5: Create Integration Tests
**File**: `crates/ds-rs/tests/cpu_osd_rendering_tests.rs`
- Test signal connection
- Verify drawing callback execution
- Validate coordinate transformations
- Performance benchmarks

## Validation Gates

```bash
# Build and format check
cargo build --all-features
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings

# Run specific tests
cargo test cpu_osd
cargo test ball_tracking_visualization

# Run example with debug output
GST_DEBUG=cpudetector:5,cairooverlay:5 cargo run --example ball_tracking_visualization

# Verify visual output
# Should see green bounding boxes around detected objects
# Check console for "Drawing N detections" log messages
```

## Success Criteria
1. Bounding boxes visible around detected balls
2. Smooth 30+ FPS performance
3. Correct box positioning matching object locations
4. Labels showing class and confidence
5. No memory leaks or crashes during extended runs

## Risk Mitigation
- **Cairo not available**: Keep textoverlay fallback, log warning
- **Performance issues**: Add frame skipping option, reduce draw complexity
- **Thread synchronization**: Use try_lock to avoid blocking, skip frame if locked
- **Coordinate mismatch**: Add debug overlay showing raw coordinates

## Notes for Implementation
- Use `../gstreamer-rs/examples/src/bin/overlay-composition.rs` as reference pattern
- Cairo coordinates are in pixels, detections may be normalized
- Consider adding debug visualization mode with extra information
- The draw callback runs on streaming thread, minimize processing
- Test with both videotestsrc and actual video files

---

**Confidence Score: 9/10**

High confidence due to:
- Clear examples in gstreamer-rs repository
- Well-understood Cairo drawing API
- Existing metadata bridge infrastructure
- Straightforward signal connection pattern

Minor uncertainty around:
- Exact timestamp synchronization between detector and OSD
- Performance impact with many detections