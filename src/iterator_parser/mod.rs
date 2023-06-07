use std::collections::HashMap;

mod lex;

use crate::value::Value;
use lex::{lex, LexError, Token};

macro_rules! check_comma {
    ($needs_comma:expr,$value:expr,$token:expr) => {
        match $needs_comma {
            true => Err(ParseError::NeedsComma),
            false => {
                $needs_comma = true;
                Ok($value)
            }
        }
    };
}

struct Parser {
    /// Tokens produced
    tokens: Vec<Token>,

    curr_idx: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            curr_idx: 0,
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        let token = self.next_token();
        match token {
            Token::Null => Ok(Value::Null),
            Token::Float(f) => Ok(Value::Number(f)),
            Token::True => Ok(Value::Boolean(true)),
            Token::False => Ok(Value::Boolean(false)),
            Token::String(s) => self.parse_string(s),
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            _ => Err(ParseError::ExpectedValue),
        }
    }

    fn parse_string(&mut self, s: String) -> ParseResult {
        todo!()
    }

    fn parse_array(&mut self) -> ParseResult {
        let mut array: Vec<Value> = Vec::new();
        let mut needs_comma = false;
        loop {
            let token = self.next_token();
            let result: ParseResult = match &token {
                Token::String(s) => {
                    check_comma!(needs_comma, Value::String(s.to_string()), token)
                }
                Token::Float(f) => {
                    check_comma!(needs_comma, Value::Number(*f), token)
                }
                Token::Null => check_comma!(needs_comma, Value::Null, token),
                Token::True => check_comma!(needs_comma, Value::Boolean(true), token),
                Token::False => check_comma!(needs_comma, Value::Boolean(false), token),
                Token::LeftBrace => self.parse_object(),
                Token::LeftBracket => self.parse_array(),
                Token::RightBracket => break,
                Token::Comma => {
                    if needs_comma {
                        needs_comma = false;
                        continue;
                    } else {
                        Err(ParseError::TrailingComma)
                    }
                }
                _ => Err(ParseError::ExpectedValue),
            };
            match result {
                Ok(value) => array.push(value),
                Err(err) => return Err(err),
            }
        }
        Ok(Value::Array(array))
    }

    fn parse_object(&mut self) -> ParseResult {
        let mut map: HashMap<String, Value> = HashMap::new();
        loop {
            let token = self.next_token();
            match &token {
                Token::String(key) => {
                    let token = self.next_token();
                    match &token {
                        Token::Colon => {
                            let value = self.parse()?;
                            map.insert(key.to_string(), value);
                        }
                        _ => return Err(ParseError::ExpectedColon),
                    }
                    let next = self.next_token();
                    match &next {
                        Token::Comma => {}
                        Token::RightBrace => break, // object parsing ended
                        _ => return Err(ParseError::ExpectedComma),
                    }
                }
                _ => return Err(ParseError::ExpectedProperty),
            };
        }
        Ok(Value::Object(map))
    }

    fn next_token(&mut self) -> Token {
        self.curr_idx += 1;

        self.tokens[self.curr_idx].clone()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ParseError {
    LexError(LexError),

    EarlyEOF,
    UnclosedBracket,
    UnclosedBrace,

    ExpectedColon,
    ExpectedComma,
    ExpectedValue,
    ExpectedProperty,

    NeedsComma,
    TrailingComma,
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError::LexError(err)
    }
}

pub type ParseResult = Result<Value, ParseError>;

pub fn parse<S>(input: S) -> ParseResult
where
    S: Into<String>,
{
    let tokens = lex(input.into());
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_object;

    #[test]
    fn just_null() {
        let value = parse("null");
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), Value::Null);
    }

    #[test]
    fn just_true() {
        let value = parse("true");
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn just_false() {
        let value = parse("false");
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), Value::Boolean(false));
    }

    #[test]
    fn array_with_null() {
        let value = parse("[null]");
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), Value::Array(vec!(Value::Null)))
    }

    #[test]
    fn array_with_true_false() {
        let value = parse("[true,false]");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Array(vec![Value::Boolean(true), Value::Boolean(false)])
        )
    }

    #[test]
    fn array_with_numbers() {
        let value = parse("[1, 2, 3]");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])
        )
    }

    #[test]
    fn array_with_object() {
        let value = parse("[{\"key\": null}]");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Array(vec![json_object! { "key" => Value::Null }])
        )
    }

    #[test]
    fn object_with_number() {
        let value = parse("{\"key\": 1}");
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), json_object! { "key" => Value::Number(1.0) })
    }

    #[test]
    fn object_with_string() {
        let value = parse("{\"key\": \"value\"}");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            json_object! { "key" => Value::String("value".to_string()) }
        )
    }

    #[test]
    fn object_with_null() {
        let value = parse("{\"key\": null}");
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), json_object! { "key" => Value::Null })
    }

    #[test]
    fn object_with_true() {
        let value = parse("{\"key\": true}");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            json_object! { "key" => Value::Boolean(true) }
        )
    }

    #[test]
    fn object_with_false() {
        let value = parse("{\"key\": false}");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            json_object! { "key" => Value::Boolean(false) }
        )
    }

    #[test]
    fn nested_object() {
        let value = parse("{\"key\": { \"innerkey\": null } }");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            json_object! { "key" => json_object! { "innerkey" => Value::Null}}
        )
    }

    #[test]
    fn err_unclosed_array() {
        let err = parse("[null");
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), ParseError::UnclosedBracket)
    }

    #[test]
    fn err_unclosed_object() {
        let err = parse("{\"key\":\"value\"");
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), ParseError::UnclosedBrace)
    }

    #[test]
    fn err_expected_value() {
        let err = parse("]");
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), ParseError::ExpectedValue)
    }
}
