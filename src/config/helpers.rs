use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{Value, RuneError, Document, parser};

pub(super) fn parse_gather_paths(content: &str) -> HashMap<String, String> {
    let mut paths = HashMap::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with('#') {
            continue;
        }
        
        if trimmed.starts_with("gather") {
            if let Some(path_and_rest) = trimmed.strip_prefix("gather").map(|s| s.trim()) {
                if let Some(path) = extract_quoted_string(path_and_rest) {
                    let alias = if let Some(as_pos) = path_and_rest.find(" as ") {
                        let after_as = &path_and_rest[as_pos + 4..].trim();
                        after_as.split_whitespace().next()
                            .map(|s| s.to_string())
                    } else {
                        None
                    };
                    
                    let final_alias = alias.unwrap_or_else(|| {
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

pub(super) fn resolve_path(raw_path: &str, base_dir: &Path) -> PathBuf {
    let path_str = raw_path.trim();
    
    if path_str.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path_str[2..]);
        }
    }
    
    if path_str.starts_with('/') {
        return PathBuf::from(path_str);
    }
    
    base_dir.join(path_str)
}

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

pub(super) fn evaluate_conditional(
    cond: &crate::ast::ConditionalValue,
    parser: &parser::Parser,
    doc: &Document,
) -> Value {
    use crate::resolver;
    
    let condition_met = match &cond.condition {
        crate::ast::Condition::Equals(path, expected) => {
            let path_segments: Vec<String> = path.split('.').map(String::from).collect();
            
            let actual = if path_segments.len() >= 2 {
                match path_segments[0].as_str() {
                    "env" => {
                        resolver::parse_dollar_reference(path_segments.clone()).ok()
                    }
                    "sys" => {
                        resolver::parse_dollar_reference(path_segments.clone()).ok()
                    }
                    _ => parser.resolve_reference(&path_segments, doc).cloned()
                }
            } else {
                parser.resolve_reference(&path_segments, doc).cloned()
            };
            
            if let Some(actual_value) = actual {
                &actual_value == expected
            } else {
                false
            }
        }
        crate::ast::Condition::NotEquals(path, expected) => {
            let path_segments: Vec<String> = path.split('.').map(String::from).collect();
            
            let actual = if path_segments.len() >= 2 {
                match path_segments[0].as_str() {
                    "env" => {
                        resolver::parse_dollar_reference(path_segments.clone()).ok()
                    }
                    "sys" => {
                        resolver::parse_dollar_reference(path_segments.clone()).ok()
                    }
                    _ => parser.resolve_reference(&path_segments, doc).cloned()
                }
            } else {
                parser.resolve_reference(&path_segments, doc).cloned()
            };
            
            if let Some(actual_value) = actual {
                &actual_value != expected
            } else {
                true
            }
        }
        crate::ast::Condition::Exists(path) => {
            let path_segments: Vec<String> = path.split('.').map(String::from).collect();
            
            if path_segments.len() >= 2 {
                match path_segments[0].as_str() {
                    "env" => resolver::parse_dollar_reference(path_segments).is_ok(),
                    "sys" => resolver::parse_dollar_reference(path_segments).is_ok(),
                    _ => parser.resolve_reference(&path_segments, doc).is_some()
                }
            } else {
                parser.resolve_reference(&path_segments, doc).is_some()
            }
        }
        crate::ast::Condition::NotExists(path) => {
            let path_segments: Vec<String> = path.split('.').map(String::from).collect();
            
            if path_segments.len() >= 2 {
                match path_segments[0].as_str() {
                    "env" => resolver::parse_dollar_reference(path_segments).is_err(),
                    "sys" => resolver::parse_dollar_reference(path_segments).is_err(),
                    _ => parser.resolve_reference(&path_segments, doc).is_none()
                }
            } else {
                parser.resolve_reference(&path_segments, doc).is_none()
            }
        }
    };
    
    if condition_met {
        cond.then_value.clone()
    } else {
        cond.else_value.clone().unwrap_or(Value::Null)
    }
}

pub(super) fn resolve_value_recursively(
    value: &Value,
    parser: &parser::Parser,
    main_doc: &Document,
) -> Result<Value, RuneError> {
    match value {
        Value::Conditional(cond) => {
            let resolved = evaluate_conditional(cond, parser, main_doc);
            resolve_value_recursively(&resolved, parser, main_doc)
        }
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
