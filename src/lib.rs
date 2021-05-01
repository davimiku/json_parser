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

    use super::combinator_parse;

    use crate::{json_object, value::Value};
    use std::collections::BTreeMap;

    #[test]
    fn parse_full_json() {
        let input = r#"
            {
                "str_val": "value",
                "null_val": null,
                "true_val": true,
                "false_val": false,
                "int_val": 5,
                "arr_val": [
                    "one",
                    2,
                    false
                ],
                "obj_val": {
                    "nested_key": "nested_value"
                }
            }
        "#;

        let expected = Value::Object(json_object!(
            "str_val".to_string() => Value::String("value".to_string()),
            "null_val".to_string() => Value::Null,
            "true_val".to_string() => Value::Bool(true),
            "false_val".to_string() => Value::Bool(false),
            "int_val".to_string() => Value::Number(5),
            "arr_val".to_string() => Value::Array(vec! [Value::String("one".to_string()), Value::Number(2), Value::Bool(false)]),
            "obj_val".to_string() => Value::Object(json_object!(
                "nested_key".to_string() => Value::String("nested_value".to_string())
            ))
        ));

        let actual = combinator_parse(input).unwrap();
        assert_eq!(expected, actual);
    }
}
