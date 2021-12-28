extern crate lang_c;
extern crate notify;

// General imports.
use std::env;

// Imports for notify-rs.
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

// Imports for C parser.
use lang_c::driver::{parse, Config};
use lang_c::print::Printer;
use lang_c::visit::Visit;

// Modules.
mod ddlog_utils;
mod tree_utils;

fn main() {
    // Read command line arguments.
    // Arguments can't contain invalid unicode characters.
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    // Check initial input file.
    type_check_file(file_path);

    // Continue watching the file for changes.
    // TO-DO: add support for type-checking directories.
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
    // Create parser.
    // TO-DO: don't create new on each parse.
    let config = Config::default();
    let parse_output = parse(&config, file_path);
    match parse_output {
        Ok(parse) => {
            let s = &mut String::new();
            Printer::new(s).visit_translation_unit(&parse.unit);
            println!("{}", s);
        }
        Err(e) => println!("error: {:?}", e),
    }
}
