//! Memory allocation benchmarks

use hft_benchmarks::*;

fn main() {
    println!("🚀 Running Memory Allocation Benchmarks...\n");
    
    // Quick calibration for development
    quick_calibrate_tsc_frequency();
    
    // Run built-in memory benchmarks
    benchmark_allocations();           // Test different allocation sizes
    println!();
    benchmark_object_pools();          // Compare pool vs direct allocation  
    println!();
    benchmark_aligned_allocations();   // Test memory alignment impact
}