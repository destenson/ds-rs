#!/usr/bin/env python3
"""
Debug script to diagnose RTSP server startup issues
"""

import subprocess
import time
import socket
import sys
import os
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

def run_command(cmd, cwd):
    """Run a command and capture all output"""
    print(f"\n{'='*60}")
    print(f"Running: {cmd}")
    print(f"Working directory: {cwd}")
    print(f"{'='*60}\n")
    
    env = os.environ.copy()
    env['RUST_LOG'] = 'debug'
    env['RUST_BACKTRACE'] = '1'
    
    try:
        # Try running with shell=True (like test orchestrator)
        print("Attempt 1: Using shell=True (like test orchestrator)")
        process = subprocess.Popen(
            cmd,
            shell=True,
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True
        )
        
        # Wait a bit and check if it's running
        time.sleep(3)
        
        if process.poll() is not None:
            # Process died
            output = process.stdout.read()
            print(f"Process died immediately!")
            print(f"Output:\n{output}")
            return False
        
        print("Process started, checking output...")
        
        # Read some output
        for _ in range(10):
            line = process.stdout.readline()
            if line:
                print(f"  > {line.strip()}")
            time.sleep(0.5)
            
            # Check if port is open
            if check_port(8554):
                print("\n✓ Port 8554 is now open!")
                process.terminate()
                return True
        
        print("\nPort 8554 not opening...")
        process.terminate()
        remaining = process.stdout.read()
        if remaining:
            print(f"Remaining output:\n{remaining}")
        
    except Exception as e:
        print(f"Error: {e}")
        return False
    
    return False

def main():
    project_root = Path(__file__).parent.parent
    source_videos_dir = project_root / "crates" / "source-videos"
    test_video = project_root / "crates" / "ds-rs" / "tests" / "test_video.mp4"
    
    if not test_video.exists():
        print(f"Error: Test video not found at {test_video}")
        sys.exit(1)
    
    print(f"Test video found: {test_video}")
    print(f"Working directory: {source_videos_dir}")
    
    # Test different command variations
    commands = [
        # As shown in the config
        "cargo run -- serve -p 8554 -f ../ds-rs/tests/test_video.mp4",
        
        # With --release
        "cargo run --release -- serve -p 8554 -f ../ds-rs/tests/test_video.mp4",
        
        # With full --port
        "cargo run -- serve --port 8554 -f ../ds-rs/tests/test_video.mp4",
        
        # Direct binary if it exists
        "..\\..\\target\\debug\\video-source.exe serve -p 8554 -f ../ds-rs/tests/test_video.mp4",
        
        # Try with absolute path
        f'cargo run -- serve -p 8554 -f "{test_video}"',
    ]
    
    for cmd in commands:
        if run_command(cmd, str(source_videos_dir)):
            print(f"\n✓ SUCCESS with command: {cmd}")
            break
    else:
        print("\n✗ All command variations failed")
        
        # Try building first
        print("\nAttempting to build the project first...")
        build_result = subprocess.run(
            ["cargo", "build"],
            cwd=source_videos_dir,
            capture_output=True,
            text=True
        )
        
        if build_result.returncode != 0:
            print(f"Build failed:\n{build_result.stderr}")
        else:
            print("Build successful, trying again...")
            
            # Try the first command again
            if run_command(commands[0], str(source_videos_dir)):
                print("\n✓ SUCCESS after building")
            else:
                print("\n✗ Still failing after build")
                
                # Check if the binary exists
                debug_bin = project_root / "target" / "debug" / "video-source.exe"
                release_bin = project_root / "target" / "release" / "video-source.exe"
                
                print(f"\nBinary check:")
                print(f"  Debug binary exists: {debug_bin.exists()}")
                print(f"  Release binary exists: {release_bin.exists()}")

if __name__ == "__main__":
    main()