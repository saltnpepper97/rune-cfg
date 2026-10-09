// Author: Dustin Pilgrim
// License: GPL-3.0-or-later

use std::env;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use sysinfo::System;

use crate::RuneError;
use crate::ast::Value;
use crate::utils::{format_bytes, format_uptime};

/// Cache for sysinfo::System to avoid allocating and refreshing on every $sys lookup.
struct SysCache {
    sys: System,
    last_mem_refresh: Instant,
}

static SYS_CACHE: OnceLock<Mutex<SysCache>> = OnceLock::new();

fn sys_cache() -> &'static Mutex<SysCache> {
    SYS_CACHE.get_or_init(|| {
        Mutex::new(SysCache {
            sys: System::new(), // NOT new_all()
            // Force first access to refresh memory if requested.
            last_mem_refresh: Instant::now() - Duration::from_secs(3600),
        })
    })
}

/// Expand a quoted string with legacy options (quoted references remain enabled).
pub fn expand_dollar_string(s: &str) -> Result<Value, RuneError> {
    expand_dollar_string_with_options(s, crate::ParseOptions::default())
}

/// Expand only Rune namespaces inside text; shell dollar expressions stay literal.
pub fn expand_dollar_string_with_options(
    s: &str,
    options: crate::ParseOptions,
) -> Result<Value, RuneError> {
    expand_dollar_string_with_escapes(s, options, &[])
}

pub(crate) fn expand_dollar_string_with_escapes(
    s: &str,
    options: crate::ParseOptions,
    escaped_dollars: &[usize],
) -> Result<Value, RuneError> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let mut offset = 0;
    let mut whole_reference = None;
    while let Some(ch) = chars.next() {
        let escaped = escaped_dollars.contains(&offset);
        offset += 1;
        if ch != '$' || escaped {
            result.push(ch);
            continue;
        }
        let mut namespace = String::new();
        while chars
            .peek()
            .is_some_and(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        {
            namespace.push(chars.next().unwrap());
            offset += 1;
        }
        let mut path = vec![namespace.clone()];
        loop {
            if chars.peek() != Some(&'.') {
                break;
            }
            let mut lookahead = chars.clone();
            lookahead.next();
            // A trailing dot or punctuation is ordinary text, not a broken reference.
            if !lookahead
                .peek()
                .is_some_and(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            {
                break;
            }
            chars.next();
            offset += 1;
            let mut segment = String::new();
            while chars.peek().is_some_and(|c| {
                c.is_alphanumeric()
                    || *c == '_'
                    || (*c == '-' && !(namespace == "env" && options.strict_env))
            }) {
                segment.push(chars.next().unwrap());
                offset += 1;
            }
            if segment.is_empty() {
                break;
            }
            path.push(segment);
        }
        let replacement = match namespace.as_str() {
            "env" => resolve_env_with_options(&path, options)?,
            "sys" => resolve_sys(&path)?,
            _ => {
                if options.quoted_references
                    && s == format!("${}", path.join("."))
                    && chars.peek().is_none()
                    && !namespace.is_empty()
                {
                    whole_reference = Some(path.clone());
                }
                format!("${}", path.join("."))
            }
        };
        result.push_str(&replacement);
    }
    Ok(match whole_reference {
        Some(path) => Value::Reference(path),
        None => Value::String(result),
    })
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
    parse_dollar_reference_with_options(path, crate::ParseOptions::default())
}

pub fn parse_dollar_reference_with_options(
    path: Vec<String>,
    options: crate::ParseOptions,
) -> Result<Value, RuneError> {
    if path.is_empty() {
        return Ok(Value::Reference(path));
    }
    match path[0].as_str() {
        "env" => Ok(Value::String(resolve_env_with_options(&path, options)?)),
        "sys" => Ok(Value::String(resolve_sys(&path)?)),
        "runtime" => Ok(Value::Reference(path)), // runtime is resolved later
        _ => Ok(Value::Reference(path)),
    }
}

/// $env resolver
fn resolve_env(path: &[String]) -> Result<String, RuneError> {
    resolve_env_with_options(path, crate::ParseOptions::default())
}

fn resolve_env_with_options(
    path: &[String],
    options: crate::ParseOptions,
) -> Result<String, RuneError> {
    if path.len() != 2 {
        return Err(RuneError::SyntaxError {
            message: format!("Invalid $env path: {}", path.join(".")),
            line: 0,
            column: 0,
            hint: Some("Use $env.<VAR_NAME>".into()),
            code: Some(209),
        });
    }
    match env::var(&path[1]) {
        Ok(value) => Ok(value),
        Err(_) if !options.strict_env => Ok(String::new()),
        Err(_) => Err(RuneError::SyntaxError {
            message: format!(
                "Environment variable '{}' is unset or is not Unicode",
                path[1]
            ),
            line: 0,
            column: 0,
            hint: Some(format!(
                "Set {} before loading the config, or use a shell variable in commands",
                path[1]
            )),
            code: Some(218),
        }),
    }
}

/// Helper: consistent "unresolved" error for $sys.<key>
fn sys_unresolved(key: &str) -> RuneError {
    RuneError::SyntaxError {
        message: format!("Unable to resolve $sys.{}", key),
        line: 0,
        column: 0,
        hint: None,
        code: Some(213),
    }
}

/// Run `f` with a cached System, refreshing memory at most once per second.
fn with_sys_memory_refreshed<F>(f: F) -> Result<String, RuneError>
where
    F: FnOnce(&System) -> Result<String, RuneError>,
{
    let cache = sys_cache();
    let mut guard = cache.lock().unwrap();

    // Rate-limit memory refresh (tune as desired)
    if guard.last_mem_refresh.elapsed() >= Duration::from_secs(1) {
        guard.sys.refresh_memory();
        guard.last_mem_refresh = Instant::now();
    }

    f(&guard.sys)
}

/// $sys resolver using sysinfo crate (cached, targeted refresh)
fn resolve_sys(path: &[String]) -> Result<String, RuneError> {
    // Get the key and ensure it exists
    let key = path.get(1).ok_or_else(|| RuneError::SyntaxError {
        message: format!("Missing key in $sys path: {}", path.join(".")),
        line: 0,
        column: 0,
        hint: Some("Use $sys.<KEY>".into()),
        code: Some(211),
    })?;

    match key.as_str() {
        // These are static / cheap; no System instance needed
        "os" => System::name().ok_or_else(|| sys_unresolved(key)),
        "kernel_version" | "kernel-version" => System::kernel_version().ok_or_else(|| sys_unresolved(key)),
        "os_version" | "os-version" => System::os_version().ok_or_else(|| sys_unresolved(key)),
        "hostname" => System::host_name().ok_or_else(|| sys_unresolved(key)),
        "cpu_arch" | "cpu-arch" => {
            let arch = System::cpu_arch();
            if arch.is_empty() {
                Err(sys_unresolved(key))
            } else {
                Ok(arch)
            }
        }
        "uptime" => Ok(format_uptime(System::uptime())),

        // Needs cached System; no refresh required for cpu list length
        "cpu_count" | "cpu-count" => {
            let cache = sys_cache();
            let guard = cache.lock().unwrap();
            Ok(guard.sys.cpus().len().to_string())
        }

        // Memory needs refresh; we refresh memory only, and rate-limit it
        "memory_total" | "memory-total" => {
            with_sys_memory_refreshed(|sys| Ok(format_bytes(sys.total_memory())))
        }
        "memory_free" | "memory-free" => {
            with_sys_memory_refreshed(|sys| Ok(format_bytes(sys.free_memory())))
        }
        "memory_used" | "memory-used" => {
            with_sys_memory_refreshed(|sys| Ok(format_bytes(sys.used_memory())))
        }

        other => Err(RuneError::SyntaxError {
            message: format!("Unknown $sys key: {}", other),
            line: 0,
            column: 0,
            hint: Some(
                "Available keys: os, kernel_version, os_version, hostname, cpu_arch, cpu_count, memory_total, memory_free, memory_used, uptime".into()
            ),
            code: Some(212),
        }),
    }
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
        ];

        for &key in &keys {
            let input = format!("$sys.{}", key);
            let result =
                expand_dollar_string(&input).unwrap_or_else(|_| panic!("Failed on key: {}", key));

            match result {
                Value::String(s) => {
                    assert!(!s.is_empty(), "Value for $sys.{} should not be empty", key);
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
