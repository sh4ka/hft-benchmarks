//! Server-optimized timing benchmarks for production validation

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use hft_benchmarks::{PrecisionTimer, calibrate_tsc_frequency, configure_for_server_cpu_benchmarks, check_server_environment};
use hft_benchmarks::mock_core::{Timestamp, Price, Quantity, SPSCRingBuffer, WaitFreeHashTable};

fn benchmark_timestamp_operations_server(c: &mut Criterion) {
    // Validate server environment
    let env = check_server_environment();
    env.print_detailed_report();
    
    if !env.is_optimal {
        eprintln!("⚠️  Server environment is not optimal. Results may not be reliable.");
        eprintln!("💡 Run: sudo ./scripts/server-benchmark-setup.sh run");
    }
    
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("server_timestamp_operations");
    
    // Use server-optimized configuration
    configure_for_server_cpu_benchmarks(&mut group);
    
    group.bench_function("timestamp_now", |b| {
        b.iter(|| {
            std::hint::black_box(Timestamp::now());
        })
    });
    
    group.bench_function("timestamp_duration", |b| {
        let ts1 = Timestamp::now();
        // Small delay to ensure measurable difference
        for _ in 0..10 {
            std::hint::spin_loop();
        }
        let ts2 = Timestamp::now();
        
        b.iter(|| {
            std::hint::black_box(ts2.as_nanos() - ts1.as_nanos());
        })
    });
    
    group.bench_function("precision_timer_overhead", |b| {
        b.iter(|| {
            let timer = PrecisionTimer::start();
            let elapsed = timer.stop();
            std::hint::black_box(elapsed);
        })
    });
    
    group.finish();
}

fn benchmark_core_types_server(c: &mut Criterion) {
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("server_core_types");
    
    configure_for_server_cpu_benchmarks(&mut group);
    
    let price1 = Price::new(100.00);
    let price2 = Price::new(50.00);
    
    group.bench_function("price_arithmetic", |b| {
        b.iter(|| {
            let sum = std::hint::black_box(price1 + price2);
            let diff = std::hint::black_box(price1 - price2);
            let mult = std::hint::black_box(price1 + price1);
            (sum, diff, mult)
        })
    });
    
    let qty1 = Quantity::new(1000);
    let qty2 = Quantity::new(500);
    
    group.bench_function("quantity_operations", |b| {
        b.iter(|| {
            let sum = std::hint::black_box(qty1 + qty2);
            let diff = std::hint::black_box(qty1 - qty2);
            let mult = std::hint::black_box(qty1 + qty1);
            (sum, diff, mult)
        })
    });
    
    group.finish();
}

fn benchmark_lockfree_structures_server(c: &mut Criterion) {
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("server_lockfree_operations");
    
    configure_for_server_cpu_benchmarks(&mut group);
    
    // SPSC ring buffer benchmarks with larger buffer for server testing
    let ring: SPSCRingBuffer<u64> = SPSCRingBuffer::new(8192);
    
    // Pre-populate ring buffer to 50% capacity
    for i in 0..4096 {
        ring.push(i);
    }
    
    group.bench_function("spsc_push_server", |b| {
        let mut counter = 4096u64;
        b.iter(|| {
            let value = std::hint::black_box(counter);
            if !ring.push(value) {
                // Buffer full, pop one to make space
                ring.pop();
                ring.push(value);
            }
            counter += 1;
        })
    });
    
    group.bench_function("spsc_pop_server", |b| {
        b.iter(|| {
            std::hint::black_box(ring.pop())
        })
    });
    
    group.bench_function("spsc_push_pop_pair_server", |b| {
        b.iter(|| {
            let push_result = ring.push(std::hint::black_box(42));
            let pop_result = ring.pop();
            std::hint::black_box((push_result, pop_result))
        })
    });
    
    // Wait-free hash table benchmarks with larger table
    let table: WaitFreeHashTable<u64, u64> = WaitFreeHashTable::new(4096);
    
    // Pre-populate table to 25% capacity
    for i in 0..1024 {
        table.insert(i, i * 2);
    }
    
    group.bench_function("hashtable_get_server", |b| {
        let mut key = 0u64;
        b.iter(|| {
            let lookup_key = key % 1024;
            let result = std::hint::black_box(table.get(&lookup_key));
            key += 1;
            result
        })
    });
    
    group.bench_function("hashtable_insert_server", |b| {
        let mut key = 2048u64;
        b.iter(|| {
            let insert_key = std::hint::black_box(key);
            table.insert(insert_key, insert_key * 2);
            key += 1;
        })
    });
    
    group.bench_function("hashtable_get_miss_server", |b| {
        // Create deterministic miss keys to avoid randomness
        let miss_keys: Vec<u64> = (1000000..1001000).collect();
        let mut key_index = 0;
        
        b.iter(|| {
            let key = miss_keys[key_index % miss_keys.len()];
            key_index += 1;
            std::hint::black_box(table.get(&key)) // Key that doesn't exist
        })
    });
    
    group.finish();
}

fn benchmark_scaling_server(c: &mut Criterion) {
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("server_scaling");
    
    configure_for_server_cpu_benchmarks(&mut group);
    
    // Test different data structure sizes to understand scaling behavior
    for size_exp in [10, 12, 14, 16].iter() { // 1K, 4K, 16K, 64K
        let size = 1 << size_exp;
        
        group.bench_with_input(
            BenchmarkId::new("hashtable_scaling", size),
            &size,
            |b, &size| {
                let table: WaitFreeHashTable<u64, u64> = WaitFreeHashTable::new(65536);
                
                // Pre-populate
                for i in 0..size {
                    table.insert(i, i * 2);
                }
                
                b.iter(|| {
                    let key = std::hint::black_box(fastrand::u64(0..size));
                    std::hint::black_box(table.get(&key))
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("ringbuffer_scaling", size),
            &size,
            |b, &size| {
                let ring: SPSCRingBuffer<u64> = SPSCRingBuffer::new(65536);
                
                // Pre-populate to 50% capacity
                for i in 0..size/2 {
                    ring.push(i);
                }
                
                b.iter(|| {
                    // Alternate push/pop to maintain steady state
                    if fastrand::bool() {
                        ring.push(std::hint::black_box(42));
                    } else {
                        std::hint::black_box(ring.pop());
                    }
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    server_timing_benches,
    benchmark_timestamp_operations_server,
    benchmark_core_types_server,
    benchmark_lockfree_structures_server,
    benchmark_scaling_server
);
criterion_main!(server_timing_benches);