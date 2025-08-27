# PRP-56: App Module Unit Tests

## Executive Summary
Add comprehensive unit tests for the app module (crates/ds-rs/src/app/) which currently has zero test coverage. This module handles application configuration, runtime management, and timer coordination.

## Problem Statement
The app module has 4 files with 0% test coverage:
- app/config.rs - Configuration parsing and validation
- app/runner.rs - Application lifecycle management
- app/timers.rs - Timer management for periodic tasks
- app/mod.rs - Module exports and organization

## Implementation Blueprint

### File 1: app/config.rs Testing
Test configuration parsing and validation:

1. **Valid Configuration Tests**
   - Test default configuration creation
   - Test configuration from CLI args
   - Test configuration from environment variables
   - Test configuration merging/precedence

2. **Invalid Configuration Tests**
   - Test invalid URIs
   - Test invalid backend specifications
   - Test missing required fields
   - Test type mismatches

3. **Edge Cases**
   - Empty configuration
   - Partial configuration
   - Unicode in configuration values

Reference: Look at existing config usage in main.rs and runner.rs

### File 2: app/runner.rs Testing
Test application lifecycle with mocked dependencies:

1. **Initialization Tests**
   - Test runner creation with valid config
   - Test runner creation with invalid config
   - Test GStreamer initialization
   - Test backend manager setup

2. **Runtime Tests**
   - Test run() method with mock pipeline
   - Test graceful shutdown
   - Test error handling during runtime
   - Test signal handling (SIGINT/SIGTERM)

3. **State Management Tests**
   - Test state transitions
   - Test concurrent operations
   - Test resource cleanup

Use MockBackend from PRP-55 infrastructure

### File 3: app/timers.rs Testing
Test timer management and callbacks:

1. **Timer Creation Tests**
   - Test single timer creation
   - Test multiple timer creation
   - Test timer with different intervals
   - Test timer cancellation

2. **Callback Execution Tests**
   - Test callback invocation
   - Test callback timing accuracy
   - Test callback error handling
   - Test callback with state mutation

3. **Concurrency Tests**
   - Test multiple timers running concurrently
   - Test timer cleanup on drop
   - Test timer pause/resume if supported

### File 4: app/mod.rs Testing
Minimal testing needed - mostly re-exports:
- Test module visibility
- Test public API surface
- Ensure all exports are tested via other tests

## Test Organization

Each file gets a test module at the bottom:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::*;
    use rstest::*;

    // Tests here
}
```

## Files to Reference
- crates/ds-rs/src/app/config.rs - Understand config structure
- crates/ds-rs/src/app/runner.rs - Understand runtime flow
- crates/ds-rs/src/app/timers.rs - Understand timer patterns
- crates/ds-rs/src/main.rs - See how app module is used
- crates/ds-rs/tests/main_app_test.rs - Existing app tests

## Validation Gates
```bash
# Run app module unit tests
cargo test --lib app::

# Check coverage improvement
cargo tarpaulin --out Html -j 2
# Should show app/ module with >80% coverage

# Ensure no regressions
cargo test --all
```

## Tasks
1. Analyze app/config.rs structure and dependencies
2. Write unit tests for configuration parsing
3. Write unit tests for invalid configurations
4. Analyze app/runner.rs and identify mockable dependencies
5. Write unit tests for runner initialization
6. Write unit tests for runner lifecycle
7. Analyze app/timers.rs timer implementation
8. Write unit tests for timer creation and management
9. Write unit tests for timer callbacks
10. Verify 80%+ coverage for app module

## Success Criteria
- All 4 app module files have test coverage
- Each file has >80% line coverage
- Tests use mocks for external dependencies
- Tests are fast (<100ms each)
- Tests are deterministic (no flaky tests)

## Testing Patterns to Use
- Use rstest for parameterized config tests
- Use mockall for mocking BackendManager
- Use serial_test for tests needing exclusive access
- Use tokio::test for async timer tests
- Mock ctrlc signal handling

## Notes for Implementation
- Focus on public API testing first
- Mock external dependencies (GStreamer, filesystem)
- Test both success and error paths
- Use descriptive test names (test_config_from_valid_cli_args)
- Group related tests using nested modules

**Confidence Score: 8/10**
Clear module boundaries but may need careful mocking of runner dependencies.