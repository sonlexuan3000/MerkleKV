"""
Benchmark and performance tests for MerkleKV.

Tests:
- Throughput under load
- Latency measurements
- Memory usage
- CPU usage
- Scalability with multiple clients
"""

import time
import statistics
import psutil
import os
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Dict, Tuple, NamedTuple
from dataclasses import dataclass

import pytest
from conftest import MerkleKVClient, generate_test_data

@dataclass
class BenchmarkResult:
    """Results from a benchmark test."""
    operation: str
    total_operations: int
    total_time: float
    operations_per_second: float
    avg_latency_ms: float
    p50_latency_ms: float
    p95_latency_ms: float
    p99_latency_ms: float
    min_latency_ms: float
    max_latency_ms: float
    errors: int

class PerformanceMonitor:
    """Monitor system performance during benchmarks."""
    
    def __init__(self):
        self.process = psutil.Process(os.getpid())
        self.start_cpu_percent = None
        self.start_memory_mb = None
        self.peak_memory_mb = None
        
    def start_monitoring(self):
        """Start monitoring system resources."""
        self.start_cpu_percent = self.process.cpu_percent()
        self.start_memory_mb = self.process.memory_info().rss / 1024 / 1024
        self.peak_memory_mb = self.start_memory_mb
        
    def update_peak_memory(self):
        """Update peak memory usage."""
        current_memory = self.process.memory_info().rss / 1024 / 1024
        self.peak_memory_mb = max(self.peak_memory_mb, current_memory)
        
    def get_stats(self) -> Dict[str, float]:
        """Get current performance statistics."""
        current_memory = self.process.memory_info().rss / 1024 / 1024
        return {
            "cpu_percent": self.process.cpu_percent(),
            "memory_mb": current_memory,
            "peak_memory_mb": self.peak_memory_mb,
            "memory_increase_mb": current_memory - self.start_memory_mb
        }

class TestBenchmarks:
    """Benchmark tests for performance measurement."""
    
    def benchmark_operation(self, operation_func, num_operations: int, 
                          num_clients: int = 1) -> BenchmarkResult:
        """Run a benchmark for a specific operation."""
        latencies = []
        errors = 0
        start_time = time.time()
        
        def client_worker(client_id: int) -> Tuple[List[float], int]:
            """Worker function for benchmark client."""
            client = MerkleKVClient()
            client.connect()
            
            client_latencies = []
            client_errors = 0
            
            try:
                for i in range(num_operations // num_clients):
                    op_start = time.time()
                    
                    try:
                        operation_func(client, client_id, i)
                        op_end = time.time()
                        latency_ms = (op_end - op_start) * 1000
                        client_latencies.append(latency_ms)
                    except Exception as e:
                        client_errors += 1
                        print(f"Error in client {client_id}, operation {i}: {e}")
                        
            finally:
                client.disconnect()
            
            return client_latencies, client_errors
        
        # Run benchmark with multiple clients
        with ThreadPoolExecutor(max_workers=num_clients) as executor:
            futures = [executor.submit(client_worker, i) for i in range(num_clients)]
            
            for future in as_completed(futures):
                client_latencies, client_errors = future.result()
                latencies.extend(client_latencies)
                errors += client_errors
        
        end_time = time.time()
        total_time = end_time - start_time
        
        if not latencies:
            raise RuntimeError("No successful operations completed")
        
        # Calculate statistics
        latencies.sort()
        total_ops = len(latencies)
        ops_per_sec = total_ops / total_time
        
        return BenchmarkResult(
            operation=operation_func.__name__,
            total_operations=total_ops,
            total_time=total_time,
            operations_per_second=ops_per_sec,
            avg_latency_ms=statistics.mean(latencies),
            p50_latency_ms=statistics.median(latencies),
            p95_latency_ms=latencies[int(len(latencies) * 0.95)],
            p99_latency_ms=latencies[int(len(latencies) * 0.99)],
            min_latency_ms=min(latencies),
            max_latency_ms=max(latencies),
            errors=errors
        )
    
    def print_benchmark_result(self, result: BenchmarkResult):
        """Print benchmark results in a formatted way."""
        print(f"\n{'='*60}")
        print(f"BENCHMARK RESULTS: {result.operation}")
        print(f"{'='*60}")
        print(f"Total Operations: {result.total_operations:,}")
        print(f"Total Time: {result.total_time:.2f}s")
        print(f"Throughput: {result.operations_per_second:.2f} ops/sec")
        print(f"Errors: {result.errors}")
        print(f"\nLatency Statistics (ms):")
        print(f"  Average: {result.avg_latency_ms:.2f}")
        print(f"  Median (P50): {result.p50_latency_ms:.2f}")
        print(f"  95th Percentile (P95): {result.p95_latency_ms:.2f}")
        print(f"  99th Percentile (P99): {result.p99_latency_ms:.2f}")
        print(f"  Min: {result.min_latency_ms:.2f}")
        print(f"  Max: {result.max_latency_ms:.2f}")
        print(f"{'='*60}")
    
    def benchmark_set_operations(self, server, num_operations: int = 10000, 
                               num_clients: int = 10):
        """Benchmark SET operations."""
        def set_operation(client: MerkleKVClient, client_id: int, op_id: int):
            key = f"benchmark_set_{client_id}_{op_id}"
            value = f"value_{client_id}_{op_id}"
            response = client.set(key, value)
            assert response == "OK"
        
        monitor = PerformanceMonitor()
        monitor.start_monitoring()
        
        result = self.benchmark_operation(set_operation, num_operations, num_clients)
        
        monitor.update_peak_memory()
        stats = monitor.get_stats()
        
        self.print_benchmark_result(result)
        print(f"Memory Usage: {stats['memory_mb']:.1f}MB (Peak: {stats['peak_memory_mb']:.1f}MB)")
        print(f"CPU Usage: {stats['cpu_percent']:.1f}%")
        
        # Performance assertions
        assert result.operations_per_second > 1000, f"Throughput too low: {result.operations_per_second} ops/sec"
        assert result.avg_latency_ms < 100, f"Average latency too high: {result.avg_latency_ms}ms"
        assert result.errors == 0, f"Unexpected errors: {result.errors}"
        
        return result
    
    def benchmark_get_operations(self, server, num_operations: int = 10000, 
                               num_clients: int = 10):
        """Benchmark GET operations."""
        # Pre-populate with data
        client = MerkleKVClient()
        client.connect()
        for i in range(1000):  # Pre-populate 1000 keys
            key = f"benchmark_get_{i}"
            value = f"value_{i}"
            client.set(key, value)
        client.disconnect()
        
        def get_operation(client: MerkleKVClient, client_id: int, op_id: int):
            key = f"benchmark_get_{op_id % 1000}"  # Cycle through pre-populated keys
            response = client.get(key)
            assert response.startswith("VALUE")
        
        monitor = PerformanceMonitor()
        monitor.start_monitoring()
        
        result = self.benchmark_operation(get_operation, num_operations, num_clients)
        
        monitor.update_peak_memory()
        stats = monitor.get_stats()
        
        self.print_benchmark_result(result)
        print(f"Memory Usage: {stats['memory_mb']:.1f}MB (Peak: {stats['peak_memory_mb']:.1f}MB)")
        print(f"CPU Usage: {stats['cpu_percent']:.1f}%")
        
        # Performance assertions
        assert result.operations_per_second > 2000, f"Throughput too low: {result.operations_per_second} ops/sec"
        assert result.avg_latency_ms < 50, f"Average latency too high: {result.avg_latency_ms}ms"
        assert result.errors == 0, f"Unexpected errors: {result.errors}"
        
        return result
    
    def benchmark_mixed_operations(self, server, num_operations: int = 15000, 
                                 num_clients: int = 15):
        """Benchmark mixed SET/GET/DELETE operations."""
        def mixed_operation(client: MerkleKVClient, client_id: int, op_id: int):
            operation = op_id % 3  # 0=SET, 1=GET, 2=DELETE
            key = f"benchmark_mixed_{client_id}_{op_id}"
            
            if operation == 0:  # SET
                value = f"value_{client_id}_{op_id}"
                response = client.set(key, value)
                assert response == "OK"
            elif operation == 1:  # GET
                response = client.get(key)
                # Might be NOT_FOUND if DELETE happened first
            elif operation == 2:  # DELETE
                response = client.delete(key)
                assert response == "OK"
        
        monitor = PerformanceMonitor()
        monitor.start_monitoring()
        
        result = self.benchmark_operation(mixed_operation, num_operations, num_clients)
        
        monitor.update_peak_memory()
        stats = monitor.get_stats()
        
        self.print_benchmark_result(result)
        print(f"Memory Usage: {stats['memory_mb']:.1f}MB (Peak: {stats['peak_memory_mb']:.1f}MB)")
        print(f"CPU Usage: {stats['cpu_percent']:.1f}%")
        
        # Performance assertions
        assert result.operations_per_second > 800, f"Throughput too low: {result.operations_per_second} ops/sec"
        assert result.avg_latency_ms < 80, f"Average latency too high: {result.avg_latency_ms}ms"
        assert result.errors == 0, f"Unexpected errors: {result.errors}"
        
        return result
    
    def benchmark_concurrent_connections(self, server, num_connections: int = 100):
        """Benchmark handling many concurrent connections."""
        def connection_worker(conn_id: int) -> Tuple[bool, float]:
            start_time = time.time()
            try:
                client = MerkleKVClient()
                client.connect()
                
                # Do some operations
                for i in range(10):
                    key = f"conn_{conn_id}_{i}"
                    value = f"value_{conn_id}_{i}"
                    client.set(key, value)
                    client.get(key)
                
                client.disconnect()
                end_time = time.time()
                return True, end_time - start_time
                
            except Exception as e:
                print(f"Connection {conn_id} failed: {e}")
                return False, 0
        
        monitor = PerformanceMonitor()
        monitor.start_monitoring()
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_connections) as executor:
            futures = [executor.submit(connection_worker, i) for i in range(num_connections)]
            results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        total_time = end_time - start_time
        
        successful_connections = sum(1 for success, _ in results if success)
        connection_times = [time for success, time in results if success]
        
        monitor.update_peak_memory()
        stats = monitor.get_stats()
        
        print(f"\n{'='*60}")
        print(f"CONCURRENT CONNECTIONS BENCHMARK")
        print(f"{'='*60}")
        print(f"Total Connections: {num_connections}")
        print(f"Successful Connections: {successful_connections}")
        print(f"Success Rate: {(successful_connections/num_connections)*100:.1f}%")
        print(f"Total Time: {total_time:.2f}s")
        print(f"Connections per second: {successful_connections/total_time:.2f}")
        if connection_times:
            print(f"Average connection time: {statistics.mean(connection_times):.3f}s")
        print(f"Memory Usage: {stats['memory_mb']:.1f}MB (Peak: {stats['peak_memory_mb']:.1f}MB)")
        print(f"CPU Usage: {stats['cpu_percent']:.1f}%")
        print(f"{'='*60}")
        
        # Performance assertions
        assert successful_connections >= num_connections * 0.95, f"Too many failed connections: {successful_connections}/{num_connections}"
        assert total_time < 30, f"Connection handling too slow: {total_time}s"
        
        return {
            "total_connections": num_connections,
            "successful_connections": successful_connections,
            "success_rate": successful_connections / num_connections,
            "total_time": total_time,
            "connections_per_second": successful_connections / total_time,
            "avg_connection_time": statistics.mean(connection_times) if connection_times else 0
        }
    
    def benchmark_scalability(self, server):
        """Benchmark scalability with different numbers of clients."""
        client_counts = [1, 5, 10, 20, 50]
        results = {}
        
        for num_clients in client_counts:
            print(f"\nTesting with {num_clients} clients...")
            result = self.benchmark_set_operations(server, num_operations=1000, num_clients=num_clients)
            results[num_clients] = result.operations_per_second
        
        print(f"\n{'='*60}")
        print(f"SCALABILITY BENCHMARK RESULTS")
        print(f"{'='*60}")
        for num_clients, throughput in results.items():
            print(f"{num_clients:2d} clients: {throughput:8.2f} ops/sec")
        print(f"{'='*60}")
        
        # Verify some scalability (not necessarily linear)
        assert results[10] > results[1] * 0.5, "Poor scalability with 10 clients"
        assert results[20] > results[1] * 0.3, "Poor scalability with 20 clients"
        
        return results

@pytest.mark.benchmark
class TestPerformanceBenchmarks(TestBenchmarks):
    """Performance benchmark tests (marked as slow)."""
    
    def test_set_throughput_benchmark(self, server):
        """Benchmark SET operation throughput."""
        self.benchmark_set_operations(server, num_operations=5000, num_clients=5)
    
    def test_get_throughput_benchmark(self, server):
        """Benchmark GET operation throughput."""
        self.benchmark_get_operations(server, num_operations=5000, num_clients=5)
    
    def test_mixed_operations_benchmark(self, server):
        """Benchmark mixed operations throughput."""
        self.benchmark_mixed_operations(server, num_operations=7500, num_clients=10)
    
    def test_concurrent_connections_benchmark(self, server):
        """Benchmark concurrent connection handling."""
        self.benchmark_concurrent_connections(server, num_connections=50)
    
    def test_scalability_benchmark(self, server):
        """Benchmark scalability with different client counts."""
        self.benchmark_scalability(server) 