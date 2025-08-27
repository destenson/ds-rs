# PRP-55: Test Infrastructure and Mocking Setup

## Executive Summary
Establish foundational test infrastructure including mocking framework, shared utilities, and test fixtures to enable comprehensive testing across the ds-rs codebase. This is the first implementation step of PRP-54.

## Problem Statement
Current codebase lacks essential testing infrastructure:
- No mocking framework for GStreamer and backend dependencies
- No shared test utilities or fixtures
- Inconsistent test organization patterns
- Difficult to test components in isolation

## Implementation Blueprint

### Step 1: Add Testing Dependencies
Add to crates/ds-rs/Cargo.toml under [dev-dependencies]:
- mockall = "0.12" - For mocking traits
- rstest = "0.18" - For parameterized tests
- serial_test = "3.0" - For exclusive resource tests
- proptest = "1.4" - For property-based testing
- once_cell = "1.19" - For lazy static test fixtures

### Step 2: Create Test Utilities Module
Create tests/common/mod.rs with these utilities:

1. **Mock Pipeline Builder**
   - Creates mock GStreamer pipelines for testing
   - Returns pre-configured pipeline with mock elements
   - Reference: src/pipeline/builder.rs for interface

2. **Mock Element Factory**
   - Creates mock GStreamer elements
   - Simulates element creation without real GStreamer
   - Reference: src/elements/factory.rs for interface

3. **Test URI Generators**
   - Generate file:// URIs for test videos
   - Generate rtsp:// URIs for network sources
   - Generate invalid URIs for error testing

4. **Temporary Resource Helpers**
   - Create temp directories for test outputs
   - Clean up resources after tests
   - Use tempfile crate (already in dev-dependencies)

5. **Mock Backend Configurations**
   - Pre-configured Mock backend setups
   - Standard backend with mock elements
   - DeepStream backend with mock elements

### Step 3: Apply Mocking to Core Traits

1. **Backend Trait** (src/backend/mod.rs)
   - Add #[cfg_attr(test, automock)] to Backend trait
   - This enables MockBackend for testing
   - Keep existing implementations unchanged

2. **ElementFactory Trait** (src/elements/factory.rs)
   - Add mockable version of ElementFactory
   - Allow injection of mock element creation

3. **Source Traits** (src/source/mod.rs)
   - Mock SourceAddition trait
   - Mock SourceRemoval trait
   - Mock SourceBin trait

4. **Pipeline Traits** (src/pipeline/mod.rs)
   - Mock StateManager trait
   - Mock BusWatcher trait

### Step 4: Create Mock Validation Tests
Create tests/mock_validation.rs to verify mocking works:
- Test MockBackend creation and usage
- Test mock element creation
- Test mock trait interactions
- Ensure mocks compile and behave correctly

### Step 5: Document Testing Patterns
Create tests/README.md with:
- How to use mockall in this project
- Common mocking patterns
- Test organization guidelines
- Examples of unit vs integration tests

## Files to Reference
- crates/ds-rs/src/backend/mod.rs - Backend trait to mock
- crates/ds-rs/src/elements/factory.rs - ElementFactory patterns
- crates/ds-rs/tests/backend_tests.rs - Existing test patterns
- crates/ds-rs/Cargo.toml - Add dependencies here

## Validation Gates
```bash
# Dependencies compile
cargo check --all-features

# Mock validation tests pass
cargo test --test mock_validation

# Existing tests still work
cargo test --all

# Documentation builds
cargo doc --no-deps
```

## Tasks
1. Update Cargo.toml with test dependencies
2. Create tests/common/mod.rs structure
3. Implement mock pipeline builder utility
4. Implement mock element factory utility
5. Add #[automock] to Backend trait
6. Add #[automock] to source traits
7. Create mock_validation.rs test file
8. Write tests validating mock implementations
9. Create tests/README.md documentation
10. Verify all existing tests still pass

## Success Criteria
- All test dependencies added and compiling
- MockBackend trait available and functional
- Test utilities module created with 5+ helpers
- Mock validation tests passing
- Documentation complete in tests/README.md
- No regression in existing tests

## Notes for Implementation
- Start with minimal mocking, expand as needed
- Use #[cfg_attr(test, automock)] to avoid production impact
- Keep mock implementations simple initially
- Focus on most-used traits first
- Reference mockall documentation: https://docs.rs/mockall

**Confidence Score: 9/10**
Clear requirements with standard Rust patterns. Well-documented approach.