# Current Bugs

## ✅ FIXED: Cannot shutdown (PRP-25 completed)

**Status**: RESOLVED as of 2025-08-23 via PRP-25 implementation

**Solution**: Replaced mixed event systems (Tokio + ctrlc + GStreamer) with proper GLib MainContext manual iteration pattern:
- Removed Tokio runtime from main.rs
- Replaced Application::run async method with run_with_main_context()  
- Used AtomicBool + ctrlc signal handler + glib::MainContext::default().wakeup()
- Replaced bus polling with bus.add_watch() callback pattern
- Manual main_context.iteration(true) loop that checks AtomicBool shutdown flag

**Validation**: 
- Shutdown tests now pass: `cargo test --test shutdown_test`
- Application runs for expected duration and exits cleanly with timeout
- No more repeated "Received interrupt signal, shutting down..." messages

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


