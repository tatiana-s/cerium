use crate::ast::RelationTree as Tree;
use crate::definitions::{AstRelation, ID};
use std::collections::HashMap;

#[derive(PartialEq, Clone)]
enum Type {
    VoidType,
    IntType,
    FloatType,
    CharType,
    OkType,
    ErrorType,
}

#[derive(PartialEq, Clone)]
struct FunType {
    return_type: Type,
    arg_types: Vec<Type>,
}

pub fn type_check(ast: &Tree) -> bool {
    let root_index = ast.get_root();
    let var_context: HashMap<String, Type> = HashMap::new();
    let fun_context: HashMap<String, FunType> = HashMap::new();
    type_check_trans_unit(ast.get_relation(root_index), &ast, var_context, fun_context)
        == Type::OkType
}

// Traverse the AST to type-check the program recursively.
fn type_check_trans_unit(
    node: AstRelation,
    ast: &Tree,
    var_context: HashMap<String, Type>,
    fun_context: HashMap<String, FunType>,
) -> Type {
    match node {
        AstRelation::TransUnit { id: _, body_ids } => {
            let mut body_correct = true;
            for body_id in body_ids {
                match type_check_fun_def(
                    ast.get_relation(body_id),
                    ast,
                    var_context.clone(),
                    fun_context.clone(),
                ) {
                    Type::ErrorType => body_correct = false,
                    Type::OkType => {}
                    _ => panic!("Unexpected type"),
                }
            }
            if body_correct {
                return Type::OkType;
            }
            return Type::ErrorType;
        }
        _ => panic!("Unexpected syntax"),
    }
}

fn type_check_fun_def(
    node: AstRelation,
    ast: &Tree,
    var_context: HashMap<String, Type>,
    fun_context: HashMap<String, FunType>,
) -> Type {
    match node {
        AstRelation::FunDef {
            id: _,
            fun_name,
            return_type_id,
            arg_ids,
            body_id,
        } => {
            let return_type = type_check_literal(&ast.get_relation(return_type_id));
            let (new_var_context, arg_types) = bind_arguments(arg_ids, var_context, ast);
            let mut new_fun_context = fun_context.clone();
            new_fun_context.insert(
                fun_name.clone(),
                FunType {
                    return_type,
                    arg_types,
                },
            );
            return type_check_compound(
                &ast.get_relation(body_id),
                ast,
                new_var_context,
                new_fun_context,
                fun_name,
            );
        }
        _ => panic!("Unexpected syntax"),
    }
}

fn bind_arguments(
    arg_ids: Vec<ID>,
    var_context: HashMap<String, Type>,
    ast: &Tree,
) -> (HashMap<String, Type>, Vec<Type>) {
    let mut new_var_context = var_context.clone();
    let mut arg_types = vec![];
    for arg_id in arg_ids {
        let arg_relation = &ast.get_relation(arg_id);
        match arg_relation {
            AstRelation::Arg {
                id: _,
                var_name,
                type_id,
            } => {
                let arg_type = type_check_literal(&ast.get_relation(*type_id));
                new_var_context.insert(var_name.clone(), arg_type.clone());
                arg_types.push(arg_type);
            }
            _ => panic!("Unexpected syntax"),
        }
    }
    (new_var_context, arg_types)
}

fn type_check_compound(
    node: &AstRelation,
    ast: &Tree,
    var_context: HashMap<String, Type>,
    fun_context: HashMap<String, FunType>,
    current_fun: String,
) -> Type {
    match *node {
        AstRelation::Compound { id: _, start_id } => {
            return type_check_item(
                ast.get_relation(start_id),
                ast,
                var_context,
                fun_context,
                current_fun,
            )
        }
        _ => panic!("Unexpected syntax"),
    }
}

fn type_check_item(
    node: AstRelation,
    ast: &Tree,
    var_context: HashMap<String, Type>,
    fun_context: HashMap<String, FunType>,
    current_fun: String,
) -> Type {
    match node {
        AstRelation::Item {
            id: _,
            stmt_id,
            next_stmt_id,
        } => {
            match type_check_statement(
                ast.get_relation(stmt_id),
                ast,
                var_context.clone(),
                fun_context.clone(),
                current_fun.clone(),
            ) {
                (Type::OkType, new_var_context) => {
                    return type_check_item(
                        ast.get_relation(next_stmt_id),
                        ast,
                        new_var_context,
                        fun_context,
                        current_fun,
                    )
                }
                (Type::ErrorType, _) => Type::ErrorType,
                _ => panic!("Unexpected type"),
            }
        }
        AstRelation::EndItem { id: _, stmt_id } => {
            return type_check_statement(
                ast.get_relation(stmt_id),
                ast,
                var_context,
                fun_context,
                current_fun,
            )
            .0
        }
        _ => panic!("Unexpected syntax"),
    }
}

// Since every expression can be a statement we will check them in one function.
fn type_check_statement(
    node: AstRelation,
    ast: &Tree,
    var_context: HashMap<String, Type>,
    fun_context: HashMap<String, FunType>,
    current_fun: String,
) -> (Type, HashMap<String, Type>) {
    match node {
        AstRelation::Assign {
            id: _,
            var_name,
            type_id,
            expr_id,
        } => {
            let assign_type = type_check_literal(&ast.get_relation(type_id));
            let (expr_type, new_var_context) = type_check_statement(
                ast.get_relation(expr_id),
                ast,
                var_context.clone(),
                fun_context.clone(),
                current_fun.clone(),
            );
            if assign_type == expr_type {
                let mut new_var_context = new_var_context.clone();
                new_var_context.insert(var_name.clone(), assign_type);
                return (Type::OkType, new_var_context);
            } else {
                return (Type::ErrorType, var_context.clone());
            }
        }
        AstRelation::Return { id: _, expr_id } => {
            let (expr_type, new_var_context) = type_check_statement(
                ast.get_relation(expr_id),
                ast,
                var_context.clone(),
                fun_context.clone(),
                current_fun.clone(),
            );
            let fun_type_option = fun_context.get(&current_fun);
            match fun_type_option {
                Some(fun_type) => {
                    if fun_type.return_type == expr_type {
                        return (Type::OkType, new_var_context);
                    } else {
                        return (Type::ErrorType, var_context);
                    }
                }
                None => panic!("Unexpected function name"),
            }
        }
        AstRelation::FunCall {
            id: _,
            fun_name,
            arg_ids,
        } => {
            let fun_type = fun_context.get(&fun_name).unwrap();
            let fun_types = fun_type.arg_types.clone();
            let mut counter = 0;
            for arg_id in arg_ids {
                let (arg_type, var_context) = type_check_statement(
                    ast.get_relation(arg_id),
                    ast,
                    var_context.clone(),
                    fun_context.clone(),
                    current_fun.clone(),
                );
                if fun_types[counter] != arg_type {
                    return (Type::ErrorType, var_context);
                }
                counter = counter + 1;
            }
            return (fun_type.return_type.clone(), var_context);
        }
        AstRelation::BinaryOp {
            id: _,
            arg1_id,
            arg2_id,
        } => {
            let (arg1_type, new_var_context) = type_check_statement(
                ast.get_relation(arg1_id),
                ast,
                var_context.clone(),
                fun_context.clone(),
                current_fun.clone(),
            );
            let (arg2_type, new_var_context) = type_check_statement(
                ast.get_relation(arg2_id),
                ast,
                new_var_context,
                fun_context.clone(),
                current_fun.clone(),
            );
            if arg1_type == arg2_type {
                match arg1_type {
                    Type::IntType => (Type::IntType, new_var_context),
                    Type::FloatType => (Type::FloatType, new_var_context),
                    _ => (Type::ErrorType, var_context.clone()),
                }
            } else {
                return (Type::ErrorType, var_context);
            }
        }
        AstRelation::Var { id: _, var_name } => match var_context.get(&var_name) {
            Some(var_type) => return (var_type.clone(), var_context),
            None => panic!("Unexpected variable name"),
        },
        AstRelation::Void { id: _ } => (Type::VoidType, var_context),
        AstRelation::Int { id: _ } => (Type::IntType, var_context),
        AstRelation::Float { id: _ } => (Type::FloatType, var_context),
        AstRelation::Char { id: _ } => (Type::CharType, var_context),
        _ => panic!("Unexpected syntax"),
    }
}

fn type_check_literal(node: &AstRelation) -> Type {
    match *node {
        AstRelation::Void { id: _ } => Type::VoidType,
        AstRelation::Int { id: _ } => Type::IntType,
        AstRelation::Float { id: _ } => Type::FloatType,
        AstRelation::Char { id: _ } => Type::CharType,
        _ => panic!("Unexpected syntax"),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser_interface;
    use crate::standard_type_checker::type_check;

    #[test]
    fn check_correct_program() {
        let ast = parser_interface::parse_file_into_initial_ast(&String::from(
            "./tests/dev_examples/c/example2.c",
        ));
        assert_eq!(type_check(&ast), true);
    }

    #[test]
    fn check_error_program() {
        let ast = parser_interface::parse_file_into_initial_ast(&String::from(
            "./tests/dev_examples/c/example3.c",
        ));
        assert_eq!(type_check(&ast), false);
    }
}
