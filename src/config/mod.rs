use std::fs;
use std::path::Path;
use indexmap::IndexMap;

use crate::ast::{Document, Value};
use crate::parser;
use crate::RuneError;

mod access;
mod validation;
mod conversion;
mod helpers;

/// Main configuration struct that holds parsed RUNE documents and handles resolution
pub struct RuneConfig {
    documents: IndexMap<String, Document>,
    main_doc_key: String,
    raw_content: String, // Store for error reporting
}

impl RuneConfig {
    /// Load a RUNE config file and automatically resolve imports from the same directory
    /// 
    /// # Example
    /// ```ignore
    /// let config = RuneConfig::from_file("config.rune")?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RuneError> {
        let path_ref = path.as_ref();
        
        // Auto-detect base directory for imports (same directory as the config file)
        let base_dir = path_ref.parent()
            .unwrap_or_else(|| Path::new("."));
        
        Self::from_file_with_base(path_ref, base_dir)
    }

    /// Load a RUNE config file with fallback support
    /// 
    /// Tries to load from the primary path first. If that fails (file not found),
    /// attempts to load from the fallback path.
    /// 
    /// # Example
    /// ```ignore
    /// let config = RuneConfig::from_file_with_fallback(
    ///     "~/.config/myapp/config.rune",
    ///     "/etc/myapp/config.rune"
    /// )?;
    /// ```
    pub fn from_file_with_fallback<P: AsRef<Path>>(primary: P, fallback: P) -> Result<Self, RuneError> {
        match Self::from_file(&primary) {
            Ok(config) => Ok(config),
            Err(RuneError::FileError { .. }) => {
                // Primary file not found, try fallback
                Self::from_file(&fallback).map_err(|e| {
                    // Enhance error to mention both paths
                    match e {
                        RuneError::FileError { message, .. } => RuneError::FileError {
                            message: format!(
                                "Failed to load config from primary path '{}' or fallback path '{}': {}",
                                primary.as_ref().display(),
                                fallback.as_ref().display(),
                                message
                            ),
                            path: format!(
                                "{} (fallback: {})",
                                primary.as_ref().display(),
                                fallback.as_ref().display()
                            ),
                            hint: Some("Check that at least one of the config files exists".into()),
                            code: Some(301),
                        },
                        other => other,
                    }
                })
            }
            Err(other) => Err(other), // Pass through non-file errors
        }
    }

    /// Load a RUNE config file and resolve imports from a specific base directory
    /// 
    /// # Example
    /// ```ignore
    /// let config = RuneConfig::from_file_with_base("config.rune", "/etc/myapp")?;
    /// ```
    pub fn from_file_with_base<P: AsRef<Path>>(path: P, base_dir: P) -> Result<Self, RuneError> {
        let content = fs::read_to_string(&path).map_err(|e| RuneError::FileError {
            message: format!("Failed to read file: {}", e),
            path: path.as_ref().to_string_lossy().to_string(),
            hint: Some("Check that the file exists and is readable".into()),
            code: Some(301),
        })?;

        let mut parser = parser::Parser::new(&content)?;
        let main_doc = parser.parse_document()?;
        
        // Parse gather statements to get actual file paths
        let gather_paths = helpers::parse_gather_paths(&content);
        
        // Load imported documents
        let parser_aliases: Vec<String> = parser.imports.keys().cloned().collect();
        
        for parser_alias in parser_aliases {
            // Find the actual file path for this import
            let (import_path, proper_alias) = if let Some(raw_path) = gather_paths.get(&parser_alias) {
                // Found by matching alias
                (helpers::resolve_path(raw_path, base_dir.as_ref()), parser_alias.clone())
            } else {
                // Parser might have used the raw path as key - search gather_paths for it
                if let Some((alias, raw_path)) = gather_paths.iter().find(|(_, rp)| **rp == parser_alias) {
                    (helpers::resolve_path(raw_path, base_dir.as_ref()), alias.clone())
                } else {
                    // Fallback: treat parser_alias as filename
                    (base_dir.as_ref().join(format!("{}.rune", &parser_alias)), parser_alias.clone())
                }
            };
            
            if import_path.exists() {
                let import_content = fs::read_to_string(&import_path).map_err(|e| RuneError::FileError {
                    message: format!("Failed to read import file: {}", e),
                    path: import_path.to_string_lossy().to_string(),
                    hint: Some("Check that the imported file exists".into()),
                    code: Some(302),
                })?;
                
                let mut import_parser = parser::Parser::new(&import_content)?;
                let import_doc = import_parser.parse_document()?;
                
                // Inject with the proper alias (not the raw path)
                parser.inject_import(proper_alias, import_doc);
            }
        }

        let mut documents = IndexMap::new();
        let main_key = "main".to_string();
        
        documents.insert(main_key.clone(), main_doc);
        for (alias, doc) in parser.imports {
            documents.insert(alias, doc);
        }
        
        Ok(Self {
            documents,
            main_doc_key: main_key,
            raw_content: content.to_string(),
        })
    }

    /// Parse a RUNE config from a string (no file I/O, no import resolution)
    /// 
    /// # Example
    /// ```ignore
    /// let config = RuneConfig::from_str(r#"
    ///     app_name "MyApp"
    ///     version "1.0"
    /// "#)?;
    /// ```
    pub fn from_str(content: &str) -> Result<Self, RuneError> {
        let mut parser = parser::Parser::new(content)?;
        let main_doc = parser.parse_document()?;
        
        let mut documents = IndexMap::new();
        let main_key = "main".to_string();
        
        documents.insert(main_key.clone(), main_doc);
        
        Ok(Self {
            documents,
            main_doc_key: main_key,
            raw_content: content.to_string(),
        })
    }

    pub fn document(&self) -> Option<&Document> {
        self.documents.get(&self.main_doc_key)
    }

    pub fn all_documents(&self) -> &IndexMap<String, Document> {
        &self.documents
    }

    pub fn inject_import(&mut self, alias: String, document: Document) {
        self.documents.insert(alias, document);
    }
   
    pub fn import_aliases(&self) -> Vec<String> {
        self.documents
            .keys()
            .filter(|k| *k != &self.main_doc_key)
            .cloned()
            .collect()
    }    
    
    pub fn has_document(&self, name: &str) -> bool {
        self.documents.contains_key(name)
    }

    pub fn get_document(&self, name: &str) -> Option<&Document> {
        self.documents.get(name)
    }
}

#[cfg(test)]
mod tests;
