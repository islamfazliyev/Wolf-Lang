use crate::tokens::Token;
use crate::error_handler::ParseError;
use std::io::{self, Write};
use rand::Rng;


pub fn dispatch(name: &str, args: Vec<Token>) -> Option<Result<Token, ParseError>> {
    match name {
        "input" => Some(native_input(args)),
        "clear" => Some(native_clear(args)),
        "randInt" => Some(native_rand_int(args)),
        "randFloat" => Some(native_rand_float(args)),
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
            type_name: "IO Flush Error".to_string(),
            line: 0
        })?;
    }

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).map_err(|_| ParseError::UnkownType { 
        type_name: "Failed to read input".to_string(),
        line: 0
    })?;

    Ok(Token::String(buffer.trim_end().to_string()))
}

fn native_clear(_args: Vec<Token>) -> Result<Token, ParseError> {
    print!("\x1B[2J\x1B[1;1H");
    
    io::stdout().flush().map_err(|_| ParseError::UnkownType { 
        type_name: "IO Flush Error".to_string(),
        line: 0 
    })?;

    Ok(Token::Boolean(true))
}

fn native_rand_int(args: Vec<Token>) -> Result<Token, ParseError> {
    let mut rng = rand::thread_rng();

    if args.len() < 2 {
        return Err(ParseError::UnkownType {
            type_name: "randInt requires 2 arguments: (min, max)".to_string(),
            line: 0,
        });
    }

    match (&args[0], &args[1]) {
        (Token::Integer(min), Token::Integer(max)) => {
            let min = *min as i64;
            let max = *max as i64;

            if min > max {
                return Err(ParseError::UnkownType {
                    type_name: "randInt: min cannot be greater than max".to_string(),
                    line: 0,
                });
            }

            let result = rng.gen_range(min..=max);
            Ok(Token::Integer(result)) // Token::Number tipin f64/i64 ne kullanıyorsa
        }
        _ => Err(ParseError::UnkownType {
            type_name: "randInt arguments must be numbers".to_string(),
            line: 0,
        }),
    }
}

fn native_rand_float(args: Vec<Token>) -> Result<Token, ParseError> {
    let mut rng = rand::thread_rng();

    if args.len() < 2 {
        return Err(ParseError::UnkownType {
            type_name: "randFloat requires 2 arguments: (min, max)".to_string(),
            line: 0,
        });
    }

    let min = match &args[0] {
        Token::Integer(i) => *i as f64,
        Token::Float(f) => *f,
        _ => return Err(ParseError::UnkownType {
            type_name: "randFloat first argument must be a number".to_string(),
            line: 0,
        }),
    };

    let max = match &args[1] {
        Token::Integer(i) => *i as f64,
        Token::Float(f) => *f,
        _ => return Err(ParseError::UnkownType {
            type_name: "randFloat second argument must be a number".to_string(),
            line: 0,
        }),
    };

    if min > max {
        return Err(ParseError::UnkownType {
            type_name: "randFloat: min cannot be greater than max".to_string(),
            line: 0,
        });
    }

    let result = rng.gen_range(min..=max);

    Ok(Token::Float(result)) 
}