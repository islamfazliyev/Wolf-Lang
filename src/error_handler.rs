use crate::tokens::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken { expected: Token, found: Option<Token>, line: usize },
    UnkownType { type_name: String, line: usize },
    UndeclaredVariable { name: String, line: usize },
    TypeMismatch { expected: Token, found: Token, line: usize },
    Return { value: Token }, 

    RuntimeError { message: String, line: usize },
}