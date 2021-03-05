use crate::location::Location;
use crate::token::{Token, TokenKind};
use std::string::String;

#[derive(Debug, PartialEq)]
pub enum LexErrorKind {
    CharNotRecognized(char),
    ParseNumberError,
    UnclosedQuotes,
    UnfinishedNullValue,
}

#[derive(Debug, PartialEq)]
pub struct LexError {
    kind: LexErrorKind,
    location: Location,
}

pub struct Lexer<I: Iterator<Item = char>> {
    iter: I,

    /// Current location of the cursor
    location: Location,

    /// The current character in the iterator
    curr: Option<char>,

    /// The next character in the iterator
    next: Option<char>,
}

impl<I: Iterator<Item = char>> Lexer<I> {
    /// Construct a new instance
    pub fn new(char_iter: I) -> Self {
        let mut lex = Lexer {
            iter: char_iter,
            location: Location::new(),
            curr: None,
            next: None,
        };
        lex.init();
        lex
    }

    fn init(&mut self) {
        self.next = self.iter.next();
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
        self.next = self.iter.next();
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

    /// Lexes the literal characters `null`
    fn lex_null(&mut self) -> Result<TokenKind, LexErrorKind> {
        let null_iter = "null".chars();
        for null_char in null_iter {
            if self.curr.is_none() || self.curr.unwrap() != null_char {
                return Err(LexErrorKind::UnfinishedNullValue);
            }
            self.advance();
        }
        Ok(TokenKind::Null)
    }

    /// Lexes a number
    fn lex_number(&mut self) -> Result<i64, LexErrorKind> {
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
            .parse::<i64>()
            .map_err(|_| LexErrorKind::ParseNumberError)
    }
}

impl<I: Iterator<Item = char>> Iterator for Lexer<I> {
    type Item = Result<Token, LexError>;

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
            '{' => Ok(TokenKind::LeftCurly),
            '}' => Ok(TokenKind::RightCurly),
            ',' => Ok(TokenKind::Comma),
            ':' => Ok(TokenKind::Colon),
            '"' => match self.lex_string() {
                Ok(str) => Ok(TokenKind::Str(str)),
                Err(err) => Err(err),
            },
            'n' => match self.lex_null() {
                Ok(t) => Ok(t),
                Err(err) => Err(err),
            },
            c if c.is_digit(10) => match self.lex_number() {
                Ok(i) => Ok(TokenKind::Number(i)),
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
    use crate::token::TokenKind;

    fn lex(input: &str) -> Vec<TokenKind> {
        Lexer::new(input.chars())
            .take_while(Result::is_ok)
            .flat_map(|e| e.ok())
            .map(|t| t.kind)
            .collect::<Vec<TokenKind>>()
    }

    #[test]
    fn punctuation() {
        let input = "[]{},:";
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBracket,
            TokenKind::RightBracket,
            TokenKind::LeftCurly,
            TokenKind::RightCurly,
            TokenKind::Comma,
            TokenKind::Colon,
        ];
        let actual = lex(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn key() {
        let input = "\"key\"";
        let expected: Vec<TokenKind> = vec![TokenKind::Str("key".to_string())];
        let actual = lex(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn key_colon() {
        let input = "\"key\":";
        let expected: Vec<TokenKind> = vec![TokenKind::Str("key".to_string()), TokenKind::Colon];
        let actual = lex(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_object() {
        let input = "{\"key\":\"value\"}";
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftCurly,
            TokenKind::Str("key".to_string()),
            TokenKind::Colon,
            TokenKind::Str("value".to_string()),
            TokenKind::RightCurly,
        ];
        let actual = lex(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn array_with_strings() {
        let input = "[\"A\", \"B\"]";
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBracket,
            TokenKind::Str("A".to_string()),
            TokenKind::Comma,
            TokenKind::Str("B".to_string()),
            TokenKind::RightBracket,
        ];
        let actual = lex(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn array_with_numbers() {
        let input = "[1, 2]";
        let expected: Vec<TokenKind> = vec![
            TokenKind::LeftBracket,
            TokenKind::Number(1),
            TokenKind::Comma,
            TokenKind::Number(2),
            TokenKind::RightBracket,
        ];
        let actual = lex(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_null() {
        let input = "null";
        let expected: Vec<TokenKind> = vec![TokenKind::Null];
        let actual = lex(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_int() {
        let input = "123";
        let expected: Vec<TokenKind> = vec![TokenKind::Number(123)];
        let actual = lex(input);

        assert_eq!(actual, expected);
    }
}
