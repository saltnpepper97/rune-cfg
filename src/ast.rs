#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Reference(Vec<String>), // e.g. defaults.server.host
    Interpolated(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub items: Vec<(String, Value)>, // top-level assignments/blocks
    pub metadata: Vec<(String, Value)>, // @tags
    pub globals: Vec<(String, Value)>,  // $globals
}


impl Value {
    pub fn as_object(&self) -> Option<&Vec<(String, Value)>> {
        if let Value::Object(items) = self {
            Some(items)
        } else {
            None
        }
    }
}
