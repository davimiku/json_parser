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
pub type ParseResult<'input, Output> = Result<(&'input str, Output), &'input str>;

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

pub fn parse(input: &str) -> Result<Value, &str> {
    json_value().parse(input).map(|(_, value)| value)
}

/// Integration tests across the combinator parser
#[cfg(test)]
mod tests {
    use crate::json_object;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn empty_object() {
        let input = "{}";
        let expected = Value::Object(HashMap::new());
        let actual = parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn simple_object() {
        let input = r#"{
            "a": "value",
            "b": true
        }"#;
        let expected = json_object! {
            "a" => Value::String("value".to_string()),
            "b" => Value::Boolean(true)
        };
        let actual = parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn empty_array() {
        let input = "[]";
        let expected = Value::Array(vec![]);
        let actual = parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn simple_array() {
        let input = r#"[
            "value",
            true,
            false,
            null
        ]"#;
        let expected = Value::Array(vec![
            Value::String("value".to_string()),
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Null,
        ]);
        let actual = parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn inner_object() {
        let input = r#"
        {
            "obj": {}
        }"#;
        let expected = json_object! { "obj" => Value::Object(HashMap::new()) };
        let actual = parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn inner_array() {
        let input = r#"
        {
            "arr": [
                "one",
                2,
                false
            ]
        }"#;
        let expected = json_object! { "arr" => Value::Array(vec![
            Value::String("one".to_string()), Value::Number(2.0), Value::Boolean(false)
        ]) };
        let actual = parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn all_primitives() {
        let input = r#"
            {
                "str_val": "value",
                "null_val": null,
                "true_val": true,
                "false_val": false,
                "int_val": 5
            }
        "#;

        let expected = json_object!(
            "str_val" => Value::String("value".to_string()),
            "null_val" => Value::Null,
            "true_val" => Value::Boolean(true),
            "false_val" => Value::Boolean(false),
            "int_val" => Value::Number(5.0)
        );

        let actual = parse(input).unwrap();
        assert_eq!(expected, actual);
    }
}
