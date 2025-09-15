# HFT Benchmarks

High-precision performance measurement tools for Rust applications requiring nanosecond-level timing accuracy.

[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](https://choosealicense.com/licenses/gpl-3.0/)

## Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
hft-benchmarks = { path = "../path/to/hft-benchmarks" }
```

Simple benchmark:
```rust
use hft_benchmarks::*;

fn main() {
    quick_calibrate_tsc_frequency();
    
    SimpleBench::new("my_function")
        .bench(1000, || my_expensive_function())
        .report();
}
```

Output:
```
my_function: 1000 samples, mean=245ns, p50=230ns, p95=310ns, p99=450ns, p99.9=890ns, std_dev=45.2ns
```

## Usage Examples

### Basic Timing

```rust
use hft_benchmarks::*;

// One-time setup (do this once at program start)
calibrate_tsc_frequency();

// Time a single operation
let (result, elapsed_ns) = time_function(|| {
    expensive_computation()
});
println!("Operation took {}ns", elapsed_ns);
```

### Statistical Analysis

```rust
// Collect multiple measurements for statistical analysis
let mut results = BenchmarkResults::new("algorithm_comparison".to_string());

for _ in 0..1000 {
    let timer = PrecisionTimer::start();
    your_algorithm();
    results.record(timer.stop());
}

let analysis = results.analyze();
println!("{}", analysis.summary());

// Check if performance meets requirements
if analysis.meets_target(100) {  // P99 < 100ns
    println!("✓ Performance target met");
} else {
    println!("✗ Too slow: P99 = {}ns", analysis.p99);
}
```

### Comparing Implementations

```rust
use hft_benchmarks::*;

fn main() {
    quick_calibrate_tsc_frequency();
    
    // Benchmark old implementation
    let old_perf = SimpleBench::new("old_algorithm")
        .bench(5000, || old_implementation())
        .analyze();
    
    // Benchmark new implementation
    let new_perf = SimpleBench::new("new_algorithm")
        .bench(5000, || new_implementation())
        .analyze();
    
    // Calculate improvement
    let speedup = old_perf.mean as f64 / new_perf.mean as f64;
    println!("New implementation is {:.1}x faster", speedup);
    println!("Old: {}ns P99, New: {}ns P99", old_perf.p99, new_perf.p99);
}
```

### Memory Allocation Benchmarks

```rust
use hft_benchmarks::*;

fn main() {
    quick_calibrate_tsc_frequency();
    
    // Run built-in allocation benchmarks
    benchmark_allocations();           // Test different allocation sizes
    benchmark_object_pools();          // Compare pool vs direct allocation
    benchmark_aligned_allocations();   // Test memory alignment impact
}
```

Example output:
```
Benchmarking memory allocations (10000 iterations per size)...
allocation_64B: 10000 samples, mean=89ns, p50=70ns, p95=120ns, p99=180ns
allocation_1024B: 10000 samples, mean=145ns, p50=130ns, p95=200ns, p99=280ns

Pool allocation: pool_allocation: 10000 samples, mean=65ns, p50=60ns, p95=85ns, p99=110ns
Direct allocation: direct_allocation: 10000 samples, mean=140ns, p50=130ns, p95=180ns, p99=220ns
```

## API Reference

### Setup and Calibration

```rust
// Required once at program startup for accurate timing
calibrate_tsc_frequency();        // 1000ms calibration (most accurate)
quick_calibrate_tsc_frequency();  // 100ms calibration (faster, less accurate)
```

### SimpleBench (Recommended)

Fluent API for quick benchmarking:

```rust
use hft_benchmarks::SimpleBench;

SimpleBench::new("operation_name")
    .bench(iterations, || your_function())
    .report();                     // Print results
    
// Or get analysis object
let analysis = SimpleBench::new("operation_name")
    .bench(iterations, || your_function())
    .analyze();
```

### Manual Timing

For custom measurement logic:

```rust
use hft_benchmarks::{PrecisionTimer, time_function};

// Time a single operation
let timer = PrecisionTimer::start();
expensive_operation();
let elapsed_ns = timer.stop();

// Time function with return value
let (result, elapsed_ns) = time_function(|| {
    compute_something()
});
```

### Statistical Analysis

```rust
use hft_benchmarks::BenchmarkResults;

let mut results = BenchmarkResults::new("test_name".to_string());

// Collect measurements
for _ in 0..1000 {
    let elapsed = time_operation();
    results.record(elapsed);
}

// Analyze results
let analysis = results.analyze();
println!("Mean: {}ns, P99: {}ns", analysis.mean, analysis.p99);

// Check performance target
if analysis.meets_target(500) {  // P99 < 500ns
    println!("Performance target met!");
}
```

## Understanding Results

The benchmark results show statistical distribution of timing measurements:

```
function_name: 1000 samples, mean=245ns, p50=230ns, p95=310ns, p99=450ns, p99.9=890ns, std_dev=45.2ns
```

- **mean**: Average execution time
- **p50** (median): 50% of operations complete faster than this
- **p95**: 95% of operations complete faster than this
- **p99**: 99% of operations complete faster than this (critical for tail latency)
- **p99.9**: 99.9% of operations complete faster than this
- **std_dev**: Standard deviation (consistency indicator)

### Why P99 Matters

In performance-critical systems:
- **Mean** can hide outliers that hurt user experience
- **P99** shows worst-case performance for 99% of operations
- **P99.9** reveals extreme outliers that can cause system issues

Example: A function averaging 100ns but with P99 of 10ms will cause problems despite good average performance.

## Running Tests

Run the benchmark test suite:

```bash
# From project root
cd /path/to/hft-framework/Code
cargo test --package hft-benchmarks -- --nocapture

# Or from benchmark crate directory
cd crates/hft-benchmarks
cargo test --lib -- --nocapture
```

Run example benchmarks:
```bash
cargo run --example simple_benchmark_example
```

## Best Practices

### 1. Calibration
Always calibrate before benchmarking:
```rust
// At program start
quick_calibrate_tsc_frequency();  // For development/testing
// OR
calibrate_tsc_frequency();        // For production measurements
```

### 2. Sample Size
Use appropriate sample sizes:
```rust
// Quick development check
SimpleBench::new("dev_test").bench(100, || function()).report();

// Production validation  
SimpleBench::new("prod_test").bench(10000, || function()).report();
```

### 3. Warm-up
Account for JIT compilation and cache warming:
```rust
// Warm up
for _ in 0..1000 { function(); }

// Then benchmark
SimpleBench::new("warmed_up").bench(5000, || function()).report();
```

### 4. System Considerations
- Run on isolated CPU cores for consistent results
- Disable CPU scaling for accurate measurements
- Minimize background processes during benchmarking
- Use release mode builds (`cargo run --release`)

## Common Use Cases

### 1. Development - Quick Performance Check

```rust
use hft_benchmarks::*;

fn main() {
    quick_calibrate_tsc_frequency();
    
    SimpleBench::new("new_feature")
        .bench(1000, || my_new_function())
        .report();
}
```

### 2. Optimization - Algorithm Comparison

```rust
use hft_benchmarks::*;

fn compare_algorithms() {
    quick_calibrate_tsc_frequency();
    
    println!("=== Algorithm Comparison ===");
    
    let results_a = SimpleBench::new("algorithm_a")
        .bench(5000, || algorithm_a())
        .analyze();
        
    let results_b = SimpleBench::new("algorithm_b")
        .bench(5000, || algorithm_b())
        .analyze();
    
    println!("Algorithm A: {}ns P99", results_a.p99);
    println!("Algorithm B: {}ns P99", results_b.p99);
    
    if results_b.p99 < results_a.p99 {
        let improvement = (results_a.p99 as f64 / results_b.p99 as f64 - 1.0) * 100.0;
        println!("Algorithm B is {:.1}% faster (P99)", improvement);
    }
}
```

### 3. Production - Performance Validation

```rust
use hft_benchmarks::*;

fn validate_performance() {
    calibrate_tsc_frequency();  // Full calibration for accuracy
    
    let analysis = SimpleBench::new("critical_path")
        .bench(10000, || critical_trading_function())
        .analyze();
    
    // Ensure P99 latency meets requirements
    const MAX_P99_NS: u64 = 500;
    assert!(
        analysis.meets_target(MAX_P99_NS),
        "Performance regression: P99 = {}ns (max allowed: {}ns)",
        analysis.p99,
        MAX_P99_NS
    );
    
    println!("✓ Performance validation passed");
    println!("  Mean: {}ns, P99: {}ns, P99.9: {}ns", 
             analysis.mean, analysis.p99, analysis.p999);
}
```

### 4. Memory Optimization

```rust
use hft_benchmarks::*;

fn optimize_memory_usage() {
    quick_calibrate_tsc_frequency();
    
    println!("=== Memory Allocation Comparison ===");
    
    // Test stack allocation
    SimpleBench::new("stack_alloc")
        .bench(10000, || {
            let data = [0u64; 64];  // Stack allocated
            std::hint::black_box(data);
        })
        .report();
    
    // Test heap allocation  
    SimpleBench::new("heap_alloc")
        .bench(10000, || {
            let data = vec![0u64; 64];  // Heap allocated
            std::hint::black_box(data);
        })
        .report();
    
    // Use built-in memory benchmarks
    benchmark_object_pools();
}

## Running Complete Benchmark Suite

### Memory Allocation Analysis

```bash
cargo run --example simple_benchmark_example
```
**Output:**
```
=== Vector Allocation Benchmark ===
vec_allocation: 1000 samples, mean=185ns, p50=170ns, p95=220ns, p99=992ns

=== Implementation Comparison ===
Old: 90ns P99, New: 50ns P99  
Improvement: 166.7% faster
```

### Custom Benchmarks

```rust
use hft_benchmarks::*;

fn main() {
    calibrate_tsc_frequency();
    
    // Benchmark your trading algorithm
    SimpleBench::new("order_processing")
        .bench(10000, || process_market_order())
        .report();
        
    // Memory-intensive operations
    benchmark_allocations();
    benchmark_object_pools();
}
```

## Technical Details

### Precision and Accuracy

This library uses CPU timestamp counters (TSC) for nanosecond-precision timing:

- **TSC-based timing**: Direct CPU cycle counting via `_rdtsc()` instruction
- **Memory barriers**: Prevents instruction reordering that could affect measurements
- **Calibrated conversion**: Converts CPU cycles to nanoseconds based on measured frequency
- **Minimal overhead**: ~35ns measurement overhead

### Measurement Overhead

The benchmark tools themselves have minimal impact:
```
PrecisionTimer overhead: ~35ns
Function call overhead: ~37ns
Statistical calculation: <1μs for 10k samples
Memory allocation test: ~100-500ns per iteration
```

### System Requirements

- **x86_64/ARM CPU** with stable TSC (most modern processors), on aarch64 tsc will not be available
- **Linux, macOS, or Windows**
- **Rust 1.70+**

### Limitations

- **CPU frequency scaling** can affect accuracy (disable for best results)
- **System load** impacts measurement consistency  
- **Compiler optimizations** may eliminate benchmarked code (use `std::hint::black_box`)
- **First run variance** due to cache warming and JIT compilation

### Integration with Other Tools

Use alongside other profiling tools for comprehensive analysis:
- **perf** for hardware counter analysis
- **valgrind** for memory profiling
- **flamegraph** for call stack visualization  
- **criterion** for statistical benchmarking

This library excels at **microbenchmarks** and **latency-critical code paths** where nanosecond precision matters.
