// Author: Dustin Pilgrim
// License: MIT

use std::fs;
use serde_json::json;

use crate::ast::Document;
use crate::parser::Parser;
use crate::RuneError;

/// Export a RUNE document to JSON format.
///
/// Converts all RUNE values to their JSON equivalents:
/// - Strings, numbers, booleans → direct mapping
/// - Arrays, objects → nested JSON structures
/// - Regex → `{"regex": "pattern"}`
/// - References → dotted string path
/// - Conditionals → structured object with condition/then/else
/// - Null → JSON null
///
/// Notes on block `if ... endif`:
/// - Objects can contain conditional blocks as `ObjectItem::IfBlock`.
/// - This exporter preserves that structure instead of trying to evaluate it.
///   (Evaluation depends on runtime/environment and happens elsewhere.)
///
/// # Examples
/// ```no_run
/// use rune_cfg::{RuneConfig, export};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = RuneConfig::from_file("config.rune")?;
/// // Export would require internal document access
/// # Ok(())
/// # }
/// ```
pub fn export_document_to_json(doc: &Document) -> Result<String, RuneError> {
    fn value_to_json(v: &crate::ast::Value) -> serde_json::Value {
        match v {
            crate::ast::Value::String(s) => json!(s),
            crate::ast::Value::Number(n) => json!(n),
            crate::ast::Value::Bool(b) => json!(b),
            crate::ast::Value::Array(arr) => {
                json!(arr.iter().map(value_to_json).collect::<Vec<_>>())
            }
            crate::ast::Value::Object(obj) => object_items_to_json(obj),
            crate::ast::Value::Reference(path) => json!(path.join(".")),
            crate::ast::Value::Interpolated(parts) => {
                json!(parts.iter().map(value_to_json).collect::<Vec<_>>())
            }
            crate::ast::Value::Regex(r) => json!({ "regex": r.as_str() }),
            crate::ast::Value::Conditional(c) => {
                json!({
                    "conditional": {
                        "condition": format!("{:?}", c.condition),
                        "then": value_to_json(&c.then_value),
                        "else": c.else_value.as_ref().map(value_to_json)
                    }
                })
            }
            crate::ast::Value::Null => serde_json::Value::Null,
        }
    }

    fn object_items_to_json(items: &[crate::ast::ObjectItem]) -> serde_json::Value {
        use crate::ast::ObjectItem;

        // Keep ordering stable & preserve if-blocks.
        // Represent as an array of entries so we don't lose duplicates or structure.
        //
        // Example:
        // [
        //   {"key":"timeout","value":60},
        //   {"if":{...}}
        // ]
        let mut out: Vec<serde_json::Value> = Vec::new();

        for item in items {
            match item {
                ObjectItem::Assign(k, v) => {
                    out.push(json!({
                        "key": k,
                        "value": value_to_json(v)
                    }));
                }
                ObjectItem::IfBlock(block) => {
                    out.push(json!({
                        "if": if_block_to_json(block)
                    }));
                }
            }
        }

        serde_json::Value::Array(out)
    }

    fn if_block_to_json(block: &crate::ast::IfBlock) -> serde_json::Value {
        json!({
            "condition": format!("{:?}", block.condition),
            "then": object_items_to_json(&block.then_items),
            "else": block.else_items.as_ref().map(|items| object_items_to_json(items)),
        })
    }

    let mut top = serde_json::Map::new();

    let metadata = doc
        .metadata
        .iter()
        .map(|(k, v)| (k.clone(), value_to_json(v)))
        .collect::<serde_json::Map<_, _>>();
    if !metadata.is_empty() {
        top.insert("metadata".into(), serde_json::Value::Object(metadata));
    }

    let globals = doc
        .globals
        .iter()
        .map(|(k, v)| (k.clone(), value_to_json(v)))
        .collect::<serde_json::Map<_, _>>();
    if !globals.is_empty() {
        top.insert("globals".into(), serde_json::Value::Object(globals));
    }

    let items = doc
        .items
        .iter()
        .map(|(k, v)| (k.clone(), value_to_json(v)))
        .collect::<serde_json::Map<_, _>>();
    top.insert("items".into(), serde_json::Value::Object(items));

    Ok(serde_json::to_string_pretty(&serde_json::Value::Object(top)).unwrap())
}

/// Export a RUNE file directly to JSON.
///
/// Convenience function that reads, parses, and exports in one call.
///
/// # Examples
/// ```no_run
/// use rune_cfg::export::export_rune_file;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let json = export_rune_file("config.rune")?;
/// println!("{}", json);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
/// Returns error if file doesn't exist or contains invalid RUNE syntax.
pub fn export_rune_file(path: &str) -> Result<String, RuneError> {
    let input = fs::read_to_string(path).map_err(|e| RuneError::SyntaxError {
        message: format!("Failed to read file: {}", e),
        line: 0,
        column: 0,
        hint: None,
        code: Some(500),
    })?;

    let mut parser = Parser::new(&input)?;
    let doc = parser.parse_document()?;
    export_document_to_json(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::ast::{Document, Value, ObjectItem};
    use regex::Regex;
    use std::fs;

    #[test]
    fn test_export_example_rune_to_json() {
        let defaults_input =
            fs::read_to_string("examples/defaults.rune").expect("Failed to read defaults.rune");
        let mut defaults_parser =
            Parser::new(&defaults_input).expect("Failed to create parser for defaults");
        let defaults_doc = defaults_parser
            .parse_document()
            .expect("Failed to parse defaults.rune");

        let example_input =
            fs::read_to_string("examples/example.rune").expect("Failed to read example.rune");
        let mut parser = Parser::new(&example_input).expect("Failed to create parser for example");
        let doc = parser.parse_document().expect("Failed to parse example.rune");

        parser.inject_import("defaults".to_string(), defaults_doc);

        let json_output = export_document_to_json(&doc).expect("Failed to export document to JSON");

        println!("--- Exported JSON ---\n{}", json_output);

        let deserialized: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert!(deserialized.get("items").is_some());
        assert!(deserialized.get("metadata").is_some());
    }

    #[test]
    fn test_export_regex() {
        let doc = Document {
            items: vec![(
                "pattern".to_string(),
                Value::Regex(Regex::new("^foo.*bar$").unwrap()),
            )],
            metadata: vec![],
            globals: vec![],
        };

        let json_output = export_document_to_json(&doc).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_output).unwrap();

        assert_eq!(v["items"]["pattern"]["regex"], "^foo.*bar$");
    }

    #[test]
    fn test_export_object_is_array_of_entries_now() {
        // Objects now export as arrays of {"key","value"} / {"if":...} entries to preserve structure.
        let doc = Document {
            items: vec![(
                "root".to_string(),
                Value::Object(vec![ObjectItem::Assign(
                    "a".into(),
                    Value::Number(1.0),
                )]),
            )],
            metadata: vec![],
            globals: vec![],
        };

        let json_output = export_document_to_json(&doc).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_output).unwrap();

        assert!(v["items"]["root"].is_array());
        assert_eq!(v["items"]["root"][0]["key"], "a");
        assert_eq!(v["items"]["root"][0]["value"], 1.0);
    }
}
