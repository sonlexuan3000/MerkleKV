<?php

declare(strict_types=1);

namespace MerkleKV;

use InvalidArgumentException;

/**
 * Official PHP client for MerkleKV distributed key-value store.
 * 
 * This client implements the TCP text protocol with CRLF termination and UTF-8 encoding.
 * Empty values in SET operations are automatically represented as "" at the protocol layer.
 * 
 * @example
 * $client = new MerkleKV\Client("127.0.0.1", 7379, 5.0);
 * $client->set("user:1", "alice");
 * $value = $client->get("user:1");     // "alice" or null
 * $deleted = $client->delete("user:1"); // bool
 * $client->close();
 */
class Client
{
    /** @var string */
    private $host;

    /** @var int */
    private $port;

    /** @var float */
    private $timeout;

    /** @var resource|null */
    private $socket;

    /**
     * Initialize a new MerkleKV client.
     *
     * @param string $host Server hostname or IP address
     * @param int $port Server port
     * @param float $timeout Operation timeout in seconds
     * @throws InvalidArgumentException if parameters are invalid
     */
    public function __construct(string $host = "127.0.0.1", int $port = 7379, float $timeout = 5.0)
    {
        if (empty($host)) {
            throw new InvalidArgumentException("Host cannot be empty");
        }
        
        if ($port < 1 || $port > 65535) {
            throw new InvalidArgumentException("Port must be between 1 and 65535");
        }
        
        if ($timeout <= 0) {
            throw new InvalidArgumentException("Timeout must be positive");
        }

        $this->host = $host;
        $this->port = $port;
        $this->timeout = $timeout;
        $this->socket = null;
    }

    /**
     * Destructor - automatically closes connection.
     */
    public function __destruct()
    {
        $this->close();
    }

    /**
     * Set a key-value pair.
     * Empty values are automatically handled - they are represented as "" at the protocol layer.
     *
     * @param string $key Key to set (cannot be empty)
     * @param string $value Value to set (empty values are automatically converted to "")
     * @throws InvalidArgumentException if key is invalid
     * @throws ConnectionException if connection fails
     * @throws TimeoutException if operation times out
     * @throws ProtocolException if server returns an error
     */
    public function set(string $key, string $value): void
    {
        $this->validateKey($key);
        
        $formattedValue = $this->formatValue($value);
        $command = "SET {$key} {$formattedValue}";
        
        $response = $this->sendCommand($command);
        
        if ($response === "OK") {
            return;
        }
        
        if (strpos($response, "ERROR ") === 0) {
            throw new ProtocolException(substr($response, 6));
        }
        
        throw new ProtocolException("Unexpected response: {$response}");
    }

    /**
     * Get a value by key.
     *
     * @param string $key Key to retrieve (cannot be empty)
     * @return string|null Value if found, null if key not found
     * @throws InvalidArgumentException if key is invalid
     * @throws ConnectionException if connection fails
     * @throws TimeoutException if operation times out
     * @throws ProtocolException if server returns an error
     */
    public function get(string $key): ?string
    {
        $this->validateKey($key);
        
        $command = "GET {$key}";
        $response = $this->sendCommand($command);
        
        if ($response === "(null)" || $response === "NOT_FOUND") {
            return null;
        }
        
        if (strpos($response, "ERROR ") === 0) {
            throw new ProtocolException(substr($response, 6));
        }
        
        // Strip "VALUE " prefix from server response
        if (strpos($response, "VALUE ") === 0) {
            $value = substr($response, 6);
            // Handle empty value represented as ""
            if ($value === '""') {
                return "";
            }
            return $value;
        }
        
        return $response;
    }

    /**
     * Delete a key.
     *
     * @param string $key Key to delete (cannot be empty)
     * @return bool true if key was deleted, false if key was not found
     * @throws InvalidArgumentException if key is invalid
     * @throws ConnectionException if connection fails
     * @throws TimeoutException if operation times out
     * @throws ProtocolException if server returns an error
     */
    public function delete(string $key): bool
    {
        $this->validateKey($key);
        
        $command = "DEL {$key}";
        $response = $this->sendCommand($command);
        
        switch ($response) {
            case "OK":
                return true;  // Server returns OK for both existing and non-existing keys
            case "DELETED":
                return true;
            case "NOT_FOUND":
                return false;
            default:
                if (strpos($response, "ERROR ") === 0) {
                    throw new ProtocolException(substr($response, 6));
                }
                throw new ProtocolException("Unexpected response: {$response}");
        }
    }

    /**
     * Close the connection to the server.
     * This method is idempotent and can be called multiple times safely.
     */
    public function close(): void
    {
        if ($this->socket !== null) {
            if (is_resource($this->socket)) {
                fclose($this->socket);
            }
            $this->socket = null;
        }
    }

    /**
     * Check if the client is connected.
     *
     * @return bool true if connected, false otherwise
     */
    public function isConnected(): bool
    {
        return $this->socket !== null && is_resource($this->socket) && !feof($this->socket);
    }

    /**
     * Ensure connection is established.
     *
     * @throws ConnectionException if connection fails
     * @throws TimeoutException if connection times out
     */
    private function ensureConnected(): void
    {
        if ($this->isConnected()) {
            return;
        }

        $context = stream_context_create([
            'socket' => [
                'tcp_nodelay' => true,
            ],
        ]);

        $errno = 0;
        $errstr = '';
        
        $this->socket = stream_socket_client(
            "tcp://{$this->host}:{$this->port}",
            $errno,
            $errstr,
            $this->timeout,
            STREAM_CLIENT_CONNECT,
            $context
        );

        if ($this->socket === false) {
            throw new ConnectionException("Failed to connect to {$this->host}:{$this->port}: {$errstr} ({$errno})");
        }

        // Set timeout for read/write operations
        if (!stream_set_timeout($this->socket, (int)$this->timeout, (int)(($this->timeout - floor($this->timeout)) * 1000000))) {
            $this->close();
            throw new ConnectionException("Failed to set socket timeout");
        }
    }

    /**
     * Send a command and return the response.
     *
     * @param string $command Command to send
     * @return string Server response
     * @throws ConnectionException if connection fails
     * @throws TimeoutException if operation times out
     */
    private function sendCommand(string $command): string
    {
        $this->ensureConnected();

        // Send command with CRLF termination
        $fullCommand = $command . "\r\n";
        $bytesWritten = fwrite($this->socket, $fullCommand);
        
        if ($bytesWritten === false || $bytesWritten !== strlen($fullCommand)) {
            $this->close();
            throw new ConnectionException("Failed to send command");
        }

        // Flush output
        if (!fflush($this->socket)) {
            $this->close();
            throw new ConnectionException("Failed to flush command");
        }

        // Read response until CRLF
        $response = fgets($this->socket);
        
        if ($response === false) {
            $info = stream_get_meta_data($this->socket);
            $this->close();
            
            if ($info['timed_out']) {
                throw new TimeoutException("Operation timeout");
            } else {
                throw new ConnectionException("Failed to read response");
            }
        }

        // Remove CRLF terminator
        return rtrim($response, "\r\n");
    }

    /**
     * Format a value for the SET command. Empty strings are represented as "".
     *
     * @param string $value Value to format
     * @return string Formatted value
     */
    private function formatValue(string $value): string
    {
        return $value === '' ? '""' : $value;
    }

    /**
     * Validate that key is not empty.
     *
     * @param string $key Key to validate
     * @throws InvalidArgumentException if key is invalid
     */
    private function validateKey(string $key): void
    {
        if ($key === '') {
            throw new InvalidArgumentException("Key cannot be empty");
        }
    }
}
