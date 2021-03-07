use crate::{
    lexer::{LexError, LexErrorKind, LexResult},
    location::Location,
    token::{Token, TokenKind},
    value::Value,
};

enum ParserState {
    Initial,
    ParsingArray,
    ParsingObject,
}

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

pub type ParseResult = Result<Value, ParseError>;

pub struct Parser<I: Iterator<Item = LexResult>> {
    /// Iterator that returns LexResult
    lexer: I,

    /// Finite state of the parser
    state: ParserState,
}

impl<I: Iterator<Item = LexResult>> Parser<I> {
    pub fn new(lexer: I) -> Self {
        Parser {
            lexer,
            // phantom: PhantomData {},
            state: ParserState::Initial,
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        let token = self
            .lexer
            .next()
            .unwrap_or(Err(LexError::empty()))
            .map_err(|err| ParseError::from_lex_error(&err))?;
        match token.kind {
            TokenKind::Null => Ok(Value::Null),
            TokenKind::String(s) => Ok(Value::String(s)),
            TokenKind::Number(n) => Ok(Value::Number(n)),
            TokenKind::Bool(b) => Ok(Value::Bool(b)),
            TokenKind::LeftBrace => self.parse_object(),
            TokenKind::RightBrace => make_error!(ParseErrorKind::UnexpectedRightBrace, token),
            TokenKind::LeftBracket => self.parse_array(),
            TokenKind::RightBracket => make_error!(ParseErrorKind::UnexpectedRightBracket, token),
            TokenKind::Comma => make_error!(ParseErrorKind::UnexpectedComma, token),
            TokenKind::Colon => make_error!(ParseErrorKind::UnexpectedColon, token),
            TokenKind::EOF => make_error!(ParseErrorKind::EarlyEOF, token),
        }
    }

    pub fn parse_array(&mut self) -> ParseResult {
        let mut array: Vec<Value> = Vec::new();
        let mut needs_comma = false;
        loop {
            let token = self
                .lexer
                .next()
                .unwrap_or(Ok(Token {
                    kind: TokenKind::EOF,
                    location: Location { row: 0, col: 0 },
                }))
                .map_err(|err| ParseError::from_lex_error(&err))?;
            //self.advance();
            //let token = self.lexer.curr().unwrap_or(default);
            let result: ParseResult = match &token.kind {
                TokenKind::String(s) => {
                    check_comma!(needs_comma, Value::String(s.to_string()), token)
                }
                TokenKind::Number(n) => check_comma!(needs_comma, Value::Number(n.clone()), token),
                TokenKind::Null => check_comma!(needs_comma, Value::Null, token),
                TokenKind::Bool(b) => check_comma!(needs_comma, Value::Bool(b.clone()), token),
                TokenKind::LeftBrace => self.parse_object(),
                TokenKind::LeftBracket => self.parse_array(),
                TokenKind::RightBracket => break,
                TokenKind::Comma => {
                    if needs_comma {
                        needs_comma = false;
                        continue;
                    } else {
                        Err(ParseError {
                            kind: ParseErrorKind::UnexpectedComma,
                            location: token.location,
                        })
                    }
                }
                TokenKind::RightBrace => make_error!(ParseErrorKind::UnexpectedRightBrace, token),
                TokenKind::Colon => make_error!(ParseErrorKind::UnexpectedColon, token),
                TokenKind::EOF => make_error!(ParseErrorKind::UnclosedBracket, token),
            };
            match result {
                Ok(value) => array.push(value),
                Err(err) => return Err(err),
            }
        }
        Ok(Value::Array(array))
    }

    pub fn parse_object(&mut self) -> ParseResult {
        Err(ParseError::empty())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ParseError {
    kind: ParseErrorKind,
    location: Location,
}

impl ParseError {
    pub fn empty() -> Self {
        ParseError {
            kind: ParseErrorKind::Empty,
            location: Location { row: 0, col: 0 },
        }
    }

    pub fn from_lex_error(lex_err: &LexError) -> Self {
        ParseError {
            kind: ParseErrorKind::LexError(lex_err.kind),
            location: lex_err.location,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseErrorKind {
    LexError(LexErrorKind),

    // Empty
    Empty,

    // EOF too soon
    EarlyEOF,
    UnclosedBracket,
    UnclosedBrace,

    // Needed EOF
    ExpectedEOF,

    // Missing Characters
    NeedsComma,

    // Unexpected Characters
    UnexpectedColon,
    UnexpectedComma,
    UnexpectedRightBrace,
    UnexpectedRightBracket,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(input: &str) -> ParseResult {
        let mut parser = Parser::new(Lexer::new(input.chars()));
        parser.parse()
    }

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
            Value::Array(vec![Value::Number(1), Value::Number(2), Value::Number(3)])
        )
    }
}
