# Current Bugs

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


