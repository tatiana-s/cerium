use cerium_framework::run_incremental_type_checker_once;
use cerium_framework::run_standard_type_checker_once;
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Example 2 Initial Run");
    group.bench_function("Standard", |b| b.iter(|| run_standard_type_checker_once()));
    group.bench_function("Incremental", |b| {
        b.iter(|| run_incremental_type_checker_once())
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
