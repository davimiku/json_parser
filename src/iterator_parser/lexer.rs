use super::token::{Token, TokenKind};
use crate::location::Location;
use std::string::String;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LexErrorKind {
    CharNotRecognized(char),
    ParseNumberError,
    UnclosedQuotes,
    UnfinishedNullValue,
    UnfinishedBoolValue(bool),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub location: Location,
}

pub type LexResult = Result<Token, LexError>;

pub trait IteratorWithLocation: Iterator {
    fn location(self) -> Location;
}

impl<I: Iterator> IteratorWithLocation for I {
    fn location(self) -> Location {
        todo!()
    }
}

pub struct Lexer<I: Iterator<Item = char>> {
    /// Iterator for chars from the input
    char_iter: I,

    /// Current location of the cursor
    location: Location,

    /// The current character in the iterator
    curr: Option<char>,

    /// The next character in the iterator
    next: Option<char>,
}

impl<I: Iterator<Item = char>> Lexer<I> {
    /// Construct a new instance
    pub fn new(mut char_iter: I) -> Self {
        let next = char_iter.next();
        Lexer {
            char_iter,
            location: Location::new(),
            curr: None,
            next,
        }
    }

    /// Grabs the next character from the iterator and
    /// updates the cursor location.
    fn advance(&mut self) -> Option<char> {
        self.location.move_right();
        match self.curr {
            Some(ch) => {
                if ch == '\n' {
                    self.location.move_down();
                }
            }
            None => {}
        }
        self.curr = self.next.take();
        self.next = self.char_iter.next();
        self.curr
    }

    /// Lexes a string
    fn lex_string(&mut self) -> Result<String, LexErrorKind> {
        let mut str = String::new();

        // The current character is the opening quotes
        self.advance();
        loop {
            match self.curr {
                Some(ch) => {
                    // Check if the string is ended
                    // TODO: Update for backslash escaped quotes
                    if ch == '"' {
                        break;
                    }
                    str.push(ch);
                }
                None => return Err(LexErrorKind::UnclosedQuotes),
            }
            self.advance();
        }

        Ok(str)
    }

    fn lex_literal(
        &mut self,
        expected: &str,
        ok_kind: TokenKind,
        error_kind: LexErrorKind,
    ) -> Result<TokenKind, LexErrorKind> {
        let mut expected_iter = expected.chars();
        expected_iter.next();
        for expected_char in expected_iter {
            if self.next.is_none() || self.next.unwrap() != expected_char {
                return Err(error_kind);
            }
            self.advance();
        }
        Ok(ok_kind)
    }

    /// Lexes a number
    fn lex_uint(&mut self) -> Result<u64, LexErrorKind> {
        let mut value_str = String::new();
        value_str.push(self.curr.unwrap());
        loop {
            match self.next {
                Some(ch) => {
                    if !ch.is_digit(10) {
                        break;
                    }
                    value_str.push(ch);
                    self.advance();
                }
                None => break,
            }
        }

        value_str
            .parse::<u64>()
            .map_err(|_| LexErrorKind::ParseNumberError)
    }
}

impl<I: Iterator<Item = char>> Iterator for Lexer<I> {
    type Item = LexResult;
    /// Returns the next item from the lexer, or
    /// None if the input string is finished.
    ///
    /// The item signature is:
    /// Result<Token, LexError>
    fn next(&mut self) -> Option<Self::Item> {
        // Eat a character
        let mut ch = self.advance()?;
        loop {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {}
                _ => break,
            };
            ch = self.advance()?;
        }

        let result = match ch {
            '[' => Ok(TokenKind::LeftBracket),
            ']' => Ok(TokenKind::RightBracket),
            '{' => Ok(TokenKind::LeftBrace),
            '}' => Ok(TokenKind::RightBrace),
            ',' => Ok(TokenKind::Comma),
            ':' => Ok(TokenKind::Colon),
            '"' => match self.lex_string() {
                Ok(str) => Ok(TokenKind::String(str)),
                Err(err) => Err(err),
            },
            'n' => self.lex_literal("null", TokenKind::Null, LexErrorKind::UnfinishedNullValue),
            't' => self.lex_literal(
                "true",
                TokenKind::Bool(true),
                LexErrorKind::UnfinishedBoolValue(true),
            ),
            'f' => self.lex_literal(
                "false",
                TokenKind::Bool(false),
                LexErrorKind::UnfinishedBoolValue(false),
            ),
            c if c.is_digit(10) => match self.lex_uint() {
                Ok(u) => Ok(TokenKind::UInt(u)),
                Err(err) => Err(err),
            },
            c => Err(LexErrorKind::CharNotRecognized(c)),
        };

        let location = self.location.clone();
        Some(
            result
                .map(|t| Token { kind: t, location })
                .map_err(|kind| LexError { kind, location }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<TokenKind> {
        Lexer::new(input.chars())
            .take_while(Result::is_ok)
            .flat_map(|e| e.ok())
            .map(|t| t.kind)
            .collect::<Vec<TokenKind>>()
    }

    #[test]
    fn punctuation() {
        let actual = lex("[]{},:");
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBracket,
            TokenKind::RightBracket,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Comma,
            TokenKind::Colon,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn key() {
        let actual = lex("\"key\"");
        let expected: Vec<TokenKind> = vec![TokenKind::String("key".to_string())];

        assert_eq!(actual, expected);
    }

    #[test]
    fn key_colon() {
        let actual = lex("\"key\":");
        let expected: Vec<TokenKind> = vec![TokenKind::String("key".to_string()), TokenKind::Colon];

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_object() {
        let actual = lex("{\"key\":\"value\"}");
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBrace,
            TokenKind::String("key".to_string()),
            TokenKind::Colon,
            TokenKind::String("value".to_string()),
            TokenKind::RightBrace,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn array_with_strings() {
        let actual = lex("[\"A\", \"B\"]");
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBracket,
            TokenKind::String("A".to_string()),
            TokenKind::Comma,
            TokenKind::String("B".to_string()),
            TokenKind::RightBracket,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn empty_string() {
        let actual = lex("[\"\"]");
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBracket,
            TokenKind::String("".to_string()),
            TokenKind::RightBracket,
        ];

        assert_eq!(actual, expected)
    }

    #[test]
    fn array_with_numbers() {
        let actual = lex("[1, 2]");
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBracket,
            TokenKind::UInt(1),
            TokenKind::Comma,
            TokenKind::UInt(2),
            TokenKind::RightBracket,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_null() {
        let actual = lex("null");
        let expected: Vec<TokenKind> = vec![TokenKind::Null];

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_true() {
        let actual = lex("true");
        let expected: Vec<TokenKind> = vec![TokenKind::Bool(true)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_false() {
        let actual = lex("false");
        let expected: Vec<TokenKind> = vec![TokenKind::Bool(false)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_int() {
        let actual = lex("123");
        let expected: Vec<TokenKind> = vec![TokenKind::UInt(123)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn array_with_null() {
        let actual = lex("[null]");
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBracket,
            TokenKind::Null,
            TokenKind::RightBracket,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn array_with_true_false() {
        let actual = lex("[true, false]");
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBracket,
            TokenKind::Bool(true),
            TokenKind::Comma,
            TokenKind::Bool(false),
            TokenKind::RightBracket,
        ];
        assert_eq!(actual, expected);
    }
}

// /// LexerBuffer is a wrapper around the Lexer to make
// /// it more convenient to consume Tokens
// pub struct LexerBuffer<T, I: Iterator<Item = T>> {
//     iter: I,
//     curr: Option<T>,
//     next: Option<T>,
// }

// impl<T, I: Iterator<Item = T>> LexerBuffer<T, I> {
//     /// Create a new LexerBuffer
//     pub fn new(mut iter: I) -> Self {
//         let curr = iter.next();
//         let next = iter.next();
//         LexerBuffer { iter, curr, next }
//     }

//     /// Provides a reference to the current Token
//     pub fn curr(&self) -> &T {
//         self.curr.as_ref().unwrap_or(Token {
//             kind: TokenKind::EOF,
//             location: Location { row: 0, col: 0 },
//         })
//     }

//     /// Advances the Token iterator
//     pub fn advance_token(&mut self) {
//         self.curr = self.next.take();
//         self.next = self.iter.next();
//     }

//     /// Provides a borrow of the Token iterator
//     pub fn iter(&mut self) -> &I {
//         &self.iter
//     }
// }
