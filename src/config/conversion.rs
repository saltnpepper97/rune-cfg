// Author: Dustin Pilgrim
// License: MIT

use std::collections::HashMap;

use crate::{Value, RuneError};
use crate::ast::ObjectItem;

impl TryFrom<Value> for String {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(RuneError::TypeError {
                message: format!("Expected string, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a string value in your config".into()),
                code: Some(401),
            }),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(n),
            _ => Err(RuneError::TypeError {
                message: format!("Expected number, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a number value in your config".into()),
                code: Some(402),
            }),
        }
    }
}

impl TryFrom<Value> for f32 {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(n as f32),
            _ => Err(RuneError::TypeError {
                message: format!("Expected number, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a number value in your config".into()),
                code: Some(402),
            }),
        }
    }
}

impl TryFrom<Value> for i32 {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(n as i32),
            _ => Err(RuneError::TypeError {
                message: format!("Expected number, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a number value in your config".into()),
                code: Some(402),
            }),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(n as i64),
            _ => Err(RuneError::TypeError {
                message: format!("Expected number, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a number value in your config".into()),
                code: Some(402),
            }),
        }
    }
}

impl TryFrom<Value> for u8 {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => {
                if n >= 0.0 && n <= u8::MAX as f64 {
                    Ok(n as u8)
                } else {
                    Err(RuneError::TypeError {
                        message: format!("Number {} out of range for u8", n),
                        line: 0,
                        column: 0,
                        hint: Some("Use a number between 0 and 255".into()),
                        code: Some(407),
                    })
                }
            }
            _ => Err(RuneError::TypeError {
                message: format!("Expected number, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a number value in your config".into()),
                code: Some(402),
            }),
        }
    }
}

impl TryFrom<Value> for u16 {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => {
                if n >= 0.0 && n <= u16::MAX as f64 {
                    Ok(n as u16)
                } else {
                    Err(RuneError::TypeError {
                        message: format!("Number {} out of range for u16", n),
                        line: 0,
                        column: 0,
                        hint: Some("Use a number between 0 and 65535".into()),
                        code: Some(403),
                    })
                }
            }
            _ => Err(RuneError::TypeError {
                message: format!("Expected number, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a number value in your config".into()),
                code: Some(402),
            }),
        }
    }
}

impl TryFrom<Value> for u32 {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => {
                if n >= 0.0 && n <= u32::MAX as f64 {
                    Ok(n as u32)
                } else {
                    Err(RuneError::TypeError {
                        message: format!("Number {} out of range for u32", n),
                        line: 0,
                        column: 0,
                        hint: Some("Use a number between 0 and 4294967295".into()),
                        code: Some(408),
                    })
                }
            }
            _ => Err(RuneError::TypeError {
                message: format!("Expected number, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a number value in your config".into()),
                code: Some(402),
            }),
        }
    }
}

impl TryFrom<Value> for u64 {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => {
                if n >= 0.0 && n <= u64::MAX as f64 {
                    Ok(n as u64)
                } else {
                    Err(RuneError::TypeError {
                        message: format!("Number {} out of range for u64", n),
                        line: 0,
                        column: 0,
                        hint: Some("Use a positive number within u64 range".into()),
                        code: Some(406),
                    })
                }
            }
            _ => Err(RuneError::TypeError {
                message: format!("Expected number, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a number value in your config".into()),
                code: Some(402),
            }),
        }
    }
}

impl TryFrom<Value> for usize {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => {
                if n >= 0.0 && n.is_finite() {
                    Ok(n as usize)
                } else {
                    Err(RuneError::TypeError {
                        message: format!("Number {} out of range for usize", n),
                        line: 0,
                        column: 0,
                        hint: Some("Use a positive integer".into()),
                        code: Some(409),
                    })
                }
            }
            _ => Err(RuneError::TypeError {
                message: format!("Expected number, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use a number value in your config".into()),
                code: Some(402),
            }),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            Value::Reference(ref path) if path.len() == 1 => {
                let ref_name = &path[0];
                if ref_name.to_lowercase().starts_with("tru") || ref_name.to_lowercase().starts_with("fal") {
                    Err(RuneError::TypeError {
                        message: format!("Invalid boolean value '{}'. Did you mean 'true' or 'false'?", ref_name),
                        line: 0,
                        column: 0,
                        hint: None,
                        code: Some(404),
                    })
                } else {
                    Err(RuneError::TypeError {
                        message: format!("Expected boolean (true/false), got reference to '{}'", ref_name),
                        line: 0,
                        column: 0,
                        hint: None,
                        code: Some(404),
                    })
                }
            }
            _ => Err(RuneError::TypeError {
                message: format!("Expected boolean, got {:?}", value),
                line: 0,
                column: 0,
                hint: None,
                code: Some(404),
            }),
        }
    }
}

impl<T> TryFrom<Value> for Vec<T>
where
    T: TryFrom<Value, Error = RuneError>,
{
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    result.push(T::try_from(item)?);
                }
                Ok(result)
            }
            _ => Err(RuneError::TypeError {
                message: format!("Expected array, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use an array [...] in your config".into()),
                code: Some(405),
            }),
        }
    }
}

impl<T> TryFrom<Value> for Option<T>
where
    T: TryFrom<Value, Error = RuneError>,
{
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Null => Ok(None),
            v => Ok(Some(T::try_from(v)?)),
        }
    }
}

/// Convert an object `Value` to a map, **expecting the object to be fully-resolved**.
///
/// With block `if ... endif` support, objects can contain conditional items. By the time you
/// convert to a `HashMap`, those conditionals should have been evaluated and flattened into
/// plain assignments.
///
/// If this conversion sees an `IfBlock`, it returns a type error with a helpful hint.
fn object_items_to_map(items: Vec<ObjectItem>) -> Result<HashMap<String, Value>, RuneError> {
    let mut map = HashMap::new();

    for item in items {
        match item {
            ObjectItem::Assign(key, val) => {
                map.insert(key, val);
            }
            ObjectItem::IfBlock(_) => {
                return Err(RuneError::TypeError {
                    message: "Expected object with only key/value pairs, but found an if-block".into(),
                    line: 0,
                    column: 0,
                    hint: Some("This usually means if-blocks were not resolved. Ensure you resolve/evaluate the config before converting it to a HashMap.".into()),
                    code: Some(410),
                });
            }
        }
    }

    Ok(map)
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(items) => object_items_to_map(items),
            _ => Err(RuneError::TypeError {
                message: format!("Expected object, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use an object block in your config".into()),
                code: Some(410),
            }),
        }
    }
}

impl TryFrom<Value> for HashMap<String, String> {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(items) => {
                let base = object_items_to_map(items)?;
                let mut map = HashMap::new();
                for (key, val) in base {
                    let string_val = String::try_from(val)?;
                    map.insert(key, string_val);
                }
                Ok(map)
            }
            _ => Err(RuneError::TypeError {
                message: format!("Expected object, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use an object block with string values".into()),
                code: Some(410),
            }),
        }
    }
}

impl TryFrom<Value> for (String, String) {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(arr) if arr.len() == 2 => {
                let first = String::try_from(arr[0].clone())?;
                let second = String::try_from(arr[1].clone())?;
                Ok((first, second))
            }
            _ => Err(RuneError::TypeError {
                message: "Expected array with exactly 2 string elements".into(),
                line: 0,
                column: 0,
                hint: Some("Use [\"key\", \"value\"] format".into()),
                code: Some(411),
            }),
        }
    }
}

impl TryFrom<Value> for (String, Value) {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(arr) if arr.len() == 2 => {
                let key = String::try_from(arr[0].clone())?;
                let val = arr[1].clone();
                Ok((key, val))
            }
            _ => Err(RuneError::TypeError {
                message: "Expected array with exactly 2 elements (key and value)".into(),
                line: 0,
                column: 0,
                hint: Some("Use [\"key\", value] format".into()),
                code: Some(411),
            }),
        }
    }
}

impl RuneError {
    /// Helper for file-related errors when loading/parsing configs.
    ///
    /// Keeps a consistent error code and a friendly default hint.
    pub fn file_error(message: String, path: String) -> Self {
        RuneError::FileError {
            message,
            path,
            hint: Some("Check file path and permissions".into()),
            code: Some(300),
        }
    }
}
