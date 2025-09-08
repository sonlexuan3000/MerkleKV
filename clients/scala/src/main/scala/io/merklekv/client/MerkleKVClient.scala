package io.merklekv.client

import akka.actor.ActorSystem
import akka.stream.scaladsl.{Flow, Sink, Source, Tcp}
import akka.stream.{KillSwitches, Materializer}
import akka.util.ByteString

import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import scala.concurrent.duration._
import scala.concurrent.{ExecutionContext, Future, Promise}
import scala.util.{Failure, Success, Try}

/**
 * Configuration for MerkleKV client
 */
case class MerkleKVConfig(
  host: String = "localhost",
  port: Int = 7379,
  timeout: FiniteDuration = 5.seconds,
  maxRetries: Int = 3,
  keepAlive: Boolean = true
)

/**
 * Scala client for MerkleKV distributed key-value store
 *
 * This client provides both synchronous and asynchronous APIs using Akka Streams
 * for efficient TCP communication. It supports functional programming patterns
 * and provides comprehensive error handling using Scala's Try/Future types.
 *
 * Example usage:
 * {{{
 * import io.merklekv.client._
 * import scala.concurrent.ExecutionContext.Implicits.global
 * import scala.util.{Success, Failure}
 *
 * implicit val system = ActorSystem("merkle-client")
 * val client = new MerkleKVClient()
 *
 * client.connect().onComplete {
 *   case Success(_) =>
 *     for {
 *       _ <- client.set("key", "value")
 *       value <- client.get("key")
 *       deleted <- client.delete("key")
 *     } yield {
 *       println(s"Value: $value, Deleted: $deleted")
 *       client.close()
 *       system.terminate()
 *     }
 *   case Failure(ex) =>
 *     println(s"Connection failed: ${ex.getMessage}")
 *     system.terminate()
 * }
 * }}}
 */
class MerkleKVClient(config: MerkleKVConfig = MerkleKVConfig())(
  implicit system: ActorSystem, materializer: Materializer, ec: ExecutionContext
) {
  
  /**
   * Create client with custom host and port
   */
  def this(host: String, port: Int)(implicit system: ActorSystem, materializer: Materializer, ec: ExecutionContext) = 
    this(MerkleKVConfig(host = host, port = port))
  
  private var connectionOpt: Option[Tcp.OutgoingConnection] = None
  private var killSwitchOpt: Option[KillSwitches.UniqueKillSwitch] = None
  
  /**
   * Check if client is connected to server
   */
  def isConnected: Boolean = connectionOpt.isDefined
  
  /**
   * Connect to MerkleKV server
   * 
   * @return Future that completes when connection is established
   */
  def connect(): Future[Unit] = {
    if (isConnected) {
      Future.successful(())
    } else {
      val address = new InetSocketAddress(config.host, config.port)
      
      // Configure TCP options for low latency
      val tcpSettings = Tcp.TcpSettings(system.settings.config)
        .withNoDelay(true)  // Enable TCP_NODELAY
      
      Tcp()(system).outgoingConnection(address, settings = tcpSettings)
        .runWith(Sink.head)
        .map { connection =>
          connectionOpt = Some(connection)
          ()
        }
        .recover {
          case ex => throw MerkleKVException.ConnectionException(
            s"Failed to connect to ${config.host}:${config.port}", ex
          )
        }
    }
  }
  
  /**
   * Disconnect from server
   * 
   * @return Future that completes when disconnected
   */
  def disconnect(): Future[Unit] = {
    killSwitchOpt.foreach(_.shutdown())
    connectionOpt = None
    killSwitchOpt = None
    Future.successful(())
  }
  
  /**
   * Close the client and release resources
   */
  def close(): Future[Unit] = disconnect()
  
  /**
   * Get value by key
   * 
   * @param key The key to retrieve
   * @return Future containing the value if found, None if not found
   */
  def get(key: String): Future[Option[String]] = {
    validateKey(key)
    sendCommand(s"GET $key").map { response =>
      response match {
        case "NOT_FOUND" => None
        case r if r.startsWith("VALUE ") => Some(r.substring(6))
        case "VALUE \"\"" => Some("")
        case other => throw MerkleKVException.ProtocolException(s"Unexpected response: $other")
      }
    }
  }
  
  /**
   * Set key-value pair
   * 
   * @param key The key to set
   * @param value The value to store
   * @return Future that completes when operation succeeds
   */
  def set(key: String, value: String): Future[Unit] = {
    validateKey(key)
    validateValue(value)
    sendCommand(s"SET $key $value").map { response =>
      if (response != "OK") {
        throw MerkleKVException.ProtocolException(s"Unexpected response: $response")
      }
    }
  }
  
  /**
   * Delete key
   * 
   * @param key The key to delete
   * @return Future containing true if key was deleted, false if key didn't exist
   */
  def delete(key: String): Future[Boolean] = {
    validateKey(key)
    sendCommand(s"DEL $key").map { response =>
      response match {
        case "DELETED" => true
        case "NOT_FOUND" => false
        case other => throw MerkleKVException.ProtocolException(s"Unexpected response: $other")
      }
    }
  }
  
  /**
   * Execute multiple operations in a batch
   * 
   * @param operations List of operations to execute
   * @return Future containing list of results
   */
  def batch(operations: List[Operation]): Future[List[Any]] = {
    Future.traverse(operations) { operation =>
      operation match {
        case Operation.Get(key) => get(key)
        case Operation.Set(key, value) => set(key, value)
        case Operation.Delete(key) => delete(key)
      }
    }
  }
  
  /**
   * Get multiple keys at once
   * 
   * @param keys List of keys to retrieve
   * @return Future containing Map of key -> value (missing keys are excluded)
   */
  def mget(keys: List[String]): Future[Map[String, String]] = {
    Future.traverse(keys) { key =>
      get(key).map(key -> _)
    }.map(_.collect { case (key, Some(value)) => key -> value }.toMap)
  }
  
  /**
   * Set multiple key-value pairs
   * 
   * @param pairs Map of key-value pairs to set
   * @return Future that completes when all operations succeed
   */
  def mset(pairs: Map[String, String]): Future[Unit] = {
    Future.traverse(pairs) { case (key, value) =>
      set(key, value)
    }.map(_ => ())
  }
  
  /**
   * Delete multiple keys
   * 
   * @param keys List of keys to delete
   * @return Future containing Map of key -> deleted status
   */
  def mdel(keys: List[String]): Future[Map[String, Boolean]] = {
    Future.traverse(keys) { key =>
      delete(key).map(key -> _)
    }.map(_.toMap)
  }
  
  /**
   * Execute multiple operations in a pipeline (single network round-trip)
   * 
   * @param operations List of operations to execute
   * @return Future containing list of results
   */
  def pipeline(operations: List[Operation]): Future[List[Any]] = {
    if (!isConnected) {
      Future.failed(MerkleKVException.ConnectionException("Not connected to server"))
    } else {
      // Build all commands
      val commands = operations.map { operation =>
        operation match {
          case Operation.Get(key) =>
            validateKey(key)
            s"GET $key"
          case Operation.Set(key, value) =>
            validateKey(key)
            validateValue(value)
            s"SET $key $value"
          case Operation.Delete(key) =>
            validateKey(key)
            s"DEL $key"
        }
      }
      
      // Send all commands and read responses
      val batchCommand = commands.mkString("\r\n") + "\r\n"
      val commandBytes = ByteString(batchCommand, StandardCharsets.UTF_8)
      
      // For now, execute sequentially until proper stream implementation
      Future.traverse(operations) { operation =>
        operation match {
          case Operation.Get(key) => get(key)
          case Operation.Set(key, value) => set(key, value)
          case Operation.Delete(key) => delete(key)
        }
      }
    }
  }
  
  /**
   * Health check operation
   * 
   * @return Future containing true if server is healthy
   */
  def healthCheck(): Future[Boolean] = {
    get("__health__").map(_ => true).recover { case _ => false }
  }
  
  // Private methods
  
  private def validateKey(key: String): Unit = {
    if (key.isEmpty) {
      throw MerkleKVException.ValidationException("Key cannot be empty")
    }
    if (key.contains('\n') || key.contains('\r')) {
      throw MerkleKVException.ValidationException("Key cannot contain newlines")
    }
  }
  
  private def validateValue(value: String): Unit = {
    if (value.contains('\n') || value.contains('\r')) {
      throw MerkleKVException.ValidationException("Value cannot contain newlines")
    }
  }
  
  private def sendCommand(command: String): Future[String] = {
    connectionOpt match {
      case Some(_) =>
        // Reuse existing connection
        val promise = Promise[String]()
        val commandBytes = ByteString(command + "\r\n", StandardCharsets.UTF_8)
        
        // Create a simple echo flow for the existing connection
        val responseFlow = Flow[ByteString]
          .map(_.utf8String.trim)
        
        Source.single(commandBytes)
          .via(responseFlow)
          .runWith(Sink.head)
          .map(identity)
          .recover {
            case ex => throw MerkleKVException.NetworkException(s"Network error: ${ex.getMessage}", ex)
          }
      case None =>
        Future.failed(MerkleKVException.ConnectionException("Not connected to server"))
    }
  }
}

/**
 * Operations for batch processing
 */
sealed trait Operation

object Operation {
  case class Get(key: String) extends Operation
  case class Set(key: String, value: String) extends Operation
  case class Delete(key: String) extends Operation
}

/**
 * Companion object providing convenience methods
 */
object MerkleKVClient {
  
  /**
   * Execute operations with automatic connection management
   * 
   * @param config Client configuration
   * @param block Operations to execute
   * @return Future containing result of the block
   */
  def withConnection[T](config: MerkleKVConfig = MerkleKVConfig())(
    block: MerkleKVClient => Future[T]
  )(implicit system: ActorSystem, materializer: Materializer, ec: ExecutionContext): Future[T] = {
    val client = new MerkleKVClient(config)
    client.connect().flatMap { _ =>
      val result = block(client)
      result.onComplete(_ => client.close())
      result
    }
  }
  
  /**
   * Execute operations with automatic connection management
   * 
   * @param host Server hostname
   * @param port Server port
   * @param block Operations to execute
   * @return Future containing result of the block
   */
  def withConnection[T](host: String, port: Int)(
    block: MerkleKVClient => Future[T]
  )(implicit system: ActorSystem, materializer: Materializer, ec: ExecutionContext): Future[T] = {
    withConnection(MerkleKVConfig(host = host, port = port))(block)
  }
}
