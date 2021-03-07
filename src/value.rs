use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Number(i64),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Array(v) => write!(f, "{:?}", v),
            Value::Null => f.write_str("null"),
        }
    }
}
