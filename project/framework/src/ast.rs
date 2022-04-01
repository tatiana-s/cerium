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

    pub fn delete_node(&mut self, node_id: ID) {
        self.arena.remove(&node_id);
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
    let mut fun_to_be_deleted: HashMap<ID, bool> = HashMap::new();
    // Need to check against this in the end to find functions that are completely new.
    let mut matching_new_funs: Vec<ID> = vec![];
    for fun_id in &prev_root.children {
        match prev_ast.get_relation(*fun_id) {
            AstRelation::FunDef {
                id: prev_id,
                fun_name: prev_fun_name,
                return_type_id: prev_return_type_id,
                arg_ids: prev_arg_ids,
                body_id: prev_body_id,
            } => {
                fun_to_be_deleted.insert(prev_id, true);
                'new_search: for new_fun_id in &new_root.children {
                    let node_to_compare = new_ast.get_node(*new_fun_id);
                    match node_to_compare.relation {
                        AstRelation::FunDef {
                            // IDs here are really just a lookup tool.
                            id: new_id,
                            fun_name: new_fun_name,
                            return_type_id: new_return_type_id,
                            arg_ids: new_arg_ids,
                            body_id: new_body_id,
                        } => {
                            // Case: function name matches so we keep comparing.
                            if prev_fun_name == new_fun_name {
                                matching_new_funs.push(new_id);
                                // Compare return type (could either match or not but will definitely be there).
                                let prev_return_type = prev_ast.get_relation(prev_return_type_id);
                                let new_return_type = new_ast.get_relation(new_return_type_id);
                                if !relations_match(&prev_return_type, &new_return_type) {
                                    // If return type has changed:
                                    // Delete the current return type relation.
                                    deletion_set.insert(prev_return_type);
                                    // Change the ID in the new return type to match the previous one.
                                    let replacement = replace_id_in_relation(
                                        &new_return_type,
                                        prev_return_type_id,
                                    );
                                    // Update the corresponding node in the tree.
                                    updated_tree
                                        .update_relation(prev_return_type_id, replacement.clone());
                                    // Insert the new relation.
                                    insertion_set.insert(replacement);
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
                                // Mark this function as not having to be completely deleted.
                                fun_to_be_deleted.insert(prev_id, false);
                                // Break out of the loop since we have now found a matched function.
                                break 'new_search;
                            }
                        }
                        _ => panic!("Unexpected node during diffing"),
                    }
                }
            }
            _ => panic!("Unexpected node during diffing"),
        }
    }
    // Iterate over prev functions to be deleted and add result to deletion set (pass tree to be updated as well).
    for (prev_fun_id, indicator) in fun_to_be_deleted {
        if indicator {
            let (deletions, new_updated_tree) = delete_onwards(prev_fun_id, updated_tree.clone());
            updated_tree = new_updated_tree;
            deletion_set.union(&deletions);
        }
    }
    // Iterate over new functions to see which ones aren't matching and add to insertion set (tree as well).
    for new_fun_id in &new_root.children {
        if !matching_new_funs.contains(new_fun_id) {
            let (insertions, new_updated_tree, inserted_fun_id) =
                insert_onwards(*new_fun_id, updated_tree.clone(), new_ast.clone());
            updated_tree = new_updated_tree;
            insertion_set.union(&insertions);
        }
    }
    // Return result.
    (insertion_set, deletion_set, updated_tree)
}

// Delete the node with the given ID and all its children.
// Don't forget to unlink this node from any parents before calling this.
fn delete_onwards(node_id: ID, mut ast: Tree) -> (HashSet<AstRelation>, Tree) {
    let mut delete_set: HashSet<AstRelation> = HashSet::new();
    let relation_to_be_deleted = ast.get_relation(node_id);
    let relation_to_be_deleted_clone = relation_to_be_deleted.clone();
    match relation_to_be_deleted {
        // Leaf nodes we don't have to consider any children recursively.
        AstRelation::Char { id: _ } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            return (delete_set, ast);
        }
        AstRelation::Float { id: _ } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            return (delete_set, ast);
        }
        AstRelation::Int { id: _ } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            return (delete_set, ast);
        }
        AstRelation::Void { id: _ } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            return (delete_set, ast);
        }
        // Other nodes just recursively apply function and add result to deletion set before returning.
        AstRelation::Arg {
            id: _,
            var_name: _,
            type_id,
        } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(type_id, ast);
            delete_set.union(&child_set);
            return (delete_set, updated_ast);
        }
        AstRelation::Var { id: _, var_name: _ } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            return (delete_set, ast);
        }
        AstRelation::BinaryOp {
            id: _,
            arg1_id,
            arg2_id,
        } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(arg1_id, ast);
            delete_set.union(&child_set);
            let (child_set, updated_ast) = delete_onwards(arg2_id, updated_ast);
            delete_set.union(&child_set);
            return (delete_set, updated_ast);
        }
        AstRelation::EndItem { id: _, stmt_id } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(stmt_id, ast);
            delete_set.union(&child_set);
            return (delete_set, updated_ast);
        }
        AstRelation::Item {
            id: _,
            stmt_id,
            next_stmt_id,
        } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(stmt_id, ast);
            delete_set.union(&child_set);
            let (child_set, updated_ast) = delete_onwards(next_stmt_id, updated_ast);
            delete_set.union(&child_set);
            return (delete_set, updated_ast);
        }
        AstRelation::Compound { id: _, start_id } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(start_id, ast);
            delete_set.union(&child_set);
            return (delete_set, updated_ast);
        }
        AstRelation::Return { id: _, expr_id } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(expr_id, ast);
            delete_set.union(&child_set);
            return (delete_set, updated_ast);
        }
        AstRelation::Assign {
            id: _,
            var_name: _,
            type_id,
            expr_id,
        } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(type_id, ast);
            delete_set.union(&child_set);
            let (child_set, updated_ast) = delete_onwards(expr_id, updated_ast);
            delete_set.union(&child_set);
            return (delete_set, updated_ast);
        }
        AstRelation::FunCall {
            id: _,
            fun_name: _,
            arg_ids,
        } => {
            delete_set.insert(relation_to_be_deleted_clone);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let mut updated_ast = ast.clone();
            for arg_id in arg_ids {
                let (child_set, new_updated_ast) = delete_onwards(arg_id, updated_ast.clone());
                updated_ast = new_updated_ast;
                delete_set.union(&child_set);
            }
            return (delete_set, ast);
        }
        AstRelation::FunDef {
            id: _,
            fun_name: _,
            return_type_id,
            arg_ids,
            body_id,
        } => {
            delete_set.insert(relation_to_be_deleted_clone);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let mut updated_ast = ast.clone();
            let (child_set, new_updated_ast) = delete_onwards(return_type_id, updated_ast);
            updated_ast = new_updated_ast;
            delete_set.union(&child_set);
            for arg_id in arg_ids {
                let (child_set, new_updated_ast) = delete_onwards(arg_id, updated_ast.clone());
                updated_ast = new_updated_ast;
                delete_set.union(&child_set);
            }
            let (child_set, updated_ast) = delete_onwards(body_id, updated_ast);
            delete_set.union(&child_set);
            return (delete_set, updated_ast);
        }
        AstRelation::TransUnit { id: _, body_ids } => {
            delete_set.insert(relation_to_be_deleted_clone);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let mut updated_ast = ast.clone();
            for body_id in body_ids {
                let (child_set, new_updated_ast) = delete_onwards(body_id, updated_ast);
                updated_ast = new_updated_ast;
                delete_set.union(&child_set);
            }
            return (delete_set, ast);
        }
    }
}

// Insert the node with the given ID and all its children.
// Don't forget to link this node to any parents before calling this.
// (ast = tree we are updating, new_ast = tree we get the relations to insert from.)
// Here we need to pay attention to not confuse IDs in maintained tree vs. IDs in new tree which we don't actually care about.
fn insert_onwards(node_id: ID, mut ast: Tree, new_ast: Tree) -> (HashSet<AstRelation>, Tree, ID) {
    let mut insertion_set: HashSet<AstRelation> = HashSet::new();
    let relation_to_be_inserted = new_ast.get_relation(node_id);
    match relation_to_be_inserted {
        // Leaf nodes we don't have to consider any children recursively.
        AstRelation::Char { id: _ } => {
            let new_id = ast.max_id + 1;
            let new_relation = replace_id_in_relation(&relation_to_be_inserted, new_id);
            insertion_set.insert(new_relation.clone());
            ast.add_node(new_id, new_relation);
            return (insertion_set, ast, new_id);
        }
        AstRelation::Float { id: _ } => {
            let new_id = ast.max_id + 1;
            let new_relation = replace_id_in_relation(&relation_to_be_inserted, new_id);
            insertion_set.insert(new_relation.clone());
            ast.add_node(new_id, new_relation);
            return (insertion_set, ast, new_id);
        }
        AstRelation::Int { id: _ } => {
            let new_id = ast.max_id + 1;
            let new_relation = replace_id_in_relation(&relation_to_be_inserted, new_id);
            insertion_set.insert(new_relation.clone());
            ast.add_node(new_id, new_relation);
            return (insertion_set, ast, new_id);
        }
        AstRelation::Void { id: _ } => {
            let new_id = ast.max_id + 1;
            let new_relation = replace_id_in_relation(&relation_to_be_inserted, new_id);
            insertion_set.insert(new_relation.clone());
            ast.add_node(new_id, new_relation);
            return (insertion_set, ast, new_id);
        }
        // Other nodes have to take care with linking children correctly for both relations and nodes.
        AstRelation::Arg {
            id: _,
            var_name,
            type_id,
        } => {
            let (insertions, mut updated_ast, type_child_id) =
                insert_onwards(type_id, ast, new_ast);
            insertion_set.union(&insertions);
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::Arg {
                id: new_id,
                var_name,
                type_id: type_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, type_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::Var { id: _, var_name } => {
            let new_id = ast.max_id + 1;
            let new_relation = AstRelation::Var {
                id: new_id,
                var_name,
            };
            insertion_set.insert(new_relation.clone());
            ast.add_node(new_id, new_relation);
            return (insertion_set, ast, new_id);
        }
        AstRelation::BinaryOp {
            id: _,
            arg1_id,
            arg2_id,
        } => {
            let (insertions, updated_ast, arg1_child_id) =
                insert_onwards(arg1_id, ast, new_ast.clone());
            insertion_set.union(&insertions);
            let (insertions, mut updated_ast, arg2_child_id) =
                insert_onwards(arg2_id, updated_ast, new_ast);
            insertion_set.union(&insertions);
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::BinaryOp {
                id: new_id,
                arg1_id: arg1_child_id,
                arg2_id: arg2_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, arg1_child_id);
            updated_ast.link_child(new_id, arg2_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::EndItem { id: _, stmt_id } => {
            let (insertions, mut updated_ast, stmt_child_id) =
                insert_onwards(stmt_id, ast, new_ast);
            insertion_set.union(&insertions);
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::EndItem {
                id: new_id,
                stmt_id: stmt_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, stmt_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::Item {
            id: _,
            stmt_id,
            next_stmt_id,
        } => {
            let (insertions, updated_ast, stmt_child_id) =
                insert_onwards(stmt_id, ast, new_ast.clone());
            insertion_set.union(&insertions);
            let (insertions, mut updated_ast, next_stmt_child_id) =
                insert_onwards(next_stmt_id, updated_ast, new_ast);
            insertion_set.union(&insertions);
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::Item {
                id: new_id,
                stmt_id: stmt_child_id,
                next_stmt_id: next_stmt_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, stmt_child_id);
            updated_ast.link_child(new_id, next_stmt_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::Compound { id: _, start_id } => {
            let (insertions, mut updated_ast, start_child_id) =
                insert_onwards(start_id, ast, new_ast);
            insertion_set.union(&insertions);
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::Compound {
                id: new_id,
                start_id: start_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, start_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::Return { id: _, expr_id } => {
            let (insertions, mut updated_ast, expr_child_id) =
                insert_onwards(expr_id, ast, new_ast);
            insertion_set.union(&insertions);
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::Return {
                id: new_id,
                expr_id: expr_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, expr_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::Assign {
            id: _,
            var_name,
            type_id,
            expr_id,
        } => {
            let (insertions, updated_ast, type_child_id) =
                insert_onwards(type_id, ast, new_ast.clone());
            insertion_set.union(&insertions);
            let (insertions, mut updated_ast, expr_child_id) =
                insert_onwards(expr_id, updated_ast, new_ast);
            insertion_set.union(&insertions);
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::Assign {
                id: new_id,
                var_name,
                type_id: type_child_id,
                expr_id: expr_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, type_child_id);
            updated_ast.link_child(new_id, expr_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::FunCall {
            id: _,
            fun_name,
            arg_ids,
        } => {
            let mut updated_ast = ast.clone();
            let mut new_child_ids: Vec<ID> = vec![];
            for arg_id in arg_ids {
                let (insertions, new_updated_ast, arg_child_id) =
                    insert_onwards(arg_id, updated_ast, new_ast.clone());
                new_child_ids.push(arg_child_id);
                updated_ast = new_updated_ast;
                insertion_set.union(&insertions);
            }
            let new_id = ast.max_id + 1;
            let new_relation = AstRelation::FunCall {
                id: new_id,
                fun_name,
                arg_ids: new_child_ids.clone(),
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.replace_children(new_id, new_child_ids);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::FunDef {
            id: _,
            fun_name,
            return_type_id,
            arg_ids,
            body_id,
        } => {
            let (insertions, mut updated_ast, return_child_id) =
                insert_onwards(return_type_id, ast, new_ast.clone());
            insertion_set.union(&insertions);
            let mut new_child_ids: Vec<ID> = vec![];
            for arg_id in arg_ids {
                let (insertions, new_updated_ast, arg_child_id) =
                    insert_onwards(arg_id, updated_ast, new_ast.clone());
                new_child_ids.push(arg_child_id);
                updated_ast = new_updated_ast;
                insertion_set.union(&insertions);
            }
            let (insertions, mut updated_ast, body_child_id) =
                insert_onwards(body_id, updated_ast, new_ast);
            insertion_set.union(&insertions);
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::FunDef {
                id: new_id,
                fun_name,
                return_type_id: return_child_id,
                arg_ids: new_child_ids.clone(),
                body_id: body_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.replace_children(new_id, new_child_ids);
            updated_ast.link_child(new_id, return_child_id);
            updated_ast.link_child(new_id, body_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::TransUnit { id: _, body_ids } => {
            let mut updated_ast = ast.clone();
            let mut new_child_ids: Vec<ID> = vec![];
            for body_id in body_ids {
                let (insertions, new_updated_ast, arg_child_id) =
                    insert_onwards(body_id, updated_ast, new_ast.clone());
                new_child_ids.push(arg_child_id);
                updated_ast = new_updated_ast;
                insertion_set.union(&insertions);
            }
            let new_id = ast.max_id + 1;
            let new_relation = AstRelation::TransUnit {
                id: new_id,
                body_ids: new_child_ids.clone(),
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.replace_children(new_id, new_child_ids);
            return (insertion_set, updated_ast, new_id);
        }
    }
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
