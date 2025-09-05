#!/usr/bin/env python3
"""
Very simple test without replication to verify basic server functionality.
"""

import asyncio
import pytest
import subprocess
import time
from pathlib import Path
import toml
import os

def create_simple_config(port: int, node_id: str) -> Path:
    """Create a test config file without replication."""
    config = {
        "host": "127.0.0.1",
        "port": port,
        "storage_path": f"test_data_{node_id}",
        "engine": "rwlock",
        "sync_interval_seconds": 60,
        "replication": {
            "enabled": False,  # Disable replication for now
            "mqtt_broker": "localhost",
            "mqtt_port": 1883,
            "topic_prefix": "test_disabled",
            "client_id": node_id
        }
    }
    
    temp_config = Path(f"/tmp/test_config_simple_{node_id}_{port}.toml")
    with open(temp_config, 'w') as f:
        toml.dump(config, f)
    
    return temp_config

async def start_simple_server(config_path: Path, timeout: int = 20) -> subprocess.Popen:
    """Start a MerkleKV server with the given config."""
    cmd = ["cargo", "run", "--", "--config", str(config_path)]
    print(f"Starting simple server: {' '.join(cmd)}")
    
    # Get project root
    project_root = Path.cwd()
    if "tests" in str(project_root):
        project_root = project_root.parent.parent
    
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=project_root,
        env={**os.environ, "RUST_LOG": "info"}
    )
    
    # Wait for server to start
    import socket
    start_time = time.time()
    
    # Extract port from config
    with open(config_path) as f:
        config_data = toml.load(f)
        port = config_data["port"]
    
    while time.time() - start_time < timeout:
        if process.poll() is not None:
            stdout, stderr = process.communicate()
            raise RuntimeError(f"Server failed to start: {stderr.decode()}")
        
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=1):
                print(f"✅ Simple server started on port {port}")
                return process
        except (socket.timeout, ConnectionRefusedError):
            await asyncio.sleep(0.5)
    
    process.terminate()
    raise TimeoutError(f"Simple server failed to start within {timeout} seconds")

async def execute_simple_command(host: str, port: int, command: str) -> str:
    """Execute a command on the server."""
    reader, writer = await asyncio.open_connection(host, port)
    
    try:
        writer.write(f"{command}\r\n".encode())
        await writer.drain()
        
        data = await reader.read(1024)
        return data.decode().strip()
    finally:
        writer.close()
        await writer.wait_closed()

@pytest.mark.asyncio 
async def test_simple_server_without_replication():
    """Test that server can start and handle basic commands without replication."""
    # Create config without replication
    config = create_simple_config(7450, "simple_node")
    
    server = None
    
    try:
        # Start server
        print("Starting simple server without replication...")
        server = await start_simple_server(config)
        
        # Wait a bit for server to be ready
        await asyncio.sleep(2)
        
        # Test basic commands
        result = await execute_simple_command("127.0.0.1", 7450, "SET test_key test_value")
        print(f"SET result: {result}")
        assert result == "OK"
        
        result = await execute_simple_command("127.0.0.1", 7450, "GET test_key")
        print(f"GET result: {result}")
        assert result == "VALUE test_value"
        
        result = await execute_simple_command("127.0.0.1", 7450, "DEL test_key")
        print(f"DELETE result: {result}")
        assert result == "DELETED"  # Key exists, so expect DELETED
        
        result = await execute_simple_command("127.0.0.1", 7450, "GET test_key")
        print(f"GET after DELETE result: {result}")
        assert result == "NOT_FOUND"
        
        print("✅ Simple server test passed!")
        
    except Exception as e:
        print(f"❌ Simple server test failed: {e}")
        raise
    finally:
        # Cleanup
        if server:
            server.terminate()
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()
        
        # Clean up config file
        if config.exists():
            config.unlink()

if __name__ == "__main__":
    pytest.main(["-v", __file__])
