use crate::tokens::{ Token};

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
                "int" => token.push(Token::TypeInt),
                "float" => token.push(Token::TypeFloat),
                "bool" => token.push(Token::TypeBool),
                "string" => token.push(Token::TypeString),
                "list" => token.push(Token::TypeList(Box::new(Token::Unknown))),
                "print" => token.push(Token::Print),
                "true" => token.push(Token::Boolean(true)),
                "false" => token.push(Token::Boolean(false)),
                "if" => token.push(Token::If),
                "while" => token.push(Token::While),
                "for" => token.push(Token::For),
                "fn" => token.push(Token::Func),
                "range" => token.push(Token::Range),
                "return" => token.push(Token::Return),
                //other
                "end" => token.push(Token::EndOfCondition),
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

            if slice.contains('.') {
                let value: f64 = slice.parse().map_err(|_| format!("Invalid float number: {}", slice))?;
                token.push(Token::Float(value));
            } else {
                let value: i64 = slice.parse().map_err(|_| format!("Invalid integer: {}", slice))?;
                token.push(Token::Integer(value));
            }

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
            
            '=' => { 
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    token.push(Token::Equals);
                    i += 2;
                } else {
                    token.push(Token::Assign);
                    i += 1;
                }
                continue;
            }

            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    token.push(Token::NotEquals);
                    i += 2;
                    continue;
                } else {
                    return Err(format!("Unexpected token starting at index {}", i))
                }
            }

            '<' => { 
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    token.push(Token::LesserEquals);
                    i += 2;
                } else {
                    token.push(Token::Lesser);
                    i += 1;
                }
                continue;
            }

            '>' => { 
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    token.push(Token::GreaterEquals);
                    i += 2;
                } else {
                    token.push(Token::Greater);
                    i += 1;
                }
                continue;
            }

            '#' => {
                // Skip comment until end of line
                while i < content.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }

            '+' => { token.push(Token::Plus); i += 1; continue; }
            '-' => { token.push(Token::Minus); i += 1; continue; }
            '*' => { token.push(Token::Multiply); i += 1; continue; }
            '/' => { token.push(Token::Divide); i += 1; continue; }
            '(' => { token.push(Token::LParen); i += 1; continue; }
            ')' => { token.push(Token::RParen); i += 1; continue; }
            '[' => { token.push(Token::LBracket); i += 1; continue; }
            ']' => { token.push(Token::RBracket); i += 1; continue; }
            ',' => { token.push(Token::Comma); i += 1; continue; }
            ':' => { token.push(Token::Colon); i += 1; continue; }
            '.' => { token.push(Token::Dot); i += 1; continue; }
            _ => {}
        }

        token.push(Token::EOF);

        return Err(format!("Unexpected token starting at index {}", i));
    }

    Ok(token)
}

#[cfg(test)]
mod tests {
    use crate::lexer::lexer;

    #[test]
    fn testLexer()
    {
        let content = "let string message = \"hello world\" let int num = 1 + 1 print message print num if test > >= < <= \"test\" print \"hello world\" end";
        let tokens = match lexer(&content) {
            Ok(tokens) => {
                tokens
            }
            Err(err) => {
                eprintln!("Lexer error: {}", err);
                return;
            }
        };
        println!("{:?}", tokens)
    }
    #[test]
    fn testLexer2()
    {
        let content = "while if";
        let tokens = match lexer(&content) {
            Ok(tokens) => {
                tokens
            }
            Err(err) => {
                eprintln!("Lexer error: {}", err);
                return;
            }
        };
        println!("{:?}", tokens)
    }
}