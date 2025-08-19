//! Server-optimized benchmark configuration

use std::time::Duration;

/// Configure Criterion for server benchmarking with high precision
pub fn configure_for_server_memory_benchmarks(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    // Server-specific configuration for memory benchmarks
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 server configuration
        group.sample_size(5000);                              // High sample size for precision
        group.measurement_time(Duration::from_secs(180));     // Long measurement time
        group.warm_up_time(Duration::from_secs(30));          // Extended warmup
        group.confidence_level(0.95);                         // Standard 95% confidence
        group.noise_threshold(0.005);                         // Very low noise tolerance (0.5%)
        group.significance_level(0.01);                       // Strict significance testing
    }
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 server configuration (less common but supported)
        group.sample_size(3000);
        group.measurement_time(Duration::from_secs(120));
        group.warm_up_time(Duration::from_secs(20));
        group.confidence_level(0.95);
        group.noise_threshold(0.01);
        group.significance_level(0.01);
    }
}

/// Configure Criterion for server CPU benchmarks
pub fn configure_for_server_cpu_benchmarks(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    #[cfg(target_arch = "x86_64")]
    {
        group.sample_size(10000);                             // Very high sample size
        group.measurement_time(Duration::from_secs(240));     // 4 minutes for stability
        group.warm_up_time(Duration::from_secs(20));
        group.confidence_level(0.99);                         // 99% confidence for critical measurements
        group.noise_threshold(0.002);                         // Ultra-low noise tolerance (0.2%)
        group.significance_level(0.005);                      // Very strict significance
    }
    #[cfg(target_arch = "aarch64")]
    {
        group.sample_size(7000);
        group.measurement_time(Duration::from_secs(180));
        group.warm_up_time(Duration::from_secs(15));
        group.confidence_level(0.99);
        group.noise_threshold(0.005);
        group.significance_level(0.01);
    }
}

/// Pre-warm system for server benchmarking
pub fn prewarm_server_subsystems() {
    println!("🔥 Pre-warming server subsystems...");
    
    // Warm up allocators across NUMA nodes
    for node in 0..get_numa_node_count() {
        if let Ok(_) = std::process::Command::new("numactl")
            .args([&format!("--cpunodebind={node}"), "--membind", &node.to_string()])
            .args(["echo", "warming", "node"])
            .output()
        {
            // Allocate memory on this NUMA node
            let sizes = [1024, 4096, 16384, 65536];
            for &size in &sizes {
                let vec = vec![0u8; size];
                std::hint::black_box(&vec);
                drop(vec);
            }
        }
    }
    
    // CPU cache warming
    for _ in 0..1000 {
        let data: Vec<u64> = (0..1000).collect();
        let _sum: u64 = data.iter().sum();
        std::hint::black_box(_sum);
    }
    
    // System call warming
    for _ in 0..100 {
        let _ = std::time::SystemTime::now();
        std::thread::yield_now();
    }
    
    // Allow system to settle
    std::thread::sleep(Duration::from_secs(5));
    println!("✅ Server subsystems warmed up");
}

/// Check server environment suitability
pub fn check_server_environment() -> ServerEnvironment {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut info = std::collections::HashMap::new();
    
    // Check if running in container/VM
    if is_virtualized() {
        issues.push("Running on virtualized hardware - not optimal for HFT".to_string());
    }
    
    // Check CPU count
    let cpu_count = num_cpus::get();
    info.insert("cpu_count".to_string(), cpu_count.to_string());
    if cpu_count < 8 {
        warnings.push(format!("Low CPU count: {cpu_count} (recommended: 16+)"));
    }
    
    // Check memory
    let (total_mem_gb, available_mem_gb) = get_memory_info();
    info.insert("total_memory_gb".to_string(), format!("{total_mem_gb:.1}"));
    info.insert("available_memory_gb".to_string(), format!("{available_mem_gb:.1}"));
    
    if total_mem_gb < 16.0 {
        warnings.push(format!("Low total memory: {total_mem_gb:.1}GB (recommended: 32GB+)"));
    }
    if available_mem_gb < 8.0 {
        issues.push(format!("Low available memory: {available_mem_gb:.1}GB"));
    }
    
    // Check CPU governor
    if let Some(governor) = get_cpu_governor() {
        info.insert("cpu_governor".to_string(), governor.clone());
        if governor != "performance" {
            warnings.push(format!("CPU governor not optimal: {governor} (should be 'performance')"));
        }
    }
    
    // Check system load
    let load_avg = get_load_average();
    info.insert("load_average".to_string(), format!("{load_avg:.2}"));
    if load_avg > 2.0 {
        warnings.push(format!("High system load: {load_avg:.2}"));
    }
    
    // Check for NUMA
    let numa_nodes = get_numa_node_count();
    info.insert("numa_nodes".to_string(), numa_nodes.to_string());
    if numa_nodes > 1 {
        info.insert("numa_topology".to_string(), "detected".to_string());
    }
    
    ServerEnvironment {
        is_optimal: issues.is_empty(),
        issues,
        warnings,
        info,
    }
}

/// Server environment assessment
pub struct ServerEnvironment {
    pub is_optimal: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub info: std::collections::HashMap<String, String>,
}

impl ServerEnvironment {
    pub fn print_detailed_report(&self) {
        println!("🖥️  Server Benchmark Environment Report");
        println!("=======================================");
        
        // System Information
        println!("\n📊 System Information:");
        for (key, value) in &self.info {
            println!("   {}: {}", key.replace('_', " "), value);
        }
        
        // Assessment
        if self.is_optimal {
            println!("\n✅ OPTIMAL: Server environment is ideal for HFT benchmarking");
        } else {
            println!("\n❌ ISSUES DETECTED: Server environment has problems");
        }
        
        if !self.issues.is_empty() {
            println!("\n❌ Critical Issues:");
            for issue in &self.issues {
                println!("   - {issue}");
            }
        }
        
        if !self.warnings.is_empty() {
            println!("\n⚠️  Warnings:");
            for warning in &self.warnings {
                println!("   - {warning}");
            }
        }
        
        // Recommendations
        if !self.is_optimal || !self.warnings.is_empty() {
            println!("\n💡 Recommendations:");
            println!("   - Run with: sudo ./scripts/server-benchmark-setup.sh run");
            println!("   - Ensure bare metal (no virtualization)");
            println!("   - Set CPU governor to 'performance'");
            println!("   - Stop non-essential services");
            println!("   - Use CPU isolation and NUMA binding");
        }
        
        println!("=======================================");
    }
}

// Helper functions

fn is_virtualized() -> bool {
    // Check for common virtualization indicators
    std::fs::read_to_string("/proc/cpuinfo")
        .map(|content| content.contains("hypervisor"))
        .unwrap_or(false)
    || std::path::Path::new("/proc/xen").exists()
    || std::path::Path::new("/sys/hypervisor/uuid").exists()
}

fn get_memory_info() -> (f64, f64) {
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0u64;
        let mut available_kb = 0u64;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available_kb = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        
        (
            total_kb as f64 / 1024.0 / 1024.0,       // Convert to GB
            available_kb as f64 / 1024.0 / 1024.0,
        )
    } else {
        (0.0, 0.0)
    }
}

fn get_cpu_governor() -> Option<String> {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .ok()
        .map(|s| s.trim().to_string())
}

fn get_load_average() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|content| {
            content.split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(0.0)
}

fn get_numa_node_count() -> usize {
    std::process::Command::new("numactl")
        .args(["--hardware"])
        .output()
        .ok()
        .and_then(|output| {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.lines()
                .find(|line| line.contains("available:"))
                .and_then(|line| {
                    line.split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                })
        })
        .unwrap_or(1)
}