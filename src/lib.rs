use value::Value;

mod combinator_parser;
mod iterator_parser;
mod location;

pub mod value;

pub fn iterator_parse(input: &str) -> iterator_parser::ParseResult {
    iterator_parser::parse(input)
}

pub fn combinator_parse(input: &str) -> Result<Value, &str> {
    combinator_parser::parse(input)
}

/// Integration tests
#[cfg(test)]
mod tests {

    use super::{combinator_parse, iterator_parse};

    use crate::{
        json_object,
        value::{NumberValue, Value},
    };
    use std::collections::BTreeMap;

    fn full_input() -> &'static str {
        r#"
            {
                "str_val": "value",
                "null_val": null,
                "true_val": true,
                "false_val": false,
                "uint_val": 5,
                "int_val": -6,
                "arr_val": [
                    "one",
                    2,
                    false
                ],
                "obj_val": {
                    "nested_key": "nested_value"
                }
            }
        "#
    }

    fn expected_value() -> Value {
        Value::Object(json_object!(
            "str_val".to_string() => Value::String("value".to_string()),
            "null_val".to_string() => Value::Null,
            "true_val".to_string() => Value::Bool(true),
            "false_val".to_string() => Value::Bool(false),
            "uint_val".to_string() => Value::Number(NumberValue::UInt(5)),
            "int_val".to_string() => Value::Number(NumberValue::Int(-6)),
            "arr_val".to_string() => Value::Array(vec! [Value::String("one".to_string()), Value::Number(NumberValue::UInt(2)), Value::Bool(false)]),
            "obj_val".to_string() => Value::Object(json_object!(
                "nested_key".to_string() => Value::String("nested_value".to_string())
            ))
        ))
    }

    #[test]
    fn parse_with_combinators() {
        let input = full_input();

        let expected = expected_value();

        let actual = combinator_parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_with_iterators() {
        let input = full_input();
        let expected = expected_value();
        let actual = iterator_parse(input).unwrap();
        assert_eq!(expected, actual);
    }
}
