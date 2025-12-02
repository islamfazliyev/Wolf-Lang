use crate::tokens::Token;
use crate::error_handler::ParseError;
use std::io::{self, Write};

pub fn dispatch(name: &str, args: Vec<Token>) -> Option<Result<Token, ParseError>> {
    match name {
        "input" => Some(native_input(args)),
        "clear" => Some(native_clear(args)),
        _ => None
    }
}

fn native_input(args: Vec<Token>) -> Result<Token, ParseError> {

    if let Some(first_arg) = args.first() {
        match first_arg {
            Token::String(s) => print!("{}", s),
            _ => print!("{:?}", first_arg),
        }
        // Flush stdout to ensure prompt appears before input
        io::stdout().flush().map_err(|_| ParseError::UnkownType { 
            type_name: "IO Flush Error".to_string() 
        })?;
    }

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).map_err(|_| ParseError::UnkownType { 
        type_name: "Failed to read input".to_string() 
    })?;

    Ok(Token::String(buffer.trim_end().to_string()))
}

fn native_clear(_args: Vec<Token>) -> Result<Token, ParseError> {
    print!("\x1B[2J\x1B[1;1H");
    
    io::stdout().flush().map_err(|_| ParseError::UnkownType { 
        type_name: "IO Flush Error".to_string() 
    })?;

    Ok(Token::Boolean(true))
}