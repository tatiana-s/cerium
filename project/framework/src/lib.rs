// Modules.
pub mod ast;
pub mod ddlog_interface;
pub mod definitions;
pub mod parser_interface;
pub mod standard_type_checker;

// General imports.
use std::collections::HashSet;

pub fn run_standard_type_checker_once() {
    let ast =
        parser_interface::parse_file_into_ast(&String::from("./tests/dev_examples/c/example2.c"));
    standard_type_checker::type_check(&ast);
}

pub fn run_incremental_type_checker_once() {
    let (hddlog, _) = type_checker_ddlog::run(1, false).unwrap();
    let ast =
        parser_interface::parse_file_into_ast(&String::from("./tests/dev_examples/c/example2.c"));
    let insert_set: HashSet<definitions::AstRelation> = ast::get_initial_relation_set(&ast);
    let delete_set: HashSet<definitions::AstRelation> = HashSet::new();
    ddlog_interface::run_ddlog_type_checker(&hddlog, insert_set, delete_set, false);
}
