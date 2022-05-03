use cerium_framework::single_type_check_datalog;
use cerium_framework::single_type_check_standard;
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Example 2 Initial Run");
    group.bench_function("Standard", |b| {
        b.iter(|| single_type_check_standard(String::from("./benches/dataset/program1_1.c")))
    });
    group.bench_function("Incremental", |b| {
        b.iter(|| single_type_check_datalog(String::from("./benches/dataset/program1_1.c")))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
