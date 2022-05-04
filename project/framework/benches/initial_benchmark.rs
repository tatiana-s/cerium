use cerium_framework::single_datalog_type_check;
use cerium_framework::single_standard_type_check;
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Program 1 Initial Run");
    group.bench_function("Standard", |b| {
        b.iter(|| {
            single_standard_type_check(String::from("./benches/dataset/program1/program1_1.c"))
        })
    });
    group.bench_function("Incremental", |b| {
        b.iter(|| {
            single_datalog_type_check(String::from("./benches/dataset/program1/program1_1.c"))
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
