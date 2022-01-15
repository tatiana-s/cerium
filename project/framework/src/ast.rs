extern crate clang;
// extern crate strum;
// extern crate strum_macros;

use clang::*;
use std::collections::HashSet;
// use strum_macros::{EnumDiscriminants};
use crate::definitions;

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
// -> definitions::AstRelation for kind of AST node.
// -> Location for location in input file.
// -> Children for link to other nodes in AST.
pub struct AstNode {
    node_id: definitions::ID,
    node_kind: definitions::AstRelation,
    location: Location,
    children: Vec<AstNode>,
}

impl AstNode {
    pub fn new(node_kind: definitions::AstRelation, location: Location) -> AstNode {
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
            _ => return Some(AstNode::new(node_kind, location)),
        }
    }

    fn add_child(&mut self, node_type: definitions::AstRelation, location: Location) {
        self.children.push(AstNode::new(node_type, location));
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    pub fn pretty_print(&self) {
        println!("{:#?}", self.node_kind)
    }
}

fn get_node_kind(clang_entity: Entity) -> definitions::AstRelation {
    let clang_kind = clang_entity.get_kind();
    match clang_kind {
        EntityKind::TranslationUnit => definitions::AstRelation::TransUnit {
            id: 0,
            body_ids: vec![],
        },
        _ => panic!("Need to change this"),
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
pub fn get_initial_relation_set(ast: &AstNode) -> HashSet<definitions::AstRelation> {
    initial_id_allocation();
    HashSet::new()
}

fn compute_tree_diff() {}

// Finds the differences between the to ASTs and flattens.
// Here IDs are allocated in a way that unchanged nodes retain their previous IDs.
pub fn get_diff_relation_set(
    ast: &AstNode,
    prev_ast: &AstNode,
) -> (
    HashSet<definitions::AstRelation>,
    HashSet<definitions::AstRelation>,
) {
    compute_tree_diff();
    (HashSet::new(), HashSet::new())
}

// TO-DO: unit and integration testing...
// #[cfg(test)]
// mod tests {}
