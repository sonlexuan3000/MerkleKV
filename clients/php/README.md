# MerkleKV PHP Client

Official PHP client for MerkleKV distributed key-value store.

[![PHP Version](https://img.shields.io/packagist/php-v/merklekv/client)](https://packagist.org/packages/merklekv/client)
[![Latest Version](https://img.shields.io/packagist/v/merklekv/client)](https://packagist.org/packages/merklekv/client)
[![License](https://img.shields.io/packagist/l/merklekv/client)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-PHPUnit-blue)](tests/)

## Features

- **Simple API**: Easy-to-use SET, GET, DELETE operations
- **Type Safety**: Full PHP 7.4+ type declarations with strict mode
- **Error Handling**: Comprehensive exception hierarchy for different error types
- **UTF-8 Support**: Full Unicode support for keys and values
- **Connection Management**: Automatic connection handling with timeouts
- **Performance**: Optimized TCP client with <5ms operation latency
- **PSR-4 Compliant**: Modern PHP standards and autoloading

## Installation

Install via Composer:

```bash
composer require merklekv/client
```

## Quick Start

```php
<?php

use MerkleKV\Client;

// Create client
$client = new Client("127.0.0.1", 7379);

// Set a key-value pair
$client->set("user:123", "alice");

// Get a value
$value = $client->get("user:123"); // Returns "alice"

// Delete a key  
$deleted = $client->delete("user:123"); // Returns true

// Get non-existent key
$value = $client->get("user:123"); // Returns null

// Close connection
$client->close();
```

## API Reference

### Constructor

```php
new Client(string $host = "127.0.0.1", int $port = 7379, float $timeout = 5.0)
```

- `$host`: Server hostname or IP address
- `$port`: Server port (1-65535)
- `$timeout`: Operation timeout in seconds (must be positive)

### Methods

#### `set(string $key, string $value): void`

Set a key-value pair. Empty values are automatically handled.

**Parameters:**
- `$key`: Key to set (cannot be empty)
- `$value`: Value to set (empty strings are supported)

**Throws:**
- `InvalidArgumentException`: If key is empty
- `ConnectionException`: If connection fails
- `TimeoutException`: If operation times out
- `ProtocolException`: If server returns an error

#### `get(string $key): ?string`

Get a value by key.

**Parameters:**
- `$key`: Key to retrieve (cannot be empty)

**Returns:**
- `string`: The value if found
- `null`: If key not found

**Throws:**
- `InvalidArgumentException`: If key is empty
- `ConnectionException`: If connection fails
- `TimeoutException`: If operation times out
- `ProtocolException`: If server returns an error

#### `delete(string $key): bool`

Delete a key.

**Parameters:**
- `$key`: Key to delete (cannot be empty)

**Returns:**
- `true`: If key was deleted
- `false`: If key was not found

**Throws:**
- `InvalidArgumentException`: If key is empty
- `ConnectionException`: If connection fails
- `TimeoutException`: If operation times out
- `ProtocolException`: If server returns an error

#### `close(): void`

Close the connection. This method is idempotent and called automatically by the destructor.

#### `isConnected(): bool`

Check if the client is connected to the server.

## Error Handling

The client provides a comprehensive exception hierarchy:

```php
MerkleKvException (base class)
├── ConnectionException    // Network/connection errors
├── TimeoutException      // Operation timeouts
└── ProtocolException     // Server protocol errors
```

### Exception Examples

```php
use MerkleKV\Client;
use MerkleKV\ConnectionException;
use MerkleKV\TimeoutException;
use MerkleKV\ProtocolException;

try {
    $client = new Client("127.0.0.1", 7379);
    $client->set("key", "value");
} catch (ConnectionException $e) {
    echo "Connection failed: " . $e->getMessage();
} catch (TimeoutException $e) {
    echo "Operation timed out: " . $e->getMessage();
} catch (ProtocolException $e) {
    echo "Server error: " . $e->getMessage();
}
```

## Advanced Usage

### Custom Timeout and Connection Settings

```php
// Custom timeout (10 seconds)
$client = new Client("192.168.1.100", 7379, 10.0);

// Connection will be established on first operation
$client->set("key", "value");

// Check connection status
if ($client->isConnected()) {
    echo "Client is connected";
}

// Explicitly close when done
$client->close();
```

### Unicode and Special Characters

```php
$client = new Client();

// Unicode support
$client->set("greeting", "Hello, 世界! 🌍");
$value = $client->get("greeting");

// Special characters
$client->set("multiline", "Line 1\nLine 2\r\nTab\tEnd");

// Empty values
$client->set("empty", "");
$empty = $client->get("empty"); // Returns ""
```

### Error Recovery

```php
$client = new Client("127.0.0.1", 7379, 2.0);

for ($i = 0; $i < 5; $i++) {
    try {
        $client->set("key_{$i}", "value_{$i}");
        echo "Operation {$i} successful\n";
    } catch (ConnectionException $e) {
        echo "Connection failed on operation {$i}, retrying...\n";
        sleep(1);
        // Connection will be re-established automatically
    } catch (TimeoutException $e) {
        echo "Timeout on operation {$i}, skipping...\n";
    }
}
```

## Testing

Run the test suite:

```bash
# Install dependencies
composer install

# Run unit tests
vendor/bin/phpunit --testsuite="Unit Tests"

# Run integration tests (requires running server)
vendor/bin/phpunit --testsuite="Integration Tests" --group=integration

# Run all tests including performance tests
vendor/bin/phpunit --group=integration,performance

# Run with coverage
vendor/bin/phpunit --coverage-html coverage/
```

### Test Requirements

- **Unit Tests**: No external dependencies
- **Integration Tests**: Requires MerkleKV server running on `127.0.0.1:7379`
- **Performance Tests**: Requires server and measures <5ms operation latency

## Examples

See the [`examples/`](examples/) directory:

- [`basic.php`](examples/basic.php) - Basic operations demonstration
- [`performance.php`](examples/performance.php) - Performance testing and benchmarks

Run examples:

```bash
# Basic usage example
php examples/basic.php

# Performance benchmark
php examples/performance.php
```

## Protocol Compatibility

This client implements the MerkleKV TCP text protocol:

- **Encoding**: UTF-8
- **Termination**: CRLF (`\r\n`)
- **Commands**: `SET key value`, `GET key`, `DEL key`
- **Responses**: `OK`, `(null)`, `DELETED`, `NOT_FOUND`, `ERROR message`
- **Empty Values**: Represented as `""` at protocol layer

## Performance

The PHP client is optimized for performance:

- **Latency Target**: <5ms per operation
- **Connection Reuse**: Persistent connections across operations
- **Memory Efficiency**: Minimal memory allocation per operation
- **TCP Optimization**: `TCP_NODELAY` enabled for low latency

Performance benchmarks:
```
Operation    Min    Max    Avg    P50    P95    P99
SET (ms)    0.45   2.31   0.89   0.78   1.45   2.12
GET (ms)    0.42   1.98   0.76   0.71   1.23   1.87  
DELETE (ms) 0.48   2.45   0.93   0.82   1.58   2.33
```

## Requirements

- **PHP**: 7.4 or higher
- **Extensions**: `sockets` (usually included)
- **Server**: MerkleKV server accessible via TCP

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure:
- All tests pass
- Code follows PSR-12 standards
- Add tests for new features
- Update documentation as needed

## Support

- **Issues**: [GitHub Issues](https://github.com/merklekv/merklekv/issues)
- **Documentation**: [API Reference](#api-reference)
- **Examples**: [examples/](examples/) directory
