// Modules.
pub mod ast;
pub mod ddlog_interface;
pub mod definitions;
pub mod parser_interface;
pub mod standard_type_checker;

// General imports.
use std::collections::HashSet;

// Imports for notify-rs.
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

// DDlog imports.
use differential_datalog::api::HDDlog;

// Type-check a file once with the non-incremental type checker.
pub fn single_type_check_standard(file_path: String) -> bool {
    let ast = parser_interface::parse_file_into_ast(&file_path);
    return standard_type_checker::type_check(&ast);
}

pub fn repeated_type_check_standard(file_path: &String) -> notify::Result<()> {
    // Create a channel to receive the events.
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = watcher(tx, Duration::from_secs(1)).unwrap();
    // Add the path to be watched.
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();
    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(ref _path) => {
                    // Check file on any completed write.
                    let result = single_type_check_standard(file_path.clone());
                    if result {
                        println!("Program correctly typed ✅");
                    } else {
                        println!("Program typing error ❌");
                    }
                }
                _ => {}
            },
            Err(e) => println!("error: {:?}", e),
        }
    }
}

// Type-check a file once with the incremental type checker.
pub fn single_type_check_datalog(file_path: String) {
    let (hddlog, _) = type_checker_ddlog::run(1, false).unwrap();
    let ast = parser_interface::parse_file_into_ast(&file_path);
    let insert_set: HashSet<definitions::AstRelation> = ast::get_initial_relation_set(&ast);
    let delete_set: HashSet<definitions::AstRelation> = HashSet::new();
    ddlog_interface::run_ddlog_type_checker(&hddlog, insert_set, delete_set, false);
}

// Keep re-checking file with incremental type checker after each save.
pub fn incremental_type_check(
    file_path: &String,
    initial_ast: &ast::Tree,
    hddlog: HDDlog,
    initial_result: bool,
) -> notify::Result<()> {
    // Create a channel to receive the events.
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = watcher(tx, Duration::from_secs(1)).unwrap();
    // Add the path to be watched.
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();
    let mut prev_ast = initial_ast.clone();
    let mut prev_result = initial_result;
    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(ref _path) => {
                    // Check file on any completed write.
                    // Type check initial input file.
                    let ast = parser_interface::parse_file_into_ast(file_path);
                    let (insert_set, delete_set, updated_tree) =
                        ast::get_diff_relation_set(&prev_ast, &ast);
                    let result = ddlog_interface::run_ddlog_type_checker(
                        &hddlog,
                        insert_set,
                        delete_set,
                        prev_result,
                    );
                    prev_ast = updated_tree.clone();
                    prev_result = result;
                }
                _ => {}
            },
            Err(e) => println!("error: {:?}", e),
        }
    }
}

// Find the program delta between two ASTs (mainly for benchmark tests).
pub fn compute_diff(t1: ast::Tree, t2: ast::Tree) {
    ast::get_diff_relation_set(&t1, &t2);
}

// Insert given relations into given DDlog program state (mainly for benchmark tests).
pub fn incremental_type_check_without_diff(
    prev_result: bool,
    hddlog: HDDlog,
    insertion_set: HashSet<definitions::AstRelation>,
    deletion_set: HashSet<definitions::AstRelation>,
) {
    ddlog_interface::run_ddlog_type_checker(&hddlog, insertion_set, deletion_set, prev_result);
}

// Parse file into tree of AST relations (mainly for benchmark tests).
pub fn parse_into_relation_tree(file_path: String) -> ast::Tree {
    let ast = parser_interface::parse_file_into_ast(&file_path);
    return ast;
}
