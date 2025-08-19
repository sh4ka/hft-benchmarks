//! Memory allocation benchmarking utilities

use jemallocator::Jemalloc;
use crate::BenchmarkResults;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const DEFAULT_ITERATIONS: usize = 10_000;
const ALLOCATION_SIZES: [usize; 6] = [64, 128, 256, 512, 1024, 4096];

pub fn benchmark_allocations() {
    benchmark_allocations_with_iterations(DEFAULT_ITERATIONS)
}

pub fn benchmark_allocations_with_iterations(iterations: usize) {
    println!("Benchmarking memory allocations ({iterations} iterations per size)...");
    
    for &size in &ALLOCATION_SIZES {
        let mut results = BenchmarkResults::new(format!("allocation_{size}B"));
        
        for _ in 0..iterations {
            let (_, elapsed) = crate::timing::time_function(|| vec![0u8; size]);
            results.record(elapsed);
        }
        
        println!("{}", results.analyze().summary());
    }
}

pub struct SimpleObjectPool<T> {
    objects: Vec<Box<T>>,
}

impl<T> Default for SimpleObjectPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SimpleObjectPool<T> {
    pub fn new() -> Self {
        Self {
            objects: Vec::with_capacity(1000),
        }
    }
    
    pub fn get(&mut self) -> Box<T>
    where
        T: Default,
    {
        self.objects.pop().unwrap_or_else(|| Box::new(T::default()))
    }
    
    pub fn put(&mut self, obj: Box<T>) {
        if self.objects.len() < 1000 {
            self.objects.push(obj);
        }
    }
}

/// Benchmark object pools vs direct allocation
pub fn benchmark_object_pools() {
    benchmark_object_pools_with_iterations(DEFAULT_ITERATIONS)
}

/// Benchmark object pools with custom iteration count
pub fn benchmark_object_pools_with_iterations(iterations: usize) {
    println!("Benchmarking object pools vs direct allocation...");
    
    let mut pool = SimpleObjectPool::<u64>::new();
    let mut pool_results = BenchmarkResults::new("pool_allocation".to_string());
    let mut direct_results = BenchmarkResults::new("direct_allocation".to_string());
    
    for _ in 0..iterations {
        let (obj, elapsed) = crate::timing::time_function(|| pool.get());
        pool.put(obj);
        pool_results.record(elapsed);
        
        let (_, elapsed) = crate::timing::time_function(|| Box::new(0u64));
        direct_results.record(elapsed);
    }
    
    println!("Pool allocation: {}", pool_results.analyze().summary());
    println!("Direct allocation: {}", direct_results.analyze().summary());
}

/// Benchmark allocation alignment impact
pub fn benchmark_aligned_allocations() {
    benchmark_aligned_allocations_with_iterations(DEFAULT_ITERATIONS / 2)
}

/// Benchmark aligned allocations with custom iteration count
pub fn benchmark_aligned_allocations_with_iterations(iterations: usize) {
    println!("Benchmarking aligned vs unaligned allocations...");
    
    let mut aligned_results = BenchmarkResults::new("aligned_allocation".to_string());
    let mut unaligned_results = BenchmarkResults::new("unaligned_allocation".to_string());
    
    let aligned_layout = std::alloc::Layout::from_size_align(1024, 64).unwrap();
    let unaligned_layout = std::alloc::Layout::from_size_align(1024, 8).unwrap();
    
    for _ in 0..iterations {
        let (_, elapsed) = crate::timing::time_function(|| unsafe {
            let ptr = std::alloc::alloc(aligned_layout);
            if !ptr.is_null() {
                std::alloc::dealloc(ptr, aligned_layout);
            }
        });
        aligned_results.record(elapsed);
        
        let (_, elapsed) = crate::timing::time_function(|| unsafe {
            let ptr = std::alloc::alloc(unaligned_layout);
            if !ptr.is_null() {
                std::alloc::dealloc(ptr, unaligned_layout);
            }
        });
        unaligned_results.record(elapsed);
    }
    
    println!("Aligned allocation: {}", aligned_results.analyze().summary());
    println!("Unaligned allocation: {}", unaligned_results.analyze().summary());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_object_pool() {
        let mut pool = SimpleObjectPool::<u64>::new();
        
        let obj1 = pool.get();
        let obj2 = pool.get();
        
        pool.put(obj1);
        pool.put(obj2);
        
        // Should reuse objects
        let obj3 = pool.get();
        let obj4 = pool.get();
        
        drop(obj3);
        drop(obj4);
    }
    
    #[test]
    fn test_allocation_benchmarks() {
        crate::quick_calibrate_tsc_frequency();
        
        // These tests just ensure benchmarks don't panic
        // Actual performance will vary by system
        benchmark_object_pools_with_iterations(10);
        benchmark_allocations_with_iterations(10);
        benchmark_aligned_allocations_with_iterations(10);
    }
    
    #[test]
    fn test_object_pool_reuse() {
        let mut pool = SimpleObjectPool::<u64>::new();
        
        // Initially empty, should allocate
        let obj1 = pool.get();
        assert_eq!(*obj1, 0); // Default value
        
        // Return it to pool  
        pool.put(obj1);
        
        // Pool should now have one object
        assert_eq!(pool.objects.len(), 1);
        
        // Get should now reuse from pool instead of allocating
        let obj2 = pool.get();
        assert_eq!(*obj2, 0);
        
        // Pool should be empty again
        assert_eq!(pool.objects.len(), 0);
    }
    
    #[test]
    fn test_pool_capacity_limit() {
        let mut pool = SimpleObjectPool::<u64>::new();
        
        // Fill the pool beyond capacity
        for _ in 0..1100 {
            pool.put(Box::new(42));
        }
        
        // Pool should not exceed capacity
        assert!(pool.objects.len() <= 1000);
    }
}