// Modules.
pub mod ast;
pub mod ddlog_interface;
pub mod definitions;
pub mod parser_interface;
pub mod standard_type_checker;

pub fn run_standard_type_checker() {
    let ast =
        parser_interface::parse_file_into_ast(&String::from("./tests/dev_examples/c/example2.c"));
    standard_type_checker::type_check(&ast);
}
