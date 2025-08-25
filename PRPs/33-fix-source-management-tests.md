# PRP-33: Fix Source Management Test Failures

**Status: COMPLETED** (2025-08-24)

## Problem Statement

Three source_management tests are failing with the Mock backend, preventing the test suite from achieving 100% pass rate:

1. **test_concurrent_operations**: Pipeline element addition fails when multiple threads try to add sources simultaneously
   - Error: "Failed to add element to pipeline test-pipeline"
   - Root cause: All threads generate the same source name "source-0"

2. **test_maximum_sources_limit**: Capacity check assertion fails
   - Error: "assertion failed: !controller.has_capacity().unwrap()"
   - Root cause: has_capacity() returns true when it should return false after reaching max sources

3. **test_source_state_transitions**: State changes fail with Mock backend
   - Error: "Failed to set state for source source-0: StateChangeError"
   - Root cause: Mock backend doesn't support state transitions for uridecodebin elements

## Technical Analysis

### Issue 1: Concurrent Source Name Generation
The `generate_source_id()` method uses an AtomicUsize counter, but the source name generation doesn't use this ID atomically with the element creation, causing race conditions.

### Issue 2: Capacity Check Logic
The `with_max_sources()` constructor sets a limit, but `has_capacity()` doesn't properly check against this limit.

### Issue 3: Mock Backend State Transitions
The Mock backend creates real GStreamer elements (uridecodebin) which require proper state management that the Mock backend doesn't provide.

## Proposed Solution

### 1. Fix Concurrent Source Operations
- Make source ID generation and element naming atomic
- Use the generated ID immediately in the source element name
- Ensure thread-safe source addition with proper locking

### 2. Fix Capacity Check
- Implement proper capacity checking in `has_capacity()`
- Check active sources count against max_sources limit
- Return error on add_source when at capacity

### 3. Fix Mock Backend State Transitions
- Mock backend should handle basic state transitions
- Add minimal state management for test elements
- Or use simpler test elements that don't require complex state management

## Implementation Plan

### Phase 1: Analyze Current Implementation
1. Review SourceController implementation
2. Review SourceManager ID generation
3. Understand Mock backend limitations

### Phase 2: Fix Concurrent Operations
1. Make source naming atomic with ID generation
2. Use unique names for concurrent sources
3. Add proper synchronization

### Phase 3: Fix Capacity Logic
1. Implement max_sources tracking
2. Fix has_capacity() logic
3. Add capacity enforcement in add_source

### Phase 4: Fix State Transitions
1. Add basic state handling to Mock backend
2. Or modify tests to not require state transitions
3. Ensure tests work with Mock backend limitations

## Validation

```bash
# Run the specific failing tests
cargo test --test source_management test_concurrent_operations
cargo test --test source_management test_maximum_sources_limit  
cargo test --test source_management test_source_state_transitions

# Run all source management tests
cargo test --test source_management

# Run full test suite to ensure no regressions
cargo test
```

## Success Criteria

- [x] test_concurrent_operations passes consistently
- [x] test_maximum_sources_limit passes with correct capacity checking
- [x] test_source_state_transitions passes or is properly skipped for Mock backend
- [x] All 140 tests pass (100% pass rate)
- [x] No race conditions in concurrent source operations

## Files to Modify

- `crates/ds-rs/src/source/controller.rs` - Fix concurrent operations and capacity
- `crates/ds-rs/src/source/manager.rs` - Fix ID generation atomicity
- `crates/ds-rs/src/source/video_source.rs` - Fix element naming
- `crates/ds-rs/src/backend/mock.rs` - Add basic state handling
- `crates/ds-rs/tests/source_management.rs` - Adjust tests if needed

## Risk Assessment

**Low Risk**: These are test-specific issues that don't affect production code functionality. The fixes will improve test reliability without changing core behavior.

## Timeline

- Implementation: 2-3 hours
- Testing: 1 hour
- Total: 3-4 hours

## Implementation Summary

### Changes Made

1. **Fixed Concurrent Operations Race Condition**
   - Modified `generate_source_id()` to use write lock and mark source as enabled atomically
   - This prevents multiple threads from getting the same ID

2. **Fixed Capacity Checking**
   - Added `get_max_sources()` getter method to SourceManager
   - Modified `has_capacity()` to use instance's max_sources instead of global constant

3. **Replaced Mock Backend with Standard Backend in Tests**
   - Tests now use Standard backend with compositor element
   - More reliable than Mock backend for integration testing
   - Uses videotestsrc:// URIs instead of file URIs

### Results

- All 140 tests now pass (100% pass rate)
- No race conditions in concurrent operations
- Tests are more reliable and maintainable
- Per user request, moving away from Mock backend
