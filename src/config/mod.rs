// Author: Dustin Pilgrim
// License: MIT

use std::fs;
use std::path::{Path, PathBuf};

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
        let base_dir = path_ref.parent().unwrap_or_else(|| Path::new("."));

        Self::from_file_with_base(path_ref, base_dir)
    }

    /// Load a RUNE config file with fallback support
    ///
    /// Tries to load from the primary path first. If that fails (file not found),
    /// attempts to load from the fallback path.
    pub fn from_file_with_fallback<P: AsRef<Path>>(primary: P, fallback: P) -> Result<Self, RuneError> {
        match Self::from_file(&primary) {
            Ok(config) => Ok(config),
            Err(RuneError::FileError { .. }) => {
                // Primary file not found, try fallback
                Self::from_file(&fallback).map_err(|e| match e {
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
                })
            }
            Err(other) => Err(other), // Pass through non-file errors
        }
    }

    /// Load a RUNE config file and resolve imports from a specific base directory
    ///
    /// Semantics:
    /// - `gather "file.rune"` (NO `as`) behaves like **include**:
    ///     - the gathered file's *globals + items* are merged into the main document
    ///     - and the gathered file is also available under its default alias (file stem)
    /// - `gather "file.rune" as alias` behaves like a **namespaced import** only
    pub fn from_file_with_base<P: AsRef<Path>>(path: P, base_dir: P) -> Result<Self, RuneError> {
        use std::collections::HashSet;

        let content = fs::read_to_string(&path).map_err(|e| RuneError::FileError {
            message: format!("Failed to read file: {}", e),
            path: path.as_ref().to_string_lossy().to_string(),
            hint: Some("Check that the file exists and is readable".into()),
            code: Some(301),
        })?;

        // Parse main doc (gather statements are parsed for alias discovery, but loading is done here)
        let mut main_parser = parser::Parser::new(&content)?;
        let main_doc = main_parser.parse_document()?;

        // Start documents with the main doc
        let mut documents = IndexMap::new();
        let main_key = "main".to_string();
        documents.insert(main_key.clone(), main_doc);

        // Parse gather specs (alias + path + whether alias was explicit)
        let gather_specs = helpers::parse_gather_specs(&content);

        // Prevent import cycles / repeated loads (by absolute import path string)
        let mut visited: HashSet<String> = HashSet::new();

        // Load each gathered file, recursively resolving nested gathers
        for spec in gather_specs.iter() {
            let import_path = resolve_gather_path(&spec.raw_path, base_dir.as_ref())?;

            // Keep existing behavior: silently skip missing imports
            if !import_path.exists() {
                continue;
            }

            // Load under its alias (overwrites placeholder)
            load_import_recursive(&mut documents, &spec.alias, &import_path, &mut visited)?;

            // If no explicit `as`, treat as include: merge into main doc too.
            if !spec.explicit_alias {
                // Clone after load to avoid borrow issues (and keep ordering predictable)
                let imported = documents.get(&spec.alias).cloned();
                if let Some(import_doc) = imported {
                    if let Some(main_doc_mut) = documents.get_mut(&main_key) {
                        // Include semantics: bring assignments into main.
                        // NOTE: duplicates are okay; later resolution is "first match wins" in current resolver,
                        // so we append imported values AFTER main? We want MAIN to win, so append imported FIRST.
                        // We'll *prepend* by rebuilding vectors.
                        let mut new_globals = import_doc.globals.clone();
                        new_globals.extend(main_doc_mut.globals.clone());
                        main_doc_mut.globals = new_globals;

                        let mut new_items = import_doc.items.clone();
                        new_items.extend(main_doc_mut.items.clone());
                        main_doc_mut.items = new_items;
                    }
                }
            }
        }

        Ok(Self {
            documents,
            main_doc_key: main_key,
            raw_content: content,
        })
    }

    /// Parse a RUNE config from a string (no file I/O, no import resolution)
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

/// Expand "~/" and resolve relative paths against base_dir.
fn resolve_gather_path(raw_path: &str, base_dir: &Path) -> Result<PathBuf, RuneError> {
    let mut p = if let Some(rest) = raw_path.strip_prefix("~/") {
        let home = dirs::home_dir().ok_or_else(|| RuneError::FileError {
            message: "Could not determine home directory for ~ expansion".into(),
            path: raw_path.to_string(),
            hint: Some("Set HOME or use an absolute path in gather".into()),
            code: Some(300),
        })?;
        home.join(rest)
    } else {
        PathBuf::from(raw_path)
    };

    if p.is_relative() {
        p = base_dir.join(p);
    }
    Ok(p)
}

/// Load an import file, parse its doc, inject into `documents` under `alias`,
/// then recursively load that file’s gathers.
///
/// NOTE: this always loads nested gathers as *namespaced imports* (per their alias).
/// Include/merge semantics are handled only at the top-level loader.
fn load_import_recursive(
    documents: &mut IndexMap<String, Document>,
    alias: &str,
    import_path: &Path,
    visited: &mut std::collections::HashSet<String>,
) -> Result<(), RuneError> {
    let key = import_path.to_string_lossy().to_string();
    if visited.contains(&key) {
        return Ok(());
    }
    visited.insert(key);

    let import_content = fs::read_to_string(import_path).map_err(|e| RuneError::FileError {
        message: format!("Failed to read import file: {}", e),
        path: import_path.to_string_lossy().to_string(),
        hint: Some("Check that the imported file exists".into()),
        code: Some(302),
    })?;

    let mut import_parser = parser::Parser::new(&import_content)?;
    let import_doc = import_parser.parse_document()?;

    // Overwrite any placeholder and/or previous doc with the real parsed doc
    documents.insert(alias.to_string(), import_doc);

    // Recurse into nested gathers
    let nested_specs = helpers::parse_gather_specs(&import_content);
    let nested_base = import_path.parent().unwrap_or_else(|| Path::new("."));

    for spec in nested_specs.iter() {
        let nested_path = resolve_gather_path(&spec.raw_path, nested_base)?;
        if !nested_path.exists() {
            continue;
        }

        // Nested gathers: keep them as namespaced imports only.
        load_import_recursive(documents, &spec.alias, &nested_path, visited)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
