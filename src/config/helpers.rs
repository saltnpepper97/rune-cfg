// Author: Dustin Pilgrim
// License: GPL-3.0-or-later

use std::path::PathBuf;

use crate::{Document, RuneError, Value, parser};

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

    if let Some(rest) = trimmed.strip_prefix('"')
        && let Some(end_quote) = rest.find('"')
    {
        return Some(rest[..end_quote].to_string());
    }

    if let Some(rest) = trimmed.strip_prefix('\'')
        && let Some(end_quote) = rest.find('\'')
    {
        return Some(rest[..end_quote].to_string());
    }

    None
}

pub(super) fn find_config_line(key: &str, raw_content: &str) -> (usize, String) {
    let (line, _, snippet) = find_config_location(key, raw_content);
    (line, snippet)
}

/// A block that is open while `find_config_location` walks the source.
enum ScopeFrame {
    /// `name:` ... `end`; contributes `name` to the key path.
    Object(String),
    /// `if condition:` ... `end`; contributes nothing to the key path.
    Conditional,
}

pub(super) fn find_config_location(key: &str, raw_content: &str) -> (usize, usize, String) {
    let simple_key = key.rsplit('.').next().unwrap_or(key);
    let mut frames: Vec<ScopeFrame> = Vec::new();
    // First line whose key matches only the last path segment; used when no
    // line matches the full path.
    let mut fallback: Option<(usize, usize, String)> = None;

    let path_with = |frames: &[ScopeFrame], name: &str| -> String {
        frames
            .iter()
            .filter_map(|frame| match frame {
                ScopeFrame::Object(name) => Some(name.as_str()),
                ScopeFrame::Conditional => None,
            })
            .chain(std::iter::once(name))
            .collect::<Vec<_>>()
            .join(".")
    };

    for (idx, line) in raw_content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // block conditionals: `if` opens a block closed by `end`; `else` and
        // `elseif` continue it. None of them is part of a key path.
        let first_word = trimmed.split_whitespace().next().unwrap_or("");
        if first_word == "if" {
            frames.push(ScopeFrame::Conditional);
            continue;
        }
        if matches!(first_word, "else:" | "else" | "elseif" | "else-if") {
            continue;
        }

        // scope close: `end`, or `end name` repeating the block it closes
        if first_word == "end" {
            frames.pop();
            continue;
        }

        // ignore metadata
        if trimmed.starts_with('@') {
            continue;
        }

        // scope open: foo:
        if trimmed.ends_with(':') {
            let scope_name = trimmed.trim_end_matches(':').trim().to_string();
            if path_with(&frames, &scope_name) == key {
                return (
                    idx + 1,
                    column_for_token(line, &scope_name),
                    trimmed.to_string(),
                );
            }

            frames.push(ScopeFrame::Object(scope_name));
            continue;
        }

        let line_key = if let Some((k, _)) = trimmed.split_once('=') {
            k.trim()
        } else if let Some((k, _)) = trimmed.split_once(char::is_whitespace) {
            k.trim()
        } else {
            continue;
        };

        let location = || {
            (
                idx + 1,
                column_for_token(line, line_key),
                trimmed.to_string(),
            )
        };

        if path_with(&frames, line_key) == key {
            return location();
        }

        if fallback.is_none() && line_key == simple_key {
            fallback = Some(location());
        }
    }

    fallback.unwrap_or((0, 0, "<key not found>".into()))
}

fn column_for_token(line: &str, token: &str) -> usize {
    line.find(token).map(|column| column + 1).unwrap_or(1)
}

/// Shared condition evaluation for both inline conditionals and block if/end.
///
/// Schema validation uses this same evaluator for its location snapshot, so an
/// `if`/`elseif`/`else` chain selects one branch for values and diagnostics
/// alike.
///
/// A path's value is resolved fully first, through references, inline
/// conditionals and active if-block branches, so a condition sees what reading
/// that key would return. A missing key reads as `null`: only `false` and
/// `null` are false, and `0`, `""` and `[]` are true.
pub(super) fn condition_is_met(
    condition: &crate::ast::Condition,
    parser: &parser::Parser,
    doc: &Document,
) -> Result<bool, RuneError> {
    use crate::ast::Condition;

    // A path lookup can re-enter this evaluator through an if-block on the
    // path, so conditions count toward the cycle limit too.
    let _depth = ResolutionDepth::enter()?;

    let value = |path: &str| -> Result<Value, RuneError> {
        Ok(condition_value(parser, doc, path)?.unwrap_or(Value::Null))
    };
    let operand = |expected: &Value| resolve_value_recursively(expected, parser, doc);

    Ok(match condition {
        Condition::Truthy(path) => is_truthy(&value(path)?),
        Condition::Not(inner) => !condition_is_met(inner, parser, doc)?,
        Condition::Equals(path, expected) => values_equal(&value(path)?, &operand(expected)?),
        Condition::NotEquals(path, expected) => !values_equal(&value(path)?, &operand(expected)?),
        Condition::In(path, options) => {
            let actual = value(path)?;
            let mut found = false;
            for option in options {
                if values_equal(&actual, &operand(option)?) {
                    found = true;
                    break;
                }
            }
            found
        }
        Condition::Exists(path) => condition_value(parser, doc, path)?.is_some(),
        Condition::NotExists(path) => condition_value(parser, doc, path)?.is_none(),
    })
}

/// Ruby/Lua truthiness: only `false` and `null` are false.
fn is_truthy(value: &Value) -> bool {
    !matches!(value.primary(), Value::Bool(false) | Value::Null)
}

/// Equality on primary values, ignoring any `with` attributes.
fn values_equal(actual: &Value, expected: &Value) -> bool {
    actual.primary() == expected.primary()
}

/// The fully resolved value of a condition path, or `None` when the path is
/// not defined. An unset environment variable or unknown system key is not
/// defined either.
fn condition_value(
    parser: &parser::Parser,
    doc: &Document,
    path: &str,
) -> Result<Option<Value>, RuneError> {
    let segs: Vec<String> = path.split('.').map(String::from).collect();

    let raw = match segs[0].as_str() {
        // An unset variable is missing, not the empty string a value
        // reference would read in non-strict mode.
        "env" if segs.len() == 2 => return Ok(std::env::var(&segs[1]).ok().map(Value::String)),
        "env" | "sys" | "runtime" if segs.len() >= 2 => {
            return Ok(crate::resolver::parse_dollar_reference(segs).ok());
        }
        "var" if segs.len() >= 2 => lookup_path(parser, doc, &segs[1..])?,
        _ => lookup_path(parser, doc, &segs)?,
    };

    raw.map(|value| resolve_value_recursively(value, parser, doc))
        .transpose()
}

/// The unresolved value at `path`, looking through imports and into the
/// active branch of any if-block along the way.
///
/// The first segment names a top-level key or global, or an import alias when
/// more segments follow. Inside an object the first matching assignment in
/// source order wins, the same one reading the key returns.
pub(super) fn lookup_path<'a>(
    parser: &'a parser::Parser,
    doc: &'a Document,
    path: &[String],
) -> Result<Option<&'a Value>, RuneError> {
    let (doc, path) = match parser.imports.get(&path[0]) {
        Some(import) if path.len() > 1 => (import, &path[1..]),
        _ => (doc, path),
    };

    let Some(mut current) = doc
        .items
        .iter()
        .chain(&doc.globals)
        .find_map(|(key, value)| (key == &path[0]).then_some(value))
    else {
        return Ok(None);
    };

    for seg in &path[1..] {
        let next = match current {
            Value::Object(items) => find_in_items(items, seg, parser, doc)?,
            Value::Annotated(value) => value
                .attributes
                .iter()
                .find_map(|(key, value)| (key == seg).then_some(value)),
            _ => None,
        };
        let Some(next) = next else {
            return Ok(None);
        };
        current = next;
    }

    Ok(Some(current))
}

/// The first assignment to `key` among `items`, entering only the branch each
/// if-block takes.
fn find_in_items<'a>(
    items: &'a [crate::ast::ObjectItem],
    key: &str,
    parser: &parser::Parser,
    doc: &Document,
) -> Result<Option<&'a Value>, RuneError> {
    use crate::ast::ObjectItem;

    for item in items {
        match item {
            ObjectItem::Assign(name, value) if name == key => return Ok(Some(value)),
            ObjectItem::Assign(..) => {}
            ObjectItem::IfBlock(block) => {
                let branch: &[ObjectItem] = if condition_is_met(&block.condition, parser, doc)? {
                    &block.then_items
                } else {
                    block.else_items.as_deref().unwrap_or(&[])
                };
                if let Some(value) = find_in_items(branch, key, parser, doc)? {
                    return Ok(Some(value));
                }
            }
        }
    }

    Ok(None)
}

pub(super) fn evaluate_conditional(
    cond: &crate::ast::ConditionalValue,
    parser: &parser::Parser,
    doc: &Document,
) -> Result<Value, RuneError> {
    Ok(if condition_is_met(&cond.condition, parser, doc)? {
        cond.then_value.clone()
    } else {
        cond.else_value.clone().unwrap_or(Value::Null)
    })
}

/// How deeply value resolution may nest, through references, conditionals and
/// objects, before it is treated as a reference cycle.
const MAX_RESOLUTION_DEPTH: usize = 128;

thread_local! {
    static RESOLUTION_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// One level of value resolution, released when dropped.
struct ResolutionDepth;

impl ResolutionDepth {
    fn enter() -> Result<Self, RuneError> {
        RESOLUTION_DEPTH.with(|depth| {
            if depth.get() >= MAX_RESOLUTION_DEPTH {
                return Err(RuneError::RuntimeError {
                    message: "Reference cycle: resolving a value led back to itself".into(),
                    hint: Some(
                        "Check for keys that refer to each other, or a condition that reads the key it sets"
                            .into(),
                    ),
                    code: Some(310),
                });
            }
            depth.set(depth.get() + 1);
            Ok(Self)
        })
    }
}

impl Drop for ResolutionDepth {
    fn drop(&mut self) {
        RESOLUTION_DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

fn stringify_interpolated_value(value: &Value) -> Result<String, RuneError> {
    match value.primary() {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok(String::new()),
        other => Err(RuneError::TypeError {
            message: format!(
                "$var interpolation requires a scalar value, got {:?}",
                other
            ),
            line: 0,
            column: 0,
            hint: Some("Use string/number/bool/null for $var interpolation".into()),
            code: Some(401),
        }),
    }
}

fn resolve_var_reference(
    path: &[String],
    parser: &parser::Parser,
    main_doc: &Document,
) -> Result<Value, RuneError> {
    if path.len() < 2 {
        return Err(RuneError::SyntaxError {
            message: "Invalid $var path".into(),
            line: 0,
            column: 0,
            hint: Some("Use $var.<name> or $var.<path.to.value>".into()),
            code: Some(209),
        });
    }

    let target = &path[1..];
    let Some(resolved) = lookup_path(parser, main_doc, target)? else {
        return Err(RuneError::RuntimeError {
            message: format!(
                "Variable '{}' not found for $var reference",
                target.join(".")
            ),
            hint: Some("Define the variable before using $var.<name>".into()),
            code: Some(309),
        });
    };
    resolve_value_recursively(resolved, parser, main_doc)
}

fn interpolate_var_refs_in_string(
    input: &str,
    parser: &parser::Parser,
    main_doc: &Document,
) -> Result<String, RuneError> {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        if chars[i] != '$' {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        let mut j = i + 1;
        let mut ns = String::new();
        while j < chars.len() {
            let ch = chars[j];
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                ns.push(ch);
                j += 1;
            } else {
                break;
            }
        }

        if ns.is_empty() {
            out.push('$');
            i += 1;
            continue;
        }

        let mut path = vec![ns.clone()];
        while j + 1 < chars.len()
            && chars[j] == '.'
            && (chars[j + 1].is_alphanumeric() || chars[j + 1] == '_' || chars[j + 1] == '-')
        {
            j += 1;
            let mut seg = String::new();
            while j < chars.len() {
                let ch = chars[j];
                if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                    seg.push(ch);
                    j += 1;
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
        }

        if ns == "var" {
            let v = resolve_var_reference(&path, parser, main_doc)?;
            out.push_str(&stringify_interpolated_value(&v)?);
        } else {
            out.push('$');
            out.push_str(&path.join("."));
        }

        i = j;
    }

    Ok(out)
}

pub(super) fn resolve_value_recursively(
    value: &Value,
    parser: &parser::Parser,
    main_doc: &Document,
) -> Result<Value, RuneError> {
    let _depth = ResolutionDepth::enter()?;

    match value {
        Value::Conditional(cond) => {
            let resolved = evaluate_conditional(cond, parser, main_doc)?;
            resolve_value_recursively(&resolved, parser, main_doc)
        }

        Value::Reference(path) => {
            if path.first().map(|s| s.as_str()) == Some("env") && path.len() == 2 {
                let var_name = &path[1];
                std::env::var(var_name)
                    .map(Value::String)
                    .map_err(|_| RuneError::RuntimeError {
                        message: format!("Environment variable '{}' not set", var_name),
                        hint: Some("Make sure the environment variable is defined".into()),
                        code: Some(308),
                    })
            } else if path.first().map(|s| s.as_str()) == Some("sys") {
                Ok(Value::String(format!(
                    "sys_placeholder:{}",
                    path[1..].join(".")
                )))
            } else if path.first().map(|s| s.as_str()) == Some("runtime") {
                Ok(Value::String(format!(
                    "runtime_placeholder:{}",
                    path[1..].join(".")
                )))
            } else if path.first().map(|s| s.as_str()) == Some("var") {
                resolve_var_reference(path, parser, main_doc)
            } else if let Some(resolved) = lookup_path(parser, main_doc, path)? {
                resolve_value_recursively(resolved, parser, main_doc)
            } else {
                Ok(value.clone())
            }
        }

        Value::String(s) if !parser.options.quoted_references => Ok(Value::String(s.clone())),
        Value::String(s) => Ok(Value::String(interpolate_var_refs_in_string(
            s, parser, main_doc,
        )?)),

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
                            let take_then =
                                super::helpers::condition_is_met(&block.condition, parser, doc)?;
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

        Value::Annotated(value) => {
            let primary = resolve_value_recursively(&value.value, parser, main_doc)?;
            let mut attributes = Vec::with_capacity(value.attributes.len());
            for (name, attribute) in &value.attributes {
                attributes.push((
                    name.clone(),
                    resolve_value_recursively(attribute, parser, main_doc)?,
                ));
            }
            Ok(Value::Annotated(Box::new(crate::ast::AnnotatedValue {
                value: Box::new(primary),
                attributes,
            })))
        }

        _ => Ok(value.clone()),
    }
}
