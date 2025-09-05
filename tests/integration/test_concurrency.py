"""
Concurrency and multi-client integration tests for MerkleKV.

Tests:
- Multiple concurrent clients
- Thread safety of operations
- Connection handling
- Race condition prevention
"""

import threading
import time
import random
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Dict, Tuple

import pytest
from conftest import MerkleKVClient, generate_test_data

class TestConcurrency:
    """Test concurrent access patterns."""
    
    def test_multiple_clients_same_key(self, server):
        """Test multiple clients accessing the same key concurrently."""
        num_clients = 10
        num_operations = 100
        
        def client_worker(client_id: int) -> List[Tuple[str, str]]:
            """Worker function for each client thread."""
            client = MerkleKVClient()
            client.connect()
            
            results = []
            try:
                for i in range(num_operations):
                    key = f"shared_key_{i % 10}"  # 10 different keys
                    value = f"value_from_client_{client_id}_{i}"
                    
                    # Set the key
                    response = client.set(key, value)
                    assert response == "OK"
                    
                    # Get the key (might be overwritten by other clients)
                    response = client.get(key)
                    results.append((key, response))
                    
                    # Small delay to increase chance of race conditions
                    time.sleep(0.001)
                    
            finally:
                client.disconnect()
            
            return results
        
        # Run multiple clients concurrently
        with ThreadPoolExecutor(max_workers=num_clients) as executor:
            futures = [executor.submit(client_worker, i) for i in range(num_clients)]
            all_results = []
            
            for future in as_completed(futures):
                results = future.result()
                all_results.extend(results)
        
        # Verify that all operations completed
        assert len(all_results) == num_clients * num_operations
        
        # Verify that keys still exist (last writer wins)
        client = MerkleKVClient()
        client.connect()
        
        for i in range(10):
            key = f"shared_key_{i}"
            response = client.get(key)
            assert response.startswith("VALUE")  # Key should exist with some value
        
        client.disconnect()
    
    def test_concurrent_reads_same_key(self, connected_client: MerkleKVClient):
        """Test multiple concurrent reads of the same key."""
        # Set a key first
        connected_client.set("read_test_key", "read_test_value")
        
        num_readers = 20
        reads_per_reader = 50
        
        def reader_worker(reader_id: int) -> List[str]:
            """Worker function for reader threads."""
            client = MerkleKVClient()
            client.connect()
            
            results = []
            try:
                for _ in range(reads_per_reader):
                    response = client.get("read_test_key")
                    results.append(response)
                    time.sleep(0.001)  # Small delay
            finally:
                client.disconnect()
            
            return results
        
        # Run multiple readers concurrently
        with ThreadPoolExecutor(max_workers=num_readers) as executor:
            futures = [executor.submit(reader_worker, i) for i in range(num_readers)]
            all_results = []
            
            for future in as_completed(futures):
                results = future.result()
                all_results.extend(results)
        
        # Verify all reads returned the correct value
        expected_response = "VALUE read_test_value"
        for response in all_results:
            assert response == expected_response
    
    def test_concurrent_writes_different_keys(self, server):
        """Test concurrent writes to different keys."""
        num_writers = 10
        keys_per_writer = 20
        
        def writer_worker(writer_id: int) -> List[Tuple[str, str]]:
            """Worker function for writer threads."""
            client = MerkleKVClient()
            client.connect()
            
            results = []
            try:
                for i in range(keys_per_writer):
                    key = f"writer_{writer_id}_key_{i}"
                    value = f"value_from_writer_{writer_id}_{i}"
                    
                    response = client.set(key, value)
                    assert response == "OK"
                    results.append((key, value))
                    
                    time.sleep(0.001)  # Small delay
            finally:
                client.disconnect()
            
            return results
        
        # Run multiple writers concurrently
        with ThreadPoolExecutor(max_workers=num_writers) as executor:
            futures = [executor.submit(writer_worker, i) for i in range(num_writers)]
            all_results = []
            
            for future in as_completed(futures):
                results = future.result()
                all_results.extend(results)
        
        # Verify all writes were successful
        assert len(all_results) == num_writers * keys_per_writer
        
        # Verify all keys exist with correct values
        client = MerkleKVClient()
        client.connect()
        
        for key, expected_value in all_results:
            response = client.get(key)
            assert response == f"VALUE {expected_value}"
        
        client.disconnect()
    
    def test_concurrent_mixed_operations(self, server):
        """Test concurrent mix of reads, writes, and deletes."""
        num_workers = 15
        operations_per_worker = 30
        
        def mixed_worker(worker_id: int) -> List[Tuple[str, str]]:
            """Worker function with mixed operations."""
            client = MerkleKVClient()
            client.connect()
            
            results = []
            try:
                for i in range(operations_per_worker):
                    operation = i % 4  # 0=set, 1=get, 2=delete, 3=set
                    key = f"mixed_key_{worker_id}_{i}"
                    
                    if operation == 0:  # SET
                        value = f"value_{worker_id}_{i}"
                        response = client.set(key, value)
                        assert response == "OK"
                        results.append((f"SET_{key}", response))
                    
                    elif operation == 1:  # GET
                        response = client.get(key)
                        results.append((f"GET_{key}", response))
                    
                    elif operation == 2:  # DELETE
                        response = client.delete(key)
                        # Accept both DELETED (key exists) and NOT_FOUND (key doesn't exist)
                        assert response in ["DELETED", "NOT_FOUND"]
                        results.append((f"DELETE_{key}", response))
                    
                    elif operation == 3:  # Another SET
                        value = f"value_{worker_id}_{i}_updated"
                        response = client.set(key, value)
                        assert response == "OK"
                        results.append((f"SET_{key}", response))
                    
                    time.sleep(0.001)  # Small delay
            finally:
                client.disconnect()
            
            return results
        
        # Run mixed operations concurrently
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            futures = [executor.submit(mixed_worker, i) for i in range(num_workers)]
            all_results = []
            
            for future in as_completed(futures):
                results = future.result()
                all_results.extend(results)
        
        # Verify all operations completed
        assert len(all_results) == num_workers * operations_per_worker
        
        # Verify no errors occurred
        for operation, response in all_results:
            assert not response.startswith("ERROR")
    
    def test_connection_stress_test(self, server):
        """Test many rapid connections and disconnections."""
        num_connections = 50
        
        def connection_worker(conn_id: int) -> bool:
            """Worker that connects, does one operation, and disconnects."""
            try:
                client = MerkleKVClient()
                client.connect()
                
                # Do a simple operation
                key = f"stress_key_{conn_id}"
                value = f"stress_value_{conn_id}"
                
                response = client.set(key, value)
                assert response == "OK"
                
                response = client.get(key)
                assert response == f"VALUE {value}"
                
                client.disconnect()
                return True
                
            except Exception as e:
                print(f"Connection {conn_id} failed: {e}")
                return False
        
        # Run many connections concurrently
        with ThreadPoolExecutor(max_workers=num_connections) as executor:
            futures = [executor.submit(connection_worker, i) for i in range(num_connections)]
            results = [future.result() for future in as_completed(futures)]
        
        # Verify all connections succeeded
        successful_connections = sum(results)
        assert successful_connections == num_connections
        
        # Verify data is consistent
        client = MerkleKVClient()
        client.connect()
        
        for i in range(num_connections):
            key = f"stress_key_{i}"
            expected_value = f"stress_value_{i}"
            response = client.get(key)
            assert response == f"VALUE {expected_value}"
        
        client.disconnect()
    
    def test_rapid_operations_single_client(self, connected_client: MerkleKVClient):
        """Test rapid operations from a single client."""
        num_operations = 1000
        
        start_time = time.time()
        
        for i in range(num_operations):
            key = f"rapid_key_{i}"
            value = f"rapid_value_{i}"
            
            # Set
            response = connected_client.set(key, value)
            assert response == "OK"
            
            # Get
            response = connected_client.get(key)
            assert response == f"VALUE {value}"
            
            # Delete
            response = connected_client.delete(key)
            assert response == "DELETED"  # Key exists, so expect DELETED
            
            # Verify deleted
            response = connected_client.get(key)
            assert response == "NOT_FOUND"
        
        end_time = time.time()
        duration = end_time - start_time
        operations_per_second = (num_operations * 4) / duration  # 4 ops per iteration
        
        print(f"Rapid operations: {operations_per_second:.2f} ops/sec")
        assert operations_per_second > 100  # Should handle at least 100 ops/sec
    
    def test_thread_safety_with_shared_data(self, server):
        """Test thread safety when multiple threads access shared data."""
        shared_key = "shared_thread_key"
        num_threads = 8
        operations_per_thread = 100
        
        # Set initial value
        client = MerkleKVClient()
        client.connect()
        client.set(shared_key, "initial_value")
        client.disconnect()
        
        def thread_worker(thread_id: int) -> List[str]:
            """Worker function for each thread."""
            client = MerkleKVClient()
            client.connect()
            
            results = []
            try:
                for i in range(operations_per_thread):
                    # Read current value
                    response = client.get(shared_key)
                    results.append(response)
                    
                    # Write new value
                    new_value = f"value_from_thread_{thread_id}_{i}"
                    response = client.set(shared_key, new_value)
                    assert response == "OK"
                    
                    time.sleep(0.001)  # Small delay
            finally:
                client.disconnect()
            
            return results
        
        # Run multiple threads
        threads = []
        all_results = []
        
        for i in range(num_threads):
            thread = threading.Thread(target=lambda tid=i: all_results.extend(thread_worker(tid)))
            threads.append(thread)
            thread.start()
        
        # Wait for all threads to complete
        for thread in threads:
            thread.join()
        
        # Verify all operations completed
        assert len(all_results) == num_threads * operations_per_thread
        
        # Verify final state is consistent
        client = MerkleKVClient()
        client.connect()
        response = client.get(shared_key)
        assert response.startswith("VALUE")  # Should have some value
        client.disconnect() 