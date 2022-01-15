use crate::ddlog_interface;
// use convert_variant_derive::EquivDDValue;
use convert_variant_derive::EquivRelId;
// use ddlog_interface::EquivDDValue;
use ddlog_interface::EquivRelId;
// use differential_datalog::ddval::{DDValConvert, DDValue};
use type_checker_ddlog::typedefs::*;
use type_checker_ddlog::Relations;

// Type aliases for consistency and easy changes.
pub type ID = i32;

// Defines the permitted language constructs.
#[derive(Debug, EquivRelId)]
//#[derive(EquivDDValue)]
pub enum AstRelation {
    TransUnit {
        id: ID,
        body_ids: Vec<ID>,
    },
    // Declarations.
    FunDef {
        id: ID,
        fun_name: String,
        return_id: ID,
        arg_ids: Vec<ID>,
    },
    // Statements.
    FunCall {
        id: ID,
        fun_name: String,
        arg_ids: Vec<ID>,
    },
    Compound {
        body_ids: Vec<ID>,
    },
    Assign {
        id: ID,
        var_name: String,
        expr_id: ID,
    },
    Return {
        id: ID,
        expr_id: ID,
    },
    // Expressions.
    BinaryOp {
        id: ID,
        arg1_id: ID,
        ar2_id: ID,
    },
    // Values.
    Var {
        id: ID,
        var_name: String,
    },
    // Leaf types.
    Void {
        id: ID,
    },
    Int {
        id: ID,
    },
    Float {
        id: ID,
    },
    Char {
        id: ID,
    },
}
