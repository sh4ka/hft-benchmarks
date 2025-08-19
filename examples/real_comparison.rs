//! Real algorithm comparison examples

use hft_benchmarks::*;

// Two different ways to sum numbers
fn sum_with_loop(n: u32) -> u32 {
    let mut sum = 0;
    for i in 0..n {
        sum += i;
    }
    sum
}

fn sum_with_formula(n: u32) -> u32 {
    n * (n - 1) / 2
}

// Two different ways to check if number is even
fn is_even_modulo(n: u32) -> bool {
    n % 2 == 0
}

fn is_even_bitwise(n: u32) -> bool {
    n & 1 == 0
}

fn main() {
    quick_calibrate_tsc_frequency();
    
    println!("🔬 Real Algorithm Comparisons\n");
    
    // Test 1: Sum algorithms
    println!("=== Sum Algorithms (n=100) ===");
    
    let loop_perf = SimpleBench::new("sum_with_loop")
        .bench(10000, || sum_with_loop(100))
        .analyze();
    
    let formula_perf = SimpleBench::new("sum_with_formula")
        .bench(10000, || sum_with_formula(100))
        .analyze();
    
    println!("Loop method:    mean={}ns, P99={}ns", loop_perf.mean, loop_perf.p99);
    println!("Formula method: mean={}ns, P99={}ns", formula_perf.mean, formula_perf.p99);
    
    if formula_perf.mean < loop_perf.mean {
        let improvement = (loop_perf.mean as f64 / formula_perf.mean as f64 - 1.0) * 100.0;
        println!("✅ Formula is {:.1}% faster", improvement);
    } else if loop_perf.mean < formula_perf.mean {
        let degradation = (formula_perf.mean as f64 / loop_perf.mean as f64 - 1.0) * 100.0;
        println!("⚠️  Loop is {:.1}% faster", degradation);
    } else {
        println!("🤝 Both methods perform similarly");
    }
    
    // Test 2: Even number check
    println!("\n=== Even Number Check ===");
    
    let modulo_perf = SimpleBench::new("is_even_modulo")
        .bench(10000, || is_even_modulo(12345))
        .analyze();
    
    let bitwise_perf = SimpleBench::new("is_even_bitwise")
        .bench(10000, || is_even_bitwise(12345))
        .analyze();
    
    println!("Modulo method:  mean={}ns, P99={}ns", modulo_perf.mean, modulo_perf.p99);
    println!("Bitwise method: mean={}ns, P99={}ns", bitwise_perf.mean, bitwise_perf.p99);
    
    if bitwise_perf.mean < modulo_perf.mean {
        let improvement = (modulo_perf.mean as f64 / bitwise_perf.mean as f64 - 1.0) * 100.0;
        println!("✅ Bitwise is {:.1}% faster", improvement);
    } else if modulo_perf.mean < bitwise_perf.mean {
        let degradation = (bitwise_perf.mean as f64 / modulo_perf.mean as f64 - 1.0) * 100.0;
        println!("⚠️  Modulo is {:.1}% faster", degradation);
    } else {
        println!("🤝 Both methods perform similarly");
    }
    
    // Test 3: Vector vs Array
    println!("\n=== Vector vs Array Access ===");
    
    let vec_data = vec![1, 2, 3, 4, 5];
    let array_data = [1, 2, 3, 4, 5];
    
    let vec_perf = SimpleBench::new("vector_access")
        .bench(10000, || {
            let sum = vec_data.iter().sum::<i32>();
            std::hint::black_box(sum);
        })
        .analyze();
    
    let array_perf = SimpleBench::new("array_access")
        .bench(10000, || {
            let sum = array_data.iter().sum::<i32>();
            std::hint::black_box(sum);
        })
        .analyze();
    
    println!("Vector access: mean={}ns, P99={}ns", vec_perf.mean, vec_perf.p99);
    println!("Array access:  mean={}ns, P99={}ns", array_perf.mean, array_perf.p99);
    
    if array_perf.mean < vec_perf.mean {
        let improvement = (vec_perf.mean as f64 / array_perf.mean as f64 - 1.0) * 100.0;
        println!("✅ Array is {:.1}% faster", improvement);
    } else if vec_perf.mean < array_perf.mean {
        let degradation = (array_perf.mean as f64 / vec_perf.mean as f64 - 1.0) * 100.0;
        println!("⚠️  Vector is {:.1}% faster", degradation);
    } else {
        println!("🤝 Both perform similarly");
    }
    
    println!("\n💡 Note: Results may vary between runs due to:");
    println!("   - CPU cache state");
    println!("   - System load");
    println!("   - Compiler optimizations");
    println!("   - Memory layout");
}