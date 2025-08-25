#!/usr/bin/env python3
"""
Source-Videos Control API Automation Example

This script demonstrates how to use the Source-Videos Control API
for automated video source management and monitoring.
"""

import requests
import json
import time
import os
from typing import Dict, List, Optional

class SourceVideosAPI:
    """Client for Source-Videos Control API"""
    
    def __init__(self, base_url: str = "http://localhost:3000", api_key: Optional[str] = None):
        self.base_url = f"{base_url}/api/v1"
        self.headers = {"Content-Type": "application/json"}
        if api_key:
            self.headers["X-API-Key"] = api_key
    
    def health_check(self) -> Dict:
        """Check API health status"""
        response = requests.get(f"{self.base_url}/health", headers=self.headers)
        return response.json()
    
    def start_server(self, port: int = 8554, sources: List[Dict] = None) -> Dict:
        """Start RTSP server with optional initial sources"""
        data = {
            "port": port,
            "address": "0.0.0.0",
            "sources": sources or []
        }
        response = requests.post(f"{self.base_url}/server/start", 
                                json=data, headers=self.headers)
        return response.json()
    
    def add_source(self, name: str, pattern: str = "smpte") -> Dict:
        """Add a test pattern source"""
        data = {
            "name": name,
            "type": "test_pattern",
            "pattern": pattern,
            "resolution": {"width": 1920, "height": 1080},
            "framerate": {"numerator": 30, "denominator": 1}
        }
        response = requests.post(f"{self.base_url}/sources", 
                                json=data, headers=self.headers)
        return response.json()
    
    def list_sources(self) -> List[Dict]:
        """List all video sources"""
        response = requests.get(f"{self.base_url}/sources", headers=self.headers)
        return response.json()
    
    def remove_source(self, source_id: str) -> Dict:
        """Remove a video source"""
        response = requests.delete(f"{self.base_url}/sources/{source_id}", 
                                  headers=self.headers)
        return response.json()
    
    def apply_network_profile(self, profile: str) -> Dict:
        """Apply network simulation profile"""
        data = {"profile": profile}
        response = requests.post(f"{self.base_url}/network/apply", 
                                json=data, headers=self.headers)
        return response.json()
    
    def get_network_status(self) -> Dict:
        """Get current network simulation status"""
        response = requests.get(f"{self.base_url}/network/status", headers=self.headers)
        return response.json()
    
    def scan_directory(self, path: str, recursive: bool = True, 
                      add_to_server: bool = True) -> Dict:
        """Scan directory for video files"""
        data = {
            "path": path,
            "recursive": recursive,
            "add_to_server": add_to_server
        }
        response = requests.post(f"{self.base_url}/scan", 
                                json=data, headers=self.headers)
        return response.json()
    
    def get_metrics(self) -> Dict:
        """Get server metrics"""
        response = requests.get(f"{self.base_url}/metrics", headers=self.headers)
        return response.json()
    
    def batch_operations(self, operations: List[Dict]) -> Dict:
        """Perform batch operations on sources"""
        data = {
            "operations": operations,
            "atomic": False
        }
        response = requests.post(f"{self.base_url}/sources/batch", 
                                json=data, headers=self.headers)
        return response.json()


def main():
    """Example automation workflow"""
    
    # Initialize API client
    api = SourceVideosAPI()
    
    print("Source-Videos API Automation Example")
    print("=" * 40)
    
    # 1. Check health
    print("\n1. Checking API health...")
    health = api.health_check()
    print(f"   Status: {health.get('status')}")
    print(f"   Version: {health.get('version')}")
    
    # 2. Start RTSP server with initial sources
    print("\n2. Starting RTSP server...")
    initial_sources = [
        {"name": "test1", "type": "test_pattern", "pattern": "smpte"},
        {"name": "test2", "type": "test_pattern", "pattern": "ball"}
    ]
    server_status = api.start_server(sources=initial_sources)
    print(f"   Running: {server_status.get('running')}")
    print(f"   Port: {server_status.get('port')}")
    print(f"   Sources: {server_status.get('source_count')}")
    
    # 3. Add additional sources
    print("\n3. Adding additional sources...")
    for i in range(3, 6):
        result = api.add_source(f"test{i}", "snow")
        print(f"   Added: {result.get('name')} (ID: {result.get('id')})")
    
    # 4. List all sources
    print("\n4. Current sources:")
    sources = api.list_sources()
    for source in sources:
        print(f"   - {source['name']}: {source['state']} ({source['uri']})")
    
    # 5. Apply network simulation
    print("\n5. Testing network conditions...")
    profiles = ["perfect", "4g", "3g", "poor"]
    for profile in profiles:
        api.apply_network_profile(profile)
        status = api.get_network_status()
        print(f"   {profile}: packet_loss={status['conditions']['packet_loss']}%, "
              f"latency={status['conditions']['latency_ms']}ms")
        time.sleep(1)
    
    # 6. Reset network to perfect
    api.apply_network_profile("perfect")
    
    # 7. Batch operations
    print("\n6. Performing batch operations...")
    batch_ops = [
        {"operation": "add", "source": {"name": "batch1", "type": "test_pattern", "pattern": "checkers-1"}},
        {"operation": "add", "source": {"name": "batch2", "type": "test_pattern", "pattern": "checkers-2"}},
        {"operation": "remove", "source": {"name": "test3"}}
    ]
    batch_result = api.batch_operations(batch_ops)
    print(f"   Success: {batch_result['success_count']}")
    print(f"   Failed: {batch_result['failure_count']}")
    
    # 8. Get metrics
    print("\n7. Server metrics:")
    metrics = api.get_metrics()
    print(f"   Sources: {metrics['source_count']}")
    print(f"   Connections: {metrics['active_connections']}")
    print(f"   Requests: {metrics['total_requests']}")
    
    print("\n" + "=" * 40)
    print("Automation example completed!")


if __name__ == "__main__":
    try:
        main()
    except requests.exceptions.ConnectionError:
        print("Error: Could not connect to API. Ensure the server is running with --api flag.")
    except Exception as e:
        print(f"Error: {e}")