# PRP-25: Fix Ctrl+C Shutdown and Video Window Race Condition

## Problem Statement

The application has a race condition between Ctrl+C signal handling and video window creation. When the video window pops up, Ctrl+C stops working properly. When fixing the shutdown handler, it breaks video playback. This creates a frustrating cycle where fixing one bug reintroduces the other.

### Current Issues
1. **Shutdown Hang**: App prints "Received interrupt signal, shutting down..." but doesn't exit
2. **Window Event Conflict**: When video window appears, signal handling gets disrupted
3. **Inconsistent Behavior**: Sometimes works, sometimes doesn't, depending on timing

### Root Cause Analysis
The application mixes different event handling systems:
- **Tokio async runtime** for the main application loop
- **ctrlc crate** for signal handling  
- **GStreamer's GLib event system** for pipeline events
- **Native window system events** from autovideosink

These systems are not properly coordinated, causing race conditions and event handling conflicts.

## Required Research

### 1. Study GStreamer Event Loop Patterns
**Reference Locations**:
- `../gstreamer-rs/examples/src/bin/launch_glib_main.rs` - Shows proper GLib main loop usage
- `../gstreamer-rs/examples/src/bin/playbin.rs` - Complex playback with proper shutdown
- `../gstreamer-rs/examples/src/bin/appsink.rs` - Custom event handling patterns

**Key Patterns to Extract**:
- How GLib::MainLoop integrates with GStreamer
- How bus watches handle messages properly
- How to cleanly shutdown when using GLib main loop

### 2. Analyze C Reference Implementation
**Reference Location**: `../NVIDIA-AI-IOT--deepstream_reference_apps/runtime_source_add_delete/`

**Key Files**:
- `deepstream_test_rt_src_add_del.c` - Lines 539, 701, 709 show g_main_loop usage

**Patterns to Understand**:
- How g_main_loop_new/run/quit work
- How the C code handles shutdown without signal handlers
- How bus messages trigger clean shutdown

### 3. Research Window System Integration
**Search Terms for Documentation**:
- "gstreamer autovideosink window events"
- "gstreamer signal handling windows"
- "glib main loop signal handler integration"

**Key Questions**:
- How does autovideosink create windows on Windows/Linux?
- What event loops does it expect to be running?
- How to properly coordinate window events with shutdown?

## Implementation Strategy

### Phase 1: Replace Event Loop Architecture
**Task**: Switch from tokio+ctrlc to manually iterated GLib main context

**Locations to Modify**:
- `src/main.rs` - Remove tokio runtime, use GLib main context iteration
- `src/app/mod.rs` - Keep running flag but integrate with main context
- `src/app/runner.rs` - Rewrite to manually iterate main context

**Implementation Pattern**:
```rust
// Create main loop and context
let main_loop = glib::MainLoop::new(None, false);
let main_context = glib::MainContext::default();

// Keep reference to quit the loop properly
let main_loop_quit = main_loop.clone();

while running.load(Ordering::SeqCst) {
    // Process events efficiently
    main_context.iteration(true);  // true = may block until event
    
    // Check for shutdown
    if !running.load(Ordering::SeqCst) {
        // CRITICAL: Proper shutdown sequence
        // 1. Quit the main loop
        main_loop_quit.quit();
        
        // 2. Process remaining events to handle the quit
        for _ in 0..10 {  // Limited iterations to avoid infinite loop
            if !main_context.pending() {
                break;
            }
            main_context.iteration(false);  // Non-blocking to drain events
        }
        
        // 3. Now safe to break
        break;
    }
}

// Alternative: Use run() briefly to process quit
if main_loop.is_running() {
    main_loop.quit();
    // Run briefly to process the quit event
    glib::timeout_add(Duration::from_millis(0), {
        let ml = main_loop.clone();
        move || {
            ml.quit();
            glib::ControlFlow::Break
        }
    });
    main_loop.run();  // Will return immediately due to timeout
}
```

**Proper Cleanup Pattern**:
```rust
// In signal handler
ctrlc::set_handler(move || {
    println!("Received Ctrl+C, initiating shutdown...");
    running.store(false, Ordering::SeqCst);
    
    // Wake up the context
    glib::MainContext::default().wakeup();
    
    // Quit the main loop
    if let Some(loop_ref) = main_loop_weak.upgrade() {
        loop_ref.quit();
    }
})?;
```

**Key Advantages**:
- Maintains control over the event loop
- Can check shutdown flag between iterations
- Allows integration with other event sources
- Prevents blocking on window events

**Validation Pattern**: 
- Search for MainContext::iteration patterns in gstreamer-rs examples
- Look for non-blocking event processing patterns

### Phase 2: Implement Proper Signal Handling
**Task**: Keep ctrlc but ensure it properly coordinates with main context

**Approach**:
```rust
// Use atomic flag that both signal handler and main loop check
let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

ctrlc::set_handler(move || {
    println!("Received Ctrl+C, initiating shutdown...");
    r.store(false, Ordering::SeqCst);
    
    // Wake up the main context if it's sleeping
    glib::MainContext::default().wakeup();
})?;

// In main loop:
while running.load(Ordering::SeqCst) {
    main_context.iteration(false);
    // ...
}
```

**Key Points**:
- Use AtomicBool instead of Mutex<bool> for lock-free access
- Call main_context.wakeup() to interrupt any blocking operations
- Check flag frequently in the iteration loop

### Phase 3: Coordinate Bus Message Handling
**Task**: Process bus messages during main context iteration

**Approach**:
```rust
// Add bus watch that integrates with main context
let bus = pipeline.bus().unwrap();
let running_clone = running.clone();

bus.add_watch(move |_, msg| {
    match msg.view() {
        MessageView::Eos(..) => {
            println!("End of stream");
            running_clone.store(false, Ordering::SeqCst);
            glib::ControlFlow::Break
        }
        MessageView::Error(err) => {
            eprintln!("Pipeline error: {}", err.error());
            running_clone.store(false, Ordering::SeqCst);
            glib::ControlFlow::Break
        }
        _ => glib::ControlFlow::Continue
    }
})?;

// The bus watch callbacks will be processed during main_context.iteration()
```

**Key Points**:
- Bus watch callbacks are automatically handled by MainContext
- Set running flag to false on EOS or ERROR
- Return ControlFlow::Break to remove the watch

### Phase 4: Handle Window Creation Timing
**Task**: Ensure window creation doesn't interfere with event handling

**Approach**:
- Set pipeline to PAUSED state before window creation
- Add sources after pipeline is ready
- Use sync=false on video sink to prevent blocking
- Ensure main loop quit is called from all exit paths

**Critical Cleanup**:
```rust
// On any exit path (EOS, Error, Ctrl+C):
fn cleanup_and_exit(pipeline: &Pipeline, main_loop: &MainLoop, running: &AtomicBool) {
    // 1. Set shutdown flag
    running.store(false, Ordering::SeqCst);
    
    // 2. Stop pipeline FIRST (prevents new events)
    pipeline.set_state(gst::State::Null).ok();
    
    // 3. Quit the main loop
    main_loop.quit();
    
    // 4. Wake up context to process the quit
    glib::MainContext::default().wakeup();
    
    // 5. Drain remaining events
    let ctx = glib::MainContext::default();
    while ctx.pending() {
        ctx.iteration(false);
    }
}
```

**Most Robust Pattern**:
```rust
// The quit-run-break pattern for guaranteed cleanup
if !running.load(Ordering::SeqCst) {
    // Tell main loop to quit
    main_loop.quit();
    
    // Run it briefly to process the quit
    if main_loop.is_running() {
        // Add immediate timeout to prevent blocking
        glib::idle_add_once(move || {
            main_loop_clone.quit();
        });
        main_loop.run();  // Returns when quit is processed
    }
    
    // Now safe to break
    break;
}
```

## Testing Requirements

### Unit Tests
1. **Test Graceful Shutdown**
   - Start application with video
   - Send SIGINT after window appears
   - Verify clean exit within 2 seconds

2. **Test Multiple Signals**
   - Send multiple Ctrl+C signals
   - Verify only first one triggers shutdown
   - Ensure no duplicate messages

3. **Test Window Creation Timing**
   - Start with different video sources
   - Verify window appears without blocking
   - Confirm shutdown works at any point

### Integration Tests
Use the test created in `tests/shutdown_test.rs` but enhance it to:
1. Wait for window creation before sending signal
2. Verify no zombie processes remain
3. Test with real video files on Windows

### Platform-Specific Tests
- **Windows**: Test with d3dvideosink, autovideosink
- **Linux**: Test with glimagesink, xvimagesink, autovideosink

## Success Criteria

1. **Clean Shutdown**: Ctrl+C always stops the application within 2 seconds
2. **No Race Conditions**: Window creation doesn't affect shutdown behavior
3. **Consistent Behavior**: Works reliably 100% of the time, not intermittently
4. **No Regression**: Video playback continues to work properly
5. **Cross-Platform**: Works on both Windows and Linux

## Validation Gates

```bash
# Build and format check
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings

# Run shutdown tests
cargo test --test shutdown_test -- --nocapture

# Run with real video and test Ctrl+C manually
cargo run --bin ds-app -- file://C:/Users/deste/Videos/wows-sm.1.mp4
# Press Ctrl+C after window appears - should exit within 2 seconds

# Run all tests to ensure no regression
cargo test --all
```

## Implementation Order

1. **Research Phase** (30 minutes)
   - Study gstreamer-rs examples for GLib main loop patterns
   - Review C reference implementation
   - Document findings in comments

2. **Refactor Event Loop** (2 hours)
   - Replace tokio runtime with GLib main loop
   - Update Application struct to use main loop
   - Test basic functionality

3. **Add Signal Handling** (1 hour)
   - Implement platform-specific signal handlers
   - Integrate with main loop quit
   - Test Ctrl+C behavior

4. **Fix Window Timing** (1 hour)
   - Adjust pipeline state management
   - Set proper sink properties
   - Test with various video sources

5. **Comprehensive Testing** (1 hour)
   - Run all unit tests
   - Manual testing on Windows
   - Update shutdown_test.rs if needed

## Common Pitfalls to Avoid

1. **Don't Mix Event Systems**: Choose one event loop and stick with it
2. **Don't Block Main Thread**: Window creation should be non-blocking
3. **Handle Platform Differences**: Windows and Unix signal handling differs
4. **Clean Up Resources**: Ensure pipeline reaches NULL state before exit
5. **Test Window Edge Cases**: Test when window is minimized, maximized, closed

## Documentation Requirements

Update the following after implementation:
- `BUGS.md` - Mark shutdown bug as FIXED with explanation
- `LESSONS_LEARNED.md` - Add lessons about GStreamer event loops
- `CLAUDE.md` - Document the new event loop architecture

## References

- GStreamer Application Development Manual - Chapter on GLib main loop
- gstreamer-rs examples repository - Multiple working examples
- NVIDIA DeepStream reference apps - C implementation patterns
- GLib documentation on main loops and signal handling

---

**Confidence Score**: 8/10

The plan is comprehensive and addresses the root cause (mixed event systems). The references to actual working code in gstreamer-rs examples and the C reference provide concrete patterns to follow. The main risk is platform-specific signal handling complexity on Windows.