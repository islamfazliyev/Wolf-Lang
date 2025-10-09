#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Types
    TypeString,
    TypeNumber,
    TypeBool,

    // Keywords
    Let,
    Print,

    // Identifiers and literals
    Identifier(String),
    Number(f64),
    String(String),
    Boolean(bool),

    // Operators
    Assign,    // =
    Plus,
    Minus,
    Multiply,
    Divide,

    // Parantez / blok
    LParen,
    RParen,
    LBrace,
    RBrace,

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

    // other
    EndOfCondition,
    Semicolon,
    EOF,
}
