//! Example showing simplified benchmark API usage

use hft_benchmarks::{SimpleBench, quick_calibrate_tsc_frequency};

fn main() {
    quick_calibrate_tsc_frequency();
    
    #[cfg(target_arch = "aarch64")]
    {
        let resolution_ns = 1_000_000_000u64 / 24_000_000u64;
        println!("Note: ARM64 timing resolution is ~{}ns. Very fast operations may show limited precision.\n", resolution_ns);
    }
    
    println!("=== Simple Arithmetic Benchmark ===");
    SimpleBench::new("arithmetic")
        .bench(10000, || {
            let a = 42u64;
            let b = 37u64;
            a.wrapping_add(b).wrapping_mul(2)
        })
        .report();
    
    println!("\n=== Vector Allocation Benchmark ===");
    SimpleBench::new("vec_allocation")
        .bench(1000, || {
            let vec: Vec<u64> = Vec::with_capacity(100);
            drop(vec);
        })
        .report();
    
    println!("\n=== Function Call Overhead ===");
    fn dummy_function(x: u64) -> u64 {
        x.wrapping_add(1)
    }
    
    SimpleBench::new("function_call")
        .bench(5000, || dummy_function(42))
        .report();
    
    println!("\n=== Custom Analysis Example ===");
    let analysis = SimpleBench::new("analysis_example")
        .bench(1000, || std::hint::black_box(42))
        .analyze();
    
    if analysis.meets_target(100) {
        println!("✓ Performance target met! P99: {}ns", analysis.p99);
    } else {
        println!("✗ Performance target missed. P99: {}ns", analysis.p99);
    }
    
    println!("\n=== Implementation Comparison ===");
    
    let old_analysis = SimpleBench::new("old_impl")
        .bench(1000, || {
            for _ in 0..10 {
                std::hint::black_box(42);
            }
        })
        .analyze();
    
    let new_analysis = SimpleBench::new("new_impl")
        .bench(1000, || {
            std::hint::black_box(42);
        })
        .analyze();
    
    println!("Old: {}ns P99, New: {}ns P99", old_analysis.p99, new_analysis.p99);
    
    if new_analysis.mean == 0 {
        println!("Improvement: Operation too fast to measure precisely (< {}ns resolution)", 
                 1_000_000_000u64 / 24_000_000u64);
    } else {
        let improvement = (old_analysis.mean as f64 / new_analysis.mean as f64 - 1.0) * 100.0;
        println!("Improvement: {:.1}% faster", improvement);
    }
}