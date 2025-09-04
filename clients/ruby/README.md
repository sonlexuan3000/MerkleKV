# MerkleKV Ruby Client

Official Ruby client library for [MerkleKV](https://github.com/AI-Decenter/MerkleKV) distributed key-value store.

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'merklekv'
```

And then execute:

```bash
bundle install
```

Or install it yourself:

```bash
gem install merklekv
```

## Quick Start

```ruby
require 'merklekv'

# Create client with default settings (127.0.0.1:7379, 5s timeout)
client = MerkleKV::Client.new(host: "127.0.0.1", port: 7379, timeout: 5.0)

# Set a key-value pair
client.set("user:1", "alice")

# Get a value (returns string or nil if key not found)
value = client.get("user:1")      # => "alice"

# Delete a key (returns true if deleted, false if not found)
deleted = client.delete("user:1") # => true

# Close the connection
client.close
```

## Features

- **Simple API**: Intuitive Ruby methods for all operations
- **Automatic Resource Management**: Explicit close method with automatic cleanup
- **Configurable Timeouts**: Default 5-second timeout, fully configurable
- **Unicode Support**: Full UTF-8 support for keys and values
- **Empty Value Handling**: Empty strings automatically converted to `""` protocol format
- **Comprehensive Error Handling**: Ruby exception hierarchy for different error conditions
- **Connection Persistence**: Reuses TCP connections for multiple operations

## Protocol Compliance

This client implements the MerkleKV TCP text protocol:

- **CRLF Termination**: All commands terminated with `\r\n`
- **UTF-8 Encoding**: Full Unicode support for keys and values
- **Empty Value Rule**: Empty strings are automatically represented as `""` at the protocol layer

## Error Handling

The client provides a comprehensive exception hierarchy:

```ruby
begin
  client.set("key", "value")
rescue MerkleKV::ConnectionError => e
  # Network/connection issues
  puts "Connection failed: #{e.message}"
rescue MerkleKV::TimeoutError => e
  # Operation timeout
  puts "Operation timed out: #{e.message}"
rescue MerkleKV::ProtocolError => e
  # Server returned ERROR response
  puts "Protocol error: #{e.message}"
rescue MerkleKV::Error => e
  # Base exception for all MerkleKV errors
  puts "MerkleKV error: #{e.message}"
end
```

## Configuration

### Custom Connection Parameters

```ruby
# Custom host, port, and timeout
client = MerkleKV::Client.new(
  host: "192.168.1.100",
  port: 9999,
  timeout: 10.0
)
```

### Empty Values

Empty strings are automatically handled:

```ruby
client.set("empty", "")        # Automatically converted to "" protocol format
value = client.get("empty")    # Returns ""
```

## Advanced Usage

### Connection Management

```ruby
client = MerkleKV::Client.new

# Check connection status
puts client.connected? # => false

# Connection is established automatically on first operation
client.set("key", "value")
puts client.connected? # => true

# Explicitly close connection
client.close
puts client.connected? # => false

# Safe to call close multiple times
client.close # No error
```

### Resource Management with Blocks

While Ruby doesn't have automatic resource management like some languages, you can use a simple wrapper:

```ruby
def with_client(**options)
  client = MerkleKV::Client.new(**options)
  yield client
ensure
  client&.close
end

with_client(host: "127.0.0.1") do |client|
  client.set("key", "value")
  value = client.get("key")
  puts value
end # Client automatically closed
```

## Performance

The client is optimized for performance:

- **Target Latency**: <5ms for local connections (127.0.0.1)
- **Connection Reuse**: Persistent TCP connections
- **Efficient I/O**: Direct socket operations with minimal overhead

## Thread Safety

Each `MerkleKV::Client` instance maintains its own TCP connection and is **not** thread-safe for concurrent operations. For multi-threaded scenarios, use separate client instances per thread or provide your own synchronization:

```ruby
require 'thread'

# Option 1: Separate clients per thread
clients = ThreadSafe::Array.new
threads = (1..5).map do |i|
  Thread.new do
    client = MerkleKV::Client.new
    client.set("thread:#{i}", "value#{i}")
    clients << client
  end
end
threads.each(&:join)
clients.each(&:close)

# Option 2: Synchronized access
mutex = Mutex.new
client = MerkleKV::Client.new

threads = (1..5).map do |i|
  Thread.new do
    mutex.synchronize do
      client.set("sync:#{i}", "value#{i}")
    end
  end
end
threads.each(&:join)
client.close
```

## Examples

See the `examples/` directory for complete usage examples:

- **basic_example.rb**: Comprehensive usage demonstration
- Performance testing
- Error handling patterns
- Unicode support
- Connection management

## Running Examples and Tests

```bash
# Clone repository
git clone https://github.com/AI-Decenter/MerkleKV.git
cd MerkleKV/clients/ruby

# Install dependencies
bundle install

# Run basic example (requires running MerkleKV server)
ruby examples/basic_example.rb

# Run unit tests
bundle exec rspec

# Run integration tests (requires running MerkleKV server)
RUN_INTEGRATION_TESTS=1 bundle exec rspec

# Run performance tests
RUN_PERFORMANCE_TESTS=1 RUN_INTEGRATION_TESTS=1 bundle exec rspec
```

## API Reference

### MerkleKV::Client Class

#### Constructor

```ruby
initialize(host: "127.0.0.1", port: 7379, timeout: 5.0)
```

**Parameters:**
- `host` (String): Server hostname or IP address
- `port` (Integer): Server port
- `timeout` (Float): Operation timeout in seconds

#### Methods

```ruby
# Set key-value pair
set(key, value)

# Get value by key (returns String or nil)
get(key)

# Delete key (returns true/false)
delete(key)

# Close connection
close

# Check connection status
connected?
```

## Requirements

- **Ruby 2.7.0** or later
- **MerkleKV Server**: Running instance for connections

## Troubleshooting

### Cannot Connect

```
MerkleKV::ConnectionError: Failed to connect to 127.0.0.1:7379
```

**Solutions:**
- Ensure MerkleKV server is running: `cargo run --release`
- Verify host and port settings
- Check firewall rules

### Operation Timeout

```
MerkleKV::TimeoutError: Operation timeout
```

**Solutions:**
- Increase timeout: `MerkleKV::Client.new(timeout: 10.0)`
- Check network latency
- Verify server is responsive

### Protocol Errors

```
MerkleKV::ProtocolError: Invalid command
```

**Solutions:**
- Ensure server is MerkleKV (not Redis/Memcached)
- Check server logs for errors
- Verify protocol version compatibility

## Development

After checking out the repo, run `bin/setup` to install dependencies. Then, run `rake spec` to run the tests. You can also run `bin/console` for an interactive prompt.

To install this gem onto your local machine, run `bundle exec rake install`. To release a new version, update the version number in `merklekv.gemspec`, and then run `bundle exec rake release`.

## License

MIT License - see the [LICENSE](https://github.com/AI-Decenter/MerkleKV/blob/main/LICENSE) file for details.

## Links

- **Main Repository**: https://github.com/AI-Decenter/MerkleKV
- **Protocol Documentation**: https://github.com/AI-Decenter/MerkleKV#usage-raw-tcp-protocol
- **Other Client Libraries**: https://github.com/AI-Decenter/MerkleKV/tree/main/clients
