// Author: Dustin Pilgrim
// License: MIT

use crate::RuneError;
use crate::ast::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaDocument {
    pub blocks: Vec<SchemaBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaBlock {
    pub root: String,
    pub fields: Vec<SchemaField>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaField {
    pub name: String,
    pub kind: SchemaType,
    pub required: bool,
    pub default: Option<Value>,
    pub range: Option<(f64, f64)>,
    pub fields: Vec<SchemaField>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    String,
    Int,
    Float,
    Number,
    Bool,
    Regex,
    Null,
    Any,
    Array(Box<SchemaType>),
    Enum(Vec<String>),
    Object,
}

impl SchemaDocument {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, RuneError> {
        let content = std::fs::read_to_string(&path).map_err(|e| RuneError::FileError {
            message: format!("Failed to read schema file: {}", e),
            path: path.as_ref().to_string_lossy().to_string(),
            hint: Some("Check that the schema file exists and is readable".into()),
            code: Some(601),
        })?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, RuneError> {
        let lines: Vec<(usize, String)> = content
            .lines()
            .enumerate()
            .map(|(idx, line)| (idx + 1, strip_comment(line).trim().to_string()))
            .collect();

        let mut blocks = Vec::new();
        let mut index = 0;

        while index < lines.len() {
            let (line_no, line) = &lines[index];
            if line.is_empty() {
                index += 1;
                continue;
            }

            let Some(rest) = line.strip_prefix("schema ") else {
                return Err(schema_error(
                    format!("Expected schema block, got '{}'", line),
                    *line_no,
                    "Use: schema <name>:",
                ));
            };

            let Some(root) = rest.strip_suffix(':').map(str::trim) else {
                return Err(schema_error(
                    "Expected ':' after schema name",
                    *line_no,
                    "Use: schema app:",
                ));
            };

            if root.is_empty() {
                return Err(schema_error(
                    "Expected schema name",
                    *line_no,
                    "Use: schema app:",
                ));
            }

            index += 1;
            let fields = parse_fields_until_end(&lines, &mut index)?;
            blocks.push(SchemaBlock {
                root: root.to_string(),
                fields,
                line: *line_no,
            });
        }

        Ok(Self { blocks })
    }
}

fn parse_fields_until_end(
    lines: &[(usize, String)],
    index: &mut usize,
) -> Result<Vec<SchemaField>, RuneError> {
    let mut fields = Vec::new();

    while *index < lines.len() {
        let (line_no, line) = &lines[*index];
        if line.is_empty() {
            *index += 1;
            continue;
        }

        if line == "end" {
            *index += 1;
            return Ok(fields);
        }

        if let Some(name) = line.strip_suffix(':').map(str::trim) {
            if name.is_empty() {
                return Err(schema_error(
                    "Expected object field name before ':'",
                    *line_no,
                    "Use: server:",
                ));
            }

            *index += 1;
            let nested = parse_fields_until_end(lines, index)?;
            fields.push(SchemaField {
                name: name.to_string(),
                kind: SchemaType::Object,
                required: false,
                default: None,
                range: None,
                fields: nested,
                line: *line_no,
            });
            continue;
        }

        fields.push(parse_field(line, *line_no)?);
        *index += 1;
    }

    Err(schema_error(
        "Unclosed schema block",
        lines.last().map(|(line, _)| *line).unwrap_or(0),
        "Close schema blocks and nested objects with 'end'",
    ))
}

fn parse_field(line: &str, line_no: usize) -> Result<SchemaField, RuneError> {
    let Some((name, rest)) = split_once_whitespace(line) else {
        return Err(schema_error(
            format!("Expected type for schema field '{}'", line),
            line_no,
            "Use: name string required",
        ));
    };

    let (kind, options) = parse_type(rest.trim(), line_no)?;
    let required = contains_word(options, "required");
    let range = parse_range(options, line_no)?;
    let default = parse_default(options, line_no)?;

    Ok(SchemaField {
        name: name.to_string(),
        kind,
        required,
        default,
        range,
        fields: Vec::new(),
        line: line_no,
    })
}

fn parse_type(input: &str, line_no: usize) -> Result<(SchemaType, &str), RuneError> {
    if let Some(rest) = input.strip_prefix("enum") {
        let rest = rest.trim_start();
        let Some((values, after)) = parse_bracketed(rest) else {
            return Err(schema_error(
                "Expected enum values",
                line_no,
                "Use: environment enum [\"dev\", \"prod\"]",
            ));
        };
        return Ok((SchemaType::Enum(parse_string_list(values)), after));
    }

    if input.starts_with('[') {
        let Some((inner, after)) = parse_bracketed(input) else {
            return Err(schema_error(
                "Expected array type",
                line_no,
                "Use: plugins [string]",
            ));
        };
        let (inner_type, trailing) = parse_type(inner.trim(), line_no)?;
        if !trailing.trim().is_empty() {
            return Err(schema_error(
                "Unexpected text inside array type",
                line_no,
                "Use a single array element type like [string]",
            ));
        }
        return Ok((SchemaType::Array(Box::new(inner_type)), after));
    }

    let (word, after) = split_first_word(input);
    let kind = match word {
        "string" | "str" => SchemaType::String,
        "int" | "integer" => SchemaType::Int,
        "float" => SchemaType::Float,
        "number" => SchemaType::Number,
        "bool" | "boolean" => SchemaType::Bool,
        "regex" => SchemaType::Regex,
        "null" => SchemaType::Null,
        "any" => SchemaType::Any,
        "object" => SchemaType::Object,
        _ => {
            return Err(schema_error(
                format!("Unknown schema type '{}'", word),
                line_no,
                "Use string, int, float, number, bool, regex, null, any, enum, object, or [type]",
            ));
        }
    };

    Ok((kind, after))
}

fn parse_range(input: &str, line_no: usize) -> Result<Option<(f64, f64)>, RuneError> {
    let Some(range_start) = find_word(input, "range") else {
        return Ok(None);
    };
    let after = input[(range_start + "range".len())..].trim_start();
    let range_text = split_first_word(after).0;
    let Some((min, max)) = range_text.split_once("..") else {
        return Err(schema_error(
            "Expected range in min..max form",
            line_no,
            "Use: port int range 1..65535",
        ));
    };

    let min = min.parse::<f64>().map_err(|_| {
        schema_error(
            "Invalid range minimum",
            line_no,
            "Use numeric range bounds like 1..65535",
        )
    })?;
    let max = max.parse::<f64>().map_err(|_| {
        schema_error(
            "Invalid range maximum",
            line_no,
            "Use numeric range bounds like 1..65535",
        )
    })?;
    Ok(Some((min, max)))
}

fn parse_default(input: &str, line_no: usize) -> Result<Option<Value>, RuneError> {
    let Some(default_start) = find_word(input, "default") else {
        return Ok(None);
    };
    let raw = input[(default_start + "default".len())..].trim();
    if raw.is_empty() {
        return Err(schema_error(
            "Expected value after default",
            line_no,
            "Use: debug bool default false",
        ));
    }
    Ok(Some(parse_literal_value(raw)))
}

fn parse_literal_value(raw: &str) -> Value {
    let raw = raw.trim();
    if raw == "true" {
        return Value::Bool(true);
    }
    if raw == "false" {
        return Value::Bool(false);
    }
    if raw == "null" || raw == "None" {
        return Value::Null;
    }
    if let Ok(n) = raw.parse::<f64>() {
        return Value::Number(n);
    }
    if let Some(s) = parse_quoted(raw) {
        return Value::String(s);
    }
    Value::String(raw.to_string())
}

fn parse_string_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .filter_map(|part| {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(parse_quoted(trimmed).unwrap_or_else(|| trimmed.to_string()))
            }
        })
        .collect()
}

fn parse_bracketed(input: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    for (idx, ch) in input.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some((&input[1..idx], &input[(idx + 1)..]));
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_quoted(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let quote = bytes[0] as char;
    if (quote == '"' || quote == '\'') && bytes[bytes.len() - 1] as char == quote {
        return Some(input[1..input.len() - 1].to_string());
    }
    None
}

fn split_once_whitespace(input: &str) -> Option<(&str, &str)> {
    let idx = input.find(char::is_whitespace)?;
    Some((&input[..idx], &input[idx..]))
}

fn split_first_word(input: &str) -> (&str, &str) {
    let trimmed = input.trim_start();
    if let Some(idx) = trimmed.find(char::is_whitespace) {
        (&trimmed[..idx], &trimmed[idx..])
    } else {
        (trimmed, "")
    }
}

fn contains_word(input: &str, word: &str) -> bool {
    find_word(input, word).is_some()
}

fn find_word(input: &str, word: &str) -> Option<usize> {
    input.match_indices(word).find_map(|(idx, _)| {
        let before = input[..idx].chars().next_back();
        let after = input[(idx + word.len())..].chars().next();
        let before_ok = before.map(|c| c.is_whitespace()).unwrap_or(true);
        let after_ok = after.map(|c| c.is_whitespace()).unwrap_or(true);
        before_ok
            .then_some(())
            .and_then(|_| after_ok.then_some(idx))
    })
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#')
        .map(|(before, _)| before)
        .unwrap_or(line)
}

fn schema_error(message: impl Into<String>, line: usize, hint: impl Into<String>) -> RuneError {
    RuneError::SyntaxError {
        message: message.into(),
        line,
        column: 0,
        hint: Some(hint.into()),
        code: Some(600),
    }
}

impl SchemaType {
    pub fn name(&self) -> String {
        match self {
            SchemaType::String => "string".into(),
            SchemaType::Int => "int".into(),
            SchemaType::Float => "float".into(),
            SchemaType::Number => "number".into(),
            SchemaType::Bool => "bool".into(),
            SchemaType::Regex => "regex".into(),
            SchemaType::Null => "null".into(),
            SchemaType::Any => "any".into(),
            SchemaType::Array(inner) => format!("[{}]", inner.name()),
            SchemaType::Enum(values) => format!("enum [{}]", values.join(", ")),
            SchemaType::Object => "object".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_schema_blocks() {
        let schema = SchemaDocument::from_str(
            r#"
schema app:
  name string required
  debug bool default false
  environment enum ["dev", "staging", "production"]
  server:
    host string required
    port int range 1..65535 default 8080
  end
  plugins [string]
end
"#,
        )
        .expect("schema should parse");

        assert_eq!(schema.blocks.len(), 1);
        assert_eq!(schema.blocks[0].root, "app");
        assert_eq!(schema.blocks[0].fields.len(), 5);
        assert_eq!(schema.blocks[0].fields[3].kind, SchemaType::Object);
    }
}
