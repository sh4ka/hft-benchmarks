//! High-precision timing benchmarks

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use hft_benchmarks::{PrecisionTimer, calibrate_tsc_frequency};
use hft_benchmarks::mock_core::{Timestamp, Price, Quantity, SPSCRingBuffer, WaitFreeHashTable};
use std::time::Duration;

fn benchmark_timestamp_operations(c: &mut Criterion) {
    // Environment validation temporarily disabled
    
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("timestamp_operations");
    
    // Configure for better stability on Apple Silicon
    #[cfg(target_arch = "aarch64")]
    {
        group.sample_size(1000);  // More samples for ARM64 variance
        group.measurement_time(Duration::from_secs(30));  // Longer measurement
        group.warm_up_time(Duration::from_secs(5));
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        group.sample_size(10000);
        group.measurement_time(Duration::from_secs(60));
        group.warm_up_time(Duration::from_secs(3));
    }
    
    group.bench_function("timestamp_now", |b| {
        b.iter(|| {
            let _ts = Timestamp::now();
        })
    });
    
    group.bench_function("timestamp_duration", |b| {
        let ts1 = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_nanos(1000));
        let ts2 = Timestamp::now();
        
        b.iter(|| {
            let _duration = ts2.as_nanos() - ts1.as_nanos();
        })
    });
    
    group.bench_function("precision_timer_overhead", |b| {
        b.iter(|| {
            let timer = PrecisionTimer::start();
            let _elapsed = timer.stop();
        })
    });
    
    group.finish();
}

fn benchmark_core_types(c: &mut Criterion) {
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("core_types");
    
    // Configure for stability
    #[cfg(target_arch = "aarch64")]
    {
        group.sample_size(2000);
        group.measurement_time(Duration::from_secs(20));
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        group.sample_size(5000);
        group.measurement_time(Duration::from_secs(30));
    }
    
    let price1 = Price::new(100.00);
    let price2 = Price::new(50.00);
    
    group.bench_function("price_arithmetic", |b| {
        b.iter(|| {
            let _sum = price1 + price2;
            let _diff = price1 - price2;
        })
    });
    
    let qty1 = Quantity::new(1000);
    let qty2 = Quantity::new(500);
    
    group.bench_function("quantity_operations", |b| {
        b.iter(|| {
            let _sum = qty1 + qty2;
            let _diff = qty1 - qty2;
            let _mul = qty1 + qty1;
        })
    });
    
    group.finish();
}

fn benchmark_lockfree_structures(c: &mut Criterion) {
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("lockfree_operations");
    
    // Apple Silicon needs special configuration due to different memory subsystem
    #[cfg(target_arch = "aarch64")]
    {
        group.sample_size(1500);  // Balance between accuracy and time
        group.measurement_time(Duration::from_secs(25));
        group.warm_up_time(Duration::from_secs(5));  // Longer warmup for cache stabilization
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        group.sample_size(3000);
        group.measurement_time(Duration::from_secs(45));
        group.warm_up_time(Duration::from_secs(3));
    }
    
    // SPSC ring buffer benchmarks
    let ring: SPSCRingBuffer<u64> = SPSCRingBuffer::new(4096);
    
    // Pre-populate ring buffer
    for i in 0..2000 {
        ring.push(i);
    }
    
    group.bench_function("spsc_push", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            // Use black_box to prevent optimizations that could affect timing
            let value = std::hint::black_box(counter);
            if !ring.push(value) {
                ring.pop(); // Make space
                ring.push(value);
            }
            counter += 1;
        })
    });
    
    group.bench_function("spsc_pop", |b| {
        b.iter(|| {
            ring.pop()
        })
    });
    
    group.bench_function("spsc_push_pop_pair", |b| {
        b.iter(|| {
            ring.push(42);
            ring.pop()
        })
    });
    
    // Wait-free hash table benchmarks
    let table: WaitFreeHashTable<u64, u64> = WaitFreeHashTable::new(1024);
    
    // Pre-populate table
    for i in 0..500 {
        table.insert(i, i * 2);
    }
    
    group.bench_function("hashtable_get", |b| {
        let mut key = 0u64;
        b.iter(|| {
            let lookup_key = key % 500;
            let result = std::hint::black_box(table.get(&lookup_key));
            key += 1;
            result
        })
    });
    
    group.bench_function("hashtable_insert", |b| {
        let mut key = 1000u64;
        b.iter(|| {
            table.insert(key, key * 2);
            key += 1;
        })
    });
    
    group.bench_function("hashtable_get_miss", |b| {
        // Create a set of miss keys to cycle through for more consistent timing
        let miss_keys: Vec<u64> = (999999..1000099).collect();
        let mut key_index = 0;
        
        b.iter(|| {
            let key = miss_keys[key_index % miss_keys.len()];
            key_index += 1;
            std::hint::black_box(table.get(&key)) // Key that doesn't exist
        })
    });
    
    group.finish();
}

fn benchmark_different_ring_sizes(c: &mut Criterion) {
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("ring_buffer_sizes");
    
    // Configure for consistent measurements
    group.sample_size(500);  // Fewer samples since we test multiple sizes
    group.measurement_time(Duration::from_secs(15));
    group.warm_up_time(Duration::from_secs(2));
    
    for size_bits in [8, 10, 12, 14].iter() { // 256, 1024, 4096, 16384
        let size = 1 << size_bits;
        
        group.bench_with_input(
            BenchmarkId::new("spsc_push_pop", size),
            &size,
            |b, _size| {
                // Create ring with the actual size being tested
                let ring: SPSCRingBuffer<u64> = SPSCRingBuffer::new(size);
                
                b.iter(|| {
                    ring.push(42);
                    ring.pop()
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    timing_benches,
    benchmark_timestamp_operations,
    benchmark_core_types,
    benchmark_lockfree_structures,
    benchmark_different_ring_sizes
);
criterion_main!(timing_benches);