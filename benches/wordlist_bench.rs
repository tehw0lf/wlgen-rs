//! Benchmarks for wlgen-rs performance measurement.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::sink;
use wlgen_rs::WordlistGenerator;

/// Benchmark small wordlist generation (4 combinations)
fn bench_small_wordlist(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_wordlist");

    let charsets = vec![b"ab".to_vec(), b"12".to_vec()];
    let keyspace = 4;

    group.throughput(Throughput::Elements(keyspace));

    group.bench_function("iterator", |b| {
        b.iter(|| {
            let gen = WordlistGenerator::new(charsets.clone());
            for word in gen {
                black_box(word);
            }
        });
    });

    group.bench_function("write_to_sink", |b| {
        b.iter(|| {
            let mut gen = WordlistGenerator::new(charsets.clone());
            gen.write_to(sink()).unwrap();
        });
    });

    group.finish();
}

/// Benchmark medium wordlist generation (1,000 combinations)
fn bench_medium_wordlist(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_wordlist");

    let charsets = vec![
        b"abcdefghij".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
    ];
    let keyspace = 1000;

    group.throughput(Throughput::Elements(keyspace));

    group.bench_function("iterator", |b| {
        b.iter(|| {
            let gen = WordlistGenerator::new(charsets.clone());
            for word in gen {
                black_box(word);
            }
        });
    });

    group.bench_function("write_to_sink", |b| {
        b.iter(|| {
            let mut gen = WordlistGenerator::new(charsets.clone());
            gen.write_to(sink()).unwrap();
        });
    });

    group.finish();
}

/// Benchmark large wordlist generation (100,000 combinations)
fn bench_large_wordlist(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_wordlist");

    let charsets = vec![
        b"abcdefghij".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
    ];
    let keyspace = 100_000;

    group.throughput(Throughput::Elements(keyspace));
    group.sample_size(20); // Reduce sample size for long-running benchmarks

    group.bench_function("write_to_sink", |b| {
        b.iter(|| {
            let mut gen = WordlistGenerator::new(charsets.clone());
            gen.write_to(sink()).unwrap();
        });
    });

    group.finish();
}

/// Benchmark very large wordlist generation (1,000,000 combinations)
fn bench_very_large_wordlist(c: &mut Criterion) {
    let mut group = c.benchmark_group("very_large_wordlist");

    let charsets = vec![
        b"abcdefghij".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
        b"0123456789".to_vec(), // 10 chars
    ];
    let keyspace = 1_000_000;

    group.throughput(Throughput::Elements(keyspace));
    group.sample_size(10); // Reduce sample size for very long-running benchmarks

    group.bench_function("write_to_sink", |b| {
        b.iter(|| {
            let mut gen = WordlistGenerator::new(charsets.clone());
            gen.write_to(sink()).unwrap();
        });
    });

    group.finish();
}

/// Benchmark different wordlist sizes to measure scaling
fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    for size in [10, 100, 1000, 10000].iter() {
        let charsets = match *size {
            10 => vec![b"abcdefghij".to_vec()],
            100 => vec![b"abcdefghij".to_vec(), b"0123456789".to_vec()],
            1000 => vec![
                b"abcdefghij".to_vec(),
                b"0123456789".to_vec(),
                b"0123456789".to_vec(),
            ],
            10000 => vec![
                b"abcdefghij".to_vec(),
                b"0123456789".to_vec(),
                b"0123456789".to_vec(),
                b"0123456789".to_vec(),
            ],
            _ => unreachable!(),
        };

        group.throughput(Throughput::Elements(*size));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut gen = WordlistGenerator::new(charsets.clone());
                gen.write_to(sink()).unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark keyspace calculation
fn bench_keyspace(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyspace");

    let charsets = vec![
        b"abcdefghij".to_vec(),
        b"0123456789".to_vec(),
        b"0123456789".to_vec(),
    ];

    group.bench_function("calculate", |b| {
        b.iter(|| {
            let gen = WordlistGenerator::new(charsets.clone());
            black_box(gen.keyspace());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_wordlist,
    bench_medium_wordlist,
    bench_large_wordlist,
    bench_very_large_wordlist,
    bench_scaling,
    bench_keyspace
);

criterion_main!(benches);
