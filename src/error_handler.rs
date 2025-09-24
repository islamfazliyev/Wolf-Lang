use crate::tokens::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken { expected: Token, found: Option<Token> },
    UnkownType { type_name: String},
}