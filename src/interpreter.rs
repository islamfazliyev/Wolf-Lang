use core::panic;
use std::{collections::HashMap, io::IsTerminal};
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
            Stmt::Expression(expr) => {
                self.evaluate(expr);
                Ok(())
            }
            Stmt::Print(exprs) => {
                for (index, expr) in exprs.iter().enumerate() {
                    let value = self.evaluate(expr.clone());

                    self.print_token_value(&value)?;

                    if index < exprs.len() - 1 {
                        print!("");
                    }

                }
                println!();
                Ok(())
            }

            Stmt::Let { name, data_type, value } => {
                let declared_value = self.evaluate(value);
                if Self::check_type_compatibility(&data_type, &declared_value) {
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

            Stmt::Block(statements) => {
                self.scopes.push(HashMap::new());

                for stmt in statements {
                    if let Err(e) = self.execute(stmt) {
                        self.scopes.pop(); // Hata olsa bile çıkarken temizle!
                        return Err(e);
                    }
                }
                self.scopes.pop();
                Ok(())
            }

            Stmt::If { condition, then_branch, else_branch } => {
                let evaluated_cond = self.evaluate(condition);

                let is_true = match evaluated_cond {
                    Token::Boolean(b) => b,
                    _ => panic!("Runtime Error: 'if' koşulu boolean olmalı! Bulunan: {:?}", evaluated_cond),
                };

                if is_true {
                    self.execute(*then_branch)?;
                } else if let Some(else_stmt) = else_branch {
                    self.execute(*else_stmt)?;
                }
                
                Ok(())
            }

            Stmt::While { condition, body } => {

                loop {
                    
                    let evaluated_cond = self.evaluate(condition.clone());
                    let is_true = match evaluated_cond {
                        Token::Boolean(b) => b,
                        _ => panic!("Runtime Error: 'while' koşulu boolean olmalı! Bulunan: {:?}", evaluated_cond),
                    };
    
                    if is_true {
                        self.execute(*body.clone())?;
                    } else {
                        break;
                    }

                }
                Ok(())
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

            Expr::Binary { left, op, right } => {
                let left = self.evaluate(*left);
                let right = self.evaluate(*right);

                if op == Token::Equals {
                    return Token::Boolean(left == right);
                }
                if op == Token::NotEquals {
                    return Token::Boolean(left != right);
                }

                if let (Some(l_num), Some(r_num)) = (to_float(&left), to_float(&right)) {
                    match op {
                        Token::Greater => return Token::Boolean(l_num > r_num),
                        Token::Lesser => return Token::Boolean(l_num < r_num),
                        Token::GreaterEquals => return Token::Boolean(l_num >= r_num),
                        Token::LesserEquals => return Token::Boolean(l_num <= r_num),

                        _ => {} 
                    }
                }

                match (left, op, right) {
                    (Token::Integer(l), Token::Plus, Token::Integer(r)) => Token::Integer(l + r),
                    (Token::Float(l), Token::Plus, Token::Float(r)) => Token::Float(l + r),
                    (Token::Integer(l), Token::Minus, Token::Integer(r)) => Token::Integer(l - r),
                    (Token::Float(l), Token::Minus, Token::Float(r)) => Token::Float(l - r),
                    
                    (Token::String(l), Token::Plus, Token::String(r)) => Token::String(format!("{}{}", l, r)),
                    
                    _ => panic!("Runtime Error: Type mismatch"),
                }
            }

            Expr::Unary { operator, right } => {
                let right_val = self.evaluate(*right);
                match (operator, right_val) {
                    (Token::Minus, Token::Integer(n)) => Token::Integer(-n),
                    (Token::Minus, Token::Float(n)) => Token::Float(-n),

                    (Token::Bang, Token::Boolean(b)) => Token::Boolean(!b),

                    (op, val) => panic!("Runtime Error: {:?} operator cannot used with {:?} .", op, val),
                    
                }
            }

            Expr::Assign { name, value } => {
                let new_value = self.evaluate(*value);

                for scope in self.scopes.iter_mut().rev() {
                    if let Some(old_value) = scope.get_mut(&name) {
                        
                        if Self::check_type_compatibility(old_value, &new_value) {
                            
                            *old_value = new_value.clone();
                            
                            return new_value;
                        } else {
                            panic!("Runtime Error: Type mismatch! Variable '{}' is {:?} but you tried to assign {:?}", name, old_value, new_value);
                        }
                    }
                }

                panic!("Runtime Error: Variable '{}' not declared.", name);
                
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

    fn check_type_compatibility(expected_type: &Token, actual_value: &Token) -> bool {
        match (expected_type, actual_value) {
            (Token::TypeInt, Token::Integer(_)) => true,
            (Token::TypeFloat, Token::Float(_)) => true,
            (Token::TypeString, Token::String(_)) => true,
            (Token::TypeBool, Token::Boolean(_)) => true,
            (Token::TypeList(_), Token::List(_)) => true,

            (Token::Integer(_), Token::Integer(_)) => true,
            (Token::Float(_), Token::Float(_)) => true,
            (Token::String(_), Token::String(_)) => true,
            (Token::Boolean(_), Token::Boolean(_)) => true,
            (Token::List(_), Token::List(_)) => true,

            _ => false
        }        
    }

}

fn to_float(token: &Token) -> Option<f64> {
    match token {
        Token::Integer(n) => Some(*n as f64),
        Token::Float(f) => Some(*f),
        _ => None,
    }
}