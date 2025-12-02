#[cfg(test)]
use super::*;
#[cfg(test)]
use crate::ast::Value;

#[test]
fn test_parser_basic_document() {
    let input = r#"
@meta "version1"
global_name "GlobalApp"

app:
  name global_name
  version "1.0.0"
end
"#;

    let mut parser = Parser::new(input).expect("Failed to create parser");
    let doc = parser.parse_document().expect("Failed to parse document");

    println!("--- Parsed Document ---");
    println!("{:#?}", doc);

    assert_eq!(doc.metadata.len(), 1);
    assert_eq!(doc.globals.len(), 1);
    assert_eq!(doc.items.len(), 1);

    if let Value::Object(items) = &doc.items[0].1 {
        assert!(items.iter().any(|(k, _)| k == "name"));
        assert!(items.iter().any(|(k, _)| k == "version"));
    } else {
        panic!("Expected top-level 'app' to be an object");
    }
}

#[test]
fn test_parser_with_array_and_reference() {
    let input = r#"
servers:
  hosts [
    "host1"
    "host2"
  ]
  default default_host
end
"#;

    let mut parser = Parser::new(input).expect("Failed to create parser");
    let doc = parser.parse_document().expect("Failed to parse document");

    println!("--- Parsed Document with Array ---");
    println!("{:#?}", doc);

    if let Value::Object(items) = &doc.items[0].1 {
        let hosts_val = items.iter().find(|(k, _)| k == "hosts").unwrap().1.clone();
        match hosts_val {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
            }
            _ => panic!("Expected 'hosts' to be an array"),
        }
    }
}

#[test]
fn test_global_variable_references() {
    let input = r#"
app_name "MyApp"
port 8080

app:
  name app_name
  port port
  env_var $env.HOME
end
"#;

    let mut parser = Parser::new(input).expect("Failed to create parser");
    let doc = parser.parse_document().expect("Failed to parse document");

    println!("--- Document with Global References ---");
    println!("{:#?}", doc);

    assert_eq!(doc.globals.len(), 2);
    
    if let Value::Object(items) = &doc.items[0].1 {
        if let Value::Reference(path) = &items.iter().find(|(k, _)| k == "name").unwrap().1 {
            assert_eq!(path, &["app_name".to_string()]);
        } else {
            panic!("Expected 'name' to be a Reference");
        }
        
        if let Value::Reference(path) = &items.iter().find(|(k, _)| k == "port").unwrap().1 {
            assert_eq!(path, &["port".to_string()]);
        } else {
            panic!("Expected 'port' to be a Reference");
        }
        
        if let Value::String(_) = &items.iter().find(|(k, _)| k == "env_var").unwrap().1 {
            println!("env_var correctly resolved to a String");
        } else {
            panic!("Expected 'env_var' to be a String (resolved from $env.HOME)");
        }
    } else {
        panic!("Expected 'app' to be an Object");
    }
}

#[test]
fn test_dot_notation_and_imported_variables() {
    let defaults_input = r#"
server:
  host "localhost"
  port 8000
end
"#;

    let mut defaults_parser = Parser::new(defaults_input).expect("Failed to create parser");
    let defaults_doc = defaults_parser.parse_document().expect("Failed to parse defaults");

    let input = r#"
gather "defaults.rune" as defaults
@description "Simple app using RUNE config"
name "RuneApp"

app:
  name name
  version "1.0.0"
  debug true

  server:
    host defaults.server.host
    port 8080
    timeout "30s"
  end

  plugins [
    "auth"
    "logger"
  ]
end
"#;

    let mut parser = Parser::new(input).expect("Failed to create parser");
    let doc = parser.parse_document().expect("Failed to parse main document");
    
    parser.inject_import("defaults".to_string(), defaults_doc);

    println!("--- Parsed Main Document ---");
    println!("{:#?}", doc);

    assert_eq!(doc.metadata.len(), 1);
    assert_eq!(doc.globals.len(), 1);

    if let Value::Object(items) = &doc.items[0].1 {
        let name_ref = items.iter().find(|(k, _)| k == "name").unwrap().1.clone();
        match name_ref {
            Value::Reference(path) => {
                assert_eq!(path, &["name".to_string()]);
            },
            _ => panic!("Expected 'name' to be a Reference"),
        }

        if let Some(server_items) = items
            .iter()
            .find(|(k, _)| k == "server")
            .and_then(|(_, v)| v.as_object())
        {
            if let Value::Reference(path) = &server_items.iter().find(|(k, _)| k == "host").unwrap().1 {
                assert_eq!(path, &["defaults".to_string(), "server".to_string(), "host".to_string()]);

                let resolved = parser.resolve_reference(path, &doc)
                    .expect("Failed to resolve reference");

                if let Value::String(s) = resolved {
                    assert_eq!(s, "localhost");
                } else {
                    panic!("Expected resolved value to be a string");
                }
            } else {
                panic!("Expected 'server.host' to be a Reference");
            }
        } else {
            panic!("Expected 'server' to be an Object");
        }
    } else {
        panic!("Expected top-level 'app' to be an Object");
    }
}

#[test]
fn test_empty_array() {
    let input = r#"
list []
nested:
  things []
end
"#;

    let mut parser = Parser::new(input).expect("Failed to create parser");
    let doc = parser.parse_document().expect("Failed to parse document");

    println!("--- Parsed Document with Empty Arrays ---");
    println!("{:#?}", doc);

    assert!(matches!(doc.globals[0].1, Value::Array(ref v) if v.is_empty()));

    if let Value::Object(items) = &doc.items[0].1 {
        let arr = items.iter().find(|(k, _)| k == "things").unwrap().1.clone();
        assert!(matches!(arr, Value::Array(ref v) if v.is_empty()));
    } else {
        panic!("Expected 'nested' to be an Object");
    }
}

#[test]
fn test_parse_regex_literal() {
    let input = r#"
pattern r"^foo.*bar$"
"#;

    let mut parser = Parser::new(input).expect("Failed to create parser");
    let doc = parser.parse_document().expect("Failed to parse doc");

    let val = &doc.globals[0].1;
    assert_eq!(val, &Value::Regex("^foo.*bar$".into()));
}
