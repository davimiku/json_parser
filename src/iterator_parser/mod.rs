use std::collections::BTreeMap;

use crate::{
    location::Location,
    value::{NumberValue, Value},
};
mod lexer;
mod token;
use lexer::{LexError, LexErrorKind, LexResult, Lexer};
use token::{Token, TokenKind};

macro_rules! check_comma {
    ($needs_comma:expr,$value:expr,$token:expr) => {
        match $needs_comma {
            true => Err(ParseError {
                kind: ParseErrorKind::NeedsComma,
                location: $token.location,
            }),
            false => {
                $needs_comma = true;
                Ok($value)
            }
        }
    };
}

macro_rules! make_error {
    ($kind:expr,$token:expr) => {
        Err(ParseError {
            kind: $kind,
            location: $token.location,
        })
    };
}

struct Parser<I: Iterator<Item = LexResult>> {
    /// Iterator that returns LexResult
    lexer: I,
}

impl<I: Iterator<Item = LexResult>> Parser<I> {
    pub fn new(lexer: I) -> Self {
        Parser { lexer }
    }

    pub fn parse(&mut self) -> ParseResult {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Null => Ok(Value::Null),
            TokenKind::String(s) => Ok(Value::String(s)),
            TokenKind::Int(i) => Ok(Value::Number(NumberValue::Int(i))),
            TokenKind::UInt(u) => Ok(Value::Number(NumberValue::UInt(u))),
            TokenKind::Float(f) => Ok(Value::Number(NumberValue::Float(f))),
            TokenKind::Bool(b) => Ok(Value::Bool(b)),
            TokenKind::LeftBrace => self.parse_object(),
            TokenKind::LeftBracket => self.parse_array(),
            TokenKind::EOF => make_error!(ParseErrorKind::EarlyEOF, token),
            _ => make_error!(ParseErrorKind::ExpectedValue, token),
        }
    }

    fn parse_array(&mut self) -> ParseResult {
        let mut array: Vec<Value> = Vec::new();
        let mut needs_comma = false;
        loop {
            let token = self.advance()?;
            let result: ParseResult = match &token.kind {
                TokenKind::String(s) => {
                    check_comma!(needs_comma, Value::String(s.to_string()), token)
                }
                TokenKind::Int(i) => {
                    check_comma!(needs_comma, Value::Number(NumberValue::Int(*i)), token)
                }
                TokenKind::UInt(u) => {
                    check_comma!(needs_comma, Value::Number(NumberValue::UInt(*u)), token)
                }
                TokenKind::Float(f) => {
                    check_comma!(needs_comma, Value::Number(NumberValue::Float(*f)), token)
                }
                TokenKind::Null => check_comma!(needs_comma, Value::Null, token),
                TokenKind::Bool(b) => check_comma!(needs_comma, Value::Bool(*b), token),
                TokenKind::LeftBrace => self.parse_object(),
                TokenKind::LeftBracket => self.parse_array(),
                TokenKind::RightBracket => break,
                TokenKind::Comma => {
                    if needs_comma {
                        needs_comma = false;
                        continue;
                    } else {
                        Err(ParseError {
                            kind: ParseErrorKind::TrailingComma,
                            location: token.location,
                        })
                    }
                }
                TokenKind::EOF => make_error!(ParseErrorKind::UnclosedBracket, token),
                _ => make_error!(ParseErrorKind::ExpectedValue, token),
            };
            match result {
                Ok(value) => array.push(value),
                Err(err) => return Err(err),
            }
        }
        Ok(Value::Array(array))
    }

    fn parse_object(&mut self) -> ParseResult {
        let mut map: BTreeMap<String, Value> = BTreeMap::new();
        loop {
            let token = self.advance()?;
            match &token.kind {
                TokenKind::String(key) => {
                    let token = self.advance()?;
                    match &token.kind {
                        TokenKind::Colon => {
                            let value = self.parse()?;
                            map.insert(key.to_string(), value);
                        }
                        _ => {
                            return Err(ParseError {
                                kind: ParseErrorKind::ExpectedColon,
                                location: token.location,
                            })
                        }
                    }
                    let next = self.advance()?;
                    match &next.kind {
                        TokenKind::Comma => {}
                        TokenKind::RightBrace => break, // object parsing ended
                        TokenKind::EOF => {
                            return Err(ParseError {
                                kind: ParseErrorKind::UnclosedBrace,
                                location: token.location,
                            })
                        }
                        _ => {
                            return Err(ParseError {
                                kind: ParseErrorKind::ExpectedComma,
                                location: token.location,
                            })
                        }
                    }
                }
                TokenKind::EOF => {
                    return Err(ParseError {
                        kind: ParseErrorKind::UnclosedBrace,
                        location: token.location,
                    })
                }
                _ => {
                    return Err(ParseError {
                        kind: ParseErrorKind::ExpectedProperty,
                        location: token.location,
                    })
                }
            };
        }
        Ok(Value::Object(map))
    }

    /// Advance the lexer and provide the next token
    /// or a parsing error
    fn advance(&mut self) -> Result<Token, ParseError> {
        self.lexer
            .next()
            .unwrap_or(Ok(Token {
                kind: TokenKind::EOF,
                location: Location { row: 0, col: 0 },
            }))
            .map_err(|err| ParseError::from_lex_error(&err))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ParseError {
    kind: ParseErrorKind,
    location: Location,
}

impl ParseError {
    fn from_lex_error(lex_err: &LexError) -> Self {
        ParseError {
            kind: ParseErrorKind::LexError(lex_err.kind),
            location: lex_err.location,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ParseErrorKind {
    LexError(LexErrorKind),

    // EOF too soon
    EarlyEOF,
    UnclosedBracket,
    UnclosedBrace,

    // Expected
    ExpectedColon,
    ExpectedComma,
    ExpectedValue, // Expected JSON object, array, or literal
    ExpectedProperty,

    // Missing Characters
    NeedsComma,

    // Unexpected Characters
    TrailingComma,
}

pub type ParseResult = Result<Value, ParseError>;

pub fn parse(input: &str) -> ParseResult {
    let mut parser = Parser::new(Lexer::new(input.chars()));
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{json_object, value::NumberValue};

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
        assert_eq!(value.unwrap(), Value::Bool(true));
    }

    #[test]
    fn just_false() {
        let value = parse("false");
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), Value::Bool(false));
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
            Value::Array(vec![Value::Bool(true), Value::Bool(false)])
        )
    }

    #[test]
    fn array_with_ints() {
        let value = parse("[1, 2, 3]");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Array(vec![
                Value::Number(NumberValue::UInt(1)),
                Value::Number(NumberValue::UInt(2)),
                Value::Number(NumberValue::UInt(3))
            ])
        )
    }

    #[test]
    fn array_with_object() {
        let value = parse("[{\"key\": null}]");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Array(vec![Value::Object(
                json_object! { "key" => Value::Null }
            )])
        )
    }

    #[test]
    fn object_with_int() {
        let value = parse("{\"key\": 1}");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Object(
                json_object! { "key" => Value::Number(NumberValue::UInt(1)) }
            )
        )
    }

    #[test]
    fn object_with_string() {
        let value = parse("{\"key\": \"value\"}");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Object(json_object! { "key" => Value::String("value".to_string()) })
        )
    }

    #[test]
    fn object_with_null() {
        let value = parse("{\"key\": null}");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Object(json_object! { "key" => Value::Null })
        )
    }

    #[test]
    fn object_with_true() {
        let value = parse("{\"key\": true}");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Object(json_object! { "key" => Value::Bool(true) })
        )
    }

    #[test]
    fn object_with_false() {
        let value = parse("{\"key\": false}");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Object(json_object! { "key" => Value::Bool(false) })
        )
    }

    #[test]
    fn nested_object() {
        let value = parse("{\"key\": { \"innerkey\": null } }");
        assert!(value.is_ok());
        assert_eq!(
            value.unwrap(),
            Value::Object(
                json_object! { "key" => Value::Object(json_object! { "innerkey" => Value::Null})}
            )
        )
    }

    #[test]
    fn err_unclosed_array() {
        let err = parse("[null");
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().kind, ParseErrorKind::UnclosedBracket)
    }

    #[test]
    fn err_unclosed_object() {
        let err = parse("{\"key\":\"value\"");
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().kind, ParseErrorKind::UnclosedBrace)
    }

    #[test]
    fn err_expected_value() {
        let err = parse("]");
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().kind, ParseErrorKind::ExpectedValue)
    }
}
