pub mod tokens;
pub mod lexer;
pub mod parser;
pub mod error_handler;
pub mod stdlib; 

use std::collections::HashMap;
use parser::Parser;
use lexer::lexer;
use tokens::Token;

pub struct WolfEngine {
    globals: HashMap<String, Token>,
}

impl WolfEngine {
    pub fn new() -> Self {
        WolfEngine {
            globals: HashMap::new(),
        }
    }

    pub fn push_int(&mut self, name: &str, value: i64) {
        self.globals.insert(name.to_string(), Token::Integer(value));
    }

    pub fn push_str(&mut self, name: &str, value: &str) {
        self.globals.insert(name.to_string(), Token::String(value.to_string()));
    }

    pub fn push_bool(&mut self, name: &str, value: bool) {
        self.globals.insert(name.to_string(), Token::Boolean(value));
    }

    pub fn run(&mut self, content: &str) -> Result<(), String> {
        let tokens = match lexer(content) {
            Ok(t) => t,
            Err(e) => return Err(format!("Lexer Error: {}", e)),
        };

        let mut parser = Parser::new(tokens);

        if let Some(scope) = parser.scopes.first_mut() {
            for (k, v) in &self.globals {
                scope.insert(k.clone(), v.clone());
            }
        }

        // Parser çalıştırılır
        while parser.current_token().is_some() {
            if let Err(e) = parser.sense() {
                return Err(format!("Parser Error: {:?}", e));
            }
        }

        if let Some(final_scope) = parser.scopes.first() {
            self.globals = final_scope.clone();
        }

        
        
        Ok(())
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        if let Some(Token::Integer(n)) = self.globals.get(name) {
            Some(*n)
        } else {
            None
        }
    }
    
}

pub fn run_script(content: &str) -> Result<(), String> {
    let mut engine = WolfEngine::new();
    engine.run(content)
}