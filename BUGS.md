# Current Bugs

## 🔴 ACTIVE: Float16 Model Support Issue

**Status**: ACTIVE - PRP-02 created for fix (may be fixed already, needs verification)

**Problem**:
- YOLO models using float16 (half precision) format fail to load
- Error: "Float16 models not currently supported due to lifetime issues"
- The ORT crate supports float16 but implementation has Rust lifetime issues

**Symptoms**:
```
Detection failed: Configuration error: Float16 models not currently supported due to lifetime issues
```

**Root Cause**:
- Model expects float16 input tensors but code provides float32
- Lifetime issues when creating ORT Values from float16 arrays
- The converted f16 data doesn't live long enough for Value::from_array

**Workaround**:
- Use float32 YOLO models instead of float16
- Convert models to float32 format using ONNX tools

**Planned Fix** (PRP-02):
- Implement proper f32→f16 conversion for inputs
- Fix lifetime issues by managing f16 data ownership correctly
- Support f16 output tensor extraction and conversion
- See PRPs/02-float16-model-support.md for implementation plan

## ✅ FIXED: Shutdown Issues (PRP-25 completed)

**Status**: RESOLVED as of 2025-08-23 via PRP-25 implementation  

**Final Solution**: 
- **Replaced mixed event systems** with GLib's MainLoop and native signal handling
- **Unix systems**: Use `glib::unix_signal_add(SIGINT)` integrated with main loop
- **Windows**: Fall back to `ctrlc` crate with main loop quit()
- **Main loop**: Use `main_loop.run()` which blocks until `quit()` is called
- **No race conditions**: All setup happens before main_loop.run() starts

**Validation**: 
- ✅ Shutdown tests pass: `cargo test --test shutdown_test` 
- ✅ Clean termination on Ctrl+C with proper cleanup
- ✅ No more repeated "Received interrupt signal, shutting down..." messages
- ✅ Application exits with proper status codes

**Key Improvement**: Using GLib's native event loop integration instead of manual iteration eliminates race conditions between signal handling and pipeline management.

## ✅ FIXED: Video playback issues (framerate negotiation)

**Status**: RESOLVED as of 2025-08-24

**Problem**: 
- H264 parser was detecting unreasonable framerate values (15360.0 fps)
- This exceeded the maximum allowed framerate (32.8 fps) causing caps negotiation failure
- Video would freeze on first/last frame due to failed negotiation between uridecodebin and compositor

**Root Cause**:
- Some video files contain incorrect framerate metadata in their H264 stream
- The uridecodebin would pass this invalid framerate directly to the compositor
- The compositor couldn't handle such extreme framerates, causing the pipeline to stall

**Solution Implemented**:
- Added `videorate` and `capsfilter` elements between uridecodebin and compositor
- These elements normalize the framerate to a standard 30 fps
- The pipeline now handles videos with invalid framerate metadata gracefully

**Code Changes**:
- Modified `video_source.rs::connect_pad_added_default()` 
- When connecting to compositor (Standard backend), now creates:
  1. `videorate` element to handle framerate conversion
  2. `capsfilter` element with caps set to "video/x-raw,framerate=30/1"
- Pipeline flow: uridecodebin → videorate → capsfilter → compositor

**Validation**:
- ✅ Videos now play smoothly without freezing
- ✅ No more H264 parser warnings about excessive framerate
- ✅ Proper caps negotiation between all elements
- ✅ Works with various video formats and framerates


