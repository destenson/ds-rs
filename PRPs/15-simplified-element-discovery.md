# PRP: Simplified GStreamer Element Discovery Leveraging gstreamer-rs

**Status**: NOT STARTED - No compile-time element discovery implemented

## Executive Summary

Create a lightweight compile-time element discovery system that leverages gstreamer-rs's existing excellent property handling and builder patterns. Rather than generating extensive code, this approach focuses on creating a registry of available elements with their metadata, enabling better backend detection and validation while using gstreamer-rs's existing APIs for actual element creation.

## Problem Statement

### Current State
- Backend detection relies on hardcoded element names
- No compile-time knowledge of available GStreamer elements
- Manual maintenance of element mappings between backends
- gstreamer-rs already provides excellent property handling and builders

### Desired State
- Compile-time registry of available elements and their capabilities
- Enhanced backend detection using discovered elements
- Validation layer using gstreamer-rs's existing APIs
- Minimal code generation focused on metadata only

### Business Value
Improves backend selection accuracy and provides compile-time element validation while leveraging the mature gstreamer-rs APIs, reducing complexity and maintenance burden.

## Requirements

### Functional Requirements

1. **Element Discovery**: Query available elements at compile time
2. **Metadata Registry**: Store element metadata for runtime use
3. **Backend Enhancement**: Improve backend detection with discovered elements
4. **Validation Layer**: Validate element availability before creation
5. **Integration**: Work seamlessly with gstreamer-rs APIs

### Non-Functional Requirements

1. **Minimal Overhead**: Small generated code footprint
2. **Compatibility**: Full compatibility with gstreamer-rs
3. **Simplicity**: Leverage existing APIs rather than reinventing
4. **Maintainability**: Simple, focused implementation

### Context and Research

gstreamer-rs already provides:
- `ElementFactory::make()` with builder pattern
- `set_property()` and `set_property_from_str()` for type-safe property setting
- `find_property()` for property discovery
- `ElementFactory::factories_with_type()` for element discovery
- Excellent GObject integration with property serialization

We should focus on compile-time discovery for better backend detection rather than reimplementing what gstreamer-rs already does well.

### Documentation & References

```yaml
- file: ../gstreamer-rs/gstreamer/src/element_factory.rs
  why: Existing ElementFactory implementation to leverage

- file: ../gstreamer-rs/gstreamer/src/gobject.rs
  why: Property handling already implemented

- file: crates/ds-rs/src/backend/detector.rs
  why: Backend detection to enhance with discovered elements

- url: https://gtk-rs.org/gtk-rs-core/stable/latest/docs/gstreamer/
  why: gstreamer-rs documentation for API reference

- file: crates/ds-rs/src/elements/factory.rs
  why: Current factory to enhance with discovery
```

### List of tasks to be completed

```yaml
Task 1:
CREATE crates/ds-rs/build.rs:
  - CHECK for gst-inspect-1.0 availability
  - DISCOVER available elements using gst-inspect -a
  - PARSE element names and types only
  - WRITE simple element registry to OUT_DIR

Task 2:
CREATE crates/ds-rs/src/discovery/registry.rs:
  - DEFINE ElementRegistry struct
  - LOAD discovered elements from build output
  - PROVIDE element availability queries
  - CACHE element-to-backend mappings

Task 3:
UPDATE crates/ds-rs/src/backend/detector.rs:
  - USE ElementRegistry for detection
  - CHECK discovered elements against backend requirements
  - SCORE backends based on element availability
  - IMPROVE detection accuracy

Task 4:
CREATE crates/ds-rs/src/elements/validated.rs:
  - WRAP gstreamer-rs ElementFactory
  - ADD validation before element creation
  - PROVIDE helpful error messages for missing elements
  - DELEGATE to gstreamer-rs for actual creation

Task 5:
UPDATE crates/ds-rs/src/elements/factory.rs:
  - INTEGRATE with ElementRegistry
  - USE gstreamer-rs builders directly
  - ADD compile-time element name validation
  - MAINTAIN backward compatibility

Task 6:
CREATE simple parser for gst-inspect:
  - PARSE element names from gst-inspect -a output
  - EXTRACT element class/type information
  - IGNORE complex property parsing (use gstreamer-rs runtime)
  - GENERATE minimal metadata file

Task 7:
ADD integration with existing code:
  - UPDATE backend implementations to use registry
  - ENHANCE error messages with available alternatives
  - PROVIDE migration path from manual creation
  - MAINTAIN all existing APIs

Task 8:
CREATE tests:
  - TEST element discovery in build script
  - VERIFY backend detection improvements
  - TEST integration with gstreamer-rs APIs
  - VALIDATE error handling
```

### Out of Scope
- Property type generation (use gstreamer-rs)
- Custom builders (use gstreamer-rs ElementFactory)
- Signal/action bindings (use gstreamer-rs)
- Complex code generation

## Success Criteria

- [ ] Element discovery completes in < 2 seconds
- [ ] Backend detection accuracy improved by >30%
- [ ] Zero breaking changes to existing APIs
- [ ] Full compatibility with gstreamer-rs
- [ ] Generated registry < 100KB

## Dependencies

### Technical Dependencies
- gstreamer-rs for element creation and properties
- gst-inspect-1.0 for discovery
- Basic text parsing (no complex dependencies)

### Knowledge Dependencies
- gstreamer-rs API usage
- Build script basics
- Simple text parsing

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| GStreamer not installed | High | Low | Graceful fallback to manual list |
| gst-inspect format changes | Low | Low | Simple parsing, version detection |
| Integration complexity | Low | Medium | Minimal changes, leverage existing |

## Architecture Decisions

### Decision: Leverage gstreamer-rs
**Options Considered:**
1. Full code generation system
2. Minimal discovery with gstreamer-rs usage
3. Runtime-only discovery

**Decision:** Option 2 - Minimal discovery

**Rationale:** gstreamer-rs already provides excellent APIs, focus on what's missing (compile-time discovery)

### Decision: Registry Format
**Options Considered:**
1. Generated Rust code
2. JSON metadata file
3. Simple text list

**Decision:** Option 2 - JSON metadata

**Rationale:** Simple, debuggable, small footprint

## Validation Strategy

- **Build Tests**: Verify discovery works across platforms
- **Integration Tests**: Test with gstreamer-rs APIs
- **Backend Tests**: Verify improved detection
- **Performance Tests**: Measure discovery overhead

## Future Considerations

- Runtime plugin discovery
- Hot-reload of element registry
- Integration with GStreamer plugin managers
- Cloud-based element database

## References

- gstreamer-rs Documentation
- GStreamer Element Discovery
- Build Script Best Practices

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Complete
- **Confidence Level**: 9 - Simple, focused approach leveraging existing mature libraries