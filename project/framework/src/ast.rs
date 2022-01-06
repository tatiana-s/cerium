// Type aliases for consistency and easy changes.
type Id = u32;
type Scope = (u32, u32);

// Defines the permitted language constructs.
enum NodeType {
    TranslationUnit {
        id: Id,
        body_ids: Vec<Id>,
    },
    FunctionDef {
        id: Id,
        fun_name: String,
        return_id: Id,
        arg_ids: Vec<Id>,
    },
    FunctionCall {
        id: Id,
        fun_id: Id,
        arg_ids: Vec<Id>,
    },
    // Statements.
    Block {
        ids: Vec<Id>,
    },
    Assignment {
        id: Id,
        var_name: String,
        expr_id: Id,
    },
    Return {
        id: Id,
        expr_id: Id,
    },
    // Expressions.
    BinaryOp {
        id: Id,
        arg1_id: Id,
        ar2_id: Id,
    },
    // Leaf types.
    Void {
        id: Id,
    },
    Int {
        id: Id,
    },
    Float {
        id: Id,
    },
    Char {
        id: Id,
    },
}

// Building block of AST.
struct AstNode {
    node_id: Id,
    node_type: NodeType,
    scope: Scope,
    children: Vec<AstNode>,
}

impl AstNode {}

pub fn parse_file_into_ast(file_path: &String) {}

#[cfg(test)]
mod tests {}
