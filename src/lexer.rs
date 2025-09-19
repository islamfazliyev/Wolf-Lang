use crate::tokens::{self, Token};

pub fn lexer(content: &str) -> Result<Vec<Token>, String> {
    let mut token: Vec<Token> = Vec::new();
    let mut chars = content.chars().peekable();
    let mut i = 0;
    
    while i < content.len() {
        if let Some(c) = content.get(i..i+1) {
            if c == " " {
                i += 1;
                continue;
            }
        }

        if let Some(slice) = content.get(i..i+3) {
           if slice == "let" {
                token.push(Token::Let);
                i += 3;
                continue;
            } 
        }
        
        if let Some(slice) = content.get(i..i+3) {
            if slice == "int" {
                token.push(Token::TypeNumber);
                i += 3;
                continue;
            }
        }

        if let Some(slice) = content.get(i..i+4) {
            if slice == "bool" {
                token.push(Token::TypeBool);
                i += 4;
                continue;
            }
        }

        if let Some(slice) = content.get(i..i+6) {
            if slice == "string" {
                token.push(Token::TypeString);
                i += 6;
                continue;
            }
        }

        if let Some(slice) = content.get(i..i+5) {
            if slice == "print" {
                token.push(Token::Print);
                i += 5;
                continue;
            }
        }

        

        if let Some(slice) = content.get(i..i+1) {
            if slice == "(" {
                token.push(Token::LParen);
                i += 1;
                continue;
            }
        }

        if let Some(slice) = content.get(i..i+1) {
            if slice == ")" {
                token.push(Token::RParen);
                i += 1;
                continue;
            }
        }
        
        return Err(format!("Unexpected token starting at index {}", i));
    }
    Ok(token)
}