# MerkleKV .NET Client

Official .NET client library for [MerkleKV](https://github.com/AI-Decenter/MerkleKV) distributed key-value store.

## Installation

Install via NuGet Package Manager:

```bash
dotnet add package MerkleKV.Client
```

Or via Package Manager Console in Visual Studio:

```powershell
Install-Package MerkleKV.Client
```

## Quick Start

```csharp
using MerkleKV;

// Create client with default settings (127.0.0.1:7379, 5s timeout)
using var client = new MerkleKvClient("127.0.0.1", 7379, TimeSpan.FromSeconds(5));

// Set a key-value pair
client.Set("user:1", "alice");

// Get a value (returns null if key not found)
var value = client.Get("user:1");      // Returns "alice"

// Delete a key (returns true if deleted, false if not found)
var deleted = client.Delete("user:1"); // Returns true
```

## Async Operations

```csharp
using MerkleKV;

await using var client = new MerkleKvClient();

// All operations have async variants
await client.SetAsync("user:1", "alice");
var value = await client.GetAsync("user:1");
var deleted = await client.DeleteAsync("user:1");
```

## Features

- **Synchronous and Asynchronous APIs**: Full support for both sync and async operations
- **Automatic Resource Management**: Implements `IDisposable` and `IAsyncDisposable`
- **Configurable Timeouts**: Default 5-second timeout, fully configurable
- **Unicode Support**: Full UTF-8 support for keys and values
- **Empty Value Handling**: Empty strings automatically converted to `""` protocol format
- **Comprehensive Error Handling**: Typed exceptions for different error conditions
- **Thread Safety**: Multiple client instances can be used concurrently

## Protocol Compliance

This client implements the MerkleKV TCP text protocol:

- **CRLF Termination**: All commands terminated with `\r\n`
- **UTF-8 Encoding**: Full Unicode support for keys and values
- **Empty Value Rule**: Empty strings are automatically represented as `""` at the protocol layer

## Error Handling

The client provides a comprehensive exception hierarchy:

```csharp
try
{
    client.Set("key", "value");
}
catch (MerkleKvConnectionException ex)
{
    // Network/connection issues
    Console.WriteLine($"Connection failed: {ex.Message}");
}
catch (MerkleKvTimeoutException ex)
{
    // Operation timeout
    Console.WriteLine($"Operation timed out: {ex.Message}");
}
catch (MerkleKvProtocolException ex)
{
    // Server returned ERROR response
    Console.WriteLine($"Protocol error: {ex.Message}");
}
catch (MerkleKvException ex)
{
    // Base exception for all MerkleKV errors
    Console.WriteLine($"MerkleKV error: {ex.Message}");
}
```

## Configuration

### Timeout Configuration

```csharp
// Custom timeout
var client = new MerkleKvClient("localhost", 7379, TimeSpan.FromSeconds(10));

// Or use cancellation tokens for per-operation control
using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(2));
await client.SetAsync("key", "value", cts.Token);
```

### Empty Values

Empty strings are automatically handled:

```csharp
client.Set("empty", "");        // Automatically converted to "" protocol format
var value = client.Get("empty"); // Returns ""
```

## Performance

The client is optimized for performance:

- **Target Latency**: <5ms for local connections (127.0.0.1)
- **Connection Reuse**: Persistent TCP connections
- **Efficient Encoding**: Minimal overhead UTF-8 processing

## Thread Safety

Each `MerkleKvClient` instance maintains its own TCP connection and is not thread-safe for concurrent operations. For multi-threaded scenarios:

```csharp
// Option 1: Create separate clients per thread
var client1 = new MerkleKvClient();
var client2 = new MerkleKvClient();

// Option 2: Use synchronization
private readonly object _lock = new object();

lock (_lock)
{
    client.Set("key", "value");
}
```

## Examples

See the `examples/` directory for complete usage examples:

- **BasicExample.cs**: Comprehensive usage demonstration
- **Performance testing**: Latency benchmarking
- **Error handling**: Exception handling patterns
- **Unicode support**: International character handling

## API Reference

### MerkleKvClient Class

#### Constructors

```csharp
public MerkleKvClient(string host = "127.0.0.1", int port = 7379, TimeSpan? timeout = null)
```

#### Methods

```csharp
// Synchronous operations
public void Set(string key, string value)
public string? Get(string key)
public bool Delete(string key)

// Asynchronous operations
public Task SetAsync(string key, string value, CancellationToken cancellationToken = default)
public Task<string?> GetAsync(string key, CancellationToken cancellationToken = default)
public Task<bool> DeleteAsync(string key, CancellationToken cancellationToken = default)

// Resource management
public void Dispose()
public ValueTask DisposeAsync()
```

## Requirements

- **.NET 6.0** or later
- **MerkleKV Server**: Running instance for connections

## Troubleshooting

### Cannot Connect

```
MerkleKvConnectionException: Failed to connect to 127.0.0.1:7379
```

**Solutions:**
- Ensure MerkleKV server is running: `cargo run --release`
- Verify host and port settings
- Check firewall rules

### Operation Timeout

```
MerkleKvTimeoutException: Operation timeout
```

**Solutions:**
- Increase timeout: `new MerkleKvClient(timeout: TimeSpan.FromSeconds(10))`
- Check network latency
- Verify server is responsive

### Unexpected Response

```
MerkleKvProtocolException: Unexpected response: ...
```

**Solutions:**
- Ensure server is MerkleKV (not Redis/Memcached)
- Check server logs for errors
- Verify protocol version compatibility

## License

MIT License - see the [LICENSE](https://github.com/AI-Decenter/MerkleKV/blob/main/LICENSE) file for details.

## Links

- **Main Repository**: https://github.com/AI-Decenter/MerkleKV
- **Protocol Documentation**: https://github.com/AI-Decenter/MerkleKV#usage-raw-tcp-protocol
- **Other Client Libraries**: https://github.com/AI-Decenter/MerkleKV/tree/main/clients
