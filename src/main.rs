use crate::lexer::lexer;

mod tokens;
mod lexer;
mod parser;

fn main() {
    let content = "let string print()";
    let mut _tokens = lexer(content);
    println!("{:?}", _tokens);
}