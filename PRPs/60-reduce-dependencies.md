# PRP: Reduce Build Time Through Strategic Dependency Management

## Problem Statement
The project currently has 463 dependencies (counted via `grep -c "^name = " Cargo.lock`) leading to extremely long build times. Many heavy dependencies are included by default even when not needed for core functionality. This impacts developer productivity and CI/CD pipeline performance.

## Objectives
- Reduce default dependency count by 60-80% through aggressive feature-gating
- Create a minimal core build configuration that compiles in under 1 minute
- Maintain full functionality through opt-in feature flags
- Improve build parallelization by breaking dependency chains
- Document feature flag combinations for common use cases

## Context and Research

### Current State Analysis
Based on analysis of the codebase:
- **Core crates**: ds-rs, cpuinfer, source-videos, dsl
- **Heavy dependency chains identified**:
  - ONNX Runtime (ort) + ndarray + half → ~150+ transitive deps
  - Image processing (image crate) → ravif, exr, tiff, webp → ~100+ deps
  - Web server stack (axum + tower + hyper + tokio) → ~200+ deps
  - CLI tools (clap, rustyline, comfy-table) → ~50+ deps
  - File watching (notify) → ~30+ deps

### Existing Feature Gates
The project already uses some feature-gating:
- `ds-rs`: onnx, cpu_vision, cairo-rs, logging features
- `cpuinfer`: ort feature (enabled by default)
- No feature gates in `source-videos` crate

### Best Practices Research
Reference Rust projects with effective feature-gating:
- **tokio**: Granular features for each runtime component
- **image**: Format-specific features (jpeg, png, gif, etc.)
- **serde**: Separate derive feature
- Documentation: https://doc.rust-lang.org/cargo/reference/features.html

## Implementation Blueprint

### Phase 1: Core Dependency Audit and Categorization
1. Identify absolute minimum dependencies for core GStreamer functionality
2. Map each dependency to its usage (search for import statements)
3. Create dependency groups by functionality domain
4. Document which features/examples require each dependency

### Phase 2: Restructure Feature Flags

#### Core Features (minimal set)
- `core`: Only GStreamer bindings and basic pipeline management
- No image processing, no ML inference, no web server, no CLI

#### Feature Groups to Create
- `cli`: clap + colored + comfy-table + rustyline
- `inference`: ort + ndarray + half + cpuinfer with ort
- `image-full`: Complete image crate with all formats
- `image-minimal`: Image crate with only essential formats (jpeg, png)
- `web-server`: axum + tower + hyper + tokio runtime
- `file-watch`: notify + walkdir
- `visualization`: cairo-rs
- `dev-tools`: sysinfo + env_logger
- `video-server`: Everything needed for source-videos functionality

### Phase 3: Refactor Cargo.toml Files

#### Workspace-level Changes
Move more dependencies to workspace level for consistency

#### ds-rs/Cargo.toml Restructuring
```toml
[features]
default = ["core"]  # Minimal by default
core = ["gstreamer", "gstreamer-base", "minimal-deps"]
full = ["cli", "inference", "image-full", "web-server", "visualization"]
minimal-deps = ["parking_lot", "once_cell", "thiserror", "log"]
cli = ["dep:clap", "dep:ctrlc"]
inference = ["cpuinfer/ort", "dep:nalgebra", "dep:ndarray", "dep:half"]
# ... etc
```

#### Split source-videos Into Optional Crate
Consider making source-videos a separate workspace member that's not built by default

### Phase 4: Code Refactoring

#### Conditional Compilation
Add `#[cfg(feature = "...")]` guards around feature-specific code:
- CLI argument parsing
- Image loading/processing  
- ML inference paths
- Web server endpoints

#### Abstract Feature-Specific APIs
Create traits that have different implementations based on features

### Phase 5: Optimize Remaining Dependencies

#### Replace Heavy Dependencies
- Consider replacing `tokio` with `smol` or removing async where not needed (TODO comment exists)
- Replace `image` with format-specific crates when only one format needed
- Use `tinytemplate` instead of full templating engines if applicable

#### Minimize Feature Flags on Dependencies
Review all dependency feature flags and disable unnecessary ones

## Implementation Tasks

### Setup and Analysis
1. Create comprehensive dependency usage map
2. Benchmark current build times (clean build, incremental)
3. Set up build time tracking infrastructure

### Core Refactoring
4. Create new feature flag structure in ds-rs/Cargo.toml
5. Move non-essential dependencies behind feature gates
6. Add conditional compilation directives to code
7. Update cpuinfer to not default to ort feature
8. Create minimal core configuration

### source-videos Isolation  
9. Move source-videos web/CLI deps behind feature flag
10. Make source-videos an optional workspace member
11. Create separate feature for video server functionality

### Image Processing Optimization
12. Split image crate features by format
13. Create image-minimal vs image-full features
14. Add conditional compilation for image processing code

### Build Configuration
15. Create cargo aliases for common feature combinations
16. Document feature flag combinations in README
17. Update CI/CD to test different feature sets

### Testing and Validation
18. Ensure all tests pass with minimal features
19. Create feature-specific test suites
20. Verify examples work with appropriate features

## Validation Gates

```bash
# Check that minimal build works
cargo build --no-default-features --features core

# Verify significant reduction in dependencies
cargo tree --no-default-features --features core | wc -l  # Should be < 100

# Ensure all feature combinations build
cargo build --all-features
cargo test --all-features

# Benchmark build time improvement
time cargo build --release --no-default-features --features core  # Target: < 60 seconds

# Verify no functionality regression
cargo test --all-features --all-targets
```

## Success Metrics
- Default build has < 100 direct and transitive dependencies
- Clean build time reduced by > 50%
- All existing functionality preserved through feature flags
- Clear documentation of feature requirements
- CI pipeline time reduced by > 40%

## Risk Mitigation
- Create compatibility feature flag that enables all current default features
- Extensive testing of all feature combinations
- Gradual rollout with backwards compatibility
- Clear migration guide for users

## Documentation Requirements
- Update README with feature flag guide
- Create FEATURES.md with detailed feature descriptions
- Add feature requirements to each example
- Update CLAUDE.md with new build commands

## Reference Material
- Cargo Features Documentation: https://doc.rust-lang.org/cargo/reference/features.html
- Best Practices: https://rust-lang.github.io/api-guidelines/
- Conditional Compilation: https://doc.rust-lang.org/reference/conditional-compilation.html
- Workspace Dependencies: https://doc.rust-lang.org/cargo/reference/workspaces.html

## Implementation Order
1. Start with cpuinfer - remove ort from default features
2. Then ds-rs - create minimal core configuration  
3. Finally source-videos - make it fully optional
4. Test each phase thoroughly before proceeding

## Notes for AI Implementation
- Check crates/ds-rs/src/ for actual usage of each dependency
- Look for `use` statements and feature-specific code patterns
- Reference existing feature gates as patterns to follow
- Ensure backwards compatibility with current default behavior
- Focus on build time reduction without breaking functionality

**Confidence Score: 8/10** - Clear path with concrete steps, may need adjustment based on deeper code analysis