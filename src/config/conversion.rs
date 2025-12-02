use crate::{Value, RuneError};

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
            })
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
            })
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
            })
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
            })
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

impl TryFrom<Value> for bool {
    type Error = RuneError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            Value::Reference(ref path) if path.len() == 1 => {
                let ref_name = &path[0];
                if ref_name.to_lowercase().starts_with("tru") 
                    || ref_name.to_lowercase().starts_with("fal") {
                    Err(RuneError::TypeError {
                        message: format!(
                            "Invalid boolean value '{}'. Did you mean 'true' or 'false'?",
                            ref_name
                        ),
                        line: 0,
                        column: 0,
                        hint: None,
                        code: Some(404),
                    })
                } else {
                    Err(RuneError::TypeError {
                        message: format!(
                            "Expected boolean (true/false), got reference to '{}'",
                            ref_name
                        ),
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
            })
        }
    }
}

impl<T> TryFrom<Value> for Vec<T> 
where
    T: TryFrom<Value, Error = RuneError>
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
            })
        }
    }
}

impl RuneError {
    pub fn file_error(message: String, path: String) -> Self {
        RuneError::FileError {
            message,
            path,
            hint: Some("Check file path and permissions".into()),
            code: Some(300),
        }
    }
}
