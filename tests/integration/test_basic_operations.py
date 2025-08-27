"""
Basic operations integration tests for MerkleKV.

Tests the core functionality:
- GET command
- SET command  
- DELETE command
- Error handling
- Response format validation
"""

import pytest
from conftest import MerkleKVClient, generate_test_data

class TestBasicOperations:
    """Test basic key-value operations."""
    
    def test_set_and_get_single_key(self, connected_client: MerkleKVClient):
        """Test setting and getting a single key-value pair."""
        # Set a key-value pair
        response = connected_client.set("test_key", "test_value")
        assert response == "OK"
        
        # Get the value back
        response = connected_client.get("test_key")
        assert response == "VALUE test_value"
    
    def test_get_nonexistent_key(self, connected_client: MerkleKVClient):
        """Test getting a key that doesn't exist."""
        response = connected_client.get("nonexistent_key")
        assert response == "NOT_FOUND"
    
    def test_set_overwrite_existing_key(self, connected_client: MerkleKVClient):
        """Test that setting a key overwrites the existing value."""
        # Set initial value
        connected_client.set("overwrite_key", "initial_value")
        assert connected_client.get("overwrite_key") == "VALUE initial_value"
        
        # Overwrite with new value
        connected_client.set("overwrite_key", "new_value")
        assert connected_client.get("overwrite_key") == "VALUE new_value"
    
    def test_delete_existing_key(self, connected_client: MerkleKVClient):
        """Test deleting an existing key."""
        # Set a key
        connected_client.set("delete_key", "delete_value")
        assert connected_client.get("delete_key") == "VALUE delete_value"
        
        # Delete the key
        response = connected_client.delete("delete_key")
        assert response == "OK"
        
        # Verify it's gone
        response = connected_client.get("delete_key")
        assert response == "NOT_FOUND"
    
    def test_delete_nonexistent_key(self, connected_client: MerkleKVClient):
        """Test deleting a key that doesn't exist."""
        response = connected_client.delete("nonexistent_key")
        assert response == "OK"  # DELETE should succeed even if key doesn't exist
    
    def test_multiple_operations_sequence(self, connected_client: MerkleKVClient):
        """Test a sequence of multiple operations."""
        # Set multiple keys
        test_data = {
            "key1": "value1",
            "key2": "value2", 
            "key3": "value3"
        }
        
        for key, value in test_data.items():
            response = connected_client.set(key, value)
            assert response == "OK"
        
        # Verify all keys exist
        for key, expected_value in test_data.items():
            response = connected_client.get(key)
            assert response == f"VALUE {expected_value}"
        
        # Delete one key
        response = connected_client.delete("key2")
        assert response == "OK"
        
        # Verify deleted key is gone
        response = connected_client.get("key2")
        assert response == "NOT_FOUND"
        
        # Verify other keys still exist
        assert connected_client.get("key1") == "VALUE value1"
        assert connected_client.get("key3") == "VALUE value3"
    
    def test_empty_key_and_value(self, connected_client: MerkleKVClient):
        """Test handling of empty keys and values."""
        # Test empty value
        response = connected_client.set("empty_value_key", "")
        assert response == "OK"
        
        response = connected_client.get("empty_value_key")
        assert response == 'VALUE ""'
        
        # Test empty key (should fail) - use a command with two spaces to create empty key
        response = connected_client.send_command("SET  value")
        assert "ERROR" in response
    
    def test_special_characters_in_keys_and_values(self, connected_client: MerkleKVClient):
        """Test handling of special characters in keys and values."""
        special_key = "key_with_spaces_and_symbols!@#$%^&*()"
        special_value = "value with spaces and symbols: !@#$%^&*()_+-=[]{}|;':\",./<>?"
        
        response = connected_client.set(special_key, special_value)
        assert response == "OK"
        
        response = connected_client.get(special_key)
        assert response == f"VALUE {special_value}"
    
    def test_large_values(self, connected_client: MerkleKVClient):
        """Test handling of large values."""
        large_value = "x" * 1000  # 1KB value
        
        response = connected_client.set("large_key", large_value)
        assert response == "OK"
        
        response = connected_client.get("large_key")
        assert response == f"VALUE {large_value}"
    
    def test_invalid_commands(self, connected_client: MerkleKVClient):
        """Test handling of invalid commands."""
        # Test unknown command
        response = connected_client.send_command("UNKNOWN_COMMAND")
        assert "ERROR" in response
        
        # Test GET without key
        response = connected_client.send_command("GET")
        assert "ERROR" in response
        
        # Test SET without value
        response = connected_client.send_command("SET key_only")
        assert "ERROR" in response
        
        # Test DELETE without key
        response = connected_client.send_command("DELETE")
        assert "ERROR" in response
        
        # Test empty command
        response = connected_client.send_command("")
        assert "ERROR" in response
    
    def test_command_case_insensitivity(self, connected_client: MerkleKVClient):
        """Test that commands are case insensitive."""
        # Test lowercase commands
        response = connected_client.send_command("set case_test value")
        assert response == "OK"
        
        response = connected_client.send_command("get case_test")
        assert response == "VALUE value"
        
        response = connected_client.send_command("delete case_test")
        assert response == "OK"
        
        # Test mixed case commands
        response = connected_client.send_command("SeT mixed_case value")
        assert response == "OK"
        
        response = connected_client.send_command("GeT mixed_case")
        assert response == "VALUE value"
        
        response = connected_client.send_command("DeLeTe mixed_case")
        assert response == "OK"
    
    def test_del_alias_for_delete(self, connected_client: MerkleKVClient):
        """Test that DEL is an alias for DELETE."""
        connected_client.set("del_test_key", "del_test_value")
        
        # Use DEL alias
        response = connected_client.send_command("DEL del_test_key")
        assert response == "OK"
        
        # Verify key is deleted
        response = connected_client.get("del_test_key")
        assert response == "NOT_FOUND"

class TestDataPersistence:
    """Test data persistence across server restarts."""
    
    def test_data_survives_server_restart(self, temp_test_dir, connected_client: MerkleKVClient):
        """Test that data persists when server is restarted."""
        # Set some data
        test_data = {
            "persistent_key1": "persistent_value1",
            "persistent_key2": "persistent_value2",
            "persistent_key3": "persistent_value3"
        }
        
        for key, value in test_data.items():
            connected_client.set(key, value)
        
        # Verify data is set
        for key, expected_value in test_data.items():
            response = connected_client.get(key)
            assert response == f"VALUE {expected_value}"
        
        # Disconnect client
        connected_client.disconnect()
        
        # Restart server with same storage path
        from conftest import MerkleKVServer
        
        server = MerkleKVServer()
        try:
            server.start(temp_test_dir)
            
            # Create new client
            new_client = MerkleKVClient()
            new_client.connect()
            
            # Verify data still exists
            for key, expected_value in test_data.items():
                response = new_client.get(key)
                assert response == f"VALUE {expected_value}"
            
            new_client.disconnect()
        finally:
            server.stop() 