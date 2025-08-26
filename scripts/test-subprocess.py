#!/usr/bin/env python3
"""
Test subprocess execution for RTSP server
"""

import subprocess
import sys
import os
import time
import threading
from pathlib import Path

def read_output(process, name):
    """Read output from process"""
    while True:
        line = process.stdout.readline()
        if not line:
            break
        print(f"[{name}] {line.strip()}")

def test_direct_cargo():
    """Test running cargo directly"""
    print("\n" + "="*60)
    print("TEST 1: Direct cargo run")
    print("="*60)
    
    project_root = Path(__file__).parent.parent
    cwd = project_root / "crates" / "source-videos"
    
    # Build first
    print("\nBuilding project...")
    build_cmd = ["cargo", "build", "--release"]
    build_result = subprocess.run(build_cmd, cwd=cwd, capture_output=True, text=True)
    if build_result.returncode != 0:
        print(f"Build failed: {build_result.stderr}")
        return False
    print("Build successful!")
    
    # Now try to run
    print("\nStarting RTSP server...")
    cmd = ["cargo", "run", "--release", "--", "serve", "-p", "8554", "-f", "../ds-rs/tests/test_video.mp4"]
    
    print(f"Command: {' '.join(cmd)}")
    print(f"Working directory: {cwd}")
    
    env = os.environ.copy()
    env['RUST_LOG'] = 'info'
    
    try:
        process = subprocess.Popen(
            cmd,
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1
        )
        
        print("\nWaiting for startup...")
        
        # Read output for 10 seconds
        start_time = time.time()
        while time.time() - start_time < 10:
            if process.poll() is not None:
                # Process died
                output = process.stdout.read()
                print(f"\nProcess died with code {process.poll()}!")
                print(f"Output:\n{output}")
                return False
                
            # Read any available output
            line = process.stdout.readline()
            if line:
                print(f"  > {line.strip()}")
                if "RTSP server" in line or "Starting" in line:
                    print("\n✓ Server appears to be starting!")
                    
        print("\n✓ Process still running after 10 seconds")
        print("Terminating...")
        process.terminate()
        process.wait()
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_shell_command():
    """Test with shell=True like the orchestrator"""
    print("\n" + "="*60)
    print("TEST 2: Shell command (like orchestrator)")
    print("="*60)
    
    project_root = Path(__file__).parent.parent
    cwd = project_root / "crates" / "source-videos"
    
    cmd = "cargo run --release -- serve -p 8554 -f ../ds-rs/tests/test_video.mp4"
    
    print(f"Command: {cmd}")
    print(f"Working directory: {cwd}")
    
    env = os.environ.copy()
    env['RUST_LOG'] = 'info'
    
    try:
        process = subprocess.Popen(
            cmd,
            shell=True,
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True
        )
        
        print("\nWaiting for startup...")
        
        # Read output for 10 seconds
        start_time = time.time()
        while time.time() - start_time < 10:
            if process.poll() is not None:
                # Process died
                output = process.stdout.read()
                print(f"\nProcess died with code {process.poll()}!")
                print(f"Output:\n{output}")
                return False
                
            # Read any available output
            line = process.stdout.readline()
            if line:
                print(f"  > {line.strip()}")
                
        print("\n✓ Process still running after 10 seconds")
        print("Terminating...")
        process.terminate()
        process.wait()
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        return False

def main():
    print("Testing subprocess execution methods...")
    
    # Check if test video exists
    project_root = Path(__file__).parent.parent
    test_video = project_root / "crates" / "ds-rs" / "tests" / "test_video.mp4"
    
    if not test_video.exists():
        print(f"Error: Test video not found at {test_video}")
        sys.exit(1)
    
    print(f"✓ Test video found: {test_video}")
    
    # Run tests
    success = False
    
    if test_direct_cargo():
        print("\n✓ Direct cargo run works!")
        success = True
    else:
        print("\n✗ Direct cargo run failed")
    
    if test_shell_command():
        print("\n✓ Shell command works!")
        success = True
    else:
        print("\n✗ Shell command failed")
    
    if not success:
        print("\n" + "="*60)
        print("TROUBLESHOOTING")
        print("="*60)
        print("\nTry running this command manually:")
        print("  cd", project_root / "crates" / "source-videos")
        print("  cargo run --release -- serve -p 8554 -f ../ds-rs/tests/test_video.mp4")
        print("\nIf it works manually but not in the script, it might be:")
        print("  1. Environment variables issue")
        print("  2. Working directory issue")
        print("  3. Process output buffering issue")
        print("  4. Windows-specific subprocess issue")

if __name__ == "__main__":
    main()