// External imports.
use differential_datalog::api::HDDlog;
use std::collections::HashSet;
use std::fmt;

// Internal imports.
use cerium_framework::ast;
use cerium_framework::compute_diff;
use cerium_framework::ddlog_interface;
use cerium_framework::definitions;
use cerium_framework::parse_into_relation_tree;
use cerium_framework::single_datalog_type_check;
use cerium_framework::standard_type_check_without_parse;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn set_up_datalog() -> IncrementalInput {
    // Create instance of the DDlog type checking program.
    let (hddlog, _) = type_checker_ddlog::run(1, false).unwrap();
    // Run initial type checking run.
    let (initial_result, initial_ast) = single_datalog_type_check(String::from(
        "./benches/dataset/program1/0_program1_original.c",
    ));
    // Parse modified file.
    let modified_ast = parse_into_relation_tree(String::from(
        "./benches/dataset/program1/0_program1_original.c",
    ));
    // Compute program delta.
    let (insertion_set, deletion_set, _) = compute_diff(initial_ast, modified_ast);
    return IncrementalInput::new(initial_result, hddlog, insertion_set, deletion_set);
}

pub fn set_up_standard() -> ast::Tree {
    // For standard type checker we can just immediately parse the modified file here.
    return parse_into_relation_tree(String::from(
        "./benches/dataset/program1/0_program1_original.c",
    ));
}
#[derive(Debug)]
pub struct IncrementalInput {
    result: bool,
    hddlog: HDDlog,
    insertion_set: HashSet<definitions::AstRelation>,
    deletion_set: HashSet<definitions::AstRelation>,
}

impl IncrementalInput {
    pub fn new(
        result: bool,
        hddlog: HDDlog,
        insertion_set: HashSet<definitions::AstRelation>,
        deletion_set: HashSet<definitions::AstRelation>,
    ) -> Self {
        Self {
            result,
            hddlog,
            insertion_set,
            deletion_set,
        }
    }
}

impl fmt::Display for IncrementalInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Just time actual type checking computation without any of the rest.
pub fn criterion_benchmark(c: &mut Criterion) {
    // Set up before running benchmarks.
    // Contains result, hddlog instance, insertion set, deletion set.
    let datalog_input = set_up_datalog();
    // Contains just parsed AST.
    let standard_input = set_up_standard();
    let mut group = c.benchmark_group("Program 2 - Incremental Change 1");
    group.bench_with_input(
        BenchmarkId::new("Standard", standard_input.clone()),
        &standard_input,
        |b, s| {
            b.iter(|| {
                standard_type_check_without_parse(s.clone());
            });
        },
    );
    group.bench_function("Incremental", |b| {
        b.iter(|| {
            ddlog_interface::run_ddlog_type_checker(
                &datalog_input.hddlog,
                datalog_input.insertion_set.clone(),
                datalog_input.deletion_set.clone(),
                datalog_input.result,
                true,
            );
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
