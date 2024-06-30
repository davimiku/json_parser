use std::collections::HashMap;

use crate::Value;

use super::tokenize::Token;

pub type ParseResult = Result<Value, TokenParseError>;

pub fn parse_tokens(tokens: &Vec<Token>, index: &mut usize) -> ParseResult {
    let token = &tokens[*index];
    if matches!(
        token,
        Token::Null | Token::False | Token::True | Token::Number(_) | Token::String(_)
    ) {
        *index += 1
    }
    match token {
        Token::Null => Ok(Value::Null),
        Token::False => Ok(Value::Boolean(false)),
        Token::True => Ok(Value::Boolean(true)),
        Token::Number(number) => Ok(Value::Number(*number)),
        Token::String(string) => parse_string(string),
        Token::LeftBracket => parse_array(tokens, index),
        Token::LeftBrace => parse_object(tokens, index),
        _ => Err(TokenParseError::ExpectedValue),
    }
}

fn parse_string(input: &str) -> ParseResult {
    let unescaped = unescape_string(input)?;
    Ok(Value::String(unescaped))
}

fn unescape_string(input: &str) -> Result<String, TokenParseError> {
    // Create a new string to hold the processed/unescaped characters
    let mut output = String::new();

    let mut is_escaping = false;
    let mut chars = input.chars();
    while let Some(next_char) = chars.next() {
        if is_escaping {
            match next_char {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                // `\b` (backspace) is a valid escape in JSON, but not Rust
                'b' => output.push('\u{8}'),
                // `\f` (formfeed) is a valid escape in JSON, but not Rust
                'f' => output.push('\u{12}'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                'u' => {
                    let mut sum = 0;
                    for i in 0..4 {
                        let next_char = chars.next().ok_or(TokenParseError::UnfinishedEscape)?;
                        let digit = next_char
                            .to_digit(16)
                            .ok_or(TokenParseError::InvalidHexValue)?;
                        sum += (16u32).pow(3 - i) * digit;
                    }
                    let unescaped_char =
                        char::from_u32(sum).ok_or(TokenParseError::InvalidHexValue)?;
                    output.push(unescaped_char);
                }
                // any other character *may* be escaped, ex. `\q` just push that letter `q`
                _ => output.push(next_char),
            }
            is_escaping = false;
        } else if next_char == '\\' {
            is_escaping = true;
        } else {
            output.push(next_char);
        }
    }
    Ok(output)
}

fn parse_array(tokens: &Vec<Token>, index: &mut usize) -> ParseResult {
    debug_assert!(tokens[*index] == Token::LeftBracket);

    let mut array: Vec<Value> = Vec::new();
    loop {
        // consume the previous LeftBracket or Comma token
        *index += 1;
        if tokens[*index] == Token::RightBracket {
            break;
        }

        let value = parse_tokens(tokens, index)?;
        array.push(value);

        let token = &tokens[*index];
        match token {
            Token::Comma => {}
            Token::RightBracket => break,
            _ => return Err(TokenParseError::ExpectedComma),
        }
    }
    *index += 1;

    Ok(Value::Array(array))
}

fn parse_object(tokens: &Vec<Token>, index: &mut usize) -> ParseResult {
    debug_assert!(tokens[*index] == Token::LeftBrace);

    let mut map = HashMap::new();
    loop {
        // consume the previous LeftBrace or Comma token
        *index += 1;
        if tokens[*index] == Token::RightBrace {
            break;
        }

        if let Token::String(s) = &tokens[*index] {
            *index += 1;
            if Token::Colon == tokens[*index] {
                *index += 1;
                let key = unescape_string(s)?;
                let value = parse_tokens(tokens, index)?;
                map.insert(key, value);
            } else {
                return Err(TokenParseError::ExpectedColon);
            }

            match &tokens[*index] {
                Token::Comma => {}
                Token::RightBrace => break,
                _ => return Err(TokenParseError::ExpectedComma),
            }
        } else {
            return Err(TokenParseError::ExpectedProperty);
        }
    }
    *index += 1;

    Ok(Value::Object(map))
}

#[derive(Debug, PartialEq)]
pub enum TokenParseError {
    EarlyEOF,
    UnclosedBracket,
    UnclosedBrace,

    UnfinishedEscape,
    InvalidHexValue,
    InvalidCodePointValue,

    ExpectedColon,
    ExpectedComma,
    ExpectedValue,
    ExpectedProperty,

    NeedsComma,
    TrailingComma,
}

#[cfg(test)]
mod tests {
    use crate::tokenize::Token;
    use crate::Value;

    use super::{parse_tokens, TokenParseError};

    /// Helper to reduce boilerplate of asserting on the expected value
    ///
    /// Test functions in Rust are regular functions, which can call other helper
    /// functions such as this one. In this case, we're not saving much code, so
    /// this is just being shown for the sake of example.
    ///
    /// In other cases, a function like this may really help with readability.
    fn check(input: Vec<Token>, expected: Value) {
        let actual = parse_tokens(&input, &mut 0).unwrap();
        assert_eq!(actual, expected);
    }

    fn check_error(input: Vec<Token>, expected: TokenParseError) {
        let actual = parse_tokens(&input, &mut 0).unwrap_err();
        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_null() {
        let input = vec![Token::Null];
        let expected = Value::Null;

        check(input, expected);
    }

    #[test]
    fn parses_false() {
        let input = vec![Token::False];
        let expected = Value::Boolean(false);

        check(input, expected);
    }

    #[test]
    fn parses_true() {
        let input = vec![Token::True];
        let expected = Value::Boolean(true);

        check(input, expected);
    }

    #[test]
    fn parses_number() {
        let input = vec![Token::Number(12.34)];
        let expected = Value::Number(12.34);

        check(input, expected);
    }

    #[test]
    fn parses_string_no_escapes() {
        let input = vec![Token::string("hello world")];
        let expected = Value::String(String::from("hello world"));

        check(input, expected);
    }

    #[test]
    fn parses_string_non_ascii() {
        let input = vec![Token::string("olá_こんにちは_नमस्ते_привіт")];
        let expected = Value::String(String::from("olá_こんにちは_नमस्ते_привіт"));

        check(input, expected);
    }

    #[test]
    fn parses_string_with_unescaped_emoji() {
        let input = vec![Token::string("hello 💩 world")];
        let expected = Value::String(String::from("hello 💩 world"));

        check(input, expected);
    }

    #[test]
    fn parses_string_with_unnecessarily_escaped_emoji() {
        let input = vec![Token::string(r#"hello \💩 world"#)];
        let expected = Value::String(String::from("hello 💩 world"));

        check(input, expected);
    }

    #[test]
    fn parses_string_unescape_backslash() {
        let input = vec![Token::string(r#"hello\\world"#)];
        let expected = Value::String(String::from(r#"hello\world"#));

        check(input, expected);
    }

    #[test]
    fn parses_string_unescape_newline() {
        let input = vec![Token::string(r#"hello\nworld"#)];
        let expected = Value::String(String::from("hello\nworld"));

        check(input, expected);
    }

    #[test]
    #[ignore = "decoding of UTF-16 surrogate pairs is not implemented"]
    fn parses_string_with_escaped_surrogate_pairs_for_an_emoji() {
        let input = vec![Token::string(r#"hello\uD83C\uDF3Cworld"#)];
        let expected = Value::String(String::from("hello🌼world"));

        check(input, expected);
    }

    #[test]
    fn all_the_simple_escapes() {
        let input = vec![Token::string(r#"\"\/\\\b\f\n\r\t"#)];
        let expected = Value::String(String::from("\"/\\\u{8}\u{12}\n\r\t"));

        check(input, expected);
    }

    #[test]
    fn parses_empty_arrays() {
        // []
        let input = vec![Token::LeftBracket, Token::RightBracket];
        let expected = Value::Array(vec![]);

        check(input, expected);
    }

    #[test]
    fn parses_array_one_element() {
        // [true]
        let input = vec![Token::LeftBracket, Token::True, Token::RightBracket];
        let expected = Value::Array(vec![Value::Boolean(true)]);

        check(input, expected);
    }

    #[test]
    fn parses_array_two_elements() {
        // [null, 16]
        let input = vec![
            Token::LeftBracket,
            Token::Null,
            Token::Comma,
            Token::Number(16.0),
            Token::RightBracket,
        ];
        let expected = Value::Array(vec![Value::Null, Value::Number(16.0)]);

        check(input, expected);
    }

    #[test]
    fn parses_nested_array() {
        // [null, [null]]
        let input = vec![
            Token::LeftBracket,
            Token::Null,
            Token::Comma,
            Token::LeftBracket,
            Token::Null,
            Token::RightBracket,
            Token::RightBracket,
        ];
        let expected = Value::Array(vec![Value::Null, Value::Array(vec![Value::Null])]);

        check(input, expected);
    }

    #[test]
    fn fails_array_leading_comma() {
        // [,true]
        let input = vec![
            Token::LeftBracket,
            Token::Comma,
            Token::True,
            Token::RightBracket,
        ];
        let expected = TokenParseError::ExpectedValue;

        check_error(input, expected);
    }

    #[test]
    #[ignore = "the current implementation allows trailing commas"]
    fn fails_array_trailing_comma() {
        // [true,]
        let input = vec![
            Token::LeftBracket,
            Token::True,
            Token::Comma,
            Token::RightBracket,
        ];
        let expected = TokenParseError::TrailingComma;

        check_error(input, expected);
    }

    #[test]
    fn parses_empty_object() {
        let input = vec![Token::LeftBrace, Token::RightBrace];
        let expected = Value::object([]);

        check(input, expected);
    }

    #[test]
    fn parses_object_one_string_value() {
        let input = vec![
            Token::LeftBrace,
            Token::string("name"),
            Token::Colon,
            Token::string("davimiku"),
            Token::RightBrace,
        ];
        let expected = Value::object([("name", Value::string("davimiku"))]);

        check(input, expected);
    }

    #[test]
    fn parses_object_escaped_key() {
        let input = vec![
            Token::LeftBrace,
            Token::string(r#"\u540D\u524D"#),
            Token::Colon,
            Token::string("davimiku"),
            Token::RightBrace,
        ];
        let expected = Value::object([("名前", Value::string("davimiku"))]);

        check(input, expected);
    }
}
