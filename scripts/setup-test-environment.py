#!/usr/bin/env python3
"""
Setup script for test orchestration environment
Installs required Python packages and validates environment
"""

import subprocess
import sys
import importlib.util
from pathlib import Path

def check_python_version():
    """Check Python version is 3.7+"""
    if sys.version_info < (3, 7):
        print(f"Error: Python 3.7+ required, found {sys.version}")
        return False
    print(f" Python {sys.version.split()[0]} detected")
    return True

def check_package(package_name):
    """Check if a package is installed"""
    spec = importlib.util.find_spec(package_name)
    return spec is not None

def install_requirements():
    """Install required packages"""
    requirements_file = Path(__file__).parent / "requirements.txt"
    
    if not requirements_file.exists():
        print(f"Error: {requirements_file} not found")
        return False
    
    print("\nInstalling required packages...")
    
    # Check if pip is available
    try:
        subprocess.run([sys.executable, "-m", "pip", "--version"], 
                      check=True, capture_output=True)
    except subprocess.CalledProcessError:
        print("Error: pip not found. Please install pip first.")
        return False
    
    # Install packages
    try:
        result = subprocess.run(
            [sys.executable, "-m", "pip", "install", "-r", str(requirements_file)],
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0:
            print(" All packages installed successfully")
            return True
        else:
            print(f"Error installing packages:\n{result.stderr}")
            return False
            
    except Exception as e:
        print(f"Error: {e}")
        return False

def check_installed_packages():
    """Check which required packages are already installed"""
    required = {
        'tomli': 'TOML configuration parsing',
        'requests': 'API communication'
    }
    
    optional = {
        'pytest': 'Test framework integration',
        'junit_xml': 'JUnit XML reports'
    }
    
    print("\nChecking required packages:")
    all_required_installed = True
    for package, description in required.items():
        if check_package(package):
            print(f"   {package:12} - {description}")
        else:
            print(f"   {package:12} - {description} (MISSING)")
            all_required_installed = False
    
    print("\nChecking optional packages:")
    for package, description in optional.items():
        if check_package(package):
            print(f"   {package:12} - {description}")
        else:
            print(f"  ○ {package:12} - {description} (not installed)")
    
    return all_required_installed

def validate_test_files():
    """Check that test configuration files exist"""
    script_dir = Path(__file__).parent
    config_dir = script_dir / "config"
    lib_dir = script_dir / "lib"
    
    required_files = [
        (script_dir / "test-orchestrator.py", "Main test orchestrator"),
        (config_dir / "test-scenarios.toml", "Basic test scenarios"),
        (config_dir / "network-inference-scenarios.toml", "Network inference scenarios"),
        (lib_dir / "network_controller.py", "Network simulation controller"),
        (lib_dir / "inference_metrics.py", "Inference metrics tracking"),
        (lib_dir / "test_helpers.py", "Test helper functions")
    ]
    
    print("\nValidating test files:")
    all_files_exist = True
    
    for file_path, description in required_files:
        if file_path.exists():
            print(f"   {file_path.name:35} - {description}")
        else:
            print(f"   {file_path.name:35} - {description} (MISSING)")
            all_files_exist = False
    
    return all_files_exist

def main():
    """Main setup process"""
    print("="*60)
    print("Test Orchestration Environment Setup")
    print("="*60)
    
    # Check Python version
    if not check_python_version():
        sys.exit(1)
    
    # Check installed packages
    all_installed = check_installed_packages()
    
    # Install if needed
    if not all_installed:
        print("\nSome required packages are missing.")
        response = input("Would you like to install them now? (y/n): ")
        
        if response.lower() == 'y':
            if not install_requirements():
                print("\nSetup failed. Please install packages manually:")
                print("  pip install -r scripts/requirements.txt")
                sys.exit(1)
            
            # Re-check after installation
            print("\nVerifying installation:")
            check_installed_packages()
        else:
            print("\nPlease install required packages manually:")
            print("  pip install -r scripts/requirements.txt")
            sys.exit(1)
    
    # Validate test files
    if not validate_test_files():
        print("\nWarning: Some test files are missing.")
        print("The test orchestration may not work properly.")
    
    print("\n" + "="*60)
    print("Setup complete! You can now run tests with:")
    print("  python scripts/test-orchestrator.py --scenario <scenario>")
    print("\nAvailable network inference scenarios:")
    print("  - network-inference-basic")
    print("  - network-inference-multi")
    print("  - network-inference-recovery")
    print("  - network-inference-drone")
    print("  - network-inference-stress")
    print("  - network-inference-satellite")
    print("  - network-inference-benchmark")
    print("="*60)

if __name__ == "__main__":
    main()
