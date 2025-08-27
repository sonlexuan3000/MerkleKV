k"""
Integration tests for bulk operations in MerkleKV.

This module tests the bulk operations functionality:
- MGET - Get multiple keys in one command
- MSET - Set multiple key-value pairs
- TRUNCATE - Clear all keys/values in the store
"""

import pytest
from conftest import connect_to_server


def test_mget_single_key(server_process):
    """Test MGET with a single key."""
    client = connect_to_server()
    
    # Set a key
    client.send(b"SET key1 value1\r\n")
    assert client.recv(1024) == b"OK\r\n"
     
    # Get the key using MGET
    client.send(b"MGET key1\r\n")
    response = client.recv(1024)
    assert response == b"VALUES 1\r\nkey1 value1\r\n"
    
    client.close()


def test_mget_multiple_keys(server_process):
    """Test MGET with multiple keys."""
    client = connect_to_server()
    
    # Set multiple keys
    client.send(b"SET key1 value1\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    client.send(b"SET key2 value2\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    client.send(b"SET key3 value3\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Get multiple keys using MGET
    client.send(b"MGET key1 key2 key3\r\n")
    response = client.recv(1024)
    assert b"VALUES 3\r\n" in response
    assert b"key1 value1\r\n" in response
    assert b"key2 value2\r\n" in response
    assert b"key3 value3\r\n" in response
    
    client.close()


def test_mget_nonexistent_keys(server_process):
    """Test MGET with nonexistent keys."""
    client = connect_to_server()
    
    # Set a key
    client.send(b"SET key1 value1\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Get existing and nonexistent keys using MGET
    client.send(b"MGET key1 nonexistent1 nonexistent2\r\n")
    response = client.recv(1024)
    assert b"VALUES 1\r\n" in response
    assert b"key1 value1\r\n" in response
    assert b"nonexistent1 NOT_FOUND\r\n" in response
    assert b"nonexistent2 NOT_FOUND\r\n" in response
    
    client.close()


def test_mget_all_nonexistent_keys(server_process):
    """Test MGET with all nonexistent keys."""
    client = connect_to_server()
    
    # Get nonexistent keys using MGET
    client.send(b"MGET nonexistent1 nonexistent2\r\n")
    response = client.recv(1024)
    assert response == b"NOT_FOUND\r\n"
    
    client.close()


def test_mset_single_pair(server_process):
    """Test MSET with a single key-value pair."""
    client = connect_to_server()
    
    # Set a key-value pair using MSET
    client.send(b"MSET key1 value1\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Verify the key was set
    client.send(b"GET key1\r\n")
    assert client.recv(1024) == b"VALUE value1\r\n"
    
    client.close()


def test_mset_multiple_pairs(server_process):
    """Test MSET with multiple key-value pairs."""
    client = connect_to_server()
    
    # Set multiple key-value pairs using MSET
    client.send(b"MSET key1 value1 key2 value2 key3 value3\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Verify the keys were set
    client.send(b"GET key1\r\n")
    assert client.recv(1024) == b"VALUE value1\r\n"
    
    client.send(b"GET key2\r\n")
    assert client.recv(1024) == b"VALUE value2\r\n"
    
    client.send(b"GET key3\r\n")
    assert client.recv(1024) == b"VALUE value3\r\n"
    
    client.close()


def test_mset_overwrite_existing(server_process):
    """Test MSET overwriting existing keys."""
    client = connect_to_server()
    
    # Set initial values
    client.send(b"SET key1 initial1\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    client.send(b"SET key2 initial2\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Overwrite with MSET
    client.send(b"MSET key1 updated1 key2 updated2 key3 value3\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Verify the keys were updated
    client.send(b"GET key1\r\n")
    assert client.recv(1024) == b"VALUE updated1\r\n"
    
    client.send(b"GET key2\r\n")
    assert client.recv(1024) == b"VALUE updated2\r\n"
    
    client.send(b"GET key3\r\n")
    assert client.recv(1024) == b"VALUE value3\r\n"
    
    client.close()


def test_truncate(server_process):
    """Test TRUNCATE command."""
    client = connect_to_server()
    
    # Set multiple keys
    client.send(b"SET key1 value1\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    client.send(b"SET key2 value2\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    client.send(b"SET key3 value3\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Verify keys exist
    client.send(b"GET key1\r\n")
    assert client.recv(1024) == b"VALUE value1\r\n"
    
    # Truncate the store
    client.send(b"TRUNCATE\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Verify keys no longer exist
    client.send(b"GET key1\r\n")
    assert client.recv(1024) == b"NOT_FOUND\r\n"
    
    client.send(b"GET key2\r\n")
    assert client.recv(1024) == b"NOT_FOUND\r\n"
    
    client.send(b"GET key3\r\n")
    assert client.recv(1024) == b"NOT_FOUND\r\n"
    
    # Set a new key after truncate
    client.send(b"SET new_key new_value\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    client.send(b"GET new_key\r\n")
    assert client.recv(1024) == b"VALUE new_value\r\n"
    
    client.close()


def test_bulk_operations_combined(server_process):
    """Test a combination of bulk operations."""
    client = connect_to_server()
    
    # Set multiple keys with MSET
    client.send(b"MSET key1 value1 key2 value2 key3 value3\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Get multiple keys with MGET
    client.send(b"MGET key1 key2 key3\r\n")
    response = client.recv(1024)
    assert b"VALUES 3\r\n" in response
    assert b"key1 value1\r\n" in response
    assert b"key2 value2\r\n" in response
    assert b"key3 value3\r\n" in response
    
    # Truncate the store
    client.send(b"TRUNCATE\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Verify all keys are gone
    client.send(b"MGET key1 key2 key3\r\n")
    assert client.recv(1024) == b"NOT_FOUND\r\n"
    
    # Set new keys after truncate
    client.send(b"MSET new1 val1 new2 val2\r\n")
    assert client.recv(1024) == b"OK\r\n"
    
    # Verify new keys exist
    client.send(b"MGET new1 new2\r\n")
    response = client.recv(1024)
    assert b"VALUES 2\r\n" in response
    assert b"new1 val1\r\n" in response
    assert b"new2 val2\r\n" in response
    
    client.close()
