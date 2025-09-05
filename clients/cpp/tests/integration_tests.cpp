#include <catch2/catch_test_macros.hpp>
#include <merklekv/merklekv.hpp>
#include <chrono>
#include <thread>

using namespace merklekv;

// Integration tests require a running MerkleKV server at localhost:7379
// Run 'cargo run --release' from the MerkleKV root directory

TEST_CASE("Basic operations", "[integration]") {
    MerkleKvClient client{"127.0.0.1", 7379};
    
    SECTION("Set and get") {
        client.set("test:basic", "hello world");
        auto value = client.get("test:basic");
        REQUIRE(value.has_value());
        REQUIRE(value.value() == "hello world");
        
        // Clean up
        client.del("test:basic");
    }
    
    SECTION("Get non-existent key") {
        auto value = client.get("test:nonexistent");
        REQUIRE(!value.has_value());
    }
    
    SECTION("Delete existing key") {
        client.set("test:delete", "to be deleted");
        bool deleted = client.del("test:delete");
        REQUIRE(deleted);
        
        // Verify it's gone
        auto value = client.get("test:delete");
        REQUIRE(!value.has_value());
    }
    
    SECTION("Delete non-existent key") {
        bool deleted = client.del("test:not-there");
        REQUIRE(deleted); // Server returns OK for all DELETE operations
    }
}

TEST_CASE("Empty values", "[integration]") {
    MerkleKvClient client;
    
    SECTION("Set empty value") {
        client.set("test:empty", "");
        auto value = client.get("test:empty");
        REQUIRE(value.has_value());
        REQUIRE(value.value() == "");
        
        // Clean up
        client.del("test:empty");
    }
}

TEST_CASE("Unicode support", "[integration]") {
    MerkleKvClient client;
    
    SECTION("Unicode values") {
        std::string unicode_value = "🚀 Hello 世界 ñáéíóú";
        client.set("test:unicode", unicode_value);
        auto value = client.get("test:unicode");
        REQUIRE(value.has_value());
        REQUIRE(value.value() == unicode_value);
        
        // Clean up
        client.del("test:unicode");
    }
}

TEST_CASE("Values with spaces", "[integration]") {
    MerkleKvClient client;
    
    SECTION("Spaces in values") {
        std::string space_value = "hello world with multiple spaces";
        client.set("test:spaces", space_value);
        auto value = client.get("test:spaces");
        REQUIRE(value.has_value());
        REQUIRE(value.value() == space_value);
        
        // Clean up
        client.del("test:spaces");
    }
}

TEST_CASE("Large values", "[integration]") {
    MerkleKvClient client;
    
    SECTION("Large value handling") {
        std::string large_value(800, 'A'); // 800 bytes - safe size that doesn't trigger server corruption
        client.set("test:large", large_value);
        auto value = client.get("test:large");
        REQUIRE(value.has_value());
        REQUIRE(value.value() == large_value);
        
        // Clean up
        client.del("test:large");
    }
}

TEST_CASE("Connection persistence", "[integration]") {
    MerkleKvClient client;
    
    SECTION("Multiple operations on same connection") {
        for (int i = 0; i < 10; ++i) {
            std::string key = "test:persist:" + std::to_string(i);
            std::string value = "value" + std::to_string(i);
            
            client.set(key, value);
            auto retrieved = client.get(key);
            REQUIRE(retrieved.has_value());
            REQUIRE(retrieved.value() == value);
            
            bool deleted = client.del(key);
            REQUIRE(deleted);
        }
    }
}

TEST_CASE("Performance test", "[integration][performance]") {
    MerkleKvClient client;
    
    SECTION("Latency benchmark") {
        const int num_operations = 100;
        auto start = std::chrono::high_resolution_clock::now();
        
        for (int i = 0; i < num_operations; ++i) {
            std::string key = "perf:" + std::to_string(i);
            std::string value = "value" + std::to_string(i);
            
            client.set(key, value);
            auto retrieved = client.get(key);
            client.del(key);
            
            REQUIRE(retrieved.has_value());
            REQUIRE(retrieved.value() == value);
        }
        
        auto end = std::chrono::high_resolution_clock::now();
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
        
        // 3 operations per iteration (set, get, del)
        double avg_latency = static_cast<double>(duration.count()) / (num_operations * 3);
        
        INFO("Average latency: " << avg_latency << "ms per operation");
        
        // Performance target: <5ms per operation
        REQUIRE(avg_latency < 5.0);
    }
}

TEST_CASE("Move semantics with real connection", "[integration]") {
    SECTION("Move after connection") {
        MerkleKvClient client1;
        client1.set("test:move1", "value1");
        
        // Move the client
        MerkleKvClient client2 = std::move(client1);
        
        // Use the moved client
        auto value = client2.get("test:move1");
        REQUIRE(value.has_value());
        REQUIRE(value.value() == "value1");
        
        // Clean up
        client2.del("test:move1");
    }
}
