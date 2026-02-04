use std::collections::HashMap;
use crate::{ast::{Stmt, Expr, LiteralValue}, tokens::Token, error_handler::ParseError};


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
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), ParseError> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), ParseError> {
        match stmt {
            Stmt::Print(expr) => {
                let value = self.evaluate(expr);
                self.print_token_value(&value)?;
                println!();
                Ok(())
            }

            Stmt::Let { name, data_type, value } => {
                let declared_value = self.evaluate(value);
                if self.check_type_compatibility(&data_type, &declared_value) {
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert(name, declared_value);
                    }
                    Ok(())
                } else {
                    Err(ParseError::TypeMismatch { 
                        expected: data_type, 
                        found: declared_value 
                    })
                }
            }

            _ => Ok(())
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Token {
        match expr {
            Expr::Literal(lit) => match lit {
                LiteralValue::Int(i) => Token::Integer(i),
                LiteralValue::Float(f) => Token::Float(f),
                LiteralValue::Str(s) => Token::String(s),
                LiteralValue::Bool(b) => Token::Boolean(b),
                LiteralValue::Nil => Token::Unknown,   
            }

            Expr::Variable(name) => {
                self.get_variable(&name).cloned().unwrap_or(Token::Unknown)
            }

            _ => Token::Unknown,
        }
    }

    fn get_variable(&self, name: &str) -> Option<&Token> {
        
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }

    fn print_token_value(&self, token: &Token) -> Result<(), ParseError> {
        match token {
            // Print literal values directly.
            Token::String(s) => print!("{} ", s),
            Token::Integer(n) => print!("{} ", n),
            Token::Float(f) => print!("{} ", f),
            Token::Boolean(b) => print!("{} ", b),
            Token::List(elements) => {
                for (i, element) in elements.iter().enumerate() {
                    self.print_token_value(element)?;

                    if i < elements.len() - 1 {
                        print!(", ");
                    }
                }
            },
            // If it's an identifier, we need to look up its value.
            Token::Identifier(name) => {
                if let Some(value_token) = self.get_variable(name) {
                    self.print_token_value(value_token)?;
                } else {
                    return Err(ParseError::UndeclaredVariable { name: name.clone() });
                }
            }
            // Error if trying to print something non-printable (like a keyword).
            _ => return Err(ParseError::UnexpectedToken {
                expected: Token::String("a printable value".to_string()),
                found: Some(token.clone()),
            })
        }
        Ok(())
    }

    fn check_type_compatibility(&self, expected_type: &Token, actual_value: &Token) -> bool {
        match (expected_type, actual_value) {
            (Token::TypeInt, Token::Integer(_)) => true,
            (Token::TypeFloat, Token::Float(_)) => true,
            (Token::TypeString, Token::String(_)) => true,
            (Token::TypeBool, Token::Boolean(_)) => true,
            (Token::TypeList(_), Token::List(_)) => true,
            _ => false
        }        
    }

}