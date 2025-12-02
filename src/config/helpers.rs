use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::{Value, RuneError, Document, parser};

/// Parse gather statements from raw config content to extract file paths
/// Returns a map of alias -> raw_path
pub(super) fn parse_gather_paths(content: &str) -> HashMap<String, String> {
    let mut paths = HashMap::new();
    
    // Simple regex-free parsing of gather statements
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }
        
        // Look for: gather "path" [as alias]
        if trimmed.starts_with("gather") {
            if let Some(path_and_rest) = trimmed.strip_prefix("gather").map(|s| s.trim()) {
                // Extract quoted path
                if let Some(path) = extract_quoted_string(path_and_rest) {
                    // Check for "as alias"
                    let alias = if let Some(as_pos) = path_and_rest.find(" as ") {
                        let after_as = &path_and_rest[as_pos + 4..].trim();
                        // Get first word after "as"
                        after_as.split_whitespace().next()
                            .map(|s| s.to_string())
                    } else {
                        None
                    };
                    
                    let final_alias = alias.unwrap_or_else(|| {
                        // Use filename without extension as default
                        PathBuf::from(&path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("imported")
                            .to_string()
                    });
                    
                    paths.insert(final_alias, path);
                }
            }
        }
    }
    
    paths
}

/// Extract a quoted string from input like: "path/to/file" rest of line
fn extract_quoted_string(input: &str) -> Option<String> {
    let trimmed = input.trim();
    
    if trimmed.starts_with('"') {
        if let Some(end_quote) = trimmed[1..].find('"') {
            return Some(trimmed[1..end_quote + 1].to_string());
        }
    }
    
    if trimmed.starts_with('\'') {
        if let Some(end_quote) = trimmed[1..].find('\'') {
            return Some(trimmed[1..end_quote + 1].to_string());
        }
    }
    
    None
}

/// Resolve a path with tilde expansion and relative path handling
pub(super) fn resolve_path(raw_path: &str, base_dir: &Path) -> PathBuf {
    let path_str = raw_path.trim();
    
    // Handle tilde expansion
    if path_str.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path_str[2..]);
        }
    }
    
    // Handle absolute paths
    if path_str.starts_with('/') {
        return PathBuf::from(path_str);
    }
    
    // Handle relative paths
    base_dir.join(path_str)
}

/// Find a key in the config content and return its line number + snippet
pub(super) fn find_config_line(key: &str, raw_content: &str) -> (usize, String) {
    let key_parts: Vec<&str> = key.split('.').collect();
    let mut scope_stack: Vec<String> = Vec::new();

    for (idx, line) in raw_content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.ends_with(':') && !trimmed.starts_with('@') {
            let scope_name = trimmed.trim_end_matches(':').trim().to_string();
            scope_stack.push(scope_name);
            continue;
        }

        if trimmed == "end" {
            scope_stack.pop();
            continue;
        }

        if trimmed.starts_with('@') {
            continue;
        }

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

        let simple_key = key_parts.last().unwrap_or(&key);
        if line_key == *simple_key {
            return (idx + 1, trimmed.to_string());
        }
    }

    (0, "<key not found>".into())
}

/// Recursively resolve references to their final values
pub(super) fn resolve_value_recursively(
    value: &Value,
    parser: &parser::Parser,
    main_doc: &Document,
) -> Result<Value, RuneError> {
    match value {
        Value::Reference(path) => {
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
                if let Some(resolved) = parser.resolve_reference(path, main_doc) {
                    resolve_value_recursively(resolved, parser, main_doc)
                } else {
                    Ok(value.clone())
                }
            }
        }
        Value::Array(arr) => {
            let mut resolved_array = Vec::new();
            for item in arr {
                resolved_array.push(resolve_value_recursively(item, parser, main_doc)?);
            }
            Ok(Value::Array(resolved_array))
        }
        Value::Object(items) => {
            let mut resolved_object = Vec::new();
            for (key, val) in items {
                resolved_object.push((
                    key.clone(),
                    resolve_value_recursively(val, parser, main_doc)?,
                ));
            }
            Ok(Value::Object(resolved_object))
        }
        _ => Ok(value.clone()),
    }
}
