use std::fmt::{self, Write};
use TokenKind::*;

use crate::location::Location;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    /// Key of the key/value pair or string value
    Str(String),

    /// Literals
    Number(i64),
    Null,
    Bool(bool),

    /// Punctuation
    LeftCurly,
    RightCurly,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,

    EOF,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LeftBracket => f.write_char('['),
            RightBracket => f.write_char(']'),
            LeftCurly => f.write_char('{'),
            RightCurly => f.write_char('}'),
            Comma => f.write_char(','),
            Colon => f.write_char(':'),
            Str(val) => write!(f, "{:?}", val),
            Number(val) => write!(f, "{:?}", val),
            Bool(val) => write!(f, "{:?}", val),
            Null => f.write_str("null"),
            EOF => f.write_str("END_OF_FILE"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
}
