package io.merklekv.examples

import io.merklekv.client.MerkleKVClient
import io.merklekv.client.MerkleKVConfig
import io.merklekv.client.MerkleKVException
import io.merklekv.client.Operation
import kotlinx.coroutines.*
import kotlin.time.Duration.Companion.seconds

/**
 * Basic example demonstrating MerkleKV Kotlin client usage
 */
suspend fun main() {
    try {
        println("🚀 MerkleKV Kotlin Client Example")
        println("Connecting to MerkleKV server...")
        
        val client = MerkleKVClient("localhost", 7379)
        client.use { client ->
            client.connect()
            println("✅ Connected successfully!")
            
            // Basic operations
            println("\n📝 Basic Operations:")
            
            // Set operation
            client.set("kotlin:example", "Hello from Kotlin!")
            println("SET kotlin:example = 'Hello from Kotlin!'")
            
            // Get operation
            val value = client.get("kotlin:example")
            println("GET kotlin:example = '$value'")
            
            // Update value
            client.set("kotlin:example", "Updated value")
            val updatedValue = client.get("kotlin:example")
            println("GET kotlin:example = '$updatedValue' (after update)")
            
            // Delete operation
            val deleted = client.delete("kotlin:example")
            println("DELETE kotlin:example = $deleted")
            
            // Verify deletion
            val deletedValue = client.get("kotlin:example")
            println("GET kotlin:example = $deletedValue (after deletion)")
            
            // Unicode and special characters
            println("\n🌍 Unicode Support:")
            client.set("unicode", "Hello 世界! 🚀 Café Ñoël")
            val unicodeValue = client.get("unicode")
            println("Unicode value: '$unicodeValue'")
            
            // Empty values
            println("\n📭 Empty Values:")
            client.set("empty", "")
            val emptyValue = client.get("empty")
            println("Empty value: '$emptyValue' (length: ${emptyValue?.length})")
            
            // Large values
            println("\n📊 Large Values:")
            val largeValue = "Kotlin".repeat(1000)
            client.set("large", largeValue)
            val retrievedLarge = client.get("large")
            println("Large value stored and retrieved (length: ${retrievedLarge?.length})")
            
            // Batch operations
            println("\n📦 Batch Operations:")
            val operations = listOf(
                Operation.Set("batch:1", "value1"),
                Operation.Set("batch:2", "value2"),
                Operation.Set("batch:3", "value3"),
                Operation.Get("batch:1"),
                Operation.Get("batch:2"),
                Operation.Get("batch:3")
            )
            
            val results = client.batch(operations)
            println("Batch operations completed. Results: ${results.size} operations processed")
            
            // Cleanup
            client.delete("unicode")
            client.delete("empty")
            client.delete("large")
            client.delete("batch:1")
            client.delete("batch:2")
            client.delete("batch:3")
            
            println("\n✅ Example completed successfully!")
        }
        
    } catch (e: MerkleKVException.ConnectionException) {
        println("❌ Connection failed: ${e.message}")
        println("Make sure MerkleKV server is running on localhost:7379")
    } catch (e: MerkleKVException.TimeoutException) {
        println("❌ Operation timed out: ${e.message}")
    } catch (e: Exception) {
        println("❌ Error: ${e.message}")
        e.printStackTrace()
    }
}

/**
 * Example demonstrating coroutine-based concurrent operations
 */
suspend fun concurrencyExample() {
    println("\n🔄 Concurrency Example:")
    
    try {
        MerkleKVClient.withConnection { client ->
            // Launch multiple coroutines for concurrent operations
            coroutineScope {
                val jobs = mutableListOf<Job>()
                
                repeat(10) { i ->
                    val job = launch {
                        val key = "concurrent:$i"
                        val value = "value:$i"
                        
                        client.set(key, value)
                        val retrieved = client.get(key)
                        println("Thread ${Thread.currentThread().name}: $key = $retrieved")
                    }
                    jobs.add(job)
                }
                
                jobs.joinAll()
                println("All concurrent operations completed!")
                
                // Cleanup
                repeat(10) { i ->
                    client.delete("concurrent:$i")
                }
            }
        }
    } catch (e: Exception) {
        println("❌ Concurrency example error: ${e.message}")
    }
}

/**
 * Example demonstrating custom configuration
 */
suspend fun configurationExample() {
    println("\n⚙️ Configuration Example:")
    
    val config = MerkleKVConfig(
        host = "localhost",
        port = 7379,
        timeout = 10.seconds,
        maxRetries = 5,
        keepAlive = true
    )
    
    try {
        MerkleKVClient(config).use { client ->
            client.connect()
            client.set("config:test", "Custom configuration")
            val value = client.get("config:test")
            println("Configuration test: $value")
            client.delete("config:test")
        }
    } catch (e: Exception) {
        println("❌ Configuration example error: ${e.message}")
    }
}

/**
 * Example demonstrating error handling
 */
suspend fun errorHandlingExample() {
    println("\n🛡️ Error Handling Example:")
    
    try {
        val client = MerkleKVClient("localhost", 7379)
        client.use { client ->
            client.connect()
            
            // Test validation errors
            try {
                client.set("", "empty key")
            } catch (e: MerkleKVException.ValidationException) {
                println("Caught validation error: ${e.message}")
            }
            
            try {
                client.set("key\nwith\nnewlines", "value")
            } catch (e: MerkleKVException.ValidationException) {
                println("Caught validation error: ${e.message}")
            }
            
            try {
                client.set("key", "value\nwith\nnewlines")
            } catch (e: MerkleKVException.ValidationException) {
                println("Caught validation error: ${e.message}")
            }
            
            println("Error handling tests completed!")
        }
    } catch (e: Exception) {
        println("❌ Error handling example error: ${e.message}")
    }
}
