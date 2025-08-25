# PRP-43: Network Congestion Simulation Enhancement

## Summary
Enhance the network congestion simulation capabilities in source-videos to properly test and troubleshoot the main application's recovery mechanisms for intermittent streams. This involves replacing the current basic GStreamer element implementation with the proper `netsim` element and adding comprehensive testing scenarios.

## Motivation
The source-videos component has basic network simulation infrastructure but uses generic GStreamer elements (queue, identity, valve) instead of the purpose-built `netsim` element. The main application has fault-tolerant source controllers and recovery mechanisms that need comprehensive testing under various network conditions to ensure robust handling of intermittent streams.

## Current State Analysis

### Existing Infrastructure
- **Network Module Location**: `crates/source-videos/src/network/`
  - `mod.rs`: Core types (NetworkConditions, NetworkController trait)
  - `profiles.rs`: Predefined network profiles (3G, 4G, 5G, satellite, drone, etc.)
  - `simulator.rs`: Basic NetworkSimulator with threading support
  - `gstreamer.rs`: GStreamerNetworkSimulator using queue/identity/valve elements
  - `scenarios.rs`: Dynamic scenario support (needs investigation)

- **CLI Support**: Main.rs has comprehensive network flags:
  - `--network-profile`: Apply predefined profiles
  - `--packet-loss`, `--latency`, `--bandwidth`, `--jitter`: Custom conditions
  - `--network-drop`: Periodic connection drops
  - `--per-source-network`: Per-source conditions

- **Recovery Infrastructure** (in main app):
  - `crates/ds-rs/src/source/fault_tolerant_controller.rs`: Automatic recovery
  - `crates/ds-rs/src/source/recovery/`: Recovery managers and configs
  - `crates/ds-rs/src/source/circuit_breaker/`: Circuit breaker pattern

### Current Limitations
1. Uses generic GStreamer elements instead of specialized `netsim` element
2. No support for packet reordering simulation
3. No packet duplication simulation
4. Limited dynamic scenario capabilities
5. No comprehensive test suite for recovery validation

## Technical Requirements

### 1. Replace with GStreamer netsim Element
The `netsim` element (https://gstreamer.freedesktop.org/documentation/netsim/index.html) provides:
- `drop-probability`: Packet loss simulation (0.0-1.0)
- `duplicate-probability`: Packet duplication
- `delay-probability`: Delay injection
- `delay-distribution`: Distribution pattern for delays
- `allow-reordering`: Control packet reordering behavior
- `max-kbps`: Bandwidth limiting
- `max-bucket-size`: Token bucket for burst control

### 2. Dynamic Network Scenarios
Implement time-based scenario progression for realistic testing:
- Gradual degradation patterns
- Sudden connection drops and recovery
- Oscillating quality (good → bad → good)
- Burst packet loss patterns
- Bandwidth throttling with spikes

### 3. Integration Points
- Modify `GStreamerNetworkSimulator::create_elements()` to use netsim
- Update `apply_to_elements()` to set netsim properties
- Extend NetworkConditions to include duplication and reordering
- Add scenario player to main serve command

### 4. Testing Framework
Create comprehensive tests for the main app's recovery:
- Automated test scenarios with expected outcomes
- Metrics collection (recovery time, packet loss tolerance)
- Integration with existing fault_tolerant_controller tests

## Implementation Blueprint

### Phase 1: Core netsim Integration
Files to modify:
- `crates/source-videos/src/network/gstreamer.rs`: Replace element creation
- `crates/source-videos/src/network/mod.rs`: Extend NetworkConditions struct
- `crates/source-videos/src/network/profiles.rs`: Add duplication/reordering to profiles

Implementation approach:
1. Check for netsim element availability (may need gst-plugins-bad)
2. Create fallback to current implementation if unavailable
3. Map NetworkConditions to netsim properties
4. Handle property ranges (netsim uses 0.0-1.0 for probabilities)

### Phase 2: Dynamic Scenarios
Files to examine and enhance:
- `crates/source-videos/src/network/scenarios.rs`: Current implementation
- `crates/source-videos/examples/network_simulation_demo.rs`: Extend with scenarios

Scenario types to implement:
- `CongestionPattern`: Gradual bandwidth reduction
- `IntermittentFailure`: Periodic complete drops
- `JitterStorm`: High variance in latency
- `PacketBurst`: Clustered packet loss
- `RecoveryTest`: Specific patterns to test recovery timing

### Phase 3: API and CLI Extensions
Files to modify:
- `crates/source-videos/src/main.rs`: Add scenario flags
- `crates/source-videos/src/api/routes/network.rs`: Dynamic control endpoints
- `crates/source-videos/src/rtsp/mod.rs`: Scenario application to streams

New CLI flags:
- `--network-scenario`: Apply predefined scenario
- `--scenario-file`: Load scenario from JSON/YAML
- `--scenario-loop`: Repeat scenario
- `--metrics-export`: Export recovery metrics

### Phase 4: Testing Integration
Create new test files:
- `crates/source-videos/tests/network_recovery_test.rs`
- `crates/ds-rs/tests/intermittent_source_test.rs`

Test scenarios:
1. Stream starts → degrades → recovers
2. Multiple streams with different conditions
3. Rapid connection cycling
4. Bandwidth starvation and recovery
5. Packet reordering impact on decoders

## Implementation Tasks

1. **Verify netsim availability**
   - Check if gst-plugins-bad is installed
   - Create feature flag for netsim support
   - Implement runtime detection

2. **Refactor GStreamerNetworkSimulator**
   - Replace queue/identity/valve with netsim
   - Update property mapping
   - Maintain backward compatibility

3. **Extend NetworkConditions**
   - Add duplicate_probability field
   - Add reorder_probability field
   - Add delay_distribution enum

4. **Implement ScenarioPlayer**
   - Time-based condition changes
   - Scripted scenarios from files
   - Real-time scenario control via API

5. **Create Recovery Test Suite**
   - Automated test runner
   - Metrics collection
   - Report generation

6. **Update Documentation**
   - Network simulation guide
   - Recovery testing procedures
   - Scenario creation tutorial

## Validation Gates

```bash
# Build and format check
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings

# Run existing tests
cargo test --package source-videos --lib network

# Run new network recovery tests
cargo test --package source-videos --test network_recovery_test

# Integration test with main app
cargo test --package ds-rs --test intermittent_source_test

# Manual validation
cargo run --bin source-videos -- serve -d test_videos/ --network-scenario intermittent --metrics
```

## Success Criteria
1. netsim element properly integrated when available
2. All existing network profiles work with new implementation
3. Dynamic scenarios can simulate real-world conditions
4. Main app successfully recovers from all test scenarios
5. Metrics show recovery times and success rates
6. API allows real-time network condition changes

## References
- GStreamer netsim documentation: https://gstreamer.freedesktop.org/documentation/netsim/index.html
- Example in gstreamer-rs: `../gstreamer-rs/examples/src/bin/rtpfecclient.rs`
- Fault tolerant controller: `crates/ds-rs/src/source/fault_tolerant_controller.rs`
- Current network module: `crates/source-videos/src/network/`

## Risk Mitigation
- **netsim unavailable**: Maintain current implementation as fallback
- **Performance impact**: Add benchmarks to measure overhead
- **Complexity**: Start with simple scenarios, incrementally add features
- **Platform differences**: Test on Linux/Windows/Mac with different GStreamer versions

## Confidence Score: 8/10
High confidence due to:
- Clear existing infrastructure to build upon
- Well-defined GStreamer element to use
- Good separation of concerns in current code
- Clear testing requirements

Points deducted for:
- Potential platform/GStreamer version compatibility issues
- Complexity of dynamic scenario implementation