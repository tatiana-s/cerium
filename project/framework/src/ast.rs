use crate::definitions::{AstRelation, InternalError, ID};
use crate::parser_interface;
use std::collections::HashSet;

// Building block of AST.
// -> ID for flattening into Datalog relations.
// -> definitions::AstRelation for kind of AST node.
// -> Location for location in input file.
// -> Children for link to other nodes in AST.
pub struct AstNode {
    node_id: ID,
    relation: AstRelation,
    location: Location,
    children: Vec<AstNode>,
}

// Helper struct for storing information about node location (will be useful for error reporting).
// TO-DO: define this struct.
pub struct Location {}

impl AstNode {
    pub fn new(relation: AstRelation) -> AstNode {
        AstNode {
            node_id: 0,
            relation: relation,
            location: Location {},
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, relation: AstRelation) {
        self.children.push(AstNode::new(relation));
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    pub fn pretty_print(&self) {
        println!("{:#?}", self.relation)
    }
}

pub fn parse_file_into_ast(file_path: &String) -> Result<AstNode, InternalError> {
    parser_interface::parse_with_lang_c(file_path)
}

// Flattens AST by allocating IDs (and linking through them).
// Then converts into a set of nodes (which are at this point equivalent to relations).
pub fn get_initial_relation_set(ast: &AstNode) -> HashSet<AstRelation> {
    initial_id_allocation();
    HashSet::new()
}

// Enumerate all nodes.
fn initial_id_allocation() {}

// Finds the differences between the to ASTs and flattens.
// Returns separate sets for relations that need to be deleted and relations that are inserted.
// Here IDs are allocated in a way that unchanged nodes retain their previous IDs.
pub fn get_diff_relation_set(
    ast: &AstNode,
    prev_ast: &AstNode,
) -> (HashSet<AstRelation>, HashSet<AstRelation>) {
    compute_tree_diff();
    (HashSet::new(), HashSet::new())
}

// Structural differencing.
fn compute_tree_diff() {}

// TO-DO: unit and integration testing...
// #[cfg(test)]
// mod tests {}
