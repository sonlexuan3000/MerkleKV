package io.merklekv.client

import kotlinx.coroutines.*
import kotlinx.coroutines.test.*
import org.junit.jupiter.api.*
import org.junit.jupiter.api.Assumptions.assumeTrue
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.seconds

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class MerkleKVClientTest {
    
    private lateinit var client: MerkleKVClient
    private var serverAvailable = false
    
    private fun getTestPort(): Int {
        return System.getenv("MERKLEKV_PORT")?.toIntOrNull() ?: 7379
    }
    
    @BeforeAll
    fun checkServerAvailability() {
        runBlocking {
            try {
                val testPort = getTestPort()
                val testClient = MerkleKVClient("localhost", testPort)
                testClient.use {
                    it.connect()
                    serverAvailable = true
                }
            } catch (e: Exception) {
                serverAvailable = false
                println("⚠️  MerkleKV server not available at localhost:${getTestPort()} - skipping integration tests")
            }
        }
    }
    
    @BeforeEach
    fun setUp() {
        assumeTrue(serverAvailable, "MerkleKV server not available")
        client = MerkleKVClient("localhost", getTestPort())
    }
    
    @AfterEach
    fun tearDown() {
        if (serverAvailable) {
            runBlocking {
                client.close()
            }
        }
    }
    
    // MARK: - Basic Operations Tests
    
    @Test
    fun `test set and get operation`() = runTest {
        client.connect()
        
        client.set("test:key", "test:value")
        val value = client.get("test:key")
        
        assertEquals("test:value", value)
    }
    
    @Test
    fun `test get non-existent key returns null`() = runTest {
        client.connect()
        
        val value = client.get("non:existent:key:12345")
        
        assertNull(value)
    }
    
    @Test
    fun `test delete existing key`() = runTest {
        client.connect()
        
        client.set("delete:test", "value")
        val deleted = client.delete("delete:test")
        assertTrue(deleted)
        
        val value = client.get("delete:test")
        assertNull(value)
    }
    
    @Test
    fun `test delete non-existent key`() = runTest {
        client.connect()
        
        val deleted = client.delete("non:existent:delete:12345")
        
        assertFalse(deleted)
    }
    
    // MARK: - Edge Cases Tests
    
    @Test
    fun `test empty value`() = runTest {
        client.connect()
        
        client.set("empty:key", "")
        val value = client.get("empty:key")
        
        assertEquals("", value)
    }
    
    @Test
    fun `test unicode values`() = runTest {
        client.connect()
        
        val unicodeValue = "Hello 世界 🌍 Ñoël"
        client.set("unicode:key", unicodeValue)
        val value = client.get("unicode:key")
        
        assertEquals(unicodeValue, value)
    }
    
    @Test
    fun `test large value`() = runTest {
        client.connect()
        
        val largeValue = "x".repeat(10000)
        client.set("large:key", largeValue)
        val value = client.get("large:key")
        
        assertEquals(largeValue, value)
    }
    
    @Test
    fun `test tab characters in value`() = runTest {
        client.connect()
        
        val valueWithTab = "value\twith\ttabs"
        client.set("tab:key", valueWithTab)
        val value = client.get("tab:key")
        
        assertEquals(valueWithTab, value)
    }
    
    // MARK: - Validation Tests
    
    @Test
    fun `test empty key validation`() = runTest {
        client.connect()
        
        assertThrows<MerkleKVException.ValidationException> {
            client.set("", "value")
        }
    }
    
    @Test
    fun `test newline in key validation`() = runTest {
        client.connect()
        
        assertThrows<MerkleKVException.ValidationException> {
            client.set("key\nwith\nnewlines", "value")
        }
    }
    
    @Test
    fun `test newline in value validation`() = runTest {
        client.connect()
        
        assertThrows<MerkleKVException.ValidationException> {
            client.set("key", "value\nwith\nnewlines")
        }
    }
    
    // MARK: - Pipeline Tests
    
    @Test
    fun `test pipeline operations`() = runTest {
        client.connect()
        
        val operations = listOf(
            Operation.Set("pipeline:1", "value1"),
            Operation.Set("pipeline:2", "value2"),
            Operation.Get("pipeline:1"),
            Operation.Get("pipeline:2"),
            Operation.Delete("pipeline:1"),
            Operation.Get("pipeline:1")
        )
        
        val results = client.pipeline(operations)
        assertEquals(6, results.size)
        
        // Verify results
        assertEquals(Unit, results[0])  // set result
        assertEquals(Unit, results[1])  // set result
        assertEquals("value1", results[2])  // get result
        assertEquals("value2", results[3])  // get result
        assertEquals(true, results[4])  // delete result
        assertNull(results[5])  // get deleted key result
    }
    
    @Test
    fun `test pipeline with empty operations`() = runTest {
        client.connect()
        
        val results = client.pipeline(emptyList())
        assertTrue(results.isEmpty())
    }
    
    // MARK: - Health Check Tests
    
    @Test
    fun `test health check`() = runTest {
        client.connect()
        
        val isHealthy = client.healthCheck()
        assertTrue(isHealthy)
    }
    
    // MARK: - Connection Tests
    
    @Test
    fun `test connection status`() = runTest {
        assertFalse(client.connected)
        
        client.connect()
        assertTrue(client.connected)
        
        client.disconnect()
        assertFalse(client.connected)
    }
    
    @Test
    fun `test operation on disconnected client`() = runTest {
        assertThrows<MerkleKVException.ConnectionException> {
            client.set("test", "value")
        }
    }
    
    // MARK: - Concurrency Tests
    
    @Test
    fun `test concurrent operations`() = runTest {
        client.connect()
        
        val jobs = (0 until 10).map { i ->
            async {
                client.set("concurrent:$i", "value$i")
                val value = client.get("concurrent:$i")
                assertEquals("value$i", value)
                client.delete("concurrent:$i")
            }
        }
        
        jobs.awaitAll()
    }
    
    // MARK: - Performance Tests
    
    @Test
    fun `test basic performance`() = runTest {
        client.connect()
        
        val startTime = System.nanoTime()
        
        repeat(100) { i ->
            client.set("perf:$i", "value$i")
        }
        
        val endTime = System.nanoTime()
        val averageLatency = (endTime - startTime) / 100.0 / 1_000_000.0  // Convert to ms
        
        assertTrue(averageLatency < 10.0, "Average latency should be less than 10ms, was $averageLatency ms")
    }
        
        assertEquals("", value)
    }
    
    @Test
    fun `test unicode values`() = runTest {
        client.connect()
        
        val unicodeValue = "Hello 世界 🌍 Ñoël"
        client.set("unicode:key", unicodeValue)
        val value = client.get("unicode:key")
        
        assertEquals(unicodeValue, value)
    }
    
    @Test
    fun `test large value`() = runTest {
        client.connect()
        
        val largeValue = "x".repeat(10000)
        client.set("large:key", largeValue)
        val value = client.get("large:key")
        
        assertEquals(largeValue, value)
    }
    
    @Test
    fun `test tab characters in value`() = runTest {
        client.connect()
        
        val valueWithTab = "value\twith\ttabs"
        client.set("tab:key", valueWithTab)
        val value = client.get("tab:key")
        
        assertEquals(valueWithTab, value)
    }
    
    // MARK: - Error Handling Tests
    
    @Test
    fun `test empty key validation`() = runTest {
        client.connect()
        
        assertThrows<MerkleKVException.ValidationException> {
            client.set("", "value")
        }
    }
    
    @Test
    fun `test newline in key validation`() = runTest {
        client.connect()
        
        assertThrows<MerkleKVException.ValidationException> {
            client.set("key\nwith\nnewlines", "value")
        }
    }
    
    @Test
    fun `test newline in value validation`() = runTest {
        client.connect()
        
        assertThrows<MerkleKVException.ValidationException> {
            client.set("key", "value\nwith\nnewlines")
        }
    }
    
    @Test
    fun `test operation without connection`() = runTest {
        assertThrows<MerkleKVException.ConnectionException> {
            client.get("key")
        }
    }
    
    // MARK: - Connection Tests
    
    @Test
    fun `test connection status`() = runTest {
        assertFalse(client.connected)
        
        client.connect()
        assertTrue(client.connected)
        
        client.disconnect()
        assertFalse(client.connected)
    }
    
    @Test
    fun `test multiple connects are safe`() = runTest {
        client.connect()
        assertTrue(client.connected)
        
        client.connect() // Should not throw
        assertTrue(client.connected)
    }
    
    // MARK: - Batch Operations Tests
    
    @Test
    fun `test batch operations`() = runTest {
        client.connect()
        
        val operations = listOf(
            Operation.Set("batch:1", "value1"),
            Operation.Set("batch:2", "value2"),
            Operation.Get("batch:1"),
            Operation.Get("batch:2"),
            Operation.Delete("batch:1"),
            Operation.Get("batch:1")
        )
        
        val results = client.batch(operations)
        
        assertEquals(6, results.size)
        assertEquals(Unit, results[0]) // Set operation
        assertEquals(Unit, results[1]) // Set operation
        assertEquals("value1", results[2]) // Get operation
        assertEquals("value2", results[3]) // Get operation
        assertEquals(true, results[4]) // Delete operation
        assertEquals(null, results[5]) // Get deleted key
    }
    
    // MARK: - Concurrency Tests
    
    @Test
    fun `test concurrent operations`() = runTest {
        client.connect()
        
        val jobs = mutableListOf<Job>()
        
        repeat(50) { i ->
            val job = launch {
                val key = "concurrent:$i"
                val value = "value:$i"
                
                client.set(key, value)
                val retrieved = client.get(key)
                assertEquals(value, retrieved)
            }
            jobs.add(job)
        }
        
        jobs.joinAll()
    }
    
    // MARK: - Configuration Tests
    
    @Test
    fun `test custom configuration`() {
        val config = MerkleKVConfig(
            host = "localhost",
            port = 7379,
            timeout = 10.seconds,
            maxRetries = 5
        )
        
        val configClient = MerkleKVClient(config)
        assertNotNull(configClient)
        configClient.close()
    }
    
    // MARK: - Convenience Methods Tests
    
    @Test
    fun `test withConnection convenience method`() = runTest {
        val result = MerkleKVClient.withConnection { client ->
            client.set("convenience:test", "test:value")
            client.get("convenience:test")
        }
        
        assertEquals("test:value", result)
    }
    
    @Test
    fun `test withConnection with custom host and port`() = runTest {
        val result = MerkleKVClient.withConnection("localhost", 7379) { client ->
            client.set("convenience:test2", "test:value2")
            client.get("convenience:test2")
        }
        
        assertEquals("test:value2", result)
    }
    
    // MARK: - Performance Tests
    
    @Test
    fun `test performance basic operations`() = runTest {
        client.connect()
        
        val startTime = System.currentTimeMillis()
        
        repeat(100) { i ->
            val key = "perf:test:$i"
            val value = "value:$i"
            client.set(key, value)
            client.get(key)
        }
        
        val endTime = System.currentTimeMillis()
        val duration = endTime - startTime
        
        println("100 operations took ${duration}ms (${duration / 100.0}ms per operation)")
        assertTrue(duration < 5000, "Operations took too long: ${duration}ms")
    }
}

// MARK: - Exception Tests

class MerkleKVExceptionTest {
    
    @Test
    fun `test exception hierarchy`() {
        val exceptions = listOf(
            MerkleKVException.ConnectionException("test"),
            MerkleKVException.TimeoutException(),
            MerkleKVException.ProtocolException("test"),
            MerkleKVException.ValidationException("test"),
            MerkleKVException.InvalidResponseException("test"),
            MerkleKVException.NetworkException("test", RuntimeException())
        )
        
        exceptions.forEach { exception ->
            assertTrue(exception is MerkleKVException)
            assertNotNull(exception.message)
        }
    }
    
    @Test
    fun `test exception messages`() {
        val connectionEx = MerkleKVException.ConnectionException("Connection failed")
        assertEquals("Connection failed", connectionEx.message)
        
        val timeoutEx = MerkleKVException.TimeoutException("Custom timeout message")
        assertEquals("Custom timeout message", timeoutEx.message)
        
        val protocolEx = MerkleKVException.ProtocolException("Invalid protocol")
        assertEquals("Invalid protocol", protocolEx.message)
        
        val validationEx = MerkleKVException.ValidationException("Invalid input")
        assertEquals("Invalid input", validationEx.message)
        
        val invalidResponseEx = MerkleKVException.InvalidResponseException("BAD_RESPONSE")
        assertEquals("Invalid response: BAD_RESPONSE", invalidResponseEx.message)
        
        val networkEx = MerkleKVException.NetworkException("Network error", RuntimeException("cause"))
        assertEquals("Network error", networkEx.message)
        assertNotNull(networkEx.cause)
    }
}
