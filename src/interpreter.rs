use std::collections::HashMap;
use crate::{ast::Stmt, tokens::Token};


#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Token)>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpreter {
    pub scopes: Vec<HashMap<String, Token>>,
    pub functions: HashMap<String, Function>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }
    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        
    }
}