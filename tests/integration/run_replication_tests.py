#!/usr/bin/env python3
"""
Script to run replication tests for MerkleKV.

This script provides convenient commands to test the MQTT-based replication
functionality using the public test.mosquitto.org broker.
"""

import argparse
import subprocess
import sys
from pathlib import Path

def run_simple_test():
    """Run the simple replication connectivity test."""
    print("🧪 Running simple replication test...")
    cmd = ["python", "-m", "pytest", "-v", "test_replication_simple.py"]
    result = subprocess.run(cmd, cwd=Path(__file__).parent)
    return result.returncode == 0

def run_full_tests():
    """Run the full replication test suite."""
    print("🧪 Running full replication test suite...")
    cmd = ["python", "-m", "pytest", "-v", "test_replication.py"]
    result = subprocess.run(cmd, cwd=Path(__file__).parent)
    return result.returncode == 0

def run_connectivity_test():
    """Run just the MQTT connectivity test."""
    print("🧪 Testing MQTT broker connectivity...")
    cmd = ["python", "-m", "pytest", "-v", "-k", "test_mqtt_broker_connectivity"]
    result = subprocess.run(cmd, cwd=Path(__file__).parent)
    return result.returncode == 0

def install_dependencies():
    """Install required Python dependencies."""
    print("📦 Installing Python dependencies...")
    cmd = ["pip", "install", "-r", "requirements.txt"]
    result = subprocess.run(cmd, cwd=Path(__file__).parent)
    return result.returncode == 0

def build_server():
    """Build the MerkleKV server."""
    print("🔨 Building MerkleKV server...")
    project_root = Path(__file__).parent.parent.parent
    cmd = ["cargo", "build"]
    result = subprocess.run(cmd, cwd=project_root)
    return result.returncode == 0

def main():
    parser = argparse.ArgumentParser(description="Run MerkleKV replication tests")
    parser.add_argument("command", choices=[
        "connectivity", "simple", "full", "install-deps", "build", "all"
    ], help="Test command to run")
    
    args = parser.parse_args()
    
    success = True
    
    if args.command == "install-deps":
        success = install_dependencies()
    elif args.command == "build":
        success = build_server()
    elif args.command == "connectivity":
        success = run_connectivity_test()
    elif args.command == "simple":
        success = run_simple_test()
    elif args.command == "full":
        success = run_full_tests()
    elif args.command == "all":
        print("🚀 Running complete replication test workflow...")
        success = (
            install_dependencies() and
            build_server() and
            run_connectivity_test() and
            run_simple_test() and
            run_full_tests()
        )
    
    if success:
        print("✅ All tests completed successfully!")
        sys.exit(0)
    else:
        print("❌ Some tests failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()
