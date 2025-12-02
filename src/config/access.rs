use super::*;

impl RuneConfig {
    /// Get a value from the configuration using dot notation
    pub fn get<T>(&self, path: &str) -> Result<T, RuneError> 
    where
        T: TryFrom<Value, Error = RuneError>
    {
        let value = self.get_value(path)?;
        T::try_from(value).map_err(|e| {
            enhance_error_with_line_info(e, path, &self.raw_content)
        })
    }

    /// Get an optional value - returns None if key doesn't exist, Error if exists but invalid
    pub fn get_optional<T>(&self, path: &str) -> Result<Option<T>, RuneError> 
    where
        T: TryFrom<Value, Error = RuneError>
    {
        match self.get_value(path) {
            Ok(value) => Ok(Some(T::try_from(value)?)),
            Err(e) => {
                if let RuneError::SyntaxError { code: Some(304), .. } = e {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Get an optional value with a default
    pub fn get_or<T>(&self, path: &str, default: T) -> T 
    where
        T: TryFrom<Value, Error = RuneError>
    {
        self.get(path).unwrap_or(default)
    }

    /// Get a raw Value from the configuration
    pub fn get_value(&self, path: &str) -> Result<Value, RuneError> {
        let path_segments: Vec<String> = path.split('.').map(|s| s.to_string()).collect();
        
        if let Some(main_doc) = self.documents.get(&self.main_doc_key) {
            let mut temp_parser = parser::Parser::new("").map_err(|_| RuneError::SyntaxError {
                message: "Failed to create temporary parser".into(),
                line: 0,
                column: 0,
                hint: None,
                code: Some(303),
            })?;
            
            // Inject all documents as imports
            for (alias, doc) in &self.documents {
                if alias != &self.main_doc_key {
                    temp_parser.inject_import(alias.clone(), doc.clone());
                }
            }
            
            let resolved = temp_parser.resolve_reference(&path_segments, main_doc)
                .ok_or_else(|| {
                    let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
                    if line > 0 {
                        RuneError::SyntaxError {
                            message: format!("Path '{}' found but could not be resolved on line {}", path, line),
                            line,
                            column: 0,
                            hint: Some(format!("Check the value at: {}", snippet)),
                            code: Some(304),
                        }
                    } else {
                        RuneError::SyntaxError {
                            message: format!("Path '{}' not found in configuration", path),
                            line: 0,
                            column: 0,
                            hint: Some("Check that the path exists in your config file".into()),
                            code: Some(304),
                        }
                    }
                })?;

            helpers::resolve_value_recursively(resolved, &temp_parser, main_doc)
        } else {
            Err(RuneError::SyntaxError {
                message: "No main document loaded".into(),
                line: 0,
                column: 0,
                hint: None,
                code: Some(305),
            })
        }
    }

    /// Get all keys at a given path level
    pub fn get_keys(&self, path: &str) -> Result<Vec<String>, RuneError> {
        let value = self.get_value(path)?;
        match value {
            Value::Object(items) => Ok(items.iter().map(|(k, _)| k.clone()).collect()),
            _ => Err(RuneError::TypeError {
                message: format!("Path '{}' is not an object", path),
                line: 0,
                column: 0,
                hint: Some("Only objects have keys".into()),
                code: Some(306),
            })
        }
    }

    /// Check if a configuration path exists
    pub fn has(&self, path: &str) -> bool {
        self.get_value(path).is_ok()
    }
}

fn enhance_error_with_line_info(e: RuneError, path: &str, raw_content: &str) -> RuneError {
    match e {
        RuneError::TypeError { message, hint, code, .. } => {
            let (line, snippet) = helpers::find_config_line(path, raw_content);
            if line > 0 {
                RuneError::TypeError {
                    message: format!("{}\n  → {}", message, snippet),
                    line,
                    column: 0,
                    hint,
                    code,
                }
            } else {
                RuneError::TypeError {
                    message,
                    line: 0,
                    column: 0,
                    hint,
                    code,
                }
            }
        }
        RuneError::ValidationError { message, hint, code, .. } => {
            let (line, snippet) = helpers::find_config_line(path, raw_content);
            if line > 0 {
                RuneError::ValidationError {
                    message: format!("{}\n  → {}", message, snippet),
                    line,
                    column: 0,
                    hint,
                    code,
                }
            } else {
                RuneError::ValidationError {
                    message,
                    line: 0,
                    column: 0,
                    hint,
                    code,
                }
            }
        }
        other => other,
    }
}
