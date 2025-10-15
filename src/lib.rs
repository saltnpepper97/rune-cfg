pub mod ast;
pub mod error;
pub mod export;
pub mod lexer;
pub mod parser;
pub mod resolver;
pub mod utils;

pub use error::RuneError;
pub use ast::{Document, Value};

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Main configuration struct that holds parsed RUNE documents and handles resolution
pub struct RuneConfig {
    documents: HashMap<String, Document>,
    main_doc_key: String,
    raw_content: String, // Store for error reporting
}

impl RuneConfig {
    /// Load a RUNE configuration file from disk
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
    pub fn from_str(content: &str) -> Result<Self, RuneError> {
        let mut parser = parser::Parser::new(content)?;
        let main_doc = parser.parse_document()?;
        
        let mut documents = HashMap::new();
        let main_key = "main".to_string();
        
        documents.insert(main_key.clone(), main_doc);
        
        Ok(Self {
            documents,
            main_doc_key: main_key,
            raw_content: content.to_string(),
        })
    }

    /// Load a RUNE config with imports resolved from a base directory
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
            raw_content: content.to_string(),
        })
    }

    /// Get a value from the configuration using dot notation
    pub fn get<T>(&self, path: &str) -> Result<T, RuneError> 
    where
        T: TryFrom<Value, Error = RuneError>
    {
        let value = self.get_value(path)?;
        T::try_from(value).map_err(|e| {
            // Enhance error with line information if it's a type error
            match e {
                RuneError::TypeError { message, hint, code, .. } => {
                    let (line, snippet) = self.find_config_line(path);
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
                    let (line, snippet) = self.find_config_line(path);
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
        })
    }

    /// Get a value with validation - returns detailed error with line info if validation fails
    pub fn get_validated<T, F>(&self, path: &str, validator: F, valid_values: &str) -> Result<T, RuneError> 
    where
        T: TryFrom<Value, Error = RuneError>,
        F: FnOnce(&T) -> bool,
    {
        let value = self.get_value(path)?;
        let typed_value = T::try_from(value)?;
        
        if !validator(&typed_value) {
            let (line, snippet) = self.find_config_line(path);
            return Err(RuneError::ValidationError {
                message: format!(
                    "Invalid value for `{}`\nExpected: {}",
                    path, valid_values
                ),
                line,
                column: 0,
                hint: Some(format!("Valid values are: {}\n  → {}", valid_values, snippet)),
                code: Some(450),
            });
        }
        
        Ok(typed_value)
    }

    /// Get a string value and validate it's one of the allowed values
    pub fn get_string_enum(&self, path: &str, allowed_values: &[&str]) -> Result<String, RuneError> {
        let value: String = self.get(path)?;
        let lower_value = value.to_lowercase();
        
        if !allowed_values.iter().any(|&v| v.to_lowercase() == lower_value) {
            let (line, snippet) = self.find_config_line(path);
            return Err(RuneError::ValidationError {
                message: format!(
                    "Invalid value '{}' for `{}`",
                    value, path
                ),
                line,
                column: 0,
                hint: Some(format!("Expected one of: {}\n  → {}", allowed_values.join(", "), snippet)),
                code: Some(451),
            });
        }
        
        Ok(value)
    }

    /// Get an optional value - returns None if key doesn't exist, Error if exists but invalid
    pub fn get_optional<T>(&self, path: &str) -> Result<Option<T>, RuneError> 
    where
        T: TryFrom<Value, Error = RuneError>
    {
        match self.get_value(path) {
            Ok(value) => Ok(Some(T::try_from(value)?)),
            Err(e) => {
                // Check if it's a "not found" error vs a parsing error
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

    /// Check if a path exists in the raw content (for better error reporting)
    pub fn path_exists_in_content(&self, path: &str) -> bool {
        let (line, _) = self.find_config_line(path);
        line > 0
    }

    /// Find a key in the config content and return its line number + snippet
    pub fn find_config_line(&self, key: &str) -> (usize, String) {
        let key_parts: Vec<&str> = key.split('.').collect();
        let mut scope_stack: Vec<String> = Vec::new();

        for (idx, line) in self.raw_content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and blank lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Enter new scope like `launcher:` or `theme:`
            if trimmed.ends_with(':') && !trimmed.starts_with('@') {
                let scope_name = trimmed.trim_end_matches(':').trim().to_string();
                scope_stack.push(scope_name);
                continue;
            }

            // Exit scope
            if trimmed == "end" {
                scope_stack.pop();
                continue;
            }

            // Skip directives like @author
            if trimmed.starts_with('@') {
                continue;
            }

            // Check if the line contains a key (with or without =)
            let line_key = if let Some((k, _)) = trimmed.split_once('=') {
                k.trim()
            } else if let Some((k, _)) = trimmed.split_once(char::is_whitespace) {
                k.trim()
            } else {
                continue;
            };

            let full_path = {
                let mut path = scope_stack.clone();
                path.push(line_key.to_string());
                path.join(".")
            };

            if full_path == key {
                return (idx + 1, trimmed.to_string());
            }

            // fallback: match the last component (e.g. "border_style")
            let simple_key = key_parts.last().unwrap_or(&key);
            if line_key == *simple_key {
                return (idx + 1, trimmed.to_string());
            }
        }

        (0, "<key not found>".into())
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
                    let (line, snippet) = self.find_config_line(path);
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
                    Ok(Value::String(format!("sys_placeholder:{}", path[1..].join("."))))
                } else if path[0] == "runtime" {
                    Ok(Value::String(format!("runtime_placeholder:{}", path[1..].join("."))))
                } else {
                    // Try to resolve the reference
                    if let Some(resolved) = parser.resolve_reference(path, main_doc) {
                        self.resolve_value_recursively(resolved, parser, main_doc)
                    } else {
                        // Reference couldn't be resolved - return as-is for better error handling
                        // This allows TryFrom implementations to provide context-specific errors
                        Ok(value.clone())
                    }
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

    pub fn inject_import(&mut self, alias: String, document: Document) {
        self.documents.insert(alias, document);
    }
   
    pub fn import_aliases(&self) -> Vec<String> {
        self.documents
            .keys()
            .filter(|k| *k != &self.main_doc_key)
            .cloned()
            .collect()
    }    
    
    pub fn has_document(&self, name: &str) -> bool {
        self.documents.contains_key(name)
    }

    pub fn get_document(&self, name: &str) -> Option<&Document> {
        self.documents.get(name)
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
                // Handle unresolved references that look like boolean typos
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
        
        let app_name: String = config.get("app.name").expect("Failed to get app.name");
        assert_eq!(app_name, "TestApp");
        
        let host: String = config.get("app.server.host").expect("Failed to get host");
        assert_eq!(host, "localhost");
        
        let port: u16 = config.get("app.server.port").expect("Failed to get port");
        assert_eq!(port, 8080);
        
        let debug: bool = config.get("app.debug").expect("Failed to get debug");
        assert_eq!(debug, true);
        
        let features: Vec<String> = config.get("app.features").expect("Failed to get features");
        assert_eq!(features, vec!["auth", "logging"]);
        
        assert!(config.has("app.name"));
        assert!(!config.has("app.nonexistent"));
        
        let server_keys = config.get_keys("app.server").expect("Failed to get server keys");
        assert!(server_keys.contains(&"host".to_string()));
        assert!(server_keys.contains(&"port".to_string()));
    }

    #[test]
    fn test_string_enum_validation() {
        let config_content = r#"
theme:
  border "rounded"
  invalid "bad_value"
end
"#;

        let config = RuneConfig::from_str(config_content).expect("Failed to parse config");
        
        // Valid value should work
        let border = config.get_string_enum("theme.border", &["plain", "rounded", "thick"]);
        assert!(border.is_ok());
        
        // Invalid value should error
        let invalid = config.get_string_enum("theme.invalid", &["good", "better"]);
        assert!(invalid.is_err());
    }
}
