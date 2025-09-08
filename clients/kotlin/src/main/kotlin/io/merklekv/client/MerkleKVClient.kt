package io.merklekv.client

import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.io.*
import java.net.Socket
import java.net.SocketTimeoutException
import java.nio.charset.StandardCharsets
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

/**
 * Configuration for MerkleKV client
 */
data class MerkleKVConfig(
    val host: String = "localhost",
    val port: Int = 7379,
    val timeout: Duration = 5.seconds,
    val maxRetries: Int = 3,
    val keepAlive: Boolean = true
)

/**
 * Kotlin client for MerkleKV distributed key-value store
 *
 * This client provides both synchronous and asynchronous (coroutine-based) APIs
 * for interacting with MerkleKV servers. It supports connection pooling, automatic
 * retry logic, and comprehensive error handling.
 *
 * ## Basic Usage
 * ```kotlin
 * val client = MerkleKVClient("localhost", 7379)
 * client.use { client ->
 *     runBlocking {
 *         client.connect()
 *         client.set("key", "value")
 *         val value = client.get("key")
 *         client.delete("key")
 *     }
 * }
 * ```
 *
 * ## Coroutine Usage
 * ```kotlin
 * val client = MerkleKVClient()
 * client.use { client ->
 *     client.connect()
 *     
 *     // Concurrent operations
 *     coroutineScope {
 *         launch { client.set("key1", "value1") }
 *         launch { client.set("key2", "value2") }
 *     }
 * }
 * ```
 */
class MerkleKVClient(
    private val config: MerkleKVConfig
) : AutoCloseable {
    
    /**
     * Create client with default configuration
     */
    constructor() : this(MerkleKVConfig())
    
    /**
     * Create client with custom host and port
     */
    constructor(host: String, port: Int = 7379) : this(
        MerkleKVConfig(host = host, port = port)
    )
    
    private var socket: Socket? = null
    private var inputStream: BufferedReader? = null
    private var outputStream: PrintWriter? = null
    private val connectionMutex = Mutex()
    private var isConnected = false
    
    /**
     * Check if client is connected to server
     */
    val connected: Boolean
        get() = isConnected && socket?.isConnected == true && socket?.isClosed == false
    
    /**
     * Connect to MerkleKV server
     * 
     * @throws MerkleKVException.ConnectionException if connection fails
     * @throws MerkleKVException.TimeoutException if connection times out
     */
    suspend fun connect() {
        connectionMutex.withLock {
            if (connected) return
            
            try {
                withTimeout(config.timeout) {
                    val newSocket = withContext(Dispatchers.IO) {
                        Socket(config.host, config.port).apply {
                            keepAlive = config.keepAlive
                            tcpNoDelay = true  // Enable TCP_NODELAY for low latency
                            soTimeout = config.timeout.inWholeMilliseconds.toInt()
                        }
                    }
                    
                    socket = newSocket
                    inputStream = newSocket.getInputStream().bufferedReader(StandardCharsets.UTF_8)
                    outputStream = PrintWriter(
                        newSocket.getOutputStream().bufferedWriter(StandardCharsets.UTF_8),
                        true
                    )
                    isConnected = true
                }
            } catch (e: TimeoutCancellationException) {
                throw MerkleKVException.TimeoutException("Connection timeout")
            } catch (e: Exception) {
                throw MerkleKVException.ConnectionException("Failed to connect to ${config.host}:${config.port}", e)
            }
        }
    }
    
    /**
     * Disconnect from server
     */
    suspend fun disconnect() {
        connectionMutex.withLock {
            try {
                outputStream?.close()
                inputStream?.close()
                socket?.close()
            } catch (e: Exception) {
                // Ignore errors during cleanup
            } finally {
                outputStream = null
                inputStream = null
                socket = null
                isConnected = false
            }
        }
    }
    
    /**
     * Get value by key
     * 
     * @param key The key to retrieve
     * @return The value if found, null if not found
     * @throws MerkleKVException on error
     */
    suspend fun get(key: String): String? {
        validateKey(key)
        val response = sendCommand("GET $key")
        
        return when {
            response == "NOT_FOUND" -> null
            response.startsWith("VALUE ") -> response.substring(6)
            response == "VALUE \"\"" -> ""
            else -> throw MerkleKVException.ProtocolException("Unexpected response: $response")
        }
    }
    
    /**
     * Set key-value pair
     * 
     * @param key The key to set
     * @param value The value to store
     * @throws MerkleKVException on error
     */
    suspend fun set(key: String, value: String) {
        validateKey(key)
        validateValue(value)
        val response = sendCommand("SET $key $value")
        
        if (response != "OK") {
            throw MerkleKVException.ProtocolException("Unexpected response: $response")
        }
    }
    
    /**
     * Delete key
     * 
     * @param key The key to delete
     * @return true if key was deleted, false if key didn't exist
     * @throws MerkleKVException on error
     */
    suspend fun delete(key: String): Boolean {
        validateKey(key)
        val response = sendCommand("DEL $key")
        
        return when (response) {
            "DELETED" -> true
            "NOT_FOUND" -> false
            else -> throw MerkleKVException.ProtocolException("Unexpected response: $response")
        }
    }
    
    /**
     * Execute multiple operations in a batch
     * 
     * @param operations List of operations to execute
     * @return List of results corresponding to each operation
     */
    suspend fun batch(operations: List<Operation>): List<Any?> {
        return operations.map { operation ->
            when (operation) {
                is Operation.Get -> get(operation.key)
                is Operation.Set -> set(operation.key, operation.value)
                is Operation.Delete -> delete(operation.key)
            }
        }
    }
    
    /**
     * Execute multiple operations in a pipeline (single network round-trip)
     * 
     * @param operations List of operations to execute
     * @return List of results corresponding to each operation
     */
    suspend fun pipeline(operations: List<Operation>): List<Any?> {
        if (!connected) {
            throw MerkleKVException.ConnectionException("Not connected to server")
        }
        
        return withContext(Dispatchers.IO) {
            try {
                withTimeout(config.timeout) {
                    // Build all commands
                    val commands = operations.map { operation ->
                        when (operation) {
                            is Operation.Get -> {
                                validateKey(operation.key)
                                "GET ${operation.key}"
                            }
                            is Operation.Set -> {
                                validateKey(operation.key)
                                validateValue(operation.value)
                                "SET ${operation.key} ${operation.value}"
                            }
                            is Operation.Delete -> {
                                validateKey(operation.key)
                                "DEL ${operation.key}"
                            }
                        }
                    }
                    
                    // Send all commands in one batch
                    for (command in commands) {
                        outputStream?.print(command + "\r\n")
                    }
                    outputStream?.flush()
                    
                    // Read all responses
                    val results = mutableListOf<Any?>()
                    for ((index, operation) in operations.withIndex()) {
                        val response = inputStream?.readLine()
                            ?: throw MerkleKVException.NetworkException("No response from server", IOException())
                        
                        when (operation) {
                            is Operation.Get -> {
                                when {
                                    response == "NOT_FOUND" -> results.add(null)
                                    response.startsWith("VALUE ") -> results.add(response.substring(6))
                                    response == "VALUE \"\"" -> results.add("")
                                    else -> throw MerkleKVException.ProtocolException("Unexpected response: $response")
                                }
                            }
                            is Operation.Set -> {
                                if (response == "OK") {
                                    results.add(Unit)
                                } else {
                                    throw MerkleKVException.ProtocolException("Unexpected response: $response")
                                }
                            }
                            is Operation.Delete -> {
                                when (response) {
                                    "DELETED" -> results.add(true)
                                    "NOT_FOUND" -> results.add(false)
                                    else -> throw MerkleKVException.ProtocolException("Unexpected response: $response")
                                }
                            }
                        }
                    }
                    
                    results
                }
            } catch (e: TimeoutCancellationException) {
                throw MerkleKVException.TimeoutException("Pipeline timeout")
            } catch (e: SocketTimeoutException) {
                throw MerkleKVException.TimeoutException("Socket timeout")
            } catch (e: IOException) {
                throw MerkleKVException.NetworkException("Network error", e)
            }
        }
    }
    
    /**
     * Health check operation
     * 
     * @return true if server is healthy
     */
    suspend fun healthCheck(): Boolean {
        return try {
            get("__health__")
            true
        } catch (e: Exception) {
            false
        }
    }
    
    /**
     * Close the client and release resources
     */
    override fun close() {
        runBlocking {
            disconnect()
        }
    }
    
    // Private methods
    
    private fun validateKey(key: String) {
        if (key.isEmpty()) {
            throw MerkleKVException.ValidationException("Key cannot be empty")
        }
        if (key.contains('\n') || key.contains('\r')) {
            throw MerkleKVException.ValidationException("Key cannot contain newlines")
        }
    }
    
    private fun validateValue(value: String) {
        if (value.contains('\n') || value.contains('\r')) {
            throw MerkleKVException.ValidationException("Value cannot contain newlines")
        }
    }
    
    private suspend fun sendCommand(command: String): String {
        if (!connected) {
            throw MerkleKVException.ConnectionException("Not connected to server")
        }
        
        return withContext(Dispatchers.IO) {
            try {
                withTimeout(config.timeout) {
                    // Send command with explicit CRLF
                    outputStream?.print(command + "\r\n")
                    outputStream?.flush()
                    
                    // Read response
                    val response = inputStream?.readLine()
                        ?: throw MerkleKVException.NetworkException("No response from server", IOException())
                    
                    response
                }
            } catch (e: TimeoutCancellationException) {
                throw MerkleKVException.TimeoutException("Command timeout: $command")
            } catch (e: SocketTimeoutException) {
                throw MerkleKVException.TimeoutException("Socket timeout")
            } catch (e: IOException) {
                throw MerkleKVException.NetworkException("Network error", e)
            }
        }
    }
    
    companion object {
        /**
         * Execute operations with automatic connection management
         * 
         * @param config Client configuration
         * @param block Operations to execute
         * @return Result of the block
         */
        suspend fun <T> withConnection(
            config: MerkleKVConfig = MerkleKVConfig(),
            block: suspend (MerkleKVClient) -> T
        ): T {
            return MerkleKVClient(config).use { client ->
                client.connect()
                block(client)
            }
        }
        
        /**
         * Execute operations with automatic connection management
         * 
         * @param host Server hostname
         * @param port Server port
         * @param block Operations to execute
         * @return Result of the block
         */
        suspend fun <T> withConnection(
            host: String = "localhost",
            port: Int = 7379,
            block: suspend (MerkleKVClient) -> T
        ): T = withConnection(MerkleKVConfig(host = host, port = port), block)
    }
}

/**
 * Operations for batch processing
 */
sealed class Operation {
    data class Get(val key: String) : Operation()
    data class Set(val key: String, val value: String) : Operation()
    data class Delete(val key: String) : Operation()
}
