"""
Synchronous MerkleKV client implementation.
"""

import socket
from typing import Optional
import time
from typing import Optional


class MerkleKVError(Exception):
    """Base exception for MerkleKV client errors."""
    pass


class ConnectionError(MerkleKVError):
    """Raised when connection to server fails."""
    pass


class TimeoutError(MerkleKVError):
    """Raised when operation times out."""
    pass


class ProtocolError(MerkleKVError):
    """Raised when server returns an error."""
    pass


class MerkleKVClient:
    """
    Synchronous client for MerkleKV server.
    
    This client provides a simple interface for connecting to a MerkleKV server
    and performing basic operations like GET, SET, and DELETE.
    
    Example:
        client = MerkleKVClient("localhost", 7379)
        client.connect()
        
        client.set("user:123", "john_doe")
        value = client.get("user:123")  # Returns "john_doe"
        client.delete("user:123")
        
        client.close()
    """
    
    def __init__(self, host: str = "localhost", port: int = 7379, timeout: float = 5.0):
        """
        Initialize the MerkleKV client.
        
        Args:
            host: Server hostname (default: "localhost")
            port: Server port (default: 7379)
            timeout: Socket timeout in seconds (default: 5.0)
        """
        self.host = host
        self.port = port
        self.timeout = timeout
        self._socket: Optional[socket.socket] = None
        self._connected = False
    
    def connect(self) -> None:
        """
        Connect to the MerkleKV server.
        
        Raises:
            ConnectionError: If connection fails
        """
        try:
            self._socket = socket.create_connection((self.host, self.port), timeout=self.timeout)
            # Enable TCP_NODELAY for better latency
            self._socket.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
            self._connected = True
        except (socket.error, socket.timeout) as e:
            raise ConnectionError(f"Failed to connect to {self.host}:{self.port}: {e}")
    
    def close(self) -> None:
        """
        Close the connection to the server.
        """
        if self._socket:
            self._socket.close()
            self._socket = None
            self._connected = False
    
    def is_connected(self) -> bool:
        """
        Check if client is connected to server.
        
        Returns:
            True if connected, False otherwise
        """
        return self._connected and self._socket is not None
    
    def _send_command(self, command: str) -> str:
        """
        Send a command to the server and return the response.
        
        Args:
            command: Command string to send
            
        Returns:
            Server response string
            
        Raises:
            ConnectionError: If not connected or connection fails
            TimeoutError: If operation times out
            ProtocolError: If server returns an error
        """
        if not self.is_connected():
            raise ConnectionError("Not connected to server. Call connect() first.")
        
        try:
            # Send command with CRLF termination
            message = f"{command}\r\n".encode('utf-8')
            self._socket.sendall(message)
            
            # Read response line by line until we get CRLF
            # Handle large responses by reading until we find CRLF
            self._socket.settimeout(self.timeout)
            buffer = b''
            
            # For potentially large responses, read in chunks until we find CRLF
            while True:
                try:
                    chunk = self._socket.recv(4096)
                    if not chunk:
                        raise ConnectionError("Connection closed by server")
                    buffer += chunk
                    
                    # Look for CRLF terminator
                    crlf_pos = buffer.find(b'\r\n')
                    if crlf_pos != -1:
                        # Found complete response
                        response = buffer[:crlf_pos].decode('utf-8')
                        break
                        
                except socket.timeout:
                    raise TimeoutError(f"Operation timed out after {self.timeout} seconds")
            
            # Check for protocol errors
            if response.startswith("ERROR "):
                error_msg = response[6:]  # Remove "ERROR " prefix
                raise ProtocolError(error_msg)
            
            return response
            
        except socket.timeout:
            raise TimeoutError(f"Operation timed out after {self.timeout} seconds")
        except socket.error as e:
            self._connected = False
            raise ConnectionError(f"Socket error: {e}")
    
    def get(self, key: str) -> Optional[str]:
        """
        Get the value for a key.
        
        Args:
            key: The key to retrieve
            
        Returns:
            The value if found, None if not found
            
        Raises:
            ValueError: If key is empty
            ConnectionError: If connection fails
            TimeoutError: If operation times out
            ProtocolError: If server returns an error
        """
        if not key:
            raise ValueError("Key cannot be empty")
        
        # Retry logic for potential server state corruption with large values
        max_retries = 3
        for attempt in range(max_retries):
            try:
                response = self._send_command(f"GET {key}")
                
                if response == "NOT_FOUND":
                    return None
                elif response.startswith("VALUE "):
                    return response[6:]  # Remove "VALUE " prefix
                elif response == "(null)":  # Alternative null response format
                    return None
                else:
                    # If we get an unexpected response that looks like corruption,
                    # try reconnecting and retrying (server state corruption workaround)
                    if "Unknown command:" in response and attempt < max_retries - 1:
                        self.close()
                        self.connect()
                        continue
                    raise ProtocolError(f"Unexpected response: {response}")
            except (ConnectionError, ProtocolError) as e:
                if attempt < max_retries - 1:
                    # Reconnect and retry for potential server state issues
                    self.close()
                    self.connect()
                    continue
                raise e
        
        raise ProtocolError(f"Failed to get key '{key}' after {max_retries} attempts")
    
    def set(self, key: str, value: str) -> bool:
        """
        Set a key-value pair.
        
        Args:
            key: The key to set
            value: The value to associate with the key
            
        Returns:
            True if successful
            
        Raises:
            ConnectionError: If not connected or connection fails  
            TimeoutError: If operation times out
            ProtocolError: If server returns an error
        """
        if not key:
            raise ValueError("Key cannot be empty")
        
        # Handle empty values by quoting them
        if value == "":
            response = self._send_command(f'SET {key} ""')
        else:
            response = self._send_command(f"SET {key} {value}")
        
        if response == "OK":
            # Workaround for server state corruption with large values
            # Reconnect if the total command size is large to prevent parser state corruption
            total_command_size = len(f"SET {key} {value}") 
            if total_command_size > 1024:  # Conservative threshold
                self.close()
                self.connect()
            return True
        else:
            raise ProtocolError(f"Unexpected response: {response}")
    
    def delete(self, key: str) -> bool:
        """
        Delete a key.
        
        Args:
            key: The key to delete
            
        Returns:
            True if successful (whether key existed or not)
            
        Raises:
            ConnectionError: If not connected or connection fails
            TimeoutError: If operation times out  
            ProtocolError: If server returns an error
        """
        if not key:
            raise ValueError("Key cannot be empty")
        
        response = self._send_command(f"DEL {key}")

        if response in ("OK", "DELETED"):
            return True
        else:
            raise ProtocolError(f"Unexpected response: {response}")
    
    def pipeline(self, commands: list) -> list:
        """
        Execute multiple commands in a pipeline for better performance.
        
        Args:
            commands: List of command strings to execute
            
        Returns:
            List of responses corresponding to each command
            
        Raises:
            ConnectionError: If not connected or connection fails
            TimeoutError: If operation times out
            ProtocolError: If server returns an error
        """
        if not commands:
            return []
        
        if not self.is_connected():
            raise ConnectionError("Not connected to server")
        
        responses = []
        for command in commands:
            try:
                response = self._send_command(command)
                responses.append(response)
            except Exception as e:
                # For pipeline, we collect partial results
                responses.append(str(e))
        
        return responses
    
    def health_check(self) -> bool:
        """
        Perform a health check by sending a PING command.
        
        Returns:
            True if server responds successfully, False otherwise
        """
        if not self.is_connected():
            return False
        
        try:
            response = self._send_command("PING")
            return response == "PONG"
        except Exception:
            return False

    def __enter__(self):
        """Context manager entry."""
        self.connect()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
