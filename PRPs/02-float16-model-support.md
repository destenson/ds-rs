# PRP-02: Float16 Model Support for ONNX Runtime

## Problem Statement
The YOLO models are using 16-bit floating point (float16/half precision) format, but the current implementation only supports 32-bit floats (float32). This causes a type mismatch error: "Float16 models not currently supported due to lifetime issues". The application fails to run inference on float16 models even though the ORT crate has float16 support via the `half` feature.

## Root Cause Analysis
1. **Type Mismatch**: Model expects float16 input tensors but code provides float32
2. **Lifetime Issues**: The current implementation has Rust borrow checker issues when creating ORT Values from float16 arrays
3. **Incomplete Implementation**: Float16 support is disabled with a hardcoded error message rather than properly implemented
4. **Output Handling**: Both input and output tensors need float16 support for complete inference

## Context and References

### Documentation URLs
- Half crate documentation: https://docs.rs/half/latest/half/
- ORT Rust documentation: https://docs.rs/ort/latest/ort/
- ORT Values documentation: https://ort.pyke.io/fundamentals/value
- ONNX Runtime float16 guide: https://onnxruntime.ai/docs/performance/model-optimizations/float16.html
- Half crate f16 type: https://docs.rs/half/latest/half/struct.f16.html

### Key Information from Research
- The `half` crate provides `f16` and `bf16` types for half-precision floating point
- ORT crate supports float16 through its `half` feature (already enabled in Cargo.toml)
- The `f16` type is repr(transparent) to u16, making memory layout compatible with ONNX Runtime
- Conversion between f32 and f16 is lossless from f16→f32 but lossy from f32→f16
- The ORT Value::from_array requires proper lifetime management for borrowed data

### Existing Patterns in Codebase
- Error handling uses `DetectorError` enum (crates/ds-rs/src/backend/cpu_vision/detector.rs)
- Model type detection already exists: checking `session.inputs[0].input_type`
- Preprocessing returns Vec<f32> which needs conversion for f16 models
- The `half` feature is already configured in Cargo.toml with proper dependencies

### Local ORT Source Reference
- ORT source at: C:\Users\deste\repos\pykeio--ort
- Float16 implementation: src/tensor.rs lines 147-148, 154-155
- Value creation: src/value.rs handles Float16/Bfloat16 specially (repr(transparent) to u16)

## Implementation Blueprint

### Phase 1: Fix Input Tensor Creation for Float16
1. Detect model input type (already implemented)
2. When float16 is detected:
   - Convert preprocessed f32 data to f16 using half crate
   - Store f16 data in a local variable to extend lifetime
   - Create ndarray with f16 type
   - Pass to Value::from_array with proper lifetime bounds

3. Lifetime solution approach:
   - Store converted f16 data in a Box or Vec to ensure it lives long enough
   - Use CowArray to manage ownership properly
   - Ensure data outlives the Value creation process

### Phase 2: Handle Float16 Output Tensors
1. Detect output tensor type from model
2. Extract float16 outputs using appropriate ORT methods
3. Convert f16 outputs back to f32 for postprocessing
4. Ensure NMS and detection logic works with converted values

### Phase 3: Optimize Performance
1. Minimize conversions between f32 and f16
2. Consider keeping intermediate results in f16 where possible
3. Benchmark performance difference between f16 and f32 models
4. Add configuration option to force f32 conversion if needed

## Task List (in order)
1. Remove the hardcoded float16 error message
2. Implement f32 to f16 conversion for input preprocessing
3. Fix lifetime issues by properly managing f16 data ownership
4. Create float16 input tensors using Value::from_array
5. Handle float16 output tensor extraction
6. Convert f16 outputs to f32 for postprocessing
7. Test with actual float16 YOLO models
8. Add unit tests for float16 tensor operations
9. Document float16 support in README
10. Benchmark performance vs float32 models

## Implementation Details

### Input Tensor Creation Pattern
The implementation should follow this pattern:
1. Preprocess image to get Vec<f32> (existing code)
2. If model expects f16:
   - Convert Vec<f32> to Vec<half::f16> using iterator and Half::from_f32()
   - Store in a variable that outlives Value creation
   - Create ndarray::Array from f16 vec with proper shape
   - Use Value::from_array with the f16 array
3. If model expects f32:
   - Use existing f32 path (already working)

### Output Tensor Extraction Pattern
1. Run inference (existing code)
2. Check output tensor type
3. If output is f16:
   - Extract as OrtOwnedTensor<half::f16, _>
   - Convert to Vec<f32> for compatibility with postprocessing
4. If output is f32:
   - Use existing extraction (already working)

### Error Handling
- Add new DetectorError variants if needed for float16-specific errors
- Provide clear error messages about f16 conversion issues
- Fall back to f32 if f16 fails (with warning)

## Validation Gates

```bash
# Clean build to ensure all changes compile
cargo clean

# Build with all required features
cargo build --features cpu_vision,nalgebra,half,ort

# Run the example that was failing
cargo run --example cpu_detection_demo --features cpu_vision,nalgebra,half

# Run tests to ensure no regression
cargo test --features cpu_vision,nalgebra,half,ort

# Verify with actual YOLO model
# Should successfully run inference without float16 errors
```

## Success Criteria
1. Float16 YOLO models load and run without errors
2. Detection results are accurate (similar to float32 models)
3. No lifetime or borrow checker errors
4. Performance is acceptable (at least 10 FPS on CPU)
5. Both float16 and float32 models work seamlessly
6. Clear error messages if float16 operations fail

## Risk Mitigation
- Keep float32 path as fallback option
- Add runtime flag to force float32 conversion if needed
- Thoroughly test with different YOLO model versions
- Document any precision loss in detection accuracy
- Ensure backward compatibility with existing float32 models

## Performance Considerations
- Float16 uses half the memory bandwidth (beneficial for large models)
- CPU inference might be slower due to conversion overhead
- Consider using SIMD instructions for batch conversions if available
- Profile to identify conversion bottlenecks

## Alternative Approaches
1. **Force Float32**: Convert model to float32 at load time (higher memory use)
2. **Dynamic Precision**: Support multiple precision levels dynamically
3. **Quantization**: Use INT8 quantization instead of float16 (needs different approach)

## Dependencies
- `half` crate (already in Cargo.toml)
- `ort` crate with `half` feature (already configured)
- No new dependencies required

## Testing Requirements
- Unit tests for f32↔f16 conversion
- Integration tests with actual float16 models
- Performance benchmarks comparing f16 vs f32
- Edge cases: very small/large values, NaN, Inf

## Documentation Updates
- Update README to mention float16 support
- Add section on precision considerations
- Document performance characteristics
- Include troubleshooting for float16 issues

## Confidence Score: 7/10
Good confidence based on:
- Clear understanding of the problem
- ORT crate already supports float16
- Lifetime issues have known solutions
- All dependencies are available

Points deducted for:
- No existing float16 examples in ORT crate
- Lifetime management can be tricky
- Potential for unexpected edge cases