use super::{
    common::{left, one_or_more, pred, right, zero_or_more},
    ParseResult, Parser,
};

/// Parses the content contained between two literal quotation characters
///
/// ```
/// "quote"
/// ```
///
/// Captures a String instance with that content.
pub(crate) fn quoted_string<'a>() -> impl Parser<'a, String> {
    right(
        match_literal("\""),
        left(
            zero_or_more(any_char.pred(|c| *c != '"')),
            match_literal("\""),
        ),
    )
    .map(|chars| chars.into_iter().collect())
}

/// Parses a whitespace character
///
/// ```
/// \n
/// ```
///
/// Captures a char with the whitespace character
pub(crate) fn whitespace_char<'a>() -> impl Parser<'a, char> {
    pred(any_char, |c| c.is_whitespace())
}

/// Parses for one to many whitespace characters
///
/// ```
/// \t \n
/// ```
///
/// Captures a Vec<char> with the whitespace characters
pub(crate) fn space1<'a>() -> impl Parser<'a, Vec<char>> {
    one_or_more(whitespace_char())
}

/// Parses for zero to many whitespace characters
///
/// ```
/// \t \n
/// ```
///
/// Captures a Vec<char> with the whitespace characters
pub(crate) fn space0<'a>() -> impl Parser<'a, Vec<char>> {
    zero_or_more(whitespace_char())
}

/// Parses using the provided parser, while ignoring whitespace
/// on both sides of the current input.
///
/// ```
///    abc\t\n
/// ```
///
/// Captures unit type (i.e. nothing), given that `trim` was used to
/// wrap `match_literal("abc") for this example.
pub(crate) fn trim<'a, P, A: 'a + Clone>(parser: P) -> impl Parser<'a, A>
where
    P: Parser<'a, A> + 'a,
{
    right(space0(), left(parser, space0()))
}

/// Parses any character
///
/// ```
/// a
/// ```
///
/// Captures char "a"
pub(crate) fn any_char(input: &str) -> ParseResult<char> {
    match input.chars().next() {
        Some(next) => Ok((&input[next.len_utf8()..], next)),
        _ => Err(input),
    }
}

/// Matches the input str to a literal str
///
/// ```
/// abc
/// ```
///
/// Captures unit type (i.e. nothing) if `match_literal("abc")` is used
pub(crate) fn match_literal<'a>(expected: &'static str) -> impl Parser<'a, ()> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok((&input[expected.len()..], ())),
        _ => Err(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_string_parser() {
        let input = "\"Hello!\"";
        let expected = Ok(("", "Hello!".to_string()));
        let actual = quoted_string().parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn zero_to_many_spaces_parser() {
        assert_eq!(Ok(("", vec![])), space0().parse(""));
        assert_eq!(Ok(("", vec![' ', ' '])), space0().parse("  "));
    }

    #[test]
    fn one_to_many_spaces_parser() {
        assert_eq!(Err(""), space1().parse(""));
        assert_eq!(Ok(("", vec![' ', ' '])), space1().parse("  "));
    }

    #[test]
    fn trim_parser() {
        let input = "    $  ";
        let expected = Ok(("", ()));
        let actual = trim(match_literal("$")).parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn any_char_parser() {
        let input = "$";
        let expected = Ok(("", '$'));
        let actual = any_char(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn literal_parser_error() {
        let input = "$$$";
        let expected = Err(input);
        assert_eq!(expected, match_literal("[").parse(input))
    }

    #[test]
    fn literal_parser() {
        let parse_joe = match_literal("Hello Joe!");
        assert_eq!(Ok(("", ())), parse_joe.parse("Hello Joe!"));
        assert_eq!(
            Ok((" Hello Robert!", ())),
            parse_joe.parse("Hello Joe! Hello Robert!")
        );
        assert_eq!(Err("Hello Mike!"), parse_joe.parse("Hello Mike!"));
    }
}
