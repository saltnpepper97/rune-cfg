// Author: Dustin Pilgrim
// License: MIT

#[cfg(test)]
use super::*;
use std::collections::HashMap;

use crate::ast::ObjectItem;

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
    assert_eq!(debug, true);

    let features: Vec<String> = config.get("app.features").expect("Failed to get features");
    assert_eq!(features, vec!["auth", "logging"]);

    assert!(config.has("app.name"));
    assert!(!config.has("app.nonexistent"));

    let server_keys = config.get_keys("app.server").expect("Failed to get server keys");
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
    let value = Value::Number(3.14);
    let result: Result<f64, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3.14);
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
    assert_eq!(result.unwrap(), true);

    let value = Value::Bool(false);
    let result: Result<bool, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
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
    let value = Value::Array(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]);

    let result: Result<Vec<i32>, RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_vec_bool_conversion() {
    let value = Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)]);

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
    let value = Value::Array(vec![Value::String("key".to_string()), Value::String("value".to_string())]);

    let result: Result<(String, String), RuneError> = value.try_into();
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        ("key".to_string(), "value".to_string())
    );
}

#[test]
fn test_tuple_string_value_conversion() {
    let value = Value::Array(vec![Value::String("config".to_string()), Value::Number(42.0)]);

    let result: Result<(String, Value), RuneError> = value.try_into();
    assert!(result.is_ok());
    let (key, val) = result.unwrap();
    assert_eq!(key, "config");
    assert_eq!(val, Value::Number(42.0));
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
    float_val 3.14
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
    assert!((f - 3.14).abs() < 0.001);

    let b: bool = config.get("types.bool_val").unwrap();
    assert_eq!(b, true);

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
    assert_eq!(flag, false);
}
