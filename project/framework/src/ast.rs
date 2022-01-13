extern crate clang;
// extern crate strum;
// extern crate strum_macros;

use std::collections::HashSet;

use clang::*;
// use strum_macros::{EnumDiscriminants};

// Type aliases for consistency and easy changes.
type ID = u32;

// Defines the permitted language constructs.
#[derive(Debug)]
// #[derive(EnumDiscriminants)]
pub enum AstRelation {
    TranslationUnit {
        id: ID,
        body_ids: Vec<ID>,
    },
    FunctionDef {
        id: ID,
        fun_name: String,
        return_id: ID,
        arg_ids: Vec<ID>,
    },
    FunctionCall {
        id: ID,
        fun_id: ID,
        arg_ids: Vec<ID>,
    },
    // Statements.
    Block {
        ids: Vec<ID>,
    },
    Assignment {
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
    // Unsupported entity kinds.
    NotSupported,
}

// Helper struct for storing information about node location (will be useful for error reporting).
// TO-DO: define this struct.
pub struct Location {}

// Helper enum for representing errors.
pub enum AstError {
    ParseError,
    TransformError,
}

// Building block of AST.
// -> ID for flattening into Datalog relations.
// -> AstRelation for kind of AST node.
// -> Location for location in input file.
// -> Children for link to other nodes in AST.
pub struct AstNode {
    node_id: ID,
    node_kind: AstRelation,
    location: Location,
    children: Vec<AstNode>,
}

impl AstNode {
    pub fn new(node_kind: AstRelation, location: Location) -> AstNode {
        AstNode {
            node_id: 0,
            node_kind: node_kind,
            location: location,
            children: Vec::new(),
        }
    }

    pub fn from_clang_entity(clang_entity: Entity) -> Option<AstNode> {
        // TO-DO: figure out how to actually get location (get_range!).
        // let clang_location = clang_entity.get_location().unwrap().get_file_location();
        let location = Location {};
        let node_kind = get_node_kind(clang_entity);
        match node_kind {
            AstRelation::NotSupported => return None,
            _ => return Some(AstNode::new(node_kind, location)),
        }
    }

    fn add_child(&mut self, node_type: AstRelation, location: Location) {
        self.children.push(AstNode::new(node_type, location));
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    pub fn pretty_print(&self) {
        println!("{:#?}", self.node_kind)
    }
}

fn get_node_kind(clang_entity: Entity) -> AstRelation {
    let clang_kind = clang_entity.get_kind();
    match clang_kind {
        EntityKind::TranslationUnit => AstRelation::TranslationUnit {
            id: 0,
            body_ids: vec![],
        },
        EntityKind::FunctionDecl => AstRelation::FunctionDef {
            id: 0,
            fun_name: String::new(),
            return_id: 0,
            arg_ids: vec![],
        },
        EntityKind::CallExpr => AstRelation::FunctionCall {
            id: 0,
            fun_id: 0,
            arg_ids: vec![],
        },
        _ => AstRelation::NotSupported,
    }
}

fn build_ast(clang_root: Entity) -> AstNode {
    // Assuming it will definitely find a supported root node.
    let root_node = AstNode::from_clang_entity(clang_root).unwrap();
    root_node
}

// Default parse method using clang to parse and then convert into internal AST.
pub fn parse_file_into_ast(file_path: &String) -> Result<AstNode, AstError> {
    // Create parser and parse input file.
    // TO-DO: don't create new on each parse?
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);
    let translation_unit = index.parser(file_path).parse();
    match translation_unit {
        Ok(unit) => {
            return Ok(build_ast(unit.get_entity()));
        }
        Err(e) => {
            println!("parsing error: {:?}", e);
            return Err(AstError::ParseError);
        }
    }
}

fn initial_id_allocation() {}

// Flattens AST by allocating IDs (and linking through them).
// Then converts into a set of nodes (which are at this point equivalent to relations).
pub fn get_initial_relation_set(ast: &AstNode) -> HashSet<AstRelation> {
    initial_id_allocation();
    HashSet::new()
}

fn compute_tree_diff() {}

// Finds the differences between the to ASTs and flattens.
// Here IDs are allocated in a way that unchanged nodes retain their previous IDs.
pub fn get_diff_relation_set(
    ast: &AstNode,
    prev_ast: &AstNode,
) -> (HashSet<AstRelation>, HashSet<AstRelation>) {
    compute_tree_diff();
    (HashSet::new(), HashSet::new())
}

// TO-DO: unit and integration testing...
#[cfg(test)]
mod tests {}
