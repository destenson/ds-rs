# PRP: Enhanced Configuration System for Source-Videos

## Executive Summary

Expand the source-videos configuration system to support all new features including directory serving, file watching, network simulation, auto-repeat, and advanced source management. Implement configuration inheritance, profiles, and runtime reloading for maximum flexibility.

## Problem Statement

### Current State
- Basic configuration with limited options
- Single flat configuration structure
- No support for complex source definitions
- Limited runtime configuration changes
- No configuration profiles or templates

### Desired State
- Comprehensive configuration for all features
- Hierarchical configuration with inheritance
- Profile-based configurations for different scenarios
- Runtime configuration hot-reload
- Configuration validation and schema
- Environment variable substitution
- Include/import of configuration fragments

### Business Value
Simplifies complex deployments, enables reproducible test scenarios, supports multi-environment configurations, and reduces configuration errors through validation.

## Requirements

### Functional Requirements

1. **Extended Source Types**: Support directory, file list, and pattern sources
2. **Watch Configuration**: Define watching behavior and rules
3. **Network Simulation**: Configure network conditions per source
4. **Auto-Repeat Settings**: Define looping behavior
5. **Profile Support**: Named configuration profiles
6. **Variable Substitution**: Environment and custom variables
7. **Configuration Includes**: Import configuration fragments
8. **Schema Validation**: Validate configuration against schema

### Non-Functional Requirements

1. **Backward Compatibility**: Existing configs must work
2. **Performance**: Fast configuration parsing and loading
3. **Validation**: Clear error messages for invalid configs
4. **Documentation**: Self-documenting schema
5. **Flexibility**: Support JSON, YAML, and TOML formats

### Context and Research

Current configuration structure in `src/config_types.rs`:
- AppConfig with server and sources
- VideoSourceConfig with basic source types
- Limited to test patterns and basic file sources

Network simulation already exists in `src/network/` with profiles and conditions.

### Documentation & References

```yaml
- file: crates/source-videos/src/config_types.rs
  why: Current configuration structures to extend

- file: crates/source-videos/src/config/mod.rs
  why: Configuration loading and management

- file: crates/source-videos/src/network/profiles.rs
  why: Network profile configuration patterns

- url: https://docs.rs/serde/latest/serde/
  why: Serialization/deserialization for config

- url: https://docs.rs/serde_json/latest/serde_json/value/
  why: Dynamic configuration handling

- url: https://json-schema.org/
  why: JSON schema for validation

- file: crates/source-videos/examples/config_examples/
  why: Example configurations to create
```

### List of tasks to be completed

```yaml
Task 1:
EXTEND src/config_types.rs:
  - ADD DirectorySource with recursive, filters, watch options
  - ADD FileListSource with explicit file paths
  - ADD LoopConfig for auto-repeat settings
  - ADD WatchConfig for file watching behavior
  - ADD NetworkSimulationConfig per source
  - ADD ProfileConfig for named profiles
  - ADD ConfigImport for including files

Task 2:
CREATE src/config/schema.rs:
  - DEFINE JSON schema for configuration
  - IMPLEMENT schema validation
  - ADD schema generation from Rust types
  - INCLUDE documentation in schema
  - SUPPORT schema versioning

Task 3:
CREATE src/config/loader.rs:
  - IMPLEMENT multi-format loading (JSON, YAML, TOML)
  - ADD environment variable substitution
  - SUPPORT configuration includes/imports
  - IMPLEMENT configuration merging
  - ADD profile resolution

Task 4:
CREATE src/config/profiles.rs:
  - DEFINE built-in profiles (development, testing, production)
  - IMPLEMENT profile inheritance
  - ADD profile override mechanism
  - SUPPORT conditional profiles
  - INCLUDE profile validation

Task 5:
CREATE src/config/validator.rs:
  - IMPLEMENT comprehensive validation
  - ADD semantic validation beyond schema
  - CHECK source conflicts and dependencies
  - VALIDATE network configurations
  - PROVIDE detailed error messages

Task 6:
EXTEND src/config/watcher.rs:
  - MONITOR configuration includes
  - IMPLEMENT hot-reload with validation
  - ADD configuration diff detection
  - HANDLE partial reload for changes
  - EMIT reload events

Task 7:
CREATE config schema file:
  - WRITE source-videos-config.schema.json
  - DOCUMENT all configuration options
  - INCLUDE examples in schema
  - ADD validation rules
  - SUPPORT IDE auto-completion

Task 8:
CREATE src/config/defaults.rs:
  - DEFINE sensible defaults for all options
  - IMPLEMENT default profiles
  - ADD platform-specific defaults
  - INCLUDE fallback configurations
  - SUPPORT default overrides

Task 9:
ADD configuration examples:
  - CREATE examples/configs/basic.toml
  - ADD examples/configs/advanced.yaml
  - INCLUDE examples/configs/multi-source.json
  - ADD examples/configs/with-watching.toml
  - CREATE examples/configs/network-sim.yaml

Task 10:
UPDATE CLI configuration handling:
  - SUPPORT --config-profile flag
  - ADD --validate-config option
  - IMPLEMENT --generate-config command
  - ADD --config-schema output
  - SUPPORT multiple config files
```

### Out of Scope
- GUI configuration editor
- Remote configuration management
- Configuration versioning/history
- Database-backed configuration

## Success Criteria

- [ ] All new features configurable through config files
- [ ] Configuration validation catches common errors
- [ ] Profile system simplifies common scenarios
- [ ] Environment variables work in all string fields
- [ ] Configuration includes enable modular configs
- [ ] Hot-reload updates running configuration
- [ ] Schema provides IDE auto-completion
- [ ] Migration from old configs is seamless

## Dependencies

### Technical Dependencies
- serde with derive features
- serde_yaml, toml for additional formats
- jsonschema for validation
- regex for variable substitution

### Knowledge Dependencies
- JSON Schema specification
- YAML/TOML syntax and features
- Configuration best practices

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Complex configuration syntax | Medium | Medium | Provide good examples and defaults |
| Breaking changes | Low | High | Maintain backward compatibility |
| Validation performance | Low | Low | Cache compiled schemas |
| Circular includes | Low | Medium | Detect and prevent cycles |

## Architecture Decisions

### Decision: Configuration Format Support
**Options Considered:**
1. JSON only
2. TOML only
3. Support JSON, YAML, and TOML

**Decision:** Option 3 - Multi-format support

**Rationale:** Different users prefer different formats; flexibility is valuable

### Decision: Variable Substitution
**Options Considered:**
1. No substitution
2. Environment variables only
3. Full template engine

**Decision:** Option 2 - Environment variables with ${VAR} syntax

**Rationale:** Covers most use cases without complexity

### Decision: Profile System
**Options Considered:**
1. No profiles
2. Simple profile selection
3. Inheritance-based profiles

**Decision:** Option 3 - Inheritance-based profiles

**Rationale:** Enables powerful configuration reuse

## Validation Strategy

### Validation Commands
```bash
# Validate configuration
cargo run -- validate-config config.toml

# Test with profile
cargo run -- serve --config config.yaml --profile development

# Generate schema
cargo run -- generate-schema > schema.json

# Test environment substitution
VIDEO_DIR=/media/videos cargo run -- serve -c config.toml

# Test hot-reload
cargo run -- serve -c config.toml --watch-config
# Then modify config.toml
```

## Configuration Examples

### Basic Configuration (TOML)
```toml
[server]
address = "0.0.0.0"
port = 8554

[[sources]]
type = "directory"
path = "${VIDEO_DIR:-/videos}"
recursive = true
watch = true
auto_repeat = true

[sources.filters]
include = ["*.mp4", "*.mkv"]
exclude = ["*.tmp"]
```

### Advanced Configuration (YAML)
```yaml
profile: production
imports:
  - network-profiles.yaml
  - sources-common.yaml

server:
  address: 0.0.0.0
  port: 8554
  
sources:
  - type: directory
    path: /media/videos
    recursive: true
    watch:
      enabled: true
      debounce: 500ms
    network_simulation:
      profile: lossy_network
    auto_repeat:
      enabled: true
      count: infinite
```

## Future Considerations

- GraphQL API for configuration management
- Configuration discovery via DNS/mDNS
- Kubernetes ConfigMap integration
- Distributed configuration synchronization
- A/B testing with configuration variants

## References

- JSON Schema Specification: https://json-schema.org/
- Serde documentation: https://serde.rs/
- Configuration management best practices

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-01-25
- **Status**: Ready for Implementation
- **Confidence Level**: 8/10 - Complex but well-defined requirements