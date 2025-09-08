/**
 * Extended tests for Node.js MerkleKV client to verify protocol compliance,
 * performance, and advanced features like pipeline operations and health checks.
 */

import { MerkleKVClient, ConnectionError, TimeoutError } from '../src';
import * as net from 'net';

describe('MerkleKVClient Extended Tests', () => {
    let client: MerkleKVClient;
    const TEST_PORT = 7379; // Port where the test server is running
    
    beforeEach(() => {
        client = new MerkleKVClient('localhost', TEST_PORT, 10000);
    });

    afterEach(async () => {
        if (client.isConnected()) {
            await client.close();
        }
    });

    describe('TCP_NODELAY Configuration', () => {
        test('should connect with TCP_NODELAY enabled for optimal latency', async () => {
            // This test verifies that socket.setNoDelay(true) is called
            // We can't directly test this without mocking, but we can test
            // that connection works properly and performance is good
            await client.connect();
            expect(client.isConnected()).toBe(true);
            
            // Test basic operation to ensure TCP_NODELAY doesn't break functionality
            await client.set('nodelay_test', 'value');
            const result = await client.get('nodelay_test');
            expect(result).toBe('value');
        });
    });

    describe('Health Check Operations', () => {
        test('should perform successful health check', async () => {
            await client.connect();
            const isHealthy = await client.healthCheck();
            expect(isHealthy).toBe(true);
        });

        test('should return false for health check when not connected', async () => {
            // Don't connect the client
            const isHealthy = await client.healthCheck();
            expect(isHealthy).toBe(false);
        });

        test('should measure health check latency', async () => {
            await client.connect();
            
            const start = process.hrtime.bigint();
            await client.healthCheck();
            const end = process.hrtime.bigint();
            
            const latencyMs = Number(end - start) / 1_000_000;
            console.log(`Health check latency: ${latencyMs.toFixed(2)}ms`);
            
            // Health check should be fast (under 5ms requirement)
            expect(latencyMs).toBeLessThan(5);
        });
    });

    describe('Pipeline Operations', () => {
        test('should execute empty pipeline', async () => {
            await client.connect();
            const results = await client.pipeline([]);
            expect(results).toEqual([]);
        });

        test('should execute single command pipeline', async () => {
            await client.connect();
            const results = await client.pipeline(['SET pipeline_single test_value']);
            expect(results).toEqual(['OK']);
        });

        test('should execute multiple commands in pipeline', async () => {
            await client.connect();
            
            const commands = [
                'SET pipeline_key1 value1',
                'SET pipeline_key2 value2',
                'GET pipeline_key1',
                'GET pipeline_key2',
                'DELETE pipeline_key1'
            ];
            
            const results = await client.pipeline(commands);
            expect(results).toEqual([
                'OK',                    // SET pipeline_key1
                'OK',                    // SET pipeline_key2
                'VALUE value1',          // GET pipeline_key1 (server format)
                'VALUE value2',          // GET pipeline_key2 (server format)
                'DELETED'                // DELETE pipeline_key1 (or 'OK')
            ]);
        });

        test('should measure pipeline operation latency', async () => {
            await client.connect();
            
            const commands = [
                'SET perf_key1 value1',
                'SET perf_key2 value2',
                'GET perf_key1',
                'GET perf_key2'
            ];
            
            const start = process.hrtime.bigint();
            await client.pipeline(commands);
            const end = process.hrtime.bigint();
            
            const latencyMs = Number(end - start) / 1_000_000;
            console.log(`Pipeline latency for ${commands.length} commands: ${latencyMs.toFixed(2)}ms`);
            
            // Pipeline should be reasonable (under 50ms for small batches)
            expect(latencyMs).toBeLessThan(50);
        });
    });

    describe('Protocol Compliance', () => {
        test('should handle DELETE command responses (OK or DELETED)', async () => {
            await client.connect();
            
            // Set a key first
            await client.set('delete_test', 'value');
            
            // Delete should succeed regardless of server response format
            const result = await client.delete('delete_test');
            expect(result).toBe(true);
            
            // Verify key is deleted
            const getValue = await client.get('delete_test');
            expect(getValue).toBeNull();
        });

        test('should handle CRLF protocol termination', async () => {
            await client.connect();
            
            // Test that commands with special characters work properly
            const testKey = 'crlf_test_key';
            const testValue = 'value with spaces and\tspecial chars!@#$%';
            
            await client.set(testKey, testValue);
            const result = await client.get(testKey);
            expect(result).toBe(testValue);
        });

        test('should handle empty values correctly', async () => {
            await client.connect();
            
            const testKey = 'empty_value_test';
            await client.set(testKey, '');
            const result = await client.get(testKey);
            expect(result).toBe('');
        });
    });

    describe('Performance Benchmarks', () => {
        test('should measure SET operation latency', async () => {
            await client.connect();
            
            const latencies: number[] = [];
            const iterations = 10;
            
            // Warmup
            for (let i = 0; i < 3; i++) {
                await client.set(`warmup_${i}`, 'value');
            }
            
            // Measure SET latencies
            for (let i = 0; i < iterations; i++) {
                const start = process.hrtime.bigint();
                await client.set(`perf_set_${i}`, `value_${i}`);
                const end = process.hrtime.bigint();
                
                const latencyMs = Number(end - start) / 1_000_000;
                latencies.push(latencyMs);
            }
            
            const avgLatency = latencies.reduce((a, b) => a + b) / latencies.length;
            const p50 = latencies.sort((a, b) => a - b)[Math.floor(latencies.length * 0.5)];
            
            console.log(`SET - Average: ${avgLatency.toFixed(2)}ms, P50: ${p50.toFixed(2)}ms`);
            
            // Should meet performance requirements
            expect(p50).toBeLessThan(5);
        });

        test('should measure GET operation latency', async () => {
            await client.connect();
            
            // Set up test data
            const testKey = 'perf_get_test';
            await client.set(testKey, 'test_value');
            
            const latencies: number[] = [];
            const iterations = 10;
            
            // Warmup
            for (let i = 0; i < 3; i++) {
                await client.get(testKey);
            }
            
            // Measure GET latencies
            for (let i = 0; i < iterations; i++) {
                const start = process.hrtime.bigint();
                await client.get(testKey);
                const end = process.hrtime.bigint();
                
                const latencyMs = Number(end - start) / 1_000_000;
                latencies.push(latencyMs);
            }
            
            const avgLatency = latencies.reduce((a, b) => a + b) / latencies.length;
            const p50 = latencies.sort((a, b) => a - b)[Math.floor(latencies.length * 0.5)];
            
            console.log(`GET - Average: ${avgLatency.toFixed(2)}ms, P50: ${p50.toFixed(2)}ms`);
            
            // Should meet performance requirements
            expect(p50).toBeLessThan(5);
        });

        test('should measure DELETE operation latency', async () => {
            await client.connect();
            
            const latencies: number[] = [];
            const iterations = 10;
            
            // Measure DELETE latencies
            for (let i = 0; i < iterations; i++) {
                // Set key first
                await client.set(`perf_delete_${i}`, 'value');
                
                const start = process.hrtime.bigint();
                await client.delete(`perf_delete_${i}`);
                const end = process.hrtime.bigint();
                
                const latencyMs = Number(end - start) / 1_000_000;
                latencies.push(latencyMs);
            }
            
            const avgLatency = latencies.reduce((a, b) => a + b) / latencies.length;
            const p50 = latencies.sort((a, b) => a - b)[Math.floor(latencies.length * 0.5)];
            
            console.log(`DELETE - Average: ${avgLatency.toFixed(2)}ms, P50: ${p50.toFixed(2)}ms`);
            
            // Should meet performance requirements
            expect(p50).toBeLessThan(5);
        });
    });

    describe('Error Handling', () => {
        test('should handle connection errors gracefully', async () => {
            const invalidClient = new MerkleKVClient('localhost', 9999); // Non-existent port
            
            await expect(invalidClient.connect()).rejects.toThrow(ConnectionError);
        });

        test('should handle timeout errors', async () => {
            const timeoutClient = new MerkleKVClient('localhost', TEST_PORT, 1); // 1ms timeout
            
            await expect(timeoutClient.connect()).rejects.toThrow(TimeoutError);
        });

        test('should handle operations when not connected', async () => {
            // Don't connect the client
            await expect(client.get('test')).rejects.toThrow(ConnectionError);
            await expect(client.set('test', 'value')).rejects.toThrow(ConnectionError);
            await expect(client.delete('test')).rejects.toThrow(ConnectionError);
            await expect(client.pipeline(['GET test'])).rejects.toThrow(ConnectionError);
        });
    });
});
