extern crate notify;

// General imports.
use std::collections::HashSet;
use std::env;

// Internal imports.
use cerium_framework::ast;
use cerium_framework::ddlog_interface;
use cerium_framework::definitions;
use cerium_framework::parser_interface;

fn main() {
    // Read command line arguments.
    // Arguments can't contain invalid unicode characters.
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    // Check if extra option is passed.
    // (Currently just "-s" for standard type checking).
    if args.len() == 3 {
        let option = &args[2];
        if *option == String::from("-s") {
            let initial_result = cerium_framework::single_type_check_standard(file_path.clone());
            if initial_result {
                println!("Program correctly typed ✅");
            } else {
                println!("Program typing error ❌");
            }
            if let Err(e) = cerium_framework::repeated_type_check_standard(file_path) {
                println!("error: {:?}", e)
            }
        }
    }

    // Create instance of the DDlog type checking program.
    let (hddlog, _) = type_checker_ddlog::run(1, false).unwrap();

    // Type check initial input file.
    let ast = parser_interface::parse_file_into_ast(file_path);
    ast.pretty_print();
    let insert_set: HashSet<definitions::AstRelation> = ast::get_initial_relation_set(&ast);
    let delete_set: HashSet<definitions::AstRelation> = HashSet::new();
    let result = ddlog_interface::run_ddlog_type_checker(&hddlog, insert_set, delete_set, false);

    // Continue watching the file for changes.
    // TO-DO: add support for type-checking directories.
    if let Err(e) = cerium_framework::incremental_type_check(file_path, &ast, hddlog, result) {
        println!("error: {:?}", e)
    }
}
