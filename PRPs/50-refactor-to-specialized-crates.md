# PRP-50: Refactor ds-rs into Specialized, Reusable Crates

## Problem Statement

The current ds-rs codebase is a monolithic crate containing all functionality from low-level GStreamer abstractions to high-level video processing features. Additionally, the project has 463 dependencies (counted via `grep -c "^name = " Cargo.lock`) causing excessively long build times. This makes it difficult to:
- Reuse specific components independently
- Test individual features in isolation
- Maintain clear separation of concerns
- Allow external projects to use only the parts they need
- Scale development across multiple teams/contributors
- Build projects quickly when only a subset of functionality is needed

## Research & References

### Rust Workspace Best Practices
- https://doc.rust-lang.org/cargo/reference/workspaces.html - Official Cargo workspace documentation
- https://earthly.dev/blog/cargo-workspace-crates/ - Monorepo patterns with Cargo
- https://medium.com/@aleksej.gudkov/rust-workspace-example-a-guide-to-managing-multi-crate-projects-82d318409260 - Multi-crate project management

### Current Structure Analysis
The existing codebase has natural boundaries visible in the module structure at `crates/ds-rs/src/`:
- Error handling (`error/`)
- Platform detection (`platform.rs`)
- Backend abstraction (`backend/`)
- GStreamer elements (`elements/`)
- Pipeline management (`pipeline/`)
- Source management (`source/`)
- Metadata extraction (`metadata/`)
- AI inference (`inference/`)
- Object tracking (`tracking/`)
- Rendering (`rendering/`)
- Multi-stream coordination (`multistream/`)

## Implementation Blueprint

### Naming Strategy
- **`ds-*` prefix**: Used for DeepStream-specific functionality and core pipeline components
- **Generic names**: Used for utilities that are NOT DeepStream-specific (e.g., `cli-utils`, `image-utils`, `video-server`)
- **Clear distinction**: Only crates that actually interface with NVIDIA DeepStream should imply DeepStream functionality

### Dependency Reduction Goals
Each crate should have minimal dependencies for its specific purpose:
- **ds-core**: 0 external dependencies (std only)
- **ds-error**: 1-2 deps (thiserror only)
- **ds-platform**: 0-2 deps (platform detection only)
- **ds-gstreamer**: 5-10 deps (gstreamer crates only)
- **Heavy deps isolated**: Image processing, ML, web servers in separate optional crates

### Phase 1: Core Foundation Crates

#### 1.1 `ds-core` - Common Types & Traits
**Purpose**: Fundamental types, traits, and utilities used across all crates
**Location**: `crates/ds-core/`
**Contents to Move**:
- `timestamp()` function from lib.rs
- Basic result types
- Common trait definitions (to be extracted)
- Shared utility functions
**Target Dependencies**: 0 (std only)

#### 1.2 `ds-error` - Error Handling
**Purpose**: Centralized error types and error handling utilities
**Location**: `crates/ds-error/`
**Contents to Move**:
- `error/mod.rs` → Error type definitions
- `error/classification.rs` → Error classification logic
- Result type alias
- Error conversion implementations
**Target Dependencies**: 1 (thiserror)

#### 1.3 `ds-platform` - Platform Detection
**Purpose**: Platform-specific detection and capabilities
**Location**: `crates/ds-platform/`
**Contents to Move**:
- `platform.rs` → Platform detection logic
- `dll_validator.rs` → Windows DLL validation (conditional compilation)
**Target Dependencies**: 0-2 (cfg-if, platform-specific)

### Phase 2: GStreamer Abstraction Layer

#### 2.1 `ds-gstreamer` - Core GStreamer Abstractions
**Purpose**: Low-level GStreamer pipeline and element abstractions
**Location**: `crates/ds-gstreamer/`
**Contents to Move**:
- `pipeline/` → Pipeline building and state management
- `messages/` → GStreamer message handling
- Basic GStreamer initialization logic from lib.rs

#### 2.2 `ds-elements` - Element Factory & Abstractions
**Purpose**: GStreamer element creation and abstraction
**Location**: `crates/ds-elements/`
**Contents to Move**:
- `elements/` → Element abstraction and factory
- Element registration logic from lib.rs
**Dependencies**: `ds-gstreamer`, `ds-backend`

#### 2.3 `ds-backend` - Backend Abstraction System
**Purpose**: Backend trait system for multiple implementations
**Location**: `crates/ds-backend/`
**Contents to Move**:
- `backend/mod.rs` → Backend trait and manager
- `backend/deepstream.rs` → NVIDIA DeepStream implementation (actual DeepStream)
- `backend/standard.rs` → Standard GStreamer implementation (NOT DeepStream)
- `backend/mock.rs` → Mock implementation for testing
**Dependencies**: `ds-platform`, `ds-error`
**Note**: Only the deepstream.rs implementation is actually DeepStream-related

### Phase 3: Video Processing Components

#### 3.1 `ds-source` - Source Management
**Purpose**: Video source addition, removal, and synchronization
**Location**: `crates/ds-source/`
**Contents to Move**:
- `source/` → All source management code
- Source-specific event handling
**Dependencies**: `ds-gstreamer`, `ds-elements`

#### 3.2 `ds-metadata` - Metadata Extraction
**Purpose**: Extract and process video metadata
**Location**: `crates/ds-metadata/`
**Contents to Move**:
- `metadata/` → Metadata extraction and types
- Bounding box and classification structures
**Dependencies**: `ds-core`

#### 3.3 `ds-inference` - AI Inference Abstractions
**Purpose**: High-level AI model inference interfaces
**Location**: `crates/ds-inference/`
**Contents to Move**:
- `inference/` → Inference configuration and processing
- Model configuration structures
**Dependencies**: `ds-metadata`

#### 3.4 `ds-tracking` - Object Tracking
**Purpose**: Multi-object tracking capabilities
**Location**: `crates/ds-tracking/`
**Contents to Move**:
- `tracking/` → Object tracking logic
- Trajectory and tracking statistics
**Dependencies**: `ds-metadata`

### Phase 4: High-Level Features

#### 4.1 `ds-rendering` - Visualization & Rendering
**Purpose**: Render bounding boxes and visualizations
**Location**: `crates/ds-rendering/`
**Contents to Move**:
- `rendering/` → All rendering implementations
- Metadata bridge for rendering
**Dependencies**: `ds-metadata`, `ds-backend`

#### 4.2 `ds-multistream` - Multi-Stream Management
**Purpose**: Orchestrate multiple video streams
**Location**: `crates/ds-multistream/`
**Contents to Move**:
- `multistream/` → Multi-stream coordination
- Pipeline pool and resource management
**Dependencies**: `ds-pipeline`, `ds-source`

#### 4.3 `ds-health` - Health Monitoring & Recovery
**Purpose**: Fault tolerance, circuit breakers, health monitoring
**Location**: `crates/ds-health/`
**Contents to Move**:
- `source/health.rs` → Health monitoring
- `source/recovery.rs` → Recovery management
- `source/circuit_breaker.rs` → Circuit breaker implementation
- `source/isolation.rs` → Error isolation
**Dependencies**: `ds-source`, `ds-error`

### Phase 5: Application Layer

#### 5.1 `ds-config` - Configuration Management
**Purpose**: Application and component configuration
**Location**: `crates/ds-config/`
**Contents to Move**:
- `config/` → Configuration structures
- `app/config.rs` → Application configuration
**Dependencies**: Most other crates (for their config types)

#### 5.2 `cli-utils` - Command Line Interface (Optional)
**Purpose**: CLI tools and argument parsing
**Location**: `crates/cli-utils/`
**Contents to Move**:
- CLI argument parsing from main.rs
- Command handling logic
**Target Dependencies**: clap, colored, ctrlc (~20 deps)
**Note**: Not DeepStream-specific, just CLI utilities

#### 5.3 `ds-app` - Application Framework
**Purpose**: High-level application runner and coordination
**Location**: `crates/ds-app/`
**Contents to Move**:
- `app/` → Application runner and timers
- Main initialization logic
**Dependencies**: All other crates

#### 5.4 `cpu-inference` - CPU Vision Processing (Optional)
**Purpose**: CPU-based ML inference (alternative to DeepStream GPU inference)
**Location**: Keep in existing `crates/cpuinfer/` but rename
**Contents to Move**:
- `backend/cpu_vision/` → CPU vision elements
- Keep existing detector implementation
**Dependencies**: `ds-backend`, `ds-metadata`
**Target Dependencies**: ort, ndarray, image (~150 deps when enabled)
**Note**: Alternative to DeepStream's GPU inference

#### 5.5 `image-utils` - Image Processing (Optional)
**Purpose**: General image loading and manipulation utilities
**Location**: `crates/image-utils/`
**Contents to Move**:
- Image processing utilities
- Format-specific handlers
**Target Dependencies**: image crate with selected features (~100 deps)
**Note**: Not DeepStream-specific

#### 5.6 `video-server` - Web/RTSP Server (Optional)
**Purpose**: HTTP API and RTSP streaming server
**Location**: Extract from `source-videos` or new crate
**Contents to Move**:
- Web server components from source-videos
- RTSP server functionality
**Target Dependencies**: axum, tower, hyper, tokio (~200 deps)
**Note**: Generic video serving, not DeepStream-specific

## Dependency Graph

```
Layer 0 (Foundation):
  ds-core ← ds-error
           ← ds-platform

Layer 1 (GStreamer):
  ds-gstreamer ← ds-backend
               ← ds-elements

Layer 2 (Processing):
  ds-metadata ← ds-inference
              ← ds-tracking
  ds-source ← ds-health

Layer 3 (Features):
  ds-rendering
  ds-multistream

Layer 4 (Application):
  ds-config
  ds-app (depends on all)

Standalone:
  ds-cpu-vision (cpuinfer)
```

## Migration Strategy

### Step 1: Create New Crate Structure
1. Create all new crate directories with Cargo.toml files
2. Update workspace Cargo.toml to include all members
3. Set up shared workspace dependencies

### Step 2: Move Code Module by Module
Start with zero-dependency crates and work up the dependency tree:
1. Move `ds-core` (no dependencies)
2. Move `ds-error` and `ds-platform`
3. Move `ds-gstreamer` base abstractions
4. Continue layer by layer

### Step 3: Update Import Paths
1. Replace `crate::` imports with explicit crate names
2. Add inter-crate dependencies in Cargo.toml files
3. Ensure all tests still pass after each move

### Step 4: Refactor Public APIs
1. Define clear public interfaces for each crate
2. Hide implementation details behind module privacy
3. Document all public APIs

## Workspace Configuration

Update root `Cargo.toml`:
```toml
[workspace]
members = [
    "crates/ds-core",
    "crates/ds-error",
    "crates/ds-platform",
    "crates/ds-gstreamer",
    "crates/ds-backend",
    "crates/ds-elements",
    "crates/ds-source",
    "crates/ds-metadata",
    "crates/ds-inference",
    "crates/ds-tracking",
    "crates/ds-rendering",
    "crates/ds-multistream",
    "crates/ds-health",
    "crates/ds-config",
    "crates/ds-app",
    "crates/ds-cpu-vision", # renamed from cpuinfer
    # Keep existing crates
    "crates/dsl",
    "crates/source-videos",
]

[workspace.dependencies]
gstreamer = "0.24.1"
gstreamer-app = "0.24.0"
gstreamer-base = "0.24.0"
gstreamer-video = "0.24.1"
log = "0.4.27"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
# Add other common dependencies
```

## Testing Strategy

### Unit Tests
Each crate should have its own unit tests in `src/lib.rs` or `tests/` directory

### Integration Tests
Create integration tests that verify inter-crate communication works correctly

### Example Applications
Update existing examples to use the new crate structure

## Benefits After Refactoring

1. **Reusability**: Each crate can be used independently in other projects
2. **Maintainability**: Clear separation of concerns makes code easier to understand
3. **Testing**: Each crate can be tested in isolation
4. **Compilation**: Only changed crates need recompilation
5. **Documentation**: Each crate has focused, specific documentation
6. **Versioning**: Crates can be versioned independently
7. **Dependency Reduction**: Users only compile dependencies for crates they use
8. **Build Speed**: Minimal builds compile in seconds vs minutes

## Validation Gates

```bash
# Build all crates
cargo build --workspace --all-features

# Run all tests
cargo test --workspace --all-features

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Generate documentation
cargo doc --workspace --no-deps --all-features

# Check for circular dependencies
cargo deps --workspace

# Verify examples still work
cargo run --example ball_tracking_visualization --all-features

# CRITICAL: Verify dependency reduction
# Minimal GStreamer pipeline should have < 50 dependencies
cargo tree -p ds-gstreamer | wc -l  # Target: < 50

# Core crates should have minimal deps
cargo tree -p ds-core --no-default-features | wc -l  # Target: < 5
cargo tree -p ds-error --no-default-features | wc -l  # Target: < 10

# Build time for minimal configuration
time cargo build -p ds-gstreamer --release  # Target: < 30 seconds

# Total unique dependencies for basic usage
cargo tree -p ds-gstreamer -p ds-backend --no-dedupe | grep "^[a-z]" | sort -u | wc -l  # Target: < 100
```

## Success Criteria

- All existing tests pass
- No circular dependencies between crates
- Each crate compiles independently
- Documentation builds without warnings
- Examples run successfully
- Clean separation of concerns achieved
- **Dependency reduction targets met**:
  - Basic GStreamer usage: < 50 dependencies (from 463)
  - Core crates: < 10 dependencies each
  - Build time for minimal config: < 30 seconds
  - Users can opt-in to heavy deps (image, ML, web) as needed

## Risk Assessment

- **High Risk**: Breaking existing functionality during migration
- **Mitigation**: Move code incrementally, test after each move
- **Medium Risk**: Creating circular dependencies
- **Mitigation**: Plan dependency graph carefully, use dependency analysis tools

## Estimated Effort

- Planning & Setup: 2 hours
- Code Migration: 8-10 hours
- Testing & Validation: 3-4 hours
- Documentation: 2 hours
- Total: 15-18 hours

## Confidence Score: 8/10

The refactoring plan is comprehensive with clear boundaries between crates. The main complexity lies in properly managing inter-crate dependencies and ensuring all imports are correctly updated. The incremental migration strategy reduces risk.
