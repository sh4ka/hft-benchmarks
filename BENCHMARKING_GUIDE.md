# 📊 Comprehensive Benchmarking Guide

A complete guide to running and understanding benchmarks in the HFT Rust framework.

## 📚 Table of Contents

1. [Quick Start](#quick-start)
2. [Available Benchmark Applications](#available-benchmark-applications)
3. [Understanding Benchmark Results](#understanding-benchmark-results)
4. [Best Practices](#best-practices)
5. [Creating Custom Benchmarks](#creating-custom-benchmarks)
6. [Troubleshooting](#troubleshooting)
7. [Performance Analysis](#performance-analysis)

## 🚀 Quick Start

### Prerequisites
```bash
# Ensure you're in the correct directory
cd /path/to/hft-framework/Code

# Verify the project builds
cargo build
```

### Run Your First Benchmark
```bash
cargo run --example simple_benchmark_example
```

**Expected output:**
```
=== Simple Arithmetic Benchmark ===
arithmetic: 10000 samples, mean=35ns, p50=30ns, p95=40ns, p99=40ns, p99.9=40ns, std_dev=8.8ns
```

## 📋 Available Benchmark Applications

### 1. Complete Example Suite
```bash
cargo run --example simple_benchmark_example
```

**What it does:**
- Arithmetic operations benchmark (10,000 samples)
- Vector allocation benchmark (1,000 samples)  
- Function call overhead measurement (5,000 samples)
- Performance target validation example
- Simulated algorithm comparison (educational)

**Best for:** First-time users and getting an overview

**Runtime:** ~3 seconds

---

### 2. Memory-Focused Benchmarks
```bash
cargo run --example memory_benchmarks
```

**What it does:**
- Tests allocation sizes from 64B to 4KB (10,000 iterations each)
- Compares object pool vs direct allocation
- Measures aligned vs unaligned memory allocation impact

**Sample output:**
```
allocation_64B: 10000 samples, mean=207ns, p50=200ns, p95=250ns, p99=270ns
Pool allocation: mean=58ns, p99=70ns
Direct allocation: mean=98ns, p99=130ns
```

**Best for:** Memory optimization and allocation strategy decisions

**Runtime:** ~5 seconds

---

### 3. Real Algorithm Comparisons
```bash
cargo run --example real_comparison
```

**What it does:**
- Sum algorithms: Loop vs mathematical formula
- Even number check: Modulo vs bitwise operations
- Data access: Vector vs array iteration

**Sample output:**
```
=== Sum Algorithms (n=100) ===
Loop method:    mean=992ns, P99=992ns
Formula method: mean=35ns, P99=40ns
✅ Formula is 2734.3% faster
```

**Best for:** Understanding realistic performance differences between algorithms

**Runtime:** ~2 seconds

---

### 4. Custom Function Examples
```bash
cargo run --example custom_benchmark
```

**What it does:**
- Demonstrates benchmarking custom functions
- Shows fibonacci recursion (slow example)
- String operation benchmarks
- Implementation comparison patterns
- Performance validation examples

**Best for:** Learning how to benchmark your own functions

**Runtime:** ~10 seconds (fibonacci is intentionally slow)

---

### 5. Built-in Test Suite
```bash
cargo test --package hft-benchmarks -- --nocapture
```

**What it does:**
- Runs 18 unit tests with benchmark output
- Validates benchmark tool functionality
- Shows micro-benchmarks during testing

**Best for:** Validation and seeing benchmarks in action

**Runtime:** ~1 second

## 📈 Understanding Benchmark Results

### Result Format Breakdown
```
function_name: 1000 samples, mean=245ns, p50=230ns, p95=310ns, p99=450ns, p99.9=890ns, std_dev=45.2ns
```

| Metric | Meaning | Why It Matters |
|--------|---------|----------------|
| **samples** | Number of measurements taken | More samples = more reliable statistics |
| **mean** | Average execution time | General performance indicator |
| **p50** (median) | 50% of operations complete faster | Typical performance |
| **p95** | 95% of operations complete faster | Good performance threshold |
| **p99** | 99% of operations complete faster | **Critical for tail latency** |
| **p99.9** | 99.9% of operations complete faster | Extreme outlier detection |
| **std_dev** | Standard deviation | Consistency indicator (lower = more predictable) |

### Why P99 is Critical

**P99 latency** represents the worst-case performance for 99% of your operations:

```
Example: Function with mean=100ns, P99=10ms
- Average case: 100ns (looks good!)
- But 1% of calls take 10ms (100x slower!)
- In HFT: Missing 1% of trading opportunities
```

**Good vs Bad P99 patterns:**

```bash
# Good: Consistent performance
mean=100ns, p99=120ns, std_dev=15ns

# Bad: Unpredictable outliers  
mean=100ns, p99=5000ns, std_dev=500ns
```

### Performance Improvement Calculations

**Improvement formula:**
```
Improvement % = (old_time / new_time - 1.0) × 100%
```

**Examples:**
```
Old: 200ns, New: 100ns → (200/100 - 1) × 100% = 100% faster
Old: 1000ns, New: 100ns → (1000/100 - 1) × 100% = 900% faster  
Old: 100ns, New: 90ns → (100/90 - 1) × 100% = 11.1% faster
```

**Realistic improvement expectations:**
- **10-50%**: Typical algorithm optimizations
- **100-300%**: Better algorithm choice
- **1000%+**: Usually artificial examples or fundamentally different approaches

## 🎯 Best Practices

### 1. Calibration (Essential)
**Always calibrate before benchmarking:**

```rust
use hft_benchmarks::*;

fn main() {
    // For development/testing (fast)
    quick_calibrate_tsc_frequency();
    
    // For production measurements (accurate)  
    calibrate_tsc_frequency();
    
    // Then run benchmarks...
}
```

### 2. Sample Size Guidelines

| Use Case | Sample Size | Example |
|----------|-------------|---------|
| Quick development check | 100-1,000 | `bench(1000, || function())` |
| Algorithm comparison | 5,000-10,000 | `bench(10000, || function())` |
| Production validation | 10,000+ | `bench(50000, || function())` |
| Micro-operations | 100,000+ | `bench(100000, || simple_math())` |

### 3. Warm-up (Important for Accurate Results)

```rust
// Warm up the function (JIT compilation, cache loading)
for _ in 0..1000 {
    function_to_benchmark();
}

// Then benchmark
SimpleBench::new("warmed_up_function")
    .bench(10000, || function_to_benchmark())
    .report();
```

### 4. Preventing Compiler Optimizations

Use `std::hint::black_box` to prevent the compiler from optimizing away your code:

```rust
// Without black_box - compiler might optimize away
SimpleBench::new("bad_example")
    .bench(1000, || {
        let result = expensive_computation();
        // Compiler thinks: "result unused, skip computation!"
    })
    .report();

// With black_box - forces computation
SimpleBench::new("good_example")
    .bench(1000, || {
        let result = expensive_computation();
        std::hint::black_box(result); // Prevents optimization
    })
    .report();
```

### 5. Release Mode for Production

**For accurate production measurements:**
```bash
# Development (faster compilation, less optimization)
cargo run --example simple_benchmark_example

# Production (slower compilation, full optimization)
cargo run --release --bin simple_benchmark_example
```

**Typical performance difference:**
- Debug mode: Functions may be 2-10x slower
- Release mode: Full compiler optimizations enabled

### 6. System Considerations

**For most accurate results:**

```bash
# Disable CPU frequency scaling (Linux)
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Minimize background processes
# Close browsers, IDEs, etc.

# Run on dedicated cores (advanced)
taskset -c 0 cargo run --release --bin simple_benchmark_example
```

## 🛠️ Creating Custom Benchmarks

### Basic Pattern

```rust
use hft_benchmarks::*;

fn my_function(input: u32) -> u32 {
    // Your function here
    input * 2 + 1
}

fn main() {
    quick_calibrate_tsc_frequency();
    
    SimpleBench::new("my_function")
        .bench(10000, || my_function(42))
        .report();
}
```

### Comparing Implementations

```rust
use hft_benchmarks::*;

fn old_implementation(data: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..data.len() {
        sum += data[i];
    }
    sum
}

fn new_implementation(data: &[i32]) -> i32 {
    data.iter().sum()
}

fn main() {
    quick_calibrate_tsc_frequency();
    
    let test_data = vec![1, 2, 3, 4, 5];
    
    let old_perf = SimpleBench::new("old_implementation")
        .bench(10000, || old_implementation(&test_data))
        .analyze();
    
    let new_perf = SimpleBench::new("new_implementation") 
        .bench(10000, || new_implementation(&test_data))
        .analyze();
    
    println!("Old: {}ns P99", old_perf.p99);
    println!("New: {}ns P99", new_perf.p99);
    
    if new_perf.p99 < old_perf.p99 {
        let improvement = (old_perf.p99 as f64 / new_perf.p99 as f64 - 1.0) * 100.0;
        println!("New is {:.1}% faster", improvement);
    }
}
```

### Advanced Statistical Analysis

```rust
use hft_benchmarks::*;

fn detailed_analysis() {
    quick_calibrate_tsc_frequency();
    
    let analysis = SimpleBench::new("detailed_function")
        .bench(10000, || expensive_function())
        .analyze();
    
    println!("=== Detailed Performance Analysis ===");
    println!("Samples: {}", analysis.count);
    println!("Min: {}ns", analysis.min);
    println!("Max: {}ns", analysis.max);
    println!("Mean: {}ns", analysis.mean);
    println!("Median (P50): {}ns", analysis.p50);
    println!("P95: {}ns", analysis.p95);
    println!("P99: {}ns", analysis.p99);
    println!("P99.9: {}ns", analysis.p999);
    println!("Standard Deviation: {:.1}ns", analysis.std_dev);
    
    // Performance validation
    const MAX_P99_NS: u64 = 1000;
    if analysis.meets_target(MAX_P99_NS) {
        println!("✅ Meets performance requirement (P99 < {}ns)", MAX_P99_NS);
    } else {
        println!("❌ Fails performance requirement (P99 = {}ns)", analysis.p99);
    }
    
    // Consistency check
    let cv = analysis.std_dev / analysis.mean as f64; // Coefficient of variation
    if cv < 0.1 {
        println!("✅ Consistent performance (CV = {:.2})", cv);
    } else {
        println!("⚠️ Inconsistent performance (CV = {:.2})", cv);
    }
}
```

## 🔧 Troubleshooting

### Common Issues

#### 1. Unrealistic Results
```
Problem: Function shows 0ns execution time
Cause: Compiler optimized away the function call
Solution: Use std::hint::black_box()
```

#### 2. Inconsistent Results
```
Problem: Results vary wildly between runs
Causes: 
- System load (close other programs)
- CPU frequency scaling (disable)
- Insufficient warm-up (add warm-up loop)
- Too few samples (increase sample count)
```

#### 3. Very High Latency
```
Problem: Simple operations showing microseconds instead of nanoseconds
Causes:
- Debug mode (use --release)
- System contention (run on isolated system)
- Memory allocation in tight loop (pre-allocate)
```

### Debug Mode vs Release Mode

```bash
# Debug results
cargo run --example simple_benchmark_example
# arithmetic: mean=150ns, p99=200ns

# Release results  
cargo run --release --bin simple_benchmark_example
# arithmetic: mean=35ns, p99=40ns
```

**Always use `--release` for production measurements!**

### Verification Commands

```bash
# Check if benchmarks compile
cargo check --bin simple_benchmark_example

# Run a quick test
cargo run --example simple_benchmark_example

# Verify test suite passes
cargo test --package hft-benchmarks

# Check for warnings
cargo clippy --bin simple_benchmark_example
```

## 📊 Performance Analysis

### Interpreting Different Performance Patterns

#### 1. Consistent Performance (Good)
```
mean=100ns, p50=98ns, p95=105ns, p99=110ns, std_dev=8ns
```
- **Pattern**: Tight distribution, small std_dev
- **Meaning**: Predictable, reliable performance
- **Good for**: Real-time systems, latency-critical code

#### 2. Bimodal Distribution (Investigate)
```
mean=150ns, p50=100ns, p95=300ns, p99=350ns, std_dev=75ns
```
- **Pattern**: Large gap between median and P95
- **Possible causes**: CPU cache misses, garbage collection, system interrupts
- **Action**: Profile to identify cause

#### 3. Long Tail (Bad for HFT)
```
mean=100ns, p50=95ns, p95=120ns, p99=2000ns, std_dev=200ns
```
- **Pattern**: Good average but terrible outliers
- **Meaning**: Rare but extreme slowdowns
- **Action**: Find and eliminate outlier causes

### Performance Targets by Operation Type

| Operation Type | Good P99 Target | Acceptable P99 | Notes |
|----------------|-----------------|----------------|-------|
| **Arithmetic** | < 50ns | < 100ns | Simple CPU operations |
| **Memory allocation** | < 200ns | < 500ns | Heap allocation overhead |
| **Hash lookup** | < 100ns | < 300ns | Hash table operations |
| **String operations** | < 1μs | < 5μs | Depends on string length |
| **File I/O** | < 1ms | < 10ms | Depends on storage type |
| **Network call** | < 100ms | < 1s | Depends on network |

### When to Be Concerned

**🚨 Red flags:**
- P99 > 10x mean (high variability)
- std_dev > 50% of mean (inconsistent)
- Performance degrades over time (memory leaks)
- Simple operations taking microseconds (debug mode)

**✅ Good signs:**
- P99 < 2x mean (consistent)
- std_dev < 20% of mean (predictable)
- Performance scales linearly with input size
- Results repeatable across runs

## 🎯 Summary

### Quick Command Reference

```bash
# First-time users
cargo run --example simple_benchmark_example

# Memory optimization
cargo run --example memory_benchmarks

# Algorithm comparison
cargo run --example real_comparison

# Custom benchmarking
cargo run --example custom_benchmark

# Validation
cargo test --package hft-benchmarks -- --nocapture

# Production measurements (always use --release!)
cargo run --release --bin simple_benchmark_example
```

### Key Takeaways

1. **Always calibrate** before benchmarking
2. **Use adequate sample sizes** (1000+ for most cases)
3. **Warm up functions** before measuring
4. **Use `std::hint::black_box`** to prevent optimization
5. **Focus on P99 latency** for critical systems
6. **Run in release mode** for production measurements
7. **Expect modest improvements** (10-100%) for most optimizations
8. **Large improvements** (1000%+) usually indicate algorithmic changes

### Next Steps

1. **Run the example benchmarks** to understand your system's baseline performance
2. **Benchmark your own functions** using the patterns in `custom_benchmark.rs`
3. **Set up performance tests** in your CI/CD pipeline
4. **Monitor performance over time** to catch regressions early

Happy benchmarking! 🚀