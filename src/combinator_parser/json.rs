use std::collections::BTreeMap;

use crate::value::Value;

use super::{
    common::{either, left, one_or_more, pair, pred, right, zero_or_more, zero_or_one},
    lexers::{any_char, match_literal, quoted_string},
    Parser,
};

/// Parses the literal characters "true"
///
/// Produces a JSON boolean true value
fn true_value<'a>() -> impl Parser<'a, Value> {
    match_literal("true").map(|_| Value::Bool(true))
}

/// Parses the literal characters "false"
///
/// Produces a JSON boolean false value
fn false_value<'a>() -> impl Parser<'a, Value> {
    match_literal("false").map(|_| Value::Bool(false))
}

/// Parses a boolean value
///
/// Either `true` or `false`
fn bool_value<'a>() -> impl Parser<'a, Value> {
    either(true_value(), false_value())
}

/// Parses the literal characters "null"
///
/// Produces a JSON null value
fn null_value<'a>() -> impl Parser<'a, Value> {
    match_literal("null").map(|_| Value::Null)
}

/// Parses the characters between quotes
///
/// Produces a JSON string value
fn string_value<'a>() -> impl Parser<'a, Value> {
    quoted_string().map(|s| Value::String(s))
}

/// Parses a number value
///
/// Produces a JSON number value
/// TODO: Implement full number parsing and remove possibility of panic
fn number_value<'a>() -> impl Parser<'a, Value> {
    one_or_more(pred(any_char, |c| c.is_numeric()))
        .map(|chars| {
            chars
                .into_iter()
                .collect::<String>()
                .parse::<i64>()
                .unwrap()
        })
        .map(|int| Value::Number(int))
}

/// Parses a primitive value as defined by JS primitives
///
/// Values in JSON that correspond to primitives in JS include:
/// - null
/// - boolean
/// - string
/// - number
///
/// Not included are objects and arrays
fn primitive_value<'a>() -> impl Parser<'a, Value> {
    either(
        bool_value(),
        either(null_value(), either(string_value(), number_value())),
    )
}

/// Parses an object key/value pair
///
/// ```
/// "my_key":null
/// ```
///
/// Captures the key as a String, and the value as a JSON Value
fn object_pair<'a>() -> impl Parser<'a, (String, Value)> {
    pair(quoted_string(), right(match_literal(":"), null_value()))
}

/// Parses an object
///
/// ```json
/// {
///   "key1": true,
///   "key2": null
/// }
/// ```
///
/// Captures the object as a Value::Object variant
fn object_value<'a>() -> impl Parser<'a, Value> {
    match_literal("{").and_then(|_| {
        left(zero_or_more(object_pair()), match_literal("}")).map(|v| {
            let mut map: BTreeMap<String, Value> = BTreeMap::new();
            for (key, val) in v {
                map.insert(key, val);
            }
            Value::Object(map)
        })
    })
}

/// Parses a JSON value that is preceded by a comma
///
/// ex.
/// ```
/// ,null
/// ```
///
/// JSON doesn't allow trailing commas on the last value
/// of an array, so this is used to parse all values of
/// an array except the first.
fn comma_preceded_value<'a>() -> impl Parser<'a, Value> {
    right(match_literal(","), json_value())
}

/// Parses an array
///
/// ```json
/// [
///    true,
///    false,
///    null
/// ]
/// ```
///
/// An array value is:
/// - "[" character
/// - and_then zero or one json values
/// - and_then zero to many ("," character and then json value)
/// - finished with "]" character
fn array_value<'a>() -> impl Parser<'a, Value> {
    match_literal("[").and_then(|_| {
        left(
            pair(
                zero_or_one(json_value()),
                zero_or_more(comma_preceded_value()),
            ),
            match_literal("]"),
        )
        .map(|(mut v1, mut v2)| {
            v1.append(&mut v2);
            Value::Array(v1)
        })
    })
}

/// Parses non-primitive values
///
/// Values in JSON that correspond to non-primitive values
/// in JS include:
/// - object
/// - array
fn nonprimitive_value<'a>() -> impl Parser<'a, Value> {
    either(object_value(), array_value())
}

pub(crate) fn json_value<'a>() -> impl Parser<'a, Value> {
    either(object_value(), array_value())
}

#[cfg(test)]
mod tests {
    use crate::json_object;

    use super::*;

    #[test]
    fn true_parser() {
        let expected = Ok(("", Value::Bool(true)));
        let actual = true_value().parse("true");
        assert_eq!(expected, actual);
    }

    #[test]
    fn false_parser() {
        let expected = Ok(("", Value::Bool(false)));
        let actual = false_value().parse("false");
        assert_eq!(expected, actual);
    }

    #[test]
    fn null_parser() {
        let expected = Ok(("", Value::Null));
        let actual = null_value().parse("null");
        assert_eq!(expected, actual);
    }

    #[test]
    fn string_parser() {
        let expected = Ok(("", Value::String("key".to_string())));
        let actual = string_value().parse("\"key\"");
        assert_eq!(expected, actual)
    }

    #[test]
    fn int_parser() {
        let expected = Ok(("", Value::Number(1)));
        let actual = number_value().parse("1");
        assert_eq!(expected, actual);
    }

    #[test]
    fn object_pair_parser() {
        let expected = Ok(("", ("key".to_string(), Value::Null)));
        let actual = object_pair().parse("\"key\":null");
        assert_eq!(expected, actual);
    }

    #[test]
    fn object_empty_parser() {
        let expected = Ok(("", Value::Object(BTreeMap::new())));
        let actual = object_value().parse("{}");
        assert_eq!(expected, actual);
    }

    #[test]
    fn object_one_parser() {
        let expected = Ok((
            "",
            Value::Object(json_object! { "key".to_string() => Value::Null }),
        ));
        let actual = object_value().parse("{\"key\":null}");
        assert_eq!(expected, actual);
    }

    #[test]
    fn object_many_parser() {
        let expected = Ok((
            "",
            Value::Object(
                json_object! { "a".to_string() => Value::Bool(true), "b".to_string() => Value::Bool(false) },
            ),
        ));
        let actual = object_value().parse("{\"a\":true,\"b\":false}");
        assert_eq!(expected, actual);
    }

    #[test]
    fn array_empty_parser() {
        let expected = Ok(("", Value::Array(vec![])));
        let actual = array_value().parse("[]");
        assert_eq!(expected, actual);
    }

    #[test]
    fn array_one_parser() {
        let expected = Ok(("", Value::Array(vec![Value::Null])));
        let actual = array_value().parse("[null]");
        assert_eq!(expected, actual);
    }

    #[test]
    fn array_many_parser() {
        let expected = Ok((
            "",
            Value::Array(vec![Value::Null, Value::Bool(true), Value::Bool(false)]),
        ));
        let actual = array_value().parse("[null,true,false]");
        assert_eq!(expected, actual);
    }
}
