# PRP: Reduce Dependencies Through Modular Crate Architecture

## Problem Statement
The project currently has 463 dependencies because all functionality is bundled into a few large crates. This creates unnecessary coupling where users who only need core GStreamer functionality must compile web servers, CLI tools, image processing, and ML inference libraries. The monolithic structure also increases build times and makes the codebase harder to maintain.

## Objectives
- Split the project into 10-15 focused, single-responsibility crates
- Each crate should have minimal dependencies for its specific purpose
- Enable users to pick only the functionality they need
- Reduce total dependencies for common use cases by 70-80%
- Improve build parallelization and incremental compilation
- Create clear API boundaries between components

## Context and Research

### Current Architecture Problems
- `ds-rs` crate bundles: GStreamer pipelines, ML inference, image processing, CLI, visualization
- `source-videos` includes: Web server, RTSP server, CLI REPL, file watching
- `cpuinfer` is relatively focused but still pulls in heavy ML dependencies by default
- No clear separation between runtime and development tools

### Successful Modular Rust Projects
Reference architectures:
- **tokio ecosystem**: tokio-core, tokio-util, tokio-stream, tokio-test
- **async-std**: Separate crates for different runtime components  
- **gstreamer-rs**: Already split into gstreamer, gstreamer-video, gstreamer-app, etc.
- **bevy engine**: Modular game engine with 50+ optional crates

## Implementation Blueprint

### Proposed Crate Structure

#### Core Runtime Crates (minimal deps)
1. **`ds-core`** - Core traits, types, error handling (no deps except std)
2. **`ds-pipeline`** - GStreamer pipeline abstractions (only gstreamer deps)
3. **`ds-backend`** - Backend trait definitions and mock implementation
4. **`ds-source`** - Video source management abstractions

#### Backend Implementation Crates
5. **`ds-backend-deepstream`** - NVIDIA DeepStream backend
6. **`ds-backend-standard`** - Standard GStreamer backend
7. **`ds-backend-cpu`** - CPU inference backend (with ort)

#### Feature-Specific Crates
8. **`ds-inference`** - ML inference abstractions (without implementations)
9. **`ds-inference-ort`** - ONNX Runtime implementation
10. **`ds-inference-tflite`** - TensorFlow Lite implementation
11. **`ds-tracker`** - Object tracking algorithms
12. **`ds-visualization`** - Cairo-based visualization

#### Tool Crates
13. **`ds-cli`** - Command-line interface (clap, colored)
14. **`ds-repl`** - Interactive REPL (rustyline)
15. **`ds-server`** - Web/RTSP server components (axum, tower)
16. **`ds-file-watch`** - File system monitoring (notify)

#### Development/Testing Crates
17. **`ds-test-utils`** - Testing utilities and mocks
18. **`ds-examples`** - Example applications

### Dependency Flow Architecture

```
User Application
       ↓
   ds-pipeline (5 deps)
       ↓
   ds-backend trait (0 deps)
       ↓
   [Choose one or more]
   - ds-backend-standard (10 deps)
   - ds-backend-deepstream (15 deps)  
   - ds-backend-cpu + ds-inference-ort (150 deps)
```

### Migration Strategy

#### Phase 1: Create Core Abstractions
1. Extract trait definitions to `ds-core`
2. Move pipeline abstractions to `ds-pipeline`
3. Create `ds-backend` with trait definitions
4. Ensure these compile with minimal dependencies

#### Phase 2: Split Backend Implementations
5. Move each backend to its own crate
6. Make DeepStream backend optional
7. CPU inference becomes separate crate

#### Phase 3: Extract Tools and Servers
8. Move CLI to `ds-cli` crate
9. Extract web/RTSP servers to `ds-server`
10. Separate REPL into `ds-repl`

#### Phase 4: Modularize Features
11. Split inference implementations
12. Extract visualization to separate crate
13. Create focused tracker crate

### Example Cargo.toml After Split

For a user who only needs basic pipeline functionality:
```toml
[dependencies]
ds-pipeline = "0.2"
ds-backend-standard = "0.2"
# Total: ~20 dependencies instead of 463
```

For ML inference use case:
```toml
[dependencies]
ds-pipeline = "0.2"
ds-backend-cpu = "0.2"
ds-inference-ort = "0.2"
# Total: ~170 dependencies (only ML-related)
```

For full application with CLI:
```toml
[dependencies]
ds-pipeline = "0.2"
ds-backend-deepstream = "0.2"
ds-cli = "0.2"
# Total: ~50 dependencies
```

## Implementation Tasks

### Setup
1. Create workspace structure with new crate directories
2. Set up inter-crate dependencies in workspace Cargo.toml
3. Establish versioning strategy for crates

### Core Extraction
4. Extract core traits and types to ds-core
5. Move pipeline abstractions to ds-pipeline
6. Create backend trait crate
7. Verify minimal dependency count for core crates

### Backend Separation  
8. Extract DeepStream backend to separate crate
9. Extract standard backend to separate crate
10. Move CPU inference to dedicated crate
11. Update backend manager to load backends dynamically

### Tool Extraction
12. Move CLI code to ds-cli crate
13. Extract web server to ds-server
14. Separate REPL into ds-repl
15. Move file watching to ds-file-watch

### Feature Modularization
16. Split inference implementations by framework
17. Extract visualization components
18. Create dedicated tracker crate

### Integration
19. Update examples to use new crate structure
20. Create integration tests across crate boundaries
21. Document crate selection guide

## Validation Gates

```bash
# Verify core crates have minimal dependencies
cargo tree -p ds-core --no-default-features | wc -l  # Should be < 10
cargo tree -p ds-pipeline --no-default-features | wc -l  # Should be < 30

# Check that crates can be used independently
cargo build -p ds-pipeline --no-default-features
cargo build -p ds-backend-standard

# Ensure examples work with new structure
cargo run --example basic_pipeline -p ds-examples

# Verify significant dependency reduction
cargo build -p ds-pipeline -p ds-backend-standard
cargo tree | wc -l  # Should be < 100 for basic usage

# All tests should pass
cargo test --workspace
```

## Benefits of This Approach

### For Users
- Only compile what you need
- Faster build times for specific use cases
- Clearer understanding of dependencies
- Better API stability (smaller surface area per crate)

### For Development
- Parallel development on different crates
- Clearer ownership and responsibilities
- Easier to test in isolation
- Better incremental compilation
- Simpler to add new backends or features

### For CI/CD
- Can test crates in parallel
- Faster feedback on changes
- Only affected crates need rebuilding
- Can have different release cycles

## Migration Guide

### For Current Users
```toml
# Old way
ds-rs = "0.1"

# New way - choose what you need
ds-pipeline = "0.2"
ds-backend-standard = "0.2"
ds-cli = { version = "0.2", optional = true }
```

### Feature Flag Mapping
- `onnx` → use `ds-inference-ort` crate
- `cairo-rs` → use `ds-visualization` crate
- CLI features → use `ds-cli` crate

## Success Metrics
- Basic GStreamer usage requires < 30 dependencies
- Full CLI application requires < 100 dependencies  
- ML inference standalone requires < 200 dependencies
- Clean build of basic pipeline < 30 seconds
- Incremental builds < 5 seconds

## Notes for Implementation
- Start with extracting core traits - this is least disruptive
- Keep backward compatibility through a `ds-rs` meta-crate initially
- Each crate should have its own README and examples
- Use workspace features to manage common dependencies
- Consider using cargo workspaces' inheritance features

**Confidence Score: 9/10** - Modular architecture is the right approach for long-term maintainability and will naturally solve the dependency problem while providing other benefits.