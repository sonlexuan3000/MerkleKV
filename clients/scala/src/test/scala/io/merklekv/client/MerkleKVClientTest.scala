package io.merklekv.client

import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should.Matchers
import org.scalatest.concurrent.ScalaFutures
import scala.concurrent.duration._
import scala.concurrent.ExecutionContext.Implicits.global

class MerkleKVClientTest extends AnyFlatSpec with Matchers with ScalaFutures {
  
  implicit val defaultPatience: PatienceConfig = PatienceConfig(
    timeout = 10.seconds,
    interval = 100.milliseconds
  )

  "MerkleKVClient" should "connect successfully" in {
    val client = new MerkleKVClient("localhost", 7379)
    
    whenReady(client.connect()) { result =>
      result shouldBe true
    }
    
    client.close()
  }

  it should "handle basic SET operation" in {
    val client = new MerkleKVClient("localhost", 7379)
    
    whenReady(client.connect()) { _ =>
      whenReady(client.set("test_key", "test_value")) { result =>
        result shouldBe "OK"
      }
    }
    
    client.close()
  }

  it should "handle basic GET operation" in {
    val client = new MerkleKVClient("localhost", 7379)
    
    whenReady(client.connect()) { _ =>
      whenReady(client.set("test_key", "test_value")) { _ =>
        whenReady(client.get("test_key")) { result =>
          result shouldBe Some("test_value")
        }
      }
    }
    
    client.close()
  }

  it should "return None for non-existent keys" in {
    val client = new MerkleKVClient("localhost", 7379)
    
    whenReady(client.connect()) { _ =>
      whenReady(client.get("non_existent_key")) { result =>
        result shouldBe None
      }
    }
    
    client.close()
  }

  it should "handle DELETE operation" in {
    val client = new MerkleKVClient("localhost", 7379)
    
    whenReady(client.connect()) { _ =>
      whenReady(client.set("test_key", "test_value")) { _ =>
        whenReady(client.delete("test_key")) { result =>
          result shouldBe true
        }
        
        whenReady(client.get("test_key")) { result =>
          result shouldBe None
        }
      }
    }
    
    client.close()
  }

  it should "handle EXISTS operation" in {
    val client = new MerkleKVClient("localhost", 7379)
    
    whenReady(client.connect()) { _ =>
      whenReady(client.set("test_key", "test_value")) { _ =>
        whenReady(client.exists("test_key")) { result =>
          result shouldBe true
        }
      }
      
      whenReady(client.exists("non_existent_key")) { result =>
        result shouldBe false
      }
    }
    
    client.close()
  }

  it should "handle pipeline operations" in {
    val client = new MerkleKVClient("localhost", 7379)
    
    whenReady(client.connect()) { _ =>
      val commands = List(
        "SET key1 value1",
        "SET key2 value2",
        "GET key1"
      )
      
      whenReady(client.pipeline(commands)) { results =>
        results should have size 3
        results(0) shouldBe "OK"
        results(1) shouldBe "OK"
        results(2) shouldBe "value1"
      }
    }
    
    client.close()
  }

  it should "perform health check" in {
    val client = new MerkleKVClient("localhost", 7379)
    
    whenReady(client.connect()) { _ =>
      whenReady(client.healthCheck()) { result =>
        result shouldBe true
      }
    }
    
    client.close()
  }

  it should "handle connection errors gracefully" in {
    val client = new MerkleKVClient("invalid_host", 7379)
    
    whenReady(client.connect().failed) { exception =>
      exception shouldBe a[MerkleKVException]
    }
  }

  it should "handle concurrent operations" in {
    val client = new MerkleKVClient("localhost", 7379)
    
    whenReady(client.connect()) { _ =>
      val futures = (1 to 10).map { i =>
        client.set(s"concurrent_key_$i", s"value_$i")
      }
      
      whenReady(Future.sequence(futures)) { results =>
        results should have size 10
        results.foreach(_ shouldBe "OK")
      }
    }
    
    client.close()
  }

  it should "handle large values" in {
    val client = new MerkleKVClient("localhost", 7379)
    val largeValue = "x" * 10000
    
    whenReady(client.connect()) { _ =>
      whenReady(client.set("large_key", largeValue)) { result =>
        result shouldBe "OK"
      }
      
      whenReady(client.get("large_key")) { result =>
        result shouldBe Some(largeValue)
      }
    }
    
    client.close()
  }
}
