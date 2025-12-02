#[cfg(test)]
use super::*;

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
