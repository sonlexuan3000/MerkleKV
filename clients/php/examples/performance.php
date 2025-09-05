<?php

declare(strict_types=1);

require_once __DIR__ . '/../vendor/autoload.php';

use MerkleKV\Client;

echo "MerkleKV PHP Client - Performance Test\n";
echo "=====================================\n\n";

function measurePerformance(string $operation, callable $fn): float {
    $start = microtime(true);
    $fn();
    $end = microtime(true);
    return ($end - $start) * 1000; // Convert to milliseconds
}

try {
    $client = new Client("127.0.0.1", 7379, 5.0);
    
    $iterations = 1000;
    $key = "perf_test";
    $value = "performance_test_value_" . str_repeat("x", 100);
    
    echo "Running performance test with {$iterations} iterations...\n\n";
    
    // Warm up
    echo "Warming up connection...\n";
    $client->set($key, $value);
    $client->get($key);
    
    // Test SET performance
    echo "Testing SET performance...\n";
    $setTimes = [];
    for ($i = 0; $i < $iterations; $i++) {
        $time = measurePerformance("SET", function() use ($client, $key, $value, $i) {
            $client->set($key . "_" . $i, $value);
        });
        $setTimes[] = $time;
    }
    
    // Test GET performance
    echo "Testing GET performance...\n";
    $getTimes = [];
    for ($i = 0; $i < $iterations; $i++) {
        $time = measurePerformance("GET", function() use ($client, $key, $i) {
            $client->get($key . "_" . $i);
        });
        $getTimes[] = $time;
    }
    
    // Test DELETE performance
    echo "Testing DELETE performance...\n";
    $deleteTimes = [];
    for ($i = 0; $i < $iterations; $i++) {
        $time = measurePerformance("DELETE", function() use ($client, $key, $i) {
            $client->delete($key . "_" . $i);
        });
        $deleteTimes[] = $time;
    }
    
    // Calculate statistics
    function calculateStats(array $times): array {
        sort($times);
        $count = count($times);
        return [
            'min' => min($times),
            'max' => max($times),
            'avg' => array_sum($times) / $count,
            'p50' => $times[intval($count * 0.5)],
            'p95' => $times[intval($count * 0.95)],
            'p99' => $times[intval($count * 0.99)],
        ];
    }
    
    $setStats = calculateStats($setTimes);
    $getStats = calculateStats($getTimes);
    $deleteStats = calculateStats($deleteTimes);
    
    // Display results
    echo "\n" . str_repeat("=", 60) . "\n";
    echo "PERFORMANCE RESULTS ({$iterations} iterations)\n";
    echo str_repeat("=", 60) . "\n";
    
    printf("%-10s %8s %8s %8s %8s %8s %8s\n", 
           "Operation", "Min", "Max", "Avg", "P50", "P95", "P99");
    echo str_repeat("-", 60) . "\n";
    
    printf("%-10s %7.2f %7.2f %7.2f %7.2f %7.2f %7.2f\n",
           "SET (ms)", $setStats['min'], $setStats['max'], $setStats['avg'],
           $setStats['p50'], $setStats['p95'], $setStats['p99']);
    
    printf("%-10s %7.2f %7.2f %7.2f %7.2f %7.2f %7.2f\n",
           "GET (ms)", $getStats['min'], $getStats['max'], $getStats['avg'],
           $getStats['p50'], $getStats['p95'], $getStats['p99']);
    
    printf("%-10s %7.2f %7.2f %7.2f %7.2f %7.2f %7.2f\n",
           "DELETE (ms)", $deleteStats['min'], $deleteStats['max'], $deleteStats['avg'],
           $deleteStats['p50'], $deleteStats['p95'], $deleteStats['p99']);
    
    echo str_repeat("-", 60) . "\n";
    
    // Calculate throughput
    $totalTime = array_sum($setTimes) + array_sum($getTimes) + array_sum($deleteTimes);
    $totalOps = $iterations * 3;
    $throughput = ($totalOps / ($totalTime / 1000)); // ops/second
    
    printf("Total operations: %d\n", $totalOps);
    printf("Total time: %.2f ms\n", $totalTime);
    printf("Throughput: %.0f ops/sec\n", $throughput);
    
    // Performance targets check
    echo "\n" . str_repeat("=", 60) . "\n";
    echo "PERFORMANCE TARGET CHECK (<5ms per operation)\n";
    echo str_repeat("=", 60) . "\n";
    
    $targets = [
        'SET avg' => $setStats['avg'],
        'GET avg' => $getStats['avg'],
        'DELETE avg' => $deleteStats['avg'],
        'SET P95' => $setStats['p95'],
        'GET P95' => $getStats['p95'],
        'DELETE P95' => $deleteStats['p95'],
    ];
    
    foreach ($targets as $metric => $value) {
        $status = $value < 5.0 ? "✅ PASS" : "❌ FAIL";
        printf("%-12s: %6.2f ms - %s\n", $metric, $value, $status);
    }
    
    $client->close();
    echo "\n🎯 Performance test completed!\n";

} catch (Exception $e) {
    echo "\n❌ Error: " . $e->getMessage() . "\n";
    echo "Make sure MerkleKV server is running on 127.0.0.1:7379\n";
    exit(1);
}
