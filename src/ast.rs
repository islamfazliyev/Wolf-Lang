use crate::tokens::Token;

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,
}


#[derive(Debug, Clone)]
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

    ListAssign {
        list_name: String,
        index: Box<Expr>, 
        value: Box<Expr>
    }

}



pub enum Stmt {

    Expression(Expr),

    Let {
        name: String,
        data_type: Token,
        value: Expr
    },
    Print(Expr),
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    For {
        var_name: String,
        start_value: Expr,
        end_value: Expr,
        body: Box<Stmt>,
    },
    Func {
        name: String,
        params: Vec<(String, Token)>,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    }

}