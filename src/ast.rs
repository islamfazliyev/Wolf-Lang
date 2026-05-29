use crate::tokens::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StmtNode {
    pub stmt: Stmt,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {            
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),

    Literal(LiteralValue),
    Variable(String),

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Assign {
        name: String,
        value: Box<Expr>
    },

    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },

    List(Vec<Expr>),
    Index {
        list: Box<Expr>,
        index: Box<Expr>,
    },

    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    FieldGet {
        object: Box<Expr>,
        field: String,
    },
    FieldSet {
        object: Box<Expr>,
        field: String,
        value: Box<Expr>,
    }

}


#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {

    Expression(Expr),

    Let {
        name: String,
        data_type: Token,
        value: Expr
    },
    Print(Vec<Expr>),
    Block(Vec<StmtNode>),
    If {
        condition: Expr,
        then_branch: Box<StmtNode>,
        else_branch: Option<Box<StmtNode>>,
    },
    While {
        condition: Expr,
        body: Box<StmtNode>,
    },
    For {
        var_name: String,
        start_value: Expr,
        end_value: Expr,
        body: Box<StmtNode>,
    },
    Func {
        name: String,
        params: Vec<(String, Token)>,
        body: Vec<StmtNode>,
    },

    Struct {
        name: String,
        body: Vec<StmtNode>
    },

    Impl {
        name: String,
        body: Vec<StmtNode>
    },

    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    ListAssign {
        list_name: String,
        indices: Vec<Expr>, 
        value: Expr 
    },

    Import {
        directory: String,
        identifier: String
    },
}