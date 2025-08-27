# PRP-44: Fix False Detection Bug in CPU Detector

## Problem Statement
The CPU detector is producing hundreds of false detections per frame, making tracking completely unusable. Analysis of BUGS.md lines 1-150 shows:
- 324 false detections per frame
- Many detections at position (0.0, 0.0)
- Confidence scores in the range of 404814.00 instead of 0.0-1.0
- Invalid object classes (dog, kite, broccoli, cell phone in ball tracking)
- Tiny or invalid bounding box sizes

## Root Cause Analysis
Based on code review in `crates/cpuinfer/src/detector.rs`:

1. **Missing Sigmoid Activation**: YOLOv5 outputs raw logits that need sigmoid activation for confidence scores and x,y coordinates
2. **Incorrect Format Detection**: The auto-detection of YOLO format may be misinterpreting the output tensor layout
3. **Missing Confidence Normalization**: Raw confidence values are being used without proper normalization
4. **Coordinate Scaling Issues**: Bounding box coordinates may not be properly scaled from model space to image space

## Implementation Blueprint

### Phase 1: Diagnose the Exact Issue
1. Add comprehensive debug logging to understand raw output values
2. Verify which YOLO version is being detected
3. Check if output format is transposed correctly
4. Validate confidence score ranges before and after processing

### Phase 2: Fix YOLO Post-Processing
1. Apply sigmoid activation to objectness scores and x,y coordinates (YOLOv5)
2. Ensure proper confidence calculation (objectness * class_score for v5)
3. Fix coordinate transformation from model space to image space
4. Validate NMS threshold is appropriate

### Phase 3: Add Safety Checks
1. Add confidence value validation (must be 0.0-1.0)
2. Add coordinate bounds checking
3. Add maximum detection limit per frame
4. Add class ID validation

## Key Files to Modify
- `crates/cpuinfer/src/detector.rs` - Main detection post-processing logic
  - Function: `postprocess_yolov5` (lines 435-580)
  - Function: `detect_yolo_version` (lines 402-433)
- `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs` - Integration point

## Technical Details

### YOLOv5 Output Format
YOLOv5 outputs in format `[1, 25200, 85]` or transposed `[1, 85, 25200]` where:
- First 4 values: cx, cy, w, h (need sigmoid for cx, cy in v5)
- 5th value: objectness (needs sigmoid)
- Next 80 values: class scores (need sigmoid)

### Correct Post-Processing Steps
1. Apply sigmoid to appropriate values
2. Calculate confidence = sigmoid(objectness) * sigmoid(max_class_score)
3. Transform coordinates from normalized to pixel space
4. Apply confidence threshold (typically 0.25-0.5)
5. Apply NMS with IoU threshold (typically 0.45-0.65)

### Reference Implementation Patterns
Look at these files for correct patterns:
- Python YOLO implementations typically use `torch.sigmoid()` on raw outputs
- Check confidence threshold defaults (usually 0.25 for YOLO)
- NMS IoU threshold (usually 0.45)

## Testing Strategy
1. Use the ball tracking example as primary test case
2. Verify detection count is reasonable (< 10 per frame for simple scenes)
3. Verify confidence scores are in [0.0, 1.0] range
4. Verify bounding boxes are within image bounds
5. Test with different YOLO models (v5n, v5s, v8n)

## Validation Gates
```bash
# Build and check
cd crates/ds-rs
cargo build --all-features
cargo clippy --all-targets --all-features -- -D warnings

# Run unit tests
cargo test --test cpu_backend_tests

# Run ball tracking example and verify no false detections
cargo run --example ball_tracking_visualization --all-features

# Verify confidence scores are normalized (0.0-1.0)
# Verify detection count is reasonable (<10 for test video)
# Verify no detections at (0,0) unless object actually there
```

## Success Criteria
- Detection confidence scores in range [0.0, 1.0]
- Reasonable number of detections per frame (<10 for simple scenes)
- No detections at (0,0) unless legitimate
- Correct object classes for the model being used
- Ball tracking example works smoothly

## Implementation Notes
- Start by adding debug logging to understand current output values
- Test changes incrementally - fix one issue at a time
- Consider adding a "strict mode" flag for extra validation during debugging
- The yolov5n.onnx model in use should detect COCO classes (80 classes)

## References
- YOLO output format documentation: https://github.com/ultralytics/yolov5/issues/708
- Sigmoid activation requirement: https://github.com/ultralytics/yolov5/issues/6998
- Example Python implementation showing sigmoid: https://github.com/ultralytics/yolov5/blob/master/models/yolo.py

## Risk Assessment
- **High Risk**: Breaking existing working detections if any
- **Mitigation**: Add feature flag for new post-processing if needed

## Estimated Effort
- Research & Diagnosis: 1 hour
- Implementation: 2-3 hours
- Testing & Validation: 1 hour
- Total: 4-5 hours

## Confidence Score: 8/10
The issue is well-diagnosed with clear symptoms. The fix involves standard YOLO post-processing that is well-documented. Main uncertainty is whether there are additional issues beyond the sigmoid activation.