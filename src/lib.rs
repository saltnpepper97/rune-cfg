pub mod ast;
pub mod error;
pub mod export;
pub mod lexer;
pub mod parser;

pub use error::RuneError;
pub use ast::{Document, Value};

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Main configuration struct that holds parsed RUNE documents and handles resolution
pub struct RuneConfig {
    documents: HashMap<String, Document>,
    main_doc_key: String,
}

impl RuneConfig {
    /// Load a RUNE configuration file from disk
    /// 
    /// # Example (doc-test friendly)
    /// ```rust
    /// use rune_cfg::RuneConfig;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Instead of reading a file, we use inline content for doc-test
    /// let rune_content = r#"
    /// app:
    ///   name "MyApp"
    ///   version "1.0.0"
    /// end
    /// "#;
    ///
    /// let config = RuneConfig::from_str(rune_content)?;
    /// let app_name: String = config.get("app.name")?;
    /// println!("{}", app_name);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RuneError> {
        let content = fs::read_to_string(&path).map_err(|e| RuneError::FileError {
            message: format!("Failed to read file: {}", e),
            path: path.as_ref().to_string_lossy().to_string(),
            hint: Some("Check that the file exists and is readable".into()),
            code: Some(301),
        })?;
        
        Self::from_str(&content)
    }

    /// Parse RUNE configuration from a string
    /// 
    /// # Example
    /// ```rust
    /// use rune_cfg::RuneConfig;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = r#"
    /// app:
    ///   name "MyApp"
    ///   version "1.0.0"
    /// end
    /// "#;
    /// let config = RuneConfig::from_str(content)?;
    /// let app_name: String = config.get("app.name")?;
    /// println!("{}", app_name);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_str(content: &str) -> Result<Self, RuneError> {
        let mut parser = parser::Parser::new(content)?;
        let main_doc = parser.parse_document()?;
        
        let mut documents = HashMap::new();
        let main_key = "main".to_string();
        
        documents.insert(main_key.clone(), main_doc);
        
        Ok(Self {
            documents,
            main_doc_key: main_key,
        })
    }

    /// Load a RUNE config with imports resolved from a base directory
    /// 
    /// This will automatically load any `gather` statements relative to the base directory
    pub fn from_file_with_imports<P: AsRef<Path>>(path: P, base_dir: P) -> Result<Self, RuneError> {
        let content = fs::read_to_string(&path).map_err(|e| RuneError::FileError {
            message: format!("Failed to read file: {}", e),
            path: path.as_ref().to_string_lossy().to_string(),
            hint: Some("Check that the file exists and is readable".into()),
            code: Some(301),
        })?;

        let mut parser = parser::Parser::new(&content)?;
        
        // First, parse the main document to find imports
        let main_doc = parser.parse_document()?;
        
        // Load all imported documents - collect aliases first to avoid borrow checker issues
        let aliases: Vec<String> = parser.imports.keys().cloned().collect();
        for alias in aliases {
            let import_path = base_dir.as_ref().join(format!("{}.rune", alias));
            if import_path.exists() {
                let import_content = fs::read_to_string(&import_path).map_err(|e| RuneError::FileError {
                    message: format!("Failed to read import file: {}", e),
                    path: import_path.to_string_lossy().to_string(),
                    hint: Some("Check that the imported file exists".into()),
                    code: Some(302),
                })?;
                
                let mut import_parser = parser::Parser::new(&import_content)?;
                let import_doc = import_parser.parse_document()?;
                parser.inject_import(alias, import_doc);
            }
        }

        let mut documents = HashMap::new();
        let main_key = "main".to_string();
        
        // Store main document and all imports
        documents.insert(main_key.clone(), main_doc);
        for (alias, doc) in parser.imports {
            documents.insert(alias, doc);
        }
        
        Ok(Self {
            documents,
            main_doc_key: main_key,
        })
    }

    /// Get a value from the configuration using dot notation
    ///  
    /// # Example
    /// ```rust
    /// use rune_cfg::RuneConfig;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let rune_content = r#"
    /// app:
    ///   server:
    ///     host "localhost"
    ///     port 8080
    ///   end
    /// end
    /// "#;
    /// let config = RuneConfig::from_str(rune_content)?;
    ///
    /// let host: String = config.get("app.server.host")?;
    /// let port: u16 = config.get("app.server.port")?;
    /// println!("{}:{}", host, port);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get<T>(&self, path: &str) -> Result<T, RuneError> 
    where
        T: TryFrom<Value, Error = RuneError>
    {
        let value = self.get_value(path)?;
        T::try_from(value)
    }

    /// Get a raw Value from the configuration
    pub fn get_value(&self, path: &str) -> Result<Value, RuneError> {
        let path_segments: Vec<String> = path.split('.').map(|s| s.to_string()).collect();
        
        if let Some(main_doc) = self.documents.get(&self.main_doc_key) {
            // Create a temporary parser for resolution (this is a bit awkward but works with current design)
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
                .ok_or_else(|| RuneError::SyntaxError {
                    message: format!("Path '{}' not found in configuration", path),
                    line: 0,
                    column: 0,
                    hint: Some("Check that the path exists in your config file".into()),
                    code: Some(304),
                })?;

            // Recursively resolve references until we get a concrete value
            self.resolve_value_recursively(resolved, &temp_parser, main_doc)
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

    /// Helper method to recursively resolve references to their final values
    fn resolve_value_recursively(
        &self,
        value: &Value,
        parser: &parser::Parser,
        main_doc: &Document,
    ) -> Result<Value, RuneError> {
        match value {
            Value::Reference(path) => {
                // Check for env/sys/runtime
                if path[0] == "env" && path.len() == 2 {
                    let var_name = &path[1];
                    std::env::var(var_name)
                        .map(Value::String)
                        .map_err(|_| RuneError::RuntimeError {
                            message: format!("Environment variable '{}' not set", var_name),
                            hint: Some("Make sure the environment variable is defined".into()),
                            code: Some(308),
                        })
                } else if path[0] == "sys" {
                    // You could implement sys variables (like OS info)
                    Ok(Value::String(format!("sys_placeholder:{}", path[1..].join("."))))
                } else if path[0] == "runtime" {
                    // Implement runtime-specific variables
                    Ok(Value::String(format!("runtime_placeholder:{}", path[1..].join("."))))
                } else {
                    // Otherwise treat as document reference
                    let resolved = parser
                        .resolve_reference(path, main_doc)
                        .ok_or_else(|| RuneError::SyntaxError {
                            message: format!("Reference {:?} could not be resolved", path),
                            line: 0,
                            column: 0,
                            hint: Some("Check that the referenced value exists".into()),
                            code: Some(307),
                        })?;

                    self.resolve_value_recursively(resolved, parser, main_doc)
                }
            }
            Value::Array(arr) => {
                let mut resolved_array = Vec::new();
                for item in arr {
                    resolved_array.push(self.resolve_value_recursively(item, parser, main_doc)?);
                }
                Ok(Value::Array(resolved_array))
            }
            Value::Object(items) => {
                let mut resolved_object = Vec::new();
                for (key, val) in items {
                    resolved_object.push((
                        key.clone(),
                        self.resolve_value_recursively(val, parser, main_doc)?,
                    ));
                }
                Ok(Value::Object(resolved_object))
            }
            _ => Ok(value.clone()),
        }
    }

    /// Get all keys at a given path level
    /// 
    /// # Example (doc-test friendly)
    /// ```rust
    /// use rune_cfg::RuneConfig;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let rune_content = r#"
    /// app:
    ///   server:
    ///     host "localhost"
    ///     port 8080
    ///   end
    /// end
    /// "#;
    ///
    /// let config = RuneConfig::from_str(rune_content)?;
    /// let server_keys = config.get_keys("app.server")?;
    /// assert!(server_keys.contains(&"host".to_string()));
    /// assert!(server_keys.contains(&"port".to_string()));
    /// # Ok(())
    /// # }
    /// ```
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

    /// Get the main document
    pub fn document(&self) -> Option<&Document> {
        self.documents.get(&self.main_doc_key)
    }

    /// Get all loaded documents (main + imports)
    pub fn all_documents(&self) -> &HashMap<String, Document> {
        &self.documents
    }
}

// Implement conversions from Value to common Rust types
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
            _ => Err(RuneError::TypeError {
                message: format!("Expected boolean, got {:?}", value),
                line: 0,
                column: 0,
                hint: Some("Use true or false in your config".into()),
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

// Add a FileError variant to your RuneError enum
impl RuneError {
    // Helper constructor for file errors
    pub fn file_error(message: String, path: String) -> Self {
        RuneError::FileError {
            message,
            path,
            hint: Some("Check file path and permissions".into()),
            code: Some(300),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_string() {
        let config_content = r#"
@description "Test config"
app_name "TestApp"

app:
  name app_name
  version "1.0.0"
  debug true
  
  server:
    host "localhost"
    port 8080
  end
  
  features [
    "auth"
    "logging"
  ]
end
"#;

        let config = RuneConfig::from_str(config_content).expect("Failed to parse config");
        
        // Test basic string value
        let app_name: String = config.get("app.name").expect("Failed to get app.name");
        assert_eq!(app_name, "TestApp");
        
        // Test nested value
        let host: String = config.get("app.server.host").expect("Failed to get host");
        assert_eq!(host, "localhost");
        
        // Test number conversion
        let port: u16 = config.get("app.server.port").expect("Failed to get port");
        assert_eq!(port, 8080);
        
        // Test boolean
        let debug: bool = config.get("app.debug").expect("Failed to get debug");
        assert_eq!(debug, true);
        
        // Test array
        let features: Vec<String> = config.get("app.features").expect("Failed to get features");
        assert_eq!(features, vec!["auth", "logging"]);
        
        // Test has()
        assert!(config.has("app.name"));
        assert!(!config.has("app.nonexistent"));
        
        // Test get_keys()
        let server_keys = config.get_keys("app.server").expect("Failed to get server keys");
        assert!(server_keys.contains(&"host".to_string()));
        assert!(server_keys.contains(&"port".to_string()));
    }
}

