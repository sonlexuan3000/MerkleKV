package io.merklekv.client

import scala.util.control.NoStackTrace

/**
 * Base exception for all MerkleKV client errors
 */
sealed abstract class MerkleKVException(message: String, cause: Throwable = null) 
  extends Exception(message, cause) with NoStackTrace

object MerkleKVException {
  
  /**
   * Connection to MerkleKV server failed
   */
  final case class ConnectionException(message: String, cause: Throwable = null) 
    extends MerkleKVException(message, cause)
  
  /**
   * Operation timed out
   */
  final case class TimeoutException(message: String = "Operation timed out") 
    extends MerkleKVException(message)
  
  /**
   * Protocol-level error from server
   */
  final case class ProtocolException(message: String) 
    extends MerkleKVException(message)
  
  /**
   * Client-side validation error
   */
  final case class ValidationException(message: String) 
    extends MerkleKVException(message)
  
  /**
   * Invalid response from server
   */
  final case class InvalidResponseException(response: String) 
    extends MerkleKVException(s"Invalid response: $response")
  
  /**
   * Network I/O error
   */
  final case class NetworkException(message: String, cause: Throwable) 
    extends MerkleKVException(message, cause)
}
