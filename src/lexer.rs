use crate::tokens::{self, Token};

pub fn lexer(content: &str) -> Result<Vec<Token>, String> {
    let mut token: Vec<Token> = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let c = chars[i];

        // Skip spaces
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // ---------- Identifiers and keywords ----------
        if c.is_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }

            let slice: String = chars[start..i].iter().collect();

            match slice.as_str() {
                "let" => token.push(Token::Let),
                "int" => token.push(Token::TypeNumber),
                "bool" => token.push(Token::TypeBool),
                "string" => token.push(Token::TypeString),
                "print" => token.push(Token::Print),
                "true" => token.push(Token::Boolean(true)),
                "false" => token.push(Token::Boolean(false)),
                _ => token.push(Token::Identifier(slice)),
            }

            continue;
        }

        // ---------- Numbers --------------
        if c.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let slice: String = chars[start..i].iter().collect();
            let value: f64 = slice.parse().map_err(|_| format!("Invalid number: {}", slice))?;
            token.push(Token::Number(value));
            continue;
        }

        // ---------- Strings --------------

        if c == '"' {
            i += 1; // skip opening
            let start = i;
            while i < chars.len() && chars[i] != '"' {
                i += 1;
            }
            if i >= chars.len() {
                return Err("Unterminated string literal".to_string());
            }
            let slice: String = chars[start..i].iter().collect();
            token.push(Token::String(slice));
            i += 1; // skip closing
            continue;
        }

        // ---------- Operators ----------
        match c {
            '=' => { token.push(Token::Assign); i += 1; continue; }
            '+' => { token.push(Token::Plus); i += 1; continue; }
            '-' => { token.push(Token::Minus); i += 1; continue; }
            '*' => { token.push(Token::Multiply); i += 1; continue; }
            '/' => { token.push(Token::Divide); i += 1; continue; }
            '(' => { token.push(Token::LParen); i += 1; continue; }
            ')' => { token.push(Token::RParen); i += 1; continue; }
            '{' => { token.push(Token::LBrace); i += 1; continue; }
            '}' => { token.push(Token::RBrace); i += 1; continue; }
            ';' => { token.push(Token::Semicolon); i += 1; continue; }
            _ => {}
        }

        token.push(Token::EOF);

        return Err(format!("Unexpected token starting at index {}", i));
    }

    Ok(token)
}