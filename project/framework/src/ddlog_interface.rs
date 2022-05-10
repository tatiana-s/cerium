// DDlog imports.
use differential_datalog::api::HDDlog;
use differential_datalog::ddval::{DDValConvert, DDValue};
use differential_datalog::program::{RelId, Update};
use differential_datalog::{DDlog, DDlogDynamic, DeltaMap};
use type_checker_ddlog::typedefs::ddlog_std::Vec as DDlogVec;
use type_checker_ddlog::typedefs::*;
use type_checker_ddlog::Relations;

// General imports.
use std::collections::HashSet;

// Internal imports.
use crate::definitions::AstRelation;

enum UpdateKind {
    InsertUpdate,
    DeleteUpdate,
}

pub fn run_ddlog_type_checker(
    hddlog: &HDDlog,
    insert_set: HashSet<AstRelation>,
    delete_set: HashSet<AstRelation>,
    prev_result: bool,
    disable_output: bool,
) -> bool {
    println!("{:?}", insert_set);
    println!("{:?}", delete_set);
    // Start transaction.
    hddlog.transaction_start().unwrap();
    // Updates.
    let delete_updates = delete_set
        .iter()
        .map(|x| convert_relation(x, UpdateKind::DeleteUpdate));
    hddlog
        .apply_updates(&mut delete_updates.into_iter())
        .unwrap();
    let insert_updates = insert_set
        .iter()
        .map(|x| convert_relation(x, UpdateKind::InsertUpdate));
    hddlog
        .apply_updates(&mut insert_updates.into_iter())
        .unwrap();
    // See result.
    // Comment/uncomment dump delta debug statement.
    let mut delta = hddlog.transaction_commit_dump_changes().unwrap();
    // dump_delta(&delta);
    let ok_program = delta.get_rel(Relations::OkProgram as RelId);
    let mut new_result = false;
    if !disable_output {
        if prev_result {
            for (_, weight) in ok_program.iter() {
                if *weight == -1 {
                    println!("Program typing error ❌");
                }
            }
            if ok_program.len() == 0 {
                println!("Program correctly typed ✅");
                new_result = true;
            }
        } else {
            for (_, weight) in ok_program.iter() {
                if *weight == 1 {
                    println!("Program correctly typed ✅");
                    new_result = true;
                }
            }
            if ok_program.len() == 0 {
                println!("Program typing error ❌");
            }
        }
    }
    new_result
}

// Use a procedural macro to convert AST relations to equivalent DDlog relations.
// (As they are syntactically almost identical due to direct mapping).
pub trait EquivRelId {
    fn get_equiv_relid(&self) -> Relations;
}

fn convert_relation(ast_relation: &AstRelation, update_type: UpdateKind) -> Update<DDValue> {
    match update_type {
        UpdateKind::InsertUpdate => Update::Insert {
            relid: ast_relation.get_equiv_relid() as RelId,
            v: get_equiv_ddvalue(ast_relation),
        },
        UpdateKind::DeleteUpdate => Update::DeleteValue {
            relid: ast_relation.get_equiv_relid() as RelId,
            v: get_equiv_ddvalue(ast_relation),
        },
    }
}

// See relation changes (for debugging purposes).
#[allow(dead_code)]
fn dump_delta(delta: &DeltaMap<DDValue>) {
    for (_, changes) in delta.iter() {
        for (val, weight) in changes.iter() {
            println!("{} {:+}", val, weight);
        }
    }
}

// Need to do some type conversion to convert to DDlog vectors and relations.
// (TO-DO: maybe automate this as a macro?)
fn get_equiv_ddvalue(ast_relation: &AstRelation) -> DDValue {
    match ast_relation.clone() {
        AstRelation::TransUnit { id, body_ids } => {
            let mut converted_body_ids: DDlogVec<i32> = DDlogVec::new();
            for vec_id in body_ids {
                converted_body_ids.push(vec_id);
            }
            TransUnit {
                id,
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
            let mut converted_arg_ids: DDlogVec<i32> = DDlogVec::new();
            for vec_id in arg_ids {
                converted_arg_ids.push(vec_id);
            }
            FunDef {
                id,
                fun_name,
                return_type_id,
                arg_ids: converted_arg_ids,
                body_id,
            }
            .into_ddvalue()
        }
        AstRelation::FunCall {
            id,
            fun_name,
            arg_ids,
        } => {
            let mut converted_arg_ids: DDlogVec<i32> = DDlogVec::new();
            for vec_id in arg_ids {
                converted_arg_ids.push(vec_id);
            }
            FunCall {
                id,
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
        } => Assign {
            id,
            var_name,
            type_id,
            expr_id,
        }
        .into_ddvalue(),
        AstRelation::Return { id, expr_id } => Return { id, expr_id }.into_ddvalue(),
        AstRelation::If {
            id,
            cond_id,
            then_id,
        } => If {
            id,
            cond_id,
            then_id,
        }
        .into_ddvalue(),
        AstRelation::IfElse {
            id,
            cond_id,
            then_id,
            else_id,
        } => IfElse {
            id,
            cond_id,
            then_id,
            else_id,
        }
        .into_ddvalue(),
        AstRelation::While {
            id,
            cond_id,
            body_id,
        } => While {
            id,
            cond_id,
            body_id,
        }
        .into_ddvalue(),
        AstRelation::Compound { id, start_id } => Compound { id, start_id }.into_ddvalue(),
        AstRelation::Item {
            id,
            stmt_id,
            next_stmt_id,
        } => Item {
            id,
            stmt_id,
            next_stmt_id,
        }
        .into_ddvalue(),
        AstRelation::EndItem { id, stmt_id } => EndItem { id, stmt_id }.into_ddvalue(),
        AstRelation::BinaryOp {
            id,
            arg1_id,
            arg2_id,
        } => BinaryOp {
            id,
            arg1_id,
            arg2_id,
        }
        .into_ddvalue(),
        AstRelation::Var { id, var_name } => Var { id, var_name }.into_ddvalue(),
        AstRelation::Arg {
            id,
            var_name,
            type_id,
        } => Arg {
            id,
            var_name,
            type_id,
        }
        .into_ddvalue(),
        AstRelation::Void { id } => Void { id }.into_ddvalue(),
        AstRelation::Int { id } => Int { id }.into_ddvalue(),
        AstRelation::Float { id } => Float { id }.into_ddvalue(),
        AstRelation::Char { id } => Char { id }.into_ddvalue(),
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
        let id: ID = 0;
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
