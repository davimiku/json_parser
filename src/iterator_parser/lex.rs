use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // punctuation
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,

    // literal values
    Null,
    False,
    True,

    Float(f64),

    /// Key of the key/value pair or string value
    String(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::LeftBracket => f.write_str("["),
            Token::RightBracket => f.write_str("]"),
            Token::LeftBrace => f.write_str("{"),
            Token::RightBrace => f.write_str("}"),
            Token::Comma => f.write_str(","),
            Token::Colon => f.write_str(":"),
            Token::Null => f.write_str("null"),
            Token::False => f.write_str("false"),
            Token::True => f.write_str("true"),
            Token::String(val) => write!(f, "{val:?}"),
            Token::Float(val) => write!(f, "{val:?}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexError {
    CharNotRecognized(char),
    ParseNumberError,
    UnclosedQuotes,
    UnfinishedNullValue,
    UnfinishedBoolValue(bool),
}

struct Lexer {
    /// Characters from the input
    chars: Vec<char>,

    /// Current index being lexed
    curr_idx: usize,
}

pub fn lex(input: String) -> Vec<Token> {
    let lexer = &mut Lexer {
        chars: input.chars().collect(),
        curr_idx: 0,
    };

    let mut tokens = vec![];
    while lexer.curr_idx < lexer.chars.len() - 1 {
        let ch = curr_char(lexer);
        let token = lex_token(lexer, ch)?;
        tokens.push(token);
        advance(lexer);
    }
    tokens

    // Ok(tokens)
}

fn curr_char(lexer: &Lexer) -> char {
    lexer.chars[lexer.curr_idx]
}

fn advance(lexer: &mut Lexer) {
    loop {
        lexer.curr_idx += 1;
        if !curr_char(lexer).is_whitespace() {
            break;
        }
    }
}

fn lex_token(lexer: &mut Lexer, ch: char) -> Result<Token, LexError> {
    match ch {
        '[' => Ok(Token::LeftBracket),
        ']' => Ok(Token::RightBracket),
        '{' => Ok(Token::LeftBrace),
        '}' => Ok(Token::RightBrace),
        ',' => Ok(Token::Comma),
        ':' => Ok(Token::Colon),
        '"' => lex_string(lexer).map(Token::String),
        'n' => lex_literal(lexer, "null", Token::Null, LexError::UnfinishedNullValue),
        't' => lex_literal(
            lexer,
            "true",
            Token::True,
            LexError::UnfinishedBoolValue(true),
        ),
        'f' => lex_literal(
            lexer,
            "false",
            Token::False,
            LexError::UnfinishedBoolValue(false),
        ),
        '-' => lex_float(lexer).map(Token::Float),
        c if c.is_ascii_digit() => lex_float(lexer).map(Token::Float),

        c => Err(LexError::CharNotRecognized(c)),
    }
}

/// Lexes a string
fn lex_string(lexer: &mut Lexer) -> Result<String, LexError> {
    debug_assert!(curr_char(lexer) == '"');

    let mut str = String::new();

    loop {
        advance(lexer);
        let ch = curr_char(lexer);
        if ch == '"' {
            break;
        }
        str.push(ch);
    }

    Ok(str)
}

fn lex_literal(
    lexer: &mut Lexer,
    expected: &str,
    ok_kind: Token,
    error_kind: LexError,
) -> Result<Token, LexError> {
    let mut expected_iter = expected.chars();
    expected_iter.next();

    for expected_char in expected_iter {
        let ch = curr_char(lexer);
        if ch != expected_char {
            return Err(error_kind);
        }
        advance(lexer);
    }
    Ok(ok_kind)
}

/// Lexes a number as f64
fn lex_float(lexer: &mut Lexer) -> Result<f64, LexError> {
    let mut val = String::new();
    val.push(curr_char(lexer));

    let mut has_decimal = false;

    loop {
        let ch = curr_char(lexer);
        if !ch.is_ascii_digit() {
            if ch == '.' && !has_decimal {
                has_decimal = true;
            } else {
                break;
            }
        }
        val.push(ch);
        advance(lexer);
    }

    val.parse::<f64>().map_err(|_| LexError::ParseNumberError)
}

// impl Lexer {
//     fn curr_char(&self) -> char {
//         self.chars[self.curr_idx]
//     }

//     fn advance(&mut self) {
//         loop {
//             self.curr_idx += 1;
//             if !self.curr_char().is_whitespace() {
//                 break;
//             }
//         }
//     }

//     fn lex_token(&mut self, ch: char) -> Result<Token, LexError> {
//         match ch {
//             '[' => Ok(Token::LeftBracket),
//             ']' => Ok(Token::RightBracket),
//             '{' => Ok(Token::LeftBrace),
//             '}' => Ok(Token::RightBrace),
//             ',' => Ok(Token::Comma),
//             ':' => Ok(Token::Colon),
//             '"' => self.lex_string().map(Token::String),
//             'n' => self.lex_literal("null", Token::Null, LexError::UnfinishedNullValue),
//             't' => self.lex_literal("true", Token::True, LexError::UnfinishedBoolValue(true)),
//             'f' => self.lex_literal("false", Token::False, LexError::UnfinishedBoolValue(false)),
//             '-' => self.lex_float().map(Token::Float),
//             c if c.is_ascii_digit() => self.lex_float().map(Token::Float),

//             c => Err(LexError::CharNotRecognized(c)),
//         }
//     }

//     /// Lexes a string
//     fn lex_string(&mut self) -> Result<String, LexError> {
//         debug_assert!(self.curr_char() == '"');

//         let mut str = String::new();

//         loop {
//             self.advance();
//             let ch = self.curr_char();
//             if ch == '"' {
//                 break;
//             }
//             str.push(ch);
//         }

//         Ok(str)
//     }

//     fn lex_literal(
//         &mut self,
//         expected: &str,
//         ok_kind: Token,
//         error_kind: LexError,
//     ) -> Result<Token, LexError> {
//         let mut expected_iter = expected.chars();
//         expected_iter.next();

//         for expected_char in expected_iter {
//             let ch = self.curr_char();
//             if ch != expected_char {
//                 return Err(error_kind);
//             }
//             self.advance();
//         }
//         Ok(ok_kind)
//     }

//     /// Lexes a number as f64
//     fn lex_float(&mut self) -> Result<f64, LexError> {
//         let mut val = String::new();
//         val.push(self.curr_char());

//         let mut has_decimal = false;

//         loop {
//             let ch = self.curr_char();
//             if !ch.is_ascii_digit() {
//                 if ch == '.' && !has_decimal {
//                     has_decimal = true;
//                 } else {
//                     break;
//                 }
//             }
//             val.push(ch);
//             self.advance();
//         }

//         val.parse::<f64>().map_err(|_| LexError::ParseNumberError)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_test<T>(input: &str, expected: T)
    where T: Into<Vec<Token>> {
        let actual = lex(input.into());

        assert_eq!(actual, expected.into())

    }

    #[test]
    fn punctuation() {
        let expected = [
            Token::LeftBracket,
            Token::RightBracket,
            Token::LeftBrace,
            Token::RightBrace,
            Token::Comma,
            Token::Colon,
        ];
        lex_test("[]{},:", expected);
    }

    #[test]
    fn key() {
        let expected = [Token::String("key".to_string())];
        lex_test("\"key\"", expected);
    }

    #[test]
    fn key_colon() {
        let expected = [Token::String("key".to_string()), Token::Colon];
        lex_test("\"key\":", expected);
    }

    #[test]
    fn simple_object() {
        let expected = [
            Token::LeftBrace,
            Token::String("key".to_string()),
            Token::Colon,
            Token::String("value".to_string()),
            Token::RightBrace,
        ];
       lex_test("{\"key\":\"value\"}", expected);
    }

    #[test]
    fn array_with_strings() {
        let expected = [
            Token::LeftBracket,
            Token::String("A".to_string()),
            Token::Comma,
            Token::String("B".to_string()),
            Token::RightBracket,
        ];
        lex_test("[\"A\", \"B\"]", expected);
    }

    #[test]
    fn empty_string() {
        let expected = [
            Token::LeftBracket,
            Token::String("".to_string()),
            Token::RightBracket,
        ];
        lex_test("[\"\"]", expected);
    }

    #[test]
    fn array_with_numbers() {
        let expected = [
            Token::LeftBracket,
            Token::Float(1.0),
            Token::Comma,
            Token::Float(2.0),
            Token::RightBracket,
        ];
       lex_test("[1, 2]", expected);

    }

    #[test]
    fn just_null() {
        let expected = [Token::Null];
         lex_test("null", expected);

    }

    #[test]
    fn just_true() {
        let expected = [Token::True];
         lex_test("true", expected);

    }

    #[test]
    fn just_false() {
        let expected = [Token::False];
        lex_test("false", expected);
    }

    #[test]
    fn positive_int() {
        let expected = [Token::Float(123.0)];
        lex_test("123", expected);
    }

    #[test]
    fn negative_int() {
        let expected = [Token::Float(-123.0)];
        lex_test("-123", expected);
    }

    #[test]
    fn array_with_null() {
        let expected = [Token::LeftBracket, Token::Null, Token::RightBracket];
        lex_test("[null]", expected);
    }

    #[test]
    fn array_with_true_false() {
        let expected = [
            Token::LeftBracket,
            Token::True,
            Token::Comma,
            Token::False,
            Token::RightBracket,
        ];
        lex_test("[true, false]", expected);
    }
}
