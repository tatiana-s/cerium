use crate::definitions::{AstNodeKind, AstRelation, ID};
use std::collections::{HashMap, HashSet};

// For storing information about node location (will be useful for error reporting).
#[derive(Debug, Clone, Copy)]
struct Location {}

// Main tree representing program that we will maintain throughout runtime.
// For simplicity make the whole tree have the same lifetime (arena allocation).
#[derive(Debug, Clone)]
pub struct RelationTree {
    arena: HashMap<ID, RelationNode>,
    max_id: ID,
    root_id: ID,
}

impl RelationTree {
    pub fn new() -> Self {
        Self {
            arena: HashMap::new(),
            max_id: 0,
            root_id: 0,
        }
    }

    pub fn get_node(&mut self, index: ID) -> RelationNode {
        let result = self.arena.get(&index);
        match result {
            Some(node) => node.clone(),
            None => panic!("No node with this ID in tree"),
        }
    }

    pub fn get_relation(&mut self, index: ID) -> AstRelation {
        let result = self.arena.get(&index);
        match result {
            Some(node) => node.relation.clone(),
            None => panic!("No relation with this ID in tree"),
        }
    }

    pub fn add_node(&mut self, node_id: ID, node_kind: AstNodeKind, relation: AstRelation) {
        self.arena
            .insert(node_id, RelationNode::new(node_id, node_kind, relation));
        if node_id > self.max_id {
            self.max_id = node_id;
        }
    }

    pub fn add_root_node(&mut self, node_id: ID, node_kind: AstNodeKind, relation: AstRelation) {
        self.arena
            .insert(node_id, RelationNode::new(node_id, node_kind, relation));
        self.root_id = node_id;
        if node_id > self.max_id {
            self.max_id = node_id;
        }
    }

    pub fn link_child(&mut self, node_id: ID, child_id: ID) {
        if self.arena.contains_key(&node_id) && self.arena.contains_key(&child_id) {
            let node = self.arena.get(&node_id).unwrap();
            node.clone().link_child(child_id);
            self.arena.insert(node_id, node.clone());
        }
    }

    pub fn replace_children(&mut self, node_id: ID, child_ids: Vec<ID>) {
        if self.arena.contains_key(&node_id) {
            self.arena
                .get(&node_id)
                .unwrap()
                .replace_children(child_ids);
        }
    }

    pub fn size(&mut self) -> usize {
        self.arena.len()
    }

    pub fn pretty_print(&mut self) {
        self.arena
            .get(&self.root_id)
            .unwrap()
            .pretty_print(&String::from(""), &self.arena);
    }

    pub fn get_root(&mut self) -> ID {
        self.root_id
    }
}

// Building block of AST.
#[derive(Debug, Clone)]
pub struct RelationNode {
    node_id: ID,
    relation: AstRelation,
    node_kind: AstNodeKind,
    location: Location,
    children: Vec<ID>,
}

impl RelationNode {
    fn new(node_id: ID, node_kind: AstNodeKind, relation: AstRelation) -> Self {
        Self {
            node_id,
            relation,
            node_kind,
            location: Location {},
            children: Vec::new(),
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

    fn pretty_print(&self, indent: &String, arena: &HashMap<ID, RelationNode>) {
        println!("{}{:?}", indent, self.relation);
        let new_indent = format!("{}{}", indent, "   ");
        for child_id in &self.children {
            arena
                .get(child_id)
                .unwrap()
                .pretty_print(&new_indent, arena)
        }
    }
}

// More minimal tree we will construct on every pass after the initial pass.
// Explicitly not using IDs here to distinguish from indices for access and the ones used in relations.
// We also don't need to be able to modify it since it will only be constructed once.
#[derive(Debug, Clone)]
pub struct UpdateTree {
    arena: HashMap<usize, UpdateNode>,
    root_key: usize,
}

impl UpdateTree {
    pub fn new() -> Self {
        Self {
            arena: HashMap::new(),
            root_key: 0,
        }
    }

    pub fn get_node(&self, key: usize) -> UpdateNode {
        let result = self.arena.get(&key);
        match result {
            Some(node) => node.clone(),
            None => panic!("No node with this ID in tree"),
        }
    }

    pub fn add_node(&mut self, key: usize, node_kind: AstNodeKind) {
        self.arena.insert(key, UpdateNode::new(node_kind));
    }

    pub fn add_root_node(&mut self, key: usize, node_kind: AstNodeKind) {
        self.arena.insert(key, UpdateNode::new(node_kind));
        self.root_key = key;
    }

    pub fn link_child(&mut self, node_key: usize, child_key: usize) {
        if self.arena.contains_key(&node_key) && self.arena.contains_key(&node_key) {
            self.arena.get(&node_key).unwrap().link_child(child_key);
        }
    }

    pub fn replace_children(&mut self, node_key: usize, child_keys: Vec<usize>) {
        if self.arena.contains_key(&node_key) {
            self.arena
                .get(&node_key)
                .unwrap()
                .replace_children(child_keys);
        }
    }

    pub fn size(&self) -> usize {
        self.arena.len()
    }

    pub fn pretty_print(&self) {
        self.arena
            .get(&self.root_key)
            .unwrap()
            .pretty_print(&String::from(""), &self.arena);
    }
}

// Building block of AST.
#[derive(Debug, Clone)]
pub struct UpdateNode {
    node_kind: AstNodeKind,
    location: Location,
    children: Vec<usize>,
}

impl UpdateNode {
    fn new(node_kind: AstNodeKind) -> Self {
        Self {
            node_kind,
            location: Location {},
            children: Vec::new(),
        }
    }

    fn link_child(&mut self, child_key: usize) {
        self.children.push(child_key);
    }

    fn replace_children(&mut self, child_keys: Vec<usize>) {
        self.children = child_keys;
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    fn pretty_print(&self, indent: &String, arena: &HashMap<usize, UpdateNode>) {
        println!("{}{:?}", indent, self.node_kind);
        let new_indent = format!("{}{}", indent, "   ");
        for child_key in &self.children {
            arena
                .get(child_key)
                .unwrap()
                .pretty_print(&new_indent, arena)
        }
    }
}

// Flattens AST and converts into a set of relations.
pub fn get_initial_relation_set(ast: &RelationTree) -> HashSet<AstRelation> {
    let mut relation_set: HashSet<AstRelation> = HashSet::new();
    for node in ast.clone().arena {
        relation_set.insert(node.1.relation);
    }
    relation_set
}

// Finds the differences between the to ASTs with structural differencing and flattens.
// Returns separate sets for relations that need to be deleted and relations that are inserted.
// Here IDs are allocated in a way that unchanged nodes retain their previous IDs.
pub fn get_diff_relation_set(
    prev_ast: &RelationTree,
    new_ast: &UpdateTree,
) -> (HashSet<AstRelation>, HashSet<AstRelation>, RelationTree) {
    (HashSet::new(), HashSet::new(), *prev_ast)
}

#[cfg(test)]
mod tests {}
