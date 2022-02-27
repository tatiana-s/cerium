extern crate notify;

// General imports.
use std::collections::HashSet;
use std::env;

// Imports for notify-rs.
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

// DDlog imports.
use differential_datalog::api::HDDlog;

// Modules.
pub mod ast;
pub mod ddlog_interface;
pub mod definitions;
pub mod parser_interface;

fn main() {
    // Read command line arguments.
    // Arguments can't contain invalid unicode characters.
    // TO-DO: support for argument options.
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    // Create instance of the DDlog type checking program.
    let (hddlog, _) = type_checker_ddlog::run(1, false).unwrap();

    // Type check initial input file.
    let ast = parser_interface::parse_file_into_ast(file_path);
    // ast.pretty_print();
    let insert_set: HashSet<definitions::AstRelation> = ast::get_initial_relation_set(&ast);
    let delete_set: HashSet<definitions::AstRelation> = HashSet::new();
    ddlog_interface::run_ddlog_type_checker(&hddlog, insert_set, delete_set);

    // Continue watching the file for changes.
    // TO-DO: add support for type-checking directories.
    if let Err(e) = watch_for_write(file_path, &ast, hddlog) {
        println!("error: {:?}", e)
    }
}

// Watches file for writes and passes it off to the parser in the event of one.
fn watch_for_write(
    file_path: &String,
    initial_ast: &ast::Tree,
    hddlog: HDDlog,
) -> notify::Result<()> {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    let mut watcher: RecommendedWatcher = watcher(tx, Duration::from_secs(1)).unwrap();

    // Add the path to be watched.
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();

    let mut prev_ast = initial_ast.clone();
    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(ref _path) => {
                    // Check file on any completed write.
                    // Type check initial input file.
                    let ast = parser_interface::parse_file_into_ast(file_path);
                    let (insert_set, delete_set) = ast::get_diff_relation_set(&ast, &prev_ast);
                    ddlog_interface::run_ddlog_type_checker(&hddlog, insert_set, delete_set);
                    prev_ast = ast.clone()
                }
                _ => {}
            },
            Err(e) => println!("error: {:?}", e),
        }
    }
}
