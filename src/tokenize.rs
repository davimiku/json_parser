use std::num::ParseFloatError;

/// A Token is
#[derive(Debug, PartialEq)]
pub enum Token {
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `,`
    Comma,
    /// `:`
    Colon,

    /// `null`
    Null,
    /// `false`
    False,
    /// `true`
    True,

    /// Any number literal
    Number(f64),

    /// Key of the key/value pair or string value
    String(String),
}

#[cfg(test)]
impl Token {
    pub(crate) fn string(input: &str) -> Self {
        Self::String(String::from(input))
    }
}

/// One of the possible errors that could occur while tokenizing the input
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenizeError {
    /// Character is not part of a JSON token
    CharNotRecognized(char),

    /// Unable to parse the float
    ParseNumberError(ParseFloatError),

    /// String was never completed
    UnclosedQuotes,

    /// The input appeared to be the start of a literal value but did not finish
    UnfinishedLiteralValue,

    /// The input ended early
    UnexpectedEof,
}

pub fn tokenize(input: String) -> Result<Vec<Token>, TokenizeError> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    let mut tokens = Vec::new();
    while index < chars.len() {
        let token = make_token(&chars, &mut index)?;
        tokens.push(token);
        index += 1;
    }
    Ok(tokens)
}

fn make_token(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    let mut ch = chars[*index];
    while ch.is_ascii_whitespace() {
        *index += 1;
        if *index >= chars.len() {
            return Err(TokenizeError::UnexpectedEof);
        }
        ch = chars[*index];
    }
    let token = match ch {
        '[' => Token::LeftBracket,
        ']' => Token::RightBracket,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        ',' => Token::Comma,
        ':' => Token::Colon,

        'n' => tokenize_null(chars, index)?,
        't' => tokenize_true(chars, index)?,
        'f' => tokenize_false(chars, index)?,

        c if c.is_ascii_digit() || c == '-' => tokenize_float(chars, index)?,

        '"' => tokenize_string(chars, index)?,

        ch => return Err(TokenizeError::CharNotRecognized(ch)),
    };

    Ok(token)
}

fn tokenize_null(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in "null".chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1; // index is incremented in the main loop
    Ok(Token::Null)
}

fn tokenize_true(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in "true".chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1; // index is incremented in the main loop
    Ok(Token::True)
}

fn tokenize_false(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in "false".chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1; // index is incremented in the main loop
    Ok(Token::False)
}

fn tokenize_string(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    debug_assert!(chars[*index] == '"');
    let mut string = String::new();
    let mut is_escaping = false;

    loop {
        *index += 1;
        if *index >= chars.len() {
            return Err(TokenizeError::UnclosedQuotes);
        }

        let ch = chars[*index];
        match ch {
            '"' if !is_escaping => break,
            '\\' => is_escaping = !is_escaping,
            _ => is_escaping = false,
        }

        string.push(ch);
    }

    Ok(Token::String(string))
}

fn tokenize_float(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    let mut unparsed_num = String::new();
    let mut has_decimal = false;

    while *index < chars.len() {
        let ch = chars[*index];
        match ch {
            c if c.is_ascii_digit() || c == '-' => unparsed_num.push(c),
            c if c == '.' && !has_decimal => {
                unparsed_num.push('.');
                has_decimal = true;
            }

            _ => break,
        }
        *index += 1;
    }

    // outer loop increments index
    *index -= 1;

    match unparsed_num.parse() {
        Ok(f) => Ok(Token::Number(f)),
        Err(err) => Err(TokenizeError::ParseNumberError(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token, TokenizeError};

    #[test]
    fn just_comma() {
        let input = String::from(",");
        let expected = [Token::Comma];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn all_punctuation() {
        let input = String::from("[{]},:");
        let expected = [
            Token::LeftBracket,
            Token::LeftBrace,
            Token::RightBracket,
            Token::RightBrace,
            Token::Comma,
            Token::Colon,
        ];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_ken() {
        let input = String::from("\"ken\"");
        let expected = [Token::string("ken")];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn escaped_quote() {
        let input = String::from(r#""the \" is OK""#);
        let expected = [Token::String(String::from(r#"the \" is OK"#))];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn unclosed_string() {
        let input = String::from("\"unclosed");
        let expected = Err(TokenizeError::UnclosedQuotes);

        let actual = tokenize(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn key_colon() {
        let input = String::from("\"key\":");
        let expected = [Token::string("key"), Token::Colon];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_object() {
        let input = String::from("{\"key\":\"value\"}");
        let expected = [
            Token::LeftBrace,
            Token::string("key"),
            Token::Colon,
            Token::string("value"),
            Token::RightBrace,
        ];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn array_with_numbers() {
        let input = String::from("[123.4, 567.8]");
        let expected = [
            Token::LeftBracket,
            Token::Number(123.4),
            Token::Comma,
            Token::Number(567.8),
            Token::RightBracket,
        ];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn array_with_strings() {
        let input = String::from("[\"A\", \"B\"]");
        let expected = [
            Token::LeftBracket,
            Token::string("A"),
            Token::Comma,
            Token::string("B"),
            Token::RightBracket,
        ];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn empty_string() {
        let input = String::from("[\"\"]");
        let expected = [Token::LeftBracket, Token::string(""), Token::RightBracket];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_null() {
        let input = String::from("null");
        let expected = [Token::Null];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_false() {
        let input = String::from("false");
        let expected = [Token::False];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_true() {
        let input = String::from("true");
        let expected = [Token::True];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn true_comma() {
        let input = String::from("true,");
        let expected = [Token::True, Token::Comma];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn integer() {
        let input = String::from("123");
        let expected = [Token::Number(123.0)];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn floating_point() {
        let input = String::from("1.23");
        let expected = [Token::Number(1.23)];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn negative_int() {
        let input = String::from("-123");
        let expected = [Token::Number(-123.0)];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn array_with_null() {
        let input = String::from("[null]");
        let expected = [Token::LeftBracket, Token::Null, Token::RightBracket];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn array_with_true_false() {
        let input = String::from("[true, false]");
        let expected = [
            Token::LeftBracket,
            Token::True,
            Token::Comma,
            Token::False,
            Token::RightBracket,
        ];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }
}
