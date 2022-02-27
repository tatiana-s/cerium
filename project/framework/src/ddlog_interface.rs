// DDlog imports.
use differential_datalog::api::HDDlog;
use differential_datalog::ddval::{DDValConvert, DDValue};
use differential_datalog::program::{RelId, Update};
use differential_datalog::{DDlog, DDlogDynamic, DDlogInventory, DeltaMap};
use type_checker_ddlog::typedefs::ddlog_std::Vec as DDlogVec;
use type_checker_ddlog::typedefs::*;
use type_checker_ddlog::Relations;

// General imports.
use std::collections::HashSet;
use std::convert::TryFrom;

// Internal imports.
use crate::definitions::AstRelation;

enum UpdateType {
    InsertUpdate,
    DeleteUpdate,
}

pub fn run_ddlog_type_checker(
    hddlog: &HDDlog,
    insert_set: HashSet<AstRelation>,
    delete_set: HashSet<AstRelation>,
) {
    // TO-DO: Handle errors here instead of just unwrapping.
    hddlog.transaction_start().unwrap();
    let insert_updates = insert_set
        .iter()
        .map(|x| convert_relation(x, UpdateType::InsertUpdate));
    hddlog
        .apply_updates(&mut insert_updates.into_iter())
        .unwrap();
    let delta = hddlog.transaction_commit_dump_changes().unwrap();
    // Comment/uncomment debug statement.
    // dump_delta(&hddlog, &delta);
    // TO-DO: return result back to user in some way.
    hddlog.stop().unwrap();
}

// Use a procedural macro to convert AST relations to equivalent DDlog relations.
// (As they are syntactically almost identical due to direct mapping).
pub trait EquivRelId {
    fn get_equiv_relid(&self) -> Relations;
}

fn convert_relation(ast_relation: &AstRelation, update_type: UpdateType) -> Update<DDValue> {
    match update_type {
        InsertUpdate => Update::Insert {
            relid: ast_relation.get_equiv_relid() as RelId,
            v: get_equiv_ddvalue(ast_relation),
        },
        DeleteUpdate => panic!("Not implemented yet"),
    }
}

// See relation changes (for debugging purposes).
fn dump_delta(ddlog: &HDDlog, delta: &DeltaMap<DDValue>) {
    for (rel, changes) in delta.iter() {
        for (val, weight) in changes.iter() {
            println!("{} {:+}", val, weight);
        }
    }
}

// Need to do some type conversion since we need ID to be usize in Rust for vector access but i32 in the DDlog code.
// (TO-DO: maybe automate this as a macro as)
fn get_equiv_ddvalue(ast_relation: &AstRelation) -> DDValue {
    match ast_relation.clone() {
        AstRelation::TransUnit { id, body_ids } => {
            let converted_id = i32::try_from(id).unwrap();
            let mut converted_body_ids: DDlogVec<i32> = DDlogVec::new();
            for vec_id in body_ids {
                let converted_vec_id = i32::try_from(vec_id).unwrap();
                converted_body_ids.push(converted_vec_id);
            }
            TransUnit {
                id: converted_id,
                body_ids: converted_body_ids,
            }
            .into_ddvalue()
        }
        AstRelation::FunDef {
            id,
            fun_name,
            return_type_id,
            arg_ids,
            body_id,
        } => {
            let converted_id = i32::try_from(id).unwrap();
            let converted_return_id = i32::try_from(return_type_id).unwrap();
            let mut converted_arg_ids: DDlogVec<i32> = DDlogVec::new();
            for vec_id in arg_ids {
                let converted_vec_id = i32::try_from(vec_id).unwrap();
                converted_arg_ids.push(converted_vec_id);
            }
            let converted_body_id = i32::try_from(body_id).unwrap();
            FunDef {
                id: converted_id,
                fun_name,
                return_type_id: converted_return_id,
                arg_ids: converted_arg_ids,
                body_id: converted_body_id,
            }
            .into_ddvalue()
        }
        AstRelation::FunCall {
            id,
            fun_name,
            arg_ids,
        } => {
            let converted_id = i32::try_from(id).unwrap();
            let mut converted_arg_ids: DDlogVec<i32> = DDlogVec::new();
            for vec_id in arg_ids {
                let converted_vec_id = i32::try_from(vec_id).unwrap();
                converted_arg_ids.push(converted_vec_id);
            }
            FunCall {
                id: converted_id,
                fun_name,
                arg_ids: converted_arg_ids,
            }
            .into_ddvalue()
        }
        AstRelation::Assign {
            id,
            var_name,
            type_id,
            expr_id,
        } => {
            let converted_id = i32::try_from(id).unwrap();
            let converted_type_id = i32::try_from(type_id).unwrap();
            let converted_expr_id = i32::try_from(expr_id).unwrap();
            Assign {
                id: converted_id,
                var_name,
                type_id: converted_type_id,
                expr_id: converted_expr_id,
            }
            .into_ddvalue()
        }
        AstRelation::Return { id, expr_id } => {
            let converted_id = i32::try_from(id).unwrap();
            let converted_expr_id = i32::try_from(expr_id).unwrap();
            Return {
                id: converted_id,
                expr_id: converted_expr_id,
            }
            .into_ddvalue()
        }
        AstRelation::Compound { id, start_id } => {
            let converted_id = i32::try_from(id).unwrap();
            let converted_start_id = i32::try_from(start_id).unwrap();
            Compound {
                id: converted_id,
                start_id: converted_start_id,
            }
            .into_ddvalue()
        }
        AstRelation::Item {
            id,
            stmt_id,
            next_stmt_id,
        } => {
            let converted_id = i32::try_from(id).unwrap();
            let converted_stmt_id = i32::try_from(stmt_id).unwrap();
            let converted_next_id = i32::try_from(next_stmt_id).unwrap();
            Item {
                id: converted_id,
                stmt_id: converted_stmt_id,
                next_stmt_id: converted_next_id,
            }
            .into_ddvalue()
        }
        AstRelation::EndItem { id, stmt_id } => {
            let converted_id = i32::try_from(id).unwrap();
            let converted_stmt_id = i32::try_from(stmt_id).unwrap();
            EndItem {
                id: converted_id,
                stmt_id: converted_stmt_id,
            }
            .into_ddvalue()
        }
        AstRelation::BinaryOp {
            id,
            arg1_id,
            arg2_id,
        } => {
            let converted_id = i32::try_from(id).unwrap();
            let converted_arg1_id = i32::try_from(arg1_id).unwrap();
            let converted_arg2_id = i32::try_from(arg2_id).unwrap();
            BinaryOp {
                id: converted_id,
                arg1_id: converted_arg1_id,
                arg2_id: converted_arg2_id,
            }
            .into_ddvalue()
        }
        AstRelation::Var { id, var_name } => {
            let converted_id = i32::try_from(id).unwrap();
            Var {
                id: converted_id,
                var_name,
            }
            .into_ddvalue()
        }
        AstRelation::Arg {
            id,
            var_name,
            type_id,
        } => {
            let converted_id = i32::try_from(id).unwrap();
            let converted_type_id = i32::try_from(type_id).unwrap();
            Arg {
                id: converted_id,
                var_name,
                type_id: converted_type_id,
            }
            .into_ddvalue()
        }
        AstRelation::Void { id } => {
            let converted_id = i32::try_from(id).unwrap();
            Void { id: converted_id }.into_ddvalue()
        }
        AstRelation::Int { id } => {
            let converted_id = i32::try_from(id).unwrap();
            Int { id: converted_id }.into_ddvalue()
        }
        AstRelation::Float { id } => {
            let converted_id = i32::try_from(id).unwrap();
            Void { id: converted_id }.into_ddvalue()
        }
        AstRelation::Char { id } => {
            let converted_id = i32::try_from(id).unwrap();
            Char { id: converted_id }.into_ddvalue()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ddlog_interface::get_equiv_ddvalue;
    use crate::ddlog_interface::EquivRelId;
    use crate::definitions::AstRelation;
    use differential_datalog::ddval::DDValConvert;
    use type_checker_ddlog::typedefs::ddlog_std::Vec as DDlogVec;
    use type_checker_ddlog::typedefs::*;
    use type_checker_ddlog::Relations;

    // Conversion macro test.
    #[test]
    fn convert_int_to_relid() {
        let int_relation = AstRelation::Int { id: 0 };
        let converted_int_relation = int_relation.get_equiv_relid();
        let expected = Relations::Int;
        assert_eq!(converted_int_relation, expected);
    }

    // Manual type conversion test.
    #[test]
    fn convert_fundef_to_ddvalue() {
        let id: usize = 0;
        let fundef_relation = AstRelation::FunDef {
            id: id,
            fun_name: String::from("function"),
            return_type_id: 0,
            arg_ids: vec![1, 2, 3],
            body_id: 0,
        };
        let converted_int_relation = get_equiv_ddvalue(&fundef_relation);
        let mut expected_arg_ids: DDlogVec<i32> = DDlogVec::new();
        expected_arg_ids.push(1);
        expected_arg_ids.push(2);
        expected_arg_ids.push(3);
        let expected_id: i32 = 0;
        let expected = FunDef {
            id: expected_id,
            fun_name: String::from("function"),
            return_type_id: 0,
            arg_ids: expected_arg_ids,
            body_id: 0,
        }
        .into_ddvalue();
        assert_eq!(converted_int_relation, expected);
    }
}
