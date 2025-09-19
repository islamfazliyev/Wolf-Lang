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

    // other
    Semicolon,
    EOF,
}
