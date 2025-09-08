use std::time::Instant;
use std::env;
use merklekv_client::{Client, Result};
use serde_json::json;

#[derive(Clone)]
struct BenchmarkConfig {
    host: String,
    port: u16,
    warmup_ops: usize,
    workload_ops: usize,
    pipeline_depths: Vec<usize>,
    keyspace_size: usize,
    value_size: usize,
    get_set_ratio: f64,
}

struct BenchmarkResult {
    operation: String,
    depth: usize,
    p50_latency: f64,
    p95_latency: f64,
    p99_latency: f64,
    throughput: f64,
    errors: usize,
    total_ops: usize,
}

fn main() -> Result<()> {
    env_logger::init();

    let mut config = BenchmarkConfig {
        host: "localhost".to_string(),
        port: 7379,
        warmup_ops: 5000,
        workload_ops: 100000,
        pipeline_depths: vec![1, 8, 32, 128],
        keyspace_size: 10000,
        value_size: 16,
        get_set_ratio: 0.5,
    };

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    for i in 0..args.len() {
        match args.get(i).map(|s| s.as_str()) {
            Some("-host") => {
                if let Some(host) = args.get(i + 1) {
                    config.host = host.clone();
                }
            }
            Some("-port") => {
                if let Some(port) = args.get(i + 1) {
                    config.port = port.parse().unwrap_or(7379);
                }
            }
            Some("-ops") => {
                if let Some(ops) = args.get(i + 1) {
                    config.workload_ops = ops.parse().unwrap_or(100000);
                }
            }
            Some("-warmup") => {
                if let Some(warmup) = args.get(i + 1) {
                    config.warmup_ops = warmup.parse().unwrap_or(5000);
                }
            }
            Some("-depths") => {
                if let Some(depths_str) = args.get(i + 1) {
                    config.pipeline_depths = depths_str
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                }
            }
            _ => {}
        }
    }

    println!("Rust MerkleKV Client Benchmark");
    println!("==============================");
    println!("Host: {}:{}", config.host, config.port);
    println!("Warmup: {} ops", config.warmup_ops);
    println!("Workload: {} ops", config.workload_ops);
    println!("Pipeline depths: {:?}", config.pipeline_depths);
    println!("Keyspace: {} keys", config.keyspace_size);
    println!("Value size: {} bytes", config.value_size);
    println!();

    // Run benchmark
    let results = run_benchmark(config)?;

    // Print results
    println!("Results:");
    println!("========");
    for result in &results {
        println!("Operation: {} (depth={})", result.operation, result.depth);
        println!("  P50: {:.2} ms", result.p50_latency);
        println!("  P95: {:.2} ms", result.p95_latency);
        println!("  P99: {:.2} ms", result.p99_latency);
        println!("  Throughput: {:.1} ops/sec", result.throughput);
        println!("  Errors: {}/{}", result.errors, result.total_ops);
        println!();
    }

    // Output JSON for CI parsing
    let json_results: serde_json::Value = json!(
        results.iter().map(|r| json!({
            "operation": r.operation,
            "depth": r.depth,
            "p50_ms": r.p50_latency,
            "p95_ms": r.p95_latency,
            "p99_ms": r.p99_latency,
            "throughput_ops_sec": r.throughput,
            "errors": r.errors,
            "total_ops": r.total_ops
        })).collect::<Vec<_>>()
    );
    
    println!("JSON Results:");
    println!("{}", serde_json::to_string_pretty(&json_results).unwrap());

    // Check p50 compliance (< 5ms)
    let max_p50 = results.iter()
        .map(|r| r.p50_latency)
        .fold(0.0, f64::max);

    print!("Performance Gate: ");
    if max_p50 < 5.0 {
        println!("✅ PASS (max p50: {:.2} ms)", max_p50);
        std::process::exit(0);
    } else {
        println!("❌ FAIL (max p50: {:.2} ms)", max_p50);
        std::process::exit(1);
    }
}

fn run_benchmark(config: BenchmarkConfig) -> Result<Vec<BenchmarkResult>> {
    let addr = format!("{}:{}", config.host, config.port);
    let mut client = Client::connect(&addr)?;

    // Test connectivity
    client.health_check()?;
    println!("Connected to MerkleKV server");

    // Generate test value
    let test_value = "x".repeat(config.value_size);

    let mut results = Vec::new();

    // Run warmup
    println!("Running warmup ({} ops)...", config.warmup_ops);
    run_warmup(&mut client, &config, &test_value)?;

    // Benchmark each pipeline depth
    for &depth in &config.pipeline_depths {
        println!("Benchmarking pipeline depth {}...", depth);
        
        let result = benchmark_pipeline_depth(&mut client, &config, &test_value, depth)?;
        results.push(result);
    }

    Ok(results)
}

fn run_warmup(client: &mut Client, config: &BenchmarkConfig, test_value: &str) -> Result<()> {
    let mut commands = Vec::new();
    
    for i in 0..config.warmup_ops {
        let key = format!("warmup_key_{}", i % config.keyspace_size);
        
        if (i as f64 / config.warmup_ops as f64) < config.get_set_ratio {
            commands.push(format!("GET {}", key));
        } else {
            commands.push(format!("SET {} {}", key, test_value));
        }
        
        // Execute in batches of 100
        if commands.len() >= 100 || i == config.warmup_ops - 1 {
            let _ = client.pipeline(commands.clone());
            commands.clear();
        }
    }
    
    Ok(())
}

fn benchmark_pipeline_depth(
    client: &mut Client, 
    config: &BenchmarkConfig, 
    test_value: &str, 
    depth: usize
) -> Result<BenchmarkResult> {
    let mut latencies = Vec::new();
    let mut total_ops = 0;
    let mut errors = 0;

    let start = Instant::now();

    while total_ops < config.workload_ops {
        // Prepare batch of commands
        let batch_size = depth.min(config.workload_ops - total_ops);
        let mut commands = Vec::with_capacity(batch_size);
        
        for i in 0..batch_size {
            let key = format!("bench_key_{}", (total_ops + i) % config.keyspace_size);
            
            if (i as f64 / batch_size as f64) < config.get_set_ratio {
                commands.push(format!("GET {}", key));
            } else {
                commands.push(format!("SET {} {}", key, test_value));
            }
        }

        // Measure pipeline operation
        let op_start = Instant::now();
        match client.pipeline(commands) {
            Ok(_) => {
                let op_duration = op_start.elapsed();
                // Use actual total latency for pipeline operations, similar to Go client
                latencies.push(op_duration.as_secs_f64() * 1000.0);
            }
            Err(_) => {
                errors += 1;
            }
        }

        total_ops += batch_size;
    }

    let duration = start.elapsed();
    let throughput = total_ops as f64 / duration.as_secs_f64();

    // Calculate percentiles - convert to per-operation latency for pipeline
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let per_op_latencies: Vec<f64> = latencies.iter()
        .map(|&total_latency| total_latency / depth as f64)
        .collect();
    
    let p50 = if per_op_latencies.is_empty() { 0.0 } else { per_op_latencies[per_op_latencies.len() * 50 / 100] };
    let p95 = if per_op_latencies.is_empty() { 0.0 } else { per_op_latencies[per_op_latencies.len() * 95 / 100] };
    let p99 = if per_op_latencies.is_empty() { 0.0 } else { per_op_latencies[per_op_latencies.len() * 99 / 100] };

    Ok(BenchmarkResult {
        operation: "Pipeline".to_string(),
        depth,
        p50_latency: p50,
        p95_latency: p95,
        p99_latency: p99,
        throughput,
        errors,
        total_ops,
    })
}
