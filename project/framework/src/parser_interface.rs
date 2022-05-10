extern crate lang_c;

use lang_c::ast as parse_ast;
use lang_c::driver::{parse, Config};
use lang_c::print::Printer;
use lang_c::span::Span;
use lang_c::visit::*;

use crate::ast::Tree;
use crate::definitions::{AstRelation, ID};

pub fn parse_file_into_ast(file_path: &String) -> Tree {
    parse_with_lang_c(file_path)
}

fn parse_with_lang_c(file_path: &String) -> Tree {
    let config = Config::default();
    let parse_output = parse(&config, file_path);
    match parse_output {
        Ok(parse) => {
            let s = &mut String::new();
            Printer::new(s).visit_translation_unit(&parse.unit);
            println!("{}", s);
            let mut ast_builder = AstBuilder::new();
            return AstBuilder::build_tree(&mut ast_builder, &parse.unit);
        }
        Err(e) => {
            panic!("Error during parsing: {:?}", e)
        }
    }
}

struct AstBuilder {
    tree: Tree,
    current_max_id: ID,
}

// Traverse the parser output creating internal AST tree while keeping IDs consistent between nodes and relations.
// Uses a pattern similar to the Visit module in lang_c.
impl<'a> AstBuilder {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            current_max_id: 0,
        }
    }

    pub fn build_tree(&mut self, node: &'a parse_ast::TranslationUnit) -> Tree {
        Self::visit_translation_unit(self, node)
    }

    // For now we will assume a single translation unit as root of tree.
    fn visit_translation_unit(&mut self, node: &'a parse_ast::TranslationUnit) -> Tree {
        let mut body_ids = vec![];
        for element in &node.0 {
            body_ids.push(self.visit_external_declaration(&element.node, &element.span));
        }
        let node_id = self.current_max_id;
        self.current_max_id = self.current_max_id + 1;
        let relation = AstRelation::TransUnit {
            id: node_id,
            body_ids: body_ids.clone(),
        };
        self.tree.add_root_node(node_id, relation);
        self.tree.replace_children(node_id, body_ids);
        return self.tree.clone();
    }

    // At the moment all declarations just traverse down to function definitions.
    fn visit_external_declaration(
        &mut self,
        node: &'a parse_ast::ExternalDeclaration,
        _span: &'a Span,
    ) -> ID {
        match *node {
            // No new node created here, just traverse.
            parse_ast::ExternalDeclaration::FunctionDefinition(ref f) => {
                return self.visit_function_definition(&f.node, &f.span)
            }
            _ => panic!("Feature not implemented"),
        }
    }

    // A function definition results in multiple nodes.
    fn visit_function_definition(
        &mut self,
        node: &'a parse_ast::FunctionDefinition,
        _span: &'a Span,
    ) -> ID {
        // Get return type node ID (after creating node).
        // We are for now assuming that there is only a type specifier (in any case, it will just get the last specifier).
        let mut return_type_id = 0;
        for specifier in &node.specifiers {
            return_type_id = self.visit_declaration_specifier(&specifier.node, &specifier.span);
        }
        // Get function body compound ID (after creating node).
        let body_id = self.visit_statement(&node.statement.node, &node.statement.span);
        // We'll create the function definition node in the declarator since it hold most of the information.
        return self.visit_declarator_for_function(
            &node.declarator.node,
            &node.declarator.span,
            return_type_id,
            body_id,
        );
    }

    fn visit_declaration_specifier(
        &mut self,
        node: &'a parse_ast::DeclarationSpecifier,
        _span: &'a Span,
    ) -> ID {
        match *node {
            parse_ast::DeclarationSpecifier::TypeSpecifier(ref t) => {
                return self.visit_type_specifier(&t.node, &t.span)
            }
            _ => panic!("Feature not implemented"),
        }
    }

    fn visit_type_specifier(&mut self, node: &'a parse_ast::TypeSpecifier, _span: &'a Span) -> ID {
        match *node {
            parse_ast::TypeSpecifier::Void => {
                let node_id = self.current_max_id;
                self.current_max_id = self.current_max_id + 1;
                let relation = AstRelation::Void { id: node_id };
                self.tree.add_node(node_id, relation);
                return node_id;
            }
            parse_ast::TypeSpecifier::Int => {
                let node_id = self.current_max_id;
                self.current_max_id = self.current_max_id + 1;
                let relation = AstRelation::Int { id: node_id };
                self.tree.add_node(node_id, relation);
                return node_id;
            }
            parse_ast::TypeSpecifier::Char => {
                let node_id = self.current_max_id;
                self.current_max_id = self.current_max_id + 1;
                let relation = AstRelation::Char { id: node_id };
                self.tree.add_node(node_id, relation);
                return node_id;
            }
            parse_ast::TypeSpecifier::Float => {
                let node_id = self.current_max_id;
                self.current_max_id = self.current_max_id + 1;
                let relation = AstRelation::Float { id: node_id };
                self.tree.add_node(node_id, relation);
                return node_id;
            }
            _ => panic!("Feature not implemented"),
        }
    }

    fn visit_statement(&mut self, node: &'a parse_ast::Statement, _span: &'a Span) -> ID {
        match *node {
            parse_ast::Statement::Compound(ref c) => {
                // TO-DO: check whether there's a better way to initialize this.
                let mut next_stmt_id = 0;
                let mut start_id = 0;
                let mut counter = 0;
                // We will traverse the compound backwards in order to link the block items.
                for item in c.iter().rev() {
                    let stmt_id = self.visit_block_item(&item.node, &item.span);
                    // Case: last item in compound.
                    if counter == 0 {
                        let node_id = self.current_max_id;
                        self.current_max_id = self.current_max_id + 1;
                        let relation = AstRelation::EndItem {
                            id: node_id,
                            stmt_id,
                        };
                        self.tree.add_node(node_id, relation);
                        self.tree.link_child(node_id, stmt_id);
                        next_stmt_id = node_id;
                        // Case: first item in compound (could also be last).
                        if counter == c.len() - 1 {
                            start_id = node_id;
                        }
                    } else {
                        let node_id = self.current_max_id;
                        self.current_max_id = self.current_max_id + 1;
                        let relation = AstRelation::Item {
                            id: node_id,
                            stmt_id,
                            next_stmt_id,
                        };
                        self.tree.add_node(node_id, relation);
                        self.tree.link_child(node_id, stmt_id);
                        self.tree.link_child(node_id, next_stmt_id);
                        next_stmt_id = node_id;
                        // Case: first item in compound (could also be last).
                        if counter == c.len() - 1 {
                            start_id = node_id;
                        }
                    }
                    counter = counter + 1;
                }
                let node_id = self.current_max_id;
                self.current_max_id = self.current_max_id + 1;
                let relation = AstRelation::Compound {
                    id: node_id,
                    start_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, start_id);
                return node_id;
            }
            parse_ast::Statement::Expression(Some(ref e)) => {
                return self.visit_expression(&e.node, &e.span)
            }
            parse_ast::Statement::Return(Some(ref r)) => {
                let expr_id = self.visit_expression(&r.node, &r.span);
                let node_id = self.current_max_id;
                self.current_max_id = self.current_max_id + 1;
                let relation = AstRelation::Return {
                    id: node_id,
                    expr_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, expr_id);
                return node_id;
            }
            parse_ast::Statement::If(ref i) => {
                return self.visit_if_statement(&i.node, &i.span);
            }
            parse_ast::Statement::While(ref w) => {
                return self.visit_while_statement(&w.node, &w.span);
            }
            _ => panic!("Feature not implemented"),
        }
    }

    fn visit_block_item(&mut self, node: &'a parse_ast::BlockItem, _span: &'a Span) -> ID {
        match *node {
            parse_ast::BlockItem::Statement(ref s) => {
                return self.visit_statement(&s.node, &s.span)
            }
            parse_ast::BlockItem::Declaration(ref d) => {
                return self.visit_declaration(&d.node, &d.span)
            }
            _ => panic!("Feature not implemented"),
        }
    }

    // Currently just deals with normal assignments.
    fn visit_declaration(&mut self, node: &'a parse_ast::Declaration, _span: &'a Span) -> ID {
        let mut type_id = 0;
        for specifier in &node.specifiers {
            type_id = self.visit_declaration_specifier(&specifier.node, &specifier.span);
        }
        return self.visit_init_declarator(
            &node.declarators[0].node,
            &node.declarators[0].span,
            type_id,
        );
    }

    fn visit_init_declarator(
        &mut self,
        node: &'a parse_ast::InitDeclarator,
        _span: &'a Span,
        type_id: ID,
    ) -> ID {
        let var_name = self.visit_declarator(&node.declarator.node, &node.declarator.span);
        if let Some(ref initializer) = node.initializer {
            match initializer.node {
                parse_ast::Initializer::Expression(ref e) => {
                    let expr_id = self.visit_expression(&e.node, &e.span);
                    let node_id = self.current_max_id;
                    self.current_max_id = self.current_max_id + 1;
                    let relation = AstRelation::Assign {
                        id: node_id,
                        var_name: var_name.clone(),
                        type_id,
                        expr_id,
                    };
                    self.tree.add_node(node_id, relation);
                    self.tree.link_child(node_id, type_id);
                    self.tree.link_child(node_id, expr_id);
                    return node_id;
                }
                _ => panic!("Feature not implemented"),
            }
        } else {
            panic!("Feature not implemented")
        }
    }

    fn visit_while_statement(
        &mut self,
        node: &'a parse_ast::WhileStatement,
        _span: &'a Span,
    ) -> ID {
        let cond_id = self.visit_expression(&node.expression.node, &node.expression.span);
        let body_id = self.visit_statement(&node.statement.node, &node.statement.span);
        let node_id = self.current_max_id;
        self.current_max_id = self.current_max_id + 1;
        let relation = AstRelation::While {
            id: node_id,
            cond_id,
            body_id,
        };
        self.tree.add_node(node_id, relation);
        self.tree.link_child(node_id, cond_id);
        self.tree.link_child(node_id, body_id);
        return node_id;
    }

    fn visit_if_statement(&mut self, node: &'a parse_ast::IfStatement, _span: &'a Span) -> ID {
        let cond_id = self.visit_expression(&node.condition.node, &node.condition.span);
        let then_id = self.visit_statement(&node.then_statement.node, &node.then_statement.span);
        if let Some(ref e) = node.else_statement {
            let else_id = self.visit_statement(&e.node, &e.span);
            let node_id = self.current_max_id;
            self.current_max_id = self.current_max_id + 1;
            let relation = AstRelation::IfElse {
                id: node_id,
                cond_id,
                then_id,
                else_id,
            };
            self.tree.add_node(node_id, relation);
            self.tree.link_child(node_id, cond_id);
            self.tree.link_child(node_id, then_id);
            self.tree.link_child(node_id, else_id);
            return node_id;
        } else {
            let node_id = self.current_max_id;
            self.current_max_id = self.current_max_id + 1;
            let relation = AstRelation::If {
                id: node_id,
                cond_id,
                then_id,
            };
            self.tree.add_node(node_id, relation);
            self.tree.link_child(node_id, cond_id);
            self.tree.link_child(node_id, then_id);
            return node_id;
        }
    }

    fn visit_expression(&mut self, node: &'a parse_ast::Expression, _span: &'a Span) -> ID {
        match *node {
            parse_ast::Expression::Identifier(ref i) => {
                let var_name = i.node.name.clone();
                let node_id = self.current_max_id;
                self.current_max_id = self.current_max_id + 1;
                let relation = AstRelation::Var {
                    id: node_id,
                    var_name: var_name.clone(),
                };
                self.tree.add_node(node_id, relation);
                return node_id;
            }
            parse_ast::Expression::Constant(ref c) => return self.visit_constant(&c.node, &c.span),
            parse_ast::Expression::Call(ref c) => {
                return self.visit_call_expression(&c.node, &c.span)
            }
            parse_ast::Expression::BinaryOperator(ref b) => {
                return self.visit_binary_operator_expression(&b.node, &b.span)
            }
            parse_ast::Expression::Statement(ref s) => self.visit_statement(&s.node, &s.span),
            _ => panic!("Feature not implemented"),
        }
    }

    fn visit_call_expression(
        &mut self,
        node: &'a parse_ast::CallExpression,
        _span: &'a Span,
    ) -> ID {
        let fun_name;
        match node.callee.node {
            parse_ast::Expression::Identifier(ref i) => fun_name = i.node.name.clone(),
            _ => panic!("Expected a function identifier"),
        }
        let mut arg_ids = vec![];
        for argument in &node.arguments {
            arg_ids.push(self.visit_expression(&argument.node, &argument.span))
        }
        let node_id = self.current_max_id;
        self.current_max_id = self.current_max_id + 1;
        let relation = AstRelation::FunCall {
            id: node_id,
            fun_name: fun_name.clone(),
            arg_ids: arg_ids.clone(),
        };
        self.tree.add_node(node_id, relation);
        self.tree.replace_children(node_id, arg_ids);
        return node_id;
    }

    fn visit_binary_operator_expression(
        &mut self,
        node: &'a parse_ast::BinaryOperatorExpression,
        _span: &'a Span,
    ) -> ID {
        let arg1_id = self.visit_expression(&node.lhs.node, &node.lhs.span);
        let arg2_id = self.visit_expression(&node.rhs.node, &node.rhs.span);
        let node_id = self.current_max_id;
        self.current_max_id = self.current_max_id + 1;
        match node.operator.node {
            parse_ast::BinaryOperator::Plus => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::Minus => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::Multiply => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::Divide => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::Greater => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::GreaterOrEqual => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::Less => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::LessOrEqual => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::Equals => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::LogicalAnd => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::LogicalOr => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            parse_ast::BinaryOperator::Assign => {
                let relation = AstRelation::BinaryOp {
                    id: node_id,
                    arg1_id,
                    arg2_id,
                };
                self.tree.add_node(node_id, relation);
                self.tree.link_child(node_id, arg1_id);
                self.tree.link_child(node_id, arg2_id);
                return node_id;
            }
            _ => panic!("Feature not implemented"),
        }
    }

    fn visit_constant(&mut self, node: &'a parse_ast::Constant, _span: &'a Span) -> ID {
        let node_id = self.current_max_id;
        self.current_max_id = self.current_max_id + 1;
        match *node {
            parse_ast::Constant::Integer(_) => {
                let relation = AstRelation::Int { id: node_id };
                self.tree.add_node(node_id, relation);
                return node_id;
            }
            parse_ast::Constant::Float(_) => {
                let relation = AstRelation::Float { id: node_id };
                self.tree.add_node(node_id, relation);
                return node_id;
            }
            parse_ast::Constant::Character(_) => {
                let relation = AstRelation::Char { id: node_id };
                self.tree.add_node(node_id, relation);
                return node_id;
            }
        }
    }

    // Put together function definition node.
    fn visit_declarator_for_function(
        &mut self,
        node: &'a parse_ast::Declarator,
        _span: &'a Span,
        return_type_id: ID,
        body_id: ID,
    ) -> ID {
        let fun_name = self.visit_declarator_kind(&node.kind.node, &node.kind.span);
        // TO-DO: figure out in which case you have multiple derived declarators/what extensions are.
        let mut arg_ids = vec![];
        for derived in &node.derived {
            arg_ids = self.visit_derived_declarator(&derived.node, &derived.span);
        }
        let node_id = self.current_max_id;
        self.current_max_id = self.current_max_id + 1;
        let relation = AstRelation::FunDef {
            id: node_id,
            fun_name: fun_name.clone(),
            return_type_id,
            arg_ids: arg_ids.clone(),
            body_id,
        };
        self.tree.add_node(node_id, relation);
        self.tree.replace_children(node_id, arg_ids);
        self.tree.link_child(node_id, return_type_id);
        self.tree.link_child(node_id, body_id);
        return node_id;
    }

    // Get function name.
    fn visit_declarator_kind(
        &mut self,
        node: &'a parse_ast::DeclaratorKind,
        _span: &'a Span,
    ) -> String {
        match *node {
            parse_ast::DeclaratorKind::Identifier(ref i) => return i.node.name.clone(),
            _ => panic!("Feature not implemented"),
        }
    }

    // Traverse to function declarator (for now, will need this for other declarators later too).
    fn visit_derived_declarator(
        &mut self,
        node: &'a parse_ast::DerivedDeclarator,
        _span: &'a Span,
    ) -> Vec<ID> {
        match *node {
            parse_ast::DerivedDeclarator::Function(ref f) => {
                return self.visit_function_declarator(&f.node, &f.span)
            }
            _ => panic!("Feature not implemented"),
        }
    }

    fn visit_function_declarator(
        &mut self,
        node: &'a parse_ast::FunctionDeclarator,
        _span: &'a Span,
    ) -> Vec<ID> {
        let mut arg_ids = vec![];
        for arg in &node.parameters {
            arg_ids.push(self.visit_parameter_declaration(&arg.node, &arg.span));
        }
        return arg_ids;
    }

    fn visit_parameter_declaration(
        &mut self,
        node: &'a parse_ast::ParameterDeclaration,
        _span: &'a Span,
    ) -> ID {
        let mut type_id = 0;
        for specifier in &node.specifiers {
            type_id = self.visit_declaration_specifier(&specifier.node, &specifier.span);
        }
        let var_name;
        if let Some(ref declarator) = node.declarator {
            var_name = self.visit_declarator(&declarator.node, &declarator.span);
        } else {
            var_name = String::from("");
        }
        let node_id = self.current_max_id;
        self.current_max_id = self.current_max_id + 1;
        let relation = AstRelation::Arg {
            id: node_id,
            var_name: var_name.clone(),
            type_id,
        };
        self.tree.add_node(node_id, relation);
        self.tree.link_child(node_id, type_id);
        return node_id;
    }

    // Separate method for argument declarator since we only need the variable name from here.
    fn visit_declarator(&mut self, node: &'a parse_ast::Declarator, _span: &'a Span) -> String {
        return self.visit_declarator_kind(&node.kind.node, &node.kind.span);
    }
}

#[cfg(test)]
mod tests {
    use crate::parser_interface;

    // Run with "cargo test print_for_debug -- --show-output".
    #[test]
    fn print_for_debug() {
        parser_interface::parse_with_lang_c(&String::from("./tests/dev_examples/c/example2.c"))
            .pretty_print();
    }
}
