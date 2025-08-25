#!/usr/bin/env python3
"""
Live Display Automation with GStreamer

This script demonstrates:
1. Starting video sources via the Control API
2. Launching GStreamer pipelines to display the streams
3. Dynamic control and monitoring

Requirements:
- Python 3.6+
- requests library (pip install requests)
- GStreamer installed with Python bindings (python3-gst-1.0)
"""

import sys
import time
import json
import signal
import argparse
import subprocess
from typing import List, Dict, Optional
from threading import Thread
from queue import Queue

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

try:
    import gi
    gi.require_version('Gst', '1.0')
    from gi.repository import Gst, GLib
except ImportError:
    print("GStreamer Python bindings not found. Using subprocess fallback.")
    USE_GSTREAMER_BINDINGS = False
else:
    USE_GSTREAMER_BINDINGS = True
    Gst.init(None)


class GStreamerDisplay:
    """Manages GStreamer display pipelines"""
    
    def __init__(self):
        self.pipelines = []
        self.processes = []
    
    def launch_display_subprocess(self, rtsp_url: str, title: str = "Stream") -> subprocess.Popen:
        """Launch GStreamer using subprocess (fallback method)"""
        pipeline = (
            f"gst-launch-1.0 "
            f"rtspsrc location={rtsp_url} latency=100 ! "
            f"decodebin ! "
            f"videoconvert ! "
            f"videoscale ! "
            f"video/x-raw,width=640,height=480 ! "
            f"fpsdisplaysink video-sink=autovideosink text-overlay=true sync=false"
        )
        
        process = subprocess.Popen(
            pipeline,
            shell=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        self.processes.append(process)
        print(f"Launched display for {title} (PID: {process.pid})")
        return process
    
    def launch_display_native(self, rtsp_url: str, title: str = "Stream"):
        """Launch GStreamer using Python bindings"""
        pipeline_str = (
            f"rtspsrc location={rtsp_url} latency=100 ! "
            f"decodebin ! "
            f"videoconvert ! "
            f"videoscale ! "
            f"video/x-raw,width=640,height=480 ! "
            f"fpsdisplaysink video-sink=autovideosink text-overlay=true sync=false"
        )
        
        pipeline = Gst.parse_launch(pipeline_str)
        self.pipelines.append(pipeline)
        
        # Set up bus watch for messages
        bus = pipeline.get_bus()
        bus.add_signal_watch()
        bus.connect("message", self._on_message, title)
        
        # Start playing
        pipeline.set_state(Gst.State.PLAYING)
        print(f"Launched display for {title}")
        
        return pipeline
    
    def launch_mosaic(self, rtsp_urls: List[str], width: int = 1280, height: int = 720):
        """Launch multiple streams in a mosaic layout"""
        if not USE_GSTREAMER_BINDINGS:
            print("Mosaic requires GStreamer Python bindings")
            return None
        
        # Calculate grid dimensions
        import math
        num_streams = len(rtsp_urls)
        cols = math.ceil(math.sqrt(num_streams))
        rows = math.ceil(num_streams / cols)
        
        cell_width = width // cols
        cell_height = height // rows
        
        # Build pipeline string
        pipeline_str = f"compositor name=comp ! videoconvert ! videoscale ! "
        pipeline_str += f"video/x-raw,width={width},height={height} ! "
        pipeline_str += f"fpsdisplaysink video-sink=autovideosink sync=false"
        
        # Add each source
        for idx, url in enumerate(rtsp_urls):
            x = (idx % cols) * cell_width
            y = (idx // cols) * cell_height
            
            pipeline_str += f" rtspsrc location={url} latency=100 ! decodebin ! "
            pipeline_str += f"videoconvert ! videoscale ! "
            pipeline_str += f"video/x-raw,width={cell_width},height={cell_height} ! "
            pipeline_str += f"comp.sink_{idx} "
            pipeline_str += f"comp.sink_{idx}::xpos={x} comp.sink_{idx}::ypos={y}"
        
        pipeline = Gst.parse_launch(pipeline_str)
        self.pipelines.append(pipeline)
        
        # Start playing
        pipeline.set_state(Gst.State.PLAYING)
        print(f"Launched mosaic display with {num_streams} streams")
        
        return pipeline
    
    def _on_message(self, bus, message, title):
        """Handle GStreamer bus messages"""
        t = message.type
        if t == Gst.MessageType.EOS:
            print(f"{title}: End of stream")
        elif t == Gst.MessageType.ERROR:
            err, debug = message.parse_error()
            print(f"{title}: Error: {err}")
    
    def stop_all(self):
        """Stop all pipelines"""
        # Stop native pipelines
        for pipeline in self.pipelines:
            pipeline.set_state(Gst.State.NULL)
        
        # Kill subprocess pipelines
        for process in self.processes:
            process.terminate()
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
        
        self.pipelines.clear()
        self.processes.clear()


class SourceVideosController:
    """Controls the Source-Videos API"""
    
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = f"{base_url}/api/v1"
        self.headers = {"Content-Type": "application/json"}
    
    def start_server(self, sources: List[Dict]) -> Dict:
        """Start RTSP server with sources"""
        data = {
            "port": 8554,
            "address": "0.0.0.0",
            "sources": sources
        }
        response = requests.post(
            f"{self.base_url}/server/start",
            json=data,
            headers=self.headers
        )
        return response.json()
    
    def get_urls(self) -> List[str]:
        """Get RTSP URLs"""
        response = requests.get(f"{self.base_url}/server/urls")
        return response.json()
    
    def apply_network_profile(self, profile: str) -> Dict:
        """Apply network simulation profile"""
        response = requests.post(
            f"{self.base_url}/network/apply",
            json={"profile": profile},
            headers=self.headers
        )
        return response.json()
    
    def get_metrics(self) -> Dict:
        """Get server metrics"""
        response = requests.get(f"{self.base_url}/metrics")
        return response.json()


class LiveDisplayAutomation:
    """Main automation controller"""
    
    def __init__(self):
        self.api = SourceVideosController()
        self.display = GStreamerDisplay()
        self.running = True
        
        # Set up signal handlers
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)
    
    def _signal_handler(self, signum, frame):
        """Handle shutdown signals"""
        print("\nShutting down...")
        self.running = False
        self.cleanup()
        sys.exit(0)
    
    def setup_sources(self) -> List[str]:
        """Set up video sources and return URLs"""
        print("Setting up video sources...")
        
        # Define test sources
        sources = [
            {"name": "smpte", "type": "test_pattern", "pattern": "smpte"},
            {"name": "ball", "type": "test_pattern", "pattern": "ball"},
            {"name": "snow", "type": "test_pattern", "pattern": "snow"},
            {"name": "bars", "type": "test_pattern", "pattern": "bar"}
        ]
        
        # Start server with sources
        result = self.api.start_server(sources)
        print(f"Server running: {result.get('running')}")
        
        # Get URLs
        urls = self.api.get_urls()
        print(f"Available streams: {len(urls)}")
        for url in urls:
            print(f"  - {url}")
        
        return urls
    
    def launch_displays(self, urls: List[str], mode: str = "individual"):
        """Launch display pipelines"""
        print(f"\nLaunching displays in {mode} mode...")
        
        if mode == "mosaic":
            if USE_GSTREAMER_BINDINGS:
                self.display.launch_mosaic(urls)
            else:
                print("Mosaic mode requires GStreamer Python bindings")
                mode = "individual"
        
        if mode == "individual":
            for i, url in enumerate(urls[:4]):  # Limit to 4 streams
                title = f"Stream {i+1}"
                if USE_GSTREAMER_BINDINGS:
                    self.display.launch_display_native(url, title)
                else:
                    self.display.launch_display_subprocess(url, title)
                time.sleep(0.5)  # Stagger launches
    
    def monitor_loop(self):
        """Interactive monitoring loop"""
        print("\nMonitoring active. Commands:")
        print("  n - Cycle network profiles")
        print("  m - Show metrics")
        print("  q - Quit")
        
        profiles = ["perfect", "4g", "3g", "poor"]
        profile_idx = 0
        
        while self.running:
            try:
                # Non-blocking input (requires Unix-like system)
                import select
                if sys.stdin in select.select([sys.stdin], [], [], 0.1)[0]:
                    cmd = sys.stdin.readline().strip()
                    
                    if cmd == 'n':
                        profile_idx = (profile_idx + 1) % len(profiles)
                        profile = profiles[profile_idx]
                        print(f"Applying network profile: {profile}")
                        result = self.api.apply_network_profile(profile)
                        print(f"  {result.get('message')}")
                    
                    elif cmd == 'm':
                        metrics = self.api.get_metrics()
                        print("Metrics:")
                        print(f"  Sources: {metrics['source_count']}")
                        print(f"  Connections: {metrics['active_connections']}")
                        print(f"  Requests: {metrics['total_requests']}")
                    
                    elif cmd == 'q':
                        self.running = False
                        break
            
            except KeyboardInterrupt:
                break
            except:
                # Fallback for Windows
                time.sleep(1)
    
    def cleanup(self):
        """Clean up resources"""
        print("Cleaning up...")
        self.display.stop_all()
        print("All displays stopped")
    
    def run(self, display_mode: str = "individual"):
        """Main execution"""
        try:
            # Set up sources
            urls = self.setup_sources()
            
            # Launch displays
            self.launch_displays(urls, display_mode)
            
            # Monitor loop
            if USE_GSTREAMER_BINDINGS:
                # Run GLib main loop in thread
                def glib_loop():
                    loop = GLib.MainLoop()
                    loop.run()
                
                glib_thread = Thread(target=glib_loop, daemon=True)
                glib_thread.start()
            
            self.monitor_loop()
            
        finally:
            self.cleanup()


def main():
    parser = argparse.ArgumentParser(description="Live Display Automation")
    parser.add_argument(
        "--mode",
        choices=["individual", "mosaic"],
        default="individual",
        help="Display mode"
    )
    parser.add_argument(
        "--api-url",
        default="http://localhost:3000",
        help="API base URL"
    )
    
    args = parser.parse_args()
    
    # Check API availability
    try:
        response = requests.get(f"{args.api_url}/api/v1/health")
        if response.status_code != 200:
            raise Exception("API not healthy")
    except Exception as e:
        print(f"Error: Cannot connect to API at {args.api_url}")
        print("Start server with: cargo run -- serve --api")
        sys.exit(1)
    
    # Run automation
    automation = LiveDisplayAutomation()
    automation.run(args.mode)


if __name__ == "__main__":
    main()