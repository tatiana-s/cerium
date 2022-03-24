use crate::definitions::{AstRelation, ID};
use std::collections::HashSet;

// For storing information about node location (will be useful for error reporting).
#[derive(Debug, Clone, Copy)]
struct Location {}

// For simplicity make the whole tree have the same lifetime (arena allocation).
#[derive(Debug, Clone)]
pub struct Tree {
    arena: Vec<AstNode>,
}

impl Tree {
    pub fn new() -> Self {
        Self { arena: vec![] }
    }

    pub fn get_relation_at_index(&self, index: ID) -> AstRelation {
        self.arena[index].relation.clone()
    }

    // Add node to arena (unconnected).
    pub fn add_node(&mut self, relation: AstRelation, node_id: ID) {
        self.arena.push(AstNode::new(node_id, relation));
        // println!("{:?}", self.arena);
    }

    pub fn add_root_node(&mut self, relation: AstRelation, node_id: ID) {
        self.arena.push(AstNode::new_root(node_id, relation));
    }

    pub fn link_child(&mut self, node_id: ID, child_id: ID) {
        if self.arena.len() >= node_id && self.arena.len() >= child_id {
            self.arena[node_id].link_child(child_id);
        }
    }

    // TO-DO: check that all children are contained here.
    pub fn replace_children(&mut self, node_id: ID, child_ids: Vec<ID>) {
        if self.arena.len() >= node_id {
            self.arena[node_id].replace_children(child_ids);
        }
    }

    pub fn size(&self) -> usize {
        self.arena.len()
    }

    pub fn pretty_print(&self) {
        let root_index = Self::find_root_index(self);
        self.arena[root_index].pretty_print(&String::from(""), &self.arena);
    }

    pub fn find_root_index(&self) -> usize {
        let mut counter = 0;
        for node in &self.arena {
            if node.is_root {
                return counter;
            }
            counter = counter + 1;
        }
        panic!("Couldn't find root node")
    }
}

// Building block of AST.
#[derive(Debug, Clone)]
pub struct AstNode {
    node_id: ID,
    relation: AstRelation,
    location: Location,
    children: Vec<ID>,
    is_root: bool,
}

impl AstNode {
    fn new(node_id: ID, relation: AstRelation) -> Self {
        Self {
            node_id,
            relation,
            location: Location {},
            children: Vec::new(),
            is_root: false,
        }
    }

    fn new_root(node_id: ID, relation: AstRelation) -> Self {
        Self {
            node_id,
            relation,
            location: Location {},
            children: Vec::new(),
            is_root: true,
        }
    }

    fn link_child(&mut self, child_id: ID) {
        self.children.push(child_id);
    }

    fn replace_children(&mut self, child_ids: Vec<ID>) {
        self.children = child_ids;
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    fn pretty_print(&self, indent: &String, arena: &Vec<AstNode>) {
        println!("{}{:?}", indent, self.relation);
        let new_indent = format!("{}{}", indent, "   ");
        for child_id in &self.children {
            arena[*child_id].pretty_print(&new_indent, arena)
        }
    }
}

// Flattens AST and converts into a set of relations.
pub fn get_initial_relation_set(ast: &Tree) -> HashSet<AstRelation> {
    let mut relation_set: HashSet<AstRelation> = HashSet::new();
    for node in ast.clone().arena {
        relation_set.insert(node.relation);
    }
    relation_set
}

// Finds the differences between the to ASTs with structural differencing and flattens.
// Returns separate sets for relations that need to be deleted and relations that are inserted.
// Here IDs are allocated in a way that unchanged nodes retain their previous IDs.
pub fn get_diff_relation_set(
    prev_ast: &Tree,
    new_ast: &Tree,
) -> (HashSet<AstRelation>, HashSet<AstRelation>) {
    (HashSet::new(), HashSet::new())
}

#[cfg(test)]
mod tests {}
