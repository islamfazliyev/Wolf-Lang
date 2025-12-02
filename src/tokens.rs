#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Types
    TypeString,
    TypeInt,
    TypeFloat,
    TypeBool,
    TypeList(Box<Token>),

    // Keywords
    Let,
    Print,

    // Identifiers and literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Token>),

    // Operators
    Assign,    // =
    Plus,
    Minus,
    Multiply,
    Divide,
    Dot,

    // Parantez / blok
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    //Conditions
    If,
    For,
    While,

    //Condition Operators
    Equals,
    Greater,
    Lesser,
    GreaterEquals,
    LesserEquals,
    NotEquals,

    //Functions
    Func,

    // other
    EndOfCondition,
    Range,
    Comma,
    Colon,
    Return,
    Unknown,
    EOF,
}
