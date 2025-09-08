package io.merklekv.client

/**
 * Exceptions thrown by the MerkleKV client
 */
sealed class MerkleKVException(message: String, cause: Throwable? = null) : Exception(message, cause) {
    
    /**
     * Connection to MerkleKV server failed
     */
    class ConnectionException(message: String, cause: Throwable? = null) : MerkleKVException(message, cause)
    
    /**
     * Operation timed out
     */
    class TimeoutException(message: String = "Operation timed out") : MerkleKVException(message)
    
    /**
     * Protocol-level error from server
     */
    class ProtocolException(message: String) : MerkleKVException(message)
    
    /**
     * Client-side validation error
     */
    class ValidationException(message: String) : MerkleKVException(message)
    
    /**
     * Invalid response from server
     */
    class InvalidResponseException(response: String) : MerkleKVException("Invalid response: $response")
    
    /**
     * Network I/O error
     */
    class NetworkException(message: String, cause: Throwable) : MerkleKVException(message, cause)
}
