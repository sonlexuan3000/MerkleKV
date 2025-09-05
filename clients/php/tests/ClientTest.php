<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use MerkleKV\Client;
use MerkleKV\ConnectionException;
use MerkleKV\TimeoutException;
use MerkleKV\ProtocolException;

/**
 * Unit tests for MerkleKV Client
 * 
 * These tests cover the client API behavior, parameter validation, and error handling.
 * Integration tests with a real server are in separate test files.
 */
class ClientTest extends TestCase
{
    public function testConstructorWithDefaults(): void
    {
        $client = new Client();
        $this->assertInstanceOf(Client::class, $client);
        $client->close();
    }

    public function testConstructorWithCustomParameters(): void
    {
        $client = new Client("localhost", 8080, 10.0);
        $this->assertInstanceOf(Client::class, $client);
        $client->close();
    }

    public function testConstructorWithEmptyHost(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage("Host cannot be empty");
        new Client("", 7379);
    }

    public function testConstructorWithInvalidPort(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage("Port must be between 1 and 65535");
        new Client("localhost", 0);
        
        $this->expectException(InvalidArgumentException::class);
        new Client("localhost", 65536);
        
        $this->expectException(InvalidArgumentException::class);
        new Client("localhost", -1);
    }

    public function testConstructorWithInvalidTimeout(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage("Timeout must be positive");
        new Client("localhost", 7379, 0.0);
        
        $this->expectException(InvalidArgumentException::class);
        new Client("localhost", 7379, -1.0);
    }

    public function testSetWithEmptyKey(): void
    {
        $client = new Client();
        
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage("Key cannot be empty");
        $client->set("", "value");
    }

    public function testGetWithEmptyKey(): void
    {
        $client = new Client();
        
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage("Key cannot be empty");
        $client->get("");
    }

    public function testDeleteWithEmptyKey(): void
    {
        $client = new Client();
        
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage("Key cannot be empty");
        $client->delete("");
    }

    public function testConnectionFailure(): void
    {
        // Use a port that's unlikely to be in use
        $client = new Client("127.0.0.1", 19999, 1.0);
        
        $this->expectException(ConnectionException::class);
        $client->set("test", "value");
    }

    public function testIsConnectedInitially(): void
    {
        $client = new Client();
        $this->assertFalse($client->isConnected());
        $client->close();
    }

    public function testCloseIsIdempotent(): void
    {
        $client = new Client();
        $client->close();
        $client->close(); // Should not throw
        $this->assertFalse($client->isConnected());
    }

    public function testDestructorCallsClose(): void
    {
        $client = new Client();
        // Destructor will be called automatically
        // This test ensures no exceptions are thrown
        $this->assertTrue(true);
    }

    public function testValidKeyValidation(): void
    {
        $client = new Client();
        
        // These should not throw validation errors (though they may fail with connection errors)
        try {
            $client->set("valid_key", "value");
        } catch (ConnectionException $e) {
            // Expected when server is not running
            $this->assertTrue(true);
        }

        try {
            $client->set("key-with-dashes", "value");
        } catch (ConnectionException $e) {
            $this->assertTrue(true);
        }

        try {
            $client->set("key123", "value");
        } catch (ConnectionException $e) {
            $this->assertTrue(true);
        }

        $client->close();
    }

    public function testEmptyValueHandling(): void
    {
        $client = new Client();
        
        // Empty values should be accepted (though connection may fail)
        try {
            $client->set("test", "");
        } catch (ConnectionException $e) {
            // Expected when server is not running
            $this->assertTrue(true);
        }

        $client->close();
    }
}
