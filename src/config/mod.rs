// Author: Dustin Pilgrim
// License: MIT

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::RuneError;
use crate::ast::{Document, ObjectItem, Value};
use crate::parser;

mod access;
mod conversion;
mod helpers;
mod validation;

/// What a provider knows about one `gather` target.
///
/// The provider is the only thing the loader asks about the outside world, so
/// the same algorithm serves the runtime's filesystem reads and the language
/// server's editor-buffer-over-disk snapshot.
#[derive(Debug, Clone)]
pub(crate) enum ImportSource {
    /// The target's text.
    Text(String),
    /// The target is not there. The runtime skips it silently.
    Missing,
    /// The target is there but its text cannot be read.
    Unreadable(String),
}

/// One `gather` that could not be loaded, in runtime traversal order.
#[derive(Debug)]
pub(crate) struct ImportFailure {
    /// The path exactly as the statement wrote it.
    pub(crate) raw_path: String,
    /// The path the loader resolved it to, which is the file the error names.
    pub(crate) resolved_path: PathBuf,
    /// The error `RuneConfig::from_file_with_base` reports for it.
    pub(crate) error: RuneError,
}

impl ImportFailure {
    fn new(raw_path: &str, resolved_path: Option<PathBuf>, error: RuneError) -> Self {
        Self {
            raw_path: raw_path.to_string(),
            resolved_path: resolved_path.unwrap_or_else(|| PathBuf::from(raw_path)),
            error,
        }
    }
}

/// A config, plus every import that failed while it was loaded.
///
/// The runtime treats a failure as fatal and the language server reports it as
/// a diagnostic, so the failure is carried out of the loader rather than
/// turned into a partial config on the inside.
pub(crate) struct LoadedConfig {
    pub(crate) config: RuneConfig,
    /// Empty for a config whose imports all loaded.
    pub(crate) failures: Vec<ImportFailure>,
}

/// Whether one `gather` target contributed a document.
enum GatherLoad {
    Loaded,
    /// The target is not there, which is not an error.
    Missing,
}

/// Main configuration struct that holds parsed RUNE documents and handles resolution
pub struct RuneConfig {
    documents: IndexMap<String, Document>,
    main_doc_key: String,
    raw_content: String, // Store for error reporting
    /// Memoized fully-resolved root object (references, conditionals, `$env`/`$sys`/`var`,
    /// and inline/block `if` all flattened). Built lazily on first value access and reused
    /// for every subsequent lookup, since `documents` is immutable for an instance except
    /// via `inject_import` (which resets this).
    ///
    /// Note: `$env` references are resolved via the process environment at first access and
    /// then frozen for the lifetime of this instance. This is intentional — callers rebuild a
    /// fresh `RuneConfig` when the underlying files change.
    resolved_root: std::sync::OnceLock<Value>,
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
    pub fn from_file_with_fallback<P: AsRef<Path>>(
        primary: P,
        fallback: P,
    ) -> Result<Self, RuneError> {
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
        let content = fs::read_to_string(&path).map_err(|e| RuneError::FileError {
            message: format!("Failed to read file: {}", e),
            path: path.as_ref().to_string_lossy().to_string(),
            hint: Some("Check that the file exists and is readable".into()),
            code: Some(301),
        })?;

        let loaded = Self::from_text_with_provider(&content, base_dir.as_ref(), &mut |import| {
            filesystem_import(import)
        })?;

        // The runtime has always reported the first import that could not be
        // loaded as an error, and it still does.
        match loaded.failures.into_iter().next() {
            Some(failure) => Err(failure.error),
            None => Ok(loaded.config),
        }
    }

    /// Build a config from `content` and the imports a provider resolves.
    ///
    /// This is the only implementation of gather loading. The public
    /// filesystem-backed constructor is a wrapper over it, and the language
    /// server calls it with its own snapshot, so the editor's view of a
    /// config cannot drift from the runtime's.
    ///
    /// Loading stops at the first failure, exactly as the runtime does: the
    /// config is then as far as the loader got, and `failures` names the
    /// import that stopped it. A root that does not parse is still `Err`.
    pub(crate) fn from_text_with_provider(
        content: &str,
        base_dir: &Path,
        provider: &mut dyn FnMut(&Path) -> ImportSource,
    ) -> Result<LoadedConfig, RuneError> {
        // Parse main doc (gather statements are parsed for alias discovery, but loading is done here)
        let mut main_parser = parser::Parser::new(content)?;
        let main_doc = main_parser.parse_document()?;

        // Start documents with the main doc
        let mut documents = IndexMap::new();
        let main_key = "main".to_string();
        documents.insert(main_key.clone(), main_doc);

        // Parse gather specs (alias + path + whether alias was explicit)
        let gather_specs = helpers::parse_gather_specs(content);

        // Prevent import cycles / repeated loads (by absolute import path string)
        let mut visited: HashSet<String> = HashSet::new();

        // Load each gathered file, recursively resolving nested gathers
        let mut failures = Vec::new();

        for spec in gather_specs.iter() {
            let import_path = match resolve_gather_path(&spec.raw_path, base_dir) {
                Ok(path) => path,
                Err(error) => {
                    failures.push(ImportFailure::new(&spec.raw_path, None, error));
                    break;
                }
            };

            match load_gather(&mut documents, spec, &import_path, &mut visited, provider) {
                Ok(GatherLoad::Loaded) => {
                    // If no explicit `as`, treat as include: merge into main doc too.
                    if !spec.explicit_alias {
                        // Clone after load to avoid borrow issues (and keep ordering predictable)
                        let imported = documents.get(&spec.alias).cloned();
                        if let Some(import_doc) = imported
                            && let Some(main_doc_mut) = documents.get_mut(&main_key)
                        {
                            merge_overrides_into_document(main_doc_mut, &import_doc);
                        }
                    }
                }
                // Keep existing behavior: silently skip missing imports
                Ok(GatherLoad::Missing) => {}
                Err(failure) => {
                    failures.push(*failure);
                    break;
                }
            }
        }

        Ok(LoadedConfig {
            config: Self {
                documents,
                main_doc_key: main_key,
                raw_content: content.to_string(),
                resolved_root: std::sync::OnceLock::new(),
            },
            failures,
        })
    }

    /// Parse a RUNE config from a string (no file I/O, no import resolution)
    ///
    /// The inherent name is retained for compatibility; FromStr is also
    /// implemented below for callers that prefer the standard trait.
    #[allow(clippy::should_implement_trait)]
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
            resolved_root: std::sync::OnceLock::new(),
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
        // Documents changed: drop the memoized resolution so it rebuilds on next access.
        self.resolved_root = std::sync::OnceLock::new();
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

fn merge_overrides_into_document(target: &mut Document, overrides: &Document) {
    merge_named_values(&mut target.globals, &overrides.globals);
    merge_named_values(&mut target.items, &overrides.items);
}

fn merge_named_values(target: &mut Vec<(String, Value)>, overrides: &[(String, Value)]) {
    for (override_key, override_value) in overrides {
        let Some((_, target_value)) = target.iter_mut().find(|(key, _)| key == override_key) else {
            target.push((override_key.clone(), override_value.clone()));
            continue;
        };

        if let (Value::Object(target_items), Value::Object(override_items)) =
            (&mut *target_value, override_value)
        {
            merge_object_items(target_items, override_items);
        } else {
            *target_value = override_value.clone();
        }
    }
}

fn merge_object_items(target: &mut Vec<ObjectItem>, overrides: &[ObjectItem]) {
    for override_item in overrides {
        let ObjectItem::Assign(override_key, override_value) = override_item else {
            target.push(override_item.clone());
            continue;
        };

        let Some(ObjectItem::Assign(_, target_value)) = target
            .iter_mut()
            .find(|item| matches!(item, ObjectItem::Assign(key, _) if key == override_key))
        else {
            target.push(override_item.clone());
            continue;
        };

        if let (Value::Object(target_items), Value::Object(override_items)) =
            (&mut *target_value, override_value)
        {
            merge_object_items(target_items, override_items);
        } else {
            *target_value = override_value.clone();
        }
    }
}

/// Expand "~/" and resolve relative paths against base_dir.
fn resolve_gather_path(raw_path: &str, base_dir: &Path) -> Result<PathBuf, RuneError> {
    let mut p = if let Some(rest) = raw_path.strip_prefix("~/") {
        let home = home_dir_fallback().ok_or_else(|| RuneError::FileError {
            message: "Could not determine home directory for ~ expansion".into(),
            path: raw_path.to_string(),
            hint: Some(
                "Set HOME (or USERPROFILE on Windows) or use an absolute path in gather".into(),
            ),
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

impl std::str::FromStr for RuneConfig {
    type Err = RuneError;

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        RuneConfig::from_str(content)
    }
}

/// Best-effort home directory lookup without external crates.
fn home_dir_fallback() -> Option<PathBuf> {
    // Unix-like: HOME
    if let Some(home) = std::env::var_os("HOME")
        && !home.is_empty()
    {
        return Some(PathBuf::from(home));
    }

    // Windows: USERPROFILE, or HOMEDRIVE + HOMEPATH
    if let Some(up) = std::env::var_os("USERPROFILE")
        && !up.is_empty()
    {
        return Some(PathBuf::from(up));
    }

    let drive = std::env::var_os("HOMEDRIVE");
    let path = std::env::var_os("HOMEPATH");
    match (drive, path) {
        (Some(d), Some(p)) if !d.is_empty() && !p.is_empty() => {
            Some(PathBuf::from(d).join(PathBuf::from(p)))
        }
        _ => None,
    }
}

/// The runtime's own import provider: the filesystem.
///
/// A path that is not there is `Missing` and gets skipped, which is the
/// behavior the loader has always had; a path that is there but whose text
/// cannot be read is an error.
fn filesystem_import(path: &Path) -> ImportSource {
    match fs::read_to_string(path) {
        Ok(text) => ImportSource::Text(text),
        Err(_) if !path.exists() => ImportSource::Missing,
        Err(error) => ImportSource::Unreadable(error.to_string()),
    }
}

/// Load one gathered file through `provider`, parse it, inject it under its
/// alias, then recursively load that file’s gathers.
///
/// NOTE: this always loads nested gathers as *namespaced imports* (per their alias).
/// Include/merge semantics are handled only at the top-level loader.
fn load_gather(
    documents: &mut IndexMap<String, Document>,
    spec: &helpers::GatherSpec,
    import_path: &Path,
    visited: &mut HashSet<String>,
    provider: &mut dyn FnMut(&Path) -> ImportSource,
) -> Result<GatherLoad, Box<ImportFailure>> {
    let key = import_path.to_string_lossy().to_string();
    if visited.contains(&key) {
        return Ok(GatherLoad::Loaded);
    }
    visited.insert(key);

    let import_content = match provider(import_path) {
        ImportSource::Text(text) => text,
        ImportSource::Missing => return Ok(GatherLoad::Missing),
        ImportSource::Unreadable(message) => {
            return Err(Box::new(ImportFailure::new(
                &spec.raw_path,
                Some(import_path.to_path_buf()),
                RuneError::FileError {
                    message: format!("Failed to read import file: {}", message),
                    path: import_path.to_string_lossy().to_string(),
                    hint: Some("Check that the imported file exists".into()),
                    code: Some(302),
                },
            )));
        }
    };

    let failure = |error| {
        Box::new(ImportFailure::new(
            &spec.raw_path,
            Some(import_path.to_path_buf()),
            error,
        ))
    };
    let mut import_parser = parser::Parser::new(&import_content).map_err(failure)?;
    let import_doc = import_parser.parse_document().map_err(failure)?;

    // Overwrite any placeholder and/or previous doc with the real parsed doc
    documents.insert(spec.alias.clone(), import_doc);

    // Recurse into nested gathers
    let nested_specs = helpers::parse_gather_specs(&import_content);
    let nested_base = import_path.parent().unwrap_or_else(|| Path::new("."));

    for nested in nested_specs.iter() {
        let nested_path = resolve_gather_path(&nested.raw_path, nested_base).map_err(failure)?;

        // Nested gathers: keep them as namespaced imports only.
        load_gather(documents, nested, &nested_path, visited, provider)?;
    }

    Ok(GatherLoad::Loaded)
}

#[cfg(test)]
mod tests;
