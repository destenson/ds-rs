# PRP: Code Generation System for GStreamer Element Types

## Executive Summary

Build upon PRP-14's parser foundation to generate strongly-typed Rust wrappers for discovered GStreamer elements at compile time. This system will create type-safe builder patterns, property setters with correct types, and signal/action bindings automatically from parsed element metadata.

## Problem Statement

### Current State
- Manual element creation with string-based property names
- No compile-time validation of property types or values
- Runtime errors for invalid property names or types
- Manual maintenance of element wrapper code
- No IDE autocomplete for element-specific properties

### Desired State
- Generated Rust types for each discovered element
- Type-safe property setters with proper Rust types
- Compile-time validation of property names and types
- IDE autocomplete and documentation from gst-inspect
- Zero-cost abstractions maintaining runtime performance

### Business Value
Dramatically improves developer productivity through type safety and IDE support while eliminating entire classes of runtime errors related to element configuration.

## Requirements

### Functional Requirements

1. **Type Generation**: Generate Rust structs for each element
2. **Builder Pattern**: Create builder types with fluent API
3. **Property Methods**: Generate typed setters for each property
4. **Signal Bindings**: Generate type-safe signal connection methods
5. **Action Methods**: Generate typed action invocation methods
6. **Documentation**: Include gst-inspect descriptions as doc comments

### Non-Functional Requirements

1. **Zero-Cost**: Generated code must have no runtime overhead
2. **Compatibility**: Work alongside existing manual element creation
3. **Incremental**: Only regenerate when elements change
4. **Debuggable**: Generated code should be readable and debuggable

### Context and Research

Code generation in Rust build scripts typically writes to `$OUT_DIR` and uses `include!` macro to include generated code. The generated code should follow Rust naming conventions (snake_case for methods, CamelCase for types).

Property types from gst-inspect map to Rust types:
- Boolean -> bool
- Integer/Unsigned Integer -> i32/u32 with range validation
- Float/Double -> f32/f64
- String -> &str or String
- Enum -> Generated enum type
- Flags -> Bitflags type

### Documentation & References

```yaml
- url: https://github.com/rust-lang/rust-bindgen/blob/master/bindgen/codegen/mod.rs
  why: Reference implementation of Rust code generation

- url: https://docs.rs/quote/latest/quote/
  why: Quasi-quoting for code generation

- url: https://docs.rs/proc-macro2/latest/proc_macro2/
  why: Token stream manipulation for code generation

- file: crates/ds-rs/src/elements/mod.rs
  why: Current element patterns to maintain compatibility

- file: crates/ds-rs/src/pipeline/builder.rs
  why: Builder pattern implementation to mirror

- url: https://rust-lang.github.io/api-guidelines/naming.html
  why: Rust API naming guidelines for generated code

- url: https://docs.rs/bitflags/latest/bitflags/
  why: For generating flag types from GStreamer flags
```

### List of tasks to be completed

```yaml
Task 1:
CREATE crates/ds-rs/src/build_support/codegen/mod.rs:
  - DESIGN CodeGenerator struct
  - IMPLEMENT generate_element_types(elements: Vec<GstElementInfo>)
  - COORDINATE type, builder, and method generation
  - WRITE generated code to OUT_DIR/elements.rs

Task 2:
CREATE crates/ds-rs/src/build_support/codegen/types.rs:
  - MAP GStreamer types to Rust types
  - HANDLE Integer ranges as const generics or validation
  - GENERATE enum types from string choices
  - CREATE bitflags for flag properties
  - HANDLE optional vs required properties

Task 3:
CREATE crates/ds-rs/src/build_support/codegen/element.rs:
  - GENERATE struct ElementNameElement
  - ADD inner: gst::Element field
  - IMPLEMENT Deref to gst::Element
  - ADD phantom data for compile-time element type
  - GENERATE From<gst::Element> with runtime validation

Task 4:
CREATE crates/ds-rs/src/build_support/codegen/builder.rs:
  - GENERATE struct ElementNameBuilder
  - CREATE new() constructor
  - ADD typed property setter methods
  - IMPLEMENT build() -> Result<ElementNameElement>
  - ADD validation for property constraints

Task 5:
CREATE crates/ds-rs/src/build_support/codegen/properties.rs:
  - GENERATE setter methods for each property
  - ADD doc comments from property descriptions
  - IMPLEMENT range validation for numeric types
  - HANDLE property flags (readable, writable, controllable)
  - CREATE getter methods for readable properties

Task 6:
CREATE crates/ds-rs/src/build_support/codegen/signals.rs:
  - GENERATE connect_signal_name() methods
  - MAP signal parameters to Rust types
  - CREATE type-safe callback signatures
  - ADD signal emission methods where applicable

Task 7:
CREATE crates/ds-rs/src/build_support/codegen/actions.rs:
  - GENERATE action invocation methods
  - MAP action parameters and return types
  - CREATE type-safe method signatures
  - HANDLE GValue conversions

Task 8:
UPDATE build.rs:
  - LOAD parsed element data from PRP-14
  - INVOKE code generator
  - WRITE elements.rs to OUT_DIR
  - SET rerun conditions for generated files

Task 9:
CREATE crates/ds-rs/src/generated/mod.rs:
  - INCLUDE generated code with include!(concat!(env!("OUT_DIR"), "/elements.rs"))
  - RE-EXPORT generated types
  - ADD feature flag for optional generation

Task 10:
ADD Cargo.toml dependencies:
  - ADD quote for code generation
  - ADD proc-macro2 for token streams
  - ADD heck for case conversion
  - MARK as build-dependencies

Task 11:
CREATE integration tests:
  - TEST generated type compilation
  - VERIFY property type safety
  - TEST builder pattern usage
  - VALIDATE signal connections
```

### Out of Scope
- Procedural macros (covered in PRP-16)
- Runtime integration (covered in PRP-17)
- Custom derive macros
- Dynamic property discovery at runtime

## Success Criteria

- [ ] Generated code compiles without warnings
- [ ] All property types correctly mapped to Rust types
- [ ] Builder pattern provides ergonomic API
- [ ] Documentation appears in IDE autocomplete
- [ ] No runtime performance overhead vs manual creation
- [ ] Generated code under 10MB for ~200 elements

## Dependencies

### Technical Dependencies
- Successful completion of PRP-14
- Quote and proc-macro2 crates
- Bitflags for flag types
- Heck for naming conversions

### Knowledge Dependencies
- Rust code generation patterns
- GStreamer type system
- Builder pattern implementation

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Large generated code size | Medium | Low | Conditional compilation and features |
| Name conflicts | Low | Medium | Namespace isolation and prefixing |
| Type mapping complexity | High | Medium | Conservative mapping with runtime validation |
| Compilation time impact | Medium | Medium | Incremental generation and caching |

## Architecture Decisions

### Decision: Code Generation Approach
**Options Considered:**
1. String concatenation
2. Quote/proc-macro2 quasi-quoting
3. Template engine (Tera/Handlebars)

**Decision:** Option 2 - Quote/proc-macro2

**Rationale:** Type-safe code generation, better IDE support, standard in Rust ecosystem

### Decision: Property Type Mapping
**Options Considered:**
1. Direct 1:1 mapping with runtime validation
2. Newtype wrappers for each property
3. Generic types with const constraints

**Decision:** Option 1 - Direct mapping with validation

**Rationale:** Simpler API, less code bloat, validation still occurs at build time where possible

## Validation Strategy

- **Compilation Tests**: Ensure generated code compiles
- **Type Safety Tests**: Verify property type checking
- **Integration Tests**: Use generated types in pipelines
- **Benchmark Tests**: Compare performance vs manual creation

## Future Considerations

- Incremental compilation optimization
- Custom derive for element traits
- Async signal handling
- Property change notifications
- Dynamic element loading

## References

- Rust Code Generation with Quote
- GStreamer Type System Documentation
- Builder Pattern in Rust
- Bindgen Code Generation Patterns

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-08-23
- **Last Modified**: 2025-08-23
- **Status**: Complete
- **Confidence Level**: 7 - Complex code generation but well-established patterns