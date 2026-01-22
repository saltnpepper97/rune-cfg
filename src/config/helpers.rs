// Author: Dustin Pilgrim
// License MIT

use std::path::PathBuf;

use crate::{parser, Document, RuneError, Value};

/// Gather statement parsed from a file.
#[derive(Debug, Clone)]
pub(super) struct GatherSpec {
    /// The alias key the parser will use for this gather (either explicit `as`, or file stem).
    pub alias: String,
    /// The raw path string inside quotes.
    pub raw_path: String,
    /// True if the config explicitly used `as alias`.
    pub explicit_alias: bool,
}

/// Parse gather statements from raw file content.
/// - Skips fully-commented lines (starting with '#').
/// - Supports `gather "path"` or `gather "path" as alias`.
/// - Allows trailing inline comments.
pub(super) fn parse_gather_specs(content: &str) -> Vec<GatherSpec> {
    let mut out = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if !trimmed.starts_with("gather") {
            continue;
        }

        let Some(rest) = trimmed.strip_prefix("gather").map(|s| s.trim()) else {
            continue;
        };

        // Extract the quoted path
        let Some(path) = extract_quoted_string(rest) else {
            continue;
        };

        // Determine if there is an explicit `as alias` after the quoted path.
        // We search the remainder after the closing quote.
        let (explicit_alias, alias_opt) = {
            let quote_char = rest.chars().next().unwrap_or('"');
            let after_open = &rest[1..];

            let end_rel = if quote_char == '"' {
                after_open.find('"')
            } else if quote_char == '\'' {
                after_open.find('\'')
            } else {
                None
            };

            if let Some(end_rel) = end_rel {
                // +2 accounts for opening quote + the closing quote itself
                let after_quote = rest[(end_rel + 2)..].trim();

                // allow: `as alias` (with arbitrary whitespace)
                if let Some(_as_pos) = after_quote.find("as") {
                    // require `as` to be a standalone token boundary-ish:
                    // simplest: split whitespace and look for "as"
                    let mut it = after_quote.split_whitespace();
                    let mut found_as = false;
                    let mut alias: Option<String> = None;

                    while let Some(tok) = it.next() {
                        if !found_as && tok == "as" {
                            found_as = true;
                            alias = it.next().map(|s| s.to_string());
                            break;
                        }
                    }

                    if found_as {
                        (true, alias)
                    } else {
                        (false, None)
                    }
                } else {
                    (false, None)
                }
            } else {
                (false, None)
            }
        };

        let alias = alias_opt.unwrap_or_else(|| {
            PathBuf::from(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("imported")
                .to_string()
        });

        out.push(GatherSpec {
            alias,
            raw_path: path,
            explicit_alias,
        });
    }

    out
}

/// Extract the first quoted string from the input.
/// Supports "double" or 'single' quotes.
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

pub(super) fn find_config_line(key: &str, raw_content: &str) -> (usize, String) {
    let key_parts: Vec<&str> = key.split('.').collect();
    let mut scope_stack: Vec<String> = Vec::new();

    for (idx, line) in raw_content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // scope open: foo:
        if trimmed.ends_with(':') && !trimmed.starts_with('@') {
            let scope_name = trimmed.trim_end_matches(':').trim().to_string();
            scope_stack.push(scope_name);
            continue;
        }

        // scope close: end / endif
        if trimmed == "end" || trimmed == "endif" {
            scope_stack.pop();
            continue;
        }

        // ignore metadata
        if trimmed.starts_with('@') {
            continue;
        }

        // ignore control keywords lines (if / else / elseif)
        if trimmed.starts_with("if ") || trimmed == "else:" || trimmed.starts_with("elseif ") {
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

/// Shared condition evaluation for both inline conditionals and block if/endif.
fn condition_is_met(
    condition: &crate::ast::Condition,
    parser: &parser::Parser,
    doc: &Document,
) -> bool {
    use crate::resolver;

    fn resolve_path_value(parser: &parser::Parser, doc: &Document, path: &str) -> Option<Value> {
        let segs: Vec<String> = path.split('.').map(String::from).collect();

        if segs.len() >= 2 {
            match segs[0].as_str() {
                "env" | "sys" | "runtime" => resolver::parse_dollar_reference(segs).ok(),
                _ => parser.resolve_reference(&segs, doc).cloned(),
            }
        } else {
            parser.resolve_reference(&segs, doc).cloned()
        }
    }

    match condition {
        crate::ast::Condition::Equals(path, expected) => resolve_path_value(parser, doc, path)
            .as_ref()
            .map(|actual| actual == expected)
            .unwrap_or(false),
        crate::ast::Condition::NotEquals(path, expected) => resolve_path_value(parser, doc, path)
            .as_ref()
            .map(|actual| actual != expected)
            .unwrap_or(true),
        crate::ast::Condition::Exists(path) => resolve_path_value(parser, doc, path).is_some(),
        crate::ast::Condition::NotExists(path) => resolve_path_value(parser, doc, path).is_none(),
    }
}

pub(super) fn evaluate_conditional(
    cond: &crate::ast::ConditionalValue,
    parser: &parser::Parser,
    doc: &Document,
) -> Value {
    if condition_is_met(&cond.condition, parser, doc) {
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
            if path.get(0).map(|s| s.as_str()) == Some("env") && path.len() == 2 {
                let var_name = &path[1];
                std::env::var(var_name)
                    .map(Value::String)
                    .map_err(|_| RuneError::RuntimeError {
                        message: format!("Environment variable '{}' not set", var_name),
                        hint: Some("Make sure the environment variable is defined".into()),
                        code: Some(308),
                    })
            } else if path.get(0).map(|s| s.as_str()) == Some("sys") {
                Ok(Value::String(format!("sys_placeholder:{}", path[1..].join("."))))
            } else if path.get(0).map(|s| s.as_str()) == Some("runtime") {
                Ok(Value::String(format!("runtime_placeholder:{}", path[1..].join("."))))
            } else if let Some(resolved) = parser.resolve_reference(path, main_doc) {
                resolve_value_recursively(resolved, parser, main_doc)
            } else {
                Ok(value.clone())
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
            use crate::ast::ObjectItem;

            fn flatten_items(
                out: &mut Vec<ObjectItem>,
                items: &[ObjectItem],
                parser: &parser::Parser,
                doc: &Document,
            ) -> Result<(), RuneError> {
                for item in items {
                    match item {
                        ObjectItem::Assign(k, v) => {
                            let rv = super::helpers::resolve_value_recursively(v, parser, doc)?;
                            out.push(ObjectItem::Assign(k.clone(), rv));
                        }
                        ObjectItem::IfBlock(block) => {
                            let take_then = super::helpers::condition_is_met(&block.condition, parser, doc);
                            let branch: &[ObjectItem] = if take_then {
                                &block.then_items
                            } else {
                                block.else_items.as_deref().unwrap_or(&[])
                            };
                            flatten_items(out, branch, parser, doc)?;
                        }
                    }
                }
                Ok(())
            }

            let mut flattened: Vec<ObjectItem> = Vec::new();
            flatten_items(&mut flattened, items, parser, main_doc)?;
            Ok(Value::Object(flattened))
        }

        _ => Ok(value.clone()),
    }
}
