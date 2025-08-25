# PRP-42: Production Hardening - Systematic unwrap() Replacement

## Summary
Replace 301 unwrap() calls across 43 files in ds-rs crate to prevent runtime panics in production. This PRP focuses on the most critical paths first to achieve production stability.

## Background
The codebase review identified 301 unwrap() calls that could cause runtime panics. Any single unwrap() failure will crash the entire application, making the system unsuitable for 24/7 production operation.

## Requirements
1. Replace all unwrap() calls in critical paths with proper error handling
2. Maintain backward compatibility with existing APIs
3. Preserve all existing functionality
4. Add context to errors for better debugging
5. Ensure no performance regression

## Implementation Plan

### Phase 1: Critical Modules (High Priority)
Target modules with highest unwrap() concentration:
- `multistream/` - 22 occurrences
- `source/` - 17 occurrences  
- `backend/cpu_vision/` - 28 occurrences
- `pipeline/` - 10 occurrences

### Phase 2: Core Infrastructure
- `elements/` - 6 occurrences
- `rendering/` - 17 occurrences
- `metadata/` - 3 occurrences

### Phase 3: Support Modules
- `app/` - 7 occurrences
- `config/` - 8 occurrences
- `inference/` - 2 occurrences
- `messages/` - 2 occurrences
- `tracking/` - 1 occurrence
- `platform.rs` - 2 occurrences

## Technical Approach

### Error Handling Patterns

1. **For Option types**: Replace `.unwrap()` with:
   ```rust
   // Before
   let value = some_option.unwrap();
   
   // After - with context
   let value = some_option.ok_or_else(|| {
       Error::MissingValue("Expected value not found".into())
   })?;
   
   // After - with default
   let value = some_option.unwrap_or_default();
   
   // After - with custom default
   let value = some_option.unwrap_or_else(|| default_value());
   ```

2. **For Result types**: Replace `.unwrap()` with:
   ```rust
   // Before
   let value = some_result.unwrap();
   
   // After - propagate error
   let value = some_result?;
   
   // After - with context
   let value = some_result.map_err(|e| {
       Error::ProcessingFailed(format!("Failed to process: {}", e))
   })?;
   
   // After - with anyhow context
   let value = some_result.context("Failed to process value")?;
   ```

3. **For Mutex/RwLock**: Replace `.unwrap()` with:
   ```rust
   // Before
   let guard = mutex.lock().unwrap();
   
   // After - with error handling
   let guard = mutex.lock().map_err(|e| {
       Error::LockPoisoned(format!("Mutex poisoned: {}", e))
   })?;
   
   // After - for non-critical paths
   let guard = match mutex.lock() {
       Ok(g) => g,
       Err(poisoned) => {
           log::warn!("Mutex poisoned, recovering: {}", poisoned);
           poisoned.into_inner()
       }
   };
   ```

4. **For tests**: Keep `.unwrap()` or use `.expect()`:
   ```rust
   // In tests, unwrap() is acceptable
   #[test]
   fn test_something() {
       let value = create_test_value().unwrap();
       // or with message
       let value = create_test_value().expect("Test value creation failed");
   }
   ```

### Module-Specific Strategies

#### source/ module
- Convert SourceController methods to return Result
- Add SourceError enum with specific error variants
- Use error context for source URIs and IDs

#### multistream/ module  
- Add MultiStreamError enum
- Handle pipeline pool errors gracefully
- Implement fallback for resource allocation failures

#### backend/cpu_vision/ module
- Add DetectorError, TrackerError enums
- Handle ONNX model loading failures
- Gracefully handle missing metadata

## Success Criteria
1. Zero unwrap() calls in non-test code (except where absolutely safe)
2. All existing tests still pass
3. New error paths have test coverage
4. Error messages provide sufficient context for debugging
5. No performance regression in benchmarks

## Testing Strategy
1. Run existing test suite after each module conversion
2. Add error case tests for new Result returns
3. Test error propagation through call chains
4. Verify panic-free operation under error conditions

## Rollback Plan
Changes are incremental and can be reverted module by module if issues arise.

## Files to Modify (Priority Order)

### Critical Path Files (Phase 1)
1. `crates/ds-rs/src/source/circuit_breaker.rs` - 21 unwraps
2. `crates/ds-rs/src/backend/cpu_vision/elements.rs` - 28 unwraps
3. `crates/ds-rs/src/multistream/pipeline_pool.rs` - 22 unwraps
4. `crates/ds-rs/src/source/isolation.rs` - 17 unwraps
5. `crates/ds-rs/src/source/health.rs` - 18 unwraps

### Core Infrastructure (Phase 2)
6. `crates/ds-rs/src/rendering/standard_renderer.rs` - 7 unwraps
7. `crates/ds-rs/src/pipeline/builder.rs` - 1 unwrap
8. `crates/ds-rs/src/pipeline/mod.rs` - 6 unwraps
9. `crates/ds-rs/src/backend/cpu_vision/cpudetector/imp.rs` - 16 unwraps

### Support Modules (Phase 3)
10. `crates/ds-rs/src/app/mod.rs` - 4 unwraps
11. `crates/ds-rs/src/config/mod.rs` - 8 unwraps
12. Remaining files with 1-3 unwraps each

## Validation Commands
```bash
# Count remaining unwraps
grep -r "\.unwrap()" crates/ds-rs/src --include="*.rs" | grep -v "test" | wc -l

# Run test suite
cargo test --package ds-rs

# Check for compilation warnings
cargo build --package ds-rs 2>&1 | grep -i warn

# Run clippy for additional checks
cargo clippy --package ds-rs -- -D warnings
```

## Documentation Updates
- Update CHANGELOG.md with production hardening improvements
- Add error handling guidelines to CONTRIBUTING.md
- Document new error types in module documentation

## Completion Checklist
- [ ] Phase 1: Critical modules (source/, multistream/, backend/)
- [ ] Phase 2: Core infrastructure (pipeline/, rendering/, elements/)
- [ ] Phase 3: Support modules (app/, config/, etc.)
- [ ] All tests passing
- [ ] Error documentation complete
- [ ] Performance benchmarks show no regression
- [ ] README.md updated with production-ready status