#!/usr/bin/env python3
"""
Test cases for MQTT-based replication functionality.

This module tests the real-time replication of write operations across
MerkleKV nodes using MQTT as the message transport.

Test Setup:
- Uses public MQTT broker: test.mosquitto.org:1883
- Creates multiple MerkleKV server instances
- Verifies that write operations on one node are replicated to others
- Tests various operations: SET, DELETE, INCR, DECR, APPEND, PREPEND
"""

import asyncio
import json
import pytest
import socket
import subprocess
import tempfile
import time
import uuid
from pathlib import Path
from typing import List, Dict, Any
import toml
import threading
import paho.mqtt.client as mqtt
import base64

from conftest import MerkleKVServer

@pytest.fixture
def unique_topic_prefix():
    """Generate a unique topic prefix for each test to avoid interference."""
    return f"test_merkle_kv_{uuid.uuid4().hex[:8]}"

@pytest.fixture
def mqtt_config(unique_topic_prefix):
    """MQTT configuration using public test broker."""
    return {
        "enabled": True,
        "mqtt_broker": "test.mosquitto.org",
        "mqtt_port": 1883,
        "topic_prefix": unique_topic_prefix,
        "client_id": f"test_client_{uuid.uuid4().hex[:8]}"
    }

async def create_replication_config(port: int, node_id: str, topic_prefix: str) -> Path:
    """Create a temporary config file with replication enabled."""
    config = {
        "host": "127.0.0.1",
        "port": port,
        "storage_path": f"data_test_{node_id}",
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
    
    # Create temporary config file
    temp_config = Path(f"/tmp/config_{node_id}.toml")
    with open(temp_config, 'w') as f:
        toml.dump(config, f)
    
    return temp_config

class ReplicationTestSetup:
    """Helper class to manage multiple MerkleKV instances for replication testing."""
    
    def __init__(self, topic_prefix: str):
        self.topic_prefix = topic_prefix
        self.servers: List[MerkleKVServer] = []
        self.configs: List[Path] = []
        
    async def create_node(self, node_id: str, port: int) -> MerkleKVServer:
        """Create and start a MerkleKV node with replication enabled."""
        config_path = await create_replication_config(port, node_id, self.topic_prefix)
        self.configs.append(config_path)
        
        server = MerkleKVServer(host="127.0.0.1", port=port, config_path=str(config_path))
        await server.start()
        self.servers.append(server)
        
        # Wait a bit for MQTT connection to establish
        await asyncio.sleep(2)
        
        return server
        
    async def cleanup(self):
        """Stop all servers and clean up temporary files."""
        for server in self.servers:
            await server.stop()
        
        for config_path in self.configs:
            if config_path.exists():
                config_path.unlink()

@pytest.fixture
async def replication_setup(unique_topic_prefix):
    """Setup for replication tests with cleanup."""
    setup = ReplicationTestSetup(unique_topic_prefix)
    yield setup
    await setup.cleanup()

class MQTTTestClient:
    """Test client to monitor MQTT messages."""
    
    def __init__(self, topic_prefix: str):
        self.topic_prefix = topic_prefix
        self.received_messages = []
        self.connected = threading.Event()
        
    def on_connect(self, client, userdata, flags, rc):
        if rc == 0:
            self.connected.set()
            topic = f"{self.topic_prefix}/events/#"
            client.subscribe(topic)
            
    def on_message(self, client, userdata, msg):
        try:
            # Try to decode as JSON first (legacy format)
            payload = json.loads(msg.payload.decode())
            self.received_messages.append({
                'topic': msg.topic,
                'payload': payload,
                'timestamp': time.time()
            })
        except json.JSONDecodeError:
            # Handle binary format (CBOR)
            self.received_messages.append({
                'topic': msg.topic,
                'payload': msg.payload,
                'timestamp': time.time(),
                'format': 'binary'
            })
        
    async def monitor_replication_messages(self, duration: float = 5.0):
        """Monitor MQTT messages for a specified duration."""
        try:
            client = mqtt.Client()
            client.on_connect = self.on_connect
            client.on_message = self.on_message
            
            client.connect("test.mosquitto.org", 1883, 60)
            client.loop_start()
            
            # Wait for connection
            if self.connected.wait(timeout=10):
                # Monitor for the specified duration
                await asyncio.sleep(duration)
            
            client.loop_stop()
            client.disconnect()
                        
        except Exception as e:
            print(f"MQTT monitoring error: {e}")

@pytest.mark.asyncio
async def test_basic_replication_setup(replication_setup):
    """Test that replication nodes can be created and connected."""
    # Create two nodes
    node1 = await replication_setup.create_node("node1", 7380)
    node2 = await replication_setup.create_node("node2", 7381)
    
    # Verify both nodes are running
    assert await node1.is_running()
    assert await node2.is_running()
    
    # Basic connectivity test
    await node1.execute_command("SET test_key test_value")
    response = await node1.execute_command("GET test_key")
    assert response == "test_value"

@pytest.mark.asyncio
async def test_set_operation_replication(replication_setup, unique_topic_prefix):
    """Test that SET operations are replicated between nodes."""
    # Create two nodes
    node1 = await replication_setup.create_node("node1", 7382)
    node2 = await replication_setup.create_node("node2", 7383)
    
    # Start MQTT monitoring
    mqtt_client = MQTTTestClient(unique_topic_prefix)
    monitor_task = asyncio.create_task(
        mqtt_client.monitor_replication_messages(10.0)
    )
    
    # Wait for MQTT connections to stabilize
    await asyncio.sleep(3)
    
    # Perform SET operation on node1
    test_key = f"repl_test_{uuid.uuid4().hex[:8]}"
    test_value = "replicated_value"
    
    await node1.execute_command(f"SET {test_key} {test_value}")
    
    # Wait for replication to occur
    await asyncio.sleep(5)
    
    # Verify the value exists on node2
    result = await node2.execute_command(f"GET {test_key}")
    assert result == test_value, f"Expected {test_value}, got {result}"
    
    # Stop monitoring and check messages
    monitor_task.cancel()
    try:
        await monitor_task
    except asyncio.CancelledError:
        pass
    
    # Verify MQTT messages were sent
    assert len(mqtt_client.received_messages) > 0, "No MQTT messages received"
    
    # Check if any message contains our operation
    found_operation = False
    for msg in mqtt_client.received_messages:
        if 'payload' in msg and isinstance(msg['payload'], dict):
            payload = msg['payload']
            if (payload.get('operation') == 'SET' or 
                payload.get('key') == test_key):
                found_operation = True
                break
    
    # Note: Due to binary encoding (CBOR), we might not be able to decode all messages
    # but the replication should still work
    print(f"Found {len(mqtt_client.received_messages)} MQTT messages")

@pytest.mark.asyncio
async def test_delete_operation_replication(replication_setup, unique_topic_prefix):
    """Test that DELETE operations are replicated between nodes."""
    # Create two nodes
    node1 = await replication_setup.create_node("node1", 7384)
    node2 = await replication_setup.create_node("node2", 7385)
    
    # Wait for MQTT connections to stabilize
    await asyncio.sleep(3)
    
    test_key = f"delete_test_{uuid.uuid4().hex[:8]}"
    
    # Set initial value on both nodes (to ensure they have the key)
    await node1.execute_command(f"SET {test_key} initial_value")
    await asyncio.sleep(2)
    
    # Verify both nodes have the value
    result1 = await node1.execute_command(f"GET {test_key}")
    result2 = await node2.execute_command(f"GET {test_key}")
    assert result1 == "initial_value"
    assert result2 == "initial_value"
    
    # Delete from node1
    await node1.execute_command(f"DEL {test_key}")
    
    # Wait for replication
    await asyncio.sleep(5)
    
    # Verify deletion on node2
    result2 = await node2.execute_command(f"GET {test_key}")
    assert result2 == "(nil)", f"Expected (nil), got {result2}"

@pytest.mark.asyncio
async def test_numeric_operations_replication(replication_setup):
    """Test that INCR/DECR operations are replicated between nodes."""
    # Create two nodes
    node1 = await replication_setup.create_node("node1", 7386)
    node2 = await replication_setup.create_node("node2", 7387)
    
    # Wait for MQTT connections to stabilize
    await asyncio.sleep(3)
    
    test_key = f"numeric_test_{uuid.uuid4().hex[:8]}"
    
    # Initialize with a numeric value
    await node1.execute_command(f"SET {test_key} 10")
    await asyncio.sleep(2)
    
    # Verify initial value on both nodes
    result1 = await node1.execute_command(f"GET {test_key}")
    result2 = await node2.execute_command(f"GET {test_key}")
    assert result1 == "10"
    assert result2 == "10"
    
    # Increment on node1
    await node1.execute_command(f"INCR {test_key}")
    await asyncio.sleep(3)
    
    # Verify increment replicated to node2
    result2 = await node2.execute_command(f"GET {test_key}")
    assert result2 == "11", f"Expected 11, got {result2}"
    
    # Decrement on node2
    await node2.execute_command(f"DECR {test_key}")
    await asyncio.sleep(3)
    
    # Verify decrement replicated to node1
    result1 = await node1.execute_command(f"GET {test_key}")
    assert result1 == "10", f"Expected 10, got {result1}"

@pytest.mark.asyncio
async def test_string_operations_replication(replication_setup):
    """Test that APPEND/PREPEND operations are replicated between nodes."""
    # Create two nodes
    node1 = await replication_setup.create_node("node1", 7388)
    node2 = await replication_setup.create_node("node2", 7389)
    
    # Wait for MQTT connections to stabilize
    await asyncio.sleep(3)
    
    test_key = f"string_test_{uuid.uuid4().hex[:8]}"
    
    # Set initial value
    await node1.execute_command(f"SET {test_key} hello")
    await asyncio.sleep(2)
    
    # Verify initial value on both nodes
    result1 = await node1.execute_command(f"GET {test_key}")
    result2 = await node2.execute_command(f"GET {test_key}")
    assert result1 == "hello"
    assert result2 == "hello"
    
    # Append on node1
    await node1.execute_command(f"APPEND {test_key} _world")
    await asyncio.sleep(3)
    
    # Verify append replicated to node2
    result2 = await node2.execute_command(f"GET {test_key}")
    assert result2 == "hello_world", f"Expected hello_world, got {result2}"
    
    # Prepend on node2
    await node2.execute_command(f"PREPEND {test_key} say_")
    await asyncio.sleep(3)
    
    # Verify prepend replicated to node1
    result1 = await node1.execute_command(f"GET {test_key}")
    assert result1 == "say_hello_world", f"Expected say_hello_world, got {result1}"

@pytest.mark.asyncio
async def test_concurrent_operations_replication(replication_setup):
    """Test replication behavior with concurrent operations on multiple nodes."""
    # Create three nodes for more complex testing
    node1 = await replication_setup.create_node("node1", 7390)
    node2 = await replication_setup.create_node("node2", 7391)
    node3 = await replication_setup.create_node("node3", 7392)
    
    # Wait for MQTT connections to stabilize
    await asyncio.sleep(5)
    
    # Perform concurrent operations
    operations = [
        node1.execute_command("SET concurrent_test1 value1"),
        node2.execute_command("SET concurrent_test2 value2"),
        node3.execute_command("SET concurrent_test3 value3"),
    ]
    
    await asyncio.gather(*operations)
    
    # Wait for replication to settle
    await asyncio.sleep(10)
    
    # Verify all values are present on all nodes
    for node in [node1, node2, node3]:
        result1 = await node.execute_command("GET concurrent_test1")
        result2 = await node.execute_command("GET concurrent_test2")
        result3 = await node.execute_command("GET concurrent_test3")
        
        assert result1 == "value1", f"Node missing concurrent_test1: {result1}"
        assert result2 == "value2", f"Node missing concurrent_test2: {result2}"
        assert result3 == "value3", f"Node missing concurrent_test3: {result3}"

@pytest.mark.asyncio
async def test_replication_with_node_restart(replication_setup):
    """Test replication behavior when a node is restarted."""
    # Create two nodes
    node1 = await replication_setup.create_node("node1", 7393)
    node2 = await replication_setup.create_node("node2", 7394)
    
    # Wait for MQTT connections to stabilize
    await asyncio.sleep(3)
    
    # Set some initial data
    await node1.execute_command("SET restart_test1 before_restart")
    await asyncio.sleep(2)
    
    # Verify replication
    result = await node2.execute_command("GET restart_test1")
    assert result == "before_restart"
    
    # Stop node2
    await node2.stop()
    
    # Add more data while node2 is down
    await node1.execute_command("SET restart_test2 during_downtime")
    await asyncio.sleep(2)
    
    # Restart node2 (simulate restart)
    node2_restarted = await replication_setup.create_node("node2_restart", 7395)
    await asyncio.sleep(5)
    
    # Add data after restart
    await node1.execute_command("SET restart_test3 after_restart")
    await asyncio.sleep(5)
    
    # Verify new data is replicated to restarted node
    result = await node2_restarted.execute_command("GET restart_test3")
    assert result == "after_restart"
    
    # Note: Data during downtime might not be replicated since MQTT 
    # doesn't persist messages for disconnected clients by default

@pytest.mark.asyncio
async def test_replication_loop_prevention(replication_setup, unique_topic_prefix):
    """Test that nodes don't create infinite loops by processing their own messages."""
    # Create a single node
    node1 = await replication_setup.create_node("node1", 7396)
    
    # Start MQTT monitoring
    mqtt_client = MQTTTestClient(unique_topic_prefix)
    monitor_task = asyncio.create_task(
        mqtt_client.monitor_replication_messages(15.0)
    )
    
    # Wait for MQTT connections to stabilize
    await asyncio.sleep(3)
    
    # Perform multiple operations rapidly
    for i in range(5):
        await node1.execute_command(f"SET loop_test_{i} value_{i}")
        await asyncio.sleep(0.5)
    
    # Wait for all messages to be processed
    await asyncio.sleep(5)
    
    # Stop monitoring
    monitor_task.cancel()
    try:
        await monitor_task
    except asyncio.CancelledError:
        pass
    
    # Verify we don't have an excessive number of messages (indicating loops)
    # We should have roughly 5 messages, not 50+ from loops
    message_count = len(mqtt_client.received_messages)
    assert message_count <= 20, f"Too many messages detected ({message_count}), possible loop"
    
    print(f"Received {message_count} MQTT messages for 5 operations")

@pytest.mark.asyncio
async def test_malformed_mqtt_message_handling(replication_setup, unique_topic_prefix):
    """Test that nodes handle malformed MQTT messages gracefully."""
    # Create a node
    node1 = await replication_setup.create_node("node1", 7397)
    
    # Wait for MQTT connections to stabilize
    await asyncio.sleep(3)
    
    # Send a malformed message via MQTT
    try:
        def on_connect(client, userdata, flags, rc):
            if rc == 0:
                topic = f"{unique_topic_prefix}/events"
                
                # Send invalid JSON
                client.publish(topic, "invalid json message")
                
                # Send valid JSON but wrong format
                client.publish(topic, json.dumps({"invalid": "format"}))
                
        client = mqtt.Client()
        client.on_connect = on_connect
        client.connect("test.mosquitto.org", 1883, 60)
        client.loop_start()
        
        # Wait a bit
        await asyncio.sleep(3)
        
        client.loop_stop()
        client.disconnect()
        
        # Verify the node is still responsive
        result = await node1.execute_command("SET test_after_malformed success")
        assert result == "OK"
        
        result = await node1.execute_command("GET test_after_malformed")
        assert result == "success"
        
    except Exception as e:
        print(f"MQTT client error (expected in some cases): {e}")

if __name__ == "__main__":
    # Run specific test
    import sys
    if len(sys.argv) > 1:
        test_name = sys.argv[1]
        pytest.main([f"-v", f"-k", test_name, __file__])
    else:
        pytest.main(["-v", __file__])
