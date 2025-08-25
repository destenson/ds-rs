#!/usr/bin/env python3
"""
Helper functions for test orchestration
Common Python utilities for process management, test result parsing, and reporting
"""

import json
import logging
import os
import re
import signal
import subprocess
import sys
import time
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
import threading
import queue

logger = logging.getLogger(__name__)

# Process management
@dataclass
class ProcessInfo:
    """Information about a managed process"""
    name: str
    process: subprocess.Popen
    start_time: datetime
    output_queue: queue.Queue = field(default_factory=queue.Queue)
    error_queue: queue.Queue = field(default_factory=queue.Queue)
    output_thread: Optional[threading.Thread] = None
    error_thread: Optional[threading.Thread] = None
    
    @property
    def is_running(self) -> bool:
        return self.process.poll() is None
    
    @property
    def duration(self) -> timedelta:
        return datetime.now() - self.start_time
    
    def get_output(self, timeout: float = 0.1) -> List[str]:
        """Get accumulated output lines"""
        lines = []
        try:
            while True:
                line = self.output_queue.get(timeout=timeout)
                lines.append(line)
        except queue.Empty:
            pass
        return lines
    
    def get_errors(self, timeout: float = 0.1) -> List[str]:
        """Get accumulated error lines"""
        lines = []
        try:
            while True:
                line = self.error_queue.get(timeout=timeout)
                lines.append(line)
        except queue.Empty:
            pass
        return lines

class ProcessManager:
    """Manages background processes with proper cleanup"""
    
    def __init__(self):
        self.processes: Dict[str, ProcessInfo] = {}
        self._lock = threading.Lock()
        
    def start_process(
        self,
        name: str,
        command: List[str],
        cwd: Optional[str] = None,
        env: Optional[Dict[str, str]] = None,
        wait_for_pattern: Optional[str] = None,
        timeout: int = 30
    ) -> ProcessInfo:
        """Start a background process with output monitoring"""
        
        with self._lock:
            # Stop existing process with same name
            if name in self.processes:
                logger.warning(f"Process {name} already running, stopping it first")
                self.stop_process(name)
            
            # Prepare environment
            process_env = os.environ.copy()
            if env:
                process_env.update(env)
            
            # Start process
            logger.info(f"Starting {name}: {' '.join(command)}")
            process = subprocess.Popen(
                command,
                cwd=cwd,
                env=process_env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1
            )
            
            # Create process info
            info = ProcessInfo(
                name=name,
                process=process,
                start_time=datetime.now()
            )
            
            # Start output monitoring threads
            info.output_thread = threading.Thread(
                target=self._read_stream,
                args=(process.stdout, info.output_queue, f"{name}-stdout")
            )
            info.error_thread = threading.Thread(
                target=self._read_stream,
                args=(process.stderr, info.error_queue, f"{name}-stderr")
            )
            
            info.output_thread.daemon = True
            info.error_thread.daemon = True
            info.output_thread.start()
            info.error_thread.start()
            
            self.processes[name] = info
            
            # Wait for pattern if specified
            if wait_for_pattern:
                if not self._wait_for_pattern(info, wait_for_pattern, timeout):
                    self.stop_process(name)
                    raise TimeoutError(f"Timeout waiting for pattern '{wait_for_pattern}' in {name} output")
            
            logger.info(f"{name} started successfully (PID: {process.pid})")
            return info
    
    def stop_process(self, name: str, timeout: int = 5) -> Optional[Dict[str, Any]]:
        """Stop a process gracefully"""
        
        with self._lock:
            if name not in self.processes:
                return None
            
            info = self.processes[name]
            
            if info.is_running:
                logger.info(f"Stopping {name}...")
                
                # Try graceful termination first
                info.process.terminate()
                try:
                    info.process.wait(timeout=timeout)
                except subprocess.TimeoutExpired:
                    logger.warning(f"{name} didn't stop gracefully, killing...")
                    info.process.kill()
                    info.process.wait()
                
                logger.info(f"{name} stopped")
            
            # Collect final output
            output = info.get_output(timeout=0.5)
            errors = info.get_errors(timeout=0.5)
            
            # Remove from tracking
            del self.processes[name]
            
            return {
                "exit_code": info.process.returncode,
                "duration": str(info.duration),
                "output": output,
                "errors": errors
            }
    
    def stop_all(self):
        """Stop all managed processes"""
        for name in list(self.processes.keys()):
            self.stop_process(name)
    
    def _read_stream(self, stream, queue, name: str):
        """Read from a stream and put lines into a queue"""
        try:
            for line in stream:
                line = line.rstrip()
                queue.put(line)
                if logger.isEnabledFor(logging.DEBUG):
                    logger.debug(f"[{name}] {line}")
        except Exception as e:
            logger.error(f"Error reading from {name}: {e}")
        finally:
            stream.close()
    
    def _wait_for_pattern(self, info: ProcessInfo, pattern: str, timeout: int) -> bool:
        """Wait for a pattern to appear in process output"""
        
        compiled_pattern = re.compile(pattern)
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            if not info.is_running:
                return False
            
            # Check output for pattern
            output = info.get_output(timeout=0.1)
            for line in output:
                if compiled_pattern.search(line):
                    return True
            
            time.sleep(0.1)
        
        return False

# Test result parsing
@dataclass
class CargoTestResults:
    """Parsed cargo test results"""
    passed: int = 0
    failed: int = 0
    ignored: int = 0
    total: int = 0
    duration: Optional[str] = None
    failed_tests: List[str] = field(default_factory=list)
    
    @property
    def success(self) -> bool:
        return self.failed == 0
    
    @property
    def success_rate(self) -> float:
        if self.total == 0:
            return 0.0
        return (self.passed / self.total) * 100

def parse_cargo_test_output(output: str) -> CargoTestResults:
    """Parse cargo test output for results"""
    
    results = CargoTestResults()
    
    # Parse test result line
    result_match = re.search(
        r'test result:.*?(\d+) passed.*?(\d+) failed.*?(\d+) ignored.*?finished in ([\d.]+)s',
        output
    )
    
    if result_match:
        results.passed = int(result_match.group(1))
        results.failed = int(result_match.group(2))
        results.ignored = int(result_match.group(3))
        results.duration = f"{result_match.group(4)}s"
        results.total = results.passed + results.failed + results.ignored
    
    # Extract failed test names
    failed_pattern = r'test (.+?) \.\.\. FAILED'
    for match in re.finditer(failed_pattern, output):
        results.failed_tests.append(match.group(1))
    
    return results

# Report generation
class TestReporter:
    """Generates test reports in various formats"""
    
    def __init__(self, results: Dict[str, Any]):
        self.results = results
        self.timestamp = datetime.now()
    
    def generate_json(self, output_path: Path) -> Path:
        """Generate JSON report"""
        
        report = {
            "timestamp": self.timestamp.isoformat(),
            "summary": self._get_summary(),
            "results": self.results
        }
        
        with open(output_path, 'w') as f:
            json.dump(report, f, indent=2, default=str)
        
        return output_path
    
    def generate_junit(self, output_path: Path) -> Path:
        """Generate JUnit XML report"""
        
        from xml.etree.ElementTree import Element, SubElement, tostring
        from xml.dom import minidom
        
        # Create root element
        testsuites = Element('testsuites')
        testsuites.set('name', 'ds-rs Test Results')
        testsuites.set('timestamp', self.timestamp.isoformat())
        
        summary = self._get_summary()
        testsuites.set('tests', str(summary['total']))
        testsuites.set('failures', str(summary['failed']))
        testsuites.set('time', str(summary.get('duration', 0)))
        
        # Add test suites
        for suite_name, result in self.results.items():
            testsuite = SubElement(testsuites, 'testsuite')
            testsuite.set('name', suite_name)
            
            if isinstance(result, dict):
                if 'test_results' in result and result['test_results']:
                    tr = result['test_results']
                    testsuite.set('tests', str(tr.total))
                    testsuite.set('failures', str(tr.failed))
                    testsuite.set('skipped', str(tr.ignored))
                    testsuite.set('time', tr.duration or '0')
                    
                    # Add failed test cases
                    for failed_test in tr.failed_tests:
                        testcase = SubElement(testsuite, 'testcase')
                        testcase.set('name', failed_test)
                        testcase.set('classname', suite_name)
                        
                        failure = SubElement(testcase, 'failure')
                        failure.set('message', 'Test failed')
                        failure.text = 'Test execution failed'
        
        # Pretty print XML
        xml_str = minidom.parseString(tostring(testsuites)).toprettyxml(indent='  ')
        
        with open(output_path, 'w') as f:
            f.write(xml_str)
        
        return output_path
    
    def generate_html(self, output_path: Path) -> Path:
        """Generate HTML report"""
        
        summary = self._get_summary()
        
        html = f"""<!DOCTYPE html>
<html>
<head>
    <title>Test Execution Report - ds-rs</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        h1 {{ color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 30px 0; }}
        .summary-card {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px; }}
        .summary-card h3 {{ margin: 0 0 10px 0; font-size: 0.9em; opacity: 0.9; }}
        .summary-card .value {{ font-size: 2em; font-weight: bold; }}
        .passed {{ background: linear-gradient(135deg, #84fab0 0%, #8fd3f4 100%); }}
        .failed {{ background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%); }}
        .rate {{ background: linear-gradient(135deg, #fa709a 0%, #fee140 100%); }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th {{ background: #34495e; color: white; padding: 12px; text-align: left; }}
        td {{ padding: 10px; border-bottom: 1px solid #ecf0f1; }}
        tr:hover {{ background: #f8f9fa; }}
        .status-passed {{ color: #27ae60; font-weight: bold; }}
        .status-failed {{ color: #e74c3c; font-weight: bold; }}
        .timestamp {{ color: #7f8c8d; font-size: 0.9em; margin-top: 20px; }}
        .footer {{ text-align: center; margin-top: 40px; padding-top: 20px; border-top: 1px solid #ecf0f1; color: #7f8c8d; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Test Execution Report</h1>
        <div class="timestamp">Generated: {self.timestamp.strftime('%Y-%m-%d %H:%M:%S')}</div>
        
        <div class="summary">
            <div class="summary-card">
                <h3>Total Test Suites</h3>
                <div class="value">{summary['total']}</div>
            </div>
            <div class="summary-card passed">
                <h3>Passed</h3>
                <div class="value">{summary['passed']}</div>
            </div>
            <div class="summary-card failed">
                <h3>Failed</h3>
                <div class="value">{summary['failed']}</div>
            </div>
            <div class="summary-card rate">
                <h3>Success Rate</h3>
                <div class="value">{summary['success_rate']:.1f}%</div>
            </div>
        </div>
        
        <h2>Test Results</h2>
        <table>
            <tr>
                <th>Test Suite</th>
                <th>Status</th>
                <th>Tests</th>
                <th>Duration</th>
                <th>Details</th>
            </tr>
"""
        
        for suite_name, result in self.results.items():
            status = "PASSED" if result.get('success', False) else "FAILED"
            status_class = "status-passed" if result.get('success', False) else "status-failed"
            
            test_info = "N/A"
            duration = "N/A"
            details = ""
            
            if isinstance(result, dict) and 'test_results' in result and result['test_results']:
                tr = result['test_results']
                test_info = f"{tr.passed} passed, {tr.failed} failed, {tr.ignored} ignored"
                duration = tr.duration or "N/A"
                if tr.failed_tests:
                    details = f"Failed: {', '.join(tr.failed_tests[:3])}"
                    if len(tr.failed_tests) > 3:
                        details += f" and {len(tr.failed_tests) - 3} more"
            
            html += f"""
            <tr>
                <td><strong>{suite_name}</strong></td>
                <td class="{status_class}">{status}</td>
                <td>{test_info}</td>
                <td>{duration}</td>
                <td>{details}</td>
            </tr>
"""
        
        html += """
        </table>
        
        <div class="footer">
            <p>DeepStream Rust Port - Test Orchestration System</p>
        </div>
    </div>
</body>
</html>
"""
        
        with open(output_path, 'w') as f:
            f.write(html)
        
        return output_path
    
    def _get_summary(self) -> Dict[str, Any]:
        """Calculate summary statistics"""
        
        total = len(self.results)
        passed = sum(1 for r in self.results.values() if r.get('success', False))
        failed = total - passed
        success_rate = (passed / total * 100) if total > 0 else 0
        
        return {
            'total': total,
            'passed': passed,
            'failed': failed,
            'success_rate': success_rate
        }

# Environment validation
class EnvironmentValidator:
    """Validates the test environment"""
    
    def __init__(self):
        self.results = {
            'valid': True,
            'required': {},
            'optional': {},
            'warnings': []
        }
    
    def check_command(
        self,
        command: str,
        required: bool = True,
        min_version: Optional[str] = None
    ) -> bool:
        """Check if a command is available"""
        
        try:
            result = subprocess.run(
                [command, '--version'],
                capture_output=True,
                text=True,
                timeout=5
            )
            
            if result.returncode == 0:
                version = result.stdout.strip()
                
                check_result = {
                    'found': True,
                    'version': version
                }
                
                # Simple version check
                if min_version and min_version not in version:
                    self.results['warnings'].append(
                        f"{command} version may be too old. Required: {min_version}"
                    )
                
                if required:
                    self.results['required'][command] = check_result
                else:
                    self.results['optional'][command] = check_result
                
                return True
            else:
                return self._handle_not_found(command, required)
        except (subprocess.TimeoutExpired, FileNotFoundError):
            return self._handle_not_found(command, required)
    
    def check_gstreamer_plugins(self, plugins: List[str]) -> bool:
        """Check if GStreamer plugins are available"""
        
        if not self.results['optional'].get('gst-inspect-1.0', {}).get('found', False):
            self.results['warnings'].append("GStreamer not found, skipping plugin checks")
            return True
        
        all_found = True
        for plugin in plugins:
            try:
                result = subprocess.run(
                    ['gst-inspect-1.0', plugin],
                    capture_output=True,
                    timeout=5
                )
                
                if result.returncode != 0:
                    self.results['warnings'].append(f"GStreamer plugin not found: {plugin}")
                    all_found = False
            except Exception:
                self.results['warnings'].append(f"Failed to check GStreamer plugin: {plugin}")
                all_found = False
        
        return all_found
    
    def _handle_not_found(self, command: str, required: bool) -> bool:
        """Handle a command that wasn't found"""
        
        check_result = {'found': False}
        
        if required:
            self.results['required'][command] = check_result
            self.results['valid'] = False
            logger.error(f"Required command not found: {command}")
        else:
            self.results['optional'][command] = check_result
            self.results['warnings'].append(f"Optional command not found: {command}")
        
        return False
    
    def validate(self) -> Dict[str, Any]:
        """Run all validation checks"""
        
        # Check required commands
        self.check_command('cargo', required=True, min_version='1.70')
        self.check_command('git', required=True, min_version='2.0')
        
        # Check optional commands
        self.check_command('gst-inspect-1.0', required=False, min_version='1.16')
        self.check_command('python', required=False, min_version='3.8')
        
        # Check GStreamer plugins if available
        self.check_gstreamer_plugins([
            'videotestsrc',
            'x264enc',
            'rtph264pay',
            'rtph264depay',
            'avdec_h264'
        ])
        
        return self.results

# Retry logic
def retry_on_failure(
    func,
    max_attempts: int = 3,
    delay: float = 5.0,
    exponential_backoff: bool = True,
    jitter: bool = True
):
    """Retry a function on failure with configurable backoff"""
    
    import random
    
    for attempt in range(max_attempts):
        try:
            return func()
        except Exception as e:
            if attempt == max_attempts - 1:
                raise
            
            wait_time = delay
            if exponential_backoff:
                wait_time *= (2 ** attempt)
            if jitter:
                wait_time *= (0.5 + random.random())
            
            logger.warning(f"Attempt {attempt + 1} failed: {e}. Retrying in {wait_time:.1f}s...")
            time.sleep(wait_time)
    
    return None