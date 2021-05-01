use super::Parser;

/// Run two parsers in series to produce a combined result of their outputs
pub(crate) fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input| {
        parser1.parse(input).and_then(|(next_input, result1)| {
            parser2
                .parse(next_input)
                .map(|(last_input, result2)| (last_input, (result1, result2)))
        })
    }
}

/// Changes the type of the result using a mapping function
///
/// Considered a "functor" in other languages
pub(crate) fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    F: Fn(A) -> B,
{
    move |input| {
        parser
            .parse(input)
            .map(|(next_input, result)| (next_input, map_fn(result)))
    }
}

/// Runs two parses in sequence, passing the result of the first
/// parser into the second.
pub(crate) fn and_then<'a, P, F, A, B, NextP>(parser: P, f: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    NextP: Parser<'a, B>,
    F: Fn(A) -> NextP,
{
    move |input| match parser.parse(input) {
        Ok((next_input, result)) => f(result).parse(next_input),
        Err(err) => Err(err),
    }
}

/// Combines two parsers and takes the left element in the result
pub(crate) fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
where
    P1: Parser<'a, R1> + 'a,
    P2: Parser<'a, R2> + 'a,
    R1: Clone + 'a,
    R2: 'a,
{
    map(pair(parser1, parser2), |(left, _right)| left)
}

/// Combines two parsers and takes the right element in the result
pub(crate) fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
where
    P1: Parser<'a, R1> + 'a,
    P2: Parser<'a, R2> + 'a,
    R1: Clone + 'a,
    R2: 'a,
{
    map(pair(parser1, parser2), |(_left, right)| right)
}

/// Parses zero or one match of the provided parser
pub(crate) fn zero_or_one<'a, P, Output>(parser: P) -> impl Parser<'a, Vec<Output>>
where
    P: Parser<'a, Output>,
{
    move |mut input| {
        let mut result = Vec::new();

        if let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }

        Ok((input, result))
    }
}

/// Parses zero or more matches of the provided parser
pub(crate) fn zero_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>,
{
    move |mut input| {
        let mut result = Vec::new();

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }

        Ok((input, result))
    }
}

/// Parses where there is at least one match for the provided parser
pub(crate) fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>,
{
    move |mut input| {
        let mut result = Vec::new();

        if let Ok((next_input, first_item)) = parser.parse(input) {
            input = next_input;
            result.push(first_item);
        } else {
            return Err(input);
        }

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }

        Ok((input, result))
    }
}

/// Runs the provided parser and returns Ok if the predicate is true
/// and Err if the predicate is false
pub(crate) fn pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parser<'a, A>
where
    P: Parser<'a, A>,
    F: Fn(&A) -> bool,
{
    move |input| {
        if let Ok((next_input, value)) = parser.parse(input) {
            if predicate(&value) {
                return Ok((next_input, value));
            }
        }
        Err(input)
    }
}

/// Tries the two parsers, in order, only using the second parser provided
/// the first parser fails
pub(crate) fn either<'a, P1, P2, A>(parser1: P1, parser2: P2) -> impl Parser<'a, A>
where
    P1: Parser<'a, A>,
    P2: Parser<'a, A>,
{
    move |input| match parser1.parse(input) {
        ok @ Ok(_) => ok,
        Err(_) => parser2.parse(input),
    }
}

#[cfg(test)]
mod tests {

    use crate::combinator_parser::lexers::{any_char, match_literal, quoted_string};

    use super::*;

    #[test]
    fn left_combinator() {
        let test_parser = left(quoted_string(), match_literal("{"));
        assert_eq!(
            Ok(("}", "test".to_string())),
            test_parser.parse("\"test\"{}")
        );
        assert_eq!(Err("bad"), test_parser.parse("bad"));
        assert_eq!(Err("}"), test_parser.parse("\"bad\"}"));
    }

    #[test]
    fn right_combinator() {
        let test_parser = right(match_literal("{"), quoted_string());
        assert_eq!(
            Ok(("}", "test".to_string())),
            test_parser.parse("{\"test\"}")
        );
        assert_eq!(Err("bad"), test_parser.parse("bad"));
        assert_eq!(Err("!bad"), test_parser.parse("{!bad"));
    }

    #[test]
    fn zero_or_one_combinator() {
        let parser = zero_or_one(match_literal("yeet"));
        assert_eq!(Ok(("", vec![])), parser.parse(""));
        assert_eq!(Ok(("teey", vec![])), parser.parse("teey"));
        assert_eq!(Ok(("", vec![()])), parser.parse("yeet"));
        assert_eq!(Ok(("yeet", vec![()])), parser.parse("yeetyeet"));
    }

    #[test]
    fn zero_or_more_combinator() {
        let parser = zero_or_more(match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Ok(("ahah", vec![])), parser.parse("ahah"));
        assert_eq!(Ok(("", vec![])), parser.parse(""));
    }

    #[test]
    fn one_or_more_combinator() {
        let parser = one_or_more(match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Err("ahah"), parser.parse("ahah"));
        assert_eq!(Err(""), parser.parse(""));
    }

    #[test]
    fn predicate_combinator() {
        let parser = pred(any_char, |c| *c == 'o');
        assert_eq!(Ok(("mg", 'o')), parser.parse("omg"));
        assert_eq!(Err("lol"), parser.parse("lol"));
    }
}
