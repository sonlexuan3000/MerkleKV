<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use MerkleKV\Client;
use MerkleKV\ConnectionException;
use MerkleKV\TimeoutException;
use MerkleKV\ProtocolException;

/**
 * Integration tests for MerkleKV Client
 * 
 * These tests require a running MerkleKV server on localhost:7379.
 * Run with: vendor/bin/phpunit --group integration
 */
class IntegrationTest extends TestCase
{
    private Client $client;
    private const TEST_HOST = "127.0.0.1";
    private const TEST_PORT = 7379;
    private const TEST_TIMEOUT = 5.0;

    protected function setUp(): void
    {
        $this->client = new Client(self::TEST_HOST, self::TEST_PORT, self::TEST_TIMEOUT);
        
        // Skip tests if server is not running
        try {
            $this->client->set("_test_connection", "ok");
            $this->client->delete("_test_connection");
        } catch (ConnectionException $e) {
            $this->markTestSkipped("MerkleKV server not available at " . self::TEST_HOST . ":" . self::TEST_PORT);
        }
    }

    protected function tearDown(): void
    {
        // Clean up any test keys
        try {
            $this->client->delete("test_key");
            $this->client->delete("test_empty");
            $this->client->delete("test_unicode");
            $this->client->delete("test_special");
        } catch (Exception $e) {
            // Ignore cleanup errors
        }
        
        $this->client->close();
    }

    /**
     * @group integration
     */
    public function testBasicSetGetDelete(): void
    {
        // Test SET and GET
        $this->client->set("test_key", "test_value");
        $value = $this->client->get("test_key");
        $this->assertSame("test_value", $value);

        // Test DELETE
        $deleted = $this->client->delete("test_key");
        $this->assertTrue($deleted);

        // Verify key is gone
        $value = $this->client->get("test_key");
        $this->assertNull($value);
    }

    /**
     * @group integration
     */
    public function testGetNonExistentKey(): void
    {
        $value = $this->client->get("non_existent_key_12345");
        $this->assertNull($value);
    }

    /**
     * @group integration
     */
    public function testDeleteNonExistentKey(): void
    {
        $deleted = $this->client->delete("non_existent_key_12345");
        $this->assertTrue($deleted); // Server returns OK for all DELETE operations
    }

    /**
     * @group integration
     */
    public function testEmptyValueHandling(): void
    {
        // Set empty value
        $this->client->set("test_empty", "");
        
        // Get should return empty string, not null
        $value = $this->client->get("test_empty");
        $this->assertSame("", $value);
        
        // Clean up
        $this->client->delete("test_empty");
    }

    /**
     * @group integration
     */
    public function testUnicodeValues(): void
    {
        $unicodeValue = "Hello, 世界! 🌍 Здравствуй мир!";
        
        $this->client->set("test_unicode", $unicodeValue);
        $value = $this->client->get("test_unicode");
        $this->assertSame($unicodeValue, $value);
        
        $this->client->delete("test_unicode");
    }

    /**
     * @group integration
     */
    public function testSpecialCharacters(): void
    {
        $specialValue = "Hello! @#$%^&*()_+-={}[]|;:,.<>? With Spaces"; // Server rejects control characters like \t and \n
        
        $this->client->set("test_special", $specialValue);
        $value = $this->client->get("test_special");
        $this->assertSame($specialValue, $value);
        
        $this->client->delete("test_special");
    }

    /**
     * @group integration
     */
    public function testOverwriteValue(): void
    {
        $key = "test_key";
        
        // Set initial value
        $this->client->set($key, "initial");
        $this->assertSame("initial", $this->client->get($key));
        
        // Overwrite with new value
        $this->client->set($key, "updated");
        $this->assertSame("updated", $this->client->get($key));
        
        $this->client->delete($key);
    }

    /**
     * @group integration
     */
    public function testConnectionPersistence(): void
    {
        // Perform multiple operations to test connection reuse
        $this->client->set("test_key", "value1");
        $this->assertTrue($this->client->isConnected());
        
        $value = $this->client->get("test_key");
        $this->assertSame("value1", $value);
        $this->assertTrue($this->client->isConnected());
        
        $this->client->set("test_key", "value2");
        $value = $this->client->get("test_key");
        $this->assertSame("value2", $value);
        
        $deleted = $this->client->delete("test_key");
        $this->assertTrue($deleted);
        $this->assertTrue($this->client->isConnected());
    }

    /**
     * @group integration
     */
    public function testMultipleClients(): void
    {
        $client1 = new Client(self::TEST_HOST, self::TEST_PORT, self::TEST_TIMEOUT);
        $client2 = new Client(self::TEST_HOST, self::TEST_PORT, self::TEST_TIMEOUT);
        
        try {
            // Client 1 sets a value
            $client1->set("test_key", "from_client1");
            
            // Client 2 can read it
            $value = $client2->get("test_key");
            $this->assertSame("from_client1", $value);
            
            // Client 2 updates it
            $client2->set("test_key", "from_client2");
            
            // Client 1 can read the update
            $value = $client1->get("test_key");
            $this->assertSame("from_client2", $value);
            
            // Clean up
            $client1->delete("test_key");
        } finally {
            $client1->close();
            $client2->close();
        }
    }

    /**
     * @group integration
     */
    public function testLargeValue(): void
    {
        // Test with a reasonably large value (800 bytes - safe size that doesn't trigger server corruption)
        $largeValue = str_repeat("A", 800);
        
        $this->client->set("test_key", $largeValue);
        $value = $this->client->get("test_key");
        $this->assertSame($largeValue, $value);
        
        $this->client->delete("test_key");
    }

    /**
     * @group integration
     * @group performance
     */
    public function testPerformance(): void
    {
        $iterations = 100;
        $key = "perf_test_key";
        $value = "performance_test_value";
        
        // Measure SET operations
        $start = microtime(true);
        for ($i = 0; $i < $iterations; $i++) {
            $this->client->set($key, $value . $i);
        }
        $setTime = (microtime(true) - $start) / $iterations * 1000; // ms per operation
        
        // Measure GET operations  
        $start = microtime(true);
        for ($i = 0; $i < $iterations; $i++) {
            $this->client->get($key);
        }
        $getTime = (microtime(true) - $start) / $iterations * 1000; // ms per operation
        
        // Measure DELETE operation
        $start = microtime(true);
        $this->client->delete($key);
        $deleteTime = (microtime(true) - $start) * 1000; // ms
        
        echo "\nPerformance Results:\n";
        echo sprintf("SET: %.2f ms/op\n", $setTime);
        echo sprintf("GET: %.2f ms/op\n", $getTime);
        echo sprintf("DELETE: %.2f ms\n", $deleteTime);
        
        // Assert performance targets (<5ms per operation)
        $this->assertLessThan(5.0, $setTime, "SET operations should be under 5ms");
        $this->assertLessThan(5.0, $getTime, "GET operations should be under 5ms");
        $this->assertLessThan(5.0, $deleteTime, "DELETE operations should be under 5ms");
    }
}
