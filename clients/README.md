# MerkleKV Client Libraries

Official client libraries for [MerkleKV](https://github.com/AI-Decenter/MerkleKV), a high-performance distributed key-value store with self-healing replication.

## Available Clients

✅ **Phase 4 Complete**: All 13 official client libraries have been implemented and are production-ready with universal port 7379 standardization:

### High-Priority & Medium-Priority Languages (Phase 1-3 Complete)

| Language | Package | Status | Default Port | Documentation |
|----------|---------|--------|:------------:|---------------|
| **Python** | `merklekv` | ✅ Ready | 7379 | [README](python/README.md) |
| **Node.js** | `@merklekv/client` | ✅ Ready | 7379 | [README](nodejs/README.md) |
| **Go** | `github.com/AI-Decenter/MerkleKV/clients/go` | ✅ Ready | 7379 | [README](go/README.md) |
| **Java** | `io.merklekv:client` | ✅ Ready | 7379 | [README](java/README.md) |
| **Rust** | `merklekv-client` | ✅ Ready | 7379 | [README](rust/README.md) |
| **C#/.NET** | `MerkleKV.Client` | ✅ Ready | 7379 | [README](dotnet/README.md) |
| **C++** | Header-only library | ✅ Ready | 7379 | [README](cpp/README.md) |
| **Ruby** | `merklekv` | ✅ Ready | 7379 | [README](ruby/README.md) |
| **PHP** | `merklekv/client` | ✅ Ready | 7379 | [README](php/README.md) |

### Low-Priority Languages (Phase 4 Complete)

| Language | Package | Status | Default Port | Documentation |
|----------|---------|--------|:------------:|---------------|
| **Swift** | `MerkleKVClient` (SPM) | ✅ Ready | 7379 | [README](swift/README.md) |
| **Kotlin** | `io.merklekv:merklekv-client` | ✅ Ready | 7379 | [README](kotlin/README.md) |
| **Scala** | `io.merklekv:merklekv-client-scala` | ✅ Ready | 7379 | [README](scala/README.md) |
| **Elixir** | `merklekv_client` | ✅ Ready | 7379 | [README](elixir/README.md) |

## Protocol Overview

All clients implement the MerkleKV TCP text protocol with consistent behavior:

- **Connection**: TCP socket to MerkleKV server (default port **7379** - universally standardized)
- **Commands**: Text-based with CRLF (`\r\n`) termination
- **Core Operations**: `GET key`, `SET key value`, `DEL key`
- **Responses**: `VALUE <data>`, `OK`, `(null)`, `NOT_FOUND`, `ERROR message`
- **Encoding**: UTF-8 with empty values represented as `""`
- **Environment Variable**: All clients support `MERKLEKV_PORT` override for testing flexibility

## Port 7379 Standardization

✅ **Universal Port Standard**: All 13 clients now default to port 7379
- **Server Configuration**: `config.toml` updated to port 7379
- **CI/CD Pipeline**: Docker and testing infrastructure unified on port 7379  
- **Environment Variables**: All test suites support `MERKLEKV_PORT=7379` override
- **Backward Compatibility**: No breaking changes - existing applications continue working

## Shared Constraints & Protocol Behavior

All clients handle these protocol characteristics consistently:

- **GET Response**: Returns `VALUE <data>` prefix that clients strip automatically
- **Empty Values**: Server returns `VALUE ""` which clients convert to empty string
- **DELETE Response**: Server returns `DELETED` for existing keys and `NOT_FOUND` for non-existing keys
- **Control Characters**: Server accepts tab (`\t`) characters in values but not newlines (`\n`) due to protocol design
- **Large Values**: Server supports values of arbitrary size using streaming protocol

## Installation & Usage

### Python

```bash
pip install merklekv
```

```python
from merklekv import MerkleKVClient

client = MerkleKVClient("localhost", 7379)  # Default port 7379
client.set("user:123", "john_doe")
value = client.get("user:123")  # Returns "john_doe"
client.delete("user:123")
```

### Node.js

```bash
npm install @merklekv/client
```

```javascript
const { MerkleKVClient } = require('@merklekv/client');

const client = new MerkleKVClient('localhost', 7379);  // Default port 7379
await client.set('user:123', 'john_doe');
const value = await client.get('user:123'); // Returns 'john_doe'
await client.delete('user:123');
```

### Go

```bash
go get github.com/AI-Decenter/MerkleKV/clients/go
```

```go
import merklekv "github.com/AI-Decenter/MerkleKV/clients/go"

client := merklekv.New("localhost", 7379)  // Default port 7379
client.Set("user:123", "john_doe")
value, _ := client.Get("user:123") // Returns "john_doe"
client.Delete("user:123")
```

### Java

```xml
<dependency>
    <groupId>io.merklekv</groupId>
    <artifactId>merklekv-client-java</artifactId>
    <version>1.0.0</version>
</dependency>
```

```java
import io.merklekv.client.MerkleKVClient;

try (MerkleKVClient client = new MerkleKVClient("localhost", 7379)) {  // Default port 7379
    client.set("user:123", "john_doe");
    String value = client.get("user:123"); // Returns "john_doe"
    client.delete("user:123");
}
```

### Rust

```toml
[dependencies]
merklekv-client = "1.0.0"
```

```rust
use merklekv_client::Client;

let mut client = Client::connect("127.0.0.1:7379")?;  // Default port 7379
client.set("user:123", "john_doe")?;
let value = client.get("user:123")?; // Returns Some("john_doe")
client.delete("user:123")?;
```

### C#/.NET

```bash
dotnet add package MerkleKV.Client
```

```csharp
using MerkleKV;

using var client = new MerkleKvClient("localhost", 7379);
await client.SetAsync("user:123", "john_doe");
var value = await client.GetAsync("user:123"); // Returns "john_doe"
await client.DeleteAsync("user:123");
```

### C++

```cmake
find_package(MerkleKV REQUIRED)
target_link_libraries(your_target MerkleKV::Client)
```

```cpp
#include <merklekv/client.hpp>

MerkleKvClient client("localhost", 7379);
client.set("user:123", "john_doe");
auto value = client.get("user:123"); // Returns optional<string>
client.del("user:123");
```

### Ruby

```bash
gem install merklekv
```

```ruby
require 'merklekv'

client = MerkleKV::Client.new(host: "localhost", port: 7379)
client.set("user:123", "john_doe")
value = client.get("user:123") # Returns "john_doe"  
client.delete("user:123")
```

### PHP

```bash
composer require merklekv/client
```

```php
<?php
use MerkleKV\Client;

$client = new Client("localhost", 7379);
$client->set("user:123", "john_doe");
$value = $client->get("user:123"); // Returns "john_doe"
$client->delete("user:123");
```

### Swift

```swift
// Package.swift
dependencies: [
    .package(url: "https://github.com/AI-Decenter/MerkleKV-Swift.git", from: "1.0.0")
]
```

```swift
import MerkleKV

let client = await MerkleKVClient(host: "localhost", port: 7379)
await client.set("user:123", "john_doe")
let value = await client.get("user:123") // Returns "john_doe"
await client.delete("user:123")
```

### Kotlin

```gradle
dependencies {
    implementation 'com.merklekv:client:1.0.0'
}
```

```kotlin
import com.merklekv.MerkleKVClient

val client = MerkleKVClient("localhost", 7379)
client.set("user:123", "john_doe")
val value = client.get("user:123") // Returns "john_doe"
client.delete("user:123")
```

### Scala

```scala
libraryDependencies += "com.merklekv" %% "client" % "1.0.0"
```

```scala
import com.merklekv.MerkleKVClient

val client = new MerkleKVClient("localhost", 7379)
client.set("user:123", "john_doe").recover { case _ => () }
val value = client.get("user:123") // Returns Future[Option[String]]
client.delete("user:123")
```

### Elixir

```elixir
# mix.exs
defp deps do
  [{:merklekv, "~> 1.0"}]
end
```

```elixir
{:ok, client} = MerkleKV.Client.start_link([host: "localhost", port: 7379])
:ok = MerkleKV.Client.set(client, "user:123", "john_doe")
{:ok, value} = MerkleKV.Client.get(client, "user:123") # Returns "john_doe"
:ok = MerkleKV.Client.delete(client, "user:123")
```

## Common Features

All client libraries provide:

### Core Operations
- **Connect/Disconnect**: Manage connections to MerkleKV server
- **GET**: Retrieve values by key
- **SET**: Store key-value pairs
- **DELETE**: Remove keys

### Error Handling
- **Connection Errors**: Network and connection failures
- **Timeout Errors**: Operation timeouts
- **Protocol Errors**: Server-side errors
- **Validation Errors**: Invalid inputs (empty keys, etc.)

### Advanced Features
- **Timeouts**: Configurable operation timeouts
- **Context Support**: Cancellation and deadline handling (where applicable)
- **Unicode Support**: Full UTF-8 string handling
- **Large Values**: Support for values up to available memory
- **Thread Safety**: Safe concurrent usage (language-dependent)

## API Design Consistency

All clients follow consistent naming and behavior patterns:

### Synchronous APIs
```
connect() -> void/error
get(key) -> value/null + error
set(key, value) -> success + error
delete(key) -> success + error
close() -> void/error
```

### Asynchronous APIs (Python asyncio, Node.js Promises, Go Context)
```
await/async versions of all operations
Context-aware operations with cancellation
Timeout handling via context or configuration
```

### Error Handling
```
ConnectionError: Network/connection issues
TimeoutError: Operation timeouts
ProtocolError: Server errors
ValidationError: Input validation failures
NotFoundError: Key not found (language-dependent)
```

## Testing

Each client library includes:

- **Unit Tests**: Mock socket/network operations
- **Integration Tests**: Against live MerkleKV server
- **Benchmarks**: Performance verification (<5ms local latency target)
- **Examples**: Complete usage demonstrations

### Running Integration Tests

1. **Start MerkleKV server**:
   ```bash
   cd /path/to/MerkleKV
   cargo run --release
   ```

2. **Run client tests**:
   ```bash
   # Python
   cd clients/python
   pip install -e ".[test]"
   pytest tests/

   # Node.js
   cd clients/nodejs
   npm install
   npm test

   # Go
   cd clients/go
   go test -tags=integration
   ```

## Performance Targets

All clients are designed to meet these performance benchmarks:

- **Latency**: <5ms for local connections (127.0.0.1)
- **Throughput**: >10,000 operations/second for simple operations
- **Memory**: Minimal overhead per connection
- **Concurrency**: Support multiple simultaneous operations

## Development Guidelines

When implementing new client libraries:

### Code Structure
```
clients/<language>/
├── src/                 # Main implementation
├── tests/              # Unit and integration tests
├── examples/           # Usage examples
├── docs/               # Documentation
├── README.md           # Language-specific documentation
└── <package-files>     # Package management files
```

### API Requirements
- Implement all core operations (connect, get, set, delete, close)
- Provide both sync and async APIs where language supports it
- Follow language-specific naming conventions
- Include comprehensive error handling
- Support configurable timeouts
- Handle Unicode strings properly
- Validate inputs (empty keys, etc.)

### Testing Requirements
- Unit tests with >90% code coverage
- Integration tests against live server
- Performance benchmarks
- Error condition testing
- Large value handling tests
- Unicode/encoding tests

### Documentation Requirements
- Complete API documentation
- Installation instructions
- Basic usage examples
- Advanced usage patterns
- Error handling examples
- Performance characteristics

## Contributing

1. Follow the established patterns from Phase 1 clients
2. Maintain API consistency across languages
3. Include comprehensive tests and documentation
4. Verify integration with MerkleKV server
5. Submit PR with examples and benchmarks

See the main [MerkleKV repository](https://github.com/AI-Decenter/MerkleKV) for detailed contribution guidelines.

## License

All client libraries are licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Support

- **Documentation**: Language-specific READMEs in each client directory
- **Issues**: [GitHub Issues](https://github.com/AI-Decenter/MerkleKV/issues)
- **Discussions**: [GitHub Discussions](https://github.com/AI-Decenter/MerkleKV/discussions)
- **Examples**: Complete examples in each `examples/` directory
