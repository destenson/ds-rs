# PRP-44: Network Inference Test Orchestration

## Summary
Expand the existing test orchestration scripts to launch streams under various network conditions and perform inference on them in a robust manner. This combines the network simulation capabilities from PRP-43 with the inference pipeline to create comprehensive tests that validate system resilience under degraded network conditions.

## Motivation
Real-world deployments often involve inference on video streams transmitted over unreliable networks (satellite links, drone communications, mobile networks). The system must handle packet loss, latency, bandwidth constraints, and intermittent connections while maintaining inference accuracy and recovering gracefully from failures. Current tests run in ideal conditions and don't validate this critical capability.

## Current State Analysis

### Existing Infrastructure
1. **Test Orchestration (PRP-09)**: `scripts/test-orchestrator.py`
   - ProcessManager for background processes
   - TOML-based scenario configuration
   - Health checks and retry logic
   - Platform-specific support (Windows/Linux/macOS)

2. **Network Simulation (PRP-43)**: `crates/source-videos/src/network/`
   - GStreamer netsim element integration
   - Dynamic scenarios (degrading, flaky, drone profiles)
   - Time-based condition changes
   - Packet loss, duplication, reordering, bandwidth throttling

3. **Inference Pipeline**: `crates/ds-rs/`
   - CPU-based YOLO detection (V5/V8)
   - Multi-stream support with resource management
   - Metrics collection (FPS, detections, latency)
   - Fault-tolerant source controller with recovery

4. **Robustness Features**:
   - Circuit breaker patterns
   - Exponential backoff recovery
   - Health monitoring
   - Stream isolation

### Current Limitations
- Test orchestrator doesn't integrate network simulation
- No inference testing under degraded network conditions
- Missing metrics for inference quality under network stress
- No automated recovery validation
- Limited concurrent stream testing with varied conditions

## Technical Requirements

### 1. Test Orchestrator Network Integration
Extend `test-orchestrator.py` to:
- Launch source-videos servers with network scenarios
- Configure per-stream network conditions
- Support dynamic scenario progression during tests
- Collect network-related metrics

### 2. Network-Aware Test Scenarios
New TOML scenario definitions:
- `network-inference-basic`: Single stream with various conditions
- `network-inference-multi`: Multiple concurrent streams with different profiles
- `network-inference-recovery`: Test recovery from connection drops
- `network-inference-drone`: Simulate drone mission with changing conditions
- `network-inference-stress`: High packet loss and latency stress test

### 3. Inference Quality Metrics
Track and report:
- Detection accuracy under network stress
- Frame drop rate vs network conditions
- Inference latency distribution
- Recovery time after connection loss
- False positive/negative rates during degradation

### 4. Robustness Validation
Test scenarios must verify:
- Graceful degradation (not catastrophic failure)
- Automatic recovery when conditions improve
- Resource management under stress
- No memory leaks during reconnections
- Proper error propagation and logging

## Implementation Blueprint

### Phase 1: Orchestrator Enhancement
Files to modify:
- `scripts/test-orchestrator.py`: Add network simulation support
- `scripts/lib/network_controller.py`: New module for network control
- `scripts/config/test-scenarios.toml`: Add network test scenarios

Key additions:
1. NetworkSimulationManager class to control source-videos
2. Support for --network-profile and --network-scenario flags
3. Per-stream network condition configuration
4. Real-time network condition updates via API

### Phase 2: Test Scenario Implementation
New scenario files:
- `scripts/config/network-inference-scenarios.toml`
- `scripts/config/network-profiles.toml`

Scenario structure for network tests:
- Setup: Launch source-videos with specific network conditions
- Execution: Run inference pipeline
- Validation: Check metrics against thresholds
- Cleanup: Stop servers and collect logs

### Phase 3: Metrics Collection System
Files to create:
- `scripts/lib/metrics_collector.py`: Unified metrics collection
- `scripts/lib/inference_validator.py`: Validate inference quality

Metrics to collect:
- Network stats (packet loss, latency, bandwidth utilization)
- Inference stats (FPS, detection count, confidence scores)
- System stats (CPU, memory, GPU if available)
- Recovery stats (reconnection time, frames lost)

### Phase 4: Reporting and Analysis
Files to create:
- `scripts/lib/report_generator.py`: Generate test reports
- `scripts/templates/network_test_report.html`: HTML report template

Report contents:
- Test summary with pass/fail status
- Network condition timeline
- Inference performance graphs
- Recovery event log
- Comparison against baseline

## Implementation Tasks

### Task 1: Network Controller Module
Create `scripts/lib/network_controller.py`:
- Class to manage source-videos server lifecycle
- Methods to apply network profiles and scenarios
- API client for dynamic condition updates
- Integration with ProcessManager

### Task 2: Enhanced Test Scenarios
Update scenario configuration:
- Add network simulation setup blocks
- Define network condition progressions
- Set inference quality thresholds
- Configure recovery expectations

### Task 3: Metrics Integration
Implement metrics collection:
- Parse inference pipeline output for metrics
- Query source-videos API for network stats
- Correlate events across systems
- Store in structured format (JSON/CSV)

### Task 4: Validation Framework
Create validation system:
- Define acceptable performance envelopes
- Implement statistical analysis of results
- Compare against baseline performance
- Flag anomalies and regressions

### Task 5: CI/CD Integration
Add to continuous integration:
- Nightly network stress tests
- Pre-release validation suite
- Performance regression detection
- Automated issue creation for failures

## Test Scenarios Detail

### Scenario 1: Basic Network Inference
- Launch single RTSP stream with SMPTE pattern
- Apply progressive network degradation over 5 minutes
- Run YOLO inference on stream
- Validate: Detection count remains stable (±10%)

### Scenario 2: Multi-Stream Mixed Conditions
- Launch 4 streams with different network profiles:
  - Stream 1: Perfect conditions (baseline)
  - Stream 2: Mobile 3G profile
  - Stream 3: Satellite with intermittent drops
  - Stream 4: Noisy radio with high jitter
- Run inference on all streams concurrently
- Validate: Each stream maintains minimum FPS threshold

### Scenario 3: Recovery Testing
- Launch stream with good conditions
- After 30 seconds, simulate connection drop for 10 seconds
- Restore connection with degraded conditions
- Gradually improve to original quality
- Validate: System recovers within 5 seconds, no memory leaks

### Scenario 4: Drone Mission Simulation
- Use drone-urban scenario (building interference)
- Simulate 5-minute urban flight pattern
- Track inference quality throughout mission
- Validate: Critical detections not missed during degradation

### Scenario 5: Stress Test
- Apply extreme conditions:
  - 30% packet loss
  - 1000ms latency
  - 500kbps bandwidth
- Run for 10 minutes
- Validate: System remains stable, no crashes

## Validation Gates

```bash
# Syntax and style checks
python -m black scripts/
python -m pylint scripts/
python -m mypy scripts/

# Unit tests for new modules
python -m pytest scripts/tests/test_network_controller.py
python -m pytest scripts/tests/test_metrics_collector.py

# Integration test with basic scenario
python scripts/test-orchestrator.py --scenario network-inference-basic

# Full test suite
python scripts/test-orchestrator.py --scenario network-inference-all

# Generate report
python scripts/generate_report.py --input test_results.json --output report.html
```

## Success Criteria
1. All network inference scenarios pass validation thresholds
2. System recovers from connection drops within 5 seconds
3. No memory leaks during 1-hour stress test
4. Inference accuracy degrades gracefully (not catastrophically)
5. Clear correlation between network conditions and performance
6. Comprehensive test reports with actionable insights

## File Structure
```
scripts/
├── test-orchestrator.py         # Main orchestrator (modify)
├── config/
│   ├── test-scenarios.toml     # Existing scenarios (modify)
│   ├── network-inference-scenarios.toml  # New network scenarios
│   └── network-profiles.toml   # Network profile definitions
├── lib/
│   ├── network_controller.py   # Network simulation control (new)
│   ├── metrics_collector.py    # Metrics collection (new)
│   ├── inference_validator.py  # Inference validation (new)
│   └── report_generator.py     # Report generation (new)
├── tests/
│   ├── test_network_controller.py  # Unit tests (new)
│   └── test_metrics_collector.py   # Unit tests (new)
└── templates/
    └── network_test_report.html    # Report template (new)
```

## Dependencies
- Existing: Python 3.8+, tomli, subprocess, socket
- New additions:
  - requests: For API communication with source-videos
  - pandas: For metrics analysis
  - matplotlib/plotly: For report graphs
  - jinja2: For HTML report generation

## Risk Mitigation
- **Network simulation unavailable**: Fallback to basic delay injection
- **High resource usage**: Implement resource limits and throttling
- **Flaky tests**: Add retry logic with exponential backoff
- **Platform differences**: Test on all target platforms in CI

## References
- Test Orchestrator: `scripts/test-orchestrator.py`
- Network Simulation: `crates/source-videos/src/network/`
- Inference Pipeline: `crates/ds-rs/src/backend/cpu_vision/`
- Multi-stream: `crates/ds-rs/src/multistream/`
- Fault Tolerance: `crates/ds-rs/src/source/fault_tolerant_controller.rs`

## Future Enhancements
- Machine learning-based anomaly detection in test results
- Automatic network condition tuning to find breaking points
- Integration with cloud testing infrastructure
- Real device testing (actual drones, satellite links)
- Comparison with competitor solutions

## Confidence Score: 9/10
High confidence due to:
- Clear existing infrastructure to build upon
- Well-defined network simulation capabilities (PRP-43)
- Established test orchestration framework (PRP-09)
- Strong fault tolerance mechanisms already in place

Point deducted for:
- Complexity of correlating metrics across systems