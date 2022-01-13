// DDlog imports.
use differential_datalog::api::HDDlog;
use differential_datalog::ddval::{DDValConvert, DDValue};
use differential_datalog::program::{RelId, Update};
use differential_datalog::{DDlog, DDlogDynamic, DDlogInventory, DeltaMap};
use type_checker_ddlog::typedefs::*;
use type_checker_ddlog::Relations;

// General imports.
use std::collections::HashSet;

// Internal imports.
use crate::ast;

pub fn run_ddlog_type_checker(
    hddlog: &HDDlog,
    insert_set: HashSet<ast::AstRelation>,
    delete_set: HashSet<ast::AstRelation>,
) {
    hddlog.transaction_start();
    // TO-DO: Convert set of updates to relations and insert/delete.
    let updates = vec![];
    hddlog.apply_updates(&mut updates.into_iter());
    let mut delta = hddlog.transaction_commit_dump_changes().unwrap();
    dump_delta(&hddlog, &delta);
    // TO-DO: deal with result.
    hddlog.stop().unwrap();
}

// See relation changes (for debugging purposes).
fn dump_delta(ddlog: &HDDlog, delta: &DeltaMap<DDValue>) {
    for (rel, changes) in delta.iter() {
        // TO-DO: check if line below still throws error.
        println!(
            "Changes to relation {}",
            ddlog.inventory.get_index_name(*rel).unwrap()
        );
        for (val, weight) in changes.iter() {
            println!("{} {:+}", val, weight);
        }
    }
}
