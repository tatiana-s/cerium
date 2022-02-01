extern crate lang_c;

use lang_c::ast as parse_ast;
use lang_c::driver::{parse, Config};
use lang_c::print::Printer;
use lang_c::span::Span;
use lang_c::visit::*;

use crate::ast::AstNode;
use crate::definitions::{AstRelation, InternalError};

pub fn parse_with_lang_c(file_path: &String) -> Result<AstNode, InternalError> {
    let config = Config::default();
    let parse_output = parse(&config, file_path);
    match parse_output {
        Ok(parse) => {
            let s = &mut String::new();
            Printer::new(s).visit_translation_unit(&parse.unit);
            println!("{}", s);
            Err(InternalError::TransformError)
        }
        Err(e) => {
            println!("error: {:?}", e);
            Err(InternalError::ParseError)
        }
    }
}

struct AstBuilder {
    tree: AstNode,
}

impl<'a> Visit<'a> for AstBuilder {
    fn visit_translation_unit(&mut self, node: &'a parse_ast::TranslationUnit) {
        let rel = AstRelation::TransUnit {
            id: 0,
            body_ids: vec![],
        };
        println!("Added TransUnit to tree!");
        self.tree = AstNode::new(rel);
        for element in &node.0 {
            self.visit_external_declaration(&element.node, &element.span);
        }
    }

    fn visit_external_declaration(
        &mut self,
        node: &'a parse_ast::ExternalDeclaration,
        _span: &'a Span,
    ) {
        match *node {
            parse_ast::ExternalDeclaration::FunctionDefinition(ref f) => {
                self.visit_function_definition(&f.node, &f.span)
            }
            _ => panic!("Feature not implemented"),
        }
    }

    fn visit_function_definition(
        &mut self,
        node: &'a parse_ast::FunctionDefinition,
        _span: &'a Span,
    ) {
        // Instead of using interface write your own functions which can simply return names?
    }
}
