# MerkleKV Integration Test Suite

This directory contains comprehensive integration tests for the MerkleKV distributed key-value store. The tests validate the entire system end-to-end, ensuring all components work together correctly under various conditions.

## 🚀 Quick Start

### Prerequisites

1. **Python 3.8+** installed
2. **Rust toolchain** (for building MerkleKV server)
3. **Cargo** available in PATH

### Installation

```bash
# Install Python dependencies
cd tests/integration
pip install -r requirements.txt

# Build the MerkleKV server
cd ../..  # Go back to project root
cargo build
```

### Running Tests

```bash
# Run basic tests
python run_tests.py --mode basic

# Run all tests (except benchmarks)
python run_tests.py --mode all

# Run benchmarks only
python run_tests.py --benchmark-only

# Run with verbose output
python run_tests.py --mode concurrency --verbose

# Run CI/CD tests with coverage
python run_tests.py --mode ci --report
```

## 📋 Test Categories

### 1. Basic Operations (`test_basic_operations.py`)

Tests the core functionality of the key-value store:

- ✅ GET, SET, DELETE commands
- ✅ Error handling for invalid commands
- ✅ Data persistence across server restarts
- ✅ Special characters and Unicode support
- ✅ Large value handling
- ✅ Command case insensitivity

### 2. Concurrency (`test_concurrency.py`)

Tests thread safety and concurrent access:

- ✅ Multiple clients accessing same keys
- ✅ Concurrent reads and writes
- ✅ Connection stress testing
- ✅ Thread safety validation
- ✅ Race condition prevention

### 3. Performance Benchmarks (`test_benchmark.py`)

Measures system performance under load:

- ✅ Throughput testing (ops/sec)
- ✅ Latency measurements (P50, P95, P99)
- ✅ Memory and CPU usage monitoring
- ✅ Scalability testing with multiple clients
- ✅ Connection handling capacity

### 4. Error Handling (`test_error_handling.py`)

Tests system resilience and error recovery:

- ✅ Invalid command handling
- ✅ Malformed protocol messages
- ✅ Network partition simulation
- ✅ Server crash recovery
- ✅ Resource cleanup validation

## 🎯 Test Modes

### Basic Mode

```bash
python run_tests.py --mode basic
```

- Fast execution (~30 seconds)
- Core functionality validation
- Suitable for development and CI

### Concurrency Mode

```bash
python run_tests.py --mode concurrency
```

- Tests thread safety
- Multiple client scenarios
- Connection handling

### Benchmark Mode

```bash
python run_tests.py --mode benchmark
```

- Performance measurements
- Load testing
- Resource usage analysis

### Error Mode

```bash
python run_tests.py --mode error
```

- Error handling validation
- Recovery scenario testing
- Edge case coverage

### CI Mode

```bash
python run_tests.py --mode ci
```

- Full test suite (except benchmarks)
- Coverage reporting
- JUnit XML output
- HTML coverage reports

### All Mode

```bash
python run_tests.py --mode all
```

- Complete test suite
- All categories except benchmarks
- Comprehensive validation

## 🔧 Configuration

### Environment Variables

```bash
# Server configuration
export MERKLEKV_TEST_HOST="127.0.0.1"
export MERKLEKV_TEST_PORT="7379"

# Test configuration
export RUST_LOG="info"
export PYTHONPATH="tests/integration"
```

### Command Line Options

```bash
python run_tests.py [options]

Options:
  --mode MODE           Test mode: basic, concurrency, benchmark, error, ci, all
  --host HOST           Server host (default: 127.0.0.1)
  --port PORT           Server port (default: 7379)
  --workers N           Number of parallel workers (default: auto)
  --verbose             Verbose output
  --report              Generate detailed report
  --benchmark-only      Run only benchmark tests
  --help                Show help message
```

## 📊 Performance Benchmarks

### Expected Performance Metrics

| Operation   | Throughput     | Latency (P95) | Memory Usage |
| ----------- | -------------- | ------------- | ------------ |
| SET         | >1,000 ops/sec | <100ms        | <100MB       |
| GET         | >2,000 ops/sec | <50ms         | <100MB       |
| Mixed       | >800 ops/sec   | <80ms         | <100MB       |
| Connections | >50 conn/sec   | <1s           | <200MB       |

### Running Benchmarks

```bash
# Quick benchmark
python run_tests.py --mode benchmark

# Detailed benchmark with verbose output
python run_tests.py --benchmark-only --verbose

# Scalability test
python run_tests.py --mode benchmark --workers 4
```

## 🏗️ Architecture

### Test Structure

```
tests/integration/
├── conftest.py              # Pytest configuration and fixtures
├── test_basic_operations.py # Basic functionality tests
├── test_concurrency.py      # Concurrency and thread safety
├── test_benchmark.py        # Performance benchmarks
├── test_error_handling.py   # Error handling and recovery
├── run_tests.py            # Main test runner
├── requirements.txt        # Python dependencies
└── README.md              # This file
```

### Key Components

1. **MerkleKVServer**: Manages server process lifecycle
2. **MerkleKVClient**: TCP client for server communication
3. **PerformanceMonitor**: System resource monitoring
4. **BenchmarkResult**: Performance measurement data
5. **TestRunner**: Orchestrates test execution

## 🐛 Troubleshooting

### Common Issues

1. **Server won't start**

   ```bash
   # Check if port is available
   netstat -tuln | grep 7379

   # Check Rust installation
   cargo --version
   ```

2. **Tests fail with connection errors**

   ```bash
   # Check server logs
   RUST_LOG=debug cargo run

   # Verify server is running
   telnet 127.0.0.1 7379
   ```

3. **Performance tests are slow**

   ```bash
   # Reduce test load
   python run_tests.py --mode benchmark --workers 1

   # Check system resources
   htop
   ```

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug python run_tests.py --mode basic --verbose

# Run single test
python -m pytest tests/integration/test_basic_operations.py::TestBasicOperations::test_set_and_get_single_key -v -s
```

## 📈 CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-python@v4
        with:
          python-version: "3.9"
      - uses: actions/setup-rust@v1
        with:
          toolchain: stable

      - name: Install dependencies
        run: |
          pip install -r tests/integration/requirements.txt
          cargo build

      - name: Run integration tests
        run: |
          cd tests/integration
          python run_tests.py --mode ci --report

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          file: tests/integration/coverage.xml
```

### Local CI Simulation

```bash
# Run CI tests locally
python run_tests.py --mode ci --report --workers 4

# Check coverage
open htmlcov/index.html  # macOS
xdg-open htmlcov/index.html  # Linux
```

## 📝 Test Development

### Adding New Tests

1. **Create test file**: `test_new_feature.py`
2. **Add test class**: Inherit from existing patterns
3. **Use fixtures**: Leverage `connected_client`, `server` fixtures
4. **Add markers**: Use `@pytest.mark.benchmark` for performance tests

### Example Test Structure

```python
import pytest
from conftest import MerkleKVClient

class TestNewFeature:
    """Test new feature functionality."""

    def test_new_feature_basic(self, connected_client: MerkleKVClient):
        """Test basic new feature functionality."""
        # Test implementation
        response = connected_client.send_command("NEW_COMMAND arg")
        assert response == "OK"

    @pytest.mark.benchmark
    def test_new_feature_performance(self, server):
        """Test new feature performance."""
        # Performance test implementation
        pass
```

### Best Practices

1. **Use descriptive test names**: Clear, specific test names
2. **Test one thing per test**: Single responsibility principle
3. **Use appropriate fixtures**: Reuse existing infrastructure
4. **Add proper assertions**: Clear failure messages
5. **Handle cleanup**: Ensure tests don't leave state

## 🤝 Contributing

### Running Tests Before Committing

```bash
# Run all tests
python run_tests.py --mode all

# Run benchmarks
python run_tests.py --benchmark-only

# Check for regressions
python run_tests.py --mode ci --report
```

### Test Guidelines

1. **Maintain test isolation**: Tests should not depend on each other
2. **Use realistic data**: Test with real-world scenarios
3. **Measure performance**: Include benchmarks for new features
4. **Document edge cases**: Test error conditions thoroughly
5. **Keep tests fast**: Optimize for quick feedback

## 📚 Additional Resources

- [MerkleKV Project Documentation](../README.md)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Pytest Documentation](https://docs.pytest.org/)
- [Python asyncio](https://docs.python.org/3/library/asyncio.html)

## 📞 Support

For issues with the integration tests:

1. Check the troubleshooting section above
2. Review server logs with `RUST_LOG=debug`
3. Run tests with `--verbose` flag
4. Create an issue with test output and error details
