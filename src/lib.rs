pub mod tokens;
pub mod lexer;
pub mod parser;
pub mod error_handler;
pub mod stdlib;
pub mod native_functions;
pub mod ast;

use std::collections::HashMap;
use parser::Parser;
use lexer::lexer;
use tokens::Token;

use crate::{ast::Stmt, parser::Function};

pub struct WolfEngine {
    globals: HashMap<String, Token>,
    functions: HashMap<String, Function>,
}

impl WolfEngine {
    pub fn new() -> Self {
        WolfEngine {
            globals: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn push_int(&mut self, name: &str, value: i64) {
        self.globals.insert(name.to_string(), Token::Integer(value));
    }

    pub fn push_float(&mut self, name: &str, value: f64) {
        self.globals.insert(name.to_string(), Token::Float(value));
    }

    pub fn push_str(&mut self, name: &str, value: &str) {
        self.globals.insert(name.to_string(), Token::String(value.to_string()));
    }

    pub fn push_bool(&mut self, name: &str, value: bool) {
        self.globals.insert(name.to_string(), Token::Boolean(value));
    }

    pub fn push_list(&mut self, name: &str, value: Vec<Token>) {
        self.globals.insert(name.to_string(), Token::List(value));
    }

    // pub fn run(&mut self, content: &str) -> Result<(), String> {
    //     let tokens = match lexer(content) {
    //         Ok(t) => t,
    //         Err(e) => return Err(format!("Lexer Error: {}", e)),
    //     };

    //     let mut parser = Parser::new(tokens);

    //     if let Some(scope) = parser.scopes.first_mut() {
    //         for (k, v) in &self.globals {
    //             scope.insert(k.clone(), v.clone());
    //         }
    //     }

    //     // Parser çalıştırılır
    //     while parser.current_token().is_some() {
    //         if let Err(e) = parser.parse_statement() {
    //             return Err(format!("Parser Error: {:?}", e));
    //         }
    //     }

    //     if let Some(final_scope) = parser.scopes.first() {
    //         self.globals = final_scope.clone();
    //     }

        
        
    //     Ok(())
    // }

    pub fn run(&mut self, content: &str) -> Result<(), String> {
        // 1. Run Lexer
        let tokens = match lexer(content) {
            Ok(t) => t,
            Err(e) => return Err(format!("Lexer Error: {}", e)),
        };

        // 2. Initialize Parser
        let mut parser = Parser::new(tokens);
        let mut ast_tree: Vec<Stmt> = Vec::new();

        // 3. Loop and build the AST
        // We keep calling parse_statement() until we hit EOF
        while parser.current_token().is_some() && *parser.current_token().unwrap() != Token::EOF {
            match parser.parse_statement() {
                Ok(stmt) => ast_tree.push(stmt),
                Err(e) => return Err(format!("Parser Error: {:?}", e)),
            }
        }

        // 4. PRINT THE AST (This is your test!)
        // The {:#?} formatter checks if your Enum structures are correct
        println!("AST DEBUG OUTPUT:\n{:#?}", ast_tree); 

        Ok(())
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        if let Some(Token::Integer(n)) = self.globals.get(name) {
            Some(*n)
        } else {
            None
        }
    }

    pub fn get_float(&self, name: &str) -> Option<f64> {
        if let Some(Token::Float(n)) = self.globals.get(name) {
            Some(*n)
        } else {
            None
        }
    }

    pub fn get_str(&self, name: &str) -> Option<String> {
        if let Some(Token::String(s)) = self.globals.get(name) {
            Some(s.clone())
        } else {
            None
        }
    }
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        if let Some(Token::Boolean(b)) = self.globals.get(name) {
            Some(*b)
        } else {
            None
        }
    }

    pub fn get_list(&self, name: &str) -> Option<Vec<Token>> {
        if let Some(Token::List(l)) = self.globals.get(name) {
            Some(l.clone())
        } else {
            None
        }
    }
    
}

pub fn run_script(content: &str) -> Result<(), String> {
    let mut engine = WolfEngine::new();
    engine.run(content)
}


#[cfg(test)]
mod test
{
    use crate::{WolfEngine, tokens::Token};

    #[test]
    fn integer() {
        let mut engine = WolfEngine::new();
        engine.push_int("variable", 10);
        engine.run("print variable");
        let value = engine.get_int("variable");
        println!("{:?}", value);
    }

    #[test]
    fn float() {
        let mut engine = WolfEngine::new();
        engine.push_float("variable", 10.10);
        engine.run("print variable");
        let value = engine.get_float("variable");
        println!("{:?}", value);
    }

    #[test]
    fn strings() {
        let mut engine = WolfEngine::new();
        engine.push_str("variable", "hello");
        engine.run("print variable");
        let value = engine.get_str("variable");
        println!("{:?}", value);
    }

    #[test]
    fn bools() {
        let mut engine = WolfEngine::new();
        engine.push_bool("variable", false);
        engine.run("print variable");
        let value = engine.get_bool("variable");
        println!("{:?}", value);
    }

    #[test]
    fn lists() {
        let mut engine = WolfEngine::new();
        engine.push_list("variable", vec![Token::Integer(10)]);
        engine.run("print variable[0]");
        let value = engine.get_list("variable");
        println!("{:?}", value);
    }
}