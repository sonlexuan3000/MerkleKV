# Replication Testing Guide

This document describes how to t# Test MQTT broker connectivity
pytest -v -k test_mqtt_broker_connectivity

# Simple replication test
pytest -v test_replication_simple.py

# Run full replication test suite
pytest -v test_replication.py

# Run specific test
pytest -v -k test_set_operation_replication
```

## Test structure

### test_replication_simple.py

Simple tests to verify:
- Connection to public MQTT broker
- Start 2 servers with replication enabled
- Basic connectivity testing

### test_replication.py

Full test suite including:

#### 1. Basic tests
- `test_basic_replication_setup`: Initialize multiple nodes
- `test_set_operation_replication`: SET operation replication
- `test_delete_operation_replication`: DELETE operation replication

#### 2. Numeric/string operation tests
- `test_numeric_operations_replication`: INCR/DECR
- `test_string_operations_replication`: APPEND/PREPEND

#### 3. Concurrent and edge case tests
- `test_concurrent_operations_replication`: Concurrent operations
- `test_replication_with_node_restart`: Node restart scenarios
- `test_replication_loop_prevention`: Infinite loop prevention
- `test_malformed_mqtt_message_handling`: Malformed message handling

## MQTT Configuration

Tests use a public MQTT broker:
- **Broker**: test.mosquitto.org
- **Port**: 1883
- **Topic pattern**: `test_merkle_kv_{random_id}/events/#`

Each test uses a different topic prefix to avoid conflicts.

## Test Architecture

### ReplicationTestSetup
Helper class managing multiple MerkleKV instances:
- Create config files with replication enabled
- Start/stop server instances
- Automatic cleanup

### MQTTTestClient
Client to monitor MQTT messages:
- Subscribe to replication topics
- Decode messages (JSON or CBOR)
- Message tracking for verification

## Troubleshooting

### 1. MQTT connection error
```
Failed to connect to MQTT broker
```
**Solution**: Check internet connection and firewall. The test.mosquitto.org broker can sometimes be overloaded.

### 2. Server startup failure
```
Server failed to start within timeout
```
**Solution**: 
- Check for port conflicts
- Review server output logs for debugging
- Increase timeout in config

### 3. Replication not working
```
Expected replicated_value, got (nil)
```
**Solution**:
- Check replication configuration in server
- Verify MQTT topic names
- Increase replication wait time

### 4. Slow tests
**Cause**: MQTT network latency, server startup time
**Solution**: 
- Run tests sequentially instead of parallel
- Use local MQTT broker for faster testing

## Customizing tests

### Using different MQTT broker

Edit in test files:
```python
mqtt_config = {
    "mqtt_broker": "your-broker.com",
    "mqtt_port": 1883,
    # ...
}
```

### Adding new test cases

1. Create test function with `test_` prefix
2. Use `@pytest.mark.asyncio` for async tests
3. Use `replication_setup` fixture to manage servers
4. Follow pattern: setup → action → verify → cleanup

### Debug tests

Run with verbose output:
```bash
pytest -v -s test_replication.py
```

Enable Rust logging:
```bash
RUST_LOG=debug pytest -v test_replication.py
```

## CI/CD Integration

To integrate into CI/CD pipeline:

```yaml
# Example GitHub Actions
- name: Run replication tests
  run: |
    cd tests/integration
    python run_replication_tests.py all
```

**Note**: Tests use external MQTT broker so may fail due to network issues. Consider setting up MQTT broker in CI environment.V's replication functionality using a public MQTT broker.

## Overview

MerkleKV's replication system uses MQTT to synchronize write operations between nodes in a cluster. The test cases are designed to:

- Test MQTT broker connectivity
- Verify replication of SET, DELETE, INCR, DECR, APPEND, PREPEND operations
- Test infinite loop prevention
- Test error handling for malformed messages
- Test replication in concurrent environments

## Setup

### 1. Install dependencies

```bash
cd tests/integration
pip install -r requirements.txt
```

### 2. Build MerkleKV server

```bash
# From project root directory
cargo build
```

## Running tests

### Utility script

Use the `run_replication_tests.py` script to run tests:

```bash
cd tests/integration

# Run all (install deps + build + tests)
python run_replication_tests.py all

# Test MQTT connectivity only
python run_replication_tests.py connectivity

# Simple replication test
python run_replication_tests.py simple

# Run full test suite
python run_replication_tests.py full

# Install dependencies
python run_replication_tests.py install-deps

# Build server
python run_replication_tests.py build
```

### Run tests directly with pytest

```bash
# Test kết nối MQTT broker
pytest -v -k test_mqtt_broker_connectivity

# Test replication đơn giản
pytest -v test_replication_simple.py

# Chạy toàn bộ test suite replication
pytest -v test_replication.py

# Chạy test cụ thể
pytest -v -k test_set_operation_replication
```

## Cấu trúc test cases

### test_replication_simple.py

Test đơn giản để kiểm tra:
- Kết nối đến MQTT broker công cộng
- Khởi tạo 2 server với replication enabled
- Kiểm tra connectivity cơ bản

### test_replication.py

Test suite đầy đủ bao gồm:

#### 1. Test cơ bản
- `test_basic_replication_setup`: Khởi tạo nhiều node
- `test_set_operation_replication`: Nhân bản thao tác SET
- `test_delete_operation_replication`: Nhân bản thao tác DELETE

#### 2. Test các thao tác numeric/string
- `test_numeric_operations_replication`: INCR/DECR
- `test_string_operations_replication`: APPEND/PREPEND

#### 3. Test concurrent và edge cases
- `test_concurrent_operations_replication`: Thao tác đồng thời
- `test_replication_with_node_restart`: Khởi động lại node
- `test_replication_loop_prevention`: Chống loop vô hạn
- `test_malformed_mqtt_message_handling`: Xử lý message lỗi

## Cấu hình MQTT

Tests sử dụng MQTT broker công cộng:
- **Broker**: test.mosquitto.org
- **Port**: 1883
- **Topic pattern**: `test_merkle_kv_{random_id}/events/#`

Mỗi test sử dụng topic prefix khác nhau để tránh xung đột.

## Kiến trúc test

### ReplicationTestSetup
Helper class quản lý nhiều MerkleKV instances:
- Tạo config file với replication enabled
- Khởi động/dừng các server instances
- Cleanup tự động

### MQTTTestClient
Client để monitor MQTT messages:
- Subscribe đến replication topics
- Decode messages (JSON hoặc CBOR)
- Tracking messages cho verification

## Troubleshooting

### 1. Lỗi kết nối MQTT
```
Failed to connect to MQTT broker
```
**Giải pháp**: Kiểm tra kết nối internet và firewall. Broker test.mosquitto.org đôi khi có thể bị quá tải.

### 2. Server không khởi động
```
Server failed to start within timeout
```
**Giải pháp**: 
- Kiểm tra port có bị conflict không
- Xem log server output để debug
- Tăng timeout trong config

### 3. Replication không hoạt động
```
Expected replicated_value, got (nil)
```
**Giải pháp**:
- Kiểm tra cấu hình replication trong server
- Verify MQTT topic names
- Tăng thời gian chờ replication

### 4. Tests chạy chậm
**Nguyên nhân**: MQTT network latency, server startup time
**Giải pháp**: 
- Chạy tests tuần tự thay vì parallel
- Sử dụng local MQTT broker cho testing nhanh hơn

## Tùy chỉnh tests

### Sử dụng MQTT broker khác

Chỉnh sửa trong test files:
```python
mqtt_config = {
    "mqtt_broker": "your-broker.com",
    "mqtt_port": 1883,
    # ...
}
```

### Thêm test cases mới

1. Tạo function test với prefix `test_`
2. Sử dụng `@pytest.mark.asyncio` cho async tests
3. Sử dụng `replication_setup` fixture để quản lý servers
4. Follow pattern: setup → action → verify → cleanup

### Debug tests

Chạy với verbose output:
```bash
pytest -v -s test_replication.py
```

Enable Rust logging:
```bash
RUST_LOG=debug pytest -v test_replication.py
```

## Tích hợp CI/CD

Để tích hợp vào CI/CD pipeline:

```yaml
# Example GitHub Actions
- name: Run replication tests
  run: |
    cd tests/integration
    python run_replication_tests.py all
```

**Lưu ý**: Test sử dụng external MQTT broker nên có thể bị fail do network issues. Cân nhắc setup MQTT broker trong CI environment.
