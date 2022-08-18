use super::{
    common::{left, one_or_more, pred, right, zero_or_more},
    ParseResult, Parser,
};

/// Parses the content contained between two literal quotation characters
///
/// ```text
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

/// Parses a positive integer into a u64 integer
///
/// TODO: Remove unwrap, return the ParseIntError instead
pub(crate) fn uint<'a>() -> impl Parser<'a, u64> {
    one_or_more(pred(any_char, |c| c.is_numeric())).map(|chars| {
        chars
            .into_iter()
            .collect::<String>()
            .parse::<u64>()
            .unwrap()
    })
}

/// Parses a negative number into an i64 integer
///
/// TODO: Remove the possibility of panic, instead return
/// Err if outside the bounds of i64::MIN and i64::MAX
pub(crate) fn int<'a>() -> impl Parser<'a, i64> {
    match_literal("-").and_then(|()| {
        one_or_more(pred(any_char, |c| c.is_numeric())).map(|chars| {
            chars
                .into_iter()
                .collect::<String>()
                .parse::<i64>()
                .map(|i| -i)
                .unwrap()
        })
    })
}

/// Parses a whitespace character
///
/// ```text
/// \n
/// ```
///
/// Captures a char with the whitespace character
pub(crate) fn whitespace_char<'a>() -> impl Parser<'a, char> {
    pred(any_char, |c| c.is_whitespace())
}

/// Parses for one to many whitespace characters
///
/// ```text
/// \t \n
/// ```
///
/// Captures a Vec<char> with the whitespace characters
#[allow(dead_code)]
pub(crate) fn space1<'a>() -> impl Parser<'a, Vec<char>> {
    one_or_more(whitespace_char())
}

/// Parses for zero to many whitespace characters
///
/// ```text
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
/// ```text
///    abc\t\n
/// ```
///
/// Returns the output of the provided parser.
pub(crate) fn trim<'a, P, R: 'a + Clone>(parser: P) -> impl Parser<'a, R>
where
    P: Parser<'a, R> + 'a,
{
    right(space0(), left(parser, space0()))
}

/// Parses any character
///
/// ```text
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
/// ```text
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
    fn uint_parser() {
        let input = "123";
        let expected = Ok(("", 123_u64));
        let actual = uint().parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn int_parser() {
        let input = "-123";
        let expected = Ok(("", -123_i64));
        let actual = int().parse(input);
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
