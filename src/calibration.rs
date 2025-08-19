//! CPU frequency calibration for accurate timing

extern crate std;
use std::time::{Duration, Instant};

pub fn calibrate_tsc_frequency() -> u64 {
    calibrate_with_duration_ms(1000)
}

pub fn quick_calibrate_tsc_frequency() -> u64 {
    calibrate_with_duration_ms(100)
}

fn calibrate_with_duration_ms(duration_ms: u64) -> u64 {
    let start_time = Instant::now();
    let start_counter = read_timestamp();
    
    std::thread::sleep(Duration::from_millis(duration_ms));
    
    let end_time = Instant::now();
    let end_counter = read_timestamp();
    
    let elapsed_ns = end_time.duration_since(start_time).as_nanos() as u64;
    let counter_cycles = end_counter - start_counter;
    
    let frequency_mhz = calculate_frequency_mhz(counter_cycles, elapsed_ns);
    
    if duration_ms >= 1000 {
        #[cfg(target_arch = "x86_64")]
        println!("Calibrated TSC frequency: {frequency_mhz} MHz");
        #[cfg(target_arch = "aarch64")]
        println!("Calibrated ARM counter frequency: {} MHz", frequency_mhz);
    }
    
    crate::mock_core::set_cpu_frequency_mhz(frequency_mhz);
    frequency_mhz
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn read_timestamp() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn read_timestamp() -> u64 {
    crate::timing::read_timestamp()
}

fn calculate_frequency_mhz(counter_cycles: u64, elapsed_ns: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        (counter_cycles * 1000) / elapsed_ns
    }
    #[cfg(target_arch = "aarch64")]
    {
        let counter_freq = unsafe {
            let freq: u64;
            std::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq, options(nomem, nostack));
            freq
        };
        
        if counter_freq > 0 {
            counter_freq / 1_000_000
        } else {
            (counter_cycles * 1000) / elapsed_ns
        }
    }
}

#[cfg(target_os = "linux")]
pub fn get_cpu_frequency_from_proc() -> Option<u64> {
    use std::fs;
    
    let contents = fs::read_to_string("/proc/cpuinfo").ok()?;
    
    for line in contents.lines() {
        if line.starts_with("cpu MHz") {
            if let Some(freq_str) = line.split(':').nth(1) {
                if let Ok(freq) = freq_str.trim().parse::<f64>() {
                    return Some(freq as u64);
                }
            }
        }
    }
    
    None
}

#[cfg(not(target_os = "linux"))]
pub fn get_cpu_frequency_from_proc() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tsc_calibration() {
        let freq = calibrate_tsc_frequency();
        
        #[cfg(target_arch = "x86_64")]
        {
            assert!(freq >= 1000, "CPU frequency too low: {} MHz", freq);
            assert!(freq <= 5000, "CPU frequency too high: {} MHz", freq);
        }
        #[cfg(target_arch = "aarch64")]
        {
            assert!(freq >= 10, "Counter frequency too low: {} MHz", freq);
            assert!(freq <= 100, "Counter frequency too high: {} MHz", freq);
        }
        
        assert_eq!(crate::mock_core::cpu_frequency_mhz(), freq);
    }
    
    #[test]
    fn test_quick_calibration() {
        let freq = quick_calibrate_tsc_frequency();
        
        #[cfg(target_arch = "x86_64")]
        {
            assert!(freq >= 500, "Quick calibration frequency too low: {} MHz", freq);
            assert!(freq <= 6000, "Quick calibration frequency too high: {} MHz", freq);
        }
        #[cfg(target_arch = "aarch64")]
        {
            assert!(freq >= 5, "Quick calibration frequency too low: {} MHz", freq);
            assert!(freq <= 200, "Quick calibration frequency too high: {} MHz", freq);
        }
    }
    
    #[cfg(target_os = "linux")]
    #[test]
    fn test_proc_cpuinfo_parsing() {
        let _freq = get_cpu_frequency_from_proc();
    }
}