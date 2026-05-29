use crate::tokens::{ Token};

pub fn lexer(content: &str) -> Result<Vec<(Token, usize)>, String> {
    let mut token: Vec<(Token, usize)> = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    let mut line = 1;
    
    while i < chars.len() {
        let c = chars[i];

        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }

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
                "let" => token.push((Token::Let, line)),
                "int" => token.push((Token::TypeInt, line)),
                "float" => token.push((Token::TypeFloat, line)),
                "bool" => token.push((Token::TypeBool, line)),
                "string" => token.push((Token::TypeString, line)),
                "list" => token.push((Token::TypeList(Box::new(Token::Unknown)), line)),
                "print" => token.push((Token::Print, line)),
                "true" => token.push((Token::Boolean(true), line)),
                "false" => token.push((Token::Boolean(false), line)),
                "if" => token.push((Token::If, line)),
                "else" => token.push((Token::Else, line)),
                "while" => token.push((Token::While, line)),
                "for" => token.push((Token::For, line)),
                "and" => token.push((Token::And, line)),
                "or" => token.push((Token::Or, line)),
                "fn" => token.push((Token::Func, line)),
                "struct" => token.push((Token::Struct, line)),
                "impl" => token.push((Token::Impl, line)),
                "range" => token.push((Token::Range, line)),
                "return" => token.push((Token::Return, line)),
                "import" => token.push((Token::Import, line)),
                "as" => token.push((Token::As, line)),
                //other
                "end" => token.push((Token::EndOfCondition, line)),
                _ => token.push((Token::Identifier(slice), line)),
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
                token.push((Token::Float(value), line));
            } else {
                let value: i64 = slice.parse().map_err(|_| format!("Invalid integer: {}", slice))?;
                token.push((Token::Integer(value), line));
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
            token.push((Token::String(slice), line));
            i += 1; // skip closing
            continue;
        }

        // ---------- Operators ----------
        match c {
            
            '=' => { 
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    token.push((Token::Equals, line));
                    i += 2;
                } else {
                    token.push((Token::Assign, line));
                    i += 1;
                }
                continue;
            }

            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    token.push((Token::NotEquals, line));
                    i += 2;
                    
                } else {
                    token.push((Token::Bang, line)); 
                    i += 1;
                }
                continue;
            }

            '<' => { 
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    token.push((Token::LesserEquals, line));
                    i += 2;
                } else {
                    token.push((Token::Lesser, line));
                    i += 1;
                }
                continue;
            }

            '>' => { 
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    token.push((Token::GreaterEquals, line));
                    i += 2;
                } else {
                    token.push((Token::Greater, line));
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

            '+' => { token.push((Token::Plus, line)); i += 1; continue; }
            '-' => { token.push((Token::Minus, line)); i += 1; continue; }
            '*' => { token.push((Token::Multiply, line)); i += 1; continue; }
            '/' => { token.push((Token::Divide, line)); i += 1; continue; }
            '(' => { token.push((Token::LParen, line)); i += 1; continue; }
            ')' => { token.push((Token::RParen, line)); i += 1; continue; }
            '[' => { token.push((Token::LBracket, line)); i += 1; continue; }
            ']' => { token.push((Token::RBracket, line)); i += 1; continue; }
            ',' => { token.push((Token::Comma, line)); i += 1; continue; }
            ':' => {
                if i + 1 < chars.len() && chars[i + 1] == ':' {
                    token.push((Token::DoubleColon, line));
                    i += 2;
                } else {
                    token.push((Token::Colon, line));
                    i += 1;
                }
                continue;
            }
            '.' => { token.push((Token::Dot, line)); i += 1; continue; }
            _ => {}
        }

        token.push((Token::EOF, line));

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