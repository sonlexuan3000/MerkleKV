package io.merklekv.examples

import akka.actor.ActorSystem
import akka.stream.Materializer
import io.merklekv.client._

import scala.concurrent.{ExecutionContext, Future}
import scala.concurrent.duration._
import scala.util.{Failure, Success}

/**
 * Basic example demonstrating MerkleKV Scala client usage
 */
object BasicExample extends App {
  
  implicit val system: ActorSystem = ActorSystem("merkle-example")
  implicit val materializer: Materializer = Materializer(system)
  implicit val ec: ExecutionContext = system.dispatcher
  
  println("🚀 MerkleKV Scala Client Example")
  println("Connecting to MerkleKV server...")
  
  val client = new MerkleKVClient("localhost", 7379)
  
  val program = for {
    _ <- client.connect()
    _ = println("✅ Connected successfully!")
    
    // Basic operations
    _ = println("\n📝 Basic Operations:")
    _ <- client.set("scala:example", "Hello from Scala!")
    _ = println("SET scala:example = 'Hello from Scala!'")
    
    value <- client.get("scala:example")
    _ = println(s"GET scala:example = '$value'")
    
    // Update value
    _ <- client.set("scala:example", "Updated value")
    updatedValue <- client.get("scala:example")
    _ = println(s"GET scala:example = '$updatedValue' (after update)")
    
    // Delete operation
    deleted <- client.delete("scala:example")
    _ = println(s"DELETE scala:example = $deleted")
    
    // Verify deletion
    deletedValue <- client.get("scala:example")
    _ = println(s"GET scala:example = $deletedValue (after deletion)")
    
    // Unicode and special characters
    _ = println("\n🌍 Unicode Support:")
    _ <- client.set("unicode", "Hello 世界! 🚀 Café Ñoël")
    unicodeValue <- client.get("unicode")
    _ = println(s"Unicode value: '$unicodeValue'")
    
    // Empty values
    _ = println("\n📭 Empty Values:")
    _ <- client.set("empty", "")
    emptyValue <- client.get("empty")
    _ = println(s"Empty value: '$emptyValue' (length: ${emptyValue.map(_.length)})")
    
    // Large values
    _ = println("\n📊 Large Values:")
    largeValue = "Scala" * 1000
    _ <- client.set("large", largeValue)
    retrievedLarge <- client.get("large")
    _ = println(s"Large value stored and retrieved (length: ${retrievedLarge.map(_.length)})")
    
    // Batch operations
    _ = println("\n📦 Batch Operations:")
    operations = List(
      Operation.Set("batch:1", "value1"),
      Operation.Set("batch:2", "value2"),
      Operation.Set("batch:3", "value3"),
      Operation.Get("batch:1"),
      Operation.Get("batch:2"),
      Operation.Get("batch:3")
    )
    results <- client.batch(operations)
    _ = println(s"Batch operations completed. Results: ${results.length} operations processed")
    
    // Multiple operations
    _ = println("\n🔄 Multiple Operations:")
    pairs = Map("multi:1" -> "value1", "multi:2" -> "value2", "multi:3" -> "value3")
    _ <- client.mset(pairs)
    mgetResult <- client.mget(List("multi:1", "multi:2", "multi:3"))
    _ = println(s"MGET result: $mgetResult")
    
    mdelResult <- client.mdel(List("multi:1", "multi:2", "multi:3"))
    _ = println(s"MDEL result: $mdelResult")
    
    // Cleanup
    _ <- client.delete("unicode")
    _ <- client.delete("empty")
    _ <- client.delete("large")
    _ <- client.delete("batch:1")
    _ <- client.delete("batch:2")
    _ <- client.delete("batch:3")
    
    _ <- client.close()
    _ = println("\n✅ Example completed successfully!")
    
  } yield ()
  
  program.onComplete {
    case Success(_) =>
      system.terminate()
    case Failure(ex: MerkleKVException.ConnectionException) =>
      println(s"❌ Connection failed: ${ex.getMessage}")
      println("Make sure MerkleKV server is running on localhost:7379")
      system.terminate()
    case Failure(ex: MerkleKVException.TimeoutException) =>
      println(s"❌ Operation timed out: ${ex.getMessage}")
      system.terminate()
    case Failure(ex) =>
      println(s"❌ Error: ${ex.getMessage}")
      ex.printStackTrace()
      system.terminate()
  }
}

/**
 * Example demonstrating functional programming patterns
 */
object FunctionalExample extends App {
  
  implicit val system: ActorSystem = ActorSystem("merkle-functional")
  implicit val materializer: Materializer = Materializer(system)
  implicit val ec: ExecutionContext = system.dispatcher
  
  println("\n🎯 Functional Programming Example:")
  
  val program = MerkleKVClient.withConnection("localhost", 7379) { client =>
    
    // Functional composition with for-comprehension
    val pipeline = for {
      // Set up test data
      _ <- client.mset(Map(
        "func:1" -> "functional",
        "func:2" -> "programming",
        "func:3" -> "scala"
      ))
      
      // Retrieve all values
      values <- client.mget(List("func:1", "func:2", "func:3"))
      
      // Transform the results
      concatenated = values.values.mkString(" ")
      
      // Store the result
      _ <- client.set("func:result", concatenated)
      
      // Retrieve the final result
      result <- client.get("func:result")
      
      // Cleanup
      _ <- client.mdel(List("func:1", "func:2", "func:3", "func:result"))
      
    } yield result
    
    pipeline.map { result =>
      println(s"Functional pipeline result: $result")
    }
  }
  
  program.onComplete {
    case Success(_) =>
      println("✅ Functional example completed!")
      system.terminate()
    case Failure(ex) =>
      println(s"❌ Functional example error: ${ex.getMessage}")
      system.terminate()
  }
}

/**
 * Example demonstrating error handling patterns
 */
object ErrorHandlingExample extends App {
  
  implicit val system: ActorSystem = ActorSystem("merkle-errors")
  implicit val materializer: Materializer = Materializer(system)
  implicit val ec: ExecutionContext = system.dispatcher
  
  println("\n🛡️ Error Handling Example:")
  
  val client = new MerkleKVClient("localhost", 7379)
  
  val program = client.connect().flatMap { _ =>
    
    // Test validation errors
    val validationTests = List(
      ("Empty key", () => client.set("", "value")),
      ("Key with newlines", () => client.set("key\nwith\nnewlines", "value")),
      ("Value with newlines", () => client.set("key", "value\nwith\nnewlines"))
    )
    
    Future.traverse(validationTests) { case (testName, operation) =>
      operation().recover {
        case ex: MerkleKVException.ValidationException =>
          println(s"✅ Caught validation error for $testName: ${ex.getMessage}")
        case ex =>
          println(s"❌ Unexpected error for $testName: ${ex.getMessage}")
      }
    }.flatMap { _ =>
      
      // Test operation without connection
      val disconnectedClient = new MerkleKVClient("localhost", 7379)
      disconnectedClient.get("key").recover {
        case ex: MerkleKVException.ConnectionException =>
          println(s"✅ Caught connection error: ${ex.getMessage}")
        case ex =>
          println(s"❌ Unexpected error: ${ex.getMessage}")
      }
    }
  }
  
  program.onComplete {
    case Success(_) =>
      println("✅ Error handling tests completed!")
      client.close()
      system.terminate()
    case Failure(ex) =>
      println(s"❌ Error handling example failed: ${ex.getMessage}")
      system.terminate()
  }
}

/**
 * Example demonstrating configuration options
 */
object ConfigurationExample extends App {
  
  implicit val system: ActorSystem = ActorSystem("merkle-config")
  implicit val materializer: Materializer = Materializer(system)
  implicit val ec: ExecutionContext = system.dispatcher
  
  println("\n⚙️ Configuration Example:")
  
  val config = MerkleKVConfig(
    host = "localhost",
    port = 7379,
    timeout = 10.seconds,
    maxRetries = 5,
    keepAlive = true
  )
  
  val program = MerkleKVClient.withConnection(config) { client =>
    for {
      _ <- client.set("config:test", "Custom configuration")
      value <- client.get("config:test")
      _ <- client.delete("config:test")
    } yield {
      println(s"Configuration test: $value")
    }
  }
  
  program.onComplete {
    case Success(_) =>
      println("✅ Configuration example completed!")
      system.terminate()
    case Failure(ex) =>
      println(s"❌ Configuration example error: ${ex.getMessage}")
      system.terminate()
  }
}
