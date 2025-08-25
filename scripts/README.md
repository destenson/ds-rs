# Test Orchestration Scripts

This directory contains test orchestration scripts for the ds-rs project, including comprehensive network inference testing under degraded conditions.

## Quick Start

### 1. Install Dependencies

```bash
# Install required Python packages
pip install -r scripts/requirements.txt

# Or use the setup script
python scripts/setup-test-environment.py
```

Required packages:
- `tomli` - TOML configuration file parsing
- `requests` - API communication with RTSP servers

### 2. Run Tests

```bash
# Run all tests
python scripts/test-orchestrator.py --scenario all

# Run specific network inference test
python scripts/test-orchestrator.py --scenario network-inference-basic

# Run with verbose output
python scripts/test-orchestrator.py --scenario network-inference-drone -v
```

## Network Inference Testing (PRP-44)

The network inference test orchestration validates system resilience under degraded network conditions.

### Available Network Scenarios

| Scenario | Description | Duration |
|----------|-------------|----------|
| `network-inference-basic` | Single stream with changing conditions | 5 min |
| `network-inference-multi` | 4 concurrent streams, different profiles | 10 min |
| `network-inference-recovery` | Tests automatic recovery from drops | 7 min |
| `network-inference-drone` | Simulates drone mission (urban→mountain) | 10 min |
| `network-inference-stress` | Extreme conditions (20% loss, 800ms latency) | 5 min |
| `network-inference-satellite` | High-latency satellite link | 7 min |
| `network-inference-benchmark` | Performance comparison across profiles | 15 min |

### Network Profiles

| Profile | Packet Loss | Latency | Bandwidth | Use Case |
|---------|------------|---------|-----------|----------|
| `perfect` | 0% | 0ms | Unlimited | Baseline |
| `3g` | 2% | 300ms | 384 kbps | Mobile 3G |
| `4g` | 0.5% | 50ms | 12 Mbps | Mobile 4G |
| `5g` | 0.1% | 10ms | 100 Mbps | Mobile 5G |
| `wifi` | 0.5% | 20ms | 54 Mbps | WiFi |
| `satellite` | 3% | 600ms | 1 Mbps | Satellite |
| `drone-urban` | 5% | 100ms | 5 Mbps | Drone in city |
| `drone-mountain` | 15% | 200ms | 1 Mbps | Drone in mountains |
| `poor` | 10% | 500ms | 50 kbps | Degraded network |

### Running Network Tests

```bash
# Basic network test
python scripts/test-orchestrator.py --scenario network-inference-basic

# Drone mission simulation
python scripts/test-orchestrator.py --scenario network-inference-drone

# Stress test with extreme conditions
python scripts/test-orchestrator.py --scenario network-inference-stress

# Run all network tests
python scripts/test-orchestrator.py --scenario network-inference --network-config scripts/config/network-inference-scenarios.toml
```

## Architecture

### Core Components

1. **test-orchestrator.py** - Main test runner
   - Manages test scenarios
   - Starts/stops RTSP servers
   - Handles network simulation
   - Collects results

2. **lib/network_controller.py** - Network simulation
   - `NetworkSimulationManager` - Controls RTSP servers with network conditions
   - `StreamConfig` - Per-stream network configuration
   - `NetworkCondition` - Network parameters (loss, latency, bandwidth)

3. **lib/inference_metrics.py** - Metrics tracking
   - `InferenceMonitor` - Real-time process monitoring
   - `MetricsValidator` - Validates against requirements
   - `MetricsReporter` - Generates test reports

4. **config/network-inference-scenarios.toml** - Test scenarios
   - Scenario definitions
   - Network profiles
   - Validation criteria

## Test Flow

1. **Setup Phase**
   - Start RTSP server with network simulation
   - Configure streams with initial conditions
   - Wait for server readiness

2. **Execution Phase**
   - Launch inference pipeline
   - Apply network condition changes
   - Monitor metrics in real-time

3. **Validation Phase**
   - Check detection quality
   - Verify recovery behavior
   - Validate graceful degradation

4. **Cleanup Phase**
   - Stop servers
   - Generate reports
   - Clean up resources

## Metrics Collected

- **Detection Metrics**
  - FPS (frames per second)
  - Detection count and rate
  - Average confidence
  - Processing latency

- **Network Metrics**
  - Packet loss rate
  - Latency
  - Recovery attempts
  - Recovery time

- **Stream Health**
  - Active/inactive status
  - Frames dropped
  - Error count
  - Buffer level

- **Quality Metrics**
  - Tracking continuity (0-1 score)
  - Graceful degradation
  - Complete failures

## Platform Support

### Windows (PowerShell)
```powershell
.\scripts\test-orchestrator.ps1 -Scenario network-inference-basic
```

### Linux/macOS (Bash)
```bash
./scripts/test-orchestrator.sh network-inference-basic
```

### Python (Cross-platform)
```bash
python scripts/test-orchestrator.py --scenario network-inference-basic
```

## CI/CD Integration

### GitHub Actions
```yaml
- name: Setup Python
  uses: actions/setup-python@v4
  with:
    python-version: '3.8'

- name: Install dependencies
  run: pip install -r scripts/requirements.txt

- name: Run network inference tests
  run: python scripts/test-orchestrator.py --scenario network-inference-basic
```

### Jenkins
```groovy
stage('Network Tests') {
    steps {
        sh 'pip install -r scripts/requirements.txt'
        sh 'python scripts/test-orchestrator.py --scenario network-inference'
    }
}
```

## Troubleshooting

### Common Issues

1. **"tomli package required"**
   - Install with: `pip install tomli`
   - Or run: `pip install -r scripts/requirements.txt`

2. **"Network simulation not available"**
   - Ensure network_controller.py is in scripts/lib/
   - Check that requests package is installed

3. **"RTSP server failed to start"**
   - Check port 8554 is not in use
   - Ensure source-videos crate is built
   - Verify GStreamer is installed

4. **"Failed to load network config"**
   - Check network-inference-scenarios.toml exists
   - Verify TOML syntax is correct

## Development

### Adding New Scenarios

1. Edit `config/network-inference-scenarios.toml`
2. Define scenario with setup, steps, and validation
3. Add network profiles as needed
4. Test with: `python scripts/test-orchestrator.py --scenario your-scenario`

### Custom Validation

Add validation functions to `lib/inference_metrics.py`:

```python
def validate_custom_metric(metrics, threshold):
    return metrics.custom_value > threshold
```

### Extending Network Conditions

Modify `lib/network_controller.py` to add new parameters:

```python
@dataclass
class NetworkCondition:
    # ... existing fields ...
    corruption_rate: Optional[float] = None  # New parameter
```

## License

Part of the ds-rs project. See main LICENSE file.