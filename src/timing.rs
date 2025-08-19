//! High-resolution timing utilities using CPU timestamp counter

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{_rdtsc, _mm_lfence, _mm_mfence};

/// Read timestamp counter for the current architecture
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_timestamp_with_fences() -> u64 {
    _mm_mfence();
    let tsc = _rdtsc();
    _mm_lfence();
    tsc
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn read_timestamp_with_fences() -> u64 {
    read_timestamp()
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn read_timestamp() -> u64 {
    read_virtual_counter()
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn read_virtual_counter() -> u64 {
    unsafe {
        let counter: u64;
        std::arch::asm!("mrs {}, cntvct_el0", out(reg) counter, options(nomem, nostack));
        counter
    }
}

#[cfg(target_arch = "aarch64")]
fn get_counter_frequency() -> u64 {
    unsafe {
        let freq: u64;
        std::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq, options(nomem, nostack));
        freq
    }
}

pub struct PrecisionTimer {
    start: u64,
    frequency_mhz: u64,
}

impl PrecisionTimer {
    #[inline(always)]
    pub fn start() -> Self {
        unsafe {
            let start = read_timestamp_with_fences();
            
            Self {
                start,
                frequency_mhz: crate::mock_core::cpu_frequency_mhz(),
            }
        }
    }
    
    #[inline(always)]
    pub fn stop(self) -> u64 {
        unsafe {
            let end = read_timestamp_with_fences();
            
            let cycles = end - self.start;
            if self.frequency_mhz == 0 {
                cycles
            } else {
                #[cfg(target_arch = "x86_64")]
                {
                    (cycles * 1000) / self.frequency_mhz
                }
                #[cfg(target_arch = "aarch64")]
                {
                    let counter_freq = get_counter_frequency();
                    if counter_freq > 0 {
                        if cycles == 0 {
                            0
                        } else {
                            std::cmp::max(1, (cycles * 1_000_000_000) / counter_freq)
                        }
                    } else {
                        (cycles * 1000) / self.frequency_mhz
                    }
                }
            }
        }
    }
}

pub fn time_function<F, R>(f: F) -> (R, u64)
where
    F: FnOnce() -> R,
{
    let timer = PrecisionTimer::start();
    let result = f();
    let elapsed = timer.stop();
    (result, elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::calibrate_tsc_frequency;
    
    #[test]
    fn test_precision_timer() {
        calibrate_tsc_frequency();
        
        let timer = PrecisionTimer::start();
        let _ = (0..10).sum::<i32>();
        let elapsed = timer.stop();
        
        #[cfg(target_arch = "x86_64")]
        {
            assert!(elapsed < 1000, "Elapsed time too high: {}ns", elapsed);
        }
        #[cfg(target_arch = "aarch64")]
        {
            assert!(elapsed < 5000, "Elapsed time too high: {}ns", elapsed);
        }
    }
    
    #[test]
    fn test_time_function() {
        calibrate_tsc_frequency();
        
        let (result, elapsed) = time_function(|| {
            (0..100).sum::<i32>()
        });
        
        assert_eq!(result, 4950);
        assert!(elapsed < 10000, "Function took too long: {}ns", elapsed);
    }
}