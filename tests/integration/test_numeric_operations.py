"""
Numeric operations integration tests for MerkleKV.

Tests the numeric operations functionality:
- INC command
- DEC command
- APPEND command
- PREPEND command
- Error handling for numeric operations
"""

import pytest
from conftest import MerkleKVClient

class TestNumericOperations:
    """Test numeric operations (increment and decrement)."""
    
    def test_increment_new_key(self, connected_client: MerkleKVClient):
        """Test incrementing a new key (should create with value 1)."""
        # Delete the key first to ensure it doesn't exist
        connected_client.delete("counter1")
        
        # Increment a non-existent key
        response = connected_client.increment("counter1")
        assert response == "VALUE 1"
        
        # Verify the key was created with value 1
        response = connected_client.get("counter1")
        assert response == "VALUE 1"
    
    def test_increment_with_amount(self, connected_client: MerkleKVClient):
        """Test incrementing with a specific amount."""
        # Set initial value
        connected_client.set("counter2", "10")
        
        # Increment by 5
        response = connected_client.increment("counter2", 5)
        assert response == "VALUE 15"
        
        # Verify the new value
        response = connected_client.get("counter2")
        assert response == "VALUE 15"
    
    def test_increment_negative_amount(self, connected_client: MerkleKVClient):
        """Test incrementing with a negative amount (effectively decrementing)."""
        # Set initial value
        connected_client.set("counter3", "10")
        
        # Increment by -3
        response = connected_client.increment("counter3", -3)
        assert response == "VALUE 7"
        
        # Verify the new value
        response = connected_client.get("counter3")
        assert response == "VALUE 7"
    
    def test_increment_non_numeric_value(self, connected_client: MerkleKVClient):
        """Test incrementing a non-numeric value (should return error)."""
        # Set a non-numeric value
        connected_client.set("text_key", "hello")
        
        # Try to increment
        response = connected_client.increment("text_key")
        assert "ERROR" in response
        assert "not a valid number" in response
        
        # Verify the value was not changed
        response = connected_client.get("text_key")
        assert response == "VALUE hello"
    
    def test_decrement_new_key(self, connected_client: MerkleKVClient):
        """Test decrementing a new key (should create with value -1)."""
        # Delete the key first to ensure it doesn't exist
        connected_client.delete("counter4")
        
        # Decrement a non-existent key
        response = connected_client.decrement("counter4")
        assert response == "VALUE -1"
        
        # Verify the key was created with value -1
        response = connected_client.get("counter4")
        assert response == "VALUE -1"
    
    def test_decrement_with_amount(self, connected_client: MerkleKVClient):
        """Test decrementing with a specific amount."""
        # Set initial value
        connected_client.set("counter5", "20")
        
        # Decrement by 7
        response = connected_client.decrement("counter5", 7)
        assert response == "VALUE 13"
        
        # Verify the new value
        response = connected_client.get("counter5")
        assert response == "VALUE 13"
    
    def test_decrement_below_zero(self, connected_client: MerkleKVClient):
        """Test decrementing a value below zero."""
        # Set initial value
        connected_client.set("counter6", "5")
        
        # Decrement by 10
        response = connected_client.decrement("counter6", 10)
        assert response == "VALUE -5"
        
        # Verify the new value
        response = connected_client.get("counter6")
        assert response == "VALUE -5"
    
    def test_decrement_non_numeric_value(self, connected_client: MerkleKVClient):
        """Test decrementing a non-numeric value (should return error)."""
        # Set a non-numeric value
        connected_client.set("text_key2", "world")
        
        # Try to decrement
        response = connected_client.decrement("text_key2")
        assert "ERROR" in response
        assert "not a valid number" in response
        
        # Verify the value was not changed
        response = connected_client.get("text_key2")
        assert response == "VALUE world"
    
    def test_multiple_increment_operations(self, connected_client: MerkleKVClient):
        """Test multiple increment operations on the same key."""
        # Create a new counter
        connected_client.set("multi_counter", "0")
        
        # Perform multiple increments
        connected_client.increment("multi_counter")  # +1
        connected_client.increment("multi_counter", 5)  # +5
        connected_client.increment("multi_counter", 10)  # +10
        
        # Verify final value (0 + 1 + 5 + 10 = 16)
        response = connected_client.get("multi_counter")
        assert response == "VALUE 16"
    
    def test_mixed_increment_decrement(self, connected_client: MerkleKVClient):
        """Test mixing increment and decrement operations."""
        # Create a new counter
        connected_client.set("mixed_counter", "0")
        
        # Mix increment and decrement operations
        connected_client.increment("mixed_counter", 10)  # +10
        connected_client.decrement("mixed_counter", 3)   # -3
        connected_client.increment("mixed_counter", 5)   # +5
        connected_client.decrement("mixed_counter", 7)   # -7
        
        # Verify final value (0 + 10 - 3 + 5 - 7 = 5)
        response = connected_client.get("mixed_counter")
        assert response == "VALUE 5"
    
    def test_increment_large_values(self, connected_client: MerkleKVClient):
        """Test incrementing with large values."""
        # Set a large initial value
        connected_client.set("large_counter", "1000000")
        
        # Increment by a large amount
        response = connected_client.increment("large_counter", 9000000)
        assert response == "VALUE 10000000"
        
        # Verify the new value
        response = connected_client.get("large_counter")
        assert response == "VALUE 10000000"
    
    def test_invalid_increment_commands(self, connected_client: MerkleKVClient):
        """Test handling of invalid increment commands."""
        # Test INC without key
        response = connected_client.send_command("INC")
        assert "ERROR" in response
        
        # Test INC with invalid amount
        response = connected_client.send_command("INC counter abc")
        assert "ERROR" in response
    
    def test_invalid_decrement_commands(self, connected_client: MerkleKVClient):
        """Test handling of invalid decrement commands."""
        # Test DEC without key
        response = connected_client.send_command("DEC")
        assert "ERROR" in response
        
        # Test DEC with invalid amount
        response = connected_client.send_command("DEC counter abc")
        assert "ERROR" in response


class TestStringOperations:
    """Test string operations (append and prepend)."""
    
    def test_append_to_existing_key(self, connected_client: MerkleKVClient):
        """Test appending to an existing key."""
        # Set initial value
        connected_client.set("greeting", "Hello")
        
        # Append to the value
        response = connected_client.append("greeting", " World!")
        assert response == "VALUE Hello World!"
        
        # Verify the new value
        response = connected_client.get("greeting")
        assert response == "VALUE Hello World!"
    
    def test_append_to_new_key(self, connected_client: MerkleKVClient):
        """Test appending to a new key (should create with the value)."""
        # Delete the key first to ensure it doesn't exist
        connected_client.delete("new_greeting")
        
        # Create the key first
        connected_client.set("new_greeting", "World!")
        
        # Append to the key
        response = connected_client.append("new_greeting", "!")
        assert response == "VALUE World!!"
        
        # Verify the key was created with the value
        response = connected_client.get("new_greeting")
        assert response == "VALUE World!!"
    
    def test_prepend_to_existing_key(self, connected_client: MerkleKVClient):
        """Test prepending to an existing key."""
        # Set initial value
        connected_client.set("greeting2", "World!")
        
        # Prepend to the value
        response = connected_client.prepend("greeting2", "Hello ")
        assert response == "VALUE HelloWorld!"
        
        # Verify the new value
        response = connected_client.get("greeting2")
        assert response == "VALUE HelloWorld!"
    
    def test_prepend_to_new_key(self, connected_client: MerkleKVClient):
        """Test prepending to a new key (should create with the value)."""
        # Delete the key first to ensure it doesn't exist
        connected_client.delete("new_greeting2")
        
        # Create the key first
        connected_client.set("new_greeting2", "World!")
        
        # Prepend to the key
        response = connected_client.prepend("new_greeting2", "Hello")
        assert response == "VALUE HelloWorld!"
        
        # Verify the key was created with the value
        response = connected_client.get("new_greeting2")
        assert response == "VALUE HelloWorld!"
    
    def test_multiple_append_operations(self, connected_client: MerkleKVClient):
        """Test multiple append operations on the same key."""
        # Create a new string
        connected_client.set("multi_append", "Start:")
        
        # Perform multiple appends
        connected_client.append("multi_append", " Part1")
        connected_client.append("multi_append", " Part2")
        connected_client.append("multi_append", " Part3")
        
        # Verify final value
        response = connected_client.get("multi_append")
        assert response == "VALUE Start: Part1 Part2 Part3"
    
    def test_multiple_prepend_operations(self, connected_client: MerkleKVClient):
        """Test multiple prepend operations on the same key."""
        # Create a new string
        connected_client.set("multi_prepend", "End")
        
        # Perform multiple prepends (note: prepends are added in reverse order)
        connected_client.prepend("multi_prepend", "Part3 ")
        connected_client.prepend("multi_prepend", "Part2 ")
        connected_client.prepend("multi_prepend", "Part1 ")
        
        # Verify final value
        response = connected_client.get("multi_prepend")
        assert response == "VALUE Part1Part2Part3End"
    
    def test_mixed_append_prepend(self, connected_client: MerkleKVClient):
        """Test mixing append and prepend operations."""
        # Create a new string
        connected_client.set("mixed_string", "Middle")
        
        # Mix append and prepend operations
        connected_client.prepend("mixed_string", "Start ")
        connected_client.append("mixed_string", " End")
        
        # Verify final value
        response = connected_client.get("mixed_string")
        assert response == "VALUE StartMiddle End"
    
    def test_append_prepend_to_numeric_values(self, connected_client: MerkleKVClient):
        """Test appending and prepending to numeric values."""
        # Set a numeric value
        connected_client.set("num_key", "123")
        
        # Append to the numeric value
        response = connected_client.append("num_key", "456")
        assert response == "VALUE 123456"
        
        # Prepend to the result
        response = connected_client.prepend("num_key", "000")
        assert response == "VALUE 000123456"
        
        # Verify the final value
        response = connected_client.get("num_key")
        assert response == "VALUE 000123456"
    
    def test_append_prepend_empty_strings(self, connected_client: MerkleKVClient):
        """Test appending and prepending empty strings."""
        # Set initial value
        connected_client.set("empty_test", "value")
        
        # Append empty string
        response = connected_client.append("empty_test", "")
        assert response == "VALUE value\"\""
        
        # Prepend empty string
        response = connected_client.prepend("empty_test", "")
        assert response == "VALUE \"\"value\"\""
        
        # Verify the value with quotes
        response = connected_client.get("empty_test")
        assert response == "VALUE \"\"value\"\""
    
    def test_invalid_append_commands(self, connected_client: MerkleKVClient):
        """Test handling of invalid append commands."""
        # Test APPEND without key
        response = connected_client.send_command("APPEND")
        assert "ERROR" in response
        
        # Test APPEND without value
        response = connected_client.send_command("APPEND key")
        assert "ERROR" in response
    
    def test_invalid_prepend_commands(self, connected_client: MerkleKVClient):
        """Test handling of invalid prepend commands."""
        # Test PREPEND without key
        response = connected_client.send_command("PREPEND")
        assert "ERROR" in response
        
        # Test PREPEND without value
        response = connected_client.send_command("PREPEND key")
        assert "ERROR" in response
