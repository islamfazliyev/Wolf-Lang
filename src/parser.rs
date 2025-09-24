use std::error;

use crate::{error_handler::ParseError, tokens::{self, Token}};

#[derive(Debug, Clone, PartialEq)]

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    output: Vec<String>,
    breakpoint_pos: Option<usize>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            output: Vec::new(),
            breakpoint_pos: None,
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

        // After `let` → expect type (number/string/bool)
        if let Some(next) = self.current_token() {
            match next {
                Token::TypeNumber | Token::TypeString | Token::TypeBool => {
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

        if let Some(next) = self.current_token()  {
            match next {
                Token::Number(n) => {
                    self.output.push(format!("value {}", n));
                    self.pos += 1;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: Token::Number(0.0),
                        found: Some(next.clone()),
                    })
                }
            }
        }

        Ok(())
    }

    pub fn sense(&mut self) -> Result<(), ParseError> {
        if let Some(tok) = self.current_token() {
                match tok {
                    Token::Let => self.parse_let(),
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