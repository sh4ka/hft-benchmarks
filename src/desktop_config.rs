//! Desktop-optimized benchmark configuration

use std::time::Duration;

/// Configure Criterion for desktop benchmarking with higher outlier tolerance
pub fn configure_for_desktop_memory_benchmarks(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    // Desktop-specific configuration for memory benchmarks
    #[cfg(target_arch = "aarch64")]
    {
        // Apple Silicon M1/M2 configuration - balanced for desktop use
        group.sample_size(300);                               // Reasonable sample size
        group.measurement_time(Duration::from_secs(30));      // Manageable measurement time
        group.warm_up_time(Duration::from_secs(10));          // Sufficient warmup
        group.confidence_level(0.90);                         // Lower confidence level (90% vs 95%)
        group.noise_threshold(0.03);                          // Allow 3% noise threshold for desktop
        group.significance_level(0.05);                       // More forgiving significance testing
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        // x86_64 configuration - balanced for desktop use
        group.sample_size(400);
        group.measurement_time(Duration::from_secs(40));
        group.warm_up_time(Duration::from_secs(8));
        group.confidence_level(0.90);
        group.noise_threshold(0.025);
        group.significance_level(0.05);
    }
}

/// Configure Criterion for desktop CPU benchmarks (less sensitive to memory noise)
pub fn configure_for_desktop_cpu_benchmarks(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    #[cfg(target_arch = "aarch64")]
    {
        group.sample_size(1500);
        group.measurement_time(Duration::from_secs(45));
        group.warm_up_time(Duration::from_secs(10));
        group.confidence_level(0.95);
        group.noise_threshold(0.01);
        group.significance_level(0.01);
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        group.sample_size(2000);
        group.measurement_time(Duration::from_secs(60));
        group.warm_up_time(Duration::from_secs(8));
        group.confidence_level(0.95);
        group.noise_threshold(0.01);
        group.significance_level(0.01);
    }
}

/// Pre-warm memory subsystem to reduce cold-start effects
pub fn prewarm_memory_subsystem() {
    // Allocate and deallocate various sizes to warm up the allocator
    let sizes = [64, 128, 256, 512, 1024, 2048, 4096];
    
    for _ in 0..5 {  // Multiple rounds
        for &size in &sizes {
            let vec = vec![0u8; size];
            std::hint::black_box(&vec);
            drop(vec);
        }
        
        // Small delay to let the allocator settle
        std::thread::sleep(Duration::from_millis(10));
    }
    
    // Force a garbage collection if possible (future feature)
    // Note: Currently not implemented
}

/// Check if system is suitable for desktop benchmarking
pub fn check_desktop_suitability() -> DesktopSuitability {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    
    // Check if on battery power (macOS specific)
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        if let Ok(output) = Command::new("pmset").args(["-g", "ps"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("Battery Power") {
                warnings.push("Running on battery power - results may vary".to_string());
            }
        }
    }
    
    // Check available memory
    let available_memory_gb = get_available_memory_gb();
    if available_memory_gb < 4.0 {
        issues.push(format!("Low available memory: {available_memory_gb:.1} GB"));
    } else if available_memory_gb < 8.0 {
        warnings.push(format!("Moderate available memory: {available_memory_gb:.1} GB"));
    }
    
    // Check CPU usage
    let cpu_usage = get_cpu_usage_percentage();
    if cpu_usage > 50.0 {
        issues.push(format!("High CPU usage: {cpu_usage:.1}%"));
    } else if cpu_usage > 25.0 {
        warnings.push(format!("Moderate CPU usage: {cpu_usage:.1}%"));
    }
    
    DesktopSuitability {
        is_suitable: issues.is_empty(),
        issues,
        warnings,
    }
}

/// Results of desktop suitability check
pub struct DesktopSuitability {
    pub is_suitable: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

impl DesktopSuitability {
    pub fn print_report(&self) {
        println!("🖥️  Desktop Benchmark Suitability Report");
        println!("========================================");
        
        if self.is_suitable {
            println!("✅ System is suitable for desktop benchmarking");
        } else {
            println!("❌ System has issues that may affect benchmark reliability");
        }
        
        if !self.issues.is_empty() {
            println!("\n❌ Issues:");
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
        
        if !self.is_suitable || !self.warnings.is_empty() {
            println!("\n💡 Recommendations:");
            println!("   - Close unnecessary applications");
            println!("   - Connect to AC power if on battery");
            println!("   - Run during low system activity periods");
            println!("   - Consider using a dedicated server for critical benchmarks");
        }
        
        println!("========================================");
    }
}

/// Get available memory in GB (cross-platform)
fn get_available_memory_gb() -> f64 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        if let Ok(output) = Command::new("vm_stat").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Parse vm_stat output to get free memory
            // This is simplified - real implementation would parse the format properly
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() > 1 {
                // Very rough estimation
                return 8.0; // Default assumption for macOS systems
            }
        }
        
        8.0 // Default fallback
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemAvailable:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(kb_val) = kb.parse::<u64>() {
                            return kb_val as f64 / 1024.0 / 1024.0; // Convert KB to GB
                        }
                    }
                }
            }
        }
        
        4.0 // Default fallback
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        8.0 // Default assumption
    }
}

/// Get current CPU usage percentage
fn get_cpu_usage_percentage() -> f64 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        if let Ok(output) = Command::new("top").args(["-l", "1", "-n", "0"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Look for CPU usage line
            for line in output_str.lines() {
                if line.contains("CPU usage:") {
                    // Parse idle percentage and calculate usage
                    if let Some(idle_part) = line.split(',').find(|part| part.contains("idle")) {
                        if let Some(percent_str) = idle_part.split_whitespace().next() {
                            if let Ok(idle_pct) = percent_str.trim_end_matches('%').parse::<f64>() {
                                return 100.0 - idle_pct;
                            }
                        }
                    }
                }
            }
        }
        
        10.0 // Default assumption
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        
        if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
            if let Some(load_str) = loadavg.split_whitespace().next() {
                if let Ok(load) = load_str.parse::<f64>() {
                    return (load * 100.0).min(100.0); // Convert to rough percentage
                }
            }
        }
        
        10.0 // Default assumption
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        10.0 // Default assumption
    }
}