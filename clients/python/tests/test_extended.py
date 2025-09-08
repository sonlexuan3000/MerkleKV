"""
Additional unit tests for MerkleKV Python client to achieve >90% coverage.
"""

import pytest
import socket
from unittest.mock import Mock, patch, MagicMock
from merklekv import MerkleKVClient, AsyncMerkleKVClient


class TestMerkleKVClientExtended:
    """Extended tests for MerkleKV client methods."""

    def test_pipeline_empty_commands(self):
        """Test pipeline with empty command list."""
        client = MerkleKVClient()
        result = client.pipeline([])
        assert result == []

    @patch('socket.create_connection')
    def test_pipeline_success(self, mock_connection):
        """Test successful pipeline operations."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.side_effect = [b"OK\r\n", b"VALUE test_value\r\n", b"DELETED\r\n"]
        
        client = MerkleKVClient()
        client.connect()
        
        commands = ["SET test_key test_value", "GET test_key", "DEL test_key"]
        results = client.pipeline(commands)
        
        assert len(results) == 3
        assert results[0] == "OK"
        assert results[1] == "VALUE test_value"
        assert results[2] == "DELETED"

    def test_pipeline_not_connected(self):
        """Test pipeline when not connected."""
        client = MerkleKVClient()
        
        with pytest.raises(ConnectionError):
            client.pipeline(["GET test"])

    @patch('socket.create_connection')
    def test_pipeline_with_errors(self, mock_connection):
        """Test pipeline with some command errors."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.side_effect = [b"OK\r\n", Exception("Network error")]
        
        client = MerkleKVClient()
        client.connect()
        
        commands = ["SET key1 value1", "GET invalid_key"]
        results = client.pipeline(commands)
        
        assert len(results) == 2
        assert results[0] == "OK"
        assert "Network error" in results[1]

    @patch('socket.create_connection')
    def test_health_check_success(self, mock_connection):
        """Test successful health check."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.return_value = b"PONG\r\n"
        
        client = MerkleKVClient()
        client.connect()
        
        assert client.health_check() is True
        mock_socket.sendall.assert_called_with(b"PING\r\n")

    @patch('socket.create_connection')
    def test_health_check_failure(self, mock_connection):
        """Test health check failure."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.return_value = b"ERROR\r\n"
        
        client = MerkleKVClient()
        client.connect()
        
        assert client.health_check() is False

    def test_health_check_not_connected(self):
        """Test health check when not connected."""
        client = MerkleKVClient()
        assert client.health_check() is False

    @patch('socket.create_connection')
    def test_health_check_exception(self, mock_connection):
        """Test health check with exception."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.side_effect = Exception("Network error")
        
        client = MerkleKVClient()
        client.connect()
        
        assert client.health_check() is False

    @patch('socket.create_connection')
    def test_tcp_nodelay_setting(self, mock_connection):
        """Test that TCP_NODELAY is properly set."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        
        client = MerkleKVClient()
        client.connect()
        
        # Verify TCP_NODELAY was set
        mock_socket.setsockopt.assert_called_with(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)

    @patch('socket.create_connection')
    def test_send_command_socket_timeout(self, mock_connection):
        """Test _send_command with socket timeout."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.side_effect = socket.timeout()
        
        client = MerkleKVClient()
        client.connect()
        
        with pytest.raises(TimeoutError):
            client._send_command("GET test")

    @patch('socket.create_connection')
    def test_send_command_socket_error(self, mock_connection):
        """Test _send_command with socket error."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.side_effect = socket.error("Connection lost")
        
        client = MerkleKVClient()
        client.connect()
        
        with pytest.raises(ConnectionError):
            client._send_command("GET test")

    @patch('socket.create_connection')
    def test_send_command_multiple_receives(self, mock_connection):
        """Test _send_command with multiple recv calls needed."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        # Simulate partial response requiring multiple recv calls
        mock_socket.recv.side_effect = [b"VAL", b"UE test\r\n"]
        
        client = MerkleKVClient()
        client.connect()
        
        response = client._send_command("GET test")
        assert response == "VALUE test"

    @patch('socket.create_connection')
    def test_send_command_empty_response(self, mock_connection):
        """Test _send_command with empty response triggering connection lost."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.return_value = b""  # Empty response indicates connection lost
        
        client = MerkleKVClient()
        client.connect()
        
        with pytest.raises(ConnectionError, match="Connection lost"):
            client._send_command("GET test")

    @patch('socket.create_connection')
    def test_delete_with_ok_response(self, mock_connection):
        """Test delete operation with OK response."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.return_value = b"OK\r\n"
        
        client = MerkleKVClient()
        client.connect()
        
        result = client.delete("test_key")
        assert result is True

    @patch('socket.create_connection')
    def test_delete_with_deleted_response(self, mock_connection):
        """Test delete operation with DELETED response."""
        mock_socket = Mock()
        mock_connection.return_value = mock_socket
        mock_socket.recv.return_value = b"DELETED\r\n"
        
        client = MerkleKVClient()
        client.connect()
        
        result = client.delete("test_key")
        assert result is True


class TestAsyncMerkleKVClientExtended:
    """Extended tests for AsyncMerkleKV client."""

    @pytest.mark.asyncio
    async def test_async_init(self):
        """Test async client initialization."""
        client = AsyncMerkleKVClient("localhost", 8000, timeout=30.0)
        assert client.host == "localhost"
        assert client.port == 8000
        assert client.timeout == 30.0
        assert not client.is_connected()

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_connect_tcp_nodelay(self, mock_open_connection):
        """Test async client TCP_NODELAY setting."""
        mock_reader = Mock()
        mock_writer = Mock()
        mock_socket = Mock()
        
        mock_writer.get_extra_info.return_value = mock_socket
        mock_open_connection.return_value = (mock_reader, mock_writer)
        
        client = AsyncMerkleKVClient()
        await client.connect()
        
        # Verify TCP_NODELAY was set
        mock_socket.setsockopt.assert_called_with(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_connect_no_socket(self, mock_open_connection):
        """Test async client connect when socket is not available."""
        mock_reader = Mock()
        mock_writer = Mock()
        
        mock_writer.get_extra_info.return_value = None
        mock_open_connection.return_value = (mock_reader, mock_writer)
        
        client = AsyncMerkleKVClient()
        await client.connect()
        
        assert client.is_connected()

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_pipeline_success(self, mock_open_connection):
        """Test async pipeline operations."""
        mock_reader = Mock()
        mock_writer = Mock()
        mock_open_connection.return_value = (mock_reader, mock_writer)
        
        # Mock readuntil responses
        mock_reader.readuntil.side_effect = [b"OK\r\n", b"VALUE test\r\n"]
        
        client = AsyncMerkleKVClient()
        await client.connect()
        
        commands = ["SET key value", "GET key"]
        results = await client.pipeline(commands)
        
        assert len(results) == 2
        assert results[0] == "OK"
        assert results[1] == "VALUE test"

    @pytest.mark.asyncio
    async def test_async_pipeline_empty(self):
        """Test async pipeline with empty commands."""
        client = AsyncMerkleKVClient()
        result = await client.pipeline([])
        assert result == []

    @pytest.mark.asyncio
    async def test_async_pipeline_not_connected(self):
        """Test async pipeline when not connected."""
        client = AsyncMerkleKVClient()
        
        with pytest.raises(ConnectionError):
            await client.pipeline(["GET test"])

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_health_check_success(self, mock_open_connection):
        """Test async health check success."""
        mock_reader = Mock()
        mock_writer = Mock()
        mock_open_connection.return_value = (mock_reader, mock_writer)
        mock_reader.readuntil.return_value = b"PONG\r\n"
        
        client = AsyncMerkleKVClient()
        await client.connect()
        
        result = await client.health_check()
        assert result is True

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_health_check_failure(self, mock_open_connection):
        """Test async health check failure."""
        mock_reader = Mock()
        mock_writer = Mock()
        mock_open_connection.return_value = (mock_reader, mock_writer)
        mock_reader.readuntil.return_value = b"ERROR\r\n"
        
        client = AsyncMerkleKVClient()
        await client.connect()
        
        result = await client.health_check()
        assert result is False

    @pytest.mark.asyncio
    async def test_async_health_check_not_connected(self):
        """Test async health check when not connected."""
        client = AsyncMerkleKVClient()
        result = await client.health_check()
        assert result is False

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_health_check_exception(self, mock_open_connection):
        """Test async health check with exception."""
        mock_reader = Mock()
        mock_writer = Mock()
        mock_open_connection.return_value = (mock_reader, mock_writer)
        mock_reader.readuntil.side_effect = Exception("Network error")
        
        client = AsyncMerkleKVClient()
        await client.connect()
        
        result = await client.health_check()
        assert result is False

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_delete_deleted_response(self, mock_open_connection):
        """Test async delete with DELETED response."""
        mock_reader = Mock()
        mock_writer = Mock()
        mock_open_connection.return_value = (mock_reader, mock_writer)
        mock_reader.readuntil.return_value = b"DELETED\r\n"
        
        client = AsyncMerkleKVClient()
        await client.connect()
        
        result = await client.delete("test_key")
        assert result is True

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_context_manager(self, mock_open_connection):
        """Test async context manager functionality."""
        mock_reader = Mock()
        mock_writer = Mock()
        mock_open_connection.return_value = (mock_reader, mock_writer)
        
        async with AsyncMerkleKVClient() as client:
            assert client.is_connected()
        
        # After context exit, close should have been called
        mock_writer.close.assert_called_once()

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_send_command_timeout(self, mock_open_connection):
        """Test async _send_command with timeout."""
        mock_reader = Mock()
        mock_writer = Mock()
        mock_open_connection.return_value = (mock_reader, mock_writer)
        mock_reader.readuntil.side_effect = asyncio.TimeoutError()
        
        client = AsyncMerkleKVClient()
        await client.connect()
        
        with pytest.raises(TimeoutError):
            await client._send_command("GET test")

    @pytest.mark.asyncio
    @patch('asyncio.open_connection')
    async def test_async_send_command_os_error(self, mock_open_connection):
        """Test async _send_command with OS error."""
        mock_reader = Mock()
        mock_writer = Mock()
        mock_open_connection.return_value = (mock_reader, mock_writer)
        mock_reader.readuntil.side_effect = OSError("Connection lost")
        
        client = AsyncMerkleKVClient()
        await client.connect()
        
        with pytest.raises(ConnectionError):
            await client._send_command("GET test")
