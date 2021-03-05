
#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Number(i64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Null,
}
