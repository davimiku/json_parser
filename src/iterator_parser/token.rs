use std::fmt::{self, Write};

use crate::location::Location;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    /// Literals
    Number(i64),
    Null,
    Bool(bool),

    /// Key of the key/value pair or string value
    String(String),

    /// Punctuation
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,

    EOF,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::LeftBracket => f.write_char('['),
            TokenKind::RightBracket => f.write_char(']'),
            TokenKind::LeftBrace => f.write_char('{'),
            TokenKind::RightBrace => f.write_char('}'),
            TokenKind::Comma => f.write_char(','),
            TokenKind::Colon => f.write_char(':'),
            TokenKind::String(val) => write!(f, "{:?}", val),
            TokenKind::Number(val) => write!(f, "{:?}", val),
            TokenKind::Bool(val) => write!(f, "{:?}", val),
            TokenKind::Null => f.write_str("null"),
            TokenKind::EOF => f.write_str("END_OF_FILE"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "token '{}' at row {}, col {}",
            self.kind, self.location.row, self.location.col,
        ))
    }
}
