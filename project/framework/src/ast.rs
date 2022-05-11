use crate::definitions::{AstRelation, ID};
use std::collections::{HashMap, HashSet};
use std::fmt;

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

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.max_id)
    }
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
            None => panic!("No relation with this ID ({}) in tree", index),
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

    pub fn flat_print(&self) {
        for node in &self.arena {
            println!("{:?}", node.1.relation);
        }
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
        self.max_id = *self.arena.keys().max().unwrap();
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
                                if !relations_match(
                                    &prev_return_type,
                                    &new_return_type,
                                    prev_ast,
                                    new_ast,
                                ) {
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
                                // If there are insertions/deletions and not just replacements we have to adjust the function relation.
                                let mut remaining_args: Vec<ID> = vec![];
                                let mut args_have_changed = false;
                                for (index, prev_arg_id) in prev_arg_ids.iter().enumerate() {
                                    if index < new_arg_ids.len() {
                                        let new_arg_id = new_arg_ids[index];
                                        // If a corresponding index relation exist, name and type could differ or match.
                                        let prev_arg = prev_ast.get_relation(*prev_arg_id);
                                        let new_arg = new_ast.get_relation(new_arg_id);
                                        match (prev_arg, new_arg) {
                                            (
                                                AstRelation::Arg {
                                                    id,
                                                    var_name: var_name1,
                                                    type_id: type_id1,
                                                },
                                                AstRelation::Arg {
                                                    id: _,
                                                    var_name: var_name2,
                                                    type_id: type_id2,
                                                },
                                            ) => {
                                                let prev_type = prev_ast.get_relation(type_id1);
                                                let new_type = new_ast.get_relation(type_id2);
                                                if !relations_match(
                                                    &prev_type, &new_type, prev_ast, new_ast,
                                                ) {
                                                    // Replace type.
                                                    deletion_set.insert(prev_type);
                                                    let replacement =
                                                        replace_id_in_relation(&new_type, type_id1);
                                                    updated_tree.update_relation(
                                                        type_id1,
                                                        replacement.clone(),
                                                    );
                                                    insertion_set.insert(replacement);
                                                }
                                                if var_name1 != var_name2 {
                                                    // Replace name.
                                                    let replacement = AstRelation::Arg {
                                                        id,
                                                        var_name: var_name2,
                                                        type_id: type_id1,
                                                    };
                                                    updated_tree
                                                        .update_relation(id, replacement.clone());
                                                    updated_tree
                                                        .replace_children(id, vec![type_id1]);
                                                    insertion_set.insert(replacement);
                                                }
                                            }
                                            _ => panic!("Unexpected node during diffing"),
                                        }
                                        remaining_args.push(*prev_arg_id);
                                    } else {
                                        // This means the previous argument list was longer so we need to delete some.
                                        let (deletions, new_updated_tree) =
                                            delete_onwards(*prev_arg_id, updated_tree);
                                        for relation in deletions {
                                            deletion_set.insert(relation);
                                        }
                                        updated_tree = new_updated_tree;
                                        args_have_changed = true;
                                    }
                                }
                                // This means there are more arguments in the new tree.
                                if new_arg_ids.len() > prev_arg_ids.len() {
                                    for (index, new_arg_id) in new_arg_ids.iter().enumerate() {
                                        if index >= prev_arg_ids.len() {
                                            let (insertions, new_updated_tree, updated_arg_id) =
                                                insert_onwards(
                                                    *new_arg_id,
                                                    updated_tree,
                                                    new_ast.clone(),
                                                );
                                            for relation in insertions {
                                                insertion_set.insert(relation);
                                            }
                                            updated_tree = new_updated_tree;
                                            remaining_args.push(updated_arg_id);
                                            args_have_changed = true;
                                        }
                                    }
                                }
                                if args_have_changed {
                                    deletion_set.insert(prev_ast.get_relation(prev_id));
                                    let replacement = AstRelation::FunDef {
                                        id: prev_id,
                                        fun_name: prev_fun_name,
                                        return_type_id: prev_return_type_id,
                                        // Just change arguments.
                                        arg_ids: remaining_args.clone(),
                                        body_id: prev_body_id,
                                    };
                                    insertion_set.insert(replacement.clone());
                                    updated_tree.update_relation(prev_id, replacement);
                                    updated_tree.replace_children(prev_id, remaining_args);
                                    updated_tree.link_child(prev_id, prev_return_type_id);
                                    updated_tree.link_child(prev_id, prev_body_id);
                                }

                                // Compare function bodies.
                                let prev_body = prev_ast.get_relation(prev_body_id);
                                let new_body = new_ast.get_relation(new_body_id);
                                match (prev_body, new_body) {
                                    (
                                        AstRelation::Compound {
                                            id: _,
                                            start_id: start_id1,
                                        },
                                        AstRelation::Compound {
                                            id: _,
                                            start_id: start_id2,
                                        },
                                    ) => {
                                        let (insertions, deletions, new_updated_tree, _) =
                                            compare_items(
                                                start_id1,
                                                start_id2,
                                                updated_tree.clone(),
                                                new_ast.clone(),
                                            );
                                        updated_tree = new_updated_tree;
                                        for relation in insertions {
                                            insertion_set.insert(relation);
                                        }
                                        for relation in deletions {
                                            deletion_set.insert(relation);
                                        }
                                    }
                                    _ => panic!("Unexpected node during diffing"),
                                }

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
    let mut remaining_funs: Vec<ID> = vec![];
    for (prev_fun_id, indicator) in fun_to_be_deleted {
        if indicator {
            let (deletions, new_updated_tree) = delete_onwards(prev_fun_id, updated_tree.clone());
            updated_tree = new_updated_tree;
            for relation in deletions {
                deletion_set.insert(relation);
            }
        } else {
            remaining_funs.push(prev_fun_id);
        }
    }
    // Iterate over new functions to see which ones aren't matching and add to insertion set (tree as well).
    for new_fun_id in &new_root.children {
        if !matching_new_funs.contains(new_fun_id) {
            let (insertions, new_updated_tree, inserted_fun_id) =
                insert_onwards(*new_fun_id, updated_tree.clone(), new_ast.clone());
            updated_tree = new_updated_tree;
            for relation in insertions {
                insertion_set.insert(relation);
            }
            remaining_funs.push(inserted_fun_id);
        }
    }
    // Replace root with translation unit that has the correct list of declarations.
    let mut prev_funs = vec![];
    if let AstRelation::TransUnit { id: _, body_ids } = prev_ast.get_relation(prev_ast.get_root()) {
        prev_funs = body_ids;
    }
    if !(remaining_funs.iter().all(|item| prev_funs.contains(item)))
        || !(prev_funs.iter().all(|item| remaining_funs.contains(item)))
    {
        deletion_set.insert(prev_ast.get_relation(prev_ast.get_root()));
        let final_root = AstRelation::TransUnit {
            id: prev_ast.get_root(),
            body_ids: remaining_funs.clone(),
        };
        insertion_set.insert(final_root.clone());
        updated_tree.update_relation(prev_ast.get_root(), final_root);
        updated_tree.replace_children(prev_ast.get_root(), remaining_funs);
    }
    // Return result.
    // updated_tree.pretty_print();
    (insertion_set, deletion_set, updated_tree)
}

fn compare_items(
    item_id1: ID,
    item_id2: ID,
    t1: Tree,
    t2: Tree,
) -> (HashSet<AstRelation>, HashSet<AstRelation>, Tree, ID) {
    let mut insertion_set = HashSet::new();
    let mut deletion_set = HashSet::new();
    let item1 = t1.get_relation(item_id1);
    let item2 = t2.get_relation(item_id2);
    let item1_clone = item1.clone();
    match (item1, item2) {
        (
            AstRelation::Item {
                id: id1,
                stmt_id: stmt_id1,
                next_stmt_id: next_stmt_id1,
            },
            AstRelation::Item {
                id: _,
                stmt_id: stmt_id2,
                next_stmt_id: next_stmt_id2,
            },
        ) => {
            if relations_match(
                &t1.get_relation(stmt_id1),
                &t2.get_relation(stmt_id2),
                &t1,
                &t2,
            ) {
                // If the statements match just move on to the next item.
                let (insertions, deletions, mut updated_tree, next_id) =
                    compare_items(next_stmt_id1, next_stmt_id2, t1, t2);
                // However the ID of the next statement could have changed due to a new insertion.
                if next_stmt_id1 != next_id {
                    let replacement = AstRelation::Item {
                        id: id1,
                        stmt_id: stmt_id1,
                        next_stmt_id: next_id,
                    };
                    for relation in insertions {
                        insertion_set.insert(relation);
                    }
                    for relation in deletions {
                        deletion_set.insert(relation);
                    }
                    insertion_set.insert(replacement.clone());
                    deletion_set.insert(item1_clone);
                    updated_tree.update_relation(id1, replacement);
                    updated_tree.replace_children(id1, vec![stmt_id1, next_id]);
                    return (insertion_set, deletion_set, updated_tree, id1);
                } else {
                    return (insertions, deletions, updated_tree, id1);
                }
            } else {
                // Otherwise: keep comparing the prev item and insert a new item.
                let (insertions, deletions, updated_tree, next_id) =
                    compare_items(id1, next_stmt_id2, t1, t2.clone());
                for relation in insertions {
                    insertion_set.insert(relation);
                }
                for relation in deletions {
                    deletion_set.insert(relation);
                }
                let new_id = updated_tree.max_id + 1;
                let (insertions, mut updated_tree, stmt_id) =
                    insert_onwards(stmt_id2, updated_tree, t2);
                for relation in insertions {
                    insertion_set.insert(relation);
                }
                let new_item = AstRelation::Item {
                    id: new_id,
                    stmt_id: stmt_id,
                    next_stmt_id: next_id,
                };
                insertion_set.insert(new_item.clone());
                updated_tree.add_node(new_id, new_item);
                updated_tree.link_child(new_id, stmt_id);
                updated_tree.link_child(new_id, next_id);
                return (insertion_set, deletion_set, updated_tree, new_id);
            }
        }
        (
            AstRelation::EndItem {
                id: id1,
                stmt_id: stmt_id1,
            },
            AstRelation::Item {
                id: _,
                stmt_id: stmt_id2,
                next_stmt_id: next_stmt_id2,
            },
        ) => {
            if relations_match(
                &t1.get_relation(stmt_id1),
                &t2.get_relation(stmt_id2),
                &t1,
                &t2,
            ) {
                // Insert from whole item onwards.
                let (insertions, mut updated_tree, next_item) =
                    insert_onwards(next_stmt_id2, t1, t2);
                // Change the prev item to normal instead of end item.
                let replacement = AstRelation::Item {
                    id: id1,
                    stmt_id: stmt_id1,
                    next_stmt_id: next_item,
                };
                for relation in insertions {
                    insertion_set.insert(relation);
                }
                insertion_set.insert(replacement.clone());
                deletion_set.insert(item1_clone);
                updated_tree.update_relation(id1, replacement);
                updated_tree.replace_children(id1, vec![stmt_id1, next_item]);
                return (insertion_set, deletion_set, updated_tree, id1);
            } else {
                // Otherwise: keep comparing the prev item and insert a new item.
                let (insertions, deletions, updated_tree, next_id) =
                    compare_items(id1, next_stmt_id2, t1, t2.clone());
                for relation in insertions {
                    insertion_set.insert(relation);
                }
                for relation in deletions {
                    deletion_set.insert(relation);
                }
                let (insertions, mut new_updated_tree, stmt_id) =
                    insert_onwards(stmt_id2, updated_tree, t2);
                for relation in insertions {
                    insertion_set.insert(relation);
                }
                let new_id = new_updated_tree.max_id + 1;
                let new_item = AstRelation::Item {
                    id: new_id,
                    stmt_id: stmt_id,
                    next_stmt_id: next_id,
                };
                insertion_set.insert(new_item.clone());
                new_updated_tree.add_node(new_id, new_item);
                new_updated_tree.link_child(new_id, stmt_id);
                new_updated_tree.link_child(new_id, next_id);
                return (insertion_set, deletion_set, new_updated_tree, new_id);
            }
        }
        (
            AstRelation::Item {
                id: id1,
                stmt_id: stmt_id1,
                next_stmt_id: next_stmt_id1,
            },
            AstRelation::EndItem {
                id: _,
                stmt_id: stmt_id2,
            },
        ) => {
            if relations_match(
                &t1.get_relation(stmt_id1),
                &t2.get_relation(stmt_id2),
                &t1,
                &t2,
            ) {
                // Delete from next statement onwards.
                let (deletions, mut updated_tree) = delete_onwards(next_stmt_id1, t1);
                for relation in deletions {
                    deletion_set.insert(relation);
                }
                // Make this item an end item instead.
                let replacement = AstRelation::EndItem {
                    id: id1,
                    stmt_id: stmt_id1,
                };
                insertion_set.insert(replacement.clone());
                deletion_set.insert(item1_clone);
                updated_tree.update_relation(id1, replacement);
                updated_tree.replace_children(id1, vec![stmt_id1]);
                return (insertion_set, deletion_set, updated_tree, id1);
            } else {
                // Delete from next statement onwards.
                let (deletions, updated_tree) = delete_onwards(next_stmt_id1, t1);
                for relation in deletions {
                    deletion_set.insert(relation);
                }
                // Insert the differing statement.
                let (insertions, mut updated_tree, stmt_id) =
                    insert_onwards(stmt_id2, updated_tree, t2);
                for relation in insertions {
                    insertion_set.insert(relation);
                }
                // Make this item an end item instead.
                let replacement = AstRelation::EndItem {
                    id: id1,
                    stmt_id: stmt_id,
                };
                insertion_set.insert(replacement.clone());
                deletion_set.insert(item1_clone);
                updated_tree.update_relation(id1, replacement);
                updated_tree.replace_children(id1, vec![stmt_id]);
                return (insertion_set, deletion_set, updated_tree, id1);
            }
        }
        (
            // Case: no further comparisons needed after this one.
            AstRelation::EndItem {
                id: id1,
                stmt_id: stmt_id1,
            },
            AstRelation::EndItem {
                id: _,
                stmt_id: stmt_id2,
            },
        ) => {
            if relations_match(
                &t1.get_relation(stmt_id1),
                &t2.get_relation(stmt_id2),
                &t1,
                &t2,
            ) {
                return (insertion_set, deletion_set, t1, id1);
            } else {
                let (insertions, mut updated_tree, stmt_id) = insert_onwards(stmt_id2, t1, t2);
                let replacement = AstRelation::EndItem {
                    id: id1,
                    stmt_id: stmt_id,
                };
                for relation in insertions {
                    insertion_set.insert(relation);
                }
                insertion_set.insert(replacement.clone());
                deletion_set.insert(item1_clone);
                updated_tree.update_relation(id1, replacement);
                updated_tree.replace_children(id1, vec![stmt_id]);
                return (insertion_set, deletion_set, updated_tree, id1);
            }
        }
        (_, _) => panic!("Unexpected node during diffing"),
    }
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
            for relation in child_set {
                delete_set.insert(relation);
            }
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
            for relation in child_set {
                delete_set.insert(relation);
            }
            let (child_set, updated_ast) = delete_onwards(arg2_id, updated_ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            return (delete_set, updated_ast);
        }
        AstRelation::EndItem { id: _, stmt_id } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(stmt_id, ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
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
            for relation in child_set {
                delete_set.insert(relation);
            }
            let (child_set, updated_ast) = delete_onwards(next_stmt_id, updated_ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            return (delete_set, updated_ast);
        }
        AstRelation::Compound { id: _, start_id } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(start_id, ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            return (delete_set, updated_ast);
        }
        AstRelation::While {
            id: _,
            cond_id,
            body_id,
        } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(cond_id, ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            let (child_set, updated_ast) = delete_onwards(body_id, updated_ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            return (delete_set, updated_ast);
        }
        AstRelation::IfElse {
            id: _,
            cond_id,
            then_id,
            else_id,
        } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(cond_id, ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            let (child_set, updated_ast) = delete_onwards(then_id, updated_ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            let (child_set, updated_ast) = delete_onwards(else_id, updated_ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            return (delete_set, updated_ast);
        }
        AstRelation::If {
            id: _,
            cond_id,
            then_id,
        } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(cond_id, ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            let (child_set, updated_ast) = delete_onwards(then_id, updated_ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
            return (delete_set, updated_ast);
        }
        AstRelation::Return { id: _, expr_id } => {
            delete_set.insert(relation_to_be_deleted);
            ast.delete_node(node_id);
            if node_id == ast.max_id {
                ast.max_id = *ast.arena.keys().max().unwrap();
            }
            let (child_set, updated_ast) = delete_onwards(expr_id, ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
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
            for relation in child_set {
                delete_set.insert(relation);
            }
            let (child_set, updated_ast) = delete_onwards(expr_id, updated_ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
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
                for relation in child_set {
                    delete_set.insert(relation);
                }
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
            for relation in child_set {
                delete_set.insert(relation);
            }
            for arg_id in arg_ids {
                let (child_set, new_updated_ast) = delete_onwards(arg_id, updated_ast.clone());
                updated_ast = new_updated_ast;
                for relation in child_set {
                    delete_set.insert(relation);
                }
            }
            let (child_set, updated_ast) = delete_onwards(body_id, updated_ast);
            for relation in child_set {
                delete_set.insert(relation);
            }
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
                for relation in child_set {
                    delete_set.insert(relation);
                }
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
            for relation in insertions {
                insertion_set.insert(relation);
            }
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
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let (insertions, mut updated_ast, arg2_child_id) =
                insert_onwards(arg2_id, updated_ast, new_ast);
            for relation in insertions {
                insertion_set.insert(relation);
            }
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
            for relation in insertions {
                insertion_set.insert(relation);
            }
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
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let (insertions, mut updated_ast, next_stmt_child_id) =
                insert_onwards(next_stmt_id, updated_ast, new_ast);
            for relation in insertions {
                insertion_set.insert(relation);
            }
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
            for relation in insertions {
                insertion_set.insert(relation);
            }
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
        AstRelation::While {
            id: _,
            cond_id,
            body_id,
        } => {
            let (insertions, updated_ast, cond_child_id) =
                insert_onwards(cond_id, ast, new_ast.clone());
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let (insertions, mut updated_ast, body_child_id) =
                insert_onwards(body_id, updated_ast, new_ast);
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::If {
                id: new_id,
                cond_id: cond_child_id,
                then_id: body_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, cond_child_id);
            updated_ast.link_child(new_id, body_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::IfElse {
            id: _,
            cond_id,
            then_id,
            else_id,
        } => {
            let (insertions, updated_ast, cond_child_id) =
                insert_onwards(cond_id, ast, new_ast.clone());
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let (insertions, updated_ast, then_child_id) =
                insert_onwards(then_id, updated_ast, new_ast.clone());
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let (insertions, mut updated_ast, else_child_id) =
                insert_onwards(else_id, updated_ast, new_ast);
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::IfElse {
                id: new_id,
                cond_id: cond_child_id,
                then_id: then_child_id,
                else_id: else_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, cond_child_id);
            updated_ast.link_child(new_id, then_child_id);
            updated_ast.link_child(new_id, else_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::If {
            id: _,
            cond_id,
            then_id,
        } => {
            let (insertions, updated_ast, cond_child_id) =
                insert_onwards(cond_id, ast, new_ast.clone());
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let (insertions, mut updated_ast, then_child_id) =
                insert_onwards(then_id, updated_ast, new_ast);
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let new_id = updated_ast.max_id + 1;
            let new_relation = AstRelation::If {
                id: new_id,
                cond_id: cond_child_id,
                then_id: then_child_id,
            };
            insertion_set.insert(new_relation.clone());
            updated_ast.add_node(new_id, new_relation);
            updated_ast.link_child(new_id, cond_child_id);
            updated_ast.link_child(new_id, then_child_id);
            return (insertion_set, updated_ast, new_id);
        }
        AstRelation::Return { id: _, expr_id } => {
            let (insertions, mut updated_ast, expr_child_id) =
                insert_onwards(expr_id, ast, new_ast);
            for relation in insertions {
                insertion_set.insert(relation);
            }
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
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let (insertions, mut updated_ast, expr_child_id) =
                insert_onwards(expr_id, updated_ast, new_ast);
            for relation in insertions {
                insertion_set.insert(relation);
            }
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
                for relation in insertions {
                    insertion_set.insert(relation);
                }
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
            for relation in insertions {
                insertion_set.insert(relation);
            }
            let mut new_child_ids: Vec<ID> = vec![];
            for arg_id in arg_ids {
                let (insertions, new_updated_ast, arg_child_id) =
                    insert_onwards(arg_id, updated_ast, new_ast.clone());
                new_child_ids.push(arg_child_id);
                updated_ast = new_updated_ast;
                for relation in insertions {
                    insertion_set.insert(relation);
                }
            }
            let (insertions, mut updated_ast, body_child_id) =
                insert_onwards(body_id, updated_ast, new_ast);
            for relation in insertions {
                insertion_set.insert(relation);
            }
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
                for relation in insertions {
                    insertion_set.insert(relation);
                }
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
// So effectively same structure just ignoring exact IDs.
fn relations_match(r1: &AstRelation, r2: &AstRelation, t1: &Tree, t2: &Tree) -> bool {
    match (r1, r2) {
        (AstRelation::Char { id: _ }, AstRelation::Char { id: _ }) => return true,
        (AstRelation::Float { id: _ }, AstRelation::Float { id: _ }) => return true,
        (AstRelation::Int { id: _ }, AstRelation::Int { id: _ }) => return true,
        (AstRelation::Void { id: _ }, AstRelation::Void { id: _ }) => return true,
        (
            AstRelation::Arg {
                id: _,
                var_name: var_name1,
                type_id: type_id1,
            },
            AstRelation::Arg {
                id: _,
                var_name: var_name2,
                type_id: type_id2,
            },
        ) => {
            return var_name1 == var_name2
                && relations_match(
                    &t1.get_relation(*type_id1),
                    &t2.get_relation(*type_id2),
                    t1,
                    t2,
                )
        }
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
                arg1_id: arg1_id1,
                arg2_id: arg2_id1,
            },
            AstRelation::BinaryOp {
                id: _,
                arg1_id: arg1_id2,
                arg2_id: arg2_id2,
            },
        ) => {
            return relations_match(
                &t1.get_relation(*arg1_id1),
                &t2.get_relation(*arg1_id2),
                t1,
                t2,
            ) && relations_match(
                &t1.get_relation(*arg2_id1),
                &t2.get_relation(*arg2_id2),
                t1,
                t2,
            )
        }
        (
            AstRelation::EndItem {
                id: _,
                stmt_id: stmt_id1,
            },
            AstRelation::EndItem {
                id: _,
                stmt_id: stmt_id2,
            },
        ) => {
            return relations_match(
                &t1.get_relation(*stmt_id1),
                &t2.get_relation(*stmt_id2),
                t1,
                t2,
            )
        }
        (
            AstRelation::Item {
                id: _,
                stmt_id: stmt_id1,
                next_stmt_id: next_stmt_id1,
            },
            AstRelation::Item {
                id: _,
                stmt_id: stmt_id2,
                next_stmt_id: next_stmt_id2,
            },
        ) => {
            return relations_match(
                &t1.get_relation(*stmt_id1),
                &t2.get_relation(*stmt_id2),
                t1,
                t2,
            ) && relations_match(
                &t1.get_relation(*next_stmt_id1),
                &t2.get_relation(*next_stmt_id2),
                t1,
                t2,
            )
        }
        (
            AstRelation::Compound {
                id: _,
                start_id: start_id1,
            },
            AstRelation::Compound {
                id: _,
                start_id: start_id2,
            },
        ) => {
            return relations_match(
                &t1.get_relation(*start_id1),
                &t2.get_relation(*start_id2),
                t1,
                t2,
            )
        }
        (
            AstRelation::While {
                id: _,
                cond_id: cond_id1,
                body_id: body_id1,
            },
            AstRelation::While {
                id: _,
                cond_id: cond_id2,
                body_id: body_id2,
            },
        ) => {
            return relations_match(
                &t1.get_relation(*body_id1),
                &t2.get_relation(*body_id2),
                t1,
                t2,
            ) && relations_match(
                &t1.get_relation(*cond_id1),
                &t2.get_relation(*cond_id2),
                t1,
                t2,
            )
        }
        (
            AstRelation::If {
                id: _,
                cond_id: cond_id1,
                then_id: then_id1,
            },
            AstRelation::If {
                id: _,
                cond_id: cond_id2,
                then_id: then_id2,
            },
        ) => {
            return relations_match(
                &t1.get_relation(*then_id1),
                &t2.get_relation(*then_id2),
                t1,
                t2,
            ) && relations_match(
                &t1.get_relation(*cond_id1),
                &t2.get_relation(*cond_id2),
                t1,
                t2,
            )
        }
        (
            AstRelation::IfElse {
                id: _,
                cond_id: cond_id1,
                then_id: then_id1,
                else_id: else_id1,
            },
            AstRelation::IfElse {
                id: _,
                cond_id: cond_id2,
                then_id: then_id2,
                else_id: else_id2,
            },
        ) => {
            return relations_match(
                &t1.get_relation(*then_id1),
                &t2.get_relation(*then_id2),
                t1,
                t2,
            ) && relations_match(
                &t1.get_relation(*cond_id1),
                &t2.get_relation(*cond_id2),
                t1,
                t2,
            ) && relations_match(
                &t1.get_relation(*else_id1),
                &t2.get_relation(*else_id2),
                t1,
                t2,
            )
        }
        (
            AstRelation::Return {
                id: _,
                expr_id: expr_id1,
            },
            AstRelation::Return {
                id: _,
                expr_id: expr_id2,
            },
        ) => {
            return relations_match(
                &t1.get_relation(*expr_id1),
                &t2.get_relation(*expr_id2),
                t1,
                t2,
            )
        }
        (
            AstRelation::Assign {
                id: _,
                var_name: var_name1,
                type_id: type_id1,
                expr_id: expr_id1,
            },
            AstRelation::Assign {
                id: _,
                var_name: var_name2,
                type_id: type_id2,
                expr_id: expr_id2,
            },
        ) => {
            return var_name1 == var_name2
                && return relations_match(
                    &t1.get_relation(*type_id1),
                    &t2.get_relation(*type_id2),
                    t1,
                    t2,
                ) && return relations_match(
                    &t1.get_relation(*expr_id1),
                    &t2.get_relation(*expr_id2),
                    t1,
                    t2,
                )
        }
        (
            AstRelation::FunCall {
                id: _,
                fun_name: fun_name1,
                arg_ids: arg_ids1,
            },
            AstRelation::FunCall {
                id: _,
                fun_name: fun_name2,
                arg_ids: arg_ids2,
            },
        ) => {
            let mut args_result: bool = true;
            for (index, arg_id1) in arg_ids1.iter().enumerate() {
                if !relations_match(
                    &t1.get_relation(*arg_id1),
                    &t2.get_relation(arg_ids2[index]),
                    t1,
                    t2,
                ) {
                    args_result = false;
                }
            }
            return args_result && fun_name1 == fun_name2;
        }
        (
            AstRelation::FunDef {
                id: _,
                fun_name: _,
                return_type_id: _,
                arg_ids: _,
                body_id: _,
            },
            AstRelation::FunDef {
                id: _,
                fun_name: _,
                return_type_id: _,
                arg_ids: _,
                body_id: _,
            },
        ) => panic!("Called matching function on construct that should be handled on higher level"),
        (
            AstRelation::TransUnit { id: _, body_ids: _ },
            AstRelation::TransUnit { id: _, body_ids: _ },
        ) => panic!("Called matching function on construct that should be handled on higher level"),
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
        AstRelation::While {
            id,
            cond_id: _,
            body_id: _,
        } => return *id,
        AstRelation::IfElse {
            id,
            cond_id: _,
            then_id: _,
            else_id: _,
        } => return *id,
        AstRelation::If {
            id,
            cond_id: _,
            then_id: _,
        } => return *id,
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
mod tests {
    #[test]
    fn delete_whole_tree() {}
    #[test]
    fn insert_whole_tree() {}
}
