//! Custom benchmark example - add your own functions here

use hft_benchmarks::*;

// Example functions to benchmark
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn fast_multiply(a: u64, b: u64) -> u64 {
    a.wrapping_mul(b)
}

fn string_operations() -> String {
    let mut s = String::new();
    for i in 0..10 {
        s.push_str(&i.to_string());
    }
    s
}

fn main() {
    println!("🎯 Custom Benchmark Examples\n");
    
    quick_calibrate_tsc_frequency();
    
    // Benchmark 1: Simple arithmetic
    println!("=== Arithmetic Operations ===");
    SimpleBench::new("fast_multiply")
        .bench(10000, || fast_multiply(12345, 67890))
        .report();
    
    // Benchmark 2: Recursive function (warning: slow!)
    println!("\n=== Recursive Function ===");
    SimpleBench::new("fibonacci_20")
        .bench(100, || fibonacci(20))  // Only 100 iterations - fibonacci is slow!
        .report();
    
    // Benchmark 3: String operations
    println!("\n=== String Operations ===");
    SimpleBench::new("string_ops")
        .bench(1000, || string_operations())
        .report();
    
    // Benchmark 4: Compare two implementations
    println!("\n=== Implementation Comparison ===");
    let method_a = SimpleBench::new("method_a")
        .bench(5000, || {
            // Method A: Manual loop
            let mut sum = 0u64;
            for i in 0..100 {
                sum += i;
            }
            sum
        })
        .analyze();
    
    let method_b = SimpleBench::new("method_b") 
        .bench(5000, || {
            // Method B: Iterator
            (0..100u64).sum::<u64>()
        })
        .analyze();
    
    println!("Method A (manual loop): {}ns P99", method_a.p99);
    println!("Method B (iterator):    {}ns P99", method_b.p99);
    
    if method_b.p99 < method_a.p99 {
        let improvement = (method_a.p99 as f64 / method_b.p99 as f64 - 1.0) * 100.0;
        println!("Iterator is {:.1}% faster!", improvement);
    } else {
        let degradation = (method_b.p99 as f64 / method_a.p99 as f64 - 1.0) * 100.0;
        println!("Manual loop is {:.1}% faster!", degradation);
    }
    
    // Benchmark 5: Performance validation
    println!("\n=== Performance Validation ===");
    let critical_path = SimpleBench::new("critical_operation")
        .bench(1000, || {
            // Simulate a critical operation
            std::hint::black_box(42 * 37 + 15)
        })
        .analyze();
    
    const TARGET_P99_NS: u64 = 100;
    if critical_path.meets_target(TARGET_P99_NS) {
        println!("✅ Performance target met! P99: {}ns (target: <{}ns)", 
                 critical_path.p99, TARGET_P99_NS);
    } else {
        println!("❌ Performance target missed! P99: {}ns (target: <{}ns)", 
                 critical_path.p99, TARGET_P99_NS);
    }
}