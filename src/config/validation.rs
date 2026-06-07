// Author: Dustin Pilgrim
// License: MIT

use super::*;
use crate::diagnostic::RuneDiagnostic;
use crate::schema::{SchemaDocument, SchemaField, SchemaType};

impl RuneConfig {
    pub fn get_validated<T, F>(
        &self,
        path: &str,
        validator: F,
        valid_values: &str,
    ) -> Result<T, RuneError>
    where
        T: TryFrom<Value, Error = RuneError>,
        F: FnOnce(&T) -> bool,
    {
        let value = self.get_value(path)?;
        let typed_value = T::try_from(value)?;

        if !validator(&typed_value) {
            let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
            return Err(RuneError::ValidationError {
                message: format!("Invalid value for `{}`\nExpected: {}", path, valid_values),
                line,
                column: 0,
                hint: Some(format!(
                    "Valid values are: {}\n  → {}",
                    valid_values, snippet
                )),
                code: Some(450),
            });
        }

        Ok(typed_value)
    }

    pub fn get_string_enum(
        &self,
        path: &str,
        allowed_values: &[&str],
    ) -> Result<String, RuneError> {
        let value = self.get_value(path)?;

        let string_value = match value {
            Value::String(s) => s,
            _ => {
                return Err(RuneError::TypeError {
                    message: format!("Expected string for `{}`, got {:?}", path, value),
                    line: 0,
                    column: 0,
                    hint: Some("Use a string value in your config".into()),
                    code: Some(401),
                });
            }
        };

        let lower_value = string_value.to_lowercase();

        if !allowed_values
            .iter()
            .any(|&v| v.to_lowercase() == lower_value)
        {
            let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
            return Err(RuneError::ValidationError {
                message: format!("Invalid value '{}' for `{}`", string_value, path),
                line,
                column: 0,
                hint: Some(format!(
                    "Expected one of: {}\n  → {}",
                    allowed_values.join(", "),
                    snippet
                )),
                code: Some(451),
            });
        }

        Ok(string_value)
    }

    pub fn path_exists_in_content(&self, path: &str) -> bool {
        let (line, _) = helpers::find_config_line(path, &self.raw_content);
        line > 0
    }

    pub fn validate_schema(&self, schema: &SchemaDocument) -> Vec<RuneDiagnostic> {
        let mut diagnostics = Vec::new();

        for block in &schema.blocks {
            match self.get_value(&block.root) {
                Ok(value) => {
                    if !matches!(value, Value::Object(_)) {
                        diagnostics.push(type_diagnostic(
                            &block.root,
                            "object",
                            &value_type_name(&value),
                            &self.raw_content,
                        ));
                        continue;
                    }

                    validate_fields(self, &block.root, &block.fields, &mut diagnostics);
                }
                Err(_) => {
                    diagnostics.push(
                        RuneDiagnostic::error(format!(
                            "Required schema root '{}' is missing",
                            block.root
                        ))
                        .with_code(650)
                        .with_hint(format!("Add an '{}' object to the config", block.root)),
                    );
                }
            }
        }

        diagnostics
    }
}

fn validate_fields(
    config: &RuneConfig,
    parent_path: &str,
    fields: &[SchemaField],
    diagnostics: &mut Vec<RuneDiagnostic>,
) {
    for field in fields {
        let path = format!("{}.{}", parent_path, field.name);
        match config.get_value(&path) {
            Ok(value) => validate_value(config, &path, field, &value, diagnostics),
            Err(_) if field.required || has_required_descendant(field) => {
                diagnostics.push(missing_diagnostic(&path, &config.raw_content));
            }
            Err(_) => {}
        }
    }
}

fn validate_value(
    config: &RuneConfig,
    path: &str,
    field: &SchemaField,
    value: &Value,
    diagnostics: &mut Vec<RuneDiagnostic>,
) {
    if !type_matches(&field.kind, value) {
        diagnostics.push(type_diagnostic(
            path,
            &field.kind.name(),
            &value_type_name(value),
            &config.raw_content,
        ));
        return;
    }

    if let (Some((min, max)), Value::Number(number)) = (field.range, value) {
        if *number < min || *number > max {
            diagnostics.push(
                line_diagnostic(
                    path,
                    format!("'{}' must be between {} and {}", path, min, max),
                    &config.raw_content,
                )
                .with_code(653)
                .with_hint(format!("Use a value in the range {}..{}", min, max)),
            );
        }
    }

    match (&field.kind, value) {
        (SchemaType::Enum(allowed), Value::String(actual)) => {
            if !allowed.iter().any(|value| value == actual) {
                diagnostics.push(
                    line_diagnostic(
                        path,
                        format!("'{}' must be one of: {}", path, allowed.join(", ")),
                        &config.raw_content,
                    )
                    .with_code(654)
                    .with_hint(format!(
                        "Replace '{}' with one of the allowed values",
                        actual
                    )),
                );
            }
        }
        (SchemaType::Array(inner), Value::Array(items)) => {
            for (index, item) in items.iter().enumerate() {
                if !type_matches(inner, item) {
                    diagnostics.push(
                        line_diagnostic(
                            path,
                            format!(
                                "'{}[{}]' expected {}, got {}",
                                path,
                                index,
                                inner.name(),
                                value_type_name(item)
                            ),
                            &config.raw_content,
                        )
                        .with_code(652),
                    );
                }
            }
        }
        (SchemaType::Object, Value::Object(_)) => {
            validate_fields(config, path, &field.fields, diagnostics);
        }
        _ => {}
    }
}

fn type_matches(kind: &SchemaType, value: &Value) -> bool {
    match (kind, value) {
        (SchemaType::Any, _) => true,
        (SchemaType::String, Value::String(_)) => true,
        (SchemaType::Int, Value::Number(number)) => number.fract() == 0.0,
        (SchemaType::Float | SchemaType::Number, Value::Number(_)) => true,
        (SchemaType::Bool, Value::Bool(_)) => true,
        (SchemaType::Regex, Value::Regex(_)) => true,
        (SchemaType::Null, Value::Null) => true,
        (SchemaType::Array(_), Value::Array(_)) => true,
        (SchemaType::Enum(_), Value::String(_)) => true,
        (SchemaType::Object, Value::Object(_)) => true,
        _ => false,
    }
}

fn has_required_descendant(field: &SchemaField) -> bool {
    field
        .fields
        .iter()
        .any(|child| child.required || has_required_descendant(child))
}

fn missing_diagnostic(path: &str, raw_content: &str) -> RuneDiagnostic {
    line_diagnostic(
        path,
        format!("Required config path '{}' is missing", path),
        raw_content,
    )
    .with_code(651)
    .with_hint(format!("Add '{}' to satisfy the schema", path))
}

fn type_diagnostic(path: &str, expected: &str, actual: &str, raw_content: &str) -> RuneDiagnostic {
    line_diagnostic(
        path,
        format!("'{}' expected {}, got {}", path, expected, actual),
        raw_content,
    )
    .with_code(652)
}

fn line_diagnostic(path: &str, message: String, raw_content: &str) -> RuneDiagnostic {
    let (line, snippet) = helpers::find_config_line(path, raw_content);
    let diagnostic = RuneDiagnostic::error(message);
    if line > 0 {
        diagnostic
            .with_line(line, 0)
            .with_hint(format!("Check around: {}", snippet))
    } else {
        diagnostic
    }
}

fn value_type_name(value: &Value) -> String {
    match value {
        Value::String(_) => "string".into(),
        Value::Number(number) if number.fract() == 0.0 => "int".into(),
        Value::Number(_) => "number".into(),
        Value::Bool(_) => "bool".into(),
        Value::Regex(_) => "regex".into(),
        Value::Array(_) => "array".into(),
        Value::Object(_) => "object".into(),
        Value::Reference(_) => "reference".into(),
        Value::Interpolated(_) => "interpolated".into(),
        Value::Conditional(_) => "conditional".into(),
        Value::Null => "null".into(),
    }
}
