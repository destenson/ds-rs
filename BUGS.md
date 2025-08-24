# Current Bugs

## Cannot shutdown

Currently, the application runs, opens a window, displays an image, and does not shut down cleanly when requested. It does not shut down when the window is closed. And it doesn't respond to Ctrl+C in the terminal, except with "Received interrupt signal, shutting down...".

```sh
$ cargo r --bin ds-app -- file://C:/Users/deste/Videos/wows-sm.1.mp4
   Compiling ds-rs v0.1.0 (C:\Users\deste\repos\ds-rs\crates\ds-rs)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.18s
     Running `target\debug\ds-app.exe file://C:/Users/deste/Videos/wows-sm.1.mp4`
DeepStream Rust - Runtime Source Addition/Deletion Demo
========================================================

INFO - Standard GStreamer elements detected, using standard backend
INFO - Initialized Standard GStreamer backend on X86 platform
INFO - Initialized Standard GStreamer backend on X86 platform
INFO - Initialized Standard GStreamer backend on X86 platform
Initializing pipeline with Standard GStreamer backend...
INFO - Standard backend: Using compositor for tiling
0:00:00.300195700  6260 0000022653ECE4D0 WARN         d3d11debuglayer gstd3d11device.cpp:779:gst_d3d11_device_dispose:<d3d11device0> DXGIInfoQueue: Live ID3D11Device at 0x0000022653EE9BE0, Refcount: 3
Adding source source-0 with URI: file://C:/Users/deste/Videos/wows-sm.1.mp4
Source source-0 state change ASYNC
Successfully added source source-0 - Total sources: 1
Emitting event: SourceAdded { id: SourceId(0), uri: "file://C:/Users/deste/Videos/wows-sm.1.mp4" }
0:00:05.336061100  6260 0000022653BB8FB0 WARN               h264parse gsth264parse.c:2241:gst_h264_parse_update_src_caps:<h264parse0> VUI framerate 15360.0 exceeds allowed maximum 32.8
New pad video/x-raw from source source-0
Linked source source-0 to streammux
0:00:05.415610800  6260 0000022653ECC150 FIXME               basesink gstbasesink.c:3399:gst_base_sink_default_event:<video-sink-actual-sink-d3d12video> stream-start event without group-id. Consider implementing group-id handling in the upstream elements

Received interrupt signal, shutting down...

Received interrupt signal, shutting down...

Received interrupt signal, shutting down...

Received interrupt signal, shutting down...

Received interrupt signal, shutting down...

Received interrupt signal, shutting down...
```

## Video doesn't play

Currently, the application runs, opens a window, displays an image, and does not play the video. It seems to get stuck after the first frame. It's possible it's the last frame, not the first.

The log implies that caps, specifically the framerate, are not being handled correctly, leading to the video not playing as expected.

```log
0:00:05.336061100  6260 0000022653BB8FB0 WARN               h264parse gsth264parse.c:2241:gst_h264_parse_update_src_caps:<h264parse0> VUI framerate 15360.0 exceeds allowed maximum 32.8
New pad video/x-raw from source source-0
```


