#!/usr/bin/env python3
"""
Test network simulation directly
"""

import sys
import logging
from pathlib import Path

# Add lib directory to path
sys.path.insert(0, str(Path(__file__).parent / "lib"))

from network_controller import NetworkSimulationManager, StreamConfig, NetworkCondition

logging.basicConfig(level=logging.DEBUG)

def main():
    print("Testing network simulation manager...")
    
    manager = NetworkSimulationManager()
    
    # Test with simple configuration
    stream = StreamConfig(
        source_path=str(Path(__file__).parent.parent / "crates" / "ds-rs" / "tests" / "test_video.mp4"),
        mount_point="test",
        network_condition=NetworkCondition(profile="wifi"),
        auto_repeat=True
    )
    
    print(f"\nStarting server with source: {stream.source_path}")
    print(f"Network profile: wifi")
    
    success = manager.start_server(
        "test_server",
        [stream],
        port=8558,
        api_port=9558,
        wait_for_ready=True,
        timeout=30
    )
    
    if success:
        print("\n✓ Server started successfully!")
        print(f"RTSP URLs: {manager.get_rtsp_urls('test_server')}")
        
        input("\nPress Enter to stop the server...")
        
        manager.stop_server("test_server")
        print("Server stopped.")
    else:
        print("\n✗ Failed to start server")
        print("Check the error messages above for details")
        
        # Try without network simulation
        print("\nTrying without network simulation (direct command)...")
        import subprocess
        
        cmd = ["cargo", "build", "--bin", "video-source"]
        cwd = Path(__file__).parent.parent / "crates" / "source-videos"
        
        print(f"Building first: {' '.join(cmd)}")
        print(f"Working directory: {cwd}")
        
        try:
            result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=60)
            print(f"Build exit code: {result.returncode}")
            
            if result.returncode == 0:
                print("Build successful, now trying to run...")
                cmd = ["cargo", "run", "--", "serve", "-p", "8558", "-f", "../ds-rs/tests/test_video.mp4"]
                print(f"Command: {' '.join(cmd)}")
                
                # Start the process and check if it runs
                proc = subprocess.Popen(cmd, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                
                import time
                time.sleep(5)
                
                if proc.poll() is None:
                    print("✓ Process is running")
                    proc.terminate()
                else:
                    stdout, stderr = proc.communicate()
                    print(f"✗ Process died with code {proc.poll()}")
                    print(f"stdout: {stdout[:500]}")  # First 500 chars
                    print(f"stderr: {stderr[:500]}")
        except subprocess.TimeoutExpired:
            print("Command timed out")

if __name__ == "__main__":
    main()