//! High-precision benchmarking tools for HFT systems

pub mod timing;
pub mod stats;
pub mod allocation;
pub mod calibration;
pub mod mock_core;
pub mod environment;
pub mod desktop_config;
pub mod server_config;

pub use timing::{PrecisionTimer, time_function};
pub use stats::{BenchmarkResults, BenchmarkAnalysis};
pub use calibration::{calibrate_tsc_frequency, quick_calibrate_tsc_frequency};
pub use allocation::{benchmark_allocations, benchmark_object_pools, benchmark_aligned_allocations};
pub use environment::{validate_benchmark_environment, print_environment_report, EnvironmentReport};
pub use desktop_config::{configure_for_desktop_memory_benchmarks, configure_for_desktop_cpu_benchmarks, check_desktop_suitability, DesktopSuitability};
pub use server_config::{configure_for_server_memory_benchmarks, configure_for_server_cpu_benchmarks, check_server_environment, ServerEnvironment};

pub struct SimpleBench {
    results: BenchmarkResults,
}

impl SimpleBench {
    pub fn new(name: &str) -> Self {
        Self {
            results: BenchmarkResults::new(name.to_string()),
        }
    }
    
    pub fn bench<F, R>(mut self, iterations: usize, mut f: F) -> Self
    where
        F: FnMut() -> R,
    {
        for _ in 0..iterations {
            let (_, elapsed) = time_function(&mut f);
            self.results.record(elapsed);
        }
        self
    }
    
    pub fn report(self) {
        println!("{}", self.results.analyze().summary());
    }
    
    pub fn analyze(self) -> BenchmarkAnalysis {
        self.results.analyze()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_bench() {
        quick_calibrate_tsc_frequency();
        
        let bench = SimpleBench::new("test_simple");
        let analysis = bench.bench(100, || {
            (0..10).sum::<i32>()
        }).analyze();
        
        assert_eq!(analysis.count, 100);
        assert!(analysis.mean < 10000);
    }
    
    #[test]
    fn test_bench_chaining() {
        quick_calibrate_tsc_frequency();
        
        let analysis = SimpleBench::new("chain_test")
            .bench(50, || { 42 })
            .analyze();
        
        assert_eq!(analysis.count, 50);
        assert_eq!(analysis.name, "chain_test");
    }
    
    #[test]
    fn test_time_function() {
        quick_calibrate_tsc_frequency();
        
        let (result, elapsed) = time_function(|| {
            (1..=10).sum::<i32>()
        });
        
        assert_eq!(result, 55);
        assert!(elapsed > 0);
    }
}