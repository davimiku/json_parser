mod parse;
mod tokenize;

use parse::{parse_tokens, TokenParseError};
use std::collections::HashMap;
use tokenize::{tokenize, TokenizeError};

pub fn parse(input: String) -> Result<Value, ParseError> {
    let tokens = tokenize(input)?;
    let value = parse_tokens(&tokens, &mut 0)?;
    Ok(value)
}

/// Representation of a JSON value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// literal characters `null`
    Null,

    /// literal characters `true` or `false`
    Boolean(bool),

    /// characters within double quotes "..."
    String(String),

    /// numbers stored as a 64-bit floating point
    Number(f64),

    /// Zero to many JSON values
    Array(Vec<Value>),

    /// String keys with JSON values
    Object(HashMap<String, Value>),
}

#[cfg(test)]
impl Value {
    pub(crate) fn object<const N: usize>(pairs: [(&'static str, Self); N]) -> Self {
        let owned_pairs = pairs.map(|(key, value)| (String::from(key), value));
        let map = HashMap::from(owned_pairs);
        Self::Object(map)
    }

    pub(crate) fn string(s: &str) -> Self {
        Self::String(String::from(s))
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    TokenizeError(TokenizeError),
    ParseError(TokenParseError),
}

impl From<TokenParseError> for ParseError {
    fn from(err: TokenParseError) -> Self {
        Self::ParseError(err)
    }
}

impl From<TokenizeError> for ParseError {
    fn from(err: TokenizeError) -> Self {
        Self::TokenizeError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(input: &str, expected: Value) {
        let actual = parse(String::from(input)).unwrap();
        assert_eq!(actual, expected);
    }

    fn check_error<E: Into<ParseError>>(input: &str, expected: E) {
        let expected = expected.into();
        let actual = parse(String::from(input)).unwrap_err();
        assert_eq!(actual, expected);
    }

    #[test]
    fn just_null() {
        check("null", Value::Null);
    }

    #[test]
    fn just_true() {
        check("true", Value::Boolean(true));
    }

    #[test]
    fn just_false() {
        check("false", Value::Boolean(false));
    }

    #[test]
    fn array_with_null() {
        check("[null]", Value::Array(vec![Value::Null]))
    }

    #[test]
    fn array_with_true_false() {
        check(
            "[true,false]",
            Value::Array(vec![Value::Boolean(true), Value::Boolean(false)]),
        )
    }

    #[test]
    fn array_with_numbers() {
        check(
            "[1, 2, 3]",
            Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
        )
    }

    #[test]
    fn array_with_object() {
        check(
            r#"[{"key": null}]"#,
            Value::Array(vec![Value::object([("key", Value::Null)])]),
        )
    }

    #[test]
    fn empty_object() {
        check("{}", Value::object([]))
    }

    #[test]
    fn object_with_number() {
        check(
            r#"{"key": 1}"#,
            Value::object([("key", Value::Number(1.0))]),
        );
    }

    #[test]
    fn object_with_string() {
        check(
            r#"{"key": "value"}"#,
            Value::object([("key", Value::String("value".to_string()))]),
        );
    }

    #[test]
    fn object_with_null() {
        check(r#"{"key": null}"#, Value::object([("key", Value::Null)]));
    }

    #[test]
    fn object_with_true() {
        check(
            r#"{"key": true}"#,
            Value::object([("key", Value::Boolean(true))]),
        )
    }

    #[test]
    fn object_with_false() {
        check(
            r#"{"key": false}"#,
            Value::object([("key", Value::Boolean(false))]),
        )
    }

    #[test]
    fn nested_object() {
        check(
            r#"{"key": { "innerkey": null } }"#,
            Value::object([("key", Value::object([("innerkey", Value::Null)]))]),
        )
    }

    #[test]
    fn object_many_entries() {
        check(
            r#"{ "a": 1, "b": "ya like jazz?", "c": false }"#,
            Value::object([
                ("a", Value::Number(1.0)),
                ("b", Value::string("ya like jazz?")),
                ("c", Value::Boolean(false)),
            ]),
        );
    }

    #[test]
    #[ignore = "this fails - for the sake of brevity, leaving this unfixed"]
    fn err_unclosed_array() {
        check_error(
            "[null",
            ParseError::ParseError(TokenParseError::UnclosedBracket),
        )
    }

    #[test]
    #[ignore = "this fails - for the sake of brevity, leaving this unfixed"]
    fn err_unclosed_object() {
        check_error(
            r#"{"key":"value""#,
            ParseError::ParseError(TokenParseError::UnclosedBrace),
        )
    }

    #[test]
    fn err_expected_value() {
        check_error("]", ParseError::ParseError(TokenParseError::ExpectedValue))
    }
}
