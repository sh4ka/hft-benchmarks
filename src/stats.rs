//! Statistical analysis for benchmark results

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

pub struct BenchmarkResults {
    measurements: Vec<u64>,
    name: String,
}

impl BenchmarkResults {
    pub fn new(name: String) -> Self {
        Self {
            measurements: Vec::with_capacity(10000),
            name,
        }
    }
    
    pub fn record(&mut self, nanoseconds: u64) {
        self.measurements.push(nanoseconds);
    }
    
    pub fn len(&self) -> usize {
        self.measurements.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.measurements.is_empty()
    }
    
    pub fn analyze(&self) -> BenchmarkAnalysis {
        if self.measurements.is_empty() {
            return BenchmarkAnalysis::empty(self.name.clone());
        }
        
        let mut sorted = self.measurements.clone();
        sorted.sort_unstable();
        
        let len = sorted.len();
        let sum: u64 = sorted.iter().sum();
        let mean = sum / len as u64;
        
        let variance = sorted.iter()
            .map(|&x| {
                let diff = (x as f64) - (mean as f64);
                diff * diff
            })
            .sum::<f64>() / len as f64;
        
        BenchmarkAnalysis {
            name: self.name.clone(),
            count: len,
            min: sorted[0],
            max: sorted[len - 1],
            mean,
            p50: percentile(&sorted, 50.0),
            p95: percentile(&sorted, 95.0),
            p99: percentile(&sorted, 99.0),
            p999: percentile(&sorted, 99.9),
            std_dev: variance.sqrt(),
        }
    }
    
    pub fn clear(&mut self) {
        self.measurements.clear();
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkAnalysis {
    pub name: String,
    pub count: usize,
    pub min: u64,
    pub max: u64,
    pub mean: u64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub p999: u64,
    pub std_dev: f64,
}

fn percentile(sorted_data: &[u64], p: f64) -> u64 {
    let len = sorted_data.len();
    if len == 0 { return 0; }
    if len == 1 { return sorted_data[0]; }
    
    let index = (p / 100.0 * (len - 1) as f64).round() as usize;
    sorted_data[index.min(len - 1)]
}

impl BenchmarkAnalysis {
    pub fn empty(name: String) -> Self {
        Self {
            name,
            count: 0,
            min: 0,
            max: 0,
            mean: 0,
            p50: 0,
            p95: 0,
            p99: 0,
            p999: 0,
            std_dev: 0.0,
        }
    }
    pub fn summary(&self) -> String {
        format!(
            "{}: {} samples, mean={:>6}ns, p50={:>6}ns, p95={:>6}ns, p99={:>6}ns, p99.9={:>6}ns, std_dev={:>6.1}ns",
            self.name, self.count, self.mean, self.p50, self.p95, self.p99, self.p999, self.std_dev
        )
    }
    
    pub fn meets_target(&self, target_p99_ns: u64) -> bool {
        self.p99 <= target_p99_ns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_results() {
        let mut results = BenchmarkResults::new("test".to_string());
        
        for i in 1..=100 {
            results.record(i * 10);
        }
        
        let analysis = results.analyze();
        assert_eq!(analysis.count, 100);
        assert_eq!(analysis.min, 10);
        assert_eq!(analysis.max, 1000);
        assert_eq!(analysis.mean, 505);
        assert_eq!(analysis.p50, 510);
    }
    
    #[test]
    fn test_empty_results() {
        let results = BenchmarkResults::new("empty".to_string());
        let analysis = results.analyze();
        
        assert_eq!(analysis.count, 0);
        assert_eq!(analysis.mean, 0);
    }
    
    #[test]
    fn test_target_checking() {
        let mut results = BenchmarkResults::new("target_test".to_string());
        
        for _ in 0..1000 {
            results.record(100);
        }
        
        let analysis = results.analyze();
        assert!(analysis.meets_target(150));
        assert!(!analysis.meets_target(50));
    }
    
    #[test]
    fn test_percentile_calculation() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        
        assert_eq!(percentile(&data, 50.0), 6);
        assert_eq!(percentile(&data, 90.0), 9);
        assert_eq!(percentile(&data, 99.0), 10);
        
        assert_eq!(percentile(&[], 50.0), 0);
        assert_eq!(percentile(&[42], 50.0), 42);
    }
    
    #[test]
    fn test_clear_measurements() {
        let mut results = BenchmarkResults::new("clear_test".to_string());
        results.record(100);
        results.record(200);
        
        assert_eq!(results.len(), 2);
        assert!(!results.is_empty());
        
        results.clear();
        assert_eq!(results.len(), 0);
        assert!(results.is_empty());
    }
    
    #[test]
    fn test_analysis_summary_format() {
        let mut results = BenchmarkResults::new("format_test".to_string());
        results.record(100);
        results.record(200);
        
        let analysis = results.analyze();
        let summary = analysis.summary();
        
        assert!(summary.contains("format_test"));
        assert!(summary.contains("2 samples"));
        assert!(summary.contains("mean"));
    }
}