#!/usr/bin/env python3
"""
Inference Metrics Collection and Validation for Network Testing
Tracks detection quality, recovery times, and performance under degraded conditions
"""

import json
import logging
import re
import statistics
import time
from dataclasses import dataclass, field, asdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
import threading
import queue
import subprocess

logger = logging.getLogger(__name__)

@dataclass
class DetectionMetrics:
    """Metrics for object detection performance"""
    frame_count: int = 0
    detection_count: int = 0
    fps: float = 0.0
    avg_confidence: float = 0.0
    detection_rate: float = 0.0  # detections per frame
    false_positives: int = 0
    false_negatives: int = 0
    processing_latency_ms: float = 0.0
    
@dataclass
class NetworkMetrics:
    """Network-related performance metrics"""
    packet_loss_rate: float = 0.0
    latency_ms: float = 0.0
    bandwidth_kbps: int = 0
    connection_drops: int = 0
    recovery_attempts: int = 0
    recovery_time_ms: float = 0.0
    
@dataclass
class StreamHealth:
    """Health status of a video stream"""
    stream_id: str
    is_active: bool = False
    last_frame_time: float = 0.0
    frames_dropped: int = 0
    buffer_level_bytes: int = 0
    error_count: int = 0
    
@dataclass
class InferenceQualityMetrics:
    """Overall inference quality under network stress"""
    timestamp: float = field(default_factory=time.time)
    scenario: str = ""
    detection: DetectionMetrics = field(default_factory=DetectionMetrics)
    network: NetworkMetrics = field(default_factory=NetworkMetrics)
    streams: List[StreamHealth] = field(default_factory=list)
    tracking_continuity: float = 0.0  # 0-1 score
    graceful_degradation: bool = True
    complete_failures: int = 0
    
    def to_dict(self) -> Dict:
        """Convert to dictionary for JSON serialization"""
        return {
            'timestamp': self.timestamp,
            'scenario': self.scenario,
            'detection': asdict(self.detection),
            'network': asdict(self.network),
            'streams': [asdict(s) for s in self.streams],
            'tracking_continuity': self.tracking_continuity,
            'graceful_degradation': self.graceful_degradation,
            'complete_failures': self.complete_failures
        }

class InferenceMonitor:
    """Monitors inference process and collects metrics"""
    
    def __init__(self, scenario_name: str = ""):
        self.scenario_name = scenario_name
        self.metrics = InferenceQualityMetrics(scenario=scenario_name)
        self.output_parser = InferenceOutputParser()
        self._lock = threading.Lock()
        self._monitoring = False
        self._monitor_thread = None
        self._process = None
        
    def start_monitoring(self, process: subprocess.Popen):
        """Start monitoring an inference process"""
        self._process = process
        self._monitoring = True
        self._monitor_thread = threading.Thread(
            target=self._monitor_loop,
            daemon=True
        )
        self._monitor_thread.start()
        logger.info(f"Started monitoring inference for scenario: {self.scenario_name}")
        
    def stop_monitoring(self):
        """Stop monitoring"""
        self._monitoring = False
        if self._monitor_thread:
            self._monitor_thread.join(timeout=5)
        logger.info(f"Stopped monitoring for scenario: {self.scenario_name}")
        
    def _monitor_loop(self):
        """Main monitoring loop"""
        while self._monitoring and self._process:
            # Read process output
            try:
                line = self._process.stdout.readline()
                if not line:
                    if self._process.poll() is not None:
                        # Process has ended
                        break
                    continue
                    
                # Parse the output line
                metrics_update = self.output_parser.parse_line(line.strip())
                if metrics_update:
                    self._update_metrics(metrics_update)
                    
            except Exception as e:
                logger.error(f"Error in monitor loop: {e}")
                
            time.sleep(0.1)
    
    def _update_metrics(self, update: Dict):
        """Update metrics from parsed output"""
        with self._lock:
            if 'fps' in update:
                self.metrics.detection.fps = update['fps']
            if 'detections' in update:
                self.metrics.detection.detection_count += update['detections']
            if 'frame' in update:
                self.metrics.detection.frame_count = update['frame']
            if 'confidence' in update:
                self.metrics.detection.avg_confidence = update['confidence']
            if 'latency_ms' in update:
                self.metrics.detection.processing_latency_ms = update['latency_ms']
            if 'recovery_attempt' in update:
                self.metrics.network.recovery_attempts += 1
            if 'stream_active' in update:
                stream_id = update.get('stream_id', 'default')
                stream = self._get_or_create_stream(stream_id)
                stream.is_active = update['stream_active']
            if 'tracking_continuity' in update:
                self.metrics.tracking_continuity = update['tracking_continuity']
                
    def _get_or_create_stream(self, stream_id: str) -> StreamHealth:
        """Get existing stream or create new one"""
        for stream in self.metrics.streams:
            if stream.stream_id == stream_id:
                return stream
        
        new_stream = StreamHealth(stream_id=stream_id)
        self.metrics.streams.append(new_stream)
        return new_stream
    
    def get_metrics(self) -> InferenceQualityMetrics:
        """Get current metrics"""
        with self._lock:
            # Calculate derived metrics
            if self.metrics.detection.frame_count > 0:
                self.metrics.detection.detection_rate = (
                    self.metrics.detection.detection_count / 
                    self.metrics.detection.frame_count
                )
            
            # Check for graceful degradation
            active_streams = sum(1 for s in self.metrics.streams if s.is_active)
            total_streams = len(self.metrics.streams)
            if total_streams > 0:
                if active_streams == 0:
                    self.metrics.graceful_degradation = False
                    self.metrics.complete_failures += 1
                    
            return self.metrics

class InferenceOutputParser:
    """Parses inference process output to extract metrics"""
    
    def __init__(self):
        # Common patterns in inference output
        self.patterns = {
            'fps': re.compile(r'FPS:\s*([\d.]+)'),
            'frame': re.compile(r'Frame\s+(\d+)'),
            'detections': re.compile(r'Detected\s+(\d+)\s+objects'),
            'confidence': re.compile(r'conf[idence]*=\s*([\d.]+)'),
            'latency': re.compile(r'latency[:\s]+([\d.]+)\s*ms'),
            'recovery': re.compile(r'recovery attempt|reconnecting|retrying', re.IGNORECASE),
            'stream_active': re.compile(r'stream\s+(active|inactive|connected|disconnected)', re.IGNORECASE),
            'tracking': re.compile(r'tracking.*continuity[:\s]+([\d.]+)'),
            'error': re.compile(r'error|failed|exception', re.IGNORECASE),
        }
        
    def parse_line(self, line: str) -> Optional[Dict]:
        """Parse a single output line for metrics"""
        if not line:
            return None
            
        result = {}
        
        # Check FPS
        match = self.patterns['fps'].search(line)
        if match:
            result['fps'] = float(match.group(1))
            
        # Check frame number
        match = self.patterns['frame'].search(line)
        if match:
            result['frame'] = int(match.group(1))
            
        # Check detection count
        match = self.patterns['detections'].search(line)
        if match:
            result['detections'] = int(match.group(1))
            
        # Check confidence
        match = self.patterns['confidence'].search(line)
        if match:
            result['confidence'] = float(match.group(1))
            
        # Check latency
        match = self.patterns['latency'].search(line)
        if match:
            result['latency_ms'] = float(match.group(1))
            
        # Check for recovery attempts
        if self.patterns['recovery'].search(line):
            result['recovery_attempt'] = True
            
        # Check stream status
        match = self.patterns['stream_active'].search(line)
        if match:
            status = match.group(1).lower()
            result['stream_active'] = status in ['active', 'connected']
            
        # Check tracking continuity
        match = self.patterns['tracking'].search(line)
        if match:
            result['tracking_continuity'] = float(match.group(1))
            
        # Check for errors
        if self.patterns['error'].search(line):
            result['error'] = True
            
        return result if result else None

class MetricsValidator:
    """Validates metrics against test requirements"""
    
    @staticmethod
    def validate_detection_quality(
        metrics: InferenceQualityMetrics,
        min_fps: float = 1.0,
        min_detections: int = 0,
        min_detection_rate: float = 0.0,
        max_latency_ms: float = 1000.0
    ) -> Tuple[bool, str]:
        """Validate detection quality metrics"""
        
        if metrics.detection.fps < min_fps:
            return False, f"FPS {metrics.detection.fps} below minimum {min_fps}"
            
        if metrics.detection.detection_count < min_detections:
            return False, f"Detection count {metrics.detection.detection_count} below minimum {min_detections}"
            
        if metrics.detection.detection_rate < min_detection_rate:
            return False, f"Detection rate {metrics.detection.detection_rate} below minimum {min_detection_rate}"
            
        if metrics.detection.processing_latency_ms > max_latency_ms:
            return False, f"Latency {metrics.detection.processing_latency_ms}ms exceeds maximum {max_latency_ms}ms"
            
        return True, "Detection quality validation passed"
    
    @staticmethod
    def validate_recovery(
        metrics: InferenceQualityMetrics,
        max_recovery_time_ms: float = 30000.0,
        min_success_rate: float = 0.8
    ) -> Tuple[bool, str]:
        """Validate recovery metrics"""
        
        if metrics.network.recovery_time_ms > max_recovery_time_ms:
            return False, f"Recovery time {metrics.network.recovery_time_ms}ms exceeds maximum {max_recovery_time_ms}ms"
            
        # Calculate success rate from active streams
        active_streams = sum(1 for s in metrics.streams if s.is_active)
        total_streams = len(metrics.streams)
        
        if total_streams > 0:
            success_rate = active_streams / total_streams
            if success_rate < min_success_rate:
                return False, f"Success rate {success_rate:.2f} below minimum {min_success_rate}"
                
        return True, "Recovery validation passed"
    
    @staticmethod
    def validate_graceful_degradation(
        metrics: InferenceQualityMetrics,
        allow_quality_reduction: bool = True,
        prevent_complete_failure: bool = True
    ) -> Tuple[bool, str]:
        """Validate graceful degradation behavior"""
        
        if prevent_complete_failure and metrics.complete_failures > 0:
            return False, f"Complete failures detected: {metrics.complete_failures}"
            
        if not metrics.graceful_degradation:
            return False, "System did not degrade gracefully"
            
        return True, "Graceful degradation validation passed"
    
    @staticmethod
    def validate_multi_stream(
        metrics: InferenceQualityMetrics,
        min_healthy_streams: int = 1,
        per_stream_min_fps: float = 5.0
    ) -> Tuple[bool, str]:
        """Validate multi-stream health"""
        
        healthy_streams = sum(1 for s in metrics.streams if s.is_active and s.error_count == 0)
        
        if healthy_streams < min_healthy_streams:
            return False, f"Only {healthy_streams} healthy streams, minimum {min_healthy_streams} required"
            
        # Check per-stream FPS if we have individual stream metrics
        # This would need more detailed per-stream FPS tracking in production
        
        return True, "Multi-stream validation passed"

class MetricsReporter:
    """Generates test reports from collected metrics"""
    
    def __init__(self, output_dir: Path = None):
        self.output_dir = output_dir or Path("test-results")
        self.output_dir.mkdir(exist_ok=True)
        
    def generate_report(
        self,
        scenario_name: str,
        metrics_list: List[InferenceQualityMetrics],
        format: str = "json"
    ) -> Path:
        """Generate a test report"""
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"network_inference_{scenario_name}_{timestamp}.{format}"
        output_path = self.output_dir / filename
        
        if format == "json":
            self._write_json_report(output_path, metrics_list)
        elif format == "html":
            self._write_html_report(output_path, metrics_list)
        elif format == "csv":
            self._write_csv_report(output_path, metrics_list)
        else:
            raise ValueError(f"Unsupported format: {format}")
            
        logger.info(f"Report generated: {output_path}")
        return output_path
    
    def _write_json_report(self, path: Path, metrics_list: List[InferenceQualityMetrics]):
        """Write JSON report"""
        data = {
            'scenario': metrics_list[0].scenario if metrics_list else "",
            'metrics': [m.to_dict() for m in metrics_list],
            'summary': self._generate_summary(metrics_list)
        }
        
        with open(path, 'w') as f:
            json.dump(data, f, indent=2)
    
    def _generate_summary(self, metrics_list: List[InferenceQualityMetrics]) -> Dict:
        """Generate summary statistics"""
        if not metrics_list:
            return {}
            
        fps_values = [m.detection.fps for m in metrics_list if m.detection.fps > 0]
        detection_rates = [m.detection.detection_rate for m in metrics_list]
        recovery_attempts = sum(m.network.recovery_attempts for m in metrics_list)
        
        return {
            'total_samples': len(metrics_list),
            'avg_fps': statistics.mean(fps_values) if fps_values else 0,
            'min_fps': min(fps_values) if fps_values else 0,
            'max_fps': max(fps_values) if fps_values else 0,
            'avg_detection_rate': statistics.mean(detection_rates) if detection_rates else 0,
            'total_recovery_attempts': recovery_attempts,
            'complete_failures': sum(m.complete_failures for m in metrics_list)
        }
    
    def _write_html_report(self, path: Path, metrics_list: List[InferenceQualityMetrics]):
        """Write HTML report (placeholder for future implementation)"""
        # Would generate a nice HTML report with charts
        pass
    
    def _write_csv_report(self, path: Path, metrics_list: List[InferenceQualityMetrics]):
        """Write CSV report (placeholder for future implementation)"""
        # Would generate CSV for easy analysis in Excel
        pass

# Example usage
if __name__ == "__main__":
    logging.basicConfig(level=logging.DEBUG)
    
    # Create monitor for a scenario
    monitor = InferenceMonitor("test_scenario")
    
    # Simulate some metrics updates
    test_metrics = {
        'fps': 25.5,
        'detections': 10,
        'frame': 100,
        'confidence': 0.85,
        'stream_active': True
    }
    
    monitor._update_metrics(test_metrics)
    
    # Get current metrics
    metrics = monitor.get_metrics()
    print(f"Current metrics: {metrics.to_dict()}")
    
    # Validate metrics
    validator = MetricsValidator()
    passed, msg = validator.validate_detection_quality(metrics, min_fps=20.0)
    print(f"Validation: {passed} - {msg}")
    
    # Generate report
    reporter = MetricsReporter()
    report_path = reporter.generate_report("test", [metrics], format="json")
    print(f"Report saved to: {report_path}")