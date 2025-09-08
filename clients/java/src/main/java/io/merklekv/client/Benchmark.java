package io.merklekv.client;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.util.*;
import java.util.concurrent.ThreadLocalRandom;

public class Benchmark {
    private static final String DEFAULT_HOST = "localhost";
    private static final int DEFAULT_PORT = 7379;
    private static final int DEFAULT_WARMUP_OPS = 5000;
    private static final int DEFAULT_WORKLOAD_OPS = 100000;
    private static final int[] DEFAULT_PIPELINE_DEPTHS = {1, 8, 32, 128};
    private static final int DEFAULT_KEYSPACE_SIZE = 10000;
    private static final int DEFAULT_VALUE_SIZE = 16;
    private static final double DEFAULT_GET_SET_RATIO = 0.5;

    public static void main(String[] args) throws Exception {
        BenchmarkConfig config = parseArgs(args);
        
        System.out.println("Java MerkleKV Client Benchmark");
        System.out.println("==============================");
        System.out.println("Host: " + config.host + ":" + config.port);
        System.out.println("Warmup: " + config.warmupOps + " ops");
        System.out.println("Workload: " + config.workloadOps + " ops");
        System.out.println("Pipeline depths: " + Arrays.toString(config.pipelineDepths));
        System.out.println("Keyspace: " + config.keyspaceSize + " keys");
        System.out.println("Value size: " + config.valueSize + " bytes");
        System.out.println();

        List<BenchmarkResult> results = runBenchmark(config);
        
        // Print results
        System.out.println("Results:");
        System.out.println("========");
        for (BenchmarkResult result : results) {
            System.out.printf("Operation: %s (depth=%d)%n", result.operation, result.depth);
            System.out.printf("  P50: %.2f ms%n", result.p50Latency);
            System.out.printf("  P95: %.2f ms%n", result.p95Latency);
            System.out.printf("  P99: %.2f ms%n", result.p99Latency);
            System.out.printf("  Throughput: %.1f ops/sec%n", result.throughput);
            System.out.printf("  Errors: %d/%d%n", result.errors, result.totalOps);
            System.out.println();
        }

        // Output JSON for CI parsing
        ObjectMapper mapper = new ObjectMapper();
        ArrayNode jsonResults = mapper.createArrayNode();
        for (BenchmarkResult result : results) {
            ObjectNode resultNode = mapper.createObjectNode();
            resultNode.put("operation", result.operation);
            resultNode.put("depth", result.depth);
            resultNode.put("p50_ms", result.p50Latency);
            resultNode.put("p95_ms", result.p95Latency);
            resultNode.put("p99_ms", result.p99Latency);
            resultNode.put("throughput_ops_sec", result.throughput);
            resultNode.put("total_ops", result.totalOps);
            resultNode.put("errors", result.errors);
            jsonResults.add(resultNode);
        }
        System.out.println("JSON Results:");
        System.out.println(mapper.writeValueAsString(jsonResults));

        // Check p50 compliance (< 5ms)
        double maxP50 = results.stream().mapToDouble(r -> r.p50Latency).max().orElse(0.0);
        if (maxP50 < 5.0) {
            System.out.println("Performance Gate: ✅ PASS (max p50: " + String.format("%.2f", maxP50) + " ms)");
            System.exit(0);
        } else {
            System.out.println("Performance Gate: ❌ FAIL (max p50: " + String.format("%.2f", maxP50) + " ms)");
            System.exit(1);
        }
    }

    private static BenchmarkConfig parseArgs(String[] args) {
        BenchmarkConfig config = new BenchmarkConfig();
        config.host = DEFAULT_HOST;
        config.port = DEFAULT_PORT;
        config.warmupOps = DEFAULT_WARMUP_OPS;
        config.workloadOps = DEFAULT_WORKLOAD_OPS;
        config.pipelineDepths = DEFAULT_PIPELINE_DEPTHS.clone();
        config.keyspaceSize = DEFAULT_KEYSPACE_SIZE;
        config.valueSize = DEFAULT_VALUE_SIZE;
        config.getSetRatio = DEFAULT_GET_SET_RATIO;

        for (int i = 0; i < args.length - 1; i++) {
            switch (args[i]) {
                case "-host":
                    config.host = args[i + 1];
                    break;
                case "-port":
                    config.port = Integer.parseInt(args[i + 1]);
                    break;
                case "-ops":
                    config.workloadOps = Integer.parseInt(args[i + 1]);
                    break;
                case "-warmup":
                    config.warmupOps = Integer.parseInt(args[i + 1]);
                    break;
                case "-depths":
                    String[] depthStrs = args[i + 1].split(",");
                    config.pipelineDepths = new int[depthStrs.length];
                    for (int j = 0; j < depthStrs.length; j++) {
                        config.pipelineDepths[j] = Integer.parseInt(depthStrs[j].trim());
                    }
                    break;
            }
        }
        return config;
    }

    private static List<BenchmarkResult> runBenchmark(BenchmarkConfig config) throws Exception {
        List<BenchmarkResult> results = new ArrayList<>();

        try (MerkleKVClient client = new MerkleKVClient(config.host, config.port)) {
            System.out.println("Connected to MerkleKV server");

            // Warmup
            System.out.println("Running warmup (" + config.warmupOps + " ops)...");
            warmup(client, config);

            // Test each pipeline depth
            for (int depth : config.pipelineDepths) {
                System.out.println("Benchmarking pipeline depth " + depth + "...");
                BenchmarkResult result = benchmarkPipelineDepth(client, config, depth);
                results.add(result);
            }
        }

        return results;
    }

    private static void warmup(MerkleKVClient client, BenchmarkConfig config) throws Exception {
        String testValue = "x".repeat(config.valueSize);
        List<String> commands = new ArrayList<>();
        
        for (int i = 0; i < config.warmupOps; i++) {
            String key = "warmup_key_" + ThreadLocalRandom.current().nextInt(config.keyspaceSize);
            
            if (ThreadLocalRandom.current().nextDouble() < config.getSetRatio) {
                commands.add("GET " + key);
            } else {
                commands.add("SET " + key + " " + testValue);
            }
            
            // Execute in batches of 100
            if (commands.size() >= 100 || i == config.warmupOps - 1) {
                client.pipeline(commands);
                commands.clear();
            }
        }
    }

    private static BenchmarkResult benchmarkPipelineDepth(MerkleKVClient client, BenchmarkConfig config, int depth) throws Exception {
        List<Double> latencies = new ArrayList<>();
        int totalOps = 0;
        int errors = 0;
        String testValue = "x".repeat(config.valueSize);

        long start = System.nanoTime();

        while (totalOps < config.workloadOps) {
            // Prepare batch of commands
            int batchSize = Math.min(depth, config.workloadOps - totalOps);
            List<String> commands = new ArrayList<>(batchSize);
            
            for (int i = 0; i < batchSize; i++) {
                String key = "bench_key_" + ThreadLocalRandom.current().nextInt(config.keyspaceSize);
                
                if (ThreadLocalRandom.current().nextDouble() < config.getSetRatio) {
                    commands.add("GET " + key);
                } else {
                    commands.add("SET " + key + " " + testValue);
                }
            }

            // Measure pipeline operation
            long opStart = System.nanoTime();
            try {
                client.pipeline(commands);
                long opDuration = System.nanoTime() - opStart;
                
                // Use total latency and divide by depth for per-operation latency
                double totalLatencyMs = opDuration / 1_000_000.0;
                latencies.add(totalLatencyMs);
            } catch (Exception e) {
                errors++;
            }

            totalOps += batchSize;
        }

        long duration = System.nanoTime() - start;
        double throughput = (double) totalOps / (duration / 1_000_000_000.0);

        // Calculate percentiles - convert to per-operation latency
        Collections.sort(latencies);
        List<Double> perOpLatencies = new ArrayList<>();
        for (double totalLatency : latencies) {
            perOpLatencies.add(totalLatency / depth);
        }
        Collections.sort(perOpLatencies);

        double p50 = perOpLatencies.isEmpty() ? 0.0 : perOpLatencies.get(perOpLatencies.size() * 50 / 100);
        double p95 = perOpLatencies.isEmpty() ? 0.0 : perOpLatencies.get(perOpLatencies.size() * 95 / 100);
        double p99 = perOpLatencies.isEmpty() ? 0.0 : perOpLatencies.get(perOpLatencies.size() * 99 / 100);

        return new BenchmarkResult("Pipeline", depth, p50, p95, p99, throughput, totalOps, errors);
    }

    static class BenchmarkConfig {
        String host;
        int port;
        int warmupOps;
        int workloadOps;
        int[] pipelineDepths;
        int keyspaceSize;
        int valueSize;
        double getSetRatio;
    }

    static class BenchmarkResult {
        final String operation;
        final int depth;
        final double p50Latency;
        final double p95Latency;
        final double p99Latency;
        final double throughput;
        final int totalOps;
        final int errors;

        BenchmarkResult(String operation, int depth, double p50Latency, double p95Latency, double p99Latency,
                       double throughput, int totalOps, int errors) {
            this.operation = operation;
            this.depth = depth;
            this.p50Latency = p50Latency;
            this.p95Latency = p95Latency;
            this.p99Latency = p99Latency;
            this.throughput = throughput;
            this.totalOps = totalOps;
            this.errors = errors;
        }
    }
}
