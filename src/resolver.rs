// Author: Dustin Pilgrim
// License: MIT

use std::env;
use sysinfo::{Product, System};

use crate::ast::Value;
use crate::RuneError;
use crate::utils::{format_uptime, format_bytes};

/// Expands a dollar if it refers to $env or $sys.
/// Otherwise, keeps it as a Reference.
pub fn expand_dollar_string(s: &str) -> Result<Value, RuneError> {
    // Fast path: if no '$', return plain string
    if !s.contains('$') {
        return Ok(Value::String(s.to_string()));
    }

    // If the whole string looks like just a single $reference
    if s.starts_with('$') && !s[1..].contains(' ') && !s[1..].contains('/') {
        // Parse it as a reference
        let mut chars = s.chars().peekable();
        chars.next(); // consume '$'

        let mut path = Vec::new();
        let mut ns = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                ns.push(ch);
                chars.next();
            } else {
                break;
            }
        }
        path.push(ns.clone());

        while let Some(&ch) = chars.peek() {
            if ch == '.' {
                chars.next();
                let mut seg = String::new();
                while let Some(&ch2) = chars.peek() {
                    if ch2.is_alphanumeric() || ch2 == '_' || ch2 == '-' {
                        seg.push(ch2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if seg.is_empty() {
                    return Err(RuneError::SyntaxError {
                        message: "Expected identifier after '.'".into(),
                        line: 0,
                        column: 0,
                        hint: None,
                        code: Some(210),
                    });
                }
                path.push(seg);
            } else {
                break;
            }
        }

        return match path[0].as_str() {
            "env" => Ok(Value::String(resolve_env(&path)?)),
            "sys" => Ok(Value::String(resolve_sys(&path)?)),
            _ => Ok(Value::Reference(path)),
        };
    }

    // Otherwise: do inline interpolation → replace $env/$sys in string
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            // Parse path after $
            let mut ns = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    ns.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let mut path = vec![ns.clone()];

            while let Some(&c) = chars.peek() {
                if c == '.' {
                    chars.next();
                    let mut seg = String::new();
                    while let Some(&c2) = chars.peek() {
                        if c2.is_alphanumeric() || c2 == '_' || c2 == '-' {
                            seg.push(c2);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if seg.is_empty() {
                        return Err(RuneError::SyntaxError {
                            message: "Expected identifier after '.'".into(),
                            line: 0,
                            column: 0,
                            hint: None,
                            code: Some(210),
                        });
                    }
                    path.push(seg);
                } else {
                    break;
                }
            }

            let replacement = match path[0].as_str() {
                "env" => resolve_env(&path)?,
                "sys" => resolve_sys(&path)?,
                _ => format!("${}", path.join(".")),
            };
            result.push_str(&replacement);
        } else {
            result.push(ch);
        }
    }

    Ok(Value::String(result))
}


/// Resolve a `Value::Reference` during evaluation
pub fn resolve_reference_value(value: &Value) -> Result<Value, RuneError> {
    match value {
        Value::Reference(path) if !path.is_empty() => match path[0].as_str() {
            "env" => Ok(Value::String(resolve_env(path)?)),
            "sys" => Ok(Value::String(resolve_sys(path)?)),
            _ => Ok(value.clone()), // let globals handle later
        },
        _ => Ok(value.clone()),
    }
}

/// Parse $ references into a resolved value (for $env/$sys) or Reference (for others)
/// This is called by the parser when it encounters a $ token outside of strings
pub fn parse_dollar_reference(path: Vec<String>) -> Result<Value, RuneError> {
    if path.is_empty() {
        return Ok(Value::Reference(path));
    }
    
    match path[0].as_str() {
        "env" => Ok(Value::String(resolve_env(&path)?)),
        "sys" => Ok(Value::String(resolve_sys(&path)?)),
        "runtime" => Ok(Value::Reference(path)), // runtime is resolved later
        _ => Ok(Value::Reference(path)),
    }
}

/// $env resolver
fn resolve_env(path: &[String]) -> Result<String, RuneError> {
    if path.len() != 2 {
        return Err(RuneError::SyntaxError {
            message: format!("Invalid $env path: {}", path.join(".")),
            line: 0,
            column: 0,
            hint: Some("Use $env.<VAR_NAME>".into()),
            code: Some(209),
        });
    }
    Ok(env::var(&path[1]).unwrap_or_default())
}

/// $sys resolver using sysinfo crate
fn resolve_sys(path: &[String]) -> Result<String, RuneError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Get the key and ensure it exists
    let key = path.get(1).ok_or_else(|| RuneError::SyntaxError {
        message: format!("Missing key in $sys path: {}", path.join(".")),
        line: 0,
        column: 0,
        hint: Some("Use $sys.<KEY>".into()),
        code: Some(211),
    })?;

    let value = match key.as_str() {
        "os" => System::name(),
        "kernel_version" | "kernel-version" => System::kernel_version(),
        "os_version" | "os-version" => System::os_version(),
        "hostname" => System::host_name(),
        "product_name" | "product-name" => Product::name(),
        "cpu_arch" | "cpu-arch" => Some(System::cpu_arch()),
        "cpu_count" | "cpu-count" => Some(sys.cpus().len().to_string()),
        "memory_total" | "memory-total" => Some(format_bytes(sys.total_memory())),
        "memory_free" | "memory-free" => Some(format_bytes(sys.free_memory())),
        "memory_used" | "memory-used" => Some(format_bytes(sys.used_memory())),
        "uptime" => Some(format_uptime(System::uptime())),
        other => {
            return Err(RuneError::SyntaxError {
                message: format!("Unknown $sys key: {}", other),
                line: 0,
                column: 0,
                hint: Some(
                    "Available keys: os, kernel_version, os_version, hostname, cpu_count, memory_total, memory_free, uptime".into()
                ),
                code: Some(212),
            })
        }
    };

    value.ok_or_else(|| RuneError::SyntaxError {
        message: format!("Unable to resolve $sys.{}", key),
        line: 0,
        column: 0,
        hint: None,
        code: Some(213),
    })
}

// -- Tests --

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Value;

    #[test]
    fn test_sys_expansion() {
        // List of keys we want to test
        let keys = [
            "os",
            "kernel_version",
            "os_version",
            "hostname",
            "cpu_arch",
            "cpu_count",
            "memory_total",
            "memory_free",
            "memory_used",
            "uptime",
            "product-name"
        ];

        for &key in &keys {
            let input = format!("$sys.{}", key);
            let result = expand_dollar_string(&input).expect(&format!("Failed on key: {}", key));

            match result {
                Value::String(s) => {
                    assert!(
                        !s.is_empty(),
                        "Value for $sys.{} should not be empty",
                        key
                    );
                    println!("$sys.{} = {}", key, s);
                }
                _ => panic!("Expected Value::String for $sys.{}", key),
            }
        }
    }

    #[test]
    fn test_sys_unknown_key() {
        let input = "$sys.unknown_key";
        let err = expand_dollar_string(input).unwrap_err();
        match err {
            RuneError::SyntaxError { code, .. } => {
                assert_eq!(code, Some(212));
            }
            _ => panic!("Expected SyntaxError for unknown $sys key"),
        }
    }

    #[test]
    fn test_sys_missing_key() {
        let input = "$sys";
        let err = expand_dollar_string(input).unwrap_err();
        match err {
            RuneError::SyntaxError { code, .. } => {
                assert_eq!(code, Some(211));
            }
            _ => panic!("Expected SyntaxError for missing $sys key"),
        }
    }

    #[test]
    fn test_env_expansion() {
        // Pick a common environment variable, or set one just for test
        unsafe {
            std::env::set_var("RUNE_TEST_ENV", "hello_world");
        }

        let input = "$env.RUNE_TEST_ENV";
        let result = expand_dollar_string(input).expect("Failed to expand env var");

        match result {
            Value::String(s) => assert_eq!(s, "hello_world"),
            _ => panic!("Expected Value::String for $env.RUNE_TEST_ENV"),
        }
    }

    #[test]
    fn test_env_missing_key() {
        let input = "$env";
        let err = expand_dollar_string(input).unwrap_err();
        match err {
            RuneError::SyntaxError { code, .. } => {
                assert_eq!(code, Some(209));
            }
            _ => panic!("Expected SyntaxError for missing $env key"),
        }
    }

    #[test]
    fn test_parse_dollar_reference_env() {
        unsafe {
            std::env::set_var("TEST_VAR", "test_value");
        }
        
        let path = vec!["env".to_string(), "TEST_VAR".to_string()];
        let result = parse_dollar_reference(path).expect("Failed to parse $env reference");
        
        match result {
            Value::String(s) => assert_eq!(s, "test_value"),
            _ => panic!("Expected Value::String for $env.TEST_VAR"),
        }
    }

    #[test]
    fn test_parse_dollar_reference_sys() {
        let path = vec!["sys".to_string(), "hostname".to_string()];
        let result = parse_dollar_reference(path).expect("Failed to parse $sys reference");
        
        match result {
            Value::String(s) => assert!(!s.is_empty(), "Hostname should not be empty"),
            _ => panic!("Expected Value::String for $sys.hostname"),
        }
    }
}
