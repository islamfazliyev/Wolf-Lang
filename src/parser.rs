use std::{collections::HashMap, thread::scope};

use crate::{error_handler::ParseError, tokens::{self, Token}};

#[derive(Debug, Clone, PartialEq)]

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    output: Vec<String>,
    symbol_table: HashMap<String, Token>,
    
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            output: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    fn eat(&mut self, token_type: Token) -> Result<(), ParseError> {
        if let Some(tok) = self.current_token() {
            if *tok == token_type {
                self.pos += 1;
                Ok(())
            }
            else {
                Err(ParseError::UnexpectedToken {
                    expected: token_type,
                    found: Some(tok.clone()),
                })
            }
        }
        else {
            Err(ParseError::UnexpectedToken {
                expected: token_type,
                found: None,
            })
        }
    }

    fn handle_token(&mut self, token: Token, name: &str) -> Result<(), ParseError> {
        self.output.push(name.to_string());
        self.eat(token)
    }
    
    pub fn parse_let(&mut self) -> Result<(), ParseError> {
        self.eat(Token::Let)?;

        let mut var_name: Option<String> = None;
        let mut assigned_value: Option<Token> = None;
        let mut declared_type: Option<Token> = None;

        // After `let` → expect type (number/string/bool)
        if let Some(next) = self.current_token() {
            match next {
                Token::TypeNumber | Token::TypeString | Token::TypeBool => {
                    declared_type = Some(next.clone());
                    let ty = format!("{:?}", next);
                    self.output.push(format!("type {}", ty));
                    self.pos += 1; // consume type
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: Token::TypeNumber, // just a placeholder
                        found: Some(next.clone()),
                    })
                }
            }
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: Token::TypeNumber,
                found: None,
            });
        }

        if let Some(next) = self.current_token() {
            match next {
                Token::Identifier(name) => {
                    var_name = Some(name.clone());
                    self.output.push(format!("var {}", name));
                    self.pos += 1;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: Token::Identifier("x".to_string()),
                        found: Some(next.clone()),
                    })
                }
            }
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: Token::Identifier("x".to_string()),
                found: None,
            })
        }

        // After identifier → expect '='
        self.eat(Token::Assign)?;

        if let Some(decl_ty) = declared_type.as_ref() {
            if *decl_ty == Token::TypeNumber {
                match self.parse_expr()? {
                    Token::Number(result) => {
                        assigned_value = Some(Token::Number(result));
                        self.output.push(format!("value (expression result) {}", result));
                    }
                    _ => {
                        return Err(ParseError::TypeMismatch {
                            expected: Token::TypeNumber,
                            found: Token::Boolean(false), // placeholder
                        })
                    }
                }
            } else if let Some(next) = self.current_token() {
                match next {
                    Token::String(s) => {
                        assigned_value = Some(next.clone());
                        self.output.push(format!("value {}", s));
                        self.pos += 1;
                    }
                    Token::Boolean(b) => {
                        assigned_value = Some(next.clone());
                        self.output.push(format!("value {}", b));
                        self.pos += 1;
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: Token::String("literal or expression".to_string()),
                            found: Some(next.clone()),
                        })
                    }
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: Token::String("literal or expression".to_string()),
                    found: None,
                })
            }
        }

        if let (Some(decl_ty), Some(val_tok)) = (declared_type.as_ref(), assigned_value.as_ref()) {
            let matches = match (decl_ty, val_tok) {
                (Token::TypeBool, Token::Boolean(_)) => true,
                (Token::TypeNumber, Token::Number(_)) => true,
                (Token::TypeString, Token::String(_)) => true,
                _ => false
            };

            if !matches {
                return Err(ParseError::TypeMismatch {
                    expected: decl_ty.clone(),
                    found: val_tok.clone(),
                });
            }
        }

        if let (Some(name), Some(value)) = (var_name, assigned_value) {
            self.symbol_table.insert(name, value);
            println!("{:?}", self.symbol_table);
        }
        Ok(())
    }

    fn parse_print(&mut self) -> Result<(), ParseError> {
        self.eat(Token::Print)?;
        if let Some(next) = self.current_token() {
            match next {
                Token::String(s) => {
                    println!("{:?}", s);
                    self.output.push(format!("print value: {}", s));
                    self.pos += 1;
                }

                Token::Number(n) => {
                    println!("{:?}", n);
                    self.output.push(format!("print value: {}", n));
                    self.pos += 1;
                }

                Token::Boolean(b) => {
                    println!("{:?}", b);
                    self.output.push(format!("print value: {}", b));
                    self.pos += 1;
                }

                Token::Identifier(name) => {
                    if let Some(value_token) = self.symbol_table.get(name)  {
                        println!("{:?}", value_token);
                        self.output.push(format!("print var: {} = {:?}", name, value_token));
                        self.pos += 1;
                    }
                    else {
                        return Err(ParseError::UndeclaredVariable{name: name.to_string()})
                    }
                }

                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: Token::Number(0.0),
                        found: Some(next.clone()),
                    })
                }
            }
            
        }
        else {
            // End of tokens after 'print'
            return Err(ParseError::UnexpectedToken {
                expected: Token::String("".to_string()),
                found: None,
            })
        }
        Ok(())
    }

    fn parse_condition(&mut self) -> Result<bool, ParseError> {
        let lhs_token = self.parse_expr()?;
       

        let op_token_opt = self.current_token().cloned();

        if let Some(op_token) = op_token_opt {
            match op_token {
                Token::Equals | Token::NotEquals | Token::Greater | 
                Token::Lesser | Token::GreaterEquals | Token::LesserEquals => {
                    self.pos += 1;

                    let rhs_token = self.parse_expr()?;
                    match (&lhs_token, &rhs_token) {
                        (Token::Number(lhs), Token::Number(rhs)) => {
                            let result = match op_token {
                                Token::Equals => lhs == rhs,
                                Token::NotEquals => lhs != rhs,
                                Token::Greater => lhs > rhs,
                                Token::Lesser => lhs < rhs,
                                Token::GreaterEquals => lhs >= rhs,
                                Token::LesserEquals => lhs <= rhs,
                                _ => unreachable!(),
                            };
                            return Ok(result);
                        }
                        (Token::String(lhs), Token::String(rhs)) => {
                            let result = match op_token {
                                Token::Equals => lhs == rhs,
                                Token::NotEquals => lhs != rhs,
                                _ => {
                                    return Err(ParseError::UnexpectedToken {
                                        expected: Token::Equals, 
                                        found: Some(op_token),
                                    })
                                },
                            };
                            return Ok(result);
                        }

                        (Token::Boolean(lhs), Token::Boolean(rhs)) => {
                            let result = match op_token {
                                Token::Equals => lhs == rhs,
                                Token::NotEquals => lhs != rhs,
                                _ => {
                                    return Err(ParseError::UnexpectedToken {
                                        expected: Token::Equals, 
                                        found: Some(op_token),
                                    })
                                },
                            };
                            return Ok(result);
                        }
                        _ => {
                            // Type mismatch - farklı tipler karşılaştırılamaz
                            return Err(ParseError::TypeMismatch {
                                expected: lhs_token.clone(),
                                found: rhs_token,
                            });
                        }
                    }

                }
                _ => {}
            }
        }
        Err(ParseError::UnexpectedToken {
            expected: Token::Equals,
            found: self.current_token().cloned(),
        })
    }

    fn parse_if(&mut self) -> Result<(), ParseError> {
        self.eat(Token::If)?;

        let condition_result = self.parse_condition()?; 

        let mut block_tokens: Vec<Token> = Vec::new();
        let mut block_pos = self.pos;
        let mut scope_depth = 1;

        while let Some(tok) = self.tokens.get(block_pos) {
            if *tok == Token::If {
                scope_depth += 1;
            } else if *tok == Token::EndOfCondition {
                scope_depth -= 1;
                if scope_depth == 0 {
                   
                    break;
                }
            }
            block_tokens.push(tok.clone());
            block_pos += 1;
        }

        if scope_depth != 0 {
            return Err(ParseError::UnexpectedToken {
                expected: Token::EndOfCondition,
                found: None, // Missing 'end'
            });
        }
        self.pos = block_pos + 1;

        

        if condition_result {
            let mut block_parser = Parser::new(block_tokens);
            block_parser.symbol_table = self.symbol_table.clone(); // Copy parent scope
            while let Some(_) = block_parser.current_token() {
                block_parser.sense()?;
            }
        }
        Ok(())
    }

    fn parse_expr(&mut self) -> Result<Token, ParseError> {
        let mut result = self.parse_term()?; 

        while let Some(tok) = self.current_token() {
            match tok {
                Token::Plus => {
                    self.eat(Token::Plus)?;
                    let next_term = self.parse_term()?;
                    if let (Token::Number(lhs), Token::Number(rhs)) = (&result, &next_term) {
                        result = Token::Number(lhs + rhs);
                    } else {
                        return Err(ParseError::TypeMismatch {
                            expected: Token::TypeNumber,
                            found: next_term,
                        });
                    }
                }
                Token::Minus => {
                    self.eat(Token::Minus)?;
                    let next_term = self.parse_term()?;
                    if let (Token::Number(lhs), Token::Number(rhs)) = (&result, &next_term) {
                        result = Token::Number(lhs - rhs);
                    } else {
                        return Err(ParseError::TypeMismatch {
                            expected: Token::TypeNumber,
                            found: next_term,
                        });
                    }
                }
                _ => break,
            }
        }
        Ok(result)
    }

    fn parse_term(&mut self) -> Result<Token, ParseError> {
        let mut result = self.parse_factor()?; 

        while let Some(tok) = self.current_token() {
            match tok {
                Token::Multiply => {
                    self.eat(Token::Multiply)?;
                    let next_factor = self.parse_factor()?;
                    if let (Token::Number(lhs), Token::Number(rhs)) = (&result, &next_factor) {
                        result = Token::Number(lhs * rhs);
                    } else {
                        return Err(ParseError::TypeMismatch {
                            expected: Token::TypeNumber,
                            found: next_factor,
                        });
                    }
                }
                Token::Divide => {
                    self.eat(Token::Divide)?;
                    let next_factor = self.parse_factor()?;
                    if let (Token::Number(lhs), Token::Number(rhs)) = (&result, &next_factor) {
                        
                        result = Token::Number(lhs / rhs);
                    } else {
                        return Err(ParseError::TypeMismatch {
                            expected: Token::TypeNumber,
                            found: next_factor,
                        });
                    }
                }
                _ => break,
            }
        }
        Ok(result)
    }

    fn parse_factor(&mut self) -> Result<Token, ParseError> {
        let tok = match self.current_token().cloned() {
            Some(t) => t,
            None => {
                return Err(ParseError::UnexpectedToken {
                    expected: Token::Number(0.0), // Expected something, found None
                    found: None,
                });
            }
        };

        match tok {
            Token::Number(n) => {
                self.eat(Token::Number(n))?;
                Ok(Token::Number(n))
            }
            Token::String(s) => {
                self.eat(Token::String(s.clone()))?;
                Ok(Token::String(s))
            }
            Token::Boolean(b) => {
                self.eat(Token::Boolean(b))?;
                Ok(Token::Boolean(b))
            }
            Token::Plus => {
                self.eat(Token::Plus)?;
                self.parse_factor()
            }
            Token::Minus => {
                self.eat(Token::Minus)?;
                let value_token = self.parse_factor()?;
                if let Token::Number(n) = value_token {
                    Ok(Token::Number(-n))
                } else {
                    Err(ParseError::TypeMismatch {
                        expected: Token::TypeNumber,
                        found: value_token,
                    })
                }
            }
            Token::LParen => {
                self.eat(Token::LParen)?;
                let result = self.parse_expr()?;
                self.eat(Token::RParen)?;
                Ok(result)
            }
            Token::Identifier(name) => {
                self.eat(Token::Identifier(name.clone()))?;
                if let Some(value_token) = self.symbol_table.get(&name) {
                    Ok(value_token.clone())
                } else {
                    Err(ParseError::UndeclaredVariable { name })
                }
            }
            _ => {
                println!("error here");
                Err(ParseError::UnexpectedToken {
                    expected: Token::Number(0.0),
                    found: Some(tok),
                })
            }
        }
       
    }

    pub fn sense(&mut self) -> Result<(), ParseError> {
        if let Some(tok) = self.current_token() {
                match tok {
                    Token::Let => self.parse_let(),
                    Token::Print => self.parse_print(),
                    Token::If => self.parse_if(),
                    _ => {
                        let name = format!("{:?}", tok);
                        self.output.push("unknown".to_string());
                        Err(ParseError::UnkownType { type_name: name })
                    }
                }
        } else {
            Ok(())
        }
    }
    
}