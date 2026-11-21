// Author: Dustin Pilgrim
// License: GPL-3.0-or-later

use regex::Regex;

/// The test in an `if`, inline or block.
///
/// Each path is dotted (`app.debug`), and may start with `env.`, `sys.`,
/// `runtime.` or `var.`. A bare name means a top-level key, exactly as in a
/// value reference.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// `if path`: true unless the value is `false`, `null`, or missing.
    Truthy(String),
    /// `if not condition`.
    Not(Box<Condition>),
    /// `if path = value`.
    Equals(String, Value),
    /// `if path != value`.
    NotEquals(String, Value),
    /// `if path in [a, b, ...]`.
    In(String, Vec<Value>),
    /// True when the path is defined, whatever its value. No syntax produces
    /// it; it is kept for code that builds conditions directly.
    Exists(String),
    /// True when the path is not defined.
    NotExists(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalValue {
    pub condition: Condition,
    pub then_value: Value,
    pub else_value: Option<Value>,
}

/// A primary value decorated with line-scoped named attributes.
///
/// Produced by syntax such as `font "Inter" with size 14 weight 600`.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotatedValue {
    pub value: Box<Value>,
    pub attributes: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectItem {
    Assign(String, Value),
    IfBlock(Box<IfBlock>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfBlock {
    pub condition: Condition,
    pub then_items: Vec<ObjectItem>,
    pub else_items: Option<Vec<ObjectItem>>,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Regex(Regex),
    Array(Vec<Value>),

    /// Object blocks now contain items (assignments + if-blocks)
    Object(Vec<ObjectItem>),

    Reference(Vec<String>),
    Interpolated(Vec<Value>),

    /// Inline/value conditional: `x = if cond a else b`
    Conditional(Box<ConditionalValue>),

    /// A primary value followed by inline `with name value` attributes.
    Annotated(Box<AnnotatedValue>),

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
            (Value::Annotated(a), Value::Annotated(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl Value {
    pub fn as_object(&self) -> Option<&Vec<ObjectItem>> {
        if let Value::Object(items) = self.primary() {
            Some(items)
        } else {
            None
        }
    }

    pub fn as_regex(&self) -> Option<&Regex> {
        if let Value::Regex(r) = self.primary() {
            Some(r)
        } else {
            None
        }
    }

    pub fn as_annotated(&self) -> Option<&AnnotatedValue> {
        if let Value::Annotated(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Return the primary value, whether this value is annotated or plain.
    pub fn primary(&self) -> &Value {
        match self {
            Value::Annotated(value) => &value.value,
            value => value,
        }
    }

    /// Consume this value and return its primary value, discarding attributes.
    pub fn into_primary(self) -> Value {
        match self {
            Value::Annotated(value) => *value.value,
            value => value,
        }
    }

    pub fn matches(&self, text: &str) -> bool {
        match self.primary() {
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
