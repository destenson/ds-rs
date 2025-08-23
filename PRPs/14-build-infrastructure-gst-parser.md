# PRP: Build Infrastructure and GStreamer Element Parser Foundation

## Executive Summary

Establish build-time infrastructure to query available GStreamer elements using `gst-inspect-1.0` and parse their structured text output into Rust data structures. This foundational PRP sets up the build script, parser infrastructure, and intermediate representation needed for subsequent code generation PRPs.

## Problem Statement

### Current State
- Elements are created using hardcoded strings and manual property setting
- No compile-time validation of element names or properties
- Backend abstraction requires manual mapping between element names
- No type safety for element-specific properties
- Manual maintenance of element definitions in DeepStreamElementType enum

### Desired State
- Build script that discovers available GStreamer elements at compile time
- Structured parser for `gst-inspect-1.0` output
- Intermediate representation (IR) of element metadata
- Foundation for automatic type generation in subsequent PRPs
- Cache mechanism to avoid re-parsing unchanged elements

### Business Value
Enables type-safe GStreamer element usage with compile-time validation, reducing runtime errors and improving developer experience through IDE autocomplete and type checking.

## Requirements

### Functional Requirements

1. **Build Script Setup**: Create build.rs that executes during compilation
2. **Element Discovery**: Query available GStreamer elements using `gst-inspect-1.0`
3. **Output Parser**: Parse structured text output into Rust data structures
4. **Intermediate Representation**: Define data structures for element metadata
5. **Caching System**: Cache parsed results to avoid unnecessary re-parsing
6. **Error Handling**: Graceful degradation when gst-inspect is unavailable

### Non-Functional Requirements

1. **Performance**: Build time impact < 5 seconds for full rebuild
2. **Reliability**: Build must not fail if GStreamer is not installed
3. **Maintainability**: Parser must handle different gst-inspect versions
4. **Compatibility**: Support Windows, Linux, and macOS platforms

### Context and Research

The `gst-inspect-1.0` tool outputs structured text with consistent sections:
- Factory Details (rank, name, class, description)
- Plugin Details (name, filename, version, license)
- GObject hierarchy showing inheritance
- Implemented Interfaces (for elements that implement GStreamer interfaces)
- Element Flags (SOURCE, SINK, REQUIRE_CLOCK, etc.)
- Pad Templates with capabilities
- Element Properties with types, flags, ranges, and defaults
- Element Signals (callback signatures for element events)
- Element Actions (callable methods with return types)
- Children (for bin elements containing other elements)

Build scripts in Rust use `build.rs` in the package root and write to `OUT_DIR`. The script communicates with Cargo via `println!("cargo::rerun-if-changed=...")` directives.

### Documentation & References

```yaml
- url: https://doc.rust-lang.org/cargo/reference/build-scripts.html
  why: Official Cargo documentation for build scripts

- url: https://doc.rust-lang.org/cargo/reference/build-script-examples.html
  why: Examples of build script patterns and best practices

- url: https://gstreamer.freedesktop.org/documentation/tools/gst-inspect.html
  why: Official documentation for gst-inspect-1.0 tool

- file: crates/ds-rs/src/elements/mod.rs
  why: Current element abstraction patterns to maintain compatibility

- file: crates/ds-rs/src/backend/detector.rs
  why: Existing element detection logic to integrate with

- url: https://docs.rs/regex/latest/regex/
  why: For parsing structured text patterns

- url: https://docs.rs/serde_json/latest/serde_json/
  why: For caching parsed data as JSON
```

### List of tasks to be completed

```yaml
Task 1:
CREATE crates/ds-rs/build.rs:
  - IMPLEMENT main() function with error handling
  - DETECT GStreamer installation via gst-inspect-1.0
  - SET cargo rerun conditions
  - CREATE output directory structure

Task 2:
CREATE crates/ds-rs/src/build_support/mod.rs:
  - DEFINE GstElementInfo struct for parsed data
  - DEFINE GstPropertyInfo with type information
  - DEFINE GstPadTemplate for pad metadata
  - DEFINE GstSignalInfo for signal signatures
  - DEFINE GstActionInfo for action methods
  - DEFINE GstInterfaceInfo for implemented interfaces
  - IMPLEMENT serialization traits for caching

Task 3:
CREATE crates/ds-rs/src/build_support/discovery.rs:
  - IMPLEMENT list_available_elements() using gst-inspect-1.0
  - FILTER relevant elements (exclude deprecated/debug)
  - HANDLE platform-specific command execution
  - RETURN Vec<String> of element names

Task 4:
CREATE crates/ds-rs/src/build_support/parser.rs:
  - IMPLEMENT parse_element_info(element_name: &str)
  - PARSE Factory Details section
  - PARSE Plugin Details section
  - PARSE GObject hierarchy
  - PARSE Implemented Interfaces section
  - PARSE Element Flags section
  - PARSE Pad Templates and capabilities
  - PARSE Element Properties with metadata
  - PARSE Element Signals with signatures
  - PARSE Element Actions with return types
  - PARSE Children section for bins
  - HANDLE multi-line descriptions and signatures

Task 5:
CREATE crates/ds-rs/src/build_support/cache.rs:
  - IMPLEMENT load_cache() from OUT_DIR
  - IMPLEMENT save_cache() to OUT_DIR
  - TRACK element versions for invalidation
  - USE serde_json for serialization

Task 6:
INTEGRATE with build.rs:
  - DISCOVER available elements
  - CHECK cache for each element
  - PARSE uncached elements
  - SAVE parsed data to cache
  - WRITE element list to OUT_DIR/elements.json

Task 7:
ADD Cargo.toml build dependencies:
  - ADD serde with derive feature
  - ADD serde_json for caching
  - ADD regex for parsing
  - MARK as build-dependencies

Task 8:
CREATE tests for parser:
  - TEST with sample gst-inspect output
  - VERIFY property type extraction
  - TEST pad template parsing
  - VALIDATE error handling
```

### Out of Scope
- Code generation (covered in PRP-15)
- Macro implementation (covered in PRP-16)
- Runtime integration (covered in PRP-17)
- Property validation logic

## Success Criteria

- [ ] Build script executes without errors on all platforms
- [ ] Successfully parses videotestsrc, identity, and queue elements
- [ ] Cache reduces rebuild time by >50% on unchanged elements
- [ ] Parser handles all property types (Boolean, Integer, Enum, Flags, etc.)
- [ ] Graceful fallback when GStreamer is not installed
- [ ] Generated elements.json contains accurate metadata

## Dependencies

### Technical Dependencies
- Cargo build script support
- GStreamer installation with gst-inspect-1.0
- Regex for text parsing
- Serde for serialization

### Knowledge Dependencies
- gst-inspect-1.0 output format
- Rust build script best practices
- Cross-platform process execution

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| gst-inspect format changes | Low | High | Version detection and multiple parser versions |
| Build time regression | Medium | Medium | Aggressive caching and incremental parsing |
| Platform differences | Medium | Low | Platform-specific command handling |
| Missing GStreamer | High | Low | Optional feature flag and graceful degradation |

## Architecture Decisions

### Decision: Parser Implementation
**Options Considered:**
1. Regex-based line-by-line parsing
2. Full parser combinator library (nom)
3. State machine with sections

**Decision:** Option 1 - Regex-based parsing

**Rationale:** Simpler to implement and maintain, adequate for structured text format, minimal dependencies

### Decision: Caching Strategy
**Options Considered:**
1. Single JSON file with all elements
2. Per-element cache files
3. Binary serialization with bincode

**Decision:** Option 1 - Single JSON file

**Rationale:** Simpler cache invalidation, easier debugging, reasonable performance for ~200 elements

## Validation Strategy

- **Unit Tests**: Parser correctness with fixture data
- **Integration Tests**: Build script execution in test project
- **Manual Testing**: Verify on Windows, Linux, macOS
- **Performance Testing**: Measure build time impact

## Future Considerations

- Parallel parsing of multiple elements
- Integration with GStreamer plugin package managers
- Support for custom element paths
- Runtime element discovery for dynamic plugins

## References

- Cargo Build Scripts Documentation
- GStreamer Tools Documentation
- Rust Code Generation Examples
- Similar projects: bindgen, prost-build

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Complete
- **Confidence Level**: 8 - Well-researched with clear implementation path and proven patterns