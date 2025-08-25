#!/usr/bin/env python3
"""
Environment Validation Script for ds-rs project
Checks that all required dependencies and tools are installed
"""

import subprocess
import sys
import platform
import shutil
from pathlib import Path

# ANSI color codes
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
CYAN = '\033[0;36m'
NC = '\033[0m'  # No Color

def print_header(text):
    print(f"\n{CYAN}{'='*60}{NC}")
    print(f"{CYAN}{text}{NC}")
    print(f"{CYAN}{'='*60}{NC}")

def print_success(text):
    print(f"{GREEN}{NC} {text}")

def print_error(text):
    print(f"{RED}{NC} {text}")

def print_warning(text):
    print(f"{YELLOW}{NC} {text}")

def print_info(text):
    print(f"{CYAN}{NC} {text}")

def check_command(command, version_flag="--version"):
    """Check if a command is available and get its version"""
    try:
        if shutil.which(command):
            try:
                result = subprocess.run(
                    [command, version_flag],
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                version = result.stdout.strip().split('\n')[0] if result.stdout else "unknown version"
                return True, version
            except:
                return True, "version check failed"
        return False, None
    except:
        return False, None

def check_rust():
    """Check Rust installation and components"""
    print("\nChecking Rust toolchain...")
    
    # Check rustc
    found, version = check_command("rustc")
    if found:
        print_success(f"rustc found: {version}")
    else:
        print_error("rustc not found - install from https://rustup.rs/")
        return False
    
    # Check cargo
    found, version = check_command("cargo")
    if found:
        print_success(f"cargo found: {version}")
    else:
        print_error("cargo not found")
        return False
    
    # Check rustfmt
    found, _ = check_command("rustfmt")
    if found:
        print_success("rustfmt found")
    else:
        print_warning("rustfmt not found - install with: rustup component add rustfmt")
    
    # Check clippy
    found, _ = check_command("cargo-clippy", "clippy")
    if found or check_command("cargo", "clippy")[0]:
        print_success("clippy found")
    else:
        print_warning("clippy not found - install with: rustup component add clippy")
    
    return True

def check_gstreamer():
    """Check GStreamer installation"""
    print("\nChecking GStreamer...")
    
    # Check gst-inspect
    found, version = check_command("gst-inspect-1.0")
    if found:
        print_success(f"GStreamer found: {version}")
        
        # Check for important plugins
        plugins = [
            "videotestsrc",
            "videoconvert",
            "autovideosink",
            "uridecodebin",
            "rtspserver" if platform.system() != "Windows" else None
        ]
        
        for plugin in plugins:
            if plugin:
                try:
                    result = subprocess.run(
                        ["gst-inspect-1.0", plugin],
                        capture_output=True,
                        timeout=5
                    )
                    if result.returncode == 0:
                        print_success(f"  Plugin '{plugin}' available")
                    else:
                        print_warning(f"  Plugin '{plugin}' not found")
                except:
                    print_warning(f"  Could not check plugin '{plugin}'")
        
        return True
    else:
        print_error("GStreamer not found")
        print_info("Install GStreamer from https://gstreamer.freedesktop.org/")
        
        if platform.system() == "Windows":
            print_info("  Windows: Use MSVC 64-bit runtime installer")
        elif platform.system() == "Darwin":
            print_info("  macOS: brew install gstreamer gst-plugins-base gst-plugins-good")
        else:
            print_info("  Linux: sudo apt-get install libgstreamer1.0-dev gstreamer1.0-plugins-*")
        
        return False

def check_python_packages():
    """Check required Python packages"""
    print("\nChecking Python packages...")
    
    packages = {
        "tomli": "Required for TOML configuration parsing"
    }
    
    all_found = True
    for package, description in packages.items():
        try:
            __import__(package)
            print_success(f"{package} - {description}")
        except ImportError:
            print_error(f"{package} not found - {description}")
            print_info(f"  Install with: pip install {package}")
            all_found = False
    
    return all_found

def check_build_tools():
    """Check build tools"""
    print("\nChecking build tools...")
    
    # Platform-specific checks
    if platform.system() == "Windows":
        # Check for Visual Studio or Build Tools
        vs_paths = [
            r"C:\Program Files\Microsoft Visual Studio",
            r"C:\Program Files (x86)\Microsoft Visual Studio",
            r"C:\BuildTools"
        ]
        
        vs_found = any(Path(p).exists() for p in vs_paths)
        if vs_found:
            print_success("Visual Studio Build Tools found")
        else:
            print_warning("Visual Studio Build Tools not detected")
            print_info("  Install from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022")
    
    else:
        # Check for gcc/clang
        gcc_found, gcc_version = check_command("gcc")
        clang_found, clang_version = check_command("clang")
        
        if gcc_found:
            print_success(f"gcc found: {gcc_version}")
        elif clang_found:
            print_success(f"clang found: {clang_version}")
        else:
            print_error("No C compiler found (gcc or clang)")
            return False
    
    # Check pkg-config (Unix-like systems)
    if platform.system() != "Windows":
        found, version = check_command("pkg-config")
        if found:
            print_success(f"pkg-config found: {version}")
        else:
            print_warning("pkg-config not found - may cause build issues")
            print_info("  Install with: apt-get install pkg-config (Linux) or brew install pkg-config (macOS)")
    
    return True

def check_optional_tools():
    """Check optional but useful tools"""
    print("\nChecking optional tools...")
    
    tools = {
        "git": "Version control",
        "python3": "Test orchestration scripts",
        "ffmpeg": "Video processing",
        "jq": "JSON processing (for shell scripts)",
    }
    
    for tool, description in tools.items():
        found, version = check_command(tool)
        if found:
            print_success(f"{tool} - {description}")
        else:
            print_info(f"{tool} not found - {description}")

def check_backend_availability():
    """Check which backends can be used"""
    print("\nChecking backend availability...")
    
    # Mock backend - always available
    print_success("Mock backend - Available (always)")
    
    # Standard backend - requires GStreamer
    if check_command("gst-inspect-1.0")[0]:
        print_success("Standard backend - Available")
    else:
        print_warning("Standard backend - Not available (GStreamer required)")
    
    # DeepStream backend - requires NVIDIA hardware
    nvidia_available = False
    
    # Check for NVIDIA GPU
    if check_command("nvidia-smi")[0]:
        print_info("NVIDIA GPU detected")
        nvidia_available = True
    
    # Check for DeepStream
    deepstream_paths = [
        "/opt/nvidia/deepstream",
        r"C:\DeepStream"
    ]
    
    deepstream_found = any(Path(p).exists() for p in deepstream_paths)
    
    if nvidia_available and deepstream_found:
        print_success("DeepStream backend - Available")
    elif nvidia_available:
        print_warning("DeepStream backend - GPU found but DeepStream SDK not detected")
    else:
        print_info("DeepStream backend - Not available (requires NVIDIA hardware)")

def main():
    print_header("DS-RS Environment Validation")
    
    print(f"\nPlatform: {platform.system()} {platform.machine()}")
    print(f"Python: {sys.version}")
    
    # Track validation results
    results = []
    
    # Required checks
    results.append(("Rust toolchain", check_rust()))
    results.append(("GStreamer", check_gstreamer()))
    results.append(("Build tools", check_build_tools()))
    results.append(("Python packages", check_python_packages()))
    
    # Optional checks
    check_optional_tools()
    check_backend_availability()
    
    # Summary
    print_header("Validation Summary")
    
    all_passed = True
    for name, passed in results:
        if passed:
            print_success(f"{name}: OK")
        else:
            print_error(f"{name}: FAILED")
            all_passed = False
    
    print()
    if all_passed:
        print_success("Environment is ready for ds-rs development and testing!")
        print_info("Run tests with: python scripts/test-orchestrator.py")
        return 0
    else:
        print_error("Some required components are missing.")
        print_info("Please install missing dependencies and run this script again.")
        return 1

if __name__ == "__main__":
    sys.exit(main())
