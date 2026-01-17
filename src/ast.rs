use regex::Regex;

#[derive(Debug, Clone)]
pub enum Condition {
    Equals(String, Value),
    NotEquals(String, Value),
    Exists(String),
    NotExists(String),
}

impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Condition::Equals(p1, v1), Condition::Equals(p2, v2)) => p1 == p2 && v1 == v2,
            (Condition::NotEquals(p1, v1), Condition::NotEquals(p2, v2)) => p1 == p2 && v1 == v2,
            (Condition::Exists(p1), Condition::Exists(p2)) => p1 == p2,
            (Condition::NotExists(p1), Condition::NotExists(p2)) => p1 == p2,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalValue {
    pub condition: Condition,
    pub then_value: Value,
    pub else_value: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Regex(Regex),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Reference(Vec<String>),
    Interpolated(Vec<Value>),
    Conditional(Box<ConditionalValue>),
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Reference(a), Value::Reference(b)) => a == b,
            (Value::Interpolated(a), Value::Interpolated(b)) => a == b,
            (Value::Conditional(a), Value::Conditional(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl Value {
    pub fn as_object(&self) -> Option<&Vec<(String, Value)>> {
        if let Value::Object(items) = self {
            Some(items)
        } else {
            None
        }
    }

    pub fn as_regex(&self) -> Option<&Regex> {
        if let Value::Regex(r) = self {
            Some(r)
        } else {
            None
        }
    }
    
    pub fn matches(&self, text: &str) -> bool {
        match self {
            Value::Regex(r) => r.is_match(text),
            Value::String(s) => s == text,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub items: Vec<(String, Value)>,
    pub metadata: Vec<(String, Value)>,
    pub globals: Vec<(String, Value)>,
}
