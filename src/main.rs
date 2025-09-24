use crate::{lexer::lexer, parser::Parser};

mod tokens;
mod lexer;
mod parser;
mod error_handler;

fn main() {
    
    let content = "let int message = 2";
    let tokens = match lexer(&content) {
        Ok(tokens) => {
            tokens
        }
        Err(err) => {
            eprintln!("Lexer error: {}", err);
            return;
        }
    };
    println!("{:?}", tokens);

    let mut parser = Parser::new(tokens);

    while parser.current_token().is_some() {
        match parser.sense() {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Parser error: {:?}", err);
                break;
            }
        }
    }
}