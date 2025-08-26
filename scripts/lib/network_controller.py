#!/usr/bin/env python3
"""
Network Simulation Controller for Test Orchestration
Manages source-videos servers with network simulation for inference testing
"""

import json
import logging
import os
import socket
import subprocess
import time
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
import threading
import requests

logger = logging.getLogger(__name__)

# Network profiles matching source-videos CLI
NETWORK_PROFILES = {
    "perfect": {"packet_loss": 0, "latency": 0, "bandwidth": 0},
    "3g": {"packet_loss": 2, "latency": 300, "bandwidth": 384},
    "4g": {"packet_loss": 0.5, "latency": 50, "bandwidth": 12000},
    "5g": {"packet_loss": 0.1, "latency": 10, "bandwidth": 100000},
    "wifi": {"packet_loss": 0.5, "latency": 20, "bandwidth": 54000},
    "public": {"packet_loss": 1, "latency": 40, "bandwidth": 10000},
    "satellite": {"packet_loss": 3, "latency": 600, "bandwidth": 1000},
    "broadband": {"packet_loss": 0.01, "latency": 5, "bandwidth": 100000},
    "poor": {"packet_loss": 10, "latency": 500, "bandwidth": 50},
    # Drone-specific profiles
    "drone-urban": {"packet_loss": 5, "latency": 100, "bandwidth": 5000},
    "drone-mountain": {"packet_loss": 15, "latency": 200, "bandwidth": 1000},
    "noisy-radio": {"packet_loss": 8, "latency": 150, "bandwidth": 2000},
}

# Dynamic network scenarios
NETWORK_SCENARIOS = {
    "degrading": "Network conditions gradually worsen over time",
    "flaky": "Intermittent connection drops",
    "congestion": "Bandwidth drops during peak times",
    "intermittent-satellite": "Periodic satellite link disconnections",
    "noisy-radio": "UHF/VHF radio with interference",
    "drone-urban": "Drone flying through urban environment",
    "drone-mountain": "Drone in mountainous terrain",
}

@dataclass
class NetworkCondition:
    """Network condition configuration"""
    profile: Optional[str] = None
    scenario: Optional[str] = None
    packet_loss: Optional[float] = None
    latency_ms: Optional[int] = None
    bandwidth_kbps: Optional[int] = None
    jitter_ms: Optional[int] = None
    packet_duplication: Optional[float] = None
    packet_reorder: Optional[float] = None
    
    def to_cli_args(self) -> List[str]:
        """Convert to CLI arguments for source-videos"""
        args = []
        if self.profile:
            args.extend(["--network-profile", self.profile])
        if self.scenario:
            args.extend(["--network-scenario", self.scenario])
        if self.packet_loss is not None:
            args.extend(["--packet-loss", str(self.packet_loss)])
        if self.latency_ms is not None:
            args.extend(["--latency", str(self.latency_ms)])
        if self.bandwidth_kbps is not None:
            args.extend(["--bandwidth", str(self.bandwidth_kbps)])
        if self.jitter_ms is not None:
            args.extend(["--jitter", str(self.jitter_ms)])
        if self.packet_duplication is not None:
            args.extend(["--packet-duplication", str(self.packet_duplication)])
        if self.packet_reorder is not None:
            args.extend(["--packet-reorder", str(self.packet_reorder)])
        return args

@dataclass
class StreamConfig:
    """Configuration for a single video stream"""
    source_path: str
    mount_point: str
    network_condition: NetworkCondition = field(default_factory=NetworkCondition)
    auto_repeat: bool = True
    
@dataclass
class ServerMetrics:
    """Metrics from a running RTSP server"""
    streams: int = 0
    clients: int = 0
    bytes_sent: int = 0
    errors: int = 0
    uptime_seconds: float = 0
    
class NetworkSimulationManager:
    """Manages source-videos servers with network simulation"""
    
    def __init__(self, base_port: int = 8554, force_rebuild: bool = False):
        self.base_port = base_port
        self.force_rebuild = force_rebuild  # Force rebuild binaries even if they exist
        self.servers: Dict[str, subprocess.Popen] = {}
        self.server_configs: Dict[str, List[StreamConfig]] = {}
        self.server_metrics: Dict[str, ServerMetrics] = {}
        self.server_ports: Dict[str, int] = {}  # Store actual port for each server
        self._lock = threading.Lock()
        self._monitor_threads: Dict[str, threading.Thread] = {}
        self._stop_monitors = threading.Event()
        
    def start_server(
        self,
        name: str,
        streams: List[StreamConfig],
        port: Optional[int] = None,
        api_port: Optional[int] = None,
        wait_for_ready: bool = True,
        timeout: int = 30
    ) -> bool:
        """Start an RTSP server with network simulation"""
        
        # Validate source files exist
        for stream in streams:
            source_path = Path(stream.source_path)
            if not source_path.is_absolute():
                # Try relative to project root
                project_root = Path(__file__).parent.parent.parent
                source_path = project_root / source_path
            
            if not source_path.exists():
                logger.error(f"Source file not found: {source_path}")
                logger.info(f"Searched in:")
                logger.info(f"  - {Path(stream.source_path).resolve()}")
                logger.info(f"  - {source_path}")
                return False
            else:
                logger.debug(f"Found source file: {source_path}")
        
        with self._lock:
            # Stop existing server if running
            if name in self.servers:
                logger.warning(f"Server {name} already running, stopping it first")
                self.stop_server(name)
            
            # Determine ports
            if port is None:
                port = self.base_port + len(self.servers)
            if api_port is None:
                api_port = port + 1000  # API port offset
            
            # Build command line arguments (binary path will be added later)
            args = []
            
            # Handle multiple streams
            if len(streams) == 1:
                # Single stream mode
                stream = streams[0]
                args.extend(["serve", "-f", stream.source_path])
                args.extend(["--port", str(port)])
                args.extend(["--api-port", str(api_port)])
                if stream.auto_repeat:
                    args.append("--auto-repeat")
                args.extend(stream.network_condition.to_cli_args())
            else:
                # Multi-stream mode with different conditions
                args.extend(["serve-files", "--port", str(port)])
                args.extend(["--api-port", str(api_port)])
                
                # Add all files
                for stream in streams:
                    args.extend(["-f", stream.source_path])
                
                # If all streams have same network condition, apply globally
                if all(s.network_condition == streams[0].network_condition for s in streams):
                    args.extend(streams[0].network_condition.to_cli_args())
                else:
                    # Per-source network conditions
                    conditions = []
                    for stream in streams:
                        if stream.network_condition.profile:
                            conditions.append(f"{stream.mount_point}:{stream.network_condition.profile}")
                    if conditions:
                        args.extend(["--per-source-network", ",".join(conditions)])
            
            # Start the server
            logger.info(f"Starting RTSP server {name} on port {port}")
            
            cwd = str(Path(__file__).parent.parent.parent / "crates" / "source-videos")
            logger.debug(f"Working directory: {cwd}")
            
            # Check if working directory exists
            if not Path(cwd).exists():
                logger.error(f"Working directory does not exist: {cwd}")
                return False
            
            # Check if binary already exists
            project_root = Path(cwd).parent.parent
            if os.name == 'nt':  # Windows
                binary_path = project_root / "target" / "debug" / "video-source.exe"
            else:  # Linux/macOS
                binary_path = project_root / "target" / "debug" / "video-source"
            
            # Only build if binary doesn't exist or force rebuild is requested
            if not binary_path.exists() or self.force_rebuild:
                if self.force_rebuild:
                    logger.info(f"Force rebuild requested, building video-source...")
                else:
                    logger.info(f"Binary not found at {binary_path}, building...")
                    
                build_cmd = ["cargo", "build", "--bin", "video-source"]
                build_result = subprocess.run(
                    build_cmd,
                    cwd=cwd,
                    capture_output=True,
                    text=True,
                    timeout=120  # 2 minutes for build
                )
                
                if build_result.returncode != 0:
                    logger.error(f"Failed to build video-source: {build_result.stderr[:500]}")
                    return False
                else:
                    logger.info(f"Build complete")
                    
                # Check again after build
                if not binary_path.exists():
                    logger.error(f"Binary still not found after build at {binary_path}")
                    return False
            else:
                logger.debug(f"Using existing binary: {binary_path}")
            
            # Build final command with binary path and arguments
            cmd = [str(binary_path)] + args
            
            logger.debug(f"Running binary: {' '.join(cmd)}")
            
            try:
                # Start with minimal output capture to avoid blocking
                process = subprocess.Popen(
                    cmd,
                    cwd=cwd,
                    stdout=subprocess.DEVNULL,  # Don't capture to avoid blocking
                    stderr=subprocess.DEVNULL,
                    text=True
                )
            except Exception as e:
                logger.error(f"Failed to start process: {e}")
                return False
            
            self.servers[name] = process
            self.server_configs[name] = streams
            self.server_metrics[name] = ServerMetrics()
            self.server_ports[name] = port  # Store the actual port used
            
            # Give it a moment to start
            time.sleep(2)
            
            # Wait for server to be ready
            if wait_for_ready:
                if not self._wait_for_server(name, port, api_port, timeout):
                    logger.error(f"Server {name} failed to start")
                    if process.poll() is not None:
                        logger.error(f"Process died with exit code: {process.poll()}")
                    # Clean up on failure
                    process.terminate()
                    process.wait(timeout=5)
                    del self.servers[name]
                    del self.server_configs[name]
                    del self.server_metrics[name]
                    del self.server_ports[name]
                    return False
            
            # Start monitoring thread
            monitor_thread = threading.Thread(
                target=self._monitor_server,
                args=(name, api_port),
                daemon=True
            )
            monitor_thread.start()
            self._monitor_threads[name] = monitor_thread
            
            logger.info(f"Server {name} started successfully")
            return True
    
    def _wait_for_server(self, name: str, rtsp_port: int, api_port: int, timeout: int) -> bool:
        """Wait for server to be ready"""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            # Check if process is still running
            if self.servers[name].poll() is not None:
                logger.error(f"Server {name} process died")
                return False
            
            # Check RTSP port
            try:
                sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                sock.settimeout(1)
                result = sock.connect_ex(('127.0.0.1', rtsp_port))
                sock.close()
                if result == 0:
                    logger.debug(f"RTSP port {rtsp_port} is open")
                    
                    # Also check API endpoint if available
                    try:
                        response = requests.get(f"http://127.0.0.1:{api_port}/api/status", timeout=1)
                        if response.status_code == 200:
                            logger.debug(f"API endpoint responding on port {api_port}")
                            return True
                    except:
                        # API might not be enabled, that's OK
                        return True
            except:
                pass
            
            time.sleep(0.5)
        
        return False
    
    def _monitor_server(self, name: str, api_port: int):
        """Monitor server metrics via API"""
        while not self._stop_monitors.is_set():
            try:
                # Fetch metrics from API
                response = requests.get(f"http://127.0.0.1:{api_port}/api/metrics", timeout=2)
                if response.status_code == 200:
                    data = response.json()
                    metrics = ServerMetrics(
                        streams=data.get("streams", 0),
                        clients=data.get("clients", 0),
                        bytes_sent=data.get("bytes_sent", 0),
                        errors=data.get("errors", 0),
                        uptime_seconds=data.get("uptime", 0)
                    )
                    with self._lock:
                        self.server_metrics[name] = metrics
            except:
                # API might be down or not available
                pass
            
            time.sleep(5)  # Update every 5 seconds
    
    def update_network_condition(self, name: str, stream_index: int, condition: NetworkCondition) -> bool:
        """Update network condition for a running stream via API"""
        if name not in self.servers:
            logger.error(f"Server {name} not found")
            return False
        
        # Find API port (assumes it's RTSP port + 1000)
        # This would need to be tracked properly in production
        api_port = self.base_port + 1000 + list(self.servers.keys()).index(name)
        
        try:
            # Update via API
            payload = {
                "stream_index": stream_index,
                "packet_loss": condition.packet_loss,
                "latency_ms": condition.latency_ms,
                "bandwidth_kbps": condition.bandwidth_kbps,
                "jitter_ms": condition.jitter_ms
            }
            
            response = requests.post(
                f"http://127.0.0.1:{api_port}/api/network/update",
                json=payload,
                timeout=5
            )
            
            if response.status_code == 200:
                logger.info(f"Updated network condition for {name} stream {stream_index}")
                return True
            else:
                logger.error(f"Failed to update network condition: {response.text}")
                return False
        except Exception as e:
            logger.error(f"Error updating network condition: {e}")
            return False
    
    def stop_server(self, name: str) -> bool:
        """Stop an RTSP server"""
        with self._lock:
            if name not in self.servers:
                return False
            
            logger.info(f"Stopping server {name}")
            
            # Stop monitoring thread
            if name in self._monitor_threads:
                self._stop_monitors.set()
            
            # Stop the process
            process = self.servers[name]
            process.terminate()
            
            # Wait for graceful shutdown
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                logger.warning(f"Server {name} didn't stop gracefully, killing")
                process.kill()
                process.wait()
            
            # Clean up
            del self.servers[name]
            del self.server_configs[name]
            del self.server_metrics[name]
            del self.server_ports[name]
            if name in self._monitor_threads:
                del self._monitor_threads[name]
            
            return True
    
    def get_metrics(self, name: str) -> Optional[ServerMetrics]:
        """Get current metrics for a server"""
        with self._lock:
            return self.server_metrics.get(name)
    
    def get_rtsp_urls(self, name: str) -> List[str]:
        """Get RTSP URLs for all streams on a server"""
        if name not in self.server_configs or name not in self.server_ports:
            return []
        
        port = self.server_ports[name]  # Use the actual stored port
        urls = []
        for stream in self.server_configs[name]:
            urls.append(f"rtsp://127.0.0.1:{port}/{stream.mount_point}")
        return urls
    
    def cleanup_all(self):
        """Stop all servers"""
        self._stop_monitors.set()
        for name in list(self.servers.keys()):
            self.stop_server(name)

# Example usage for testing
if __name__ == "__main__":
    logging.basicConfig(level=logging.DEBUG)
    
    manager = NetworkSimulationManager()
    
    # Test single stream with degrading network
    stream1 = StreamConfig(
        source_path="test_video.mp4",
        mount_point="test",
        network_condition=NetworkCondition(scenario="degrading")
    )
    
    success = manager.start_server("test_server", [stream1])
    if success:
        print(f"Server started, URLs: {manager.get_rtsp_urls('test_server')}")
        time.sleep(10)
        
        # Update network condition
        new_condition = NetworkCondition(packet_loss=10, latency_ms=500)
        manager.update_network_condition("test_server", 0, new_condition)
        
        time.sleep(5)
        metrics = manager.get_metrics("test_server")
        print(f"Metrics: {metrics}")
        
        manager.stop_server("test_server")