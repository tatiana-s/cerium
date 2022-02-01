use crate::ddlog_interface;
// use convert_variant_derive::EquivDDValue;
use convert_variant_derive::EquivRelId;
// use ddlog_interface::EquivDDValue;
use ddlog_interface::EquivRelId;
// use differential_datalog::ddval::{DDValConvert, DDValue};
use type_checker_ddlog::typedefs::*;
use type_checker_ddlog::Relations;

// Helper enum for representing errors throughout the pipeline.
// TO-DO: add messages for information.
pub enum InternalError {
    ParseError,
    AstBuildError,
    TransformError,
    TypeError,
}

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
        return_type_id: ID,
        arg_ids: Vec<ID>,
        body_id: ID,
    },
    // Statements.
    FunCall {
        id: ID,
        fun_name: String,
        arg_ids: Vec<ID>,
    },
    Assign {
        id: ID,
        var_name: String,
        type_id: ID,
        expr_id: ID,
    },
    Return {
        id: ID,
        expr_id: ID,
    },
    // Items in compound to represent a sequence of statements.
    Compound {
        id: ID,
        start_id: ID,
    },
    Item {
        id: ID,
        stmt_id: ID,
        next_stmt_id: ID,
    },
    EndItem {
        id: ID,
        stmt_id: ID,
    },
    // Expressions.
    BinaryOp {
        id: ID,
        arg1_id: ID,
        arg2_id: ID,
    },
    // Values.
    Var {
        id: ID,
        var_name: String,
    },
    Arg {
        id: ID,
        var_name: String,
        type_id: ID,
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
