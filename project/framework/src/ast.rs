// Type aliases for consistency and easy changes.
type ID = u32;
type Scope = (u32, u32);

// Defines the permitted language constructs.
enum NodeType {
    TranslationUnit {
        id: ID,
        body_ids: Vec<ID>,
    },
    FunctionDef {
        id: ID,
        fun_name: String,
        return_id: ID,
        arg_ids: Vec<ID>,
    },
    FunctionCall {
        id: ID,
        fun_id: ID,
        arg_ids: Vec<ID>,
    },
    // Statements.
    Block {
        ids: Vec<ID>,
    },
    Assignment {
        id: ID,
        var_name: String,
        expr_id: ID,
    },
    Return {
        id: ID,
        expr_id: ID,
    },
    // Expressions.
    BinaryOp {
        id: ID,
        arg1_id: ID,
        ar2_id: ID,
    },
    // Leaf types.
    Void {
        id: ID,
    },
    Int {
        id: ID,
    },
    Float {
        id: ID,
    },
    Char {
        id: ID,
    },
}

// Building block of AST.
struct AstNode {
    node_id: ID,
    node_type: NodeType,
    scope: Scope,
    children: Vec<AstNode>,
}

impl AstNode {}

pub fn parse_file_into_ast(file_path: &String) {}

#[cfg(test)]
mod tests {}
