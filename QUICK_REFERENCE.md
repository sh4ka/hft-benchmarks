# 🚀 Quick Benchmark Reference

## Commands (Copy & Paste)

```bash
# Overview of all benchmarks
cargo run --example simple_benchmark_example

# Memory allocation focus
cargo run --example memory_benchmarks

# Real algorithm comparisons
cargo run --example real_comparison

# Custom function examples
cargo run --example custom_benchmark

# Test suite validation
cargo test --package hft-benchmarks -- --nocapture

# Production accuracy (use this for real measurements!)
cargo run --release --example simple_benchmark_example
```

## Basic Benchmark Pattern

```rust
use hft_benchmarks::*;

fn main() {
    quick_calibrate_tsc_frequency();
    
    SimpleBench::new("my_function")
        .bench(10000, || my_function())
        .report();
}
```

## Understanding Results

```
function: 10000 samples, mean=245ns, p50=230ns, p95=310ns, p99=450ns, std_dev=45.2ns
```

- **mean**: Average time
- **p99**: 99% complete faster (most important!)
- **std_dev**: Consistency (lower = better)

## Performance Targets

| Operation | Good P99 | Acceptable P99 |
|-----------|----------|----------------|
| Arithmetic | < 50ns | < 100ns |
| Memory alloc | < 200ns | < 500ns |
| Hash lookup | < 100ns | < 300ns |

## Red Flags 🚨

- Simple operations taking microseconds (use `--release`)
- P99 > 10x mean (investigate outliers)
- Results vary wildly (close other programs)

## Pro Tips ✨

1. **Always use `--release` for production measurements**
2. **Warm up functions before benchmarking**
3. **Use `std::hint::black_box()` to prevent optimization**
4. **Realistic improvements are 10-100%, not 1000%+**