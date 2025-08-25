# PRP: Network Simulation Integration for Source-Videos Serving

## Executive Summary

Fully integrate the existing network simulation capabilities with the source serving infrastructure, enabling per-source network conditions, dynamic condition changes, and realistic network scenario testing. This allows comprehensive testing of client resilience and adaptive streaming behavior.

## Problem Statement

### Current State
- Network simulation module exists but not integrated with serving
- Cannot apply different conditions to different sources
- No dynamic network condition changes during streaming
- Limited testing of network failure scenarios
- Network simulation not exposed through CLI/config

### Desired State
- Per-source network simulation configuration
- Dynamic network condition changes during streaming
- Predefined network scenarios (flaky, congested, etc.)
- Time-based network condition scripts
- Integration with all serving modes (files, directories, patterns)
- Real-time network metrics and monitoring

### Business Value
Enables comprehensive testing of streaming resilience, validates client error recovery, supports network planning and optimization, and provides realistic testing environments without actual network issues.

## Requirements

### Functional Requirements

1. **Per-Source Simulation**: Different conditions for each source
2. **Dynamic Changes**: Modify conditions during streaming
3. **Scenario Playback**: Time-based condition sequences
4. **Profile Library**: Predefined realistic network profiles
5. **Metrics Collection**: Track actual vs simulated conditions
6. **Selective Application**: Choose which streams to affect
7. **Bandwidth Shaping**: Accurate rate limiting

### Non-Functional Requirements

1. **Accuracy**: Simulation closely matches real conditions
2. **Performance**: Minimal overhead when not simulating
3. **Granularity**: Millisecond-precision timing
4. **Scalability**: Handle many concurrent simulations
5. **Observability**: Clear metrics and logging

### Context and Research

Existing infrastructure in `src/network/`:
- NetworkSimulator trait and implementation
- NetworkProfile with standard profiles
- GStreamerNetworkSimulator for pipeline integration
- NetworkConditions structure

GStreamer provides elements for network simulation:
- queue with leaky property for packet loss
- identity element for latency injection
- queue2 for bandwidth limiting

### Documentation & References

```yaml
- file: crates/source-videos/src/network/mod.rs
  why: Existing network simulation framework

- file: crates/source-videos/src/network/profiles.rs
  why: Predefined network profiles to use

- file: crates/source-videos/src/network/simulator.rs
  why: Simulation implementation patterns

- url: https://gstreamer.freedesktop.org/documentation/coreelements/queue.html
  why: Queue element for packet simulation

- url: https://gstreamer.freedesktop.org/documentation/coreelements/identity.html
  why: Identity element for latency/jitter

- file: crates/ds-rs/tests/network_simulation_test.rs
  why: Reference test patterns for network simulation
```

### List of tasks to be completed

```yaml
Task 1:
EXTEND src/network/simulator.rs:
  - ADD per-source simulation tracking
  - IMPLEMENT condition interpolation for smooth changes
  - ADD metrics collection during simulation
  - CREATE condition history tracking
  - SUPPORT bandwidth shaping algorithms

Task 2:
CREATE src/network/scenarios.rs:
  - DEFINE Scenario trait for time-based conditions
  - IMPLEMENT common scenarios (congestion, flaky, degrading)
  - ADD scenario scripting language/format
  - CREATE scenario validation and testing
  - INCLUDE random variation within scenarios

Task 3:
INTEGRATE with src/source.rs:
  - ADD network_simulator field to VideoSource
  - IMPLEMENT simulation element injection in pipeline
  - HANDLE dynamic condition updates
  - CREATE metrics collection per source
  - SUPPORT simulation bypass for debugging

Task 4:
EXTEND src/rtsp/mod.rs:
  - INJECT simulation elements in RTSP pipeline
  - ADD per-client simulation options
  - IMPLEMENT RTCP feedback simulation
  - CREATE bandwidth estimation simulation
  - HANDLE connection drop simulation

Task 5:
CREATE src/network/metrics.rs:
  - TRACK simulated vs actual conditions
  - MEASURE simulation accuracy
  - COLLECT performance impact metrics
  - GENERATE simulation reports
  - EXPORT metrics for analysis

Task 6:
ADD configuration support:
  - EXTEND VideoSourceConfig with simulation options
  - ADD scenario references in config
  - SUPPORT inline condition definitions
  - CREATE profile inheritance
  - IMPLEMENT validation of network configs

Task 7:
ENHANCE CLI with network options:
  - ADD --network-scenario flag
  - IMPLEMENT --per-source-network option
  - ADD --network-script for scenario files
  - CREATE --network-metrics output
  - SUPPORT --network-variation for randomness

Task 8:
CREATE network testing utilities:
  - ADD network condition validator
  - IMPLEMENT baseline measurement tool
  - CREATE comparison reports
  - ADD regression testing support
  - GENERATE network test suites

Task 9:
ADD REPL network commands:
  - IMPLEMENT "network apply <source> <profile>"
  - ADD "network scenario start <file>"
  - CREATE "network metrics [source]"
  - ADD "network baseline" measurement
  - IMPLEMENT "network sweep" for testing ranges

Task 10:
CREATE examples and tests:
  - ADD examples/network_simulation_demo.rs
  - CREATE examples/network_scenarios/
  - ADD comprehensive integration tests
  - IMPLEMENT performance benchmarks
  - CREATE validation test suite
```

### Out of Scope
- Actual network device manipulation
- Kernel-level packet filtering
- Hardware network simulation
- Multi-host distributed simulation

## Success Criteria

- [ ] Each source can have independent network conditions
- [ ] Conditions can change dynamically during streaming
- [ ] Scenarios play back accurately over time
- [ ] Performance overhead is < 5% when active
- [ ] Metrics accurately reflect simulated conditions
- [ ] Integration works with all source types
- [ ] Configuration is intuitive and flexible
- [ ] Real-world scenarios are accurately simulated

## Dependencies

### Technical Dependencies
- GStreamer queue and identity elements
- tokio for async scenario execution
- Statistical libraries for variation

### Knowledge Dependencies
- Network behavior modeling
- GStreamer pipeline manipulation
- Statistical distribution for realistic variation

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Inaccurate simulation | Medium | High | Validate against real network traces |
| Performance overhead | Medium | Medium | Optimize critical paths, optional feature |
| Complex configuration | Medium | Low | Good defaults and examples |
| Platform differences | Low | Medium | Test on all platforms |

## Architecture Decisions

### Decision: Simulation Implementation
**Options Considered:**
1. External traffic shaping (tc, NetEm)
2. GStreamer pipeline elements
3. Application-level simulation

**Decision:** Option 2 - GStreamer elements

**Rationale:** Cross-platform, integrates naturally, good control

### Decision: Scenario Definition
**Options Considered:**
1. Code-based scenarios
2. JSON/YAML configuration
3. Domain-specific language

**Decision:** Option 2 - JSON/YAML configuration

**Rationale:** Declarative, shareable, versionable

### Decision: Metrics Collection
**Options Considered:**
1. Built-in metrics only
2. OpenTelemetry integration
3. Custom metrics with export

**Decision:** Option 3 - Custom with export

**Rationale:** Flexibility without heavy dependencies

## Validation Strategy

### Validation Commands
```bash
# Test per-source simulation
cargo run -- serve -d /videos \
  --network-source "source-1:lossy" \
  --network-source "source-2:congested"

# Run scenario
cargo run -- serve -d /videos \
  --network-scenario scenarios/degrading.yaml

# Test with metrics
cargo run -- serve -d /videos \
  --network-profile residential \
  --network-metrics \
  --metrics-export network-test.json

# REPL testing
cargo run -- interactive
> add source pattern smpte
> network apply source-1 lossy
> network scenario start rush-hour.yaml
> network metrics
```

## Network Scenario Examples

### Degrading Network Scenario (YAML)
```yaml
name: degrading_network
description: Network quality degrades over time
duration: 300s
events:
  - time: 0s
    conditions:
      packet_loss: 0
      latency_ms: 20
      bandwidth_kbps: 10000
  
  - time: 60s
    conditions:
      packet_loss: 1
      latency_ms: 50
      bandwidth_kbps: 5000
      
  - time: 180s
    conditions:
      packet_loss: 5
      latency_ms: 200
      bandwidth_kbps: 1000
      
  - time: 240s
    conditions:
      connection_dropped: true
      
  - time: 270s
    conditions:
      packet_loss: 2
      latency_ms: 100
      bandwidth_kbps: 3000
```

### Configuration Example
```toml
[[sources]]
type = "directory"
path = "/videos"

[sources.network_simulation]
profile = "residential"
variation = 0.1  # 10% random variation

[[sources]]
type = "pattern"
pattern = "smpte"

[sources.network_simulation]
scenario = "scenarios/rush-hour.yaml"
start_offset = "30s"
```

## Future Considerations

- Machine learning for realistic network modeling
- Integration with network emulation hardware
- Distributed simulation across multiple nodes
- Real-time adaptation based on client feedback
- Historical network data replay

## References

- GStreamer Queue element documentation
- Network simulation best practices
- tc/NetEm documentation for validation

---

## PRP Metadata

- **Author**: Claude
- **Created**: 2025-01-25
- **Status**: Ready for Implementation  
- **Confidence Level**: 9/10 - Existing framework ready for integration