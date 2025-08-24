# Current Bugs

## ⚠️ PARTIALLY FIXED: Shutdown Issues (PRP-25 in progress)

**Status**: PARTIALLY RESOLVED as of 2025-08-23 via PRP-25 implementation

**Current State**:
- ✅ Ctrl+C works when pressed after pipeline starts
- ❌ Race condition: Ctrl+C before pipeline start causes video to appear but never exit
- ❌ Pipeline starts but video doesn't display properly (window appears behind others)

**Solution Applied**: 
- Replaced mixed event systems (Tokio + ctrlc + GStreamer) with GLib MainContext manual iteration
- Used AtomicBool + ctrlc signal handler + main_context.iteration(false) tight loop
- Added race condition checks at multiple startup points

**Still Testing**:
- Race condition fix with early shutdown detection
- Pipeline state transition debugging
- Video display/window focus issues

## Video playback issues (framerate negotiation)

Currently, the application runs and opens a window, but video playback may have issues with H264 framerate negotiation.

**Symptoms**:
- H264 parser warning: "VUI framerate 15360.0 exceeds allowed maximum 32.8"
- Video may get stuck on first/last frame
- Window appears but playback may not be smooth

**Status**: Needs investigation - may be related to caps negotiation between uridecodebin and compositor

```log
0:00:05.336061100  6260 0000022653BB8FB0 WARN               h264parse gsth264parse.c:2241:gst_h264_parse_update_src_caps:<h264parse0> VUI framerate 15360.0 exceeds allowed maximum 32.8
New pad video/x-raw from source source-0
```

**Next Steps**: 
- Test with different video formats
- Check caps negotiation between source and compositor
- Verify framerate handling in pipeline setup


