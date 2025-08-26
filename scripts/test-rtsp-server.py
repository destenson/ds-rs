#!/usr/bin/env python3
"""
Simple script to test if RTSP server can start
"""

import subprocess
import time
import socket
import sys
from pathlib import Path

def check_port(port, timeout=5):
    """Check if a port is open"""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(timeout)
    try:
        result = sock.connect_ex(('127.0.0.1', port))
        sock.close()
        return result == 0
    except:
        return False

def main():
    # Find the test video
    project_root = Path(__file__).parent.parent
    test_video = project_root / "crates" / "ds-rs" / "tests" / "test_video.mp4"
    
    if not test_video.exists():
        print(f"Error: Test video not found at {test_video}")
        sys.exit(1)
    
    print(f"Found test video: {test_video}")
    
    # Build the source-videos crate first
    print("\nBuilding source-videos...")
    cwd = project_root / "crates" / "source-videos"
    
    build_cmd = ["cargo", "build", "--release"]
    result = subprocess.run(build_cmd, cwd=cwd, capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"Build failed:\n{result.stderr}")
        sys.exit(1)
    
    print("Build successful!")
    
    # Try to start the RTSP server
    print("\nStarting RTSP server...")
    
    # Use relative path from source-videos directory
    relative_video = "../ds-rs/tests/test_video.mp4"
    
    server_cmd = [
        "cargo", "run", "--release", "--",
        "serve",
        "-f", relative_video,
        "--port", "8554",
        "--mount-point", "test"
    ]
    
    print(f"Command: {' '.join(server_cmd)}")
    print(f"Working directory: {cwd}")
    
    # Start the server
    process = subprocess.Popen(
        server_cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    print("Server process started, waiting for it to be ready...")
    
    # Wait for server to start
    max_wait = 30
    for i in range(max_wait):
        # Check if process is still running
        if process.poll() is not None:
            # Process died
            stdout, stderr = process.communicate()
            print(f"\nServer process died!")
            print(f"stdout:\n{stdout}")
            print(f"stderr:\n{stderr}")
            sys.exit(1)
        
        # Check if port is open
        if check_port(8554):
            print(f"\n✓ RTSP server is running on port 8554!")
            print(f"RTSP URL: rtsp://127.0.0.1:8554/test")
            
            # Keep running for a bit to show output
            print("\nServer output (press Ctrl+C to stop):")
            try:
                while True:
                    line = process.stdout.readline()
                    if line:
                        print(line.strip())
                    time.sleep(0.1)
            except KeyboardInterrupt:
                print("\n\nStopping server...")
                process.terminate()
                process.wait()
                print("Server stopped.")
            
            return 0
        
        print(f"Waiting... ({i+1}/{max_wait})")
        time.sleep(1)
    
    print(f"\n✗ Server failed to start within {max_wait} seconds")
    
    # Get any output
    process.terminate()
    stdout, stderr = process.communicate()
    print(f"\nstdout:\n{stdout}")
    print(f"stderr:\n{stderr}")
    
    return 1

if __name__ == "__main__":
    sys.exit(main())