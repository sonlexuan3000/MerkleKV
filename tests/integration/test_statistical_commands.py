"""
Integration tests for statistical commands (STATS, INFO, PING).

These tests verify that the server correctly implements the statistical commands
that provide monitoring and diagnostic information.
"""

import pytest
import time
from conftest import send_command, connect_to_server


def test_ping_command(server_process):
    """Test the PING command returns PONG."""
    with connect_to_server() as client:
        response = send_command(client, "PING")
        assert response == "PONG"


def test_stats_command(server_process):
    """Test the STATS command returns server statistics."""
    with connect_to_server() as client:
        # First, perform some operations to generate statistics
        send_command(client, "SET key1 value1")
        send_command(client, "GET key1")
        send_command(client, "INC counter 5")
        
        # Then get the stats
        response = send_command(client, "STATS")
        
        # Verify the response starts with STATS
        assert response.startswith("STATS")
        
        # Convert the response to a dictionary for easier checking
        stats_lines = response.strip().split("\r\n")[1:]  # Skip the "STATS" line
        stats = {}
        for line in stats_lines:
            if ":" in line:
                key, value = line.split(":", 1)
                stats[key] = value
        
        # Check that essential statistics are present
        assert "uptime_seconds" in stats
        assert "total_connections" in stats
        assert "active_connections" in stats
        assert "total_commands" in stats
        assert "get_commands" in stats
        assert "set_commands" in stats
        assert "numeric_commands" in stats
        
        # Verify some specific values
        assert int(stats["get_commands"]) >= 1  # We did at least one GET
        assert int(stats["set_commands"]) >= 1  # We did at least one SET
        assert int(stats["numeric_commands"]) >= 1  # We did at least one INC


def test_info_command(server_process):
    """Test the INFO command returns server information."""
    with connect_to_server() as client:
        response = send_command(client, "INFO")
        
        # Verify the response starts with INFO
        assert response.startswith("INFO")
        
        # Convert the response to a dictionary for easier checking
        info_lines = response.strip().split("\r\n")[1:]  # Skip the "INFO" line
        info = {}
        for line in info_lines:
            if ":" in line:
                key, value = line.split(":", 1)
                info[key] = value
        
        # Check that essential information is present
        assert "version" in info
        assert "uptime_seconds" in info
        assert "uptime" in info
        assert "server_time_unix" in info
        assert "db_keys" in info


def test_stats_updates_with_commands(server_process):
    """Test that the STATS command reflects command execution counts."""
    with connect_to_server() as client:
        # Get initial stats
        response = send_command(client, "STATS")
        initial_stats_lines = response.strip().split("\r\n")[1:]
        initial_stats = {}
        for line in initial_stats_lines:
            if ":" in line:
                key, value = line.split(":", 1)
                initial_stats[key] = value
        
        # Execute a series of commands
        num_gets = 3
        num_sets = 2
        num_deletes = 1
        
        for i in range(num_sets):
            send_command(client, f"SET key{i} value{i}")
        
        for i in range(num_gets):
            send_command(client, f"GET key{i % num_sets}")
        
        send_command(client, "DELETE key0")
        
        # Get updated stats
        response = send_command(client, "STATS")
        updated_stats_lines = response.strip().split("\r\n")[1:]
        updated_stats = {}
        for line in updated_stats_lines:
            if ":" in line:
                key, value = line.split(":", 1)
                updated_stats[key] = value
        
        # Verify command counts increased by the expected amount
        assert int(updated_stats["get_commands"]) >= int(initial_stats.get("get_commands", "0")) + num_gets
        assert int(updated_stats["set_commands"]) >= int(initial_stats.get("set_commands", "0")) + num_sets
        assert int(updated_stats["delete_commands"]) >= int(initial_stats.get("delete_commands", "0")) + num_deletes


def test_info_shows_key_count(server_process):
    """Test that the INFO command shows the correct number of keys."""
    with connect_to_server() as client:
        # Clear any existing keys
        send_command(client, "TRUNCATE")
        
        # Add a known number of keys
        num_keys = 5
        for i in range(num_keys):
            send_command(client, f"SET key{i} value{i}")
        
        # Get INFO and check key count
        response = send_command(client, "INFO")
        info_lines = response.strip().split("\r\n")[1:]
        info = {}
        for line in info_lines:
            if ":" in line:
                key, value = line.split(":", 1)
                info[key] = value
        
        assert int(info["db_keys"]) == num_keys
        
        # Delete a key and verify count decreases
        send_command(client, "DELETE key0")
        
        response = send_command(client, "INFO")
        info_lines = response.strip().split("\r\n")[1:]
        info = {}
        for line in info_lines:
            if ":" in line:
                key, value = line.split(":", 1)
                info[key] = value
        
        assert int(info["db_keys"]) == num_keys - 1


def test_uptime_increases(server_process):
    """Test that the uptime reported by STATS and INFO increases over time."""
    with connect_to_server() as client:
        # Get initial uptime
        response = send_command(client, "STATS")
        stats_lines = response.strip().split("\r\n")[1:]
        stats = {}
        for line in stats_lines:
            if ":" in line:
                key, value = line.split(":", 1)
                stats[key] = value
        
        initial_uptime = int(stats["uptime_seconds"])
        
        # Wait a short time
        time.sleep(2)
        
        # Get updated uptime
        response = send_command(client, "STATS")
        stats_lines = response.strip().split("\r\n")[1:]
        stats = {}
        for line in stats_lines:
            if ":" in line:
                key, value = line.split(":", 1)
                stats[key] = value
        
        updated_uptime = int(stats["uptime_seconds"])
        
        # Verify uptime increased
        assert updated_uptime >= initial_uptime + 1  # Allow for some timing variance
