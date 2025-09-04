#include <catch2/catch_test_macros.hpp>
#include <merklekv/merklekv.hpp>

using namespace merklekv;

TEST_CASE("MerkleKvClient construction", "[constructor]") {
    SECTION("Default constructor") {
        MerkleKvClient client;
        // Should not throw
        REQUIRE(true);
    }

    SECTION("Custom host and port") {
        MerkleKvClient client{"192.168.1.1", 9999};
        REQUIRE(true);
    }

    SECTION("Custom timeout") {
        MerkleKvClient client{"127.0.0.1", 7379, std::chrono::milliseconds(10000)};
        REQUIRE(true);
    }

    SECTION("Empty host throws") {
        REQUIRE_THROWS_AS(MerkleKvClient("", 7379), std::invalid_argument);
    }
}

TEST_CASE("Key validation", "[validation]") {
    MerkleKvClient client;
    
    SECTION("Empty key throws for set") {
        REQUIRE_THROWS_AS(client.set("", "value"), std::invalid_argument);
    }
    
    SECTION("Empty key throws for get") {
        REQUIRE_THROWS_AS(client.get(""), std::invalid_argument);
    }
    
    SECTION("Empty key throws for del") {
        REQUIRE_THROWS_AS(client.del(""), std::invalid_argument);
    }
}

TEST_CASE("Move semantics", "[move]") {
    SECTION("Move constructor") {
        MerkleKvClient client1;
        MerkleKvClient client2 = std::move(client1);
        REQUIRE(true); // Should not crash
    }
    
    SECTION("Move assignment") {
        MerkleKvClient client1;
        MerkleKvClient client2;
        client2 = std::move(client1);
        REQUIRE(true); // Should not crash
    }
}

TEST_CASE("Exception hierarchy", "[exceptions]") {
    SECTION("Base exception") {
        Exception ex("test");
        REQUIRE(ex.what() == std::string("test"));
    }
    
    SECTION("Connection exception inherits from base") {
        ConnectionException ex("connection error");
        REQUIRE_NOTHROW(dynamic_cast<Exception&>(ex));
    }
    
    SECTION("Timeout exception inherits from base") {
        TimeoutException ex("timeout error");
        REQUIRE_NOTHROW(dynamic_cast<Exception&>(ex));
    }
    
    SECTION("Protocol exception inherits from base") {
        ProtocolException ex("protocol error");
        REQUIRE_NOTHROW(dynamic_cast<Exception&>(ex));
    }
}

TEST_CASE("Connection errors", "[connection]") {
    SECTION("Invalid host throws ConnectionException") {
        MerkleKvClient client{"invalid-host-that-does-not-exist", 7379};
        REQUIRE_THROWS_AS(client.set("key", "value"), ConnectionException);
    }
    
    SECTION("Connection timeout") {
        MerkleKvClient client{"192.0.2.1", 7379, std::chrono::milliseconds(100)};
        REQUIRE_THROWS_AS(client.set("key", "value"), ConnectionException);
    }
}
