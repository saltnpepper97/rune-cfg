// Author: Dustin Pilgrim
// License: MIT

#[cfg(test)]
use super::*;
use std::collections::HashMap;

use crate::RuneDiagnostic;
use crate::SchemaDocument;
use crate::ast::ObjectItem;
use crate::source::SourceIndex;

#[test]
fn test_config_from_string() {
    let config_content = r#"
@description "Test config"
app_name "TestApp"
app:
  name app_name
  version "1.0.0"
  debug true

  server:
    host "localhost"
    port 8080
  end

  features [
    "auth"
    "logging"
  ]
end
"#;
    let config = RuneConfig::from_str(config_content).expect("Failed to parse config");

    let app_name: String = config.get("app.name").expect("Failed to get app.name");
    assert_eq!(app_name, "TestApp");

    let host: String = config.get("app.server.host").expect("Failed to get host");
    assert_eq!(host, "localhost");

    let port: u16 = config.get("app.server.port").expect("Failed to get port");
    assert_eq!(port, 8080);

    let debug: bool = config.get("app.debug").expect("Failed to get debug");
    assert!(debug);

    let features: Vec<String> = config.get("app.features").expect("Failed to get features");
    assert_eq!(features, vec!["auth", "logging"]);

    assert!(config.has("app.name"));
    assert!(!config.has("app.nonexistent"));

    let server_keys = config
        .get_keys("app.server")
        .expect("Failed to get server keys");
    assert!(server_keys.contains(&"host".to_string()));
    assert!(server_keys.contains(&"port".to_string()));
}

#[test]
fn test_string_enum_validation() {
    let config_content = r#"
theme:
  border "rounded"
  invalid "bad_value"
end
"#;
    let config = RuneConfig::from_str(config_content).expect("Failed to parse config");

    let border = config.get_string_enum("theme.border", &["plain", "rounded", "thick"]);
    assert!(border.is_ok());

    let invalid = config.get_string_enum("theme.invalid", &["good", "better"]);
    assert!(invalid.is_err());
}

#[test]
fn test_order_preservation() {
    let config_content = r#"
first "1"
second "2"
third "3"
nested:
    alpha "a"
    beta "b"
    gamma "c"
end
"#;
    let config = RuneConfig::from_str(config_content).unwrap();
    let keys = config.get_keys("nested").unwrap();
    assert_eq!(keys, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn test_unaliased_gather_deep_merges_sections_with_gather_precedence() {
    let dir = tempfile::tempdir().expect("temp dir");
    let defaults_path = dir.path().join("defaults.rune");
    let config_path = dir.path().join("config.rune");

    std::fs::write(
        &defaults_path,
        r##"
field:
  gap 44.0
  pins:
    colour "#d65d26"
    size 2.0
  end
end
"##,
    )
    .expect("write defaults");
    std::fs::write(
        &config_path,
        r##"
gather "defaults.rune"

field:
  gap 20.0
  pins:
    corner "top-right"
    size 1.0
  end
end
"##,
    )
    .expect("write config");

    let config = RuneConfig::from_file(&config_path).expect("config should parse");

    assert_eq!(config.get::<f32>("field.gap").unwrap(), 44.0);
    assert_eq!(
        config.get::<String>("field.pins.colour").unwrap(),
        "#d65d26"
    );
    assert_eq!(
        config.get::<String>("field.pins.corner").unwrap(),
        "top-right"
    );
    assert_eq!(config.get::<f32>("field.pins.size").unwrap(), 2.0);
}

#[test]
fn test_unaliased_gather_merges_multiple_default_sections_in_order() {
    let dir = tempfile::tempdir().expect("temp dir");
    let first_path = dir.path().join("first.rune");
    let second_path = dir.path().join("second.rune");
    let config_path = dir.path().join("config.rune");

    std::fs::write(
        &first_path,
        r##"
app:
  theme:
    foreground "#ffffff"
    background "#000000"
  end
end
"##,
    )
    .expect("write first defaults");
    std::fs::write(
        &second_path,
        r##"
app:
  theme:
    foreground "#eeeeee"
    accent "#d65d26"
  end
end
"##,
    )
    .expect("write second defaults");
    std::fs::write(
        &config_path,
        r##"
gather "first.rune"
gather "second.rune"

app:
  theme:
    background "#111111"
  end
end
"##,
    )
    .expect("write config");

    let config = RuneConfig::from_file(&config_path).expect("config should parse");

    assert_eq!(
        config.get::<String>("app.theme.foreground").unwrap(),
        "#eeeeee"
    );
    assert_eq!(
        config.get::<String>("app.theme.background").unwrap(),
        "#000000"
    );
    assert_eq!(config.get::<String>("app.theme.accent").unwrap(), "#d65d26");
}

#[test]
fn test_var_reference_resolves_top_level_variable() {
    let config_content = r#"
mod_main "alt"

dev:
  keybinds:
    modifier $var.mod_main
  end
end
"#;

    let config = RuneConfig::from_str(config_content).expect("Failed to parse config");
    let modifier: String = config
        .get("dev.keybinds.modifier")
        .expect("Failed to resolve $var.mod_main");
    assert_eq!(modifier, "alt");
}

#[test]
fn test_var_interpolation_inside_string() {
    let config_content = r#"
mod_main "alt+shift"

dev:
  keybinds:
    reload_config "$var.mod_main+r"
  end
end
"#;

    let config = RuneConfig::from_str(config_content).expect("Failed to parse config");
    let chord: String = config
        .get("dev.keybinds.reload_config")
        .expect("Failed to resolve interpolated $var in string");
    assert_eq!(chord, "alt+shift+r");
}

// ===== String Conversion Tests =====

#[test]
fn test_string_conversion() {
    let value = Value::String("hello".to_string());
    let result: Result<String, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello");
}

#[test]
fn test_string_conversion_error() {
    let value = Value::Number(42.0);
    let result: Result<String, RuneError> = value.try_into();
    assert!(result.is_err());
}

// ===== Number Conversion Tests =====

#[test]
fn test_f64_conversion() {
    let value = Value::Number(3.25);
    let result: Result<f64, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3.25);
}

#[test]
fn test_f32_conversion() {
    let value = Value::Number(2.5);
    let result: Result<f32, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2.5_f32);
}

#[test]
fn test_i32_conversion() {
    let value = Value::Number(42.0);
    let result: Result<i32, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_i64_conversion() {
    let value = Value::Number(1234567890.0);
    let result: Result<i64, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1234567890);
}

#[test]
fn test_u8_conversion() {
    let value = Value::Number(255.0);
    let result: Result<u8, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 255);
}

#[test]
fn test_u8_conversion_out_of_range() {
    let value = Value::Number(256.0);
    let result: Result<u8, RuneError> = value.try_into();
    assert!(result.is_err());

    let value = Value::Number(-1.0);
    let result: Result<u8, RuneError> = value.try_into();
    assert!(result.is_err());
}

#[test]
fn test_u16_conversion() {
    let value = Value::Number(65535.0);
    let result: Result<u16, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 65535);
}

#[test]
fn test_u16_conversion_out_of_range() {
    let value = Value::Number(65536.0);
    let result: Result<u16, RuneError> = value.try_into();
    assert!(result.is_err());
}

#[test]
fn test_u32_conversion() {
    let value = Value::Number(4294967295.0);
    let result: Result<u32, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 4294967295);
}

#[test]
fn test_u64_conversion() {
    let value = Value::Number(123456789.0);
    let result: Result<u64, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 123456789);
}

#[test]
fn test_usize_conversion() {
    let value = Value::Number(1000.0);
    let result: Result<usize, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1000);
}

// ===== Boolean Conversion Tests =====

#[test]
fn test_bool_conversion() {
    let value = Value::Bool(true);
    let result: Result<bool, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert!(result.unwrap());

    let value = Value::Bool(false);
    let result: Result<bool, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn test_bool_conversion_from_typo() {
    let value = Value::Reference(vec!["tru".to_string()]);
    let result: Result<bool, RuneError> = value.try_into();
    assert!(result.is_err());

    let value = Value::Reference(vec!["fals".to_string()]);
    let result: Result<bool, RuneError> = value.try_into();
    assert!(result.is_err());
}

#[test]
fn test_bool_conversion_error() {
    let value = Value::String("yes".to_string());
    let result: Result<bool, RuneError> = value.try_into();
    assert!(result.is_err());
}

// ===== Array/Vec Conversion Tests =====

#[test]
fn test_vec_string_conversion() {
    let value = Value::Array(vec![
        Value::String("one".to_string()),
        Value::String("two".to_string()),
        Value::String("three".to_string()),
    ]);

    let result: Result<Vec<String>, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["one", "two", "three"]);
}

#[test]
fn test_vec_number_conversion() {
    let value = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);

    let result: Result<Vec<i32>, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_vec_bool_conversion() {
    let value = Value::Array(vec![
        Value::Bool(true),
        Value::Bool(false),
        Value::Bool(true),
    ]);

    let result: Result<Vec<bool>, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![true, false, true]);
}

#[test]
fn test_vec_mixed_types_error() {
    let value = Value::Array(vec![Value::String("one".to_string()), Value::Number(2.0)]);

    let result: Result<Vec<String>, RuneError> = value.try_into();
    assert!(result.is_err());
}

#[test]
fn test_empty_vec_conversion() {
    let value = Value::Array(vec![]);
    let result: Result<Vec<String>, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Vec::<String>::new());
}

// ===== Option Conversion Tests =====

#[test]
fn test_option_none_conversion() {
    let value = Value::Null;
    let result: Result<Option<String>, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[test]
fn test_option_some_conversion() {
    let value = Value::String("hello".to_string());
    let result: Result<Option<String>, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("hello".to_string()));
}

#[test]
fn test_option_number_conversion() {
    let value = Value::Number(42.0);
    let result: Result<Option<i32>, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(42));
}

// ===== HashMap Conversion Tests =====
//
// NOTE:
// `Value::Object` now contains `Vec<ObjectItem>`, not `Vec<(String, Value)>`.
// Update these tests to build objects using `ObjectItem::Assign(...)`.
//

#[test]
fn test_hashmap_value_conversion() {
    let value = Value::Object(vec![
        ObjectItem::Assign("key1".to_string(), Value::String("value1".to_string())),
        ObjectItem::Assign("key2".to_string(), Value::Number(42.0)),
    ]);

    let result: Result<HashMap<String, Value>, RuneError> = value.try_into();
    assert!(result.is_ok());

    let map = result.unwrap();
    assert_eq!(map.len(), 2);
    assert!(map.contains_key("key1"));
    assert!(map.contains_key("key2"));
}

#[test]
fn test_hashmap_string_conversion() {
    let value = Value::Object(vec![
        ObjectItem::Assign("name".to_string(), Value::String("Alice".to_string())),
        ObjectItem::Assign("city".to_string(), Value::String("NYC".to_string())),
    ]);

    let result: Result<HashMap<String, String>, RuneError> = value.try_into();
    assert!(result.is_ok());

    let map = result.unwrap();
    assert_eq!(map.get("name"), Some(&"Alice".to_string()));
    assert_eq!(map.get("city"), Some(&"NYC".to_string()));
}

#[test]
fn test_hashmap_string_conversion_error() {
    let value = Value::Object(vec![
        ObjectItem::Assign("name".to_string(), Value::String("Alice".to_string())),
        ObjectItem::Assign("age".to_string(), Value::Number(30.0)),
    ]);

    let result: Result<HashMap<String, String>, RuneError> = value.try_into();
    assert!(result.is_err());
}

// ===== Tuple Conversion Tests =====

#[test]
fn test_tuple_string_string_conversion() {
    let value = Value::Array(vec![
        Value::String("key".to_string()),
        Value::String("value".to_string()),
    ]);

    let result: Result<(String, String), RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ("key".to_string(), "value".to_string()));
}

#[test]
fn test_tuple_string_value_conversion() {
    let value = Value::Array(vec![
        Value::String("config".to_string()),
        Value::Number(42.0),
    ]);

    let result: Result<(String, Value), RuneError> = value.try_into();
    assert!(result.is_ok());
    let (key, val) = result.unwrap();
    assert_eq!(key, "config");
    assert_eq!(val, Value::Number(42.0));
}

#[test]
fn inline_attributes_keep_typed_primary_and_addressable_fields() {
    let config = RuneConfig::from_str(
        r#"
default_delay 120
font "Inter" with size 14 antialias true delay default_delay
"#,
    )
    .expect("inline attributes should parse");

    assert_eq!(config.get::<String>("font").unwrap(), "Inter");
    assert_eq!(config.get::<f64>("font.size").unwrap(), 14.0);
    assert!(config.get::<bool>("font.antialias").unwrap());
    assert_eq!(config.get::<f64>("font.delay").unwrap(), 120.0);
    assert_eq!(
        config.get_keys("font").unwrap(),
        vec!["size", "antialias", "delay"]
    );
}

#[test]
fn test_tuple_wrong_length_error() {
    let value = Value::Array(vec![Value::String("only_one".to_string())]);

    let result: Result<(String, String), RuneError> = value.try_into();
    assert!(result.is_err());

    let value = Value::Array(vec![
        Value::String("one".to_string()),
        Value::String("two".to_string()),
        Value::String("three".to_string()),
    ]);

    let result: Result<(String, String), RuneError> = value.try_into();
    assert!(result.is_err());
}

// ===== Integration Tests with Config =====

#[test]
fn test_config_with_all_types() {
    let config_content = r#"
types:
    string_val "hello"
    int_val 42
    float_val 3.25
    bool_val true
    null_val null
    array_val [1, 2, 3]
    nested:
        key "value"
    end
end
"#;
    let config = RuneConfig::from_str(config_content).expect("Failed to parse config");

    let s: String = config.get("types.string_val").unwrap();
    assert_eq!(s, "hello");

    let i: i32 = config.get("types.int_val").unwrap();
    assert_eq!(i, 42);

    let f: f64 = config.get("types.float_val").unwrap();
    assert!((f - 3.25).abs() < 0.001);

    let b: bool = config.get("types.bool_val").unwrap();
    assert!(b);

    let opt: Option<String> = config.get("types.null_val").unwrap();
    assert_eq!(opt, None);

    let arr: Vec<i32> = config.get("types.array_val").unwrap();
    assert_eq!(arr, vec![1, 2, 3]);
}

#[test]
fn test_config_numeric_range_validation() {
    let config_content = r#"
numbers:
    small 10
    medium 1000
    large 1000000
end
"#;
    let config = RuneConfig::from_str(config_content).unwrap();

    let small_u8: Result<u8, RuneError> = config.get("numbers.small");
    assert!(small_u8.is_ok());

    let medium_u16: Result<u16, RuneError> = config.get("numbers.medium");
    assert!(medium_u16.is_ok());

    let large_u32: Result<u32, RuneError> = config.get("numbers.large");
    assert!(large_u32.is_ok());
}

#[test]
fn test_config_type_mismatch_errors() {
    let config_content = r#"
data:
    value "not a number"
end
"#;
    let config = RuneConfig::from_str(config_content).unwrap();

    let result: Result<i32, RuneError> = config.get("data.value");
    assert!(result.is_err());
}

#[test]
fn test_config_if_blocks_flatten_to_assignments() {
    let config_content = r#"
app:
  name "A"
  if debug:
    flag true
  else:
    flag false
  endif
end
"#;

    let config = RuneConfig::from_str(config_content).expect("Failed to parse config");

    // debug isn't set, so Condition::Exists("debug") is false → else branch → flag false
    let flag: bool = config.get("app.flag").expect("Failed to get app.flag");
    assert!(!flag);
}

#[test]
fn test_schema_validation_success() {
    let schema = SchemaDocument::from_str(
        r#"
schema app:
  name string required
  version string required
  debug bool default false
  environment enum ["dev", "staging", "production"] required
  server:
    host string required
    port int range 1..65535 default 8080
  end
  plugins [string]
end
"#,
    )
    .expect("schema should parse");

    let config = RuneConfig::from_str(
        r#"
app:
  name "RuneApp"
  version "1.0.0"
  debug true
  environment "production"
  server:
    host "localhost"
    port 8080
  end
  plugins ["auth", "logger"]
end
"#,
    )
    .expect("config should parse");

    let diagnostics = config.validate_schema(&schema);
    assert_eq!(diagnostics, vec![]);
}

#[test]
fn test_schema_validation_reports_errors() {
    let schema = SchemaDocument::from_str(
        r#"
schema app:
  name string required
  version string required
  environment enum ["dev", "staging", "production"] required
  server:
    host string required
    port int range 1..65535 default 8080
  end
  plugins [string]
end
"#,
    )
    .expect("schema should parse");

    let config = RuneConfig::from_str(
        r#"
app:
  name "RuneApp"
  environment "prod"
  server:
    host "localhost"
    port "8080"
  end
  plugins ["auth", 42]
end
"#,
    )
    .expect("config should parse");

    let diagnostics = config.validate_schema(&schema);
    let messages: Vec<&str> = diagnostics.iter().map(|d| d.message.as_str()).collect();

    assert!(
        messages
            .iter()
            .any(|message| message.contains("version") && message.contains("Missing required"))
    );
    assert!(
        messages.iter().any(
            |message| message.contains("app.environment") && message.contains("must be one of")
        )
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("app.server.port") && message.contains("expected int"))
    );
    assert!(
        messages.iter().any(
            |message| message.contains("app.plugins[1]") && message.contains("expected string")
        )
    );
}

#[test]
fn test_schema_missing_field_points_to_parent_object() {
    let schema = SchemaDocument::from_str(
        r#"
schema app:
  version string required
end
"#,
    )
    .expect("schema should parse");

    let text = r#"
app:
  name "RuneApp"
end
"#;
    let config = RuneConfig::from_str(text).expect("config should parse");

    let diagnostics = config.validate_schema(&schema);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "Missing required field 'version' inside 'app'"
    );
    assert_eq!(diagnostics[0].code, Some(651));
    // The whole `app` key token: line 2, columns 1 through 4.
    let range = diagnostics[0].range.unwrap();
    assert_eq!(range.start.line, 2);
    assert_eq!(range.start.column, 1);
    assert_eq!(range.end.line, 2);
    assert_eq!(range.end.column, 4);
    assert_eq!(
        diagnostics[0].hint.as_deref(),
        Some("Add 'version' near: app:")
    );

    // Validating with an index the caller already built - the path the language
    // server takes - reports the very same diagnostic.
    let indexed = config.validate_schema_with_source(&schema, &SourceIndex::new(text));
    assert_eq!(indexed, diagnostics);
}

#[test]
fn test_schema_type_error_points_to_key_column() {
    let schema = SchemaDocument::from_str(
        r#"
schema app:
  server:
    port int required
  end
end
"#,
    )
    .expect("schema should parse");

    let text = r#"
app:
  server:
    port "8080"
  end
end
"#;
    let config = RuneConfig::from_str(text).expect("config should parse");

    let diagnostics = config.validate_schema(&schema);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        "'app.server.port' expected int, got string"
    );
    assert_eq!(diagnostics[0].code, Some(652));
    // The whole `port` key token: line 4, columns 5 through 9.
    let range = diagnostics[0].range.unwrap();
    assert_eq!(range.start.line, 4);
    assert_eq!(range.start.column, 5);
    assert_eq!(range.end.line, 4);
    assert_eq!(range.end.column, 9);
    assert_eq!(
        diagnostics[0].hint.as_deref(),
        Some("Check around: port \"8080\"")
    );

    // Validating with an index the caller already built - the path the language
    // server takes - reports the very same diagnostic.
    let indexed = config.validate_schema_with_source(&schema, &SourceIndex::new(text));
    assert_eq!(indexed, diagnostics);
}

#[test]
fn test_schema_validation_reports_range_error() {
    let schema = SchemaDocument::from_str(
        r#"
schema app:
  server:
    port int range 1..65535 required
  end
end
"#,
    )
    .expect("schema should parse");

    let config = RuneConfig::from_str(
        r#"
app:
  server:
    port 70000
  end
end
"#,
    )
    .expect("config should parse");

    let diagnostics = config.validate_schema(&schema);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("between 1 and 65535"));
}

#[test]
fn test_config_elseif_selects_first_matching_branch() {
    let config = RuneConfig::from_str(
        r#"
first false
second true
app:
  if first = true:
    value "first"
  elseif second = true:
    value "second"
  else:
    value "fallback"
  endif
end
"#,
    )
    .expect("elseif config should parse");

    let value: String = config.get("app.value").expect("selected branch value");
    assert_eq!(value, "second");
}

// ===== Resolution Memoization Tests =====

#[test]
fn test_repeated_access_is_consistent_after_memoization() {
    let config_content = r#"
mod_main "alt"

app:
  server:
    host "localhost"
    port 8080
  end
  keybinds:
    modifier $var.mod_main
  end
end
"#;

    let config = RuneConfig::from_str(config_content).expect("config should parse");

    // First access builds and caches the resolved root; subsequent accesses reuse it.
    // All must return identical values, and references must stay resolved.
    for _ in 0..5 {
        let host: String = config.get("app.server.host").expect("host");
        let port: u64 = config.get("app.server.port").expect("port");
        let modifier: String = config.get("app.keybinds.modifier").expect("modifier");
        assert_eq!(host, "localhost");
        assert_eq!(port, 8080);
        assert_eq!(modifier, "alt");
    }
}

#[test]
fn test_inject_import_invalidates_resolved_cache() {
    // Resolve once so the root is built and cached.
    let mut config =
        RuneConfig::from_str("name \"before\"\ngreeting $var.name\n").expect("config should parse");
    let before: String = config.get("greeting").expect("greeting");
    assert_eq!(before, "before");

    // Replace the main document with one that resolves differently. inject_import must drop
    // the memoized resolution so the next access reflects the new document.
    let replacement =
        RuneConfig::from_str("name \"after\"\ngreeting $var.name\n").expect("config should parse");
    let new_main = replacement.document().expect("main doc").clone();
    config.inject_import("main".to_string(), new_main);

    let after: String = config.get("greeting").expect("greeting");
    assert_eq!(after, "after");
}

// ===== Source-indexed schema validation =====
//
// Every diagnostic below is asserted as a complete range: the exact key token
// the source index reports, in 1-based UTF-16 lines and columns.

/// A parsed config plus the schema diagnostics for it, located through the
/// source index the language server builds for the same text.
fn indexed_schema_diagnostics(
    config_text: &str,
    schema_text: &str,
) -> (RuneConfig, Vec<RuneDiagnostic>) {
    let config = RuneConfig::from_str(config_text).expect("config should parse");
    let schema = SchemaDocument::from_str(schema_text).expect("schema should parse");
    let diagnostics = config.validate_schema_with_source(&schema, &SourceIndex::new(config_text));

    (config, diagnostics)
}

/// The one diagnostic whose message contains `needle`.
fn diagnostic_containing<'a>(
    diagnostics: &'a [RuneDiagnostic],
    needle: &str,
) -> &'a RuneDiagnostic {
    let matching: Vec<&RuneDiagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains(needle))
        .collect();

    assert_eq!(
        matching.len(),
        1,
        "expected exactly one diagnostic containing {needle:?}, got {diagnostics:#?}"
    );

    matching[0]
}

/// `(line, start column, end column)` of a diagnostic, all 1-based.
fn diagnostic_span(diagnostic: &RuneDiagnostic) -> (usize, usize, usize) {
    let range = diagnostic
        .range
        .unwrap_or_else(|| panic!("diagnostic has no range: {diagnostic:?}"));

    assert_eq!(
        range.start.line, range.end.line,
        "a key token never crosses a line: {diagnostic:?}"
    );

    (range.start.line, range.start.column, range.end.column)
}

#[test]
fn schema_diagnostic_points_at_the_active_elseif_occurrence() {
    let config_text = r#"first false
second true
app:
  if first = true:
    mode 100
  elseif second = true:
    mode "beta"
  else:
    mode 200
  endif
end
"#;
    let schema_text = r#"schema app:
  mode int required
end
"#;

    let (_, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    // `app.mode` is written in all three branches; only the elseif branch is
    // selected, so its own occurrence - line 7 - carries the diagnostic, and
    // the string value proves the elseif occurrence supplied the value too.
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostic_span(&diagnostics[0]), (7, 5, 9));
    assert_eq!(
        diagnostics[0].message,
        "'app.mode' expected int, got string"
    );
    assert_eq!(diagnostics[0].code, Some(652));
}

#[test]
fn schema_diagnostic_ignores_inactive_conditional_branches() {
    let config_text = r#"first true
app:
  if first = true:
    mode 100
  else:
    mode "beta"
  endif
end
"#;
    let schema_text = r#"schema app:
  mode int required
end
"#;

    let (_, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    // Control for the elseif case: the only invalid value sits in the branch the
    // condition did not select, and it must not be validated at all.
    assert_eq!(diagnostics, vec![], "inactive branches are not validated");
}

#[test]
fn schema_diagnostic_matches_the_full_path_not_the_leaf_name() {
    let config_text = r#"app:
  server:
    port "8080"
  end
  client:
    port 9090
  end
end
"#;
    let schema_text = r#"schema app:
  server:
    port int required
  end
  client:
    port string required
  end
end
"#;

    let (_, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    // Both objects hold a `port`; each diagnostic has to point at the
    // occurrence of its own full path.
    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
    assert_eq!(
        diagnostic_span(diagnostic_containing(&diagnostics, "app.server.port")),
        (3, 5, 9)
    );
    assert_eq!(
        diagnostic_span(diagnostic_containing(&diagnostics, "app.client.port")),
        (6, 5, 9)
    );
}

#[test]
fn schema_diagnostic_uses_the_first_effective_duplicate() {
    let config_text = r#"app:
  server:
    port "8080"
  end
  server:
    port 9090
  end
end
"#;
    let schema_text = r#"schema app:
  server:
    port int required
  end
end
"#;

    let (config, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    // Resolution keeps the first `server` object, so its `port` supplies the
    // value and the location; the later duplicate is shadowed.
    assert_eq!(
        config.get_value("app.server.port").unwrap(),
        Value::String("8080".to_string())
    );
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostic_span(&diagnostics[0]), (3, 5, 9));
    assert_eq!(
        diagnostics[0].message,
        "'app.server.port' expected int, got string"
    );
}

#[test]
fn missing_field_diagnostic_underlines_the_active_parent_occurrence() {
    let config_text = r#"app:
  if debug:
    server:
      host "inactive"
      port "inactive"
    end
  else:
    server:
      host 42
    end
  endif
end
"#;
    let schema_text = r#"schema app:
  server:
    host string required
    port int required
  end
end
"#;

    let (_, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    // The parent objects are duplicated across the two branches. `port` is
    // missing, so the diagnostic covers the selected `app.server` - line 8 -
    // and neither the inactive (line 3) nor the first textual key.
    let missing = diagnostic_containing(&diagnostics, "Missing required field");
    assert_eq!(
        missing.message,
        "Missing required field 'port' inside 'app.server'"
    );
    assert_eq!(missing.code, Some(651));
    assert_eq!(diagnostic_span(missing), (8, 5, 11));
    assert_eq!(missing.hint.as_deref(), Some("Add 'port' near: server:"));

    // The active `host` is the second occurrence of its path; the inactive one
    // (line 4) still advanced the pairing, so the type error covers line 9.
    let host = diagnostic_containing(&diagnostics, "app.server.host");
    assert_eq!(host.message, "'app.server.host' expected string, got int");
    assert_eq!(diagnostic_span(host), (9, 7, 11));
    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
}

#[test]
fn quoted_key_diagnostic_excludes_quotes_and_ignores_hash_in_value() {
    let config_text = r#"app:
  "port" "8080#1"
end
"#;
    let schema_text = r#"schema app:
  port int required
end
"#;

    let (_, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    // The range covers the decoded key inside its quotes, and the `#` of the
    // value neither moves the key nor truncates the hint line.
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostic_span(&diagnostics[0]), (2, 4, 8));
    assert_eq!(
        diagnostics[0].message,
        "'app.port' expected int, got string"
    );
    assert_eq!(
        diagnostics[0].hint.as_deref(),
        Some("Check around: \"port\" \"8080#1\"")
    );
}

#[test]
fn crlf_source_reports_the_same_range_as_lf() {
    let lf = "app:\n  server:\n    port \"8080\"\n  end\nend\n";
    let crlf = lf.replace('\n', "\r\n");
    let schema_text = r#"schema app:
  server:
    port int required
  end
end
"#;

    let (_, lf_diagnostics) = indexed_schema_diagnostics(lf, schema_text);
    let (_, crlf_diagnostics) = indexed_schema_diagnostics(&crlf, schema_text);

    assert_eq!(diagnostic_span(&lf_diagnostics[0]), (3, 5, 9));
    assert_eq!(diagnostic_span(&crlf_diagnostics[0]), (3, 5, 9));
    assert_eq!(
        crlf_diagnostics, lf_diagnostics,
        "a CRLF buffer reports the same line, columns, and hint as its LF form"
    );
}

#[test]
fn non_bmp_key_columns_count_utf16_code_units() {
    let config_text = "app:\n  \"\u{10400}name\" 42\nend\n";
    let schema_text = "schema app:\n  \u{10400}name string required\nend\n";

    let (_, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    // The key is one non-BMP scalar: two UTF-16 code units, four UTF-8 bytes.
    // Columns therefore run 4..10, not 4..12 (bytes) and not 4..9 (scalars).
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostic_span(&diagnostics[0]), (2, 4, 10));
    assert_eq!(
        diagnostics[0].message,
        "'app.\u{10400}name' expected string, got int"
    );
}

#[test]
fn snapshot_values_agree_with_get_value_for_conditionals_and_duplicates() {
    let config_text = r#"first false
second true
app:
  if first = true:
    mode 100
  elseif second = true:
    mode "beta"
  else:
    mode 200
  endif
  server:
    port 1
  end
  server:
    port 2
  end
end
"#;
    let schema_text = r#"schema app:
  mode int required
  server:
    port string required
  end
end
"#;

    let (config, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    // The values validation saw are the ones `get_value` reports: the selected
    // elseif branch and the first of the duplicate `server` objects.
    assert_eq!(
        config.get_value("app.mode").unwrap(),
        Value::String("beta".to_string())
    );
    assert_eq!(
        config.get_value("app.server.port").unwrap(),
        Value::Number(1.0)
    );

    // ... which is exactly what the diagnostics say, in both message and range.
    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");

    let mode = diagnostic_containing(&diagnostics, "app.mode");
    assert_eq!(mode.message, "'app.mode' expected int, got string");
    assert_eq!(diagnostic_span(mode), (7, 5, 9));

    let port = diagnostic_containing(&diagnostics, "app.server.port");
    assert_eq!(port.message, "'app.server.port' expected string, got int");
    assert_eq!(diagnostic_span(port), (12, 5, 9));
}

#[test]
fn unresolved_root_reports_the_schema_root_as_missing() {
    // A config whose root does not resolve makes `get_value` report every path
    // as missing. Validation keeps that behavior: it reports the schema root
    // instead of the fields that happen to resolve on their own.
    let config_text = r#"value $var.nope
app:
  name "RuneApp"
end
"#;
    let schema_text = r#"schema app:
  name string required
end
"#;

    let (config, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    assert!(config.get_value("app").is_err());
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(
        diagnostics[0].message,
        "Required schema root 'app' is missing"
    );
    assert_eq!(diagnostics[0].code, Some(650));
    assert_eq!(diagnostics[0].range, None);
}

#[test]
fn reference_produced_descendant_uses_the_nearest_source_backed_ancestor() {
    let config_text = r#"other:
  port "8080"
end
app:
  server $var.other
end
"#;
    let schema_text = r#"schema app:
  server:
    port int required
  end
end
"#;

    let (_, diagnostics) = indexed_schema_diagnostics(config_text, schema_text);

    // `app.server` comes from a reference, so `port` has no indexed occurrence
    // of its own; its diagnostic covers the assignment it lives in rather than
    // an unrelated `port` key elsewhere in the document.
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(
        diagnostics[0].message,
        "'app.server.port' expected int, got string"
    );
    assert_eq!(diagnostic_span(&diagnostics[0]), (5, 3, 9));
    assert_eq!(
        diagnostics[0].hint.as_deref(),
        Some("Check around: server $var.other")
    );
}
