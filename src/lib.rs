use value::Value;

mod combinator_parser;
mod iterator_parser;
mod location;

pub mod value;

pub fn iterator_parse(input: &str) -> iterator_parser::ParseResult {
    iterator_parser::parse(input)
}

pub fn combinator_parse(input: &str) -> Result<Value, &str> {
    combinator_parser::parse(input)
}
