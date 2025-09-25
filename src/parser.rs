use crate::lexer::{Lexer, Token};
use crate::RuneError;
use crate::ast::{Document, Value};
use std::collections::HashMap;
use std::env;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peek: Option<Token>,
    pub imports: HashMap<String, Document>, // store imported documents
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, RuneError> {
        let mut lexer = Lexer::new(input);
        let peek = Some(lexer.next_token()?);
        Ok(Self { lexer, peek, imports: HashMap::new() })
    }

    // Method to manually inject an import (useful for testing)
    pub fn inject_import(&mut self, alias: String, document: Document) {
        self.imports.insert(alias, document);
    }

    fn bump(&mut self) -> Result<Token, RuneError> {
        let curr = self.peek.take().ok_or(RuneError::UnexpectedEof {
            message: "Unexpected end of input".into(),
            line: self.lexer.line(),
            column: self.lexer.column(),
            hint: None,
            code: Some(201),
        })?;
        self.peek = Some(self.lexer.next_token()?);
        Ok(curr)
    }

    fn peek(&self) -> Option<&Token> {
        self.peek.as_ref()
    }

    #[allow(dead_code)]
    fn expect(&mut self, expected: Token) -> Result<Token, RuneError> {
        let token = self.bump()?;
        if token != expected {
            return Err(RuneError::SyntaxError {
                message: format!("Expected {:?}, got {:?}", expected, token),
                line: self.lexer.line(),
                column: self.lexer.column(),
                hint: Some("Check your syntax".into()),
                code: Some(202),
            });
        }
        Ok(token)
    }

    pub fn parse_document(&mut self) -> Result<Document, RuneError> {
        let mut metadata = Vec::new();
        let mut globals = Vec::new();
        let mut items = Vec::new();

        while let Some(tok) = self.peek() {
            match tok {
                Token::Newline => { self.bump()?; }
                Token::Eof => { break; }
                Token::At => {
                    self.bump()?;
                    if let Token::Ident(key) = self.bump()? {
                        let value = self.parse_value()?;
                        metadata.push((key, value));
                    } else {
                        return Err(RuneError::SyntaxError {
                            message: "Expected identifier after @".into(),
                            line: self.lexer.line(),
                            column: self.lexer.column(),
                            hint: None,
                            code: Some(203),
                        });
                    }
                }
                Token::Ident(_) => {
                    // Look ahead to see if this is a block (has colon) or assignment
                    let key = if let Token::Ident(k) = self.bump()? { k } else { unreachable!() };
                    
                    match self.peek() {
                        Some(Token::Colon) => {
                            // This is a block definition
                            self.bump()?; // consume colon
                            let mut object_items = Vec::new();

                            while let Some(tok) = self.peek() {
                                match tok {
                                    Token::Ident(_) => { object_items.push(self.parse_assignment()?); }
                                    Token::End => { self.bump()?; break; }
                                    Token::Newline => { self.bump()?; }
                                    _ => { return Err(RuneError::InvalidToken {
                                        token: format!("{:?}", tok),
                                        line: self.lexer.line(),
                                        column: self.lexer.column(),
                                        hint: Some("Expected key or 'end'".into()),
                                        code: Some(207),
                                    }); }
                                }
                            }
                            items.push((key, Value::Object(object_items)));
                        }
                        Some(Token::Equals) => {
                            // Explicit assignment with =
                            self.bump()?; // consume =
                            let value = self.parse_value()?;
                            globals.push((key, value));
                        }
                        _ => {
                            // Implicit assignment (no = needed)
                            let value = self.parse_value()?;
                            globals.push((key, value));
                        }
                    }
                }
                Token::Gather => {
                    self.bump()?;
                    let filename = if let Token::String(f) = self.bump()? { f } else {
                        return Err(RuneError::SyntaxError {
                            message: "Expected string after gather".into(),
                            line: self.lexer.line(),
                            column: self.lexer.column(),
                            hint: None,
                            code: Some(211),
                        });
                    };

                    let alias = if let Some(Token::As) = self.peek() {
                        self.bump()?;
                        if let Token::Ident(a) = self.bump()? { a } else {
                            return Err(RuneError::SyntaxError {
                                message: "Expected identifier after 'as'".into(),
                                line: self.lexer.line(),
                                column: self.lexer.column(),
                                hint: None,
                                code: Some(212),
                            });
                        }
                    } else { 
                        // Use filename without extension as default alias
                        filename.trim_end_matches(".rune").to_string()
                    };

                    // store imported alias with a placeholder document (to be replaced when loaded)
                    self.imports.insert(alias, Document { metadata: vec![], globals: vec![], items: vec![] });
                }
                Token::Dollar => {
                    return Err(RuneError::SyntaxError {
                        message: "Dollar variables ($env, $sys, $runtime) cannot be assigned at top level".into(),
                        line: self.lexer.line(),
                        column: self.lexer.column(),
                        hint: Some("Dollar variables can only be used as values, not as top-level definitions".into()),
                        code: Some(213),
                    });
                }
                _ => {
                    return Err(RuneError::InvalidToken {
                        token: format!("{:?}", tok),
                        line: self.lexer.line(),
                        column: self.lexer.column(),
                        hint: Some("Unexpected token at top-level".into()),
                        code: Some(205),
                    });
                }
            }
        }

        Ok(Document { metadata, globals, items })
    }

    /// Expands `$env.X`, `$sys.Y`, `$runtime.Z` inside a string
    fn expand_dollar_variables(&self, s: &str) -> Result<Value, RuneError> {
        let mut result = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                // start of a reference
                let mut path = Vec::new();

                // read namespace
                let mut ns = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ns.push(ch);
                        chars.next();
                    } else { break; }
                }

                if ns != "env" && ns != "sys" && ns != "runtime" {
                    return Err(RuneError::SyntaxError {
                        message: format!("Unknown namespace ${}", ns),
                        line: self.lexer.line(),
                        column: self.lexer.column(),
                        hint: Some("Use $env, $sys, or $runtime".into()),
                        code: Some(209),
                    });
                }

                path.push(ns);

                while let Some(&ch) = chars.peek() {
                    if ch == '.' {
                        chars.next(); // consume dot
                        let mut seg = String::new();
                        while let Some(&ch2) = chars.peek() {
                            if ch2.is_alphanumeric() || ch2 == '_' {
                                seg.push(ch2);
                                chars.next();
                            } else { break; }
                        }
                        if seg.is_empty() {
                            return Err(RuneError::SyntaxError {
                                message: "Expected identifier after '.'".into(),
                                line: self.lexer.line(),
                                column: self.lexer.column(),
                                hint: None,
                                code: Some(210),
                            });
                        }
                        path.push(seg);
                    } else { break; }
                }

                // --- Here is the change ---
                let expanded = if path[0] == "env" && path.len() == 2 {
                    env::var(&path[1]).unwrap_or_else(|_| "".to_string())
                } else {
                    format!("${{{}}}", path.join("."))
                };

                result.push(expanded);
            } else {
                result.push(c.to_string());
            }
        }

        Ok(Value::String(result.concat()))
    }

    fn parse_assignment(&mut self) -> Result<(String, Value), RuneError> {
        let key = if let Token::Ident(k) = self.bump()? { k } else {
            return Err(RuneError::SyntaxError {
                message: "Expected identifier for assignment".into(),
                line: self.lexer.line(),
                column: self.lexer.column(),
                hint: None,
                code: Some(208),
            });
        };

        match self.peek() {
            Some(Token::Colon) => {
                // Nested object
                self.bump()?;
                let mut items = Vec::new();
                while let Some(tok) = self.peek() {
                    match tok {
                        Token::Ident(_) => { items.push(self.parse_assignment()?); }
                        Token::End => { self.bump()?; break; }
                        Token::Newline => { self.bump()?; }
                        _ => { return Err(RuneError::InvalidToken {
                            token: format!("{:?}", tok),
                            line: self.lexer.line(),
                            column: self.lexer.column(),
                            hint: Some("Expected key or 'end'".into()),
                            code: Some(207),
                        }); }
                    }
                }
                return Ok((key, Value::Object(items)));
            }
            Some(Token::Equals) => { 
                // Explicit assignment with =
                self.bump()?; 
            }
            _ => {
                // Implicit assignment (no = needed)
            }
        }

        let value = self.parse_value()?;
        Ok((key, value))
    }

    fn parse_value(&mut self) -> Result<Value, RuneError> {
        match self.peek() {
            Some(Token::String(_)) => {
                if let Token::String(s) = self.bump()? {
                    // Expand $ references inside the string
                    self.expand_dollar_variables(&s)
                } else { unreachable!() }
            }
            Some(Token::Number(_)) => {
                if let Token::Number(n) = self.bump()? {
                    Ok(Value::Number(n))
                } else { unreachable!() }
            }
            Some(Token::Bool(_)) => {
                if let Token::Bool(b) = self.bump()? {
                    Ok(Value::Bool(b))
                } else { unreachable!() }
            }
            Some(Token::Dollar) => {
                self.bump()?; // consume $

                let namespace = if let Token::Ident(name) = self.bump()? {
                    if name != "env" && name != "sys" && name != "runtime" {
                        return Err(RuneError::SyntaxError {
                            message: format!("Unknown namespace ${}", name),
                            line: self.lexer.line(),
                            column: self.lexer.column(),
                            hint: Some("Use $env, $sys, or $runtime".into()),
                            code: Some(209),
                        });
                    }
                    name
                } else {
                    return Err(RuneError::SyntaxError {
                        message: "Expected identifier after $".into(),
                        line: self.lexer.line(),
                        column: self.lexer.column(),
                        hint: None,
                        code: Some(209),
                    });
                };

                let mut path = vec![namespace];

                // Handle dot notation for namespaced variables like $env.HOME
                while let Some(Token::Dot) = self.peek() {
                    self.bump()?; // consume .
                    if let Token::Ident(name) = self.bump()? {
                        path.push(name);
                    } else {
                        return Err(RuneError::SyntaxError {
                            message: "Expected identifier after '.'".into(),
                            line: self.lexer.line(),
                            column: self.lexer.column(),
                            hint: None,
                            code: Some(210),
                        });
                    }
                }

                Ok(Value::Reference(path))
            }
            Some(Token::Ident(_)) => {
                // Regular reference (could be import.path or local global variable)
                let mut path = Vec::new();
                
                if let Token::Ident(name) = self.bump()? {
                    path.push(name);
                } else { unreachable!() }

                // Handle dot notation for imports or nested references
                while let Some(Token::Dot) = self.peek() {
                    self.bump()?; // consume .
                    if let Token::Ident(name) = self.bump()? {
                        path.push(name);
                    } else {
                        return Err(RuneError::SyntaxError {
                            message: "Expected identifier after '.'".into(),
                            line: self.lexer.line(),
                            column: self.lexer.column(),
                            hint: None,
                            code: Some(210),
                        });
                    }
                }

                Ok(Value::Reference(path))
            }
            Some(Token::LBracket) => {
                self.bump()?; // consume [
                let mut arr = Vec::new();
                
                while let Some(tok) = self.peek() {
                    match tok {
                        Token::RBracket => { 
                            self.bump()?; // consume ]
                            break; 
                        }
                        Token::Newline => { 
                            self.bump()?; // skip newlines
                        }
                        _ => {
                            arr.push(self.parse_value()?);
                            // Commas are automatically skipped by the lexer
                        }
                    }
                }
                Ok(Value::Array(arr))
            }
            _ => {
                let token = self.bump()?;
                Err(RuneError::InvalidToken {
                    token: format!("{:?}", token),
                    line: self.lexer.line(),
                    column: self.lexer.column(),
                    hint: Some("Unexpected token in value position".into()),
                    code: Some(210),
                })
            }
        }
    }

    pub fn resolve_reference<'b>(&'b self, path: &[String], doc: &'b Document) -> Option<&'b Value> {
        if path.is_empty() { return None; }

        //println!("DEBUG: Resolving path: {:?}", path);
        //println!("DEBUG: Available imports: {:?}", self.imports.keys().collect::<Vec<_>>());

        // Check if first segment is an import alias
        let (current_doc, remaining_path): (&Document, &[String]) = {
            if let Some(import_doc) = self.imports.get(&path[0]) {
                //println!("DEBUG: Found import '{}', using imported doc", &path[0]);
                //println!("DEBUG: Imported doc items: {:?}", import_doc.items.iter().map(|(k, _)| k).collect::<Vec<_>>());
                // First segment is an import alias, use imported doc and skip first segment
                (import_doc, &path[1..])
            } else {
                //println!("DEBUG: No import found for '{}', using current doc", &path[0]);
                // Not an import alias, use current doc and full path
                (doc, path)
            }
        };

        if remaining_path.is_empty() { 
            //println!("DEBUG: Remaining path is empty");
            return None; 
        }

        //println!("DEBUG: Remaining path: {:?}", remaining_path);

        // Find the first segment in the current document
        let mut current: &Value = {
            let first_segment = &remaining_path[0];
            //println!("DEBUG: Looking for first segment: {}", first_segment);
            
            // First check items (top-level blocks/assignments)
            if let Some((_, v)) = current_doc.items.iter().find(|(k, _)| k == first_segment) {
                //println!("DEBUG: Found '{}' in items", first_segment);
                v
            }
            // Then check globals
            else if let Some((_, v)) = current_doc.globals.iter().find(|(k, _)| k == first_segment) {
                //println!("DEBUG: Found '{}' in globals", first_segment);
                v
            }
            // Not found
            else {
                //println!("DEBUG: First segment '{}' not found in document", first_segment);
                //println!("DEBUG: Available items: {:?}", current_doc.items.iter().map(|(k, _)| k).collect::<Vec<_>>());
                //println!("DEBUG: Available globals: {:?}", current_doc.globals.iter().map(|(k, _)| k).collect::<Vec<_>>());
                return None;
            }
        };

        //println!("DEBUG: Found first segment, value: {:?}", current);

        // Traverse the remaining path segments
        for seg in &remaining_path[1..] {
            //println!("DEBUG: Traversing segment: {}", seg);
            match current {
                Value::Object(items) => {
                    if let Some((_, v)) = items.iter().find(|(k, _)| k == seg) {
                        current = v;
                        //println!("DEBUG: Found segment '{}', new value: {:?}", seg, current);
                    } else {
                        //println!("DEBUG: Segment '{}' not found in object", seg);
                        //println!("DEBUG: Available keys in object: {:?}", items.iter().map(|(k, _)| k).collect::<Vec<_>>());
                        return None;
                    }
                }
                _ => {
                    //println!("DEBUG: Current value is not an object: {:?}", current);
                    return None;
                }
            }
        }

        //println!("DEBUG: Final resolved value: {:?}", current);
        Some(current)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
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

        // Debug print the entire document
        println!("--- Parsed Document ---");
        println!("{:#?}", doc);

        // Optional: assert some key values
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

        // Check that we have 2 globals
        assert_eq!(doc.globals.len(), 2);
        
        // Check the app block
        if let Value::Object(items) = &doc.items[0].1 {
            // name should reference app_name (simple reference)
            if let Value::Reference(path) = &items.iter().find(|(k, _)| k == "name").unwrap().1 {
                assert_eq!(path, &["app_name".to_string()]);
            } else {
                panic!("Expected 'name' to be a Reference");
            }
            
            // port should reference port (simple reference)
            if let Value::Reference(path) = &items.iter().find(|(k, _)| k == "port").unwrap().1 {
                assert_eq!(path, &["port".to_string()]);
            } else {
                panic!("Expected 'port' to be a Reference");
            }
            
            // env_var should be a namespaced reference to $env.HOME
            if let Value::Reference(path) = &items.iter().find(|(k, _)| k == "env_var").unwrap().1 {
                assert_eq!(path, &["env".to_string(), "HOME".to_string()]);
            } else {
                panic!("Expected 'env_var' to be a Reference");
            }
        } else {
            panic!("Expected 'app' to be an Object");
        }
    }
}

#[cfg(test)]
mod dot_reference_tests {
    use super::*;
    use crate::ast::Value;

    #[test]
    fn test_dot_notation_and_imported_variables() {
        // Mock the contents of the imported "defaults.rune" file
        let defaults_input = r#"
server:
  host "localhost"
  port 8000
end
"#;

        // Parse the defaults document
        let mut defaults_parser = Parser::new(defaults_input).expect("Failed to create parser");
        let defaults_doc = defaults_parser.parse_document().expect("Failed to parse defaults");

        // Main input that uses the imported defaults
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
        
        // Inject the defaults document AFTER parsing to replace the placeholder
        parser.inject_import("defaults".to_string(), defaults_doc);

        println!("--- Parsed Main Document ---");
        println!("{:#?}", doc);

        // Check metadata and globals
        assert_eq!(doc.metadata.len(), 1);
        assert_eq!(doc.globals.len(), 1);

        // Resolve references
        if let Value::Object(items) = &doc.items[0].1 {
            // Check 'name' reference
            let name_ref = items.iter().find(|(k, _)| k == "name").unwrap().1.clone();
            match name_ref {
                Value::Reference(path) => {
                    assert_eq!(path, &["name".to_string()]);
                },
                _ => panic!("Expected 'name' to be a Reference"),
            }

            // Check 'server.host' resolves from defaults 
            if let Some(server_items) = items
                .iter()
                .find(|(k, _)| k == "server")
                .and_then(|(_, v)| v.as_object())
            {
                if let Value::Reference(path) = &server_items.iter().find(|(k, _)| k == "host").unwrap().1 {
                    // Check the path
                    assert_eq!(path, &["defaults".to_string(), "server".to_string(), "host".to_string()]);

                    // Resolve reference
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

    // Top-level global
    assert!(matches!(doc.globals[0].1, Value::Array(ref v) if v.is_empty()));

    // Nested object
    if let Value::Object(items) = &doc.items[0].1 {
        let arr = items.iter().find(|(k, _)| k == "things").unwrap().1.clone();
        assert!(matches!(arr, Value::Array(ref v) if v.is_empty()));
    } else {
        panic!("Expected 'nested' to be an Object");
    }
}
