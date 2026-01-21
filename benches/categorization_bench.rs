//! Categorization benchmarks.

use criterion::{criterion_group, criterion_main, Criterion};

fn categorization_benchmarks(c: &mut Criterion) {
    c.bench_function("placeholder", |b| {
        b.iter(|| {
            // Placeholder benchmark
            1 + 1
        })
    });
}

criterion_group!(benches, categorization_benchmarks);
criterion_main!(benches);
