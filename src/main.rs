use json_parser_lib::{lexer::Lexer, parser::Parser};

fn main() {
    let input = "{\"array\": [null, true, false], \"hello\": \"world\"}";
    let lexer = Lexer::new(input.chars());
    let mut parser = Parser::new(lexer);
    let result = parser.parse();
    match result {
        Ok(value) => println!("{}", value),
        Err(err) => println!("{:?}", err),
    }
}
