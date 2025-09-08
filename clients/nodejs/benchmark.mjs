#!/usr/bin/env node

/**
 * MerkleKV Node.js Client Benchmark
 * Performance validation script with CI gate enforcement
 */

import pkg from './dist/index.js';
const { MerkleKVClient, MerkleKVError } = pkg;

class Benchmark {
    constructor() {
        this.config = {
            host: 'localhost',
            port: 7681,
            warmupOps: 5000,
            workloadOps: 100000,
            pipelineDepths: [1, 8, 32, 128],
            keyspaceSize: 10000,
            valueSize: 16,
            getSetRatio: 0.5
        };

        this.parseArgs();
    }

    parseArgs() {
        const args = process.argv.slice(2);
        for (let i = 0; i < args.length; i += 2) {
            const key = args[i];
            const value = args[i + 1];

            switch (key) {
                case '--host':
                    this.config.host = value;
                    break;
                case '--port':
                    this.config.port = parseInt(value);
                    break;
                case '--warmup':
                    this.config.warmupOps = parseInt(value);
                    break;
                case '--ops':
                    this.config.workloadOps = parseInt(value);
                    break;
                case '--depths':
                    this.config.pipelineDepths = value.split(',').map(d => parseInt(d.trim()));
                    break;
                case '--keys':
                    this.config.keyspaceSize = parseInt(value);
                    break;
                case '--valuesize':
                    this.config.valueSize = parseInt(value);
                    break;
            }
        }
    }

    async run() {
        console.log('Node.js MerkleKV Client Benchmark');
        console.log('=================================');
        console.log(`Host: ${this.config.host}:${this.config.port}`);
        console.log(`Warmup: ${this.config.warmupOps} ops`);
        console.log(`Workload: ${this.config.workloadOps} ops`);
        console.log(`Pipeline depths: ${this.config.pipelineDepths}`);
        console.log(`Keyspace: ${this.config.keyspaceSize} keys`);
        console.log(`Value size: ${this.config.valueSize} bytes`);
        console.log('');

        const client = new MerkleKVClient(this.config.host, this.config.port);
        
        try {
            await client.connect();
            console.log('Connected to MerkleKV server');

            // Generate test value
            const testValue = 'x'.repeat(this.config.valueSize);

            // Run warmup
            console.log(`Running warmup (${this.config.warmupOps} ops)...`);
            await this.runWarmup(client, testValue);

            // Run benchmarks for each depth
            const results = [];
            for (const depth of this.config.pipelineDepths) {
                console.log(`Benchmarking pipeline depth ${depth}...`);
                const result = await this.benchmarkPipelineDepth(client, testValue, depth);
                results.push(result);
            }

            // Print results
            this.printResults(results);

            // Check performance gate
            this.checkPerformanceGate(results);

        } catch (error) {
            console.error('Benchmark failed:', error);
            process.exit(1);
        } finally {
            await client.disconnect();
        }
    }

    async runWarmup(client, testValue) {
        const batchSize = 100;
        let completed = 0;

        while (completed < this.config.warmupOps) {
            const remaining = this.config.warmupOps - completed;
            const currentBatch = Math.min(batchSize, remaining);
            
            const commands = [];
            for (let i = 0; i < currentBatch; i++) {
                const key = `warmup_key_${Math.floor(Math.random() * this.config.keyspaceSize)}`;
                
                if (Math.random() < this.config.getSetRatio) {
                    commands.push(`GET ${key}`);
                } else {
                    commands.push(`SET ${key} ${testValue}`);
                }
            }

            await client.pipeline(commands);
            completed += currentBatch;
        }
    }

    async benchmarkPipelineDepth(client, testValue, depth) {
        const measurements = [];
        let totalOps = 0;
        let errors = 0;

        const start = Date.now();

        while (totalOps < this.config.workloadOps) {
            const batchSize = Math.min(depth, this.config.workloadOps - totalOps);
            
            const commands = [];
            for (let i = 0; i < batchSize; i++) {
                const key = `bench_key_${Math.floor(Math.random() * this.config.keyspaceSize)}`;
                
                if (Math.random() < this.config.getSetRatio) {
                    commands.push(`GET ${key}`);
                } else {
                    commands.push(`SET ${key} ${testValue}`);
                }
            }

            const opStart = Date.now();
            try {
                await client.pipeline(commands);
                const opDuration = Date.now() - opStart;
                measurements.push({ duration: opDuration, success: true });
            } catch (error) {
                const opDuration = Date.now() - opStart;
                measurements.push({ duration: opDuration, success: false });
                errors++;
            }

            totalOps += batchSize;
        }

        const totalDuration = (Date.now() - start) / 1000; // seconds
        const throughput = totalOps / totalDuration;

        // Calculate latencies (per-operation for depth > 1)
        const latencies = [];
        for (const m of measurements) {
            if (m.success) {
                // For pipeline depth 1, use actual latency. For depth > 1, calculate per-operation
                const perOpLatency = depth === 1 ? m.duration : m.duration / depth;
                latencies.push(perOpLatency);
            }
        }

        latencies.sort((a, b) => a - b);

        const p50 = latencies.length > 0 ? latencies[Math.floor(latencies.length * 0.5)] : 0;
        const p95 = latencies.length > 0 ? latencies[Math.floor(latencies.length * 0.95)] : 0;
        const p99 = latencies.length > 0 ? latencies[Math.floor(latencies.length * 0.99)] : 0;

        return {
            operation: 'Pipeline',
            depth,
            p50_ms: p50,
            p95_ms: p95,
            p99_ms: p99,
            throughput_ops_sec: throughput,
            errors,
            total_ops: totalOps
        };
    }

    printResults(results) {
        console.log('\nResults:');
        console.log('========');
        
        for (const result of results) {
            console.log(`Operation: ${result.operation} (depth=${result.depth})`);
            console.log(`  P50: ${result.p50_ms.toFixed(2)} ms`);
            console.log(`  P95: ${result.p95_ms.toFixed(2)} ms`);
            console.log(`  P99: ${result.p99_ms.toFixed(2)} ms`);
            console.log(`  Throughput: ${result.throughput_ops_sec.toFixed(1)} ops/sec`);
            console.log(`  Errors: ${result.errors}/${result.total_ops}`);
            console.log('');
        }

        // Output JSON for CI parsing
        console.log('JSON Results:');
        console.log(JSON.stringify(results, null, 2));
    }

    checkPerformanceGate(results) {
        const maxP50 = Math.max(...results.map(r => r.p50_ms));
        
        console.log('Performance Gate:');
        if (maxP50 < 5.0) {
            console.log(`✅ PASS (max p50: ${maxP50.toFixed(2)} ms)`);
        } else {
            console.log(`❌ FAIL (max p50: ${maxP50.toFixed(2)} ms)`);
            process.exit(1);
        }
    }
}

// Run benchmark
const benchmark = new Benchmark();
benchmark.run().catch(error => {
    console.error('Benchmark error:', error);
    process.exit(1);
});
