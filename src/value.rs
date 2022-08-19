use std::{collections::BTreeMap, fmt, mem};

#[macro_export]
macro_rules! json_object(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = BTreeMap::new();
            $(
                m.insert($key.to_string(), $value);
            )+
            m
        }
    };
);

/// Representation of a JSON value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// literal characters `null`
    Null,

    /// literal characters `true` or `false`
    Boolean(bool),

    /// characters within double quotes "..."
    String(String),

    /// numbers stored as a 64-bit floating point
    Number(f64),

    /// Zero to many JSON values
    Array(Vec<Value>),

    /// String keys with JSON values
    Object(BTreeMap<String, Value>),
}

impl Value {
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match *self {
            Value::Array(ref array) => Some(array),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, Value>> {
        match *self {
            Value::Object(ref object) => Some(object),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    pub fn as_string(&self) -> Option<&String> {
        match *self {
            Value::String(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        self.as_string().is_some()
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match *self {
            Value::Boolean(b) => Some(b),
            _ => None,
        }
    }

    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    pub fn as_null(&self) -> Option<()> {
        match *self {
            Value::Null => Some(()),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    pub fn take(&mut self) -> Value {
        mem::replace(self, Value::Null)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => f.write_str("null"),
            Value::Array(vec) => {
                let mut output = "[".to_string();
                let mut iter = vec.iter().peekable();

                while let Some(value) = iter.next() {
                    output.push_str(&format!("{}", value));
                    if iter.peek().is_some() {
                        output.push_str(", ");
                    }
                }
                output.push(']');
                write!(f, "{}", output)
            }
            Value::Object(object) => {
                let mut output = "{".to_string();
                let mut iter = object.iter().peekable();

                while let Some(item) = iter.next() {
                    output.push('"');
                    output.push_str(item.0);
                    output.push_str("\": ");
                    if item.1.is_string() {
                        output.push('"');
                        output.push_str(&format!("{}", item.1));
                        output.push('"');
                    } else {
                        output.push_str(&format!("{}", item.1));
                    }
                    if iter.peek().is_some() {
                        output.push_str(", ");
                    }
                }
                output.push('}');
                write!(f, "{}", output)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_array() {
        let val = Value::Array(vec![Value::Null, Value::Boolean(true)]);
        let s = format!("{}", val);
        assert_eq!("[null, true]".to_string(), s);
    }

    #[test]
    fn display_object() {
        let val = Value::Object(json_object! { "key" => Value::String("value".to_string()) });
        let s = format!("{}", val);
        assert_eq!("{\"key\": \"value\"}".to_string(), s);
    }
}
