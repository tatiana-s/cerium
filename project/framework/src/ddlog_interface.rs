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
use crate::definitions::AstRelation;

enum UpdateType {
    Insert,
    Delete,
}

pub fn run_ddlog_type_checker(
    hddlog: &HDDlog,
    insert_set: HashSet<AstRelation>,
    delete_set: HashSet<AstRelation>,
) {
    // TO-DO: Handle errors here instead of just unwrapping.
    hddlog.transaction_start().unwrap();
    let updates = vec![];

    hddlog.apply_updates(&mut updates.into_iter()).unwrap();
    let delta = hddlog.transaction_commit_dump_changes().unwrap();
    // Comment/uncomment debug statement before.
    dump_delta(&hddlog, &delta);
    // TO-DO: return result back to user in some way.
    hddlog.stop().unwrap();
}

// Use procedural macros to convert AST relations to equivalent DDlog relations.
// (As they are syntactically almost identical due to direct mapping).

pub trait EquivRelId {
    fn get_equiv_relid(&self) -> Relations;
}

pub trait EquivDDValue {
    fn get_equiv_ddvalue(&self) -> DDValue;
}

/* fn convert_relation(ast_relation: AstRelation, update_type: UpdateType) -> Update<DDValue> {
    match update_type {
        Insert => Update::Insert {
            relid: ast_relation.get_equiv_relid() as RelId,
            v: ast_relation.get_equiv_ddvalue()
        },
        Delete => panic!("Not implemented yet"),
    }
} */

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

#[cfg(test)]
mod tests {
    use crate::ddlog_interface::EquivDDValue;
    use crate::ddlog_interface::EquivRelId;
    use crate::definitions::AstRelation;
    use differential_datalog::ddval::DDValConvert;
    use type_checker_ddlog::typedefs::*;
    use type_checker_ddlog::Relations;

    // Conversion macro tests.
    #[test]
    fn convert_int_to_relid() {
        let int_relation = AstRelation::Int { id: 0 };
        let converted_int_relation = int_relation.get_equiv_relid();
        let expected = Relations::Int;
        assert_eq!(converted_int_relation, expected);
    }

    /*     #[test]
    fn convert_int_to_ddvalue() {
        let int_relation = AstRelation::Int { id: 0 };
        let converted_int_relation = int_relation.get_equiv_ddvalue();
        let expected = Int { id: 0 }.into_ddvalue();
        assert_eq!(converted_int_relation, expected);
    } */
}
