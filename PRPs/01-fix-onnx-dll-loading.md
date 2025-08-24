# PRP-01: Fix ONNX Runtime DLL Loading Issues on Windows

## Problem Statement
The application fails to start with error code 0xc000007b (STATUS_INVALID_IMAGE_FORMAT) when running examples that use the ONNX Runtime (ort) feature on Windows. This error indicates a DLL architecture mismatch or missing dependencies. The build script warns about missing DLLs but doesn't properly handle the ort crate's download and placement of these DLLs.

## Root Cause Analysis
1. **Architecture Mismatch**: Error 0xc000007b typically indicates loading a 32-bit DLL into a 64-bit process or vice versa
2. **DLL Location Issues**: The ort crate downloads DLLs but they may not be in the expected location when build.rs runs
3. **Build Order Problem**: The build.rs for ds-rs runs before ort finishes downloading/extracting its DLLs
4. **Missing Error Detection**: The current build.rs only warns but doesn't fail or provide actionable guidance

## Context and References

### Documentation URLs
- ORT Rust crate documentation: https://docs.rs/ort/latest/ort/
- ORT setup and linking guide: https://ort.pyke.io/setup/linking
- Windows DLL error diagnosis: https://learn.microsoft.com/en-us/archive/blogs/dsvc/diagnosing-status_invalid_image_format-c000007b-errors
- ONNX Runtime releases: https://github.com/microsoft/onnxruntime/releases

### Key Information from Research
- The ort crate v1.16.3 automatically downloads ONNX Runtime binaries
- Windows requires both `onnxruntime.dll` and `onnxruntime_providers_shared.dll`
- DLLs must match the target architecture (x64 for 64-bit builds)
- The ort crate supports multiple strategies via ORT_STRATEGY environment variable
- The `copy-dylibs` feature can help with automatic DLL copying
- DLLs in the same folder as the executable resolve before system DLLs

### Existing Patterns in Codebase
- Error handling uses `thiserror` with `DeepStreamError` enum (crates/ds-rs/src/error.rs)
- Build scripts exist in both `crates/ds-rs/build.rs` and `crates/cpuinfer/build.rs`
- The project uses feature flags extensively for conditional compilation
- Mock detection fallback is already implemented when ONNX models can't load

## Implementation Blueprint

### Phase 1: Improve DLL Discovery and Validation
1. Update build.rs to search multiple locations for ONNX Runtime DLLs:
   - Check ort crate's download location (usually in target directory structure)
   - Check OUT_DIR from ort's build
   - Check system PATH
   - Check common installation directories

2. Add architecture validation:
   - Verify DLLs match the target architecture (x86 vs x64)
   - Use Windows API or file headers to detect DLL architecture
   - Fail the build with clear error if architecture mismatch detected

3. Implement robust DLL copying:
   - Copy to all necessary locations (deps/, examples/, project root)
   - Verify copies succeeded with file size/hash checks
   - Create symbolic links as fallback if copying fails

### Phase 2: Runtime DLL Loading Enhancement
1. Add runtime DLL validation in lib.rs init():
   - Check for DLL presence before attempting to use ort features
   - Provide specific error messages about missing/mismatched DLLs
   - Suggest remediation steps in error messages

2. Implement DLL loader helper:
   - Create a module for Windows-specific DLL management
   - Use LoadLibrary/GetProcAddress for explicit loading
   - Provide fallback to mock detector if DLLs unavailable

3. Add environment variable support:
   - Honor ORT_DYLIB_PATH for custom DLL locations
   - Support ORT_STRATEGY for different loading strategies
   - Document these in error messages

### Phase 3: User Experience Improvements
1. Enhanced error messages:
   - Detect specific failure modes (missing vs wrong architecture)
   - Provide actionable steps to fix each issue
   - Include download URLs for correct ONNX Runtime version

2. Build-time diagnostics:
   - Print detected DLL paths and architectures
   - Show ort crate configuration
   - Warn about potential conflicts with System32 DLLs

3. Documentation updates:
   - Add Windows setup section to README
   - Document troubleshooting steps
   - Provide scripts for manual DLL download/placement

## Task List (in order)
1. Enhance build.rs DLL discovery logic to find ort-downloaded DLLs
2. Add Windows-specific DLL architecture detection
3. Implement comprehensive DLL copying to all required locations
4. Add runtime DLL validation in library initialization
5. Create detailed error types for different DLL loading failures
6. Implement environment variable support for DLL paths
7. Add build-time diagnostic output for debugging
8. Create helper script for manual DLL setup
9. Update documentation with Windows-specific instructions
10. Add integration tests for DLL loading scenarios

## Validation Gates

```bash
# Clean build to ensure build script runs
cargo clean

# Build with ort feature
cargo build --features ort

# Verify DLLs are copied to correct locations
dir target\debug\*.dll
dir target\debug\deps\*.dll  
dir target\debug\examples\*.dll

# Run example that uses ONNX
cargo run --example cpu_detection_demo --features cpu_vision,nalgebra,half

# Run tests with ort feature
cargo test --features ort

# Check for proper error messages when DLLs are missing
# (temporarily rename DLLs to test error handling)
```

## Success Criteria
1. Examples run without 0xc000007b error
2. Clear error messages when DLLs are missing or wrong architecture
3. Build script successfully finds and copies ort-downloaded DLLs
4. Works on both debug and release builds
5. Works with different target architectures (x86, x64)
6. Graceful fallback to mock detector when DLLs unavailable

## Risk Mitigation
- Test on multiple Windows versions (10, 11)
- Test with both MSVC and GNU toolchains
- Ensure no regression for non-Windows platforms
- Maintain backward compatibility with existing code
- Don't break builds that don't use ort feature

## Confidence Score: 8/10
High confidence due to clear error pattern and well-understood root cause. Points deducted for potential complexity in detecting DLL architecture and handling various ort download scenarios.