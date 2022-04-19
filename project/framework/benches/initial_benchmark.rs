use cerium_framework::run_standard_type_checker;
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("standard - example 2", |b| {
        b.iter(|| run_standard_type_checker())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
