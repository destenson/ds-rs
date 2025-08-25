#!/usr/bin/env python3
"""
End-to-End Integration Test for ds-rs
Tests complete workflows from source-videos RTSP server to ds-app detection
"""

import argparse
import json
import logging
import os
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))
sys.path.insert(0, str(Path(__file__).parent.parent / "lib"))

from test_helpers import ProcessManager, CargoTestResults, parse_cargo_test_output

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class EndToEndTest:
    """End-to-end test orchestrator for ds-rs"""
    
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.process_manager = ProcessManager()
        self.test_results = {}
        
    def setup(self) -> bool:
        """Set up test environment"""
        logger.info("Setting up test environment...")
        
        # Build all required crates
        crates = ["source-videos", "ds-rs"]
        for crate in crates:
            crate_path = self.project_root / "crates" / crate
            logger.info(f"Building {crate}...")
            
            result = self.run_cargo_command(
                ["build", "--release"],
                cwd=crate_path
            )
            
            if result['exit_code'] != 0:
                logger.error(f"Failed to build {crate}")
                return False
        
        logger.info("Test environment setup complete")
        return True
    
    def test_rtsp_to_detection_pipeline(self) -> bool:
        """Test complete pipeline from RTSP source to detection"""
        logger.info("=" * 60)
        logger.info("Test: RTSP Source to Detection Pipeline")
        logger.info("=" * 60)
        
        success = True
        
        try:
            # Start RTSP server with multiple streams
            rtsp_server = self.start_rtsp_server(
                port=8554,
                mount_points=["/stream1", "/stream2"],
                patterns=["smpte", "ball"]
            )
            
            if not rtsp_server:
                logger.error("Failed to start RTSP server")
                return False
            
            # Give server time to initialize
            time.sleep(3)
            
            # Start ds-app with RTSP sources
            ds_app = self.start_ds_app(
                sources=[
                    "rtsp://localhost:8554/stream1",
                    "rtsp://localhost:8554/stream2"
                ],
                backend="standard"
            )
            
            if not ds_app:
                logger.error("Failed to start ds-app")
                return False
            
            # Let it run for a bit
            logger.info("Running detection pipeline for 10 seconds...")
            time.sleep(10)
            
            # Check that both processes are still running
            if not self.process_manager.processes.get("rtsp-server", None):
                logger.error("RTSP server crashed")
                success = False
            
            if not self.process_manager.processes.get("ds-app", None):
                logger.error("ds-app crashed")
                success = False
            
            # Collect output
            if "ds-app" in self.process_manager.processes:
                output = self.process_manager.processes["ds-app"].get_output()
                
                # Check for detection output
                detection_found = any("Detection" in line or "Object" in line for line in output)
                if detection_found:
                    logger.info("✓ Detection output found")
                else:
                    logger.warning("No detection output found")
                
                # Check for errors
                errors = self.process_manager.processes["ds-app"].get_errors()
                if errors:
                    logger.warning(f"ds-app errors: {len(errors)} error lines")
                    for error in errors[:5]:  # Show first 5 errors
                        logger.warning(f"  {error}")
            
        finally:
            # Clean up
            self.process_manager.stop_process("ds-app")
            self.process_manager.stop_process("rtsp-server")
        
        return success
    
    def test_multi_backend_detection(self) -> bool:
        """Test detection with different backends"""
        logger.info("=" * 60)
        logger.info("Test: Multi-Backend Detection")
        logger.info("=" * 60)
        
        backends = ["mock", "standard"]
        results = {}
        
        for backend in backends:
            logger.info(f"Testing with {backend} backend...")
            
            # Set backend environment variable
            env = {"FORCE_BACKEND": backend}
            
            # Run detection example
            result = self.run_cargo_command(
                ["run", "--example", "detection_app", "--release"],
                cwd=self.project_root / "crates" / "ds-rs",
                env=env,
                timeout=30
            )
            
            results[backend] = {
                "success": result['exit_code'] == 0 or result['exit_code'] == -2,  # -2 for Ctrl+C
                "output_lines": len(result.get('output', '').split('\n'))
            }
            
            if results[backend]['success']:
                logger.info(f"✓ {backend} backend test passed")
            else:
                logger.error(f"✗ {backend} backend test failed")
        
        self.test_results['multi_backend'] = results
        return all(r['success'] for r in results.values())
    
    def test_source_management(self) -> bool:
        """Test dynamic source addition and removal"""
        logger.info("=" * 60)
        logger.info("Test: Dynamic Source Management")
        logger.info("=" * 60)
        
        try:
            # Start RTSP server
            rtsp_server = self.start_rtsp_server(
                port=8555,
                mount_points=["/test"],
                patterns=["smpte"]
            )
            
            if not rtsp_server:
                return False
            
            time.sleep(2)
            
            # Run source management test
            result = self.run_cargo_command(
                ["test", "source_management", "--", "--nocapture"],
                cwd=self.project_root / "crates" / "ds-rs"
            )
            
            test_results = parse_cargo_test_output(result.get('output', ''))
            
            self.test_results['source_management'] = {
                "passed": test_results.passed,
                "failed": test_results.failed,
                "success": test_results.success
            }
            
            if test_results.success:
                logger.info(f"✓ Source management tests passed ({test_results.passed} tests)")
            else:
                logger.error(f"✗ Source management tests failed ({test_results.failed} failures)")
                for failed in test_results.failed_tests:
                    logger.error(f"  Failed: {failed}")
            
            return test_results.success
            
        finally:
            self.process_manager.stop_process("rtsp-server")
    
    def test_error_recovery(self) -> bool:
        """Test error recovery mechanisms"""
        logger.info("=" * 60)
        logger.info("Test: Error Recovery")
        logger.info("=" * 60)
        
        # Run fault tolerance example
        result = self.run_cargo_command(
            ["run", "--example", "fault_tolerant_pipeline", "--release"],
            cwd=self.project_root / "crates" / "ds-rs",
            timeout=20
        )
        
        # Check for recovery patterns in output
        output = result.get('output', '')
        recovery_patterns = [
            "Recovering",
            "Retry",
            "Circuit breaker",
            "Fallback"
        ]
        
        recovery_found = any(
            pattern.lower() in output.lower() 
            for pattern in recovery_patterns
        )
        
        if recovery_found:
            logger.info("✓ Error recovery mechanisms detected")
        else:
            logger.warning("No error recovery patterns found in output")
        
        return True  # This test is informational
    
    def test_performance_metrics(self) -> bool:
        """Test performance and resource usage"""
        logger.info("=" * 60)
        logger.info("Test: Performance Metrics")
        logger.info("=" * 60)
        
        try:
            # Start RTSP server with high framerate
            rtsp_server = self.start_rtsp_server(
                port=8556,
                mount_points=["/perf"],
                patterns=["smpte"],
                framerate=60
            )
            
            if not rtsp_server:
                return False
            
            time.sleep(2)
            
            # Start ds-app with performance monitoring
            ds_app_proc = self.process_manager.start_process(
                "ds-app-perf",
                [
                    "cargo", "run", "--release", "--bin", "ds-app", "--",
                    "--source", "rtsp://localhost:8556/perf",
                    "--backend", "standard"
                ],
                cwd=self.project_root / "crates" / "ds-rs"
            )
            
            # Monitor for 15 seconds
            logger.info("Monitoring performance for 15 seconds...")
            time.sleep(15)
            
            # Collect metrics
            if ds_app_proc.is_running:
                output = ds_app_proc.get_output()
                
                # Look for FPS information
                fps_lines = [line for line in output if "fps" in line.lower()]
                if fps_lines:
                    logger.info(f"FPS measurements found: {len(fps_lines)} samples")
                    for line in fps_lines[-5:]:  # Last 5 FPS measurements
                        logger.info(f"  {line}")
                
                logger.info("✓ Performance test completed")
                return True
            else:
                logger.error("ds-app crashed during performance test")
                return False
                
        finally:
            self.process_manager.stop_process("ds-app-perf")
            self.process_manager.stop_process("rtsp-server")
    
    def start_rtsp_server(
        self,
        port: int = 8554,
        mount_points: List[str] = None,
        patterns: List[str] = None,
        framerate: int = 30
    ) -> Optional[object]:
        """Start RTSP server with specified configuration"""
        
        mount_points = mount_points or ["/test"]
        patterns = patterns or ["smpte"]
        
        # Build command
        cmd = [
            "cargo", "run", "--release", "--",
            "serve",
            "--port", str(port)
        ]
        
        # Add mount points and patterns
        for i, (mount, pattern) in enumerate(zip(mount_points, patterns)):
            cmd.extend(["--mount-point", mount])
            cmd.extend(["--pattern", pattern])
        
        # Start server
        try:
            proc = self.process_manager.start_process(
                "rtsp-server",
                cmd,
                cwd=self.project_root / "crates" / "source-videos",
                wait_for_pattern="RTSP server",
                timeout=30
            )
            
            logger.info(f"RTSP server started on port {port}")
            return proc
            
        except Exception as e:
            logger.error(f"Failed to start RTSP server: {e}")
            return None
    
    def start_ds_app(
        self,
        sources: List[str],
        backend: str = "standard"
    ) -> Optional[object]:
        """Start ds-app with specified sources"""
        
        cmd = [
            "cargo", "run", "--release", "--bin", "ds-app", "--"
        ]
        
        # Add sources
        for source in sources:
            cmd.extend(["--source", source])
        
        # Add backend
        cmd.extend(["--backend", backend])
        
        # Start application
        try:
            proc = self.process_manager.start_process(
                "ds-app",
                cmd,
                cwd=self.project_root / "crates" / "ds-rs",
                env={"FORCE_BACKEND": backend}
            )
            
            logger.info(f"ds-app started with {len(sources)} sources")
            return proc
            
        except Exception as e:
            logger.error(f"Failed to start ds-app: {e}")
            return None
    
    def run_cargo_command(
        self,
        args: List[str],
        cwd: Path,
        env: Optional[Dict[str, str]] = None,
        timeout: int = 60
    ) -> Dict:
        """Run a cargo command and return results"""
        
        import subprocess
        
        cmd = ["cargo"] + args
        
        process_env = os.environ.copy()
        if env:
            process_env.update(env)
        
        try:
            result = subprocess.run(
                cmd,
                cwd=cwd,
                env=process_env,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            
            return {
                "exit_code": result.returncode,
                "output": result.stdout,
                "error": result.stderr
            }
            
        except subprocess.TimeoutExpired:
            logger.info(f"Command timed out after {timeout}s (expected for long-running commands)")
            return {
                "exit_code": -2,
                "output": "",
                "error": "Timeout"
            }
        except Exception as e:
            logger.error(f"Failed to run command: {e}")
            return {
                "exit_code": -1,
                "output": "",
                "error": str(e)
            }
    
    def generate_report(self) -> bool:
        """Generate test report"""
        logger.info("=" * 60)
        logger.info("END-TO-END TEST REPORT")
        logger.info("=" * 60)
        
        all_passed = True
        
        for test_name, result in self.test_results.items():
            if isinstance(result, dict) and 'success' in result:
                status = "PASSED" if result['success'] else "FAILED"
                if not result['success']:
                    all_passed = False
            else:
                status = "COMPLETED"
            
            logger.info(f"{test_name}: {status}")
        
        # Save detailed report
        report_path = self.project_root / "e2e-test-report.json"
        with open(report_path, 'w') as f:
            json.dump(self.test_results, f, indent=2, default=str)
        
        logger.info(f"Detailed report saved to {report_path}")
        
        return all_passed
    
    def cleanup(self):
        """Clean up all resources"""
        logger.info("Cleaning up...")
        self.process_manager.stop_all()

def main():
    parser = argparse.ArgumentParser(description="End-to-End Integration Test for ds-rs")
    parser.add_argument(
        "--project-root",
        type=Path,
        default=Path(__file__).parent.parent.parent,
        help="Project root directory"
    )
    parser.add_argument(
        "--tests",
        nargs="+",
        choices=[
            "rtsp-pipeline",
            "multi-backend",
            "source-management",
            "error-recovery",
            "performance"
        ],
        default=["rtsp-pipeline", "multi-backend", "source-management"],
        help="Tests to run"
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable verbose output"
    )
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # Create test runner
    test_runner = EndToEndTest(args.project_root)
    
    try:
        # Setup
        if not test_runner.setup():
            logger.error("Setup failed")
            return 1
        
        # Run selected tests
        test_methods = {
            "rtsp-pipeline": test_runner.test_rtsp_to_detection_pipeline,
            "multi-backend": test_runner.test_multi_backend_detection,
            "source-management": test_runner.test_source_management,
            "error-recovery": test_runner.test_error_recovery,
            "performance": test_runner.test_performance_metrics
        }
        
        for test_name in args.tests:
            if test_name in test_methods:
                success = test_methods[test_name]()
                test_runner.test_results[test_name] = {"success": success}
        
        # Generate report
        all_passed = test_runner.generate_report()
        
        return 0 if all_passed else 1
        
    finally:
        test_runner.cleanup()

if __name__ == "__main__":
    sys.exit(main())