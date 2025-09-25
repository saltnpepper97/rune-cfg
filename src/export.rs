use crate::parser::Parser;
use crate::ast::Document;
use crate::RuneError;
use serde_json::json;
use std::fs;

pub fn export_document_to_json(doc: &Document) -> Result<String, RuneError> {
    // Convert Document -> serde_json::Value recursively
    fn value_to_json(v: &crate::ast::Value) -> serde_json::Value {
        match v {
            crate::ast::Value::String(s) => json!(s),
            crate::ast::Value::Number(n) => json!(n),
            crate::ast::Value::Bool(b) => json!(b),
            crate::ast::Value::Array(arr) => json!(arr.iter().map(value_to_json).collect::<Vec<_>>()),
            crate::ast::Value::Object(obj) => {
                let map = obj.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect::<serde_json::Map<_, _>>();
                serde_json::Value::Object(map)
            },
            crate::ast::Value::Reference(path) => {
                // Just serialize references as dotted strings
                json!(path.join("."))
            },
            crate::ast::Value::Interpolated(parts) => {
                json!(parts.iter().map(value_to_json).collect::<Vec<_>>())
            }
        }
    }

    let mut top = serde_json::Map::new();

    // Optionally include metadata and globals
    let metadata = doc.metadata.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect::<serde_json::Map<_, _>>();
    if !metadata.is_empty() { top.insert("metadata".into(), serde_json::Value::Object(metadata)); }

    let globals = doc.globals.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect::<serde_json::Map<_, _>>();
    if !globals.is_empty() { top.insert("globals".into(), serde_json::Value::Object(globals)); }

    let items = doc.items.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect::<serde_json::Map<_, _>>();
    top.insert("items".into(), serde_json::Value::Object(items));

    Ok(serde_json::to_string_pretty(&serde_json::Value::Object(top)).unwrap())
}

/// Export from a `.rune` file directly
pub fn export_rune_file(path: &str) -> Result<String, RuneError> {
    let input = fs::read_to_string(path)
        .map_err(|e| RuneError::SyntaxError { 
            message: format!("Failed to read file: {}", e), 
            line: 0, column: 0, hint: None, code: Some(500)
        })?;
    
    let mut parser = Parser::new(&input)?;
    let doc = parser.parse_document()?;
    export_document_to_json(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use std::fs;

    #[test]
    fn test_export_example_rune_to_json() {
        // Read defaults.rune
        let defaults_input = fs::read_to_string("examples/defaults.rune")
            .expect("Failed to read defaults.rune");
        let mut defaults_parser = Parser::new(&defaults_input).expect("Failed to create parser for defaults");
        let defaults_doc = defaults_parser.parse_document().expect("Failed to parse defaults.rune");

        // Read example.rune
        let example_input = fs::read_to_string("examples/example.rune")
            .expect("Failed to read example.rune");
        let mut parser = Parser::new(&example_input).expect("Failed to create parser for example");
        let mut doc = parser.parse_document().expect("Failed to parse example.rune");

        // Inject the defaults import
        parser.inject_import("defaults".to_string(), defaults_doc);

        // Export to JSON
        let json_output = export_document_to_json(&doc).expect("Failed to export document to JSON");

        println!("--- Exported JSON ---\n{}", json_output);

        // Optional: you can deserialize and assert
        let deserialized: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert!(deserialized.get("items").is_some());
        assert!(deserialized.get("metadata").is_some());
    }
}
