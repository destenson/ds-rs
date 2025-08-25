#!/usr/bin/env python3
"""
Test Orchestrator for ds-rs project
Cross-platform test runner that manages test scenarios, RTSP servers, and test environments
Now with network simulation support for inference testing under degraded conditions
"""

import argparse
import json
import logging
import os
import platform
import socket
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Any
import signal
import atexit
import threading

# Add lib directory to path for helper imports
sys.path.insert(0, str(Path(__file__).parent / "lib"))

try:
    import tomli
except ImportError:
    print("Error: tomli package required. Install with: pip install tomli")
    sys.exit(1)

# Import network simulation support
try:
    from network_controller import NetworkSimulationManager, StreamConfig, NetworkCondition
    NETWORK_SIMULATION_AVAILABLE = True
except ImportError:
    NETWORK_SIMULATION_AVAILABLE = False
    print("Warning: Network simulation not available. Some tests may be skipped.")

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)

class ProcessManager:
    """Manages background processes with proper cleanup"""
    
    def __init__(self):
        self.processes = {}
        atexit.register(self.cleanup_all)
        
    def start_process(self, name: str, command: str, cwd: str = None, env: Dict = None) -> subprocess.Popen:
        """Start a background process"""
        if name in self.processes:
            logger.warning(f"Process {name} already running, stopping it first")
            self.stop_process(name)
            
        logger.info(f"Starting process {name}: {command}")
        
        # Merge environment variables
        process_env = os.environ.copy()
        if env:
            process_env.update(env)
            
        # Start the process
        process = subprocess.Popen(
            command,
            shell=True,
            cwd=cwd,
            env=process_env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        self.processes[name] = process
        return process
        
    def stop_process(self, name: str) -> bool:
        """Stop a background process"""
        if name not in self.processes:
            return False
            
        process = self.processes[name]
        logger.info(f"Stopping process {name}")
        
        # Try graceful shutdown first
        if platform.system() == "Windows":
            process.terminate()
        else:
            process.send_signal(signal.SIGTERM)
            
        # Wait for process to terminate
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            logger.warning(f"Process {name} didn't stop gracefully, forcing kill")
            process.kill()
            process.wait()
            
        del self.processes[name]
        return True
        
    def cleanup_all(self):
        """Clean up all running processes"""
        for name in list(self.processes.keys()):
            self.stop_process(name)

class TestOrchestrator:
    """Main test orchestration class with network simulation support"""
    
    def __init__(self, config_path: Path):
        self.config_path = config_path
        self.config = self._load_config()
        self.process_manager = ProcessManager()
        self.project_root = Path(__file__).parent.parent
        self.test_results = []
        self.network_manager = NetworkSimulationManager() if NETWORK_SIMULATION_AVAILABLE else None
        self.inference_metrics = {}
        
    def _load_config(self) -> Dict:
        """Load TOML configuration"""
        with open(self.config_path, 'rb') as f:
            return tomli.load(f)
            
    def _check_port(self, host: str, port: int, timeout: int = 5) -> bool:
        """Check if a TCP port is open"""
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(timeout)
        try:
            result = sock.connect_ex((host, port))
            sock.close()
            return result == 0
        except:
            return False
            
    def _wait_for_health_check(self, health_check: Dict) -> bool:
        """Wait for a health check to pass"""
        check_type = health_check.get('type', 'tcp')
        timeout = health_check.get('timeout', 10)
        
        if check_type == 'tcp':
            host = health_check.get('host', '127.0.0.1')
            port = health_check.get('port')
            
            logger.info(f"Waiting for TCP port {host}:{port} to be available...")
            start_time = time.time()
            
            while time.time() - start_time < timeout:
                if self._check_port(host, port):
                    logger.info(f"Port {host}:{port} is available")
                    return True
                time.sleep(1)
                
            logger.error(f"Timeout waiting for port {host}:{port}")
            return False
            
        return True
        
    def _handle_network_update(self, step: Dict) -> bool:
        """Handle network condition update step"""
        if not self.network_manager:
            logger.warning("Network simulation not available, skipping network update")
            return True
        
        stream_index = step.get('stream_index', 0)
        wait = step.get('wait', 0)
        
        # Build network condition
        condition = NetworkCondition(
            profile=step.get('profile'),
            packet_loss=step.get('packet_loss'),
            latency_ms=step.get('latency_ms') or step.get('latency'),
            bandwidth_kbps=step.get('bandwidth_kbps') or step.get('bandwidth'),
            jitter_ms=step.get('jitter_ms') or step.get('jitter')
        )
        
        logger.info(f"Updating network condition for stream {stream_index}")
        success = self.network_manager.update_network_condition('rtsp_server', stream_index, condition)
        
        if wait > 0:
            logger.info(f"Waiting {wait} seconds after network update")
            time.sleep(wait)
        
        return success
    
    def _run_network_sequence(self, step: Dict) -> bool:
        """Run a sequence of network condition changes over time"""
        if not self.network_manager:
            logger.warning("Network simulation not available, skipping network sequence")
            return True
        
        duration = step.get('duration', 30)
        conditions = step.get('conditions', [])
        stream_index = step.get('stream_index', 0)
        
        logger.info(f"Running network sequence for {duration} seconds")
        
        # Sort conditions by time
        conditions.sort(key=lambda x: x.get('time', 0))
        
        start_time = time.time()
        condition_index = 0
        
        while time.time() - start_time < duration:
            elapsed = time.time() - start_time
            
            # Check if we need to apply next condition
            if condition_index < len(conditions):
                next_condition = conditions[condition_index]
                if elapsed >= next_condition.get('time', 0):
                    # Apply this condition
                    network_cond = NetworkCondition(
                        profile=next_condition.get('profile'),
                        packet_loss=next_condition.get('packet_loss'),
                        latency_ms=next_condition.get('latency') or next_condition.get('latency_ms'),
                        bandwidth_kbps=next_condition.get('bandwidth') or next_condition.get('bandwidth_kbps')
                    )
                    
                    logger.info(f"Applying network condition at t={elapsed:.1f}s")
                    self.network_manager.update_network_condition('rtsp_server', stream_index, network_cond)
                    condition_index += 1
            
            time.sleep(1)
        
        return True
    
    def _validate_metrics(self, step: Dict) -> bool:
        """Validate inference metrics"""
        check = step.get('check')
        expected = step.get('expected')
        min_value = step.get('min_value')
        max_value = step.get('max_value')
        timeout = step.get('timeout', 30)
        
        logger.info(f"Validating metrics: {check}")
        
        # Simple example - in production this would check actual metrics
        # from the running inference process
        start_time = time.time()
        while time.time() - start_time < timeout:
            # Check metrics (placeholder - would read from actual system)
            if check == "recovery_attempts":
                # Simulate checking recovery attempts
                if self.inference_metrics.get('recovery_attempts', 0) >= min_value:
                    logger.info(f"Validation passed: {check} >= {min_value}")
                    return True
            elif check == "stream_active":
                # Check if stream is active
                if self.network_manager:
                    metrics = self.network_manager.get_metrics('rtsp_server')
                    if metrics and metrics.streams > 0:
                        logger.info(f"Validation passed: stream is active")
                        return True
            elif check == "tracking_continuity":
                # Check tracking continuity
                continuity = self.inference_metrics.get('tracking_continuity', 0)
                if continuity >= min_value:
                    logger.info(f"Validation passed: tracking continuity {continuity} >= {min_value}")
                    return True
            time.sleep(1)
        
        logger.error(f"Validation failed: {check} did not meet criteria")
        return False
    
    def _run_command(self, step: Dict, scenario_env: Dict = None) -> bool:
        """Run a single test command or special step type"""
        # Handle special step types
        step_type = step.get('type', 'command')
        
        if step_type == 'network_update':
            return self._handle_network_update(step)
        elif step_type == 'validate_metrics':
            return self._validate_metrics(step)
        elif step_type == 'network_sequence':
            # Handle dynamic network condition changes over time
            return self._run_network_sequence(step)
        
        # Regular command execution
        name = step.get('name', 'unnamed')
        command = step['command']
        cwd = step.get('cwd', '.')
        expected_exit_code = step.get('expected_exit_code', 0)
        retry_count = step.get('retry_count', 0)
        allow_failure = step.get('allow_failure', False)
        timeout = step.get('timeout', 300)
        background = step.get('background', False)
        
        # Resolve cwd relative to project root
        full_cwd = self.project_root / cwd
        
        # Merge environment variables
        env = os.environ.copy()
        if scenario_env:
            env.update(scenario_env)
        if 'env' in step:
            env.update(step['env'])
            
        # Apply defaults from config
        if 'defaults' in self.config:
            for key, value in self.config['defaults'].items():
                env_key = key.upper()
                if env_key not in env:
                    env[env_key] = str(value)
        
        logger.info(f"Running: {name}")
        logger.debug(f"Command: {command}")
        logger.debug(f"CWD: {full_cwd}")
        
        # Run with retries
        for attempt in range(retry_count + 1):
            if attempt > 0:
                logger.info(f"Retry attempt {attempt}/{retry_count}")
                
            try:
                result = subprocess.run(
                    command,
                    shell=True,
                    cwd=full_cwd,
                    env=env,
                    capture_output=True,
                    text=True,
                    timeout=timeout
                )
                
                if result.returncode == expected_exit_code:
                    logger.info(f" {name} completed successfully")
                    return True
                elif allow_failure:
                    logger.warning(f" {name} failed but marked as allow_failure")
                    return True
                else:
                    logger.error(f" {name} failed with exit code {result.returncode}")
                    if result.stdout:
                        logger.debug(f"stdout: {result.stdout}")
                    if result.stderr:
                        logger.debug(f"stderr: {result.stderr}")
                        
            except subprocess.TimeoutExpired:
                logger.error(f" {name} timed out after {timeout} seconds")
                
            except Exception as e:
                logger.error(f" {name} failed with error: {e}")
                
        return False
        
    def _setup_scenario(self, scenario: Dict) -> bool:
        """Setup a test scenario (start RTSP server with optional network simulation)"""
        setup = scenario.get('setup', {})
        
        # Start RTSP server if needed
        if 'rtsp_server' in setup and setup['rtsp_server'].get('enabled', False):
            server_config = setup['rtsp_server']
            
            # Check if network simulation is requested
            if self.network_manager and ('streams' in server_config or 'network_profile' in server_config):
                # Use network simulation manager
                logger.info("Starting RTSP server with network simulation")
                
                # Build stream configurations
                streams = []
                if 'streams' in server_config:
                    for stream_cfg in server_config['streams']:
                        network_cond = NetworkCondition(
                            profile=stream_cfg.get('network_profile'),
                            scenario=stream_cfg.get('network_scenario'),
                            packet_loss=stream_cfg.get('packet_loss'),
                            latency_ms=stream_cfg.get('latency_ms'),
                            bandwidth_kbps=stream_cfg.get('bandwidth_kbps'),
                            jitter_ms=stream_cfg.get('jitter_ms')
                        )
                        
                        stream = StreamConfig(
                            source_path=str(self.project_root / stream_cfg['source']),
                            mount_point=stream_cfg.get('mount_point', 'test'),
                            network_condition=network_cond,
                            auto_repeat=stream_cfg.get('auto_repeat', True)
                        )
                        streams.append(stream)
                else:
                    # Single stream with default configuration
                    network_cond = NetworkCondition(
                        profile=server_config.get('network_profile', 'perfect'),
                        scenario=server_config.get('network_scenario')
                    )
                    stream = StreamConfig(
                        source_path=str(self.project_root / "crates/ds-rs/tests/test_video.mp4"),
                        mount_point="test",
                        network_condition=network_cond,
                        auto_repeat=True
                    )
                    streams = [stream]
                
                # Start server with network simulation
                success = self.network_manager.start_server(
                    'rtsp_server',
                    streams,
                    port=server_config.get('port', 8554),
                    api_port=server_config.get('api_port'),
                    wait_for_ready=True,
                    timeout=server_config.get('timeout', 30)
                )
                
                if not success:
                    logger.error("Failed to start RTSP server with network simulation")
                    return False
                    
                logger.info(f"RTSP URLs: {self.network_manager.get_rtsp_urls('rtsp_server')}")
                
            else:
                # Traditional RTSP server start without network simulation
                self.process_manager.start_process(
                    'rtsp_server',
                    server_config['command'],
                    cwd=self.project_root / server_config['cwd']
                )
                
                # Wait for startup
                startup_delay = server_config.get('startup_delay', 5)
                logger.info(f"Waiting {startup_delay} seconds for RTSP server to start...")
                time.sleep(startup_delay)
                
                # Health check
                if 'health_check' in server_config:
                    if not self._wait_for_health_check(server_config['health_check']):
                        logger.error("RTSP server health check failed")
                        return False
                    
        # Create test files if needed
        if 'test_files' in setup and setup['test_files'].get('enabled', False):
            for file_config in setup['test_files'].get('files', []):
                pattern = file_config['pattern']
                duration = file_config['duration']
                output = file_config['output']
                
                command = f"cargo run -- generate --pattern {pattern} --duration {duration} --output {output}"
                cwd = self.project_root / "crates/source-videos"
                
                logger.info(f"Generating test file: {output}")
                result = subprocess.run(command, shell=True, cwd=cwd, capture_output=True, text=True)
                
                if result.returncode != 0:
                    logger.error(f"Failed to generate test file: {output}")
                    return False
                    
        return True
        
    def _cleanup_scenario(self, scenario: Dict):
        """Clean up after a test scenario"""
        cleanup = scenario.get('cleanup', {})
        
        # Stop RTSP server (network or traditional)
        if self.network_manager and 'rtsp_server' in self.network_manager.servers:
            logger.info("Stopping network simulation RTSP server")
            self.network_manager.stop_server('rtsp_server')
        elif cleanup.get('stop_rtsp_server', False):
            self.process_manager.stop_process('rtsp_server')
            
        # Remove test files
        if cleanup.get('remove_test_files', False):
            test_files = ['test_smpte.mp4', 'test_ball.mp4']  # Add more as needed
            for file in test_files:
                file_path = self.project_root / "crates/ds-rs" / file
                if file_path.exists():
                    logger.info(f"Removing test file: {file}")
                    file_path.unlink()
                    
    def run_scenario(self, scenario_name: str) -> bool:
        """Run a single test scenario"""
        if scenario_name not in self.config['scenarios']:
            logger.error(f"Scenario '{scenario_name}' not found in configuration")
            return False
            
        scenario = self.config['scenarios'][scenario_name]
        logger.info(f"\n{'='*60}")
        logger.info(f"Running scenario: {scenario_name}")
        logger.info(f"Description: {scenario.get('description', 'No description')}")
        logger.info(f"{'='*60}\n")
        
        # Check if this scenario includes other scenarios
        if 'include_scenarios' in scenario:
            success = True
            for included in scenario['include_scenarios']:
                if not self.run_scenario(included):
                    success = False
                    if not scenario.get('continue_on_failure', False):
                        return False
            return success
            
        # Setup
        if not self._setup_scenario(scenario):
            logger.error("Scenario setup failed")
            self._cleanup_scenario(scenario)
            return False
            
        # Get scenario environment variables
        scenario_env = scenario.get('env', {})
        
        # Run steps
        steps = scenario.get('steps', [])
        parallel = scenario.get('parallel', False)
        success = True
        
        if parallel:
            logger.warning("Parallel execution not yet implemented, running sequentially")
            
        for step in steps:
            if not self._run_command(step, scenario_env):
                success = False
                if not scenario.get('continue_on_failure', False):
                    break
                    
        # Cleanup
        self._cleanup_scenario(scenario)
        
        # Report results
        if success:
            logger.info(f"\n Scenario '{scenario_name}' completed successfully")
        else:
            logger.error(f"\n Scenario '{scenario_name}' failed")
            
        self.test_results.append({
            'scenario': scenario_name,
            'success': success
        })
        
        return success
        
    def list_scenarios(self):
        """List all available test scenarios"""
        print("\nAvailable test scenarios:")
        print("-" * 60)
        
        for name, scenario in self.config['scenarios'].items():
            description = scenario.get('description', 'No description')
            print(f"  {name:20} - {description}")
            
        print()
        
    def print_summary(self):
        """Print test execution summary"""
        print("\n" + "="*60)
        print("TEST EXECUTION SUMMARY")
        print("="*60)
        
        total = len(self.test_results)
        passed = sum(1 for r in self.test_results if r['success'])
        failed = total - passed
        
        for result in self.test_results:
            status = " PASSED" if result['success'] else " FAILED"
            print(f"  {result['scenario']:20} - {status}")
            
        print("-"*60)
        print(f"Total: {total}, Passed: {passed}, Failed: {failed}")
        
        if failed == 0:
            print("\n All tests passed!")
        else:
            print(f"\n {failed} test(s) failed")

def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description='Test Orchestrator for ds-rs project',
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument(
        '--scenario', '-s',
        help='Test scenario to run (default: all)',
        default='all'
    )
    
    parser.add_argument(
        '--config', '-c',
        help='Path to configuration file',
        default='scripts/config/test-scenarios.toml'
    )
    
    parser.add_argument(
        '--network-config',
        help='Path to network inference scenarios configuration file',
        default='scripts/config/network-inference-scenarios.toml'
    )
    
    parser.add_argument(
        '--list', '-l',
        action='store_true',
        help='List available test scenarios'
    )
    
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Enable verbose output'
    )
    
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Show what would be executed without running tests'
    )
    
    args = parser.parse_args()
    
    # Set logging level
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
        
    # Find config file
    config_path = Path(args.config)
    if not config_path.is_absolute():
        config_path = Path(__file__).parent.parent / args.config
        
    if not config_path.exists():
        logger.error(f"Configuration file not found: {config_path}")
        sys.exit(1)
        
    # Check if we're running network inference tests
    scenario = args.scenario
    if scenario.startswith('network-inference'):
        # Load network inference scenarios instead
        network_config_path = Path(args.network_config)
        if not network_config_path.is_absolute():
            network_config_path = Path(__file__).parent.parent / args.network_config
            
        if network_config_path.exists():
            logger.info(f"Loading network inference scenarios from {network_config_path}")
            config_path = network_config_path
        else:
            logger.warning(f"Network config not found: {network_config_path}, using default config")
    
    # Create orchestrator
    orchestrator = TestOrchestrator(config_path)
    
    # List scenarios if requested
    if args.list:
        orchestrator.list_scenarios()
        sys.exit(0)
        
    # Run scenario
    try:
        success = orchestrator.run_scenario(args.scenario)
        orchestrator.print_summary()
        sys.exit(0 if success else 1)
        
    except KeyboardInterrupt:
        logger.info("\nTest execution interrupted by user")
        orchestrator.process_manager.cleanup_all()
        sys.exit(130)
        
    except Exception as e:
        logger.error(f"Test execution failed: {e}")
        orchestrator.process_manager.cleanup_all()
        sys.exit(1)

if __name__ == '__main__':
    main()
