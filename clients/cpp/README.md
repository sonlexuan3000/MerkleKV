# MerkleKV C++ Client

Official C++ client library for [MerkleKV](https://github.com/AI-Decenter/MerkleKV) distributed key-value store.

## Installation

### Using CMake FetchContent (Recommended)

```cmake
cmake_minimum_required(VERSION 3.16)
project(my_project)

include(FetchContent)
FetchContent_Declare(
    merklekv-cpp
    GIT_REPOSITORY https://github.com/AI-Decenter/MerkleKV.git
    GIT_TAG main
    SOURCE_SUBDIR clients/cpp
)
FetchContent_MakeAvailable(merklekv-cpp)

add_executable(my_app main.cpp)
target_link_libraries(my_app PRIVATE merklekv::client)
```

### Using find_package (After Installation)

```cmake
find_package(merklekv-cpp REQUIRED)
target_link_libraries(my_app PRIVATE merklekv::client)
```

### Manual Installation

```bash
git clone https://github.com/AI-Decenter/MerkleKV.git
cd MerkleKV/clients/cpp
mkdir build && cd build
cmake ..
make install
```

## Quick Start

```cpp
#include <merklekv/merklekv.hpp>
#include <iostream>

int main() {
    try {
        // Create client with default settings (127.0.0.1:7379, 5s timeout)
        merklekv::MerkleKvClient client{"127.0.0.1", 7379};
        
        // Set a key-value pair
        client.set("user:1", "alice");
        
        // Get a value (returns std::optional<std::string>)
        auto value = client.get("user:1");
        if (value) {
            std::cout << "Value: " << *value << std::endl;
        }
        
        // Delete a key (returns true if deleted, false if not found)
        bool deleted = client.del("user:1");
        std::cout << "Deleted: " << std::boolalpha << deleted << std::endl;
        
    } catch (const merklekv::Exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
}
```

## Features

- **Header-Only Library**: Single header include, no separate compilation needed
- **Modern C++17**: Uses std::optional, move semantics, and RAII
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **Exception Safety**: RAII for automatic resource management
- **Unicode Support**: Full UTF-8 support for keys and values
- **Empty Value Handling**: Empty strings automatically converted to `""` protocol format
- **Configurable Timeouts**: Default 5-second timeout, fully configurable
- **Move Semantics**: Efficient resource transfer

## Protocol Compliance

This client implements the MerkleKV TCP text protocol:

- **CRLF Termination**: All commands terminated with `\r\n`
- **UTF-8 Encoding**: Full Unicode support for keys and values
- **Empty Value Rule**: Empty strings are automatically represented as `""` at the protocol layer

## Error Handling

The client provides a comprehensive exception hierarchy:

```cpp
#include <merklekv/merklekv.hpp>

try {
    client.set("key", "value");
} catch (const merklekv::ConnectionException& e) {
    // Network/connection issues
    std::cerr << "Connection failed: " << e.what() << std::endl;
} catch (const merklekv::TimeoutException& e) {
    // Operation timeout
    std::cerr << "Operation timed out: " << e.what() << std::endl;
} catch (const merklekv::ProtocolException& e) {
    // Server returned ERROR response
    std::cerr << "Protocol error: " << e.what() << std::endl;
} catch (const merklekv::Exception& e) {
    // Base exception for all MerkleKV errors
    std::cerr << "MerkleKV error: " << e.what() << std::endl;
}
```

## Configuration

### Custom Timeout

```cpp
using namespace std::chrono_literals;

// Custom timeout
merklekv::MerkleKvClient client{"localhost", 7379, 10s};
```

### Empty Values

Empty strings are automatically handled:

```cpp
client.set("empty", "");        // Automatically converted to "" protocol format
auto value = client.get("empty"); // Returns std::optional containing ""
```

## Advanced Usage

### Move Semantics

```cpp
merklekv::MerkleKvClient create_client() {
    return merklekv::MerkleKvClient{"192.168.1.100", 7379};
}

auto client = create_client(); // Efficient move construction
client.set("key", "value");
```

### RAII and Exception Safety

```cpp
void safe_operations() {
    merklekv::MerkleKvClient client; // Automatic cleanup on scope exit
    
    client.set("temp:key", "temporary value");
    
    // Even if exception occurs, client is automatically cleaned up
    if (some_condition()) {
        throw std::runtime_error("Something went wrong");
    }
    
    client.del("temp:key");
} // Client automatically disconnects here
```

## Performance

The client is optimized for performance:

- **Target Latency**: <5ms for local connections (127.0.0.1)
- **Connection Reuse**: Persistent TCP connections
- **Zero-Copy Operations**: Efficient string handling
- **Header-Only**: No dynamic linking overhead

## Thread Safety

Each `MerkleKvClient` instance maintains its own TCP connection and is **not** thread-safe for concurrent operations. For multi-threaded scenarios, use separate client instances per thread or provide your own synchronization:

```cpp
#include <mutex>

class ThreadSafeClient {
private:
    merklekv::MerkleKvClient client_;
    std::mutex mutex_;
    
public:
    void set(const std::string& key, const std::string& value) {
        std::lock_guard<std::mutex> lock(mutex_);
        client_.set(key, value);
    }
    
    std::optional<std::string> get(const std::string& key) {
        std::lock_guard<std::mutex> lock(mutex_);
        return client_.get(key);
    }
};
```

## Examples

See the `examples/` directory for complete usage examples:

- **basic_example.cpp**: Comprehensive usage demonstration
- Performance benchmarking
- Error handling patterns
- Unicode support
- Move semantics demonstration

## Building Examples and Tests

```bash
git clone https://github.com/AI-Decenter/MerkleKV.git
cd MerkleKV/clients/cpp
mkdir build && cd build
cmake -DMERKLEKV_BUILD_EXAMPLES=ON -DMERKLEKV_BUILD_TESTS=ON ..
make

# Run examples
./examples/basic_example

# Run tests (requires running MerkleKV server)
ctest
```

## API Reference

### MerkleKvClient Class

#### Constructor

```cpp
explicit MerkleKvClient(
    const std::string& host = "127.0.0.1",
    uint16_t port = 7379,
    std::chrono::milliseconds timeout = std::chrono::milliseconds(5000)
);
```

#### Methods

```cpp
// Set key-value pair
void set(const std::string& key, const std::string& value);

// Get value by key (returns std::optional)
std::optional<std::string> get(const std::string& key);

// Delete key (returns true if deleted, false if not found)  
bool del(const std::string& key);
```

#### Resource Management

```cpp
// Destructor automatically closes connection
~MerkleKvClient();

// Move constructor and assignment (non-copyable)
MerkleKvClient(MerkleKvClient&& other) noexcept;
MerkleKvClient& operator=(MerkleKvClient&& other) noexcept;
```

## Requirements

- **C++17** or later
- **CMake 3.16** or later for building
- **Network Libraries**: 
  - Linux/macOS: Built-in socket APIs
  - Windows: ws2_32 (automatically linked)

## Troubleshooting

### Cannot Connect

```
merklekv::ConnectionException: Failed to connect to 127.0.0.1:7379
```

**Solutions:**
- Ensure MerkleKV server is running: `cargo run --release`
- Verify host and port settings
- Check firewall rules

### Compilation Errors

**Missing C++17 support:**
```cmake
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
```

**Windows linking issues:**
- The client automatically links `ws2_32` on Windows
- Ensure you're using a recent version of MSVC or MinGW

### Runtime Issues

**Timeout errors:**
```cpp
// Increase timeout
merklekv::MerkleKvClient client{"host", 7379, std::chrono::seconds(10)};
```

## License

MIT License - see the [LICENSE](https://github.com/AI-Decenter/MerkleKV/blob/main/LICENSE) file for details.

## Links

- **Main Repository**: https://github.com/AI-Decenter/MerkleKV
- **Protocol Documentation**: https://github.com/AI-Decenter/MerkleKV#usage-raw-tcp-protocol
- **Other Client Libraries**: https://github.com/AI-Decenter/MerkleKV/tree/main/clients
