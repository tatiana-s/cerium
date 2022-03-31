use crate::definitions::{AstRelation, ID};
use std::collections::{HashMap, HashSet};

// For storing information about node location (will be useful for error reporting).
#[derive(Debug, Clone, Copy)]
struct Location {}

// Main tree representing program that we will maintain throughout runtime.
// For simplicity make the whole tree have the same lifetime (arena allocation).
#[derive(Debug, Clone)]
pub struct Tree {
    arena: HashMap<ID, AstNode>,
    max_id: ID,
    root_id: ID,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            arena: HashMap::new(),
            max_id: 0,
            root_id: 0,
        }
    }

    pub fn get_node(&self, index: ID) -> AstNode {
        let result = self.arena.get(&index);
        match result {
            Some(node) => node.clone(),
            None => panic!("No node with this ID in tree"),
        }
    }

    pub fn get_relation(&self, index: ID) -> AstRelation {
        let result = self.arena.get(&index);
        match result {
            Some(node) => node.relation.clone(),
            None => panic!("No relation with this ID in tree"),
        }
    }

    pub fn add_node(&mut self, node_id: ID, relation: AstRelation) {
        self.arena.insert(node_id, AstNode::new(node_id, relation));
        if node_id > self.max_id {
            self.max_id = node_id;
        }
    }

    pub fn add_root_node(&mut self, node_id: ID, relation: AstRelation) {
        self.arena.insert(node_id, AstNode::new(node_id, relation));
        self.root_id = node_id;
        if node_id > self.max_id {
            self.max_id = node_id;
        }
    }

    pub fn link_child(&mut self, node_id: ID, child_id: ID) {
        if self.arena.contains_key(&node_id) && self.arena.contains_key(&child_id) {
            self.arena.get_mut(&node_id).unwrap().link_child(child_id);
        }
    }

    pub fn replace_children(&mut self, node_id: ID, child_ids: Vec<ID>) {
        if self.arena.contains_key(&node_id) {
            self.arena
                .get_mut(&node_id)
                .unwrap()
                .replace_children(child_ids);
        }
    }

    pub fn size(&self) -> usize {
        self.arena.len()
    }

    pub fn pretty_print(&self) {
        self.arena
            .get(&self.root_id)
            .unwrap()
            .pretty_print(&String::from(""), &self.arena);
    }

    pub fn get_root(&self) -> ID {
        self.root_id
    }

    pub fn update_relation(&mut self, node_id: ID, relation: AstRelation) {
        if self.arena.contains_key(&node_id) {
            self.arena
                .get_mut(&node_id)
                .unwrap()
                .update_relation(relation);
        }
    }
}

// Building block of AST.
#[derive(Debug, Clone)]
pub struct AstNode {
    node_id: ID,
    relation: AstRelation,
    location: Location,
    children: Vec<ID>,
}

impl AstNode {
    fn new(node_id: ID, relation: AstRelation) -> Self {
        Self {
            node_id,
            relation,
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

    fn pretty_print(&self, indent: &String, arena: &HashMap<ID, AstNode>) {
        println!("{}{:?}", indent, self.relation);
        let new_indent = format!("{}{}", indent, "   ");
        for child_id in &self.children {
            arena
                .get(child_id)
                .unwrap()
                .pretty_print(&new_indent, arena)
        }
    }

    fn update_relation(&mut self, relation: AstRelation) {
        self.relation = relation;
    }
}

// Flattens AST and converts into a set of relations.
pub fn get_initial_relation_set(ast: &Tree) -> HashSet<AstRelation> {
    let mut relation_set: HashSet<AstRelation> = HashSet::new();
    for node in ast.clone().arena {
        relation_set.insert(node.1.relation);
    }
    relation_set
}

// Finds the differences between the to ASTs with structural differencing and flattens.
// Returns separate sets for relations that need to be deleted and relations that are inserted.
// Here IDs are allocated in a way that unchanged nodes retain their previous IDs.
// (By adjusting towards the existing tree.)
pub fn get_diff_relation_set(
    prev_ast: &Tree,
    new_ast: &Tree,
) -> (HashSet<AstRelation>, HashSet<AstRelation>, Tree) {
    let mut updated_tree = prev_ast.clone();
    let prev_root = prev_ast.get_node(prev_ast.get_root());
    let new_root = new_ast.get_node(new_ast.get_root());
    let mut insertion_set = HashSet::new();
    let mut deletion_set = HashSet::new();
    // For now we are assuming all top level declarations are function and we will identify them by names.
    // (Also assuming you are more likely to change function order rather than name).
    for fun_id in &prev_root.children {
        match prev_ast.get_relation(*fun_id) {
            AstRelation::FunDef {
                id: prev_id,
                fun_name: prev_fun_name,
                return_type_id: prev_return_type_id,
                arg_ids: prev_arg_ids,
                body_id: prev_body_id,
            } => {
                for new_fun_id in &new_root.children {
                    let node_to_compare = new_ast.get_node(*new_fun_id);
                    match node_to_compare.relation {
                        AstRelation::FunDef {
                            // Ignore IDs in new tree, they are just a lookup tool.
                            id: _,
                            fun_name: new_fun_name,
                            return_type_id: new_return_type_id,
                            arg_ids: new_arg_ids,
                            body_id: new_body_id,
                        } => {
                            // Case: function name matches so we keep comparing.
                            if prev_fun_name == new_fun_name {
                                // Compare return type (could either match or not but will definitely be there).
                                let prev_return_type = prev_ast.get_relation(prev_return_type_id);
                                let new_return_type = new_ast.get_relation(new_return_type_id);
                                if !relations_match(&prev_return_type, &new_return_type) {
                                    // If return type has changed:
                                    // Delete the current return type relation.
                                    deletion_set.insert(prev_return_type);
                                    // Change the ID in the new return type to match the previous one.
                                    // Update the corresponding node in the tree.
                                    updated_tree.update_relation(
                                        prev_return_type_id,
                                        replace_id_in_relation(
                                            &new_return_type,
                                            prev_return_type_id,
                                        ),
                                    );
                                }
                                // Compare argument types (in this case order matters).
                                for (index, prev_arg_id) in prev_arg_ids.iter().enumerate() {
                                    if index < new_arg_ids.len() {
                                        let new_arg_id = new_arg_ids[index];
                                        // If a corresponding index relation exist, name and type could differ or match.
                                        if !relations_match(
                                            &prev_ast.get_relation(*prev_arg_id),
                                            &new_ast.get_relation(new_arg_id),
                                        ) {}
                                    }
                                }
                                // Compare function bodies.
                            } else {
                            }
                        }
                        _ => panic!("Unexpected node during diffing"),
                    }
                }
            }
            _ => panic!("Unexpected node during diffing"),
        }
    }
    (insertion_set, deletion_set, updated_tree)
}

fn replace_id_in_relation(r: &AstRelation, id: ID) -> AstRelation {
    match r {
        AstRelation::Void { id: _ } => return AstRelation::Void { id },
        AstRelation::Int { id: _ } => return AstRelation::Int { id },
        AstRelation::Float { id: _ } => return AstRelation::Float { id },
        AstRelation::Char { id: _ } => return AstRelation::Char { id },
        _ => panic!("ID replacement not implemented for this relation type"),
    }
}

// Return true if they are of the same type (and have the same name, if applicable).
// So effectively ignoring exact IDs (this doesn't mean children haven't changed).
fn relations_match(r1: &AstRelation, r2: &AstRelation) -> bool {
    match (r1, r2) {
        (AstRelation::Char { id: _ }, AstRelation::Char { id: _ }) => return true,
        (AstRelation::Float { id: _ }, AstRelation::Float { id: _ }) => return true,
        (AstRelation::Int { id: _ }, AstRelation::Int { id: _ }) => return true,
        (AstRelation::Void { id: _ }, AstRelation::Void { id: _ }) => return true,
        (
            AstRelation::Arg {
                id: _,
                var_name: var_name1,
                type_id: _,
            },
            AstRelation::Arg {
                id: _,
                var_name: var_name2,
                type_id: _,
            },
        ) => return var_name1 == var_name2,
        (
            AstRelation::Var {
                id: _,
                var_name: var_name1,
            },
            AstRelation::Var {
                id: _,
                var_name: var_name2,
            },
        ) => return var_name1 == var_name2,
        (
            AstRelation::BinaryOp {
                id: _,
                arg1_id: _,
                arg2_id: _,
            },
            AstRelation::BinaryOp {
                id: _,
                arg1_id: _,
                arg2_id: _,
            },
        ) => return true,
        (
            AstRelation::EndItem { id: _, stmt_id: _ },
            AstRelation::EndItem { id: _, stmt_id: _ },
        ) => return true,
        (
            AstRelation::Item {
                id: _,
                stmt_id: _,
                next_stmt_id: _,
            },
            AstRelation::Item {
                id: _,
                stmt_id: _,
                next_stmt_id: _,
            },
        ) => return true,
        (
            AstRelation::Compound { id: _, start_id: _ },
            AstRelation::Compound { id: _, start_id: _ },
        ) => return true,
        (AstRelation::Return { id: _, expr_id: _ }, AstRelation::Return { id: _, expr_id: _ }) => {
            return true
        }
        (
            AstRelation::Assign {
                id: _,
                var_name: var_name1,
                type_id: _,
                expr_id: _,
            },
            AstRelation::Assign {
                id: _,
                var_name: var_name2,
                type_id: _,
                expr_id: _,
            },
        ) => return var_name1 == var_name2,
        (
            AstRelation::FunCall {
                id: _,
                fun_name: fun_name1,
                arg_ids: _,
            },
            AstRelation::FunCall {
                id: _,
                fun_name: fun_name2,
                arg_ids: _,
            },
        ) => return fun_name1 == fun_name2,
        (
            AstRelation::FunDef {
                id: _,
                fun_name: fun_name1,
                return_type_id: _,
                arg_ids: _,
                body_id: _,
            },
            AstRelation::FunDef {
                id: _,
                fun_name: fun_name2,
                return_type_id: _,
                arg_ids: _,
                body_id: _,
            },
        ) => return fun_name1 == fun_name2,
        (
            AstRelation::TransUnit { id: _, body_ids: _ },
            AstRelation::TransUnit { id: _, body_ids: _ },
        ) => return true,
        (_, _) => return false,
    }
}

pub fn get_relation_id(r: &AstRelation) -> ID {
    match r {
        AstRelation::Char { id } => return *id,
        AstRelation::Float { id } => return *id,
        AstRelation::Int { id } => return *id,
        AstRelation::Void { id } => return *id,
        AstRelation::Arg {
            id,
            var_name: _,
            type_id: _,
        } => return *id,
        AstRelation::Var { id, var_name: _ } => return *id,
        AstRelation::BinaryOp {
            id,
            arg1_id: _,
            arg2_id: _,
        } => return *id,
        AstRelation::EndItem { id, stmt_id: _ } => return *id,
        AstRelation::Item {
            id,
            stmt_id: _,
            next_stmt_id: _,
        } => return *id,
        AstRelation::Compound { id, start_id: _ } => return *id,
        AstRelation::Return { id, expr_id: _ } => return *id,
        AstRelation::Assign {
            id,
            var_name: _,
            type_id: _,
            expr_id: _,
        } => return *id,
        AstRelation::FunCall {
            id,
            fun_name: _,
            arg_ids: _,
        } => return *id,
        AstRelation::FunDef {
            id,
            fun_name: _,
            return_type_id: _,
            arg_ids: _,
            body_id: _,
        } => return *id,
        AstRelation::TransUnit { id, body_ids: _ } => return *id,
    }
}

#[cfg(test)]
mod tests {}
