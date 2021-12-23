extern crate notify;

use std::env;
use std::fs;

// Imports for notify-rs.
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

// Modules.
mod ddlog_utils;
mod parser;
mod tree_lib;

fn main() {
    // Read command line arguments.
    // Arguments can't contain invalid unicode characters.
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    // Check initial input file.
    type_check_file(file_path);

    // Continue watching the file for changes.
    if let Err(e) = watch_for_write(file_path) {
        println!("error: {:?}", e)
    }
}

// Watches file for writes and passes it off to the parser in the event of one.
fn watch_for_write(file_path: &String) -> notify::Result<()> {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    let mut watcher: RecommendedWatcher = watcher(tx, Duration::from_secs(1)).unwrap();

    // Add the path to be watched.
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(ref _path) => {
                    // Check file on any completed write.
                    type_check_file(file_path);
                }
                _ => {}
            },
            Err(e) => println!("error: {:?}", e),
        }
    }
}

fn type_check_file(file_path: &String) {
    // Read input file.
    let file_contents = fs::read_to_string(file_path).expect("File couldn't be read");
    println!("{}", file_contents);
    // Parse input.
    parser::parse_input_to_sexp(&file_contents);
}
