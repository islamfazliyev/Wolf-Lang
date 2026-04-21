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
    Else,
    For,
    While,

    //Condition Operators
    Equals,
    Greater,
    Lesser,
    GreaterEquals,
    LesserEquals,
    NotEquals,
    And,
    Or,

    //Functions
    Func,

    Import,


    //struct-impl
    Struct,
    Impl,

    StructInstance {
        type_name: String,
        fields: Vec<(String, Token)>,
    },
    
    // other
    EndOfCondition,
    Range,
    Comma,
    Colon,
    Return,
    Bang,
    As,
    Unknown,
    EOF,
}
