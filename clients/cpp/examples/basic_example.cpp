#include <iostream>
#include <iomanip>
#include <chrono>
#include <string>
#include <merklekv/merklekv.hpp>

using namespace merklekv;

int main() {
    std::cout << "MerkleKV C++ Client Example\n";
    std::cout << "===========================\n\n";

    try {
        // Create client with custom timeout
        MerkleKvClient client{"127.0.0.1", 7379, std::chrono::milliseconds(5000)};
        
        std::cout << "1. Basic Operations:\n";
        
        // Set a value
        client.set("user:1", "alice");
        std::cout << "✓ Set user:1 = alice\n";

        // Get the value
        auto value = client.get("user:1");
        std::cout << "✓ Get user:1 = " << (value ? *value : "(null)") << "\n";

        // Delete the key
        bool deleted = client.del("user:1");
        std::cout << "✓ Delete user:1 = " << (deleted ? "true" : "false") << "\n";

        // Try to get after delete
        auto afterDelete = client.get("user:1");
        std::cout << "✓ Get user:1 after delete = " << (afterDelete ? *afterDelete : "(null)") << "\n";

        std::cout << "\n2. Special Values:\n";
        
        // Empty value (automatically handled as "")
        client.set("empty:test", "");
        auto emptyValue = client.get("empty:test");
        std::cout << "✓ Empty value: '" << (emptyValue ? *emptyValue : "(null)") << "'\n";

        // Unicode value
        std::string unicodeValue = "🚀 Hello 世界! ñáéíóú";
        client.set("unicode:test", unicodeValue);
        auto retrievedUnicode = client.get("unicode:test");
        std::cout << "✓ Unicode value: " << (retrievedUnicode ? *retrievedUnicode : "(null)") << "\n";

        // Value with spaces
        std::string spacesValue = "value with multiple spaces";
        client.set("spaces:test", spacesValue);
        auto retrievedSpaces = client.get("spaces:test");
        std::cout << "✓ Spaces value: '" << (retrievedSpaces ? *retrievedSpaces : "(null)") << "'\n";

        // Clean up
        client.del("empty:test");
        client.del("unicode:test");
        client.del("spaces:test");

        std::cout << "\n3. Performance Test (1000 operations):\n";
        
        auto start = std::chrono::high_resolution_clock::now();
        
        for (int i = 0; i < 1000; ++i) {
            std::string key = "perf:" + std::to_string(i);
            std::string val = "value" + std::to_string(i);
            
            client.set(key, val);
            client.get(key);
            client.del(key);
        }
        
        auto end = std::chrono::high_resolution_clock::now();
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
        
        double avgLatency = static_cast<double>(duration.count()) / 3000.0; // 3000 total operations
        std::cout << "✓ Average latency: " << std::fixed << std::setprecision(2) << avgLatency << "ms per operation\n";
        
        if (avgLatency < 5.0) {
            std::cout << "✓ Performance target met (<5ms)\n";
        } else {
            std::cout << "⚠️ Performance target not met (>5ms)\n";
        }

        std::cout << "\n4. Move Semantics:\n";
        
        // Demonstrate move semantics
        MerkleKvClient client1;
        client1.set("move:test", "movable value");
        
        MerkleKvClient client2 = std::move(client1);
        auto movedValue = client2.get("move:test");
        std::cout << "✓ Moved client value: " << (movedValue ? *movedValue : "(null)") << "\n";
        
        client2.del("move:test");

    } catch (const ConnectionException& e) {
        std::cerr << "❌ Connection error: " << e.what() << "\n";
        std::cerr << "   Make sure MerkleKV server is running: cargo run --release\n";
        return 1;
    } catch (const TimeoutException& e) {
        std::cerr << "❌ Timeout error: " << e.what() << "\n";
        return 1;
    } catch (const ProtocolException& e) {
        std::cerr << "❌ Protocol error: " << e.what() << "\n";
        return 1;
    } catch (const Exception& e) {
        std::cerr << "❌ MerkleKV error: " << e.what() << "\n";
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "❌ Standard error: " << e.what() << "\n";
        return 1;
    }

    std::cout << "\n✅ Example completed successfully!\n";
    return 0;
}
