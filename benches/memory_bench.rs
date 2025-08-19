//! Memory allocation benchmarks

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use hft_benchmarks::{calibrate_tsc_frequency, configure_for_desktop_memory_benchmarks, check_desktop_suitability};
use hft_benchmarks::mock_core::{ObjectPool, NumaArenaAllocator};

fn benchmark_allocators(c: &mut Criterion) {
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("memory_allocation");
    
    // baseline
    group.bench_function("std_alloc_1kb", |b| {
        b.iter(|| {
            let _data = vec![0u8; 1024];
        })
    });
    
    // allocation
    let pool = ObjectPool::<Vec<u8>>::new();
    group.bench_function("pool_alloc_1kb", |b| {
        b.iter(|| {
            let obj = pool.get(|| vec![0u8; 1024]);
            pool.put(obj);
        })
    });
    
    // NUMA allocation
    let numa_alloc = NumaArenaAllocator::new(0);
    group.bench_function("numa_alloc_1kb", |b| {
        b.iter(|| {
            let _data = numa_alloc.allocate(1024);
        })
    });
    
    group.finish();
}

fn benchmark_pool_sizes(c: &mut Criterion) {
    let suitability = check_desktop_suitability();
    suitability.print_report();
    
    if !suitability.is_suitable {
        eprintln!("Warning: System conditions may lead to noisy benchmark results");
    }
    
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("object_pool_sizes");
    
    configure_for_desktop_memory_benchmarks(&mut group);
    
    for size in [64, 128, 256, 512, 1024].iter() {
        let pool = ObjectPool::<Vec<u8>>::new();
        
        group.bench_with_input(BenchmarkId::new("pool_alloc", size), size, |b, size| {
            for _ in 0..100 { // warmup
                let obj = pool.get(|| vec![0u8; *size]);
                std::hint::black_box(&obj);
                pool.put(obj);
            }
            
            b.iter(|| {
                let obj = pool.get(|| vec![0u8; *size]);
                // box the object to prevent optimization by rust
                std::hint::black_box(&obj);
                // reduce memory contention noise
                std::hint::spin_loop();
                pool.put(obj);
            })
        });
    }
    
    group.finish();
}

fn benchmark_memory_patterns(c: &mut Criterion) {
    calibrate_tsc_frequency();
    let mut group = c.benchmark_group("memory_patterns");
    
    // allocation pattern
    group.bench_function("sequential_alloc", |b| {
        b.iter(|| {
            let mut vecs = Vec::new();
            for i in 0..100 {
                vecs.push(vec![0u8; i * 8]);
            }
            drop(vecs);
        })
    });
    
    let pool = ObjectPool::<Vec<u8>>::new();
    group.bench_function("pool_sequential_alloc", |b| {
        b.iter(|| {
            let mut objs = Vec::new();
            for i in 0..100 {
                let obj = pool.get(|| vec![0u8; i * 8]);
                objs.push(obj);
            }
            for obj in objs {
                pool.put(obj);
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    memory_benches,
    benchmark_allocators,
    benchmark_pool_sizes,
    benchmark_memory_patterns
);
criterion_main!(memory_benches);