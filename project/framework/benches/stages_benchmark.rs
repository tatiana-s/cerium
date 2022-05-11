// External imports.
use std::collections::HashSet;

// Internal imports.
use cerium_framework::ast;
use cerium_framework::compute_diff;
use cerium_framework::ddlog_interface;
use cerium_framework::parse_into_relation_tree;
use cerium_framework::standard_type_check_without_parse;
use criterion::{criterion_group, criterion_main, Criterion};

// Just time actual type checking computation without any of the rest.
pub fn criterion_benchmark(c: &mut Criterion) {
    // We will separately set up and run the benchmarks in order to have to not deal with passing inputs.
    let (hddlog, _) = type_checker_ddlog::run(1, false).unwrap();
    let mut group = c.benchmark_group("Stage Timing");
    group.bench_function("DDlog setup", |b| {
        b.iter(|| {
            type_checker_ddlog::run(1, false).unwrap();
        })
    });
    let initial_ast = parse_into_relation_tree(String::from(
        "./benches/dataset/program2/4_program2_original.c",
    ));
    group.bench_function("Initial parse", |b| {
        b.iter(|| {
            parse_into_relation_tree(String::from(
                "./benches/dataset/program2/4_program2_original.c",
            ));
        })
    });
    let initial_insertions = ast::get_initial_relation_set(&initial_ast);
    group.bench_function("Extract insertion set", |b| {
        b.iter(|| {
            ast::get_initial_relation_set(&initial_ast);
        })
    });
    group.bench_function("Initial datalog type check", |b| {
        b.iter(|| {
            ddlog_interface::run_ddlog_type_checker(
                &hddlog,
                initial_insertions.clone(),
                HashSet::new(),
                false,
                true,
            );
        })
    });
    group.bench_function("Standard type check", |b| {
        b.iter(|| {
            standard_type_check_without_parse(initial_ast.clone());
        })
    });
    let modified_ast = parse_into_relation_tree(String::from(
        "./benches/dataset/program2/4_program2_change.c",
    ));
    group.bench_function("Modified parse", |b| {
        b.iter(|| {
            parse_into_relation_tree(String::from(
                "./benches/dataset/program2/4_program2_change.c",
            ));
        })
    });
    let (insertion_set, deletion_set, _) = compute_diff(initial_ast.clone(), modified_ast.clone());
    group.bench_function("Compute program delta", |b| {
        b.iter(|| {
            compute_diff(initial_ast.clone(), modified_ast.clone());
        })
    });
    group.bench_function("Modified datalog type check", |b| {
        b.iter(|| {
            ddlog_interface::run_ddlog_type_checker(
                &hddlog,
                insertion_set.clone(),
                deletion_set.clone(),
                false,
                true,
            );
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
