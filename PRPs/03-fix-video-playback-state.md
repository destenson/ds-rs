# PRP-03: Fix Video Playback State Management

## Problem Statement
The video pipeline fails to reach PLAYING state and display video. State transitions are failing, with elements regressing from Paused → Ready → Null instead of progressing to Playing. No video window appears despite the pipeline appearing to run.

## Root Cause Analysis
Based on the debug output and codebase analysis:

1. **Asynchronous state change handling** - The pipeline sets state to PLAYING but doesn't wait for async completion
2. **Source synchronization issue** - Sources are added after pipeline starts transitioning to PAUSED
3. **State regression** - Elements fail to negotiate and fall back to NULL state
4. **Missing state synchronization** - Dynamic elements aren't properly synchronized with parent pipeline state

## Implementation Requirements

### Context and References
- Study how gstreamer-rs examples handle dynamic pipeline construction (../gstreamer-rs/examples/src/bin/decodebin.rs)
- Review how autovideosink creates windows on Windows (check ../gstreamer-rs examples for d3d11videosink usage)
- Understand GStreamer state change async handling patterns
- Reference the C implementation in ../NVIDIA-AI-IOT--deepstream_reference_apps/runtime_source_add_delete/

### Critical Files to Modify
1. `crates/ds-rs/src/app/mod.rs` - Main application state management
2. `crates/ds-rs/src/source/video_source.rs` - Source element state handling
3. `crates/ds-rs/src/source/addition.rs` - Source addition synchronization
4. `crates/ds-rs/src/pipeline/state.rs` - Pipeline state management (may need creation)

### Implementation Strategy

#### Phase 1: Fix Pipeline Initialization Order
- Move source addition BEFORE setting pipeline to PAUSED
- Ensure all static elements are added and linked before any state changes
- Set pipeline to PAUSED, add sources, then transition to PLAYING

#### Phase 2: Implement Proper Async State Handling
- After setting state to PLAYING, wait for async state change completion
- Use `pipeline.get_state()` with proper timeout to ensure state transition completes
- Handle ASYNC state changes by waiting for STATE_CHANGED messages on the bus

#### Phase 3: Source State Synchronization
- When adding sources dynamically, use `element.sync_state_with_parent()`
- Ensure sources inherit pipeline clock and base time
- Wait for source state changes to complete before proceeding

#### Phase 4: Add State Validation
- Implement state checking after each transition
- Add logging for all state changes with timestamps
- Validate that all elements reach target state before continuing

### Key Implementation Details

1. **Order of Operations**:
   - Create pipeline and add all static elements
   - Set pipeline to PAUSED
   - Add initial source(s) 
   - Wait for PAUSED state to be reached
   - Set pipeline to PLAYING
   - Wait for PLAYING state confirmation

2. **Dynamic Source Addition**:
   - When adding sources to running pipeline:
     - Add source to pipeline
     - Call sync_state_with_parent()
     - Wait for state change completion
     - Only then proceed with linking

3. **Window Creation on Windows**:
   - autovideosink needs the pipeline in PLAYING state to create window
   - Ensure message pump is running (GLib MainLoop)
   - May need to set window handle if embedding

### Testing Approach

1. Test basic pipeline without dynamic sources first
2. Test with single static source
3. Test with dynamic source addition
4. Verify window appears and video plays

### Validation Gates

```bash
# Build and check
cargo build --release

# Test state management
cargo test --test pipeline_tests -- --nocapture

# Run application with debug output
GST_DEBUG=3 cargo run --release --bin ds-app -- file://D:/files/large/wows-sm.1.mp4

# Success criteria:
# - No state regression messages
# - Pipeline reaches PLAYING state
# - Video window appears
# - Video content is visible
```

### Expected Outcomes

1. Pipeline successfully transitions NULL → READY → PAUSED → PLAYING
2. Video window appears when pipeline reaches PLAYING
3. Video content displays correctly
4. No state regression or element failures
5. Clean shutdown without errors

### Risk Mitigation

- Keep existing async handling for backwards compatibility
- Add state transition timeouts to prevent hangs  
- Implement rollback if state changes fail
- Add comprehensive logging for debugging

## Implementation Checklist

- [ ] Fix pipeline initialization order
- [ ] Implement proper async state waiting
- [ ] Fix source state synchronization
- [ ] Add state validation and logging
- [ ] Test with various video sources
- [ ] Verify window creation on Windows
- [ ] Update tests for new behavior
- [ ] Document state management patterns

## Notes

- The Standard backend (compositor) already handles framerate normalization
- State changes must be atomic - either all succeed or rollback
- Windows autovideosink may need special handling for window creation
- Consider adding state machine abstraction for cleaner code

## Confidence Score: 8/10

High confidence in diagnosis based on clear state regression in logs. Solution follows established GStreamer patterns from reference examples. Main uncertainty is Windows-specific autovideosink behavior.