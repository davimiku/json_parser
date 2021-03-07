use lexer::Lexer;
use parser::Parser;

mod cursor;
mod lexer;
mod location;
mod parser;
mod token;
mod value;
// use std::iter::Map;

fn main() {
    let input = "[null, true, false, 1, \"hello\"]";
    let lexer = Lexer::new(input.chars());
    let mut parser = Parser::new(lexer);
    let result = parser.parse();
    match result {
        Ok(value) => println!("{}", value),
        Err(err) => println!("{:?}", err),
    }
}

#[test]
fn test() {
    assert!(true);
}
