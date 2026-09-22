// Author: Dustin Pilgrim
// License: MIT

//! The workspace file index the language server answers cross-file questions
//! from.
//!
//! Every `.rune` file the index knows about stores the `Arc<SourceIndex>` built
//! from its latest text, the ordered schema candidates it may bind to, the
//! candidate it currently resolves to, and whether it is a schema. Files reach
//! the index from three places, and the index keeps them apart:
//!
//! * workspace members, enumerated by a full scan and kept current by the
//!   filesystem events the client sends,
//! * editor buffers, which overlay whatever is on disk and always win,
//! * on-demand caches, read because a request or a schema candidate needed
//!   them.
//!
//! Cross-file navigation is answered from workspace members plus the buffers
//! the editor currently holds; an on-demand cache never adds a file to it on
//! its own.
//!
//! This module owns every production filesystem call the language server
//! makes, and every one of them is only reached from
//! `tokio::task::spawn_blocking`: nothing here is async and nothing here takes
//! a lock, so a scan, a watcher batch, or a candidate probe can neither block
//! the runtime nor interleave with a rebuild.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;
use tower_lsp::lsp_types::Url;

use crate::source::{SourceIndex, Span, starts_with_schema_block};

/// Directory basenames that are never bulk-scanned, whatever the client asks
/// for. Every hidden directory is excluded as well.
const DEFAULT_EXCLUDED_DIRS: [&str; 1] = ["target"];

/// Directory basenames excluded from bulk indexing.
///
/// The defaults always apply; the client's `initializationOptions.exclude`
/// names extend them and never replace them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ScanExcludes {
    names: Vec<String>,
}

impl ScanExcludes {
    /// Exclusions from `initializationOptions.exclude`.
    ///
    /// An absent or malformed option is not an error and leaves the defaults
    /// in place. An entry that does not name one plain directory - an empty
    /// string, `.`, `..`, or a path with more than one component - is ignored.
    pub(crate) fn from_initialize_options(options: Option<&Value>) -> Self {
        let names = options
            .and_then(|options| options.get("exclude"))
            .and_then(Value::as_array)
            .map(|names| {
                names
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|name| is_plain_directory_name(name))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        Self { names }
    }

    /// True when a directory with this basename is skipped before descending.
    pub(crate) fn excludes_dir(&self, name: &str) -> bool {
        name.starts_with('.')
            || DEFAULT_EXCLUDED_DIRS.contains(&name)
            || self.names.iter().any(|excluded| excluded == name)
    }
}

/// True when a client exclusion names exactly one plain directory.
fn is_plain_directory_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\\')
}

/// One indexed `.rune` file.
#[derive(Debug, Clone)]
pub(crate) struct IndexedFile {
    /// The buffer this file's structure and text are answered from.
    pub(crate) source: Arc<SourceIndex>,
    /// Ordered schema candidates, nearest and highest priority first. Empty
    /// for a schema document, which is what other documents bind to.
    pub(crate) candidates: Vec<PathBuf>,
    /// The first candidate that is indexed or present on disk.
    pub(crate) resolved: Option<Url>,
    /// True when this file is a schema rather than a config.
    pub(crate) is_schema: bool,
    /// True for a file the workspace scan or a watcher event contributed, as
    /// opposed to an editor buffer outside the workspace or a file a request
    /// read on demand.
    pub(crate) workspace_member: bool,
    /// True while an editor buffer overlays whatever is on disk.
    pub(crate) open: bool,
}

/// A file whose text is known and whose candidates are planned, ready to be
/// resolved into an [`IndexedFile`] in a blocking task.
#[derive(Debug, Clone)]
pub(crate) struct PreparedEntry {
    pub(crate) uri: Url,
    pub(crate) source: Arc<SourceIndex>,
    pub(crate) candidates: Vec<PathBuf>,
}

/// One entry to build: a prepared file, the membership its candidates already
/// have, and where its text came from.
#[derive(Debug, Clone)]
pub(crate) struct EntryPlan {
    pub(crate) prepared: PreparedEntry,
    pub(crate) indexed: Vec<bool>,
    pub(crate) workspace_member: bool,
    pub(crate) open: bool,
}

/// One cached binding to re-resolve, with the candidate paths it was resolved
/// from and, for each, whether it is currently indexed.
#[derive(Debug, Clone)]
pub(crate) struct Resolution {
    pub(crate) uri: Url,
    pub(crate) candidates: Vec<PathBuf>,
    pub(crate) indexed: Vec<bool>,
}

/// Every `.rune` file the server knows about, keyed by URI.
#[derive(Debug, Clone, Default)]
pub(crate) struct WorkspaceFileIndex {
    files: HashMap<Url, IndexedFile>,
    excludes: ScanExcludes,
}

impl WorkspaceFileIndex {
    pub(crate) fn new(excludes: ScanExcludes) -> Self {
        Self {
            files: HashMap::new(),
            excludes,
        }
    }

    pub(crate) fn entry(&self, uri: &Url) -> Option<&IndexedFile> {
        self.files.get(uri)
    }

    /// The exclusions this index was built with, so a rebuild carries them on.
    pub(crate) fn excludes(&self) -> &ScanExcludes {
        &self.excludes
    }

    pub(crate) fn source(&self, uri: &Url) -> Option<Arc<SourceIndex>> {
        self.files.get(uri).map(|file| Arc::clone(&file.source))
    }

    /// The indexed source of a candidate path, when the index already holds it.
    pub(crate) fn source_for_path(&self, path: &Path) -> Option<Arc<SourceIndex>> {
        candidate_uri(path).and_then(|uri| self.source(&uri))
    }

    /// True when a `file://` path is already indexed.
    pub(crate) fn contains_path(&self, path: &Path) -> bool {
        candidate_uri(path).is_some_and(|uri| self.files.contains_key(&uri))
    }

    pub(crate) fn is_workspace_member(&self, uri: &Url) -> bool {
        self.files
            .get(uri)
            .is_some_and(|file| file.workspace_member)
    }

    pub(crate) fn insert(&mut self, uri: Url, file: IndexedFile) {
        self.files.insert(uri, file);
    }

    pub(crate) fn remove(&mut self, uri: &Url) {
        self.files.remove(uri);
    }

    pub(crate) fn set_resolved(&mut self, uri: &Url, resolved: Option<Url>) {
        if let Some(file) = self.files.get_mut(uri) {
            file.resolved = resolved;
        }
    }

    /// Indexed, non-schema configs whose cached resolved schema is
    /// `schema_uri`, drawn from workspace members and currently open
    /// documents. No directory is walked and no file is read.
    pub(crate) fn configs_bound_to(&self, schema_uri: &Url) -> Vec<Url> {
        let mut configs: Vec<Url> = self
            .files
            .iter()
            .filter(|(uri, file)| {
                !file.is_schema
                    && file.resolved.as_ref() == Some(schema_uri)
                    && (file.workspace_member || file.open)
                    && is_rune_uri(uri)
            })
            .map(|(uri, _)| uri.clone())
            .collect();
        configs.sort_by(|left, right| left.as_str().cmp(right.as_str()));

        configs
    }

    /// True when a watched path may take part in bulk indexing: a `.rune` file
    /// inside a workspace folder and below no excluded directory. A file the
    /// index already holds is always eligible, so a schema the editor named
    /// from outside the workspace still refreshes.
    pub(crate) fn indexes_event(&self, path: &Path, folders: &[PathBuf]) -> bool {
        if !is_rune_path(path) {
            return false;
        }
        if self.contains_path(path) {
            return true;
        }

        let Some(folder) = containing_folder(folders, path) else {
            return false;
        };
        let Ok(relative) = path.strip_prefix(folder) else {
            return false;
        };

        // The file's own name is not a directory, so it is skipped: every
        // remaining component is a directory between the folder and the file.
        relative
            .components()
            .rev()
            .skip(1)
            .all(|component| match component {
                Component::Normal(name) => name
                    .to_str()
                    .is_some_and(|name| !self.excludes.excludes_dir(name)),
                _ => true,
            })
    }

    /// One re-resolution per entry whose candidate list mentions one of the
    /// changed URIs, carrying the membership its candidates currently have.
    ///
    /// Sources are never rebuilt: only the cached resolution is refreshed.
    pub(crate) fn resolutions(&self, changed: &[Url]) -> Vec<Resolution> {
        let changed: HashSet<&Url> = changed.iter().collect();

        self.files
            .iter()
            .filter(|(_, file)| {
                file.candidates.iter().any(|candidate| {
                    candidate_uri(candidate).is_some_and(|uri| changed.contains(&uri))
                })
            })
            .map(|(uri, file)| Resolution {
                uri: uri.clone(),
                indexed: file
                    .candidates
                    .iter()
                    .map(|candidate| self.contains_path(candidate))
                    .collect(),
                candidates: file.candidates.clone(),
            })
            .collect()
    }
}

/// One candidate of a schema search: already indexed, or a path to read.
#[derive(Debug)]
pub(crate) enum CandidateSlot {
    Indexed(Arc<SourceIndex>),
    Read(PathBuf),
}

/// What a schema candidate search found.
pub(crate) enum CandidateHit {
    Indexed(Arc<SourceIndex>),
    Read { path: PathBuf, text: String },
}

/// Walk candidates in order: the first one the index holds, or the first one
/// that can be read from disk, wins.
///
/// The order is the candidate order, so a missing nearest candidate never hides
/// a present further one, and only the candidates that are really needed are
/// read.
pub(crate) fn first_candidate(slots: Vec<CandidateSlot>) -> Option<CandidateHit> {
    for slot in slots {
        match slot {
            CandidateSlot::Indexed(source) => return Some(CandidateHit::Indexed(source)),
            CandidateSlot::Read(path) => {
                if let Some(text) = read_text(&path) {
                    return Some(CandidateHit::Read { path, text });
                }
            }
        }
    }

    None
}

/// The text of one file on disk, or `None` when it is missing or unreadable.
///
/// Unreadable and missing paths are skipped, which is the best-effort
/// behavior the server has always had.
pub(crate) fn read_text(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// The source index for `text`, reusing `existing` when it already holds
/// exactly this text.
///
/// This is what keeps unchanged text from being reparsed: a version-only
/// change, a rebuild overlay, and a watcher event that reports the same bytes
/// all keep the same `Arc`.
pub(crate) fn reuse_or_index(existing: Option<Arc<SourceIndex>>, text: &str) -> Arc<SourceIndex> {
    match existing {
        Some(existing) if existing.text() == text => existing,
        _ => Arc::new(SourceIndex::new(text)),
    }
}

/// Prepare one file whose text the caller already holds.
///
/// The candidates are planned from the same parse the entry will use, so one
/// update never tokenizes a buffer twice.
pub(crate) fn prepare_text_entry(
    uri: Url,
    text: String,
    folders: &[PathBuf],
    existing: Option<Arc<SourceIndex>>,
) -> PreparedEntry {
    let source = reuse_or_index(existing, &text);
    let candidates = candidate_paths(&uri, &source, folders);

    PreparedEntry {
        uri,
        source,
        candidates,
    }
}

/// Read and prepare the files an event named.
///
/// A file that cannot be read is left out, and the caller decides what a
/// missing file means. `existing` holds the sources the index already has, so
/// a file whose text did not change is not reparsed.
pub(crate) fn prepare_disk_entries(
    paths: Vec<(Url, PathBuf)>,
    folders: &[PathBuf],
    existing: &HashMap<Url, Arc<SourceIndex>>,
) -> Vec<PreparedEntry> {
    let mut entries = Vec::new();

    for (uri, path) in paths {
        let Some(text) = read_text(&path) else {
            continue;
        };
        let reuse = existing.get(&uri).cloned();
        entries.push(prepare_text_entry(uri, text, folders, reuse));
    }

    entries
}

/// Build entries, resolving each candidate list against the membership the
/// plan carries and, for everything else, the filesystem.
pub(crate) fn build_entries(plans: Vec<EntryPlan>) -> Vec<(Url, IndexedFile)> {
    plans
        .into_iter()
        .map(|plan| {
            let uri = plan.prepared.uri;
            let file = entry_from_source(
                &uri,
                plan.prepared.source,
                plan.prepared.candidates,
                &plan.indexed,
                plan.workspace_member,
                plan.open,
            );
            (uri, file)
        })
        .collect()
}

/// The first candidate that is already indexed, per `indexed`, or present on
/// disk.
pub(crate) fn resolve_candidates(candidates: &[PathBuf], indexed: &[bool]) -> Option<Url> {
    candidates
        .iter()
        .enumerate()
        .find_map(|(index, candidate)| {
            let uri = candidate_uri(candidate)?;
            (indexed.get(index).copied().unwrap_or(false) || is_file(candidate)).then_some(uri)
        })
}

/// Re-resolve cached bindings, returning the resolution each one now has.
pub(crate) fn resolve_all(resolutions: Vec<Resolution>) -> Vec<(Url, Option<Url>)> {
    resolutions
        .into_iter()
        .map(|resolution| {
            let resolved = resolve_candidates(&resolution.candidates, &resolution.indexed);
            (resolution.uri, resolved)
        })
        .collect()
}

/// A complete rebuild of the index.
///
/// The roots are walked and every eligible file is read, and the buffers the
/// editor currently holds are overlaid on top, so a rebuild can never replace
/// editor text with disk text. Entries the previous index held that the scan
/// did not find and that no buffer covers are carried over when they are
/// on-demand caches, and dropped when they were workspace members a rebuild no
/// longer sees.
pub(crate) fn build_workspace_index(
    roots: &[PathBuf],
    folders: &[PathBuf],
    excludes: &ScanExcludes,
    buffers: &HashMap<Url, Arc<SourceIndex>>,
    previous: &WorkspaceFileIndex,
) -> WorkspaceFileIndex {
    let members = enumerate_rune_files(roots, excludes);
    let member_uris: HashSet<Url> = members.iter().map(|(uri, _)| uri.clone()).collect();

    let mut index = WorkspaceFileIndex::new(excludes.clone());

    // Buffers first: an open buffer is a candidate for other documents even
    // when its file does not exist on disk, so it has to be resolvable before
    // anything that may bind to it is built.
    for (uri, source) in buffers {
        let candidates = candidate_paths(uri, source, folders);
        let indexed = candidates
            .iter()
            .map(|candidate| is_indexed_candidate(candidate, &member_uris, &index, buffers))
            .collect::<Vec<_>>();
        let file = entry_from_source(
            uri,
            Arc::clone(source),
            candidates,
            &indexed,
            member_uris.contains(uri),
            true,
        );
        index.insert(uri.clone(), file);
    }

    for (uri, path) in members {
        if index.entry(&uri).is_some() {
            continue;
        }
        let Some(text) = read_text(&path) else {
            continue;
        };
        let source = Arc::new(SourceIndex::new(&text));
        let candidates = candidate_paths(&uri, &source, folders);
        let indexed = candidates
            .iter()
            .map(|candidate| is_indexed_candidate(candidate, &member_uris, &index, buffers))
            .collect::<Vec<_>>();
        let file = entry_from_source(&uri, source, candidates, &indexed, true, false);
        index.insert(uri, file);
    }

    // On-demand caches stay: they are files a request read - an external
    // schema, or a config outside every workspace folder - and this scan knows
    // no more about them than the index already does. Their candidates are
    // planned again, because the workspace folders may have moved.
    for (uri, file) in &previous.files {
        if file.workspace_member || index.entry(uri).is_some() {
            continue;
        }
        let source = Arc::clone(&file.source);
        let candidates = candidate_paths(uri, &source, folders);
        let indexed = candidates
            .iter()
            .map(|candidate| is_indexed_candidate(candidate, &member_uris, &index, buffers))
            .collect::<Vec<_>>();
        let cached = entry_from_source(uri, source, candidates, &indexed, false, false);
        index.insert(uri.clone(), cached);
    }

    index
}

/// Every eligible `.rune` file under `roots`, deduplicated by URI.
///
/// The root is canonicalized before anything under it is trusted, a directory
/// whose canonical path leaves the root is never descended into, a directory is
/// never visited twice, and every symlink is skipped - which is how a cycle, a
/// link to a parent, and a link to `/` are all handled: never followed.
pub(crate) fn enumerate_rune_files(
    roots: &[PathBuf],
    excludes: &ScanExcludes,
) -> Vec<(Url, PathBuf)> {
    let mut files = Vec::new();
    let mut uris = HashSet::new();
    let mut visited = HashSet::new();

    for root in roots {
        let Ok(root) = std::fs::canonicalize(root) else {
            continue;
        };
        if !is_dir(&root) || !visited.insert(root.clone()) {
            continue;
        }
        walk(&root, &root, excludes, &mut visited, &mut uris, &mut files);
    }

    files
}

fn walk(
    root: &Path,
    dir: &Path,
    excludes: &ScanExcludes,
    visited: &mut HashSet<PathBuf>,
    uris: &mut HashSet<Url>,
    files: &mut Vec<(Url, PathBuf)>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        // `DirEntry::file_type` does not follow a symlink, so this rejects
        // links to files and to directories alike before either is used.
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();

        if file_type.is_dir() {
            if excluded_dir_name(&entry.file_name(), excludes) {
                continue;
            }
            let Ok(canonical) = std::fs::canonicalize(&path) else {
                continue;
            };
            if !canonical.starts_with(root) || !visited.insert(canonical.clone()) {
                continue;
            }
            walk(root, &canonical, excludes, visited, uris, files);
            continue;
        }

        if !file_type.is_file() || !is_rune_path(&path) {
            continue;
        }
        let Ok(uri) = Url::from_file_path(&path) else {
            continue;
        };
        if uris.insert(uri.clone()) {
            files.push((uri, path));
        }
    }
}

/// True when a directory basename is excluded before it is descended into.
fn excluded_dir_name(name: &std::ffi::OsStr, excludes: &ScanExcludes) -> bool {
    match name.to_str() {
        Some(name) => excludes.excludes_dir(name),
        // A name that is not valid UTF-8 can never be named by the client, so
        // it is not indexed.
        None => true,
    }
}

fn is_dir(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
}

fn is_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
}

fn candidate_uri(path: &Path) -> Option<Url> {
    Url::from_file_path(path).ok()
}

fn is_indexed_candidate(
    candidate: &Path,
    members: &HashSet<Url>,
    index: &WorkspaceFileIndex,
    buffers: &HashMap<Url, Arc<SourceIndex>>,
) -> bool {
    candidate_uri(candidate).is_some_and(|uri| {
        members.contains(&uri) || buffers.contains_key(&uri) || index.entry(&uri).is_some()
    })
}

/// The index entry of one file whose text is already parsed.
fn entry_from_source(
    uri: &Url,
    source: Arc<SourceIndex>,
    candidates: Vec<PathBuf>,
    indexed: &[bool],
    workspace_member: bool,
    open: bool,
) -> IndexedFile {
    let is_schema = is_schema_document(uri, &source);
    let candidates = if is_schema { Vec::new() } else { candidates };
    let resolved = resolve_candidates(&candidates, indexed);

    IndexedFile {
        source,
        candidates,
        resolved,
        is_schema,
        workspace_member,
        open,
    }
}

/// Ordered schema candidates of one document.
///
/// An explicit `@schema` directive supplies its own candidate order and may
/// name any path, an excluded or hidden directory and an outside schema
/// included. A document without one discovers ancestor `schema.rune` files,
/// nearest first, and never above the deepest workspace folder containing it.
/// This only plans paths: nothing here touches the filesystem, so exclusions
/// never hide a candidate a directive named.
pub(crate) fn candidate_paths(
    uri: &Url,
    source: &SourceIndex,
    folders: &[PathBuf],
) -> Vec<PathBuf> {
    let Ok(path) = uri.to_file_path() else {
        return Vec::new();
    };

    if let Some(directive) = schema_directive(source) {
        return path
            .parent()
            .map(|dir| schema_candidates(&directive.reference, dir))
            .unwrap_or_default();
    }

    discovery_candidates(folders, &path)
}

/// Ancestor `schema.rune` candidates for one config file, nearest first.
///
/// The walk stops at the deepest workspace folder containing the file, so a
/// schema above that folder is never discovered. A file outside every folder
/// keeps the unbounded walk up to the filesystem root.
fn discovery_candidates(folders: &[PathBuf], config_path: &Path) -> Vec<PathBuf> {
    let Some(mut directory) = config_path.parent().map(Path::to_path_buf) else {
        return Vec::new();
    };

    let boundary = containing_folder(folders, config_path).cloned();

    let mut candidates = Vec::new();
    loop {
        candidates.push(directory.join("schema.rune"));

        if boundary.as_ref().is_some_and(|folder| directory == *folder) {
            break;
        }
        if !directory.pop() {
            break;
        }
    }

    candidates
}

/// The deepest workspace folder containing `path`.
fn containing_folder<'a>(folders: &'a [PathBuf], path: &Path) -> Option<&'a PathBuf> {
    folders
        .iter()
        .filter(|folder| path.starts_with(folder))
        .max_by_key(|folder| folder.components().count())
}

fn schema_candidates(reference: &str, config_dir: &Path) -> Vec<PathBuf> {
    if is_schema_path_reference(reference) {
        return vec![expand_schema_path(reference, config_dir)];
    }

    let file_name = format!("{}.rune", reference);
    let mut candidates = vec![
        config_dir.join("schemas").join(&file_name),
        config_dir.join(".rune").join("schemas").join(&file_name),
    ];

    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(
            PathBuf::from(home)
                .join(".config")
                .join("rune")
                .join("schemas")
                .join(&file_name),
        );
    }

    candidates.push(PathBuf::from("/usr/local/share/rune/schemas").join(&file_name));
    candidates.push(PathBuf::from("/usr/share/rune/schemas").join(&file_name));
    candidates
}

pub(crate) fn is_schema_path_reference(reference: &str) -> bool {
    reference.starts_with('.')
        || reference.starts_with('/')
        || reference.starts_with('~')
        || reference.contains('/')
        || reference.contains('\\')
        || reference.ends_with(".rune")
}

pub(crate) fn expand_schema_path(reference: &str, config_dir: &Path) -> PathBuf {
    if let Some(rest) = reference.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }

    let path = PathBuf::from(reference);
    if path.is_absolute() {
        path
    } else {
        config_dir.join(path)
    }
}

fn is_rune_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "rune")
}

fn is_rune_uri(uri: &Url) -> bool {
    uri.to_file_path().is_ok_and(|path| is_rune_path(&path))
}

fn is_schema_file(uri: &Url) -> bool {
    uri.to_file_path().is_ok_and(|path| {
        path.file_name()
            .is_some_and(|name| name == std::ffi::OsStr::new("schema.rune"))
    })
}

/// True for a schema document: a `schema.rune` file, or a buffer whose first
/// statement is a `schema <name>:` block. The block is decided from real
/// tokens, so a commented-out or quoted `schema` never counts.
pub(crate) fn is_schema_document(uri: &Url, source: &SourceIndex) -> bool {
    is_schema_file(uri) || starts_with_schema_block(source.text())
}

/// The first `@schema "reference"` directive: the decoded reference plus the
/// span of the quoted token it was read from.
///
/// The directive is found through the indexed entries, so a `#` inside the
/// quoted reference never ends the directive early, and the span is the real
/// token span rather than the rest of the line the directive sits on.
pub(crate) fn schema_directive(source: &SourceIndex) -> Option<SchemaDirective> {
    let entry = source.entries().iter().find(|entry| {
        entry.kind == crate::source::SourceEntryKind::Metadata
            && entry.name.as_deref() == Some("schema")
    })?;

    let reference_span = source.quoted_value_span(entry)?;
    let raw = source
        .text()
        .get(reference_span.start..reference_span.end)?;
    let reference = parse_quoted_string(raw)?;

    Some(SchemaDirective {
        reference,
        reference_span,
    })
}

pub(crate) fn parse_quoted_string(input: &str) -> Option<String> {
    let rest = input.strip_prefix('"')?;
    let mut escaped = false;
    let mut value = String::new();

    for ch in rest.chars() {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '"' {
            return Some(value);
        }

        value.push(ch);
    }

    None
}

/// A decoded `@schema "reference"` directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchemaDirective {
    /// The decoded reference, with the quotes and escapes removed.
    pub(crate) reference: String,
    /// Byte span of the quoted token the reference was read from, quotes
    /// included. This is what a cursor is compared against, so a cursor on the
    /// `@schema` keyword or in a trailing comment is not on the reference.
    pub(crate) reference_span: Span,
}

/// The schema references an `@schema` value completes to, as `(label, file)`
/// pairs read from the schema directories.
///
/// The directories are enumerated in one blocking task and only when the
/// cursor is inside an `@schema` value, so no other completion reads a
/// directory.
pub(crate) fn schema_reference_items(config_dir: Option<&Path>) -> Vec<(String, PathBuf)> {
    let mut directories = Vec::new();
    if let Some(config_dir) = config_dir {
        directories.push(config_dir.join("schemas"));
        directories.push(config_dir.join(".rune").join("schemas"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        directories.push(
            PathBuf::from(home)
                .join(".config")
                .join("rune")
                .join("schemas"),
        );
    }
    directories.push(PathBuf::from("/usr/local/share/rune/schemas"));
    directories.push(PathBuf::from("/usr/share/rune/schemas"));

    let mut items: Vec<(String, PathBuf)> = Vec::new();
    for directory in directories {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("rune") {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if items.iter().any(|(label, _)| label == stem) {
                continue;
            }

            items.push((stem.to_string(), path));
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "app:\n  name \"Rune\"\nend\n";

    #[test]
    fn parses_schema_directive() {
        let source = SourceIndex::new(
            r#"
app:
  name "RuneApp"
end
@schema "stasis"
"#,
        );
        let directive = schema_directive(&source).unwrap();

        assert_eq!(directive.reference, "stasis");
        assert_eq!(
            &source.text()[directive.reference_span.start..directive.reference_span.end],
            "\"stasis\"",
            "the stored span is the quoted token itself"
        );
    }

    #[test]
    fn schema_directive_allows_path_references() {
        let directive =
            schema_directive(&SourceIndex::new(r#"@schema "./schemas/app.rune""#)).unwrap();
        assert_eq!(directive.reference, "./schemas/app.rune");
    }

    /// A `#` inside the quoted reference is part of the reference, not the
    /// start of a comment.
    #[test]
    fn schema_directive_keeps_comment_markers_inside_the_reference() {
        let source = SourceIndex::new("@schema \"./schemas/app#1.rune\"");
        let directive = schema_directive(&source).unwrap();

        assert_eq!(directive.reference, "./schemas/app#1.rune");
        assert_eq!(
            &source.text()[directive.reference_span.start..directive.reference_span.end],
            "\"./schemas/app#1.rune\""
        );
    }

    #[test]
    fn named_schema_candidates_include_project_and_system_paths() {
        let config_dir = Path::new("/tmp/rune-project");
        let candidates = schema_candidates("stasis", config_dir);

        assert!(candidates.contains(&PathBuf::from("/tmp/rune-project/schemas/stasis.rune")));
        assert!(candidates.contains(&PathBuf::from(
            "/tmp/rune-project/.rune/schemas/stasis.rune"
        )));
        assert!(candidates.contains(&PathBuf::from("/usr/local/share/rune/schemas/stasis.rune")));
        assert!(candidates.contains(&PathBuf::from("/usr/share/rune/schemas/stasis.rune")));
    }

    #[test]
    fn path_schema_candidate_resolves_relative_to_config_dir() {
        let config_dir = Path::new("/tmp/rune-project/config");
        let candidates = schema_candidates("../schemas/app.rune", config_dir);

        assert_eq!(
            candidates,
            vec![PathBuf::from(
                "/tmp/rune-project/config/../schemas/app.rune"
            )]
        );
    }

    #[test]
    fn rune_file_name_is_treated_as_path_reference() {
        let config_dir = Path::new("/tmp/rune-project");
        let candidates = schema_candidates("stasis.rune", config_dir);

        assert_eq!(
            candidates,
            vec![PathBuf::from("/tmp/rune-project/stasis.rune")]
        );
    }

    #[test]
    fn named_schema_content_is_treated_as_schema_document() {
        let uri = Url::from_file_path("/tmp/rune-project/schemas/stasis.rune").unwrap();
        let source = SourceIndex::new("# App schema\nschema app:\n  name string required\nend\n");

        assert!(is_schema_document(&uri, &source));
    }

    /// A schema directory the client excludes still resolves when a directive
    /// names it: exclusions are a bulk-indexing rule, not a resolution rule.
    #[test]
    fn directive_candidates_ignore_excluded_and_hidden_directories() {
        let uri = Url::from_file_path("/tmp/rune-project/config.rune").unwrap();
        let source = SourceIndex::new("@schema \"./vendor/rune/app.rune\"\napp:\nend\n");

        assert_eq!(
            candidate_paths(&uri, &source, &[PathBuf::from("/tmp/rune-project")]),
            vec![PathBuf::from("/tmp/rune-project/vendor/rune/app.rune")]
        );
    }

    /// Discovery is bounded by the deepest workspace folder, and free of any
    /// boundary at all outside every folder.
    #[test]
    fn discovery_candidates_stop_at_the_containing_folder() {
        let folders = vec![PathBuf::from("/tmp/rune-project")];
        let bounded =
            discovery_candidates(&folders, Path::new("/tmp/rune-project/sub/config.rune"));
        assert_eq!(
            bounded,
            vec![
                PathBuf::from("/tmp/rune-project/sub/schema.rune"),
                PathBuf::from("/tmp/rune-project/schema.rune"),
            ]
        );

        let outside = discovery_candidates(&folders, Path::new("/tmp/outside/config.rune"));
        assert_eq!(outside.len(), 3, "{outside:?}");
    }

    /// The same text keeps the same `Arc`, so a version-only change or a
    /// rebuild overlay never reparses a buffer; different text replaces it.
    #[test]
    fn identical_text_reuses_the_source_and_changed_text_replaces_it() {
        let first = reuse_or_index(None, TEXT);
        let second = reuse_or_index(Some(Arc::clone(&first)), TEXT);

        assert!(
            Arc::ptr_eq(&first, &second),
            "identical text must reuse the source index"
        );

        let changed = "app:\n  name 1\nend\n";
        let third = reuse_or_index(Some(Arc::clone(&second)), changed);

        assert!(
            !Arc::ptr_eq(&second, &third),
            "changed text must replace the source index"
        );
        assert_eq!(third.text(), changed);
        // The text the earlier sources were built from is untouched.
        assert_eq!(second.text(), TEXT);
    }

    #[test]
    fn client_exclusions_extend_the_defaults_and_ignore_bad_names() {
        let options = serde_json::json!({
            "exclude": ["vendor", "", ".", "..", "a/b", "target", "\\bad", "vendor"],
        });
        let excludes = ScanExcludes::from_initialize_options(Some(&options));

        assert!(excludes.excludes_dir("vendor"));
        assert!(excludes.excludes_dir("target"), "the default still applies");
        assert!(
            excludes.excludes_dir(".git"),
            "hidden directories always apply"
        );
        assert!(!excludes.excludes_dir("src"));

        let empty = ScanExcludes::from_initialize_options(Some(&serde_json::json!({})));
        assert!(empty.excludes_dir("target"));
        assert!(!empty.excludes_dir("vendor"));
    }

    /// A malformed or absent option must not fail initialization, and must not
    /// drop a default either.
    #[test]
    fn malformed_exclude_options_leave_the_defaults() {
        for options in [
            None,
            Some(serde_json::json!(null)),
            Some(serde_json::json!({ "exclude": "vendor" })),
            Some(serde_json::json!({ "exclude": [1, true, null] })),
        ] {
            let excludes = ScanExcludes::from_initialize_options(options.as_ref());
            assert!(excludes.excludes_dir("target"), "{options:?}");
            assert!(!excludes.excludes_dir("vendor"), "{options:?}");
        }
    }
}
