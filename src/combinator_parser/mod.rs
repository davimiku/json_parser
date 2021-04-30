/// Common combinators
///
/// Combinators not directly related to a given input,
/// these combinators could be used generally in many
/// situations.
mod common;

/// Parsers focused on specific characters or sequences
/// of characters.
///
/// This module probably could be renamed, 'lexers' intends
/// to imply these deal with the source characters more
/// directly than the higher-kinded parsers elsewhere in the crate.
mod lexers;

/// Parsers related to JSON syntax
///
/// These build upon the more general parsers
/// and are directly used to build the JSON tree.
mod json;

use common::{and_then, map, pred};
use json::json_value;

use crate::value::Value;

/// Result of a parsing step
///
/// An Ok value is represented as a tuple of the remaining str
/// to parse and the output as parsed.
pub type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

pub trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;

    /// Apply a mapping function to modify the output of a parser
    fn map<F, NewOutput>(self, map_fn: F) -> BoxedParser<'a, NewOutput>
    where
        Self: Sized + 'a,
        Output: 'a,
        NewOutput: 'a,
        F: Fn(Output) -> NewOutput + 'a,
    {
        BoxedParser::new(map(self, map_fn))
    }

    /// Apply the predicate function to the output of the parser
    fn pred<F>(self, pred_fn: F) -> BoxedParser<'a, Output>
    where
        Self: Sized + 'a,
        Output: 'a,
        F: Fn(&Output) -> bool + 'a,
    {
        BoxedParser::new(pred(self, pred_fn))
    }

    /// Apply another parser to the output and return a new parser
    fn and_then<F, NextParser, NewOutput>(self, f: F) -> BoxedParser<'a, NewOutput>
    where
        Self: Sized + 'a,
        Output: 'a,
        NewOutput: 'a,
        NextParser: Parser<'a, NewOutput> + 'a,
        F: Fn(Output) -> NextParser + 'a,
    {
        BoxedParser::new(and_then(self, f))
    }
}

pub struct BoxedParser<'a, Output> {
    parser: Box<dyn Parser<'a, Output> + 'a>,
}

impl<'a, Output> BoxedParser<'a, Output> {
    fn new<P>(parser: P) -> Self
    where
        P: Parser<'a, Output> + 'a,
    {
        BoxedParser {
            parser: Box::new(parser),
        }
    }
}

impl<'a, Output> Parser<'a, Output> for BoxedParser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self.parser.parse(input)
    }
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

pub fn parse<'a>(input: &str) -> Result<Value, &str> {
    json_value().parse(input).map(|(_, value)| value)
}

/// Integration tests
#[cfg(test)]
mod tests {

    use crate::{combinator_parser::parse, json_object, value::Value};
    use std::collections::BTreeMap;

    #[test]
    fn parse_full_json() {
        let input = r#"
            {
                "str_val": "value",
                "null_val": null,
                "true_val": true,
                "false_val": false,
                "int_val": 5,
                "arr_val": [
                    "one",
                    2,
                    false,
                ],
                "obj_val": {
                    "nested_key": "nested_value"
                }
            }
        "#;

        let expected = Value::Object(json_object!(
            "str_val".to_string() => Value::String("value".to_string()),
            "null_val".to_string() => Value::Null,
            "true_val".to_string() => Value::Bool(true),
            "false_val".to_string() => Value::Bool(false),
            "int_val".to_string() => Value::Number(5),
            "arr_val".to_string() => Value::Array(vec! [Value::String("one".to_string()), Value::Number(2), Value::Bool(false)]),
            "obj_val".to_string() => Value::Object(json_object!(
                "nested_key".to_string() => Value::String("nested_value".to_string())
            ))
        ));

        let actual = parse(input);
        assert!(actual.is_ok());
    }
}
