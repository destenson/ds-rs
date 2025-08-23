# PRP: Procedural Macros and Runtime Property Discovery

## Executive Summary

Create procedural macros that provide ergonomic runtime property discovery and type-safe element creation, bridging the gap between compile-time generated types and runtime element availability. This system enables fallback to runtime discovery when compile-time generation is unavailable.

## Problem Statement

### Current State
- Generated types only available if GStreamer present at compile time
- No way to use elements not discovered during build
- Manual property setting for dynamically loaded plugins
- No runtime validation of property types

### Desired State
- Procedural macros for element creation with or without generation
- Runtime property discovery and validation
- Hybrid approach supporting both static and dynamic elements
- Graceful fallback when generated types unavailable

### Business Value
Provides maximum flexibility for element usage while maintaining type safety where possible, enabling deployment in environments where GStreamer configuration differs from build environment.

## Requirements

### Functional Requirements

1. **Element Creation Macro**: `gst_element!` macro for type-safe creation
2. **Property Setting Macro**: Type-checked property assignment
3. **Runtime Discovery**: Query element properties at runtime
4. **Fallback Mode**: Work without compile-time generation
5. **Validation Layer**: Runtime type checking for properties
6. **Plugin Loading**: Support for dynamically loaded plugins

### Non-Functional Requirements

1. **Performance**: Minimal overhead for macro expansion
2. **Compatibility**: Work with and without generated types
3. **Error Messages**: Clear compile-time error messages
4. **Flexibility**: Support both static and dynamic usage

### Context and Research

Procedural macros in Rust require a separate crate with `proc-macro = true`. The macro can check for the existence of generated types using conditional compilation and fall back to runtime discovery when unavailable.

The GStreamer API provides runtime property introspection through `gst::Element::list_properties()` and property type information through GObject introspection.

### Documentation & References

```yaml
- url: https://doc.rust-lang.org/reference/procedural-macros.html
  why: Official documentation for procedural macros

- url: https://docs.rs/syn/latest/syn/
  why: Parsing Rust syntax in procedural macros

- url: https://docs.rs/darling/latest/darling/
  why: Macro argument parsing helper

- file: crates/ds-rs/src/elements/factory.rs
  why: Current factory patterns to integrate with

- url: https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/object/trait.ObjectExt.html#tymethod.list_properties
  why: Runtime property discovery in GObject

- url: https://docs.rs/gstreamer/latest/gstreamer/trait.ElementExt.html
  why: GStreamer element property methods

- file: crates/ds-rs/src/pipeline/builder.rs
  why: Builder patterns to enhance with macros
```

### List of tasks to be completed

```yaml
Task 1:
CREATE crates/ds-rs-macros/Cargo.toml:
  - SET proc-macro = true
  - ADD syn with full features
  - ADD quote for code generation
  - ADD proc-macro2 for token streams
  - ADD darling for attribute parsing

Task 2:
CREATE crates/ds-rs-macros/src/lib.rs:
  - DEFINE gst_element! procedural macro
  - DEFINE element! attribute macro
  - EXPORT macro entry points
  - SETUP error handling

Task 3:
CREATE crates/ds-rs-macros/src/element_macro.rs:
  - PARSE macro invocation syntax
  - CHECK for generated type existence
  - GENERATE element creation code
  - HANDLE property assignments
  - ADD compile-time validation where possible

Task 4:
CREATE crates/ds-rs-macros/src/property_validation.rs:
  - IMPLEMENT property type checking
  - VALIDATE property names against known properties
  - GENERATE type conversions
  - ADD range validation for numeric types
  - HANDLE optional properties

Task 5:
CREATE crates/ds-rs/src/runtime/discovery.rs:
  - IMPLEMENT ElementIntrospector struct
  - ADD discover_properties(element_name: &str)
  - PARSE GObject property information
  - CACHE discovered properties
  - PROVIDE property type information

Task 6:
CREATE crates/ds-rs/src/runtime/validator.rs:
  - IMPLEMENT PropertyValidator trait
  - ADD validate_property(name, value, property_info)
  - CHECK type compatibility
  - VALIDATE numeric ranges
  - HANDLE enum/flags validation

Task 7:
CREATE crates/ds-rs/src/runtime/fallback.rs:
  - IMPLEMENT FallbackElementBuilder
  - ADD property setting with runtime validation
  - INTEGRATE with ElementIntrospector
  - PROVIDE error messages for invalid properties
  - SUPPORT dynamic plugin loading

Task 8:
UPDATE crates/ds-rs/src/elements/mod.rs:
  - ADD macro re-exports
  - INTEGRATE runtime discovery
  - ADD feature flags for conditional compilation
  - PROVIDE unified API surface

Task 9:
CREATE macro usage examples:
  - DEMONSTRATE gst_element! usage
  - SHOW property setting patterns
  - ILLUSTRATE fallback behavior
  - ADD migration guide from manual creation

Task 10:
CREATE crates/ds-rs-macros/tests/compile_tests.rs:
  - TEST successful macro expansion
  - TEST compile-time error messages
  - VERIFY property validation
  - TEST with and without generated types

Task 11:
UPDATE workspace Cargo.toml:
  - ADD ds-rs-macros to workspace members
  - CONFIGURE macro crate dependencies
  - SET feature flags for conditional compilation
```

### Out of Scope
- Complex type inference
- Custom derive macros for user types
- Macro-based pipeline DSL
- Property binding systems

## Success Criteria

- [ ] Macros provide clean syntax for element creation
- [ ] Compile-time errors for invalid properties when possible
- [ ] Runtime fallback works without generated types
- [ ] Performance overhead < 5% vs direct creation
- [ ] Clear error messages for property mismatches
- [ ] Works with dynamically loaded plugins

## Dependencies

### Technical Dependencies
- Procedural macro support in Rust
- Syn and quote crates
- GObject introspection APIs
- Completion of PRP-14 and PRP-15 preferred

### Knowledge Dependencies
- Procedural macro implementation
- GObject property system
- Conditional compilation techniques

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Macro complexity | High | Medium | Start with simple cases, iterate |
| Poor error messages | Medium | High | Invest in error formatting |
| Runtime overhead | Low | Medium | Benchmark and optimize hot paths |
| Version compatibility | Medium | Low | Feature detection and versioning |

## Architecture Decisions

### Decision: Macro Strategy
**Options Considered:**
1. Function-like procedural macros only
2. Attribute macros on structs
3. Hybrid with both macro types

**Decision:** Option 3 - Hybrid approach

**Rationale:** Maximum flexibility, natural syntax for different use cases

### Decision: Fallback Implementation
**Options Considered:**
1. Compile-time feature flag switching
2. Runtime detection with dynamic dispatch
3. Conditional compilation with cfg attributes

**Decision:** Option 2 - Runtime detection

**Rationale:** Works in all deployment scenarios, single binary distribution

## Validation Strategy

- **Compile Tests**: Verify macro expansion correctness
- **Runtime Tests**: Validate property discovery
- **Integration Tests**: Use macros in real pipelines
- **Error Tests**: Verify error message quality

## Future Considerations

- Macro-based pipeline DSL
- Property change notifications
- Two-way property binding
- Async property updates
- IDE macro expansion support

## References

- The Little Book of Rust Macros
- Procedural Macros Workshop
- GObject Introspection Documentation
- Syn and Quote Documentation

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Complete
- **Confidence Level**: 6 - Complex macro implementation but achievable with established patterns