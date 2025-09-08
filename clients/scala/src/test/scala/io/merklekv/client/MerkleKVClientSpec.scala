package io.merklekv.client

import akka.actor.ActorSystem
import akka.stream.Materializer
import org.scalatest.concurrent.ScalaFutures
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec
import org.scalatest.{BeforeAndAfterAll, BeforeAndAfterEach}

import scala.concurrent.ExecutionContext
import scala.concurrent.duration._
import scala.util.{Failure, Success, Try}

class MerkleKVClientSpec extends AnyWordSpec with Matchers with ScalaFutures with BeforeAndAfterAll with BeforeAndAfterEach {
  
  implicit val system: ActorSystem = ActorSystem("merkle-test")
  implicit val materializer: Materializer = Materializer(system)
  implicit val ec: ExecutionContext = system.dispatcher
  implicit val patience: PatienceConfig = PatienceConfig(timeout = 10.seconds, interval = 100.millis)
  
  private var client: MerkleKVClient = _
  private var serverAvailable = false
  
  private def getTestPort: Int = {
    sys.env.get("MERKLEKV_PORT").map(_.toInt).getOrElse(7379)
  }
  
  override def beforeAll(): Unit = {
    // Test server availability
    val testPort = getTestPort
    val testClient = new MerkleKVClient("localhost", testPort)
    Try {
      testClient.connect().futureValue
      serverAvailable = true
      testClient.close()
    } match {
      case Success(_) => 
        println(s"✅ MerkleKV server available at localhost:$testPort")
      case Failure(_) => 
        println(s"⚠️  MerkleKV server not available at localhost:$testPort - skipping integration tests")
    }
  }
  
  override def beforeEach(): Unit = {
    if (serverAvailable) {
      client = new MerkleKVClient("localhost", getTestPort)
      client.connect().futureValue
    }
  }
  
  override def afterEach(): Unit = {
    if (serverAvailable && client != null) {
      client.close().futureValue
    }
  }
  
  override def afterAll(): Unit = {
    system.terminate().futureValue
  }
  
  def assumeServerAvailable(): Unit = {
    assume(serverAvailable, "MerkleKV server not available")
  }
  
  "MerkleKVClient" should {
    
    "connect and disconnect successfully" in {
      assumeServerAvailable()
      
      val testClient = new MerkleKVClient("localhost", 7379)
      testClient.isConnected shouldBe false
      
      testClient.connect().futureValue
      testClient.isConnected shouldBe true
      
      testClient.disconnect().futureValue
      testClient.isConnected shouldBe false
    }
    
    "perform basic set and get operations" in {
      assumeServerAvailable()
      
      client.set("test:key", "test:value").futureValue
      val value = client.get("test:key").futureValue
      
      value shouldBe Some("test:value")
    }
    
    "return None for non-existent keys" in {
      assumeServerAvailable()
      
      val value = client.get("non:existent:key:12345").futureValue
      
      value shouldBe None
    }
    
    "delete existing keys" in {
      assumeServerAvailable()
      
      client.set("delete:test", "value").futureValue
      val deleted = client.delete("delete:test").futureValue
      deleted shouldBe true
      
      val value = client.get("delete:test").futureValue
      value shouldBe None
    }
    
    "return false when deleting non-existent keys" in {
      assumeServerAvailable()
      
      val deleted = client.delete("non:existent:delete:12345").futureValue
      
      deleted shouldBe false
    }
    
    "handle empty values" in {
      assumeServerAvailable()
      
      client.set("empty:key", "").futureValue
      val value = client.get("empty:key").futureValue
      
      value shouldBe Some("")
    }
    
    "handle unicode values" in {
      assumeServerAvailable()
      
      val unicodeValue = "Hello 世界 🌍 Ñoël"
      client.set("unicode:key", unicodeValue).futureValue
      val value = client.get("unicode:key").futureValue
      
      value shouldBe Some(unicodeValue)
    }
    
    "handle large values" in {
      assumeServerAvailable()
      
      val largeValue = "x" * 10000
      client.set("large:key", largeValue).futureValue
      val value = client.get("large:key").futureValue
      
      value shouldBe Some(largeValue)
    }
    
    "handle tab characters in values" in {
      assumeServerAvailable()
      
      val valueWithTab = "value\twith\ttabs"
      client.set("tab:key", valueWithTab).futureValue
      val value = client.get("tab:key").futureValue
      
      value shouldBe Some(valueWithTab)
    }
    
    "validate empty keys" in {
      assumeServerAvailable()
      
      an[MerkleKVException.ValidationException] should be thrownBy {
        client.set("", "value").futureValue
      }
    }
    
    "validate keys with newlines" in {
      assumeServerAvailable()
      
      an[MerkleKVException.ValidationException] should be thrownBy {
        client.set("key\nwith\nnewlines", "value").futureValue
      }
    }
    
    "validate values with newlines" in {
      assumeServerAvailable()
      
      an[MerkleKVException.ValidationException] should be thrownBy {
        client.set("key", "value\nwith\nnewlines").futureValue
      }
    }
    
    "fail operations when not connected" in {
      val disconnectedClient = new MerkleKVClient("localhost", 7379)
      
      an[MerkleKVException.ConnectionException] should be thrownBy {
        disconnectedClient.get("key").futureValue
      }
    }
    
    "perform batch operations" in {
      assumeServerAvailable()
      
      val operations = List(
        Operation.Set("batch:1", "value1"),
        Operation.Set("batch:2", "value2"),
        Operation.Get("batch:1"),
        Operation.Get("batch:2"),
        Operation.Delete("batch:1"),
        Operation.Get("batch:1")
      )
      
      val results = client.batch(operations).futureValue
      
      results should have length 6
      results(0) shouldBe (()) // Set operation
      results(1) shouldBe (()) // Set operation
      results(2) shouldBe Some("value1") // Get operation
      results(3) shouldBe Some("value2") // Get operation
      results(4) shouldBe true // Delete operation
      results(5) shouldBe None // Get deleted key
    }
    
    "perform multiple get operations" in {
      assumeServerAvailable()
      
      val pairs = Map(
        "mget:1" -> "value1",
        "mget:2" -> "value2",
        "mget:3" -> "value3"
      )
      
      client.mset(pairs).futureValue
      val result = client.mget(List("mget:1", "mget:2", "mget:3", "mget:missing")).futureValue
      
      result should contain only (
        "mget:1" -> "value1",
        "mget:2" -> "value2",
        "mget:3" -> "value3"
      )
    }
    
    "perform multiple set operations" in {
      assumeServerAvailable()
      
      val pairs = Map(
        "mset:1" -> "value1",
        "mset:2" -> "value2",
        "mset:3" -> "value3"
      )
      
      client.mset(pairs).futureValue
      
      client.get("mset:1").futureValue shouldBe Some("value1")
      client.get("mset:2").futureValue shouldBe Some("value2")
      client.get("mset:3").futureValue shouldBe Some("value3")
    }
    
    "perform multiple delete operations" in {
      assumeServerAvailable()
      
      // Set up test data
      client.set("mdel:1", "value1").futureValue
      client.set("mdel:2", "value2").futureValue
      
      val result = client.mdel(List("mdel:1", "mdel:2", "mdel:missing")).futureValue
      
      result should contain only (
        "mdel:1" -> true,
        "mdel:2" -> true,
        "mdel:missing" -> false
      )
    }
    
    "work with convenience withConnection method" in {
      assumeServerAvailable()
      
      val result = MerkleKVClient.withConnection("localhost", 7379) { client =>
        for {
          _ <- client.set("convenience:test", "test:value")
          value <- client.get("convenience:test")
        } yield value
      }.futureValue
      
      result shouldBe Some("test:value")
    }
    
    "work with convenience withConnection method using config" in {
      assumeServerAvailable()
      
      val config = MerkleKVConfig(host = "localhost", port = 7379, timeout = 10.seconds)
      val result = MerkleKVClient.withConnection(config) { client =>
        for {
          _ <- client.set("convenience:test2", "test:value2")
          value <- client.get("convenience:test2")
        } yield value
      }.futureValue
      
      result shouldBe Some("test:value2")
    }
  }
  
  "MerkleKVException" should {
    
    "have proper hierarchy" in {
      val exceptions = List(
        MerkleKVException.ConnectionException("test"),
        MerkleKVException.TimeoutException(),
        MerkleKVException.ProtocolException("test"),
        MerkleKVException.ValidationException("test"),
        MerkleKVException.InvalidResponseException("test"),
        MerkleKVException.NetworkException("test", new RuntimeException())
      )
      
      exceptions.foreach { exception =>
        exception shouldBe a[MerkleKVException]
        exception.getMessage should not be null
      }
    }
    
    "have correct messages" in {
      val connectionEx = MerkleKVException.ConnectionException("Connection failed")
      connectionEx.getMessage shouldBe "Connection failed"
      
      val timeoutEx = MerkleKVException.TimeoutException("Custom timeout")
      timeoutEx.getMessage shouldBe "Custom timeout"
      
      val protocolEx = MerkleKVException.ProtocolException("Invalid protocol")
      protocolEx.getMessage shouldBe "Invalid protocol"
      
      val validationEx = MerkleKVException.ValidationException("Invalid input")
      validationEx.getMessage shouldBe "Invalid input"
      
      val invalidResponseEx = MerkleKVException.InvalidResponseException("BAD_RESPONSE")
      invalidResponseEx.getMessage shouldBe "Invalid response: BAD_RESPONSE"
      
      val networkEx = MerkleKVException.NetworkException("Network error", new RuntimeException("cause"))
      networkEx.getMessage shouldBe "Network error"
      networkEx.getCause should not be null
    }
  }
}
