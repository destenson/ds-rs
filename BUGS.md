# Current Bugs

## False detections

The biggest problem we're seeing today is false detections in the ball tracking example. The CPU detector is detecting a lot of objects that aren't there, and this is causing the tracker to behave erratically.

```log
PS C:\Users\deste\repos\ds-rs> cargo r --example ball_tracking_visualization --all-features
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.54s
     Running `target\debug\examples\ball_tracking_visualization.exe`
[2025-08-25T20:49:58Z INFO  ball_tracking_visualization] [1756154998.453] Ball Tracking Visualization Example
[2025-08-25T20:49:58Z INFO  ball_tracking_visualization] [1756154998.454] =====================================
[2025-08-25T20:49:58Z INFO  ds_rs] Successfully registered custom CPU detector element
[2025-08-25T20:49:58Z INFO  ds_rs::backend::detector] Standard GStreamer elements detected, using standard backend
[2025-08-25T20:49:58Z INFO  ds_rs::backend] Initialized Standard GStreamer backend on X86 platform
[2025-08-25T20:49:58Z INFO  ball_tracking_visualization] [1756154998.712] Using Standard GStreamer backend
[2025-08-25T20:49:58Z INFO  ds_rs::backend] Initialized Standard GStreamer backend on X86 platform
[2025-08-25T20:49:58Z INFO  ds_rs::backend::standard] Standard backend: Using compositor for tiling
[2025-08-25T20:49:58Z INFO  ds_rs::backend::standard] Standard backend: Setting ONNX model path: crates/ds-rs/models/yolov5n.onnx
0:00:00.263598400 14284 000002CD46D59500 INFO             cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:283:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as glib::subclass::object::ObjectImpl>::set_property:<detector> Setting model path to: crates/ds-rs/models/yolov5n.onnx
[2025-08-25T20:49:58Z INFO  ds_rs::backend::standard] Standard backend: Created CPU detector with ONNX model
[2025-08-25T20:49:58Z INFO  ds_rs::backend::cpu_vision::elements] CPU tracker initialized with Centroid algorithm
[2025-08-25T20:49:58Z INFO  ds_rs::backend::standard] Standard backend: Using CPU tracker (Centroid algorithm)
[2025-08-25T20:49:58Z INFO  ds_rs::backend::cpu_vision::elements] CPU OSD using Cairo for rendering
[2025-08-25T20:49:58Z WARN  ds_rs::backend::cpu_vision::elements] CPU OSD created without metadata bridge - no detections will be rendered
[2025-08-25T20:49:58Z INFO  ds_rs::backend::standard] Standard backend: Using CPU OSD for visualization
[2025-08-25T20:49:58Z INFO  ds_rs::backend::standard] Standard backend: Using identity for tiler (tiling handled by compositor mux)
[2025-08-25T20:49:58Z INFO  ds_rs::pipeline::builder] Connected inference-results signal from detector to metadata bridge
[2025-08-25T20:49:58Z INFO  ds_rs::rendering] Creating Standard backend bounding box renderer
[2025-08-25T20:49:58Z INFO  ds_rs::rendering::standard_renderer] Standard renderer using Cairo for bounding box rendering
[2025-08-25T20:49:58Z INFO  ds_rs::rendering::standard_renderer] Cairo overlay created, but drawing callback not implemented without cairo-rs
[2025-08-25T20:49:58Z INFO  ds_rs::rendering::standard_renderer] Standard renderer created with Cairo overlay
[2025-08-25T20:49:58Z INFO  ds_rs::rendering::standard_renderer] Standard renderer connected to metadata source
[2025-08-25T20:49:58Z INFO  ds_rs::backend::cpu_vision::elements] Connecting metadata bridge to CPU OSD Cairo overlay
[2025-08-25T20:49:58Z INFO  ds_rs::pipeline::builder] Successfully connected metadata bridge to CPU OSD for Cairo rendering
[2025-08-25T20:49:58Z INFO  ds_rs::pipeline::builder] Configured OSD element 'osd' for dynamic rendering with Standard GStreamer backend
[2025-08-25T20:49:58Z INFO  ball_tracking_visualization] Using video: file:///C:/Users/deste/repos/ds-rs/crates/ds-rs/tests/test_video.mp4
[2025-08-25T20:49:58Z INFO  ball_tracking_visualization] [1756154998.756] Adding source: file:///C:/Users/deste/repos/ds-rs/crates/ds-rs/tests/test_video.mp4
[1756154998.756] Adding source source-0 with URI: file:///C:/Users/deste/repos/ds-rs/crates/ds-rs/tests/test_video.mp4
[1756154998.761] Connecting pad-added callback for source source-0
[1756154998.761] Syncing source source-0 state with parent pipeline
[1756154998.762] Source source-0 successfully synced with parent pipeline
Successfully added source source-0 - Total sources: 1
Emitting event: SourceAdded { id: SourceId(0), uri: "file:///C:/Users/deste/repos/ds-rs/crates/ds-rs/tests/test_video.mp4" }
[2025-08-25T20:49:58Z INFO  ball_tracking_visualization] [1756154998.763] Source SourceId(0) added to pipeline (URI: file:///C:/Users/deste/repos/ds-rs/crates/ds-rs/tests/test_video.mp4)
[2025-08-25T20:49:58Z INFO  ball_tracking_visualization] [1756154998.763] Starting ball tracking visualization pipeline
[2025-08-25T20:49:58Z INFO  tracing::span] apply_execution_providers;
[2025-08-25T20:49:58Z WARN  ort::execution_providers] No execution providers registered successfully. Falling back to CPU.
[2025-08-25T20:49:59Z INFO  ort::session] drop; self=SessionBuilder { env: "onnx_detector", allocator: Device, memory_type: Default }
0:00:00.586034300 14284 000002CD46D59500 INFO             cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:84:ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector::ensure_detector_loaded:<detector> Loaded ONNX detector from: crates/ds-rs/models/yolov5n.onnx
[1756154999.300] pad-added callback triggered for source source-0 (pad: src_0)
[1756154999.301] New pad video/x-raw from source source-0
[1756154999.308] Linked source source-0 through framerate normalizer to compositor
[1756154999.308] pad-added callback triggered for source source-0 (pad: src_1)
[1756154999.309] New pad audio/x-raw from source source-0
0:00:00.924390000 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 1
0:00:01.075030800 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 2
0:00:01.080007200 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 3
0:00:01.084057100 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 4
0:00:01.088947800 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 5
0:00:01.093711000 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 6
0:00:01.099097900 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 7
0:00:01.103624800 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 8
0:00:01.108763300 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 9
0:00:01.114006400 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 10
0:00:01.119495600 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 11
0:00:01.124195100 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 12
0:00:01.129247000 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 13
0:00:01.134051500 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 14
0:00:01.139909700 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 15
0:00:01.144601800 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 16
0:00:01.149116400 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 17
0:00:01.153871800 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 18
0:00:01.159628200 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 19
0:00:01.164074700 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 20
[2025-08-25T20:50:00Z INFO  ball_tracking_visualization] [1756155000.072] Frames: 0 | FPS: 0.0 | Objects rendered: 0 | Buffer: 0
[2025-08-25T20:50:00Z INFO  ort::memory] new_cpu; allocator=Arena memory_type=Default
[2025-08-25T20:50:00Z INFO  ort::memory] drop; self=MemoryInfo { ptr: 0x2cd7639d470, should_release: true }
0:00:02.035008000 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:427:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Frame 20: Detected 324 objects
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp] 🎆 Frame 20: Emitting 324 detections via signal
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 1: dog (class_id=16) at (0.0, 0.0) size=0.2626465x0.0023418905 conf=404814.00   
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 2: dog (class_id=16) at (0.0, 0.0) size=0.23232423x0.0029457093 conf=404814.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 3: dog (class_id=16) at (0.0, 0.0) size=0.23671876x0.008620835 conf=398157.00   
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 4: kite (class_id=33) at (0.0, 0.0) size=0.295459x0.002726841 conf=386262.00    
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 5: kite (class_id=33) at (0.0, 0.0) size=0.4171875x0.005654526 conf=376995.75   
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 6: kite (class_id=33) at (0.0, 0.0) size=0.15307617x0.0069986344 conf=375461.00 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 7: broccoli (class_id=50) at (0.0, 0.0) size=0.19306642x0.0060201646 conf=363000
.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 8: broccoli (class_id=50) at (0.0, 0.0) size=0.08811036x0.009383011 conf=339014.
50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 9: broccoli (class_id=50) at (0.0, 0.0) size=0.3254883x0.012936401 conf=291059.2
5
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 10: kite (class_id=33) at (0.0, 0.0) size=0.29370117x0.013204194 conf=283023.00 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 11: dog (class_id=16) at (0.0, 0.0) size=0.11601563x0.014728546 conf=274575.00  
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 12: kite (class_id=33) at (0.0, 0.0) size=0.03852539x0.0023058415 conf=205322.25
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 13: kite (class_id=33) at (0.0, 0.0) size=0.07631836x0.0033628463 conf=204756.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 14: dog (class_id=16) at (0.0, 0.0) size=0.065625004x0.0041971207 conf=204303.44
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 15: dog (class_id=16) at (0.0, 0.0) size=0.048999026x0.0016299248 conf=199027.12
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 16: broccoli (class_id=50) at (0.0, 0.0) size=0.045300294x0.0037053109 conf=1981
36.25
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 17: cell phone (class_id=67) at (0.0, 0.0) size=0.102832034x0.0036486627 conf=19
6721.25
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 18: dog (class_id=16) at (0.0, 0.0) size=0.1307373x0.0014393807 conf=194368.88  
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 19: broccoli (class_id=50) at (0.0, 0.0) size=0.11264649x0.0037645341 conf=19118
7.56
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 20: cell phone (class_id=67) at (0.0, 0.0) size=0.07104492x0.0016299248 conf=186
048.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 21: kite (class_id=33) at (0.0, 0.0) size=0.09660645x0.00096688274 conf=176925.3
8
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 22: cell phone (class_id=67) at (0.0, 0.0) size=0.043615725x0.0021667958 conf=17
0528.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 23: dog (class_id=16) at (0.0, 0.0) size=0.12539063x0.002287817 conf=170362.31  
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 24: dog (class_id=16) at (0.0, 0.0) size=0.110815436x0.0009784698 conf=169536.50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 25: broccoli (class_id=50) at (0.0, 0.0) size=0.12275391x0.00044288635 conf=1692
26.75
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 26: broccoli (class_id=50) at (0.0, 0.0) size=0.12172852x0.00073900225 conf=1625
04.12
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 27: cell phone (class_id=67) at (0.0, 0.0) size=0.06551514x0.0005281806 conf=152
626.12
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 28: kite (class_id=33) at (0.0, 0.0) size=0.11601563x0.0028452873 conf=145446.88
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 29: cell phone (class_id=67) at (0.0, 0.0) size=0.08811036x0.0013569832 conf=143
112.38
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 30: cell phone (class_id=67) at (0.0, 0.0) size=0.29370117x0.0015565396 conf=135
460.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 31: cell phone (class_id=67) at (0.0, 0.0) size=0.2036133x0.0017972946 conf=1271
32.91
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 32: broccoli (class_id=50) at (0.0, 0.0) size=0.09697266x0.0030384064 conf=12673
5.75
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 33: cell phone (class_id=67) at (0.0, 0.0) size=0.16362305x0.002581358 conf=1265
00.72
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 34: cell phone (class_id=67) at (0.0, 0.0) size=0.08063965x0.00471468 conf=11121
4.88
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 35: cell phone (class_id=67) at (0.0, 0.0) size=0.12626953x0.0033885955 conf=965
25.19
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 36: dog (class_id=16) at (0.0, 0.0) size=0.0046806335x0.016324997 conf=85780.50 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 37: dog (class_id=16) at (0.0, 0.0) size=0.0023666383x0.05912018 conf=84588.00  
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 38: dog (class_id=16) at (0.0, 0.0) size=0.0038063051x0.0062776566 conf=84375.62
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 39: dog (class_id=16) at (0.0, 0.0) size=0.0009916306x0.04247589 conf=84124.31  
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 40: dog (class_id=16) at (0.0, 0.0) size=0.0017120362x0.019826889 conf=84111.00 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 41: dog (class_id=16) at (0.0, 0.0) size=0.0065139774x0.03341217 conf=83568.25  
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 42: dog (class_id=16) at (0.0, 0.0) size=0.0028656006x0.00202775 conf=83395.50  
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 43: cell phone (class_id=67) at (0.0, 0.0) size=0.13212891x0.0029122352 conf=833
80.94
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 44: dog (class_id=16) at (0.0, 0.0) size=0.0027030946x0.0017020226 conf=83157.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 45: dog (class_id=16) at (0.0, 0.0) size=0.0030853273x0.0047507286 conf=83157.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 46: kite (class_id=33) at (0.0, 0.0) size=0.0038520815x0.023071289 conf=77162.25
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 47: kite (class_id=33) at (0.0, 0.0) size=0.003037262x0.009491158 conf=77087.62 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 48: cell phone (class_id=67) at (0.0, 0.0) size=0.17387696x0.0059274673 conf=769
67.19
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 49: dog (class_id=16) at (0.0, 0.0) size=0.006074524x0.006529999 conf=76958.88  
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 50: kite (class_id=33) at (0.0, 0.0) size=0.0029548646x0.05100403 conf=76709.88
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 51: kite (class_id=33) at (0.0, 0.0) size=0.0027236938x0.0032598495 conf=76451.3
8
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 52: kite (class_id=33) at (0.0, 0.0) size=0.0017051698x0.0018977165 conf=76322.1
2
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 53: kite (class_id=33) at (0.0, 0.0) size=0.0015529633x0.002483511 conf=76183.75
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 54: kite (class_id=33) at (0.0, 0.0) size=0.0013710023x0.022844696 conf=76174.50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 55: kite (class_id=33) at (0.0, 0.0) size=0.0063629155x0.002791214 conf=71225.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 56: cell phone (class_id=67) at (0.0, 0.0) size=0.22514649x0.002287817 conf=5873
4.28
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 57: cell phone (class_id=67) at (0.0, 0.0) size=0.1130127x0.002992058 conf=53748
.75
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 58: cell phone (class_id=67) at (0.0, 0.0) size=0.2072754x0.00085036753 conf=537
47.56
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 59: cell phone (class_id=67) at (0.0, 0.0) size=0.19072266x0.0007866383 conf=490
23.44
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 60: cell phone (class_id=67) at (0.0, 0.0) size=0.13652344x0.00080788136 conf=44
141.56
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 61: cell phone (class_id=67) at (0.0, 0.0) size=0.35126954x0.0011394024 conf=436
21.44
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 62: cell phone (class_id=67) at (0.0, 0.0) size=0.31464845x0.0012604237 conf=399
02.84
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 63: cell phone (class_id=67) at (0.0, 0.0) size=0.12648927x0.0007357836 conf=391
96.88
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 64: cell phone (class_id=67) at (0.0, 0.0) size=0.25532228x0.0011844635 conf=355
08.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 65: cell phone (class_id=67) at (0.0, 0.0) size=0.1987793x0.0007216215 conf=3540
8.44
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 66: cell phone (class_id=67) at (0.0, 0.0) size=0.15878907x0.001290679 conf=3082
9.39
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 67: cell phone (class_id=67) at (0.0, 0.0) size=0.19555666x0.0026225566 conf=270
42.09
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 68: cell phone (class_id=67) at (0.0, 0.0) size=0.21723634x0.0043953895 conf=546
6.09
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 69: cell phone (class_id=67) at (0.0, 0.0) size=0.0012931824x0.005355835 conf=41
48.16
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 70: cell phone (class_id=67) at (0.0, 0.0) size=0.037866212x0.008074951 conf=398
3.67
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 71: cell phone (class_id=67) at (0.0, 0.0) size=0.018585205x0.0019659519 conf=39
03.59
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 72: cell phone (class_id=67) at (0.0, 0.0) size=0.028527834x0.0009372711 conf=37
98.11
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 73: cell phone (class_id=67) at (0.0, 0.0) size=0.0018013001x0.0028452873 conf=3
546.82
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 74: cell phone (class_id=67) at (0.0, 0.0) size=0.0015289307x0.0030744553 conf=3
321.84
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 75: cell phone (class_id=67) at (0.0, 0.0) size=0.029205324x0.00023624898 conf=3
279.66
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 76: cell phone (class_id=67) at (0.0, 0.0) size=0.0016857148x0.005422783 conf=32
46.77
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 77: cell phone (class_id=67) at (0.0, 0.0) size=0.00033102036x0.003929329 conf=3
227.55
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 78: cell phone (class_id=67) at (0.0, 0.0) size=0.00067405705x0.004312992 conf=3
167.36
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 79: cell phone (class_id=67) at (0.0, 0.0) size=0.0019180299x0.0009939194 conf=3
161.58
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 80: cell phone (class_id=67) at (0.0, 0.0) size=0.02975464x0.00069394114 conf=31
09.50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 81: cell phone (class_id=67) at (0.0, 0.0) size=0.0002723694x0.0046786307 conf=3
102.72
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 82: cell phone (class_id=67) at (0.0, 0.0) size=0.0004088402x0.00552578 conf=305
7.98
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 83: cell phone (class_id=67) at (0.0, 0.0) size=0.0018585206x0.0011986255 conf=3
035.52
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 84: cell phone (class_id=67) at (0.0, 0.0) size=0.017404176x0.0027165413 conf=30
29.13
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 85: cell phone (class_id=67) at (0.0, 0.0) size=0.0022670748x0.0070243836 conf=3
015.11
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 86: cell phone (class_id=67) at (0.0, 0.0) size=0.001476288x0.00437994 conf=2982
.27
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 87: cell phone (class_id=67) at (0.0, 0.0) size=0.00030612946x0.003537941 conf=2
978.55
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 88: cell phone (class_id=67) at (0.0, 0.0) size=0.0021549226x0.0011799574 conf=2
916.10
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 89: cell phone (class_id=67) at (0.0, 0.0) size=0.00073413854x0.0036924363 conf=
2865.88
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 90: cell phone (class_id=67) at (0.0, 0.0) size=0.00036363603x0.0073591233 conf=
2817.73
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 91: cell phone (class_id=67) at (0.0, 0.0) size=0.00067634584x0.0036486627 conf=
2802.55
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 92: cell phone (class_id=67) at (0.0, 0.0) size=0.0010934831x0.011195756 conf=27
75.85
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 93: cell phone (class_id=67) at (0.0, 0.0) size=0.00036935808x0.0036769868 conf=
2764.86
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 94: cell phone (class_id=67) at (0.0, 0.0) size=0.0013240814x0.00038945675 conf=
2733.82
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 95: cell phone (class_id=67) at (0.0, 0.0) size=0.0022945404x0.0045344355 conf=2
733.66
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 96: cell phone (class_id=67) at (0.0, 0.0) size=0.030212404x0.00015441478 conf=2
729.27
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 97: cell phone (class_id=67) at (0.0, 0.0) size=0.0007089615x0.006602097 conf=27
00.89
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 98: cell phone (class_id=67) at (0.0, 0.0) size=0.0010110856x0.002502823 conf=26
78.16
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 99: cell phone (class_id=67) at (0.0, 0.0) size=0.0007398606x0.007858658 conf=26
77.76
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 100: cell phone (class_id=67) at (0.0, 0.0) size=0.000917244x0.0010994911 conf=2
674.83
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 101: cell phone (class_id=67) at (0.0, 0.0) size=0.0013446808x0.001636362 conf=2
674.36
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 102: cell phone (class_id=67) at (0.0, 0.0) size=0.0021549226x0.0020045757 conf=
2607.66
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 103: cell phone (class_id=67) at (0.0, 0.0) size=0.0011146546x0.0003254056 conf=
2600.35
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 104: cell phone (class_id=67) at (0.0, 0.0) size=0.0011020661x0.00030689835 conf
=2593.68
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 105: cell phone (class_id=67) at (0.0, 0.0) size=0.0019706727x0.0012314558 conf=
2592.59
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 106: cell phone (class_id=67) at (0.0, 0.0) size=0.0004852295x0.0031594278 conf=
2585.09
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 107: cell phone (class_id=67) at (0.0, 0.0) size=0.00028982165x0.0031002045 conf
=2579.41
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 108: cell phone (class_id=67) at (0.0, 0.0) size=0.0008960724x0.00068364147 conf
=2556.28
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 109: cell phone (class_id=67) at (0.0, 0.0) size=0.00058078766x0.0053764344 conf
=2554.98
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 110: cell phone (class_id=67) at (0.0, 0.0) size=0.0002519131x0.0055051805 conf=
2541.77
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 111: cell phone (class_id=67) at (0.0, 0.0) size=0.0012485505x0.00033441783 conf
=2533.70
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 112: cell phone (class_id=67) at (0.0, 0.0) size=0.0014533997x0.00007644296 conf
=2527.95
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 113: cell phone (class_id=67) at (0.0, 0.0) size=0.0016857148x0.000030416251 con
f=2515.26
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 114: cell phone (class_id=67) at (0.0, 0.0) size=0.020141602x0.0014226437 conf=2
501.47
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 115: cell phone (class_id=67) at (0.0, 0.0) size=0.00031843188x0.002855587 conf=
2495.93
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 116: cell phone (class_id=67) at (0.0, 0.0) size=0.0002519131x0.0021835328 conf=
2489.52
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 117: cell phone (class_id=67) at (0.0, 0.0) size=0.0020725252x0.00275774 conf=24
68.50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 118: cell phone (class_id=67) at (0.0, 0.0) size=0.00026807786x0.0030744553 conf
=2468.29
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 119: cell phone (class_id=67) at (0.0, 0.0) size=0.000349617x0.0029019357 conf=2
446.20
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 120: cell phone (class_id=67) at (0.0, 0.0) size=0.001015091x0.00085680484 conf=
2437.50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 121: cell phone (class_id=67) at (0.0, 0.0) size=0.00082569127x0.0060201646 conf
=2437.44
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 122: cell phone (class_id=67) at (0.0, 0.0) size=0.0026504518x0.00012697578 conf
=2411.02
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 123: cell phone (class_id=67) at (0.0, 0.0) size=0.0032707215x0.00070745946 conf
=2409.75
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 124: cell phone (class_id=67) at (0.0, 0.0) size=0.03438721x0.0006778479 conf=24
08.30
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 125: cell phone (class_id=67) at (0.0, 0.0) size=0.0014705659x0.005613327 conf=2
405.76
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 126: cell phone (class_id=67) at (0.0, 0.0) size=0.0004723549x0.00021597148 conf
=2398.16
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 127: cell phone (class_id=67) at (0.0, 0.0) size=0.00092811586x0.00068621634 con
f=2390.66
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 128: cell phone (class_id=67) at (0.0, 0.0) size=0.0014419556x0.005154991 conf=2
383.15
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 129: cell phone (class_id=67) at (0.0, 0.0) size=0.0005876541x0.0022182942 conf=
2379.32
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 130: cell phone (class_id=67) at (0.0, 0.0) size=0.0007398606x0.0036486627 conf=
2370.28
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 131: cell phone (class_id=67) at (0.0, 0.0) size=0.00042514803x0.00033860208 con
f=2367.70
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 132: cell phone (class_id=67) at (0.0, 0.0) size=0.027575685x0.0003204167 conf=2
358.17
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 133: cell phone (class_id=67) at (0.0, 0.0) size=0.0020565034x0.000033916534 con
f=2355.94
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 134: cell phone (class_id=67) at (0.0, 0.0) size=0.0016147614x0.007559967 conf=2
335.31
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 135: cell phone (class_id=67) at (0.0, 0.0) size=0.029260255x0.0001438737 conf=2
326.64
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 136: cell phone (class_id=67) at (0.0, 0.0) size=0.0020645142x0.00015803576 conf
=2312.40
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 137: cell phone (class_id=67) at (0.0, 0.0) size=0.00040569308x0.0011085033 conf
=2301.35
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 138: cell phone (class_id=67) at (0.0, 0.0) size=0.0029777528x0.00779686 conf=22
96.44
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 139: cell phone (class_id=67) at (0.0, 0.0) size=0.0004929543x0.00018330217 conf
=2295.16
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 140: cell phone (class_id=67) at (0.0, 0.0) size=0.0008686066x0.0008471489 conf=
2290.02
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 141: cell phone (class_id=67) at (0.0, 0.0) size=0.0015346528x0.0005001784 conf=
2274.91
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 142: cell phone (class_id=67) at (0.0, 0.0) size=0.0002966881x0.0014561176 conf=
2273.82
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 143: cell phone (class_id=67) at (0.0, 0.0) size=0.0004471779x0.00021935106 conf
=2265.24
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 144: cell phone (class_id=67) at (0.0, 0.0) size=0.0004102707x0.00202775 conf=22
63.57
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 145: cell phone (class_id=67) at (0.0, 0.0) size=0.00050268177x0.002531147 conf=
2256.88
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 146: cell phone (class_id=67) at (0.0, 0.0) size=0.00026807786x0.0014445306 conf
=2256.47
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 147: cell phone (class_id=67) at (0.0, 0.0) size=0.0015060426x0.0012359619 conf=
2256.38
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 148: cell phone (class_id=67) at (0.0, 0.0) size=0.00067920686x0.00019215346 con
f=2256.24
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 149: cell phone (class_id=67) at (0.0, 0.0) size=0.00032587053x0.0014561176 conf
=2256.06
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 150: cell phone (class_id=67) at (0.0, 0.0) size=0.0007455826x0.0005301118 conf=
2242.41
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 151: cell phone (class_id=67) at (0.0, 0.0) size=0.0003084183x0.0015501023 conf=
2236.20
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 152: cell phone (class_id=67) at (0.0, 0.0) size=0.00026807786x0.0012269497 conf
=2233.30
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 153: cell phone (class_id=67) at (0.0, 0.0) size=0.00060653687x0.0017213345 conf
=2228.05
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 154: cell phone (class_id=67) at (0.0, 0.0) size=0.0005066872x0.000115630035 con
f=2220.87
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 155: cell phone (class_id=67) at (0.0, 0.0) size=0.0017189026x0.005613327 conf=2
208.36
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 156: cell phone (class_id=67) at (0.0, 0.0) size=0.00031843188x0.0008304119 conf
=2207.54
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 157: cell phone (class_id=67) at (0.0, 0.0) size=0.0023117065x0.00021098256 conf
=2204.01
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 158: broccoli (class_id=50) at (0.0, 0.0) size=0.0027328492x0.007302475 conf=220
0.81
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 159: cell phone (class_id=67) at (0.0, 0.0) size=0.03284912x0.002522135 conf=219
1.16
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 160: cell phone (class_id=67) at (0.0, 0.0) size=0.0004835129x0.00013952852 conf
=2186.48
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 161: cell phone (class_id=67) at (0.0, 0.0) size=0.00031356813x0.0011612893 conf
=2165.50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 162: cell phone (class_id=67) at (0.0, 0.0) size=0.0024238587x0.000072379415 con
f=2152.64
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 163: cell phone (class_id=67) at (0.0, 0.0) size=0.00062541966x0.0023792267 conf
=2146.88
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 164: cell phone (class_id=67) at (0.0, 0.0) size=0.00064029696x0.0017148972 conf
=2125.81
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 165: cell phone (class_id=67) at (0.0, 0.0) size=0.030963136x0.00005133748 conf=
2121.23
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 166: broccoli (class_id=50) at (0.0, 0.0) size=0.0023849488x0.007621765 conf=211
1.62
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 167: cell phone (class_id=67) at (0.0, 0.0) size=0.00044202807x0.00013840199 con
f=2091.26
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 168: cell phone (class_id=67) at (0.0, 0.0) size=0.0013389587x0.005654526 conf=2
075.52
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 169: cell phone (class_id=67) at (0.0, 0.0) size=0.0015117646x0.006756592 conf=2
074.61
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 170: cell phone (class_id=67) at (0.0, 0.0) size=0.0018653871x0.0000813514 conf=
2047.81
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 171: cell phone (class_id=67) at (0.0, 0.0) size=0.00055646896x0.0006649733 conf
=1976.97
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 172: cell phone (class_id=67) at (0.0, 0.0) size=0.000433445x0.0004193902 conf=1
951.16
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 173: broccoli (class_id=50) at (0.0, 0.0) size=0.0010190965x0.00048279762 conf=1
924.39
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 174: cell phone (class_id=67) at (0.0, 0.0) size=0.0017944337x0.000080104175 con
f=1893.92
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 175: cell phone (class_id=67) at (0.0, 0.0) size=0.0151519785x0.0006778479 conf=
1888.09
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 176: cell phone (class_id=67) at (0.0, 0.0) size=0.0016925812x0.0001147449 conf=
1881.03
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 177: broccoli (class_id=50) at (0.0, 0.0) size=0.0022590638x0.0077402117 conf=18
69.11
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 178: cell phone (class_id=67) at (0.0, 0.0) size=0.0002421856x0.0017972946 conf=
1863.79
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 179: cell phone (class_id=67) at (0.0, 0.0) size=0.002080536x0.000059062244 conf
=1859.76
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 180: cell phone (class_id=67) at (0.0, 0.0) size=0.0006557465x0.0054639815 conf=
1854.81
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 181: cell phone (class_id=67) at (0.0, 0.0) size=0.0028312684x0.0006997347 conf=
1854.47
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 182: cell phone (class_id=67) at (0.0, 0.0) size=0.033856202x0.0000409171 conf=1
853.03
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 183: cell phone (class_id=67) at (0.0, 0.0) size=0.002206421x0.00014274717 conf=
1852.02
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 184: broccoli (class_id=50) at (0.0, 0.0) size=0.0009315491x0.002116585 conf=185
1.51
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 185: cell phone (class_id=67) at (0.0, 0.0) size=0.001253128x0.00011208952 conf=
1846.81
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 186: cell phone (class_id=67) at (0.0, 0.0) size=0.022338867x0.005973816 conf=18
32.65
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 187: broccoli (class_id=50) at (0.0, 0.0) size=0.0005876541x0.0009372711 conf=18
13.21
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 188: cell phone (class_id=67) at (0.0, 0.0) size=0.0005945206x0.0013260841 conf=
1811.42
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 189: cell phone (class_id=67) at (0.0, 0.0) size=0.022210695x0.00032170417 conf=
1809.96
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 190: cell phone (class_id=67) at (0.0, 0.0) size=0.031622317x0.00013952852 conf=
1805.86
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 191: broccoli (class_id=50) at (0.0, 0.0) size=0.00071182253x0.00022466182 conf=
1784.89
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 192: cell phone (class_id=67) at (0.0, 0.0) size=0.0004199982x0.0009121657 conf=
1781.32
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 193: cell phone (class_id=67) at (0.0, 0.0) size=0.0007169724x0.0023702146 conf=
1776.45
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 194: broccoli (class_id=50) at (0.0, 0.0) size=0.00067405705x0.00022466182 conf=
1758.24
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 195: cell phone (class_id=67) at (0.0, 0.0) size=0.00072555547x0.0011612893 conf
=1748.28
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 196: broccoli (class_id=50) at (0.0, 0.0) size=0.000668335x0.0011394024 conf=174
7.88
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 197: broccoli (class_id=50) at (0.0, 0.0) size=0.0006185532x0.0011394024 conf=17
42.50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 198: cell phone (class_id=67) at (0.0, 0.0) size=0.04650879x0.00017766953 conf=1
731.79
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 199: broccoli (class_id=50) at (0.0, 0.0) size=0.00041370394x0.003537941 conf=17
10.96
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 200: cell phone (class_id=67) at (0.0, 0.0) size=0.038891602x0.0014844418 conf=1
630.77
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 201: cell phone (class_id=67) at (0.0, 0.0) size=0.012487793x0.00096688274 conf=
1608.56
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 202: cell phone (class_id=67) at (0.0, 0.0) size=0.013339234x0.012812805 conf=16
04.11
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 203: cell phone (class_id=67) at (0.0, 0.0) size=0.00082569127x0.002323866 conf=
1600.40
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 204: cell phone (class_id=67) at (0.0, 0.0) size=0.024865724x0.00028951766 conf=
1596.04
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 205: cell phone (class_id=67) at (0.0, 0.0) size=0.027520753x0.00020615458 conf=
1567.50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 206: broccoli (class_id=50) at (0.0, 0.0) size=0.027209474x0.008620835 conf=1566
.12
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 207: kite (class_id=33) at (0.0, 0.0) size=0.00050468446x0.0027165413 conf=1549.
00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 208: cell phone (class_id=67) at (0.0, 0.0) size=0.02006836x0.00018475056 conf=1
540.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 209: cell phone (class_id=67) at (0.0, 0.0) size=0.019152833x0.0008941412 conf=1
529.62
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 210: cell phone (class_id=67) at (0.0, 0.0) size=0.024682619x0.00007825345 conf=
1510.64
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 211: cell phone (class_id=67) at (0.0, 0.0) size=0.025781251x0.00008529425 conf=
1489.64
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 212: cell phone (class_id=67) at (0.0, 0.0) size=0.025689699x0.0036769868 conf=1
477.25
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 213: kite (class_id=33) at (0.0, 0.0) size=0.0004740715x0.0010872603 conf=1472.3
4
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 214: cell phone (class_id=67) at (0.0, 0.0) size=0.031567384x0.0003955722 conf=1
452.50
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 215: cell phone (class_id=67) at (0.0, 0.0) size=0.025250245x0.00023914575 conf=
1452.20
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 216: kite (class_id=33) at (0.0, 0.0) size=0.00036649706x0.0010537863 conf=1450.
30
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 217: kite (class_id=33) at (0.0, 0.0) size=0.00055646896x0.0014277935 conf=1432.
73
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 218: broccoli (class_id=50) at (0.0, 0.0) size=0.052441407x0.0067050937 conf=141
9.03
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 219: kite (class_id=33) at (0.0, 0.0) size=0.00058994297x0.0015372277 conf=1385.
97
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 220: cell phone (class_id=67) at (0.0, 0.0) size=0.02644043x0.0012076378 conf=13
80.57
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 221: cell phone (class_id=67) at (0.0, 0.0) size=0.05214844x0.0064785006 conf=13
73.02
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 222: cell phone (class_id=67) at (0.0, 0.0) size=0.03039551x0.0011754513 conf=13
59.38
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 223: cell phone (class_id=67) at (0.0, 0.0) size=0.010821533x0.0017908573 conf=1
348.31
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 224: cell phone (class_id=67) at (0.0, 0.0) size=0.030047609x0.003311348 conf=12
89.51
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 225: kite (class_id=33) at (0.0, 0.0) size=0.012487793x0.0022620677 conf=1289.11
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 226: kite (class_id=33) at (0.0, 0.0) size=0.001387024x0.0061849593 conf=1275.93
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 227: cell phone (class_id=67) at (0.0, 0.0) size=0.039843753x0.0010537863 conf=1
270.96
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 228: cell phone (class_id=67) at (0.0, 0.0) size=0.0368042x0.001496029 conf=1260
.68
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 229: broccoli (class_id=50) at (0.0, 0.0) size=0.018804932x0.0011168718 conf=121
7.90
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 230: cell phone (class_id=67) at (0.0, 0.0) size=0.018438721x0.00028951766 conf=
1209.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 231: cell phone (class_id=67) at (0.0, 0.0) size=0.016488649x0.0018037319 conf=1
139.10
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 232: broccoli (class_id=50) at (0.0, 0.0) size=0.0012680055x0.005613327 conf=112
0.05
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 233: kite (class_id=33) at (0.0, 0.0) size=0.031384278x0.0035920143 conf=1115.98
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 234: cell phone (class_id=67) at (0.0, 0.0) size=0.024591066x0.0007357836 conf=1
098.02
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 235: kite (class_id=33) at (0.0, 0.0) size=0.02874756x0.0022788048 conf=1096.55 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 236: broccoli (class_id=50) at (0.0, 0.0) size=0.001272583x0.005613327 conf=1064
.31
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 237: broccoli (class_id=50) at (0.0, 0.0) size=0.030963136x0.0016492367 conf=106
4.29
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 238: broccoli (class_id=50) at (0.0, 0.0) size=0.02340088x0.0014226437 conf=1031
.59
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 239: broccoli (class_id=50) at (0.0, 0.0) size=0.021752931x0.00032170417 conf=10
05.56
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 240: cell phone (class_id=67) at (0.0, 0.0) size=0.021661378x0.00020454526 conf=
983.88
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 241: cell phone (class_id=67) at (0.0, 0.0) size=0.001398468x0.0055927276 conf=9
81.45
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 242: kite (class_id=33) at (0.0, 0.0) size=0.04116211x0.005294037 conf=962.86   
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 243: cell phone (class_id=67) at (0.0, 0.0) size=0.026184084x0.00038044454 conf=
933.19
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 244: cell phone (class_id=67) at (0.0, 0.0) size=0.0014533997x0.00024864078 conf
=748.68
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 245: cell phone (class_id=67) at (0.0, 0.0) size=0.035852052x0.0007924318 conf=7
02.24
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 246: kite (class_id=33) at (0.0, 0.0) size=0.04152832x0.0015320778 conf=626.78
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 247: cell phone (class_id=67) at (0.0, 0.0) size=0.027154543x0.0009121657 conf=5
60.34
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 248: cell phone (class_id=67) at (0.0, 0.0) size=0.026092531x0.0001982689 conf=5
56.10
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 249: hot dog (class_id=52) at (0.0, 0.0) size=0.00044202807x0.0015681267 conf=40
9.25
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 250: oven (class_id=69) at (0.0, 0.0) size=0.00050268177x0.0017831326 conf=222.2
7
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 251: oven (class_id=69) at (0.0, 0.0) size=0.00055217743x0.006401253 conf=87.73 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 252: mouse (class_id=64) at (0.0, 12.8) size=0.006216431x0.00061991217 conf=63.6
1
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 253: mouse (class_id=64) at (0.0, 13.4) size=0.0068756104x0.0004535079 conf=58.3
9
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 254: mouse (class_id=64) at (0.0, 13.7) size=0.0033348084x0.0008046627 conf=56.0
7
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 255: mouse (class_id=64) at (0.0, 13.4) size=0.003048706x0.00044127702 conf=55.5
6
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 256: mouse (class_id=64) at (0.0, 15.1) size=0.0076950076x0.00075638294 conf=54.
11
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 257: mouse (class_id=64) at (0.0, 16.0) size=0.006312561x0.0004396677 conf=54.00
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 258: mouse (class_id=64) at (0.0, 10.3) size=0.0067428593x0.0013466835 conf=50.8
0
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 259: mouse (class_id=64) at (0.0, 14.0) size=0.006770325x0.0015990258 conf=50.39
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 260: mouse (class_id=64) at (0.0, 14.2) size=0.0041793827x0.0006887913 conf=50.3
5
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 261: mouse (class_id=64) at (0.0, 13.8) size=0.004147339x0.0007866383 conf=49.81
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 262: mouse (class_id=64) at (0.0, 13.2) size=0.0034950257x0.00054073334 conf=49.
80
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 263: mouse (class_id=64) at (0.0, 14.8) size=0.0055297855x0.00054073334 conf=49.
42
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 264: mouse (class_id=64) at (0.0, 12.8) size=0.0010391236x0.00074479583 conf=48.
68
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 265: mouse (class_id=64) at (0.0, 14.0) size=0.0049209595x0.0005642295 conf=48.5
7
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 266: mouse (class_id=64) at (0.0, 13.4) size=0.0014705659x0.0011567831 conf=47.3
2
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 267: mouse (class_id=64) at (0.0, 12.3) size=0.0023666383x0.0011986255 conf=47.1
1
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 268: mouse (class_id=64) at (0.0, 8.5) size=0.0056625367x0.001636362 conf=47.04 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 269: mouse (class_id=64) at (0.0, 12.0) size=0.0052368166x0.0011085033 conf=46.2
7
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 270: mouse (class_id=64) at (0.0, 16.4) size=0.0076080323x0.0011844635 conf=46.1
6
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 271: skis (class_id=30) at (0.0, 11.8) size=0.0068756104x0.00057324173 conf=45.9
7
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 272: mouse (class_id=64) at (0.0, 11.1) size=0.0070678713x0.0011303902 conf=45.4
4
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 273: mouse (class_id=64) at (0.0, 15.5) size=0.0061660768x0.0011213779 conf=45.3
0
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 274: mouse (class_id=64) at (0.0, 11.5) size=0.002989197x0.00070745946 conf=44.7
9
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 275: mouse (class_id=64) at (0.0, 10.0) size=0.0062622074x0.0019054413 conf=44.7
8
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 276: mouse (class_id=64) at (0.0, 16.3) size=0.008546448x0.001331234 conf=44.53 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 277: mouse (class_id=64) at (0.0, 15.0) size=0.0022327425x0.0007679701 conf=44.4
0
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 278: mouse (class_id=64) at (0.0, 12.0) size=0.0035087587x0.0011303902 conf=44.1
9
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 279: mouse (class_id=64) at (0.0, 11.3) size=0.0032569887x0.0007415772 conf=43.5
3
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 280: mouse (class_id=64) at (0.0, 13.8) size=0.006463623x0.0011754513 conf=43.02
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 281: mouse (class_id=64) at (0.0, 12.1) size=0.0024124146x0.0012314558 conf=42.9
8
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 282: mouse (class_id=64) at (0.0, 9.0) size=0.004902649x0.0022350312 conf=42.14 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 283: skis (class_id=30) at (0.0, 13.8) size=0.0059555057x0.0006778479 conf=41.68
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 284: mouse (class_id=64) at (0.0, 16.7) size=0.0028541565x0.00046799183 conf=41.
05
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 285: mouse (class_id=64) at (0.0, 15.9) size=0.0016273499x0.0011567831 conf=40.6
3
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 286: mouse (class_id=64) at (0.0, 15.4) size=0.009750366x0.0015320778 conf=40.59
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 287: mouse (class_id=64) at (0.0, 13.1) size=0.0047882083x0.0013209343 conf=40.4
1
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 288: mouse (class_id=64) at (0.0, 12.3) size=0.0026184083x0.0011477709 conf=40.3
7
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 289: mouse (class_id=64) at (0.0, 14.8) size=0.0058410647x0.0017625332 conf=39.9
5
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 290: mouse (class_id=64) at (0.0, 11.5) size=0.0046073915x0.0011754513 conf=39.3
3
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 291: mouse (class_id=64) at (0.0, 16.5) size=0.0051589967x0.0011258841 conf=38.4
7
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 292: mouse (class_id=64) at (0.0, 13.8) size=0.007260132x0.0017483712 conf=38.36
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 293: mouse (class_id=64) at (0.0, 13.5) size=0.009484864x0.0016427994 conf=38.32
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 294: mouse (class_id=64) at (0.0, 16.1) size=0.0035636902x0.0012314558 conf=38.2
2
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 295: mouse (class_id=64) at (0.0, 11.0) size=0.0049621584x0.0018681049 conf=38.1
7
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 296: mouse (class_id=64) at (0.0, 11.9) size=0.004035187x0.0019505024 conf=38.01
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 297: mouse (class_id=64) at (0.0, 13.0) size=0.011691284x0.0016492367 conf=37.65
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 298: mouse (class_id=64) at (0.0, 11.4) size=0.017605592x0.0013685704 conf=37.61
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 299: mouse (class_id=64) at (0.0, 13.8) size=0.0042961123x0.0012855291 conf=36.7
5
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 300: mouse (class_id=64) at (0.0, 12.5) size=0.0022235871x0.0011754513 conf=36.4
0
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 301: skis (class_id=30) at (0.0, 11.9) size=0.0070404056x0.0005754948 conf=36.31
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 302: mouse (class_id=64) at (0.0, 12.2) size=0.006312561x0.00075058936 conf=36.0
1
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 303: mouse (class_id=64) at (0.0, 15.5) size=0.011425782x0.0012269497 conf=35.72
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 304: mouse (class_id=64) at (0.0, 14.0) size=0.0020484924x0.0005960941 conf=35.0
0
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 305: mouse (class_id=64) at (0.0, 14.7) size=0.0049209595x0.0006701231 conf=34.6
0
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 306: mouse (class_id=64) at (0.0, 13.0) size=0.011077881x0.0016685486 conf=34.55
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 307: mouse (class_id=64) at (0.0, 14.1) size=0.0023849488x0.00077118876 conf=33.
58
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 308: mouse (class_id=64) at (0.0, 15.4) size=0.0047515873x0.0012803794 conf=33.4
7
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 309: apple (class_id=47) at (0.0, 11.5) size=0.0064132693x0.00070745946 conf=33.
42
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 310: skis (class_id=30) at (0.0, 11.5) size=0.0065643312x0.0006269932 conf=33.21
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 311: mouse (class_id=64) at (0.0, 16.5) size=0.00501709x0.00088384154 conf=32.91
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 312: mouse (class_id=64) at (0.0, 15.9) size=0.0058410647x0.0011844635 conf=32.8
8
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 313: mouse (class_id=64) at (0.0, 15.2) size=0.0044151307x0.00044481756 conf=32.
82
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 314: mouse (class_id=64) at (0.0, 14.4) size=0.002206421x0.0005890131 conf=32.39
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 315: mouse (class_id=64) at (0.0, 12.3) size=0.003881836x0.0013054848 conf=32.30
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 316: mouse (class_id=64) at (0.0, 12.4) size=0.004346466x0.00048086644 conf=30.7
0
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 317: mouse (class_id=64) at (0.0, 12.2) size=0.00602417x0.00068621634 conf=30.18
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 318: mouse (class_id=64) at (0.0, 13.3) size=0.0042778016x0.0004020095 conf=29.8
4
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 319: mouse (class_id=64) at (0.0, 16.3) size=0.0045364383x0.0005384803 conf=29.3
9
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 320: apple (class_id=47) at (0.0, 12.4) size=0.0046440125x0.0009121657 conf=27.4
2
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 321: mouse (class_id=64) at (0.0, 14.7) size=0.006985474x0.0016183377 conf=27.19
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 322: mouse (class_id=64) at (0.0, 13.9) size=0.0024993897x0.0008304119 conf=26.9
5
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 323: apple (class_id=47) at (0.0, 11.5) size=0.0072875977x0.00068364147 conf=25.
66
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 324: mouse (class_id=64) at (0.0, 14.5) size=0.00559845x0.0006520987 conf=25.37 
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.633333333
0:00:02.249708100 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 21
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.666666666
0:00:02.259501100 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 22
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.700000000
0:00:02.269678100 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 23
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.733333333
0:00:02.279772900 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 24
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.766666666
0:00:02.289503800 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 25
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.800000000
0:00:02.299862600 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 26
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.833333333
0:00:02.322290100 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 27
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.866666666
0:00:02.336511600 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 28
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.900000000
0:00:02.347475100 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 29
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.933333333
0:00:02.356921800 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 30
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:00.966666666
0:00:02.366248700 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 31
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:01.000000000
0:00:02.376206600 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 32
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:01.033333333
0:00:02.385693300 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 33
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:01.066666666
0:00:02.396064000 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 34
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:01.100000000
0:00:02.406155700 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 35
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:01.133333333
0:00:02.415793300 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 36
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:01.166666666
0:00:02.425195700 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 37
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:01.200000000
0:00:02.435779900 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 38
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:01.233333333
0:00:02.446248700 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 39
[2025-08-25T20:50:00Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 324 detections at timestamp 0:00:01.266666666
0:00:02.456661000 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 40
[2025-08-25T20:50:01Z INFO  ball_tracking_visualization] [1756155001.074] Frames: 1 | FPS: 0.5 | Objects rendered: 1 | Buffer: 1
[2025-08-25T20:50:01Z INFO  ort::memory] new_cpu; allocator=Arena memory_type=Default
[2025-08-25T20:50:01Z INFO  ort::memory] drop; self=MemoryInfo { ptr: 0x2cd46f14490, should_release: true }
0:00:03.232628400 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:427:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Frame 40: Detected 1 objects
[2025-08-25T20:50:01Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp] 🎆 Frame 40: Emitting 1 detections via signal
[2025-08-25T20:50:01Z INFO  ds_rs::backend::cpu_vision::cpudetector::imp]   ➡️ Detection 1: person (class_id=0) at (513.9, 389.0) size=57.037502x42.75703 conf=0.83      
[2025-08-25T20:50:01Z INFO  ds_rs::backend::cpu_vision::elements] 🎯 Drawing 1 detections at timestamp 0:00:01.300000000
0:00:03.241388100 14284 000002CD46E71C60 DEBUG            cpudetector crates\ds-rs\src\backend\cpu_vision\cpudetector\imp.rs:401:<ds_rs::backend::cpu_vision::cpudetector::imp::CpuDetector as gstreamer_base::subclass::base_transform::BaseTransformImpl>::transform_ip:<detector> Processing frame 41
```

