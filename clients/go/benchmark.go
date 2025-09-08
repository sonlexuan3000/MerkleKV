package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"math/rand"
	"os"
	"sort"
	"strconv"
	"strings"
	"time"

	merklekv "github.com/AI-Decenter/MerkleKV/clients/go"
)

type BenchmarkConfig struct {
	Host         string
	Port         int
	WarmupOps    int
	WorkloadOps  int
	PipelineDepths []int
	KeyspaceSize int
	ValueSize    int
	GetSetRatio  float64
}

type BenchmarkResult struct {
	Operation    string  `json:"operation"`
	Depth        int     `json:"depth"`
	P50Latency   float64 `json:"p50_ms"`
	P95Latency   float64 `json:"p95_ms"`
	P99Latency   float64 `json:"p99_ms"`
	Throughput   float64 `json:"throughput_ops_sec"`
	Errors       int     `json:"errors"`
	TotalOps     int     `json:"total_ops"`
}

type LatencyMeasurement struct {
	duration time.Duration
	success  bool
}

func main() {
	config := BenchmarkConfig{
		Host:         "localhost",
		Port:         7379,
		WarmupOps:    5000,
		WorkloadOps:  100000,
		PipelineDepths: []int{1, 8, 32, 128},
		KeyspaceSize: 10000,
		ValueSize:    16,
		GetSetRatio:  0.5,
	}

	// Parse command line flags
	flag.StringVar(&config.Host, "host", config.Host, "Server host")
	flag.IntVar(&config.Port, "port", config.Port, "Server port")
	flag.IntVar(&config.WarmupOps, "warmup", config.WarmupOps, "Warmup operations")
	flag.IntVar(&config.WorkloadOps, "ops", config.WorkloadOps, "Workload operations")
	flag.IntVar(&config.KeyspaceSize, "keys", config.KeyspaceSize, "Keyspace size")
	flag.IntVar(&config.ValueSize, "valuesize", config.ValueSize, "Value size in bytes")

	var depthsStr string
	flag.StringVar(&depthsStr, "depths", "1,8,32,128", "Pipeline depths (comma-separated)")
	flag.Parse()

	// Parse pipeline depths
	if depthsStr != "" {
		depths := []int{}
		for _, d := range strings.Split(depthsStr, ",") {
			depth, err := strconv.Atoi(strings.TrimSpace(d))
			if err != nil {
				log.Fatalf("Invalid depth: %s", d)
			}
			depths = append(depths, depth)
		}
		config.PipelineDepths = depths
	}

	fmt.Printf("Go MerkleKV Client Benchmark\n")
	fmt.Printf("============================\n")
	fmt.Printf("Host: %s:%d\n", config.Host, config.Port)
	fmt.Printf("Warmup: %d ops\n", config.WarmupOps)
	fmt.Printf("Workload: %d ops\n", config.WorkloadOps)
	fmt.Printf("Pipeline depths: %v\n", config.PipelineDepths)
	fmt.Printf("Keyspace: %d keys\n", config.KeyspaceSize)
	fmt.Printf("Value size: %d bytes\n", config.ValueSize)
	fmt.Printf("\n")

	// Run benchmark
	results, err := runBenchmark(config)
	if err != nil {
		log.Fatalf("Benchmark failed: %v", err)
	}

	// Print results
	fmt.Printf("Results:\n")
	fmt.Printf("========\n")
	for _, result := range results {
		fmt.Printf("Operation: %s (depth=%d)\n", result.Operation, result.Depth)
		fmt.Printf("  P50: %.2f ms\n", result.P50Latency)
		fmt.Printf("  P95: %.2f ms\n", result.P95Latency)
		fmt.Printf("  P99: %.2f ms\n", result.P99Latency)
		fmt.Printf("  Throughput: %.1f ops/sec\n", result.Throughput)
		fmt.Printf("  Errors: %d/%d\n", result.Errors, result.TotalOps)
		fmt.Printf("\n")
	}

	// Output JSON for CI parsing
	jsonOutput, err := json.MarshalIndent(results, "", "  ")
	if err != nil {
		log.Fatalf("Failed to marshal results: %v", err)
	}
	fmt.Printf("JSON Results:\n%s\n", jsonOutput)

	// Check p50 compliance (< 5ms)
	maxP50 := 0.0
	for _, result := range results {
		if result.P50Latency > maxP50 {
			maxP50 = result.P50Latency
		}
	}

	fmt.Printf("Performance Gate: ")
	if maxP50 < 5.0 {
		fmt.Printf("✅ PASS (max p50: %.2f ms)\n", maxP50)
	} else {
		fmt.Printf("❌ FAIL (max p50: %.2f ms)\n", maxP50)
		os.Exit(1)
	}
}

func runBenchmark(config BenchmarkConfig) ([]BenchmarkResult, error) {
	client := merklekv.NewWithTimeout(config.Host, config.Port, 10*time.Second)
	
	err := client.Connect()
	if err != nil {
		return nil, fmt.Errorf("failed to connect: %v", err)
	}
	defer client.Close()

	// Test connectivity
	err = client.Ping()
	if err != nil {
		return nil, fmt.Errorf("server ping failed: %v", err)
	}

	// Generate test value
	testValue := strings.Repeat("x", config.ValueSize)

	var results []BenchmarkResult

	// Run warmup
	fmt.Printf("Running warmup (%d ops)...\n", config.WarmupOps)
	err = runWarmup(client, config, testValue)
	if err != nil {
		return nil, fmt.Errorf("warmup failed: %v", err)
	}

	// Benchmark each pipeline depth
	for _, depth := range config.PipelineDepths {
		fmt.Printf("Benchmarking pipeline depth %d...\n", depth)
		
		result, err := benchmarkPipelineDepth(client, config, testValue, depth)
		if err != nil {
			return nil, fmt.Errorf("benchmark depth %d failed: %v", depth, err)
		}
		
		results = append(results, result)
	}

	return results, nil
}

func runWarmup(client *merklekv.Client, config BenchmarkConfig, testValue string) error {
	commands := make([]string, 0, 100)
	
	for i := 0; i < config.WarmupOps; i++ {
		key := fmt.Sprintf("warmup_key_%d", rand.Intn(config.KeyspaceSize))
		
		if rand.Float64() < config.GetSetRatio {
			commands = append(commands, fmt.Sprintf("GET %s", key))
		} else {
			commands = append(commands, fmt.Sprintf("SET %s %s", key, testValue))
		}
		
		// Execute in batches of 100
		if len(commands) >= 100 || i == config.WarmupOps-1 {
			_, err := client.Pipeline(commands)
			if err != nil {
				return err
			}
			commands = commands[:0] // Reset slice
		}
	}
	
	return nil
}

func benchmarkPipelineDepth(client *merklekv.Client, config BenchmarkConfig, testValue string, depth int) (BenchmarkResult, error) {
	measurements := make([]LatencyMeasurement, 0, config.WorkloadOps/depth)
	totalOps := 0
	errors := 0

	start := time.Now()

	for totalOps < config.WorkloadOps {
		// Prepare batch of commands
		batchSize := depth
		if totalOps+batchSize > config.WorkloadOps {
			batchSize = config.WorkloadOps - totalOps
		}

		commands := make([]string, batchSize)
		for i := 0; i < batchSize; i++ {
			key := fmt.Sprintf("bench_key_%d", rand.Intn(config.KeyspaceSize))
			
			if rand.Float64() < config.GetSetRatio {
				commands[i] = fmt.Sprintf("GET %s", key)
			} else {
				commands[i] = fmt.Sprintf("SET %s %s", key, testValue)
			}
		}

		// Measure pipeline operation
		opStart := time.Now()
		_, err := client.Pipeline(commands)
		opDuration := time.Since(opStart)

		measurements = append(measurements, LatencyMeasurement{
			duration: opDuration,
			success:  err == nil,
		})

		if err != nil {
			errors++
		}

		totalOps += batchSize
	}

	duration := time.Since(start)
	throughput := float64(totalOps) / duration.Seconds()

	// Calculate latencies
	latencies := make([]float64, 0, len(measurements))
	for _, m := range measurements {
		if m.success {
			// Convert to per-operation latency for pipeline
			perOpLatency := float64(m.duration.Nanoseconds()) / float64(depth) / 1e6 // ms
			latencies = append(latencies, perOpLatency)
		}
	}

	sort.Float64s(latencies)

	var p50, p95, p99 float64
	if len(latencies) > 0 {
		p50 = latencies[len(latencies)*50/100]
		p95 = latencies[len(latencies)*95/100]
		p99 = latencies[len(latencies)*99/100]
	}

	return BenchmarkResult{
		Operation:  fmt.Sprintf("Pipeline"),
		Depth:      depth,
		P50Latency: p50,
		P95Latency: p95,
		P99Latency: p99,
		Throughput: throughput,
		Errors:     errors,
		TotalOps:   totalOps,
	}, nil
}
