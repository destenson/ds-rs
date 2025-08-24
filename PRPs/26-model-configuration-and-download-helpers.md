# PRP-26: Model Configuration and Download Helpers

## Executive Summary

Enhance the ONNX model configuration system to provide comprehensive model path management through config files, command line options, and environment variables. When models are missing, provide helpful error messages with live download links to official YOLO model sources. This addresses the current limitation where users must manually locate and specify model files, improving developer experience and reducing setup friction.

## Problem Statement

### Current State
- Model paths are hardcoded or require manual specification in DetectorConfig
- Missing models result in fallback to mock detection with unclear error messages
- No standardized way to configure model locations across different environments
- Users must manually research and download YOLO models
- No guidance provided when models are not found

### Desired State
- Flexible model configuration through multiple methods (config file, CLI args, env vars)
- Clear error messages with direct download links when models are missing
- Standardized model directory structure and conventions
- Optional automatic model downloading capability
- Environment-specific model configuration (development vs production)

### Business Value
Reduces setup time for new developers and deployment scenarios by providing clear guidance and flexible configuration options for YOLO model management.

## Requirements

### Functional Requirements

1. **Multi-Source Configuration**: Support model path configuration via:
   - TOML configuration files
   - Command line arguments
   - Environment variables
   - Runtime DetectorConfig API

2. **Model Discovery**: Automatic detection of models in standard locations:
   - `./models/` directory
   - `~/.cache/ds-rs/models/` directory
   - Custom directories specified in config

3. **Download Guidance**: When models are missing, provide:
   - Direct download links to official YOLO models
   - Model size and capability information
   - Installation instructions

4. **Model Metadata**: Support for model-specific configuration:
   - Input dimensions
   - YOLO version detection
   - Class names/labels
   - Confidence thresholds

### Non-Functional Requirements

1. **Backwards Compatibility**: Existing DetectorConfig API continues to work unchanged
2. **Performance**: Configuration loading should not impact inference performance
3. **Security**: Model download links should be from trusted sources only
4. **Cross-Platform**: Configuration methods work across Windows, Linux, macOS

### Context and Research

The current implementation in `crates/ds-rs/src/backend/cpu_vision/detector.rs` shows:
- DetectorConfig already has `model_path: Option<String>` field
- Missing models gracefully fall back to mock detection
- Configuration patterns exist in `crates/ds-rs/src/config/mod.rs` using TOML + serde
- Command line argument patterns exist in `crates/ds-rs/src/main.rs` using clap
- Environment variable patterns exist (FORCE_BACKEND, RUST_LOG examples)

### Documentation & References

```yaml
- file: crates/ds-rs/src/backend/cpu_vision/detector.rs
  why: Current DetectorConfig structure and model loading logic

- file: crates/ds-rs/src/config/mod.rs  
  why: Existing TOML configuration patterns and ApplicationConfig structure

- file: crates/ds-rs/src/main.rs
  why: Command line argument patterns using clap

- file: crates/ds-rs/src/platform.rs
  why: Environment variable handling patterns (CUDA_VER, GPU_ID examples)

- url: https://github.com/ultralytics/yolov5/releases
  why: Official YOLOv5 ONNX model download links

- url: https://github.com/ultralytics/ultralytics
  why: Official YOLOv8+ models and documentation

- url: https://docs.ultralytics.com/models/
  why: Complete model documentation and export guides

- url: https://huggingface.co/Ultralytics/YOLOv8
  why: Alternative model hosting with metadata
```

### List of tasks to be completed to fulfill the PRP

```yaml
Task 1:
ENHANCE DetectorConfig structure:
  - ADD model_directory field for standard model locations
  - ADD model_name field for filename without path
  - ADD model_urls field containing official download sources
  - ADD model_metadata field for dimensions, version, classes
  - PRESERVE existing model_path field for backwards compatibility
  - UPDATE Default impl to use standard model discovery

Task 2:
ADD model configuration to ApplicationConfig:
  - EXTEND config/mod.rs with ModelConfig section
  - ADD TOML serialization support for model settings
  - ADD validation for model paths and download URLs
  - ADD model directory and cache directory settings
  - UPDATE default config to include model section

Task 3:
EXTEND command line interface:
  - ADD --model-path argument to main.rs
  - ADD --model-dir argument for model directory
  - ADD --download-missing flag for auto-download behavior
  - UPDATE clap Args struct with new model-related fields
  - ENSURE arguments override config file settings

Task 4:
ADD environment variable support:
  - ADD DS_MODEL_PATH for specific model file
  - ADD DS_MODEL_DIR for model directory
  - ADD DS_AUTO_DOWNLOAD for download behavior
  - UPDATE DetectorConfig::new_with_config to check env vars
  - FOLLOW existing patterns from platform.rs

Task 5:
IMPLEMENT model discovery logic:
  - ADD check_model_locations function to search standard paths
  - ADD model_exists validation with detailed error reporting
  - ADD resolve_model_path function combining all sources
  - IMPLEMENT precedence: CLI args > env vars > config file > defaults

Task 6:
CREATE download helper system:
  - ADD ModelRegistry struct with official YOLO model definitions
  - ADD download URLs for YOLOv5n, YOLOv8n, YOLOv8s, YOLOv8m variants
  - ADD model size and capability metadata
  - ADD helper functions to format download instructions

Task 7:
ENHANCE error messaging:
  - UPDATE model not found errors with specific download links
  - ADD suggestions for model selection based on use case
  - ADD example commands for model setup
  - INCLUDE model requirements (size, performance expectations)

Task 8:
ADD configuration validation:
  - ADD validate_model_config function
  - ADD checks for supported model formats (.onnx extension)
  - ADD validation for model metadata consistency  
  - ADD helpful error messages for common configuration mistakes

Task 9:
OPTIONAL: ADD automatic downloading:
  - ADD reqwest dependency for HTTP client
  - ADD download_model function with progress reporting
  - ADD verification of downloaded model integrity
  - ADD caching logic to prevent re-downloading

Task 10:
UPDATE documentation:
  - ADD model configuration examples to config files
  - ADD environment variable documentation
  - ADD CLI usage examples with model options
  - UPDATE README with model setup instructions

Task 11:
ADD comprehensive tests:
  - ADD tests for configuration precedence (CLI > env > config)
  - ADD tests for model discovery in different locations
  - ADD tests for error message formatting with download links
  - ADD integration tests with mock HTTP responses for download testing

Task 12:
UPDATE existing detector creation:
  - UPDATE StandardBackend to use enhanced model configuration
  - UPDATE examples to demonstrate model configuration options
  - ENSURE existing tests continue to pass with new config system
```

### Out of Scope
- Automatic model format conversion (PyTorch to ONNX)
- Model training or fine-tuning capabilities  
- Integration with model versioning systems
- Model performance benchmarking
- Custom model format support beyond ONNX

## Key Implementation Notes

### Configuration Precedence (Highest to Lowest)
1. Command line arguments (`--model-path`)
2. Environment variables (`DS_MODEL_PATH`)  
3. TOML configuration file
4. DetectorConfig API settings
5. Default model discovery paths

### Model Discovery Paths
```
1. Specified path (if absolute)
2. Current working directory + models/
3. User cache directory (~/.cache/ds-rs/models/)
4. System model directory (/usr/share/ds-rs/models/ on Linux)
```

### Download Link Format
When model is missing, error should include:
- Direct download URL
- Alternative sources (GitHub releases, Hugging Face)
- Model size and estimated performance
- Installation command example

### Integration Points
- Extends existing DetectorConfig without breaking changes
- Integrates with existing TOML config system in config/mod.rs
- Uses established clap patterns from main.rs
- Follows env var conventions from platform.rs

## Success Criteria

- [ ] Model paths configurable via config file, CLI args, and environment variables
- [ ] Missing models show helpful error messages with download links
- [ ] Model discovery works across standard locations automatically
- [ ] Configuration precedence works correctly (CLI > env > config > defaults)
- [ ] Backwards compatibility maintained for existing DetectorConfig usage
- [ ] All new configuration methods work cross-platform
- [ ] Integration tests validate complete configuration flow
- [ ] Documentation includes clear setup examples for all methods

## Dependencies

### Technical Dependencies
- clap 4.5+ (already present) for command line argument parsing
- serde + toml (already present) for configuration file support  
- tokio (already present) for async operations if auto-download implemented
- reqwest or ureq (new optional dependency) for HTTP downloading
- directories crate (new) for cross-platform cache directory locations

### Knowledge Dependencies
- Understanding of YOLO model variants and their use cases
- Cross-platform directory conventions for model storage
- HTTP client patterns for reliable file downloading
- Configuration precedence best practices

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|-------------|--------|-------------------|
| Breaking existing DetectorConfig API | Low | High | Maintain backwards compatibility, add extensive tests |
| Download links become outdated | Medium | Medium | Use official sources, add multiple fallback URLs |
| Cross-platform path handling issues | Medium | High | Use directories crate, test on all platforms |
| Configuration complexity confusion | Medium | Low | Provide clear documentation and examples |

## Architecture Decisions

### Decision: Configuration Precedence Order
**Options Considered:**
1. Config file takes precedence over CLI args
2. CLI args override config file settings
3. Environment variables have highest precedence

**Decision:** CLI args > Environment variables > Config file > Defaults

**Rationale:** Matches common Unix tool conventions where command line has highest precedence for immediate overrides

### Decision: Model Discovery Strategy  
**Options Considered:**
1. Only check explicitly configured paths
2. Search multiple standard locations automatically
3. Require explicit model registration

**Decision:** Automatic discovery with fallback to multiple standard paths

**Rationale:** Reduces configuration burden while maintaining flexibility for explicit configuration

### Decision: Download Helper Approach
**Options Considered:**
1. Automatic downloading when models missing
2. Error messages with manual download instructions  
3. Interactive prompts for download permission

**Decision:** Error messages with download links, optional auto-download flag

**Rationale:** Gives users control while providing clear guidance, avoids unexpected network activity

## Validation Strategy

### Validation Commands
```bash
# Build with all features
cargo build --all-features

# Run unit tests for configuration
cargo test --lib config::tests --all-features

# Run detector tests with new config options
cargo test --features ort,ndarray backend::cpu_vision::detector::tests

# Run integration tests
cargo test --features ort,ndarray,nalgebra --test cpu_backend_tests

# Test CLI argument parsing
cargo run --bin ds-app -- --help

# Test with different configuration methods
DS_MODEL_PATH=/path/to/model.onnx cargo test
cargo run --bin ds-app -- --model-path models/yolov5n.onnx <video_uri>
```

### Manual Testing Scenarios
```bash
# Test model discovery
mkdir models && cargo run --bin ds-app <video_uri>

# Test error messaging 
rm models/*.onnx && cargo run --bin ds-app <video_uri>

# Test configuration precedence
echo 'model_path = "config.onnx"' > test.toml
DS_MODEL_PATH=env.onnx cargo run --bin ds-app --model-path cli.onnx <video_uri>
```

## Future Considerations

- Integration with model registry services (MLflow, Weights & Biases)  
- Support for remote model URLs without local caching
- Model version management and compatibility tracking
- Performance profiling for different model variants
- Integration with model optimization tools (quantization, pruning)

## References

- Ultralytics YOLO Documentation: https://docs.ultralytics.com/models/
- YOLOv5 Official Repository: https://github.com/ultralytics/yolov5
- YOLOv8+ Repository: https://github.com/ultralytics/ultralytics
- ONNX Model Zoo: https://github.com/onnx/models
- Rust Configuration Best Practices: https://rust-cli-recommendations.sunshowers.io/

---

## PRP Metadata

- **Author**: Claude Code
- **Created**: 2025-01-25
- **Last Modified**: 2025-01-25
- **Status**: Ready for Implementation
- **Confidence Level**: 9 - Clear requirements with established patterns to follow, comprehensive research completed