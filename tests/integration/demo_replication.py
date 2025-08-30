#!/usr/bin/env python3
"""
Interactive demo showing MerkleKV replication in action.

This script starts two MerkleKV nodes with replication enabled and 
demonstrates real-time data synchronization between them.
"""

import asyncio
import subprocess
import time
import uuid
from pathlib import Path
import toml
import os
import signal
import sys

async def create_demo_config(port: int, node_id: str, topic_prefix: str) -> Path:
    """Create a demo config file with replication enabled."""
    config = {
        "host": "127.0.0.1",
        "port": port,
        "storage_path": f"demo_data_{node_id}",
        "engine": "rwlock",
        "sync_interval_seconds": 60,
        "replication": {
            "enabled": True,
            "mqtt_broker": "test.mosquitto.org",
            "mqtt_port": 1883,
            "topic_prefix": topic_prefix,
            "client_id": node_id
        }
    }
    
    temp_config = Path(f"/tmp/demo_config_{node_id}.toml")
    with open(temp_config, 'w') as f:
        toml.dump(config, f)
    
    return temp_config

async def start_demo_server(config_path: Path, node_name: str) -> subprocess.Popen:
    """Start a MerkleKV server for demo."""
    cmd = ["cargo", "run", "--", "--config", str(config_path)]
    
    # Get project root
    project_root = Path.cwd()
    if "tests" in str(project_root):
        project_root = project_root.parent.parent
    
    print(f"🚀 Starting {node_name}...")
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=project_root,
        env={**os.environ, "RUST_LOG": "info"}
    )
    
    return process

async def execute_demo_command(host: str, port: int, command: str) -> str:
    """Execute a command and return response."""
    try:
        reader, writer = await asyncio.open_connection(host, port)
        
        writer.write(f"{command}\r\n".encode())
        await writer.drain()
        
        data = await reader.read(1024)
        response = data.decode().strip()
        
        writer.close()
        await writer.wait_closed()
        
        return response
    except Exception as e:
        return f"ERROR: {e}"

async def wait_for_server(port: int, timeout: int = 30) -> bool:
    """Wait for server to be ready."""
    import socket
    start_time = time.time()
    
    while time.time() - start_time < timeout:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=1):
                return True
        except (socket.timeout, ConnectionRefusedError):
            await asyncio.sleep(0.5)
    
    return False

async def demo_replication():
    """Run the replication demo."""
    print("🎯 MerkleKV Replication Demo")
    print("=" * 50)
    
    # Generate unique topic for this demo
    topic_prefix = f"demo_merkle_kv_{int(time.time())}"
    
    # Create configs
    config1 = await create_demo_config(7600, "demo_node1", topic_prefix)
    config2 = await create_demo_config(7601, "demo_node2", topic_prefix)
    
    server1 = None
    server2 = None
    
    try:
        # Start servers
        server1 = await start_demo_server(config1, "Node 1 (Port 7600)")
        server2 = await start_demo_server(config2, "Node 2 (Port 7601)")
        
        # Wait for servers to start
        print("\n⏳ Waiting for servers to start...")
        
        if not await wait_for_server(7600):
            print("❌ Failed to start Node 1")
            return
        print("✅ Node 1 ready")
        
        if not await wait_for_server(7601):
            print("❌ Failed to start Node 2")
            return
        print("✅ Node 2 ready")
        
        # Wait for MQTT connections
        print("\n⏳ Waiting for MQTT connections to establish...")
        await asyncio.sleep(5)
        
        print("\n🎬 Starting replication demo...")
        print("-" * 50)
        
        # Demo scenario 1: Basic SET/GET replication
        print("\n📝 Demo 1: Basic SET operation replication")
        print("→ Setting 'user:alice' = 'Alice Johnson' on Node 1...")
        
        result = await execute_demo_command("127.0.0.1", 7600, "SET user:alice 'Alice Johnson'")
        print(f"   Node 1 response: {result}")
        
        print("→ Waiting for replication (3 seconds)...")
        await asyncio.sleep(3)
        
        print("→ Getting 'user:alice' from Node 2...")
        result = await execute_demo_command("127.0.0.1", 7601, "GET user:alice")
        print(f"   Node 2 response: {result}")
        
        if "Alice Johnson" in result:
            print("   ✅ Replication successful!")
        else:
            print("   ⚠️  Replication may still be in progress...")
        
        # Demo scenario 2: Bi-directional replication
        print("\n📝 Demo 2: Bi-directional replication")
        print("→ Setting 'user:bob' = 'Bob Smith' on Node 2...")
        
        result = await execute_demo_command("127.0.0.1", 7601, "SET user:bob 'Bob Smith'")
        print(f"   Node 2 response: {result}")
        
        print("→ Waiting for replication (3 seconds)...")
        await asyncio.sleep(3)
        
        print("→ Getting 'user:bob' from Node 1...")
        result = await execute_demo_command("127.0.0.1", 7600, "GET user:bob")
        print(f"   Node 1 response: {result}")
        
        if "Bob Smith" in result:
            print("   ✅ Bi-directional replication working!")
        else:
            print("   ⚠️  Replication may still be in progress...")
        
        # Demo scenario 3: Multiple operations
        print("\n📝 Demo 3: Multiple operations")
        operations = [
            ("Node 1", 7600, "SET product:123 'Laptop'"),
            ("Node 2", 7601, "SET product:456 'Mouse'"),
            ("Node 1", 7600, "SET product:789 'Keyboard'"),
        ]
        
        for node_name, port, command in operations:
            print(f"→ {command} on {node_name}")
            result = await execute_demo_command("127.0.0.1", port, command)
            print(f"   Response: {result}")
            await asyncio.sleep(1)
        
        print("\n→ Waiting for all replications (5 seconds)...")
        await asyncio.sleep(5)
        
        # Check all values on both nodes
        keys = ["product:123", "product:456", "product:789"]
        
        print("\n📊 Final state verification:")
        for key in keys:
            print(f"\n   Checking '{key}':")
            
            result1 = await execute_demo_command("127.0.0.1", 7600, f"GET {key}")
            result2 = await execute_demo_command("127.0.0.1", 7601, f"GET {key}")
            
            print(f"   Node 1: {result1}")
            print(f"   Node 2: {result2}")
            
            if result1 == result2 and "VALUE" in result1:
                print("   ✅ Consistent across nodes")
            else:
                print("   ⚠️  Inconsistent or still replicating")
        
        print("\n🎉 Demo completed!")
        print("\n💡 Key observations:")
        print("   • Real-time replication via MQTT")
        print("   • Bi-directional synchronization")  
        print("   • Eventual consistency model")
        print("   • CBOR binary message format")
        print("   • Loop prevention (nodes ignore own messages)")
        
        print(f"\n📡 MQTT Topic: {topic_prefix}/events")
        print("   You can monitor messages with:")
        print(f"   mosquitto_sub -h test.mosquitto.org -t '{topic_prefix}/events/#'")
        
    except KeyboardInterrupt:
        print("\n\n⏹️  Demo interrupted by user")
    except Exception as e:
        print(f"\n❌ Demo failed: {e}")
    finally:
        print("\n🧹 Cleaning up...")
        
        # Stop servers
        if server1:
            server1.terminate()
            try:
                server1.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server1.kill()
        
        if server2:
            server2.terminate()
            try:
                server2.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server2.kill()
        
        # Clean up config files
        for config_file in [config1, config2]:
            if config_file.exists():
                config_file.unlink()
        
        print("✅ Cleanup completed")

def signal_handler(sig, frame):
    """Handle Ctrl+C gracefully."""
    print('\n\n⏹️  Demo interrupted by user')
    sys.exit(0)

if __name__ == "__main__":
    # Handle Ctrl+C gracefully
    signal.signal(signal.SIGINT, signal_handler)
    
    print("🔥 MerkleKV Replication Demo")
    print("Press Ctrl+C to stop at any time")
    print()
    
    # Run the demo
    asyncio.run(demo_replication())
