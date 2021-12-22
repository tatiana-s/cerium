pub mod parser {

    use tree_sitter::{Language, Parser};

    extern "C" {
        fn tree_sitter_c() -> Language;
    }

    // Parse string and print resulting S-expression.
    pub fn parse_input_to_sexp(input: &String) {
        // Create parser
        let language = unsafe { tree_sitter_c() };
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();
        // Parse into tree and print.
        let tree = parser.parse(input, None).unwrap();
        println!("{}", tree.root_node().to_sexp());
    }
}
