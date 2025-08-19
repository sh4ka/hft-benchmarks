//! Environment validation for consistent benchmarking

use std::fs;

/// Environment validation result
#[derive(Debug, Clone)]
pub struct EnvironmentReport {
    pub thermal_state: ThermalState,
    pub power_state: PowerState, 
    pub memory_pressure: MemoryPressure,
    pub cpu_usage: f64,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThermalState {
    Normal,
    Warm,
    Hot,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PowerState {
    AC,           // On AC power
    Battery,      // On battery
    LowBattery,   // Low battery
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPressure {
    Normal,
    Moderate,
    High,
    Critical,
}

impl EnvironmentReport {
    /// Check if environment is suitable for reliable benchmarking
    pub fn is_suitable_for_benchmarking(&self) -> bool {
        self.errors.is_empty() 
            && self.thermal_state != ThermalState::Critical
            && self.power_state != PowerState::LowBattery
            && self.memory_pressure != MemoryPressure::Critical
            && self.cpu_usage < 50.0
    }
    
    /// Get a summary message about the environment
    pub fn summary(&self) -> String {
        let mut parts = vec![
            format!("Thermal: {:?}", self.thermal_state),
            format!("Power: {:?}", self.power_state),
            format!("Memory: {:?}", self.memory_pressure),
            format!("CPU: {:.1}%", self.cpu_usage),
        ];
        
        if !self.warnings.is_empty() {
            parts.push(format!("Warnings: {}", self.warnings.len()));
        }
        
        if !self.errors.is_empty() {
            parts.push(format!("Errors: {}", self.errors.len()));
        }
        
        parts.join(", ")
    }
}

/// Validate the current environment for benchmarking
pub fn validate_benchmark_environment() -> EnvironmentReport {
    let mut report = EnvironmentReport {
        thermal_state: ThermalState::Normal,
        power_state: PowerState::Unknown,
        memory_pressure: MemoryPressure::Normal,
        cpu_usage: 0.0,
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    
    // Check thermal state
    report.thermal_state = check_thermal_state(&mut report.warnings, &mut report.errors);
    
    // Check power state  
    report.power_state = check_power_state(&mut report.warnings, &mut report.errors);
    
    // Check memory pressure
    report.memory_pressure = check_memory_pressure(&mut report.warnings, &mut report.errors);
    
    // Check CPU usage
    report.cpu_usage = check_cpu_usage(&mut report.warnings, &mut report.errors);
    
    // macOS specific checks
    #[cfg(target_os = "macos")]
    {
        check_macos_specific(&mut report.warnings, &mut report.errors);
    }
    
    report
}

/// Check thermal state
fn check_thermal_state(warnings: &mut Vec<String>, errors: &mut Vec<String>) -> ThermalState {
    #[cfg(target_os = "macos")]
    {
        // Use powermetrics to check thermal state
        if let Ok(output) = std::process::Command::new("pmset")
            .args(["-g", "thermlog"])
            .output() 
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Parse thermal state from output
            if output_str.contains("CPU_Speed_Limit") {
                let state = if output_str.contains("100") {
                    ThermalState::Normal
                } else if output_str.contains("75") {
                    warnings.push("CPU thermal throttling detected (75%)".to_string());
                    ThermalState::Warm  
                } else if output_str.contains("50") {
                    warnings.push("Significant CPU thermal throttling (50%)".to_string());
                    ThermalState::Hot
                } else {
                    errors.push("Critical CPU thermal throttling detected".to_string());
                    ThermalState::Critical
                };
                
                return state;
            }
        }
        
        // Fallback: Check temperature via system sensors
        if let Ok(output) = std::process::Command::new("system_profiler")
            .args(["SPHardwareDataType"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // This is a simplified check - real thermal monitoring needs more sophistication
            ThermalState::Normal
        } else {
            warnings.push("Could not determine thermal state".to_string());
            ThermalState::Normal
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Check /sys/class/thermal/thermal_zone*/temp
        let mut max_temp = 0;
        let mut found_temp = false;
        
        for entry in fs::read_dir("/sys/class/thermal").unwrap_or_else(|_| {
            warnings.push("Could not access thermal information".to_string());
            fs::read_dir("/tmp").unwrap()// Empty fallback
        }) {
            if let Ok(entry) = entry {
                let path = entry.path().join("temp");
                if let Ok(temp_str) = fs::read_to_string(&path) {
                    if let Ok(temp) = temp_str.trim().parse::<u32>() {
                        // Temperatures in millidegrees Celsius
                        let temp_c = temp / 1000;
                        max_temp = max_temp.max(temp_c);
                        found_temp = true;
                    }
                }
            }
        }
        
        if found_temp {
            if max_temp < 60 {
                ThermalState::Normal
            } else if max_temp < 80 {
                warnings.push(format!("Elevated CPU temperature: {max_temp}°C"));
                ThermalState::Warm
            } else if max_temp < 95 {
                warnings.push(format!("High CPU temperature: {max_temp}°C"));
                ThermalState::Hot
            } else {
                errors.push(format!("Critical CPU temperature: {max_temp}°C"));
                ThermalState::Critical
            }
        } else {
            warnings.push("Could not determine CPU temperature".to_string());
            ThermalState::Normal
        }
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        warnings.push("Thermal monitoring not supported on this platform".to_string());
        ThermalState::Normal
    }
}

/// Check power state
fn check_power_state(warnings: &mut Vec<String>, _errors: &mut Vec<String>) -> PowerState {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("pmset")
            .args(["-g", "ps"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            if output_str.contains("AC Power") {
                PowerState::AC
            } else if output_str.contains("Battery Power") {
                // Try to get battery percentage
                if let Ok(battery_output) = std::process::Command::new("pmset")
                    .args(["-g", "batt"])
                    .output()
                {
                    let battery_str = String::from_utf8_lossy(&battery_output.stdout);
                    
                    // Parse battery percentage
                    if let Some(percent_match) = battery_str.find('%') {
                        let start = battery_str[..percent_match].rfind(' ').unwrap_or(0);
                        if let Ok(percentage) = battery_str[start..percent_match].trim().parse::<u32>() {
                            if percentage < 20 {
                                warnings.push(format!("Low battery: {}%", percentage));
                                return PowerState::LowBattery;
                            } else if percentage < 50 {
                                warnings.push(format!("Battery power: {}%", percentage));
                            }
                        }
                    }
                }
                PowerState::Battery
            } else {
                PowerState::Unknown
            }
        } else {
            warnings.push("Could not determine power state".to_string());
            PowerState::Unknown
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // For non-macOS systems, assume AC power
        PowerState::AC
    }
}

/// Check memory pressure
fn check_memory_pressure(warnings: &mut Vec<String>, errors: &mut Vec<String>) -> MemoryPressure {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("memory_pressure")
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            if output_str.contains("System-wide memory free percentage:") {
                // Parse the percentage
                if let Some(line) = output_str.lines()
                    .find(|line| line.contains("System-wide memory free percentage:"))
                {
                    if let Some(percent_str) = line.split(':').nth(1) {
                        if let Ok(free_percent) = percent_str.trim().trim_end_matches('%').parse::<f64>() {
                            if free_percent > 50.0 {
                                return MemoryPressure::Normal;
                            } else if free_percent > 25.0 {
                                warnings.push(format!("Moderate memory pressure: {:.1}% free", free_percent));
                                return MemoryPressure::Moderate;
                            } else if free_percent > 10.0 {
                                warnings.push(format!("High memory pressure: {:.1}% free", free_percent));
                                return MemoryPressure::High;
                            } else {
                                errors.push(format!("Critical memory pressure: {:.1}% free", free_percent));
                                return MemoryPressure::Critical;
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: use vm_stat
        if let Ok(output) = std::process::Command::new("vm_stat").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // This would need proper parsing of vm_stat output
            // For now, assume normal
            MemoryPressure::Normal
        } else {
            warnings.push("Could not determine memory pressure".to_string());
            MemoryPressure::Normal
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut mem_total = 0;
            let mut mem_available = 0;
            
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    mem_total = line.split_whitespace().nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                } else if line.starts_with("MemAvailable:") {
                    mem_available = line.split_whitespace().nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                }
            }
            
            if mem_total > 0 {
                let available_percent = (mem_available as f64 / mem_total as f64) * 100.0;
                
                if available_percent > 50.0 {
                    MemoryPressure::Normal
                } else if available_percent > 25.0 {
                    warnings.push(format!("Moderate memory pressure: {available_percent:.1}% available"));
                    MemoryPressure::Moderate
                } else if available_percent > 10.0 {
                    warnings.push(format!("High memory pressure: {available_percent:.1}% available"));
                    MemoryPressure::High
                } else {
                    errors.push(format!("Critical memory pressure: {available_percent:.1}% available"));
                    MemoryPressure::Critical
                }
            } else {
                warnings.push("Could not parse memory information".to_string());
                MemoryPressure::Normal
            }
        } else {
            warnings.push("Could not read memory information".to_string());
            MemoryPressure::Normal
        }
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        warnings.push("Memory pressure monitoring not supported on this platform".to_string());
        MemoryPressure::Normal
    }
}

/// Check CPU usage
fn check_cpu_usage(warnings: &mut Vec<String>, _errors: &mut Vec<String>) -> f64 {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("top")
            .args(["-l", "1", "-n", "0"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Look for CPU usage line: "CPU usage: 12.34% user, 5.67% sys, 81.99% idle"
            if let Some(line) = output_str.lines()
                .find(|line| line.contains("CPU usage:"))
            {
                if let Some(idle_part) = line.split(',').find(|part| part.contains("idle")) {
                    if let Some(percent_str) = idle_part.split_whitespace().next() {
                        if let Ok(idle_percent) = percent_str.trim_end_matches('%').parse::<f64>() {
                            let usage_percent = 100.0 - idle_percent;
                            
                            if usage_percent > 75.0 {
                                warnings.push(format!("High CPU usage: {:.1}%", usage_percent));
                            } else if usage_percent > 50.0 {
                                warnings.push(format!("Moderate CPU usage: {:.1}%", usage_percent));
                            }
                            
                            return usage_percent;
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Read /proc/loadavg
        if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
            if let Some(load_str) = loadavg.split_whitespace().next() {
                if let Ok(load) = load_str.parse::<f64>() {
                    // Convert load average to rough CPU percentage
                    let usage_percent = load * 100.0;
                    
                    if usage_percent > 75.0 {
                        warnings.push(format!("High system load: {load:.2}"));
                    } else if usage_percent > 50.0 {
                        warnings.push(format!("Moderate system load: {load:.2}"));
                    }
                    
                    return usage_percent.min(100.0);
                }
            }
        }
    }
    
    warnings.push("Could not determine CPU usage".to_string());
    0.0
}

/// macOS-specific environment checks
#[cfg(target_os = "macos")]
fn check_macos_specific(warnings: &mut Vec<String>, _errors: &mut Vec<String>) {
    // Check if Spotlight is indexing
    if let Ok(output) = std::process::Command::new("mdutil")
        .args(["-s", "/"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.contains("Indexing enabled") && !output_str.contains("Indexing disabled") {
            // Check if indexing is actively running
            if let Ok(_) = std::process::Command::new("pgrep")
                .args(["mds"])
                .output()
            {
                warnings.push("Spotlight indexing may be active".to_string());
            }
        }
    }
    
    // Check for active Time Machine backups
    if let Ok(output) = std::process::Command::new("tmutil")
        .args(["currentphase"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if !output_str.trim().is_empty() && output_str != "BackupNotRunning" {
            warnings.push("Time Machine backup may be active".to_string());
        }
    }
    
    // Check for Software Update activity
    if let Ok(output) = std::process::Command::new("softwareupdate")
        .args(["-l"])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if !output_str.contains("No new software available") {
                warnings.push("Software updates available (may cause background activity)".to_string());
            }
        }
    }
}

/// Print a detailed environment report
pub fn print_environment_report(report: &EnvironmentReport) {
    println!("=== Benchmark Environment Report ===");
    println!("Thermal State: {:?}", report.thermal_state);
    println!("Power State: {:?}", report.power_state);
    println!("Memory Pressure: {:?}", report.memory_pressure);
    println!("CPU Usage: {:.1}%", report.cpu_usage);
    
    if !report.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &report.warnings {
            println!("  ⚠️  {warning}");
        }
    }
    
    if !report.errors.is_empty() {
        println!("\nErrors:");
        for error in &report.errors {
            println!("  ❌ {error}");
        }
    }
    
    println!("\nSuitable for benchmarking: {}", 
        if report.is_suitable_for_benchmarking() { "✅ Yes" } else { "❌ No" });
    println!("=====================================");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_validation() {
        let report = validate_benchmark_environment();
        
        // Should not crash and should produce a report
        assert!(!report.summary().is_empty());
        
        // Should have some reasonable values
        assert!(report.cpu_usage >= 0.0);
        assert!(report.cpu_usage <= 200.0); // Allow for multi-core
    }
    
    #[test]
    fn test_environment_report_summary() {
        let report = EnvironmentReport {
            thermal_state: ThermalState::Normal,
            power_state: PowerState::AC,
            memory_pressure: MemoryPressure::Normal,
            cpu_usage: 25.5,
            warnings: vec!["Test warning".to_string()],
            errors: vec![],
        };
        
        let summary = report.summary();
        assert!(summary.contains("Thermal: Normal"));
        assert!(summary.contains("Power: AC"));
        assert!(summary.contains("Memory: Normal"));
        assert!(summary.contains("CPU: 25.5%"));
        assert!(summary.contains("Warnings: 1"));
    }
    
    #[test]
    fn test_environment_suitability() {
        // Good environment
        let good_report = EnvironmentReport {
            thermal_state: ThermalState::Normal,
            power_state: PowerState::AC,
            memory_pressure: MemoryPressure::Normal,
            cpu_usage: 10.0,
            warnings: vec![],
            errors: vec![],
        };
        assert!(good_report.is_suitable_for_benchmarking());
        
        // Bad environment
        let bad_report = EnvironmentReport {
            thermal_state: ThermalState::Critical,
            power_state: PowerState::LowBattery,
            memory_pressure: MemoryPressure::Critical,
            cpu_usage: 90.0,
            warnings: vec![],
            errors: vec!["Critical error".to_string()],
        };
        assert!(!bad_report.is_suitable_for_benchmarking());
    }
}