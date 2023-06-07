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

    use crate::{json_object, value::Value};
    use std::collections::HashMap;

    fn test_cases_success() -> Vec<(&'static str, Value)> {
        vec![
            ("null", Value::Null),
            ("true", Value::Boolean(true)),
            ("false", Value::Boolean(false)),
            ("[null]", Value::Array(vec![Value::Null])),
            (
                "[true,false]",
                Value::Array(vec![Value::Boolean(true), Value::Boolean(false)]),
            ),
            (
                "[1.1, 2.2, 3]",
                Value::Array(vec![
                    Value::Number(1.1),
                    Value::Number(2.2),
                    Value::Number(3.0),
                ]),
            ),
            (
                "[{\"key\": null}]",
                Value::Array(vec![json_object! { "key" => Value::Null }]),
            ),
            ("{\"key\": 1}", json_object! { "key" => Value::Number(1.0) }),
            (
                "{\"key\": \"value\"}",
                json_object! { "key" => Value::String("value".to_string()) },
            ),
            ("{\"key\": null}", json_object! { "key" => Value::Null }),
            (
                "{\"key\": true}",
                json_object! { "key" => Value::Boolean(true) },
            ),
            (
                "{\"key\": false}",
                json_object! { "key" => Value::Boolean(false) },
            ),
            (
                "{\"key\": { \"innerkey\": null } }",
                json_object! { "key" => json_object! { "innerkey" => Value::Null}},
            ),
        ]
    }

    fn full_input() -> &'static str {
        r#"
            {
                "str_val": "value",
                "null_val": null,
                "true_val": true,
                "false_val": false,
                "num_val": 5,
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
        json_object!(
            "str_val" => Value::String("value".to_string()),
            "null_val" => Value::Null,
            "true_val" => Value::Boolean(true),
            "false_val" => Value::Boolean(false),
            "num_val" => Value::Number(5.0),
            "arr_val" => Value::Array(vec! [Value::String("one".to_string()), Value::Number(2.0), Value::Boolean(false)]),
            "obj_val" => json_object!(
                "nested_key" => Value::String("nested_value".to_string())
            )
        )
    }

    #[test]
    fn test_unit_cases_combinators() {
        for (input, expected) in test_cases_success() {
            let actual = combinator_parse(input).unwrap();
            assert_eq!(expected, actual)
        }
    }

    #[test]
    fn test_full_input_combinators() {
        let input = full_input();

        let expected = expected_value();

        let actual = combinator_parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_unit_cases_iterator() {
        for (input, expected) in test_cases_success() {
            let actual = iterator_parse(input).unwrap();
            assert_eq!(expected, actual)
        }
    }

    #[test]
    fn test_full_input_iterator() {
        let input = full_input();
        let expected = expected_value();
        let actual = iterator_parse(input).unwrap();
        assert_eq!(expected, actual);
    }
}
