// Author: Dustin Pilgrim
// License: MIT

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, CompletionItem, CompletionItemKind,
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidChangeWatchedFilesParams, DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentChanges, DocumentFormattingParams, DocumentSymbol,
    DocumentSymbolParams, DocumentSymbolResponse, FileChangeType, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, InitializedParams, InsertTextFormat, Location,
    MarkedString, MessageType, OneOf, OptionalVersionedTextDocumentIdentifier, Position,
    PrepareRenameResponse, Range, ReferenceParams, RenameOptions, RenameParams, ServerCapabilities,
    SymbolKind, TextDocumentEdit, TextDocumentPositionParams, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit, Url, WorkDoneProgressOptions, WorkspaceEdit,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};
use tower_lsp::{Client, LanguageServer};

use crate::config::{ImportFailure, LoadedConfig};
use crate::diagnostic::{DiagnosticSeverity, RuneDiagnostic};
use crate::lexer::{Lexer, Token};
use crate::source::{
    GatherDirective, LineIndex, ReferenceRun, SourceEntry, SourceEntryKind, SourceIndex, Span,
};
use crate::{RuneConfig, RuneError, SchemaDocument, SchemaField, SchemaType};

use super::workspace_index::{
    CandidateHit, CandidateSlot, DeclarationKey, EffectiveSource, EntryPlan, GatherEdge,
    GatherView, IndexedFile, Resolution, ScanExcludes, SchemaDirective, WorkspaceFileIndex,
    buffer_overlay, build_entries, build_workspace_index, candidate_paths, child_keys,
    effective_origin, effective_sources, expand_schema_path, first_candidate, gather_occurrences,
    gather_view, gathered_scope_keys, import_source, is_schema_document, is_schema_path_reference,
    normalize_path, parse_quoted_string, prepare_disk_entries, prepare_text_entry, read_text,
    read_texts, references_to, resolve_all, resolve_candidates, resolve_reference,
    schema_directive, schema_reference_items,
};

/// What resolving a config document's schema candidates found.
enum SchemaBinding {
    /// The source of the schema the document binds to.
    Found(Arc<SourceIndex>),
    /// No candidate could be found at all.
    Missing,
    /// A candidate was found but could not be read.
    Unreadable,
}

/// An open buffer together with the structural index built from its latest
/// text, so each request re-uses one tokenization instead of re-deriving the
/// document layout from raw lines.
#[derive(Debug, Clone)]
struct OpenDocument {
    version: i32,
    source: Arc<SourceIndex>,
}

/// A single key occurrence for cross-file references/rename. The schema field
/// declaration is flagged so `references` can honor `include_declaration`.
struct Occurrence {
    uri: Url,
    range: Range,
    is_declaration: bool,
}

/// Schema resolution and import state for one open config document: the
/// candidates a config may bind to, the schema it currently resolved to, and
/// every file its `gather` statements reach.
///
/// The index owns the live state; this is the snapshot the last reconciliation
/// reported, so the next one can tell what changed.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DocumentDependency {
    /// Ordered, nearest / highest priority first.
    schema_candidates: Vec<PathBuf>,
    /// First candidate that is open or exists on disk.
    resolved_schema: Option<Url>,
    /// Every gather target the document reaches, transitively, including
    /// candidates that are not there.
    gather_targets: Vec<Url>,
    /// The aliases the document's gather view binds, as `(alias, target)`.
    gather_bindings: Vec<(String, Url)>,
}

pub struct RuneLanguageServer {
    client: Client,
    workspace_folders: RwLock<Vec<Url>>,
    documents: RwLock<HashMap<Url, OpenDocument>>,
    /// Every `.rune` file the server knows about, with the schema each config
    /// resolved to.
    index: RwLock<WorkspaceFileIndex>,
    /// Serializes index mutation. A rebuild must not interleave with another
    /// rebuild or with an incremental update, and a snapshot taken before a
    /// blocking read has to still be current when it is swapped back in.
    ///
    /// This lock is not reentrant: a handler takes it once for its own update
    /// and drops it before it publishes anything, so no helper reached from a
    /// request path is ever called while it is held.
    index_gate: Mutex<()>,
    /// The dependency state the last reconciliation reported for each open
    /// config document, which is what makes a change detectable.
    document_dependencies: RwLock<HashMap<Url, DocumentDependency>>,
    /// The client's `workspace.workspaceEdit.documentChanges` capability.
    /// Renames report versioned document changes only when it is set.
    supports_document_changes: RwLock<bool>,
}

impl RuneLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace_folders: RwLock::new(Vec::new()),
            documents: RwLock::new(HashMap::new()),
            index: RwLock::new(WorkspaceFileIndex::default()),
            index_gate: Mutex::new(()),
            document_dependencies: RwLock::new(HashMap::new()),
            supports_document_changes: RwLock::new(false),
        }
    }

    /// Publish diagnostics for exactly the given open `.rune` documents, which
    /// includes an empty list for a document that has no diagnostics left.
    ///
    /// Nothing else is revalidated, so a document unrelated to a change is not
    /// touched by it. The buffers are snapshotted before publishing so no lock
    /// is held across a client call.
    async fn validate_open_documents(&self, targets: &[Url]) {
        let snapshot: Vec<(Url, i32, Arc<SourceIndex>)> = {
            let documents = self.documents.read().await;
            targets
                .iter()
                .filter(|uri| is_rune_file(uri))
                .filter_map(|uri| {
                    documents.get(uri).map(|document| {
                        (uri.clone(), document.version, Arc::clone(&document.source))
                    })
                })
                .collect()
        };

        for (uri, version, source) in snapshot {
            let diagnostics = self.diagnostics_for_document(&uri, &source).await;
            self.client
                .publish_diagnostics(uri, diagnostics, Some(version))
                .await;
        }
    }

    /// File paths of the workspace folders, which bound schema discovery.
    async fn workspace_paths(&self) -> Vec<PathBuf> {
        self.workspace_folders
            .read()
            .await
            .iter()
            .filter_map(|uri| uri.to_file_path().ok())
            .collect()
    }

    /// The index entry for `text` built in blocking tasks: the candidates are
    /// planned from the same parse the entry will use, and resolved against
    /// what the index already holds plus the filesystem.
    ///
    /// Nothing here touches the index: the caller holds the rebuild gate and
    /// decides when the entry is swapped in.
    async fn plan_entry(
        &self,
        uri: &Url,
        text: &str,
        open: bool,
        workspace_member: bool,
    ) -> Option<(Url, IndexedFile)> {
        let folders = self.workspace_paths().await;
        let reuse = self.index.read().await.source(uri);
        let uri = uri.clone();
        let text = text.to_string();
        let prepared =
            tokio::task::spawn_blocking(move || prepare_text_entry(uri, text, &folders, reuse))
                .await
                .ok()?;

        let indexed = {
            let index = self.index.read().await;
            prepared
                .candidates
                .iter()
                .map(|candidate| index.contains_path(candidate))
                .collect()
        };

        tokio::task::spawn_blocking(move || {
            build_entries(vec![EntryPlan {
                prepared,
                indexed,
                workspace_member,
                open,
            }])
        })
        .await
        .ok()?
        .into_iter()
        .next()
    }

    /// Index the buffer the editor just sent: the document map and the index
    /// entry are swapped in together, both pointing at the same
    /// `Arc<SourceIndex>`. Identical text keeps the `Arc` that is already
    /// there, so a version-only change never reparses the buffer.
    async fn index_open_document(&self, uri: &Url, text: &str, version: i32) {
        let workspace_member = self.index.read().await.is_workspace_member(uri);
        let Some((uri, file)) = self.plan_entry(uri, text, true, workspace_member).await else {
            return;
        };

        let source = Arc::clone(&file.source);
        self.documents
            .write()
            .await
            .insert(uri.clone(), OpenDocument { version, source });
        self.index.write().await.insert(uri.clone(), file);
    }

    /// Drop the overlay of a closed document: a workspace member goes back to
    /// the file on disk, and a file that was indexed only because it was open
    /// is dropped.
    async fn close_document(&self, uri: &Url) {
        let workspace_member = self.index.read().await.is_workspace_member(uri);
        let path = uri.to_file_path().ok();
        let disk = match (workspace_member, path) {
            (true, Some(path)) => tokio::task::spawn_blocking(move || read_text(&path))
                .await
                .ok()
                .flatten(),
            _ => None,
        };

        let restored = match disk {
            Some(text) => self.plan_entry(uri, &text, false, true).await,
            None => None,
        };

        {
            let mut index = self.index.write().await;
            match restored {
                Some((uri, file)) => index.insert(uri, file),
                None => index.remove(uri),
            }
        }
        self.documents.write().await.remove(uri);
    }

    /// Cache the text of files read on demand, as non-members, and return how
    /// many were newly indexed.
    ///
    /// One gate hold covers the whole batch, and a file another task indexed
    /// while this one was reading keeps its own entry, which is how an editor
    /// buffer that arrives mid-read always wins.
    async fn cache_reads(&self, reads: Vec<(Url, String)>) -> usize {
        let _gate = self.index_gate.lock().await;
        let mut cached = 0;

        for (uri, text) in reads {
            if self.index.read().await.source(&uri).is_some() {
                continue;
            }

            let Some((uri, file)) = self.plan_entry(&uri, &text, false, false).await else {
                continue;
            };
            self.index.write().await.insert(uri, file);
            cached += 1;
        }

        cached
    }

    /// Cache the text of a file read on demand, as a non-member, and return the
    /// source that now backs it.
    async fn cache_read(&self, uri: &Url, text: &str) -> Option<Arc<SourceIndex>> {
        self.cache_reads(vec![(uri.clone(), text.to_string())])
            .await;
        self.index.read().await.source(uri)
    }

    /// Index every file the given roots reach through their `gather`
    /// statements.
    ///
    /// A target the index already holds - an editor buffer or a file the scan
    /// found - is used as it is, and only what is left is read, in one blocking
    /// task per round and cached as a non-member. A target that cannot be read
    /// stays out of the index, which is what leaves a missing import missing
    /// and an unreadable one reportable; because every resolved path is visited
    /// once, a cycle cannot make this loop forever.
    async fn ensure_gather_closures(&self, roots: &[Url]) {
        loop {
            let pending: Vec<(Url, PathBuf)> = {
                let index = self.index.read().await;
                let mut pending: Vec<(Url, PathBuf)> = Vec::new();

                for root in roots {
                    for target in gather_view(&index, root).unresolved {
                        if pending.iter().any(|(uri, _)| *uri == target) {
                            continue;
                        }
                        if let Ok(path) = target.to_file_path() {
                            pending.push((target, path));
                        }
                    }
                }

                pending
            };

            if pending.is_empty() {
                return;
            }

            let read = tokio::task::spawn_blocking(move || read_texts(pending))
                .await
                .unwrap_or_default();
            if read.is_empty() {
                return;
            }

            // Only a round that really indexed something can have discovered
            // more targets; anything else is as far as this can go.
            if self.cache_reads(read).await == 0 {
                return;
            }
        }
    }

    /// The open documents whose gather view reaches one of the given URIs.
    ///
    /// Those are the documents whose closure may have to grow when one of the
    /// URIs changes; nothing else is read because of it.
    async fn open_dependents_of(&self, uris: &[Url]) -> Vec<Url> {
        let index = self.index.read().await;
        let documents = self.documents.read().await;
        let mut dependents: Vec<Url> = Vec::new();

        for uri in uris {
            for root in index.roots_depending_on(uri) {
                if documents.contains_key(&root) && !dependents.contains(&root) {
                    dependents.push(root);
                }
            }
        }

        dependents
    }

    /// Re-resolve cached bindings whose candidate list mentions a URI whose
    /// availability just changed. Every `Arc<SourceIndex>` is left alone.
    async fn reresolve_dependents(&self, changed: &[Url]) {
        let resolutions: Vec<Resolution> = self.index.read().await.resolutions(changed);
        if resolutions.is_empty() {
            return;
        }

        let resolved = tokio::task::spawn_blocking(move || resolve_all(resolutions))
            .await
            .unwrap_or_default();

        let mut index = self.index.write().await;
        for (uri, resolved) in resolved {
            index.set_resolved(&uri, resolved);
        }
    }

    /// Scan every workspace folder and swap in a fresh index, overlaying the
    /// buffers the editor holds.
    ///
    /// The scan runs in a blocking task while holding no lock at all, so it
    /// cannot replace editor text with disk text and cannot interleave with
    /// another rebuild.
    async fn rebuild_index(&self, excludes: ScanExcludes) {
        let roots = self.workspace_paths().await;
        let folders = roots.clone();
        let buffers: HashMap<Url, Arc<SourceIndex>> = {
            let documents = self.documents.read().await;
            documents
                .iter()
                .map(|(uri, document)| (uri.clone(), Arc::clone(&document.source)))
                .collect()
        };
        let previous = self.index.read().await.clone();

        let rebuilt = tokio::task::spawn_blocking(move || {
            build_workspace_index(&roots, &folders, &excludes, &buffers, &previous)
        })
        .await;

        if let Ok(index) = rebuilt {
            *self.index.write().await = index;
        }
    }

    /// Read and index the files an event named, replacing exactly those
    /// entries. Nothing here walks a directory, so a file nobody reported stays
    /// invisible.
    async fn reindex_disk_files(&self, uris: &[Url]) {
        let folders = self.workspace_paths().await;
        let (paths, existing, members) = {
            let index = self.index.read().await;
            let paths = uris
                .iter()
                .filter_map(|uri| uri.to_file_path().ok().map(|path| (uri.clone(), path)))
                .collect();
            let existing = uris
                .iter()
                .filter_map(|uri| index.source(uri).map(|source| (uri.clone(), source)))
                .collect();
            // Only files the index already holds carry a membership forward.
            // A file that arrives with an event and is not indexed yet becomes
            // a workspace member, because it was reported as part of the
            // workspace.
            let members: HashMap<Url, bool> = uris
                .iter()
                .filter(|uri| index.entry(uri).is_some())
                .map(|uri| (uri.clone(), index.is_workspace_member(uri)))
                .collect();

            (paths, existing, members)
        };

        let prepared = {
            let folders = folders.clone();
            tokio::task::spawn_blocking(move || prepare_disk_entries(paths, &folders, &existing))
                .await
                .unwrap_or_default()
        };

        let (plans, prepared_uris) = {
            let index = self.index.read().await;
            let plans: Vec<EntryPlan> = prepared
                .into_iter()
                .map(|prepared| EntryPlan {
                    indexed: prepared
                        .candidates
                        .iter()
                        .map(|candidate| index.contains_path(candidate))
                        .collect(),
                    workspace_member: members.get(&prepared.uri).copied().unwrap_or(true),
                    open: false,
                    prepared,
                })
                .collect();
            let prepared_uris: HashSet<Url> =
                plans.iter().map(|plan| plan.prepared.uri.clone()).collect();

            (plans, prepared_uris)
        };

        let entries = tokio::task::spawn_blocking(move || build_entries(plans))
            .await
            .unwrap_or_default();

        let mut index = self.index.write().await;
        for (uri, file) in entries {
            index.insert(uri, file);
        }
        // A file the event named that could not be read is no longer
        // available: its entry goes the way a deleted file's does.
        for uri in uris {
            if !prepared_uris.contains(uri) {
                index.remove(uri);
            }
        }
    }

    /// Find the schema among `candidates`, in order.
    ///
    /// A candidate the index holds is used as it is; everything else is read on
    /// disk inside one blocking task, in candidate order, including the schema
    /// directories a scan never descends into. A read that succeeds is cached
    /// as a non-member.
    ///
    /// This is only reached when nothing resolved from the index, so it is what
    /// keeps a candidate that appeared on disk without an event reachable.
    async fn probe_schema_candidates(&self, candidates: &[PathBuf]) -> SchemaBinding {
        let mut slots = Vec::new();
        {
            let index = self.index.read().await;
            for candidate in candidates {
                match index.source_for_path(candidate) {
                    Some(source) => slots.push(CandidateSlot::Indexed(source)),
                    None => slots.push(CandidateSlot::Read(candidate.clone())),
                }
            }
        }

        let hit = tokio::task::spawn_blocking(move || first_candidate(slots))
            .await
            .ok()
            .flatten();

        match hit {
            Some(CandidateHit::Indexed(source)) => SchemaBinding::Found(source),
            Some(CandidateHit::Read { path, text }) => match Url::from_file_path(&path) {
                Ok(uri) => match self.cache_read(&uri, &text).await {
                    Some(source) => SchemaBinding::Found(source),
                    None => SchemaBinding::Missing,
                },
                Err(_) => SchemaBinding::Missing,
            },
            None => SchemaBinding::Missing,
        }
    }

    /// Resolve the schema one config document binds to.
    ///
    /// The candidates come from the index entry - the same parse the document
    /// was indexed from - and the cached resolution decides the first one
    /// without touching the filesystem. A resolved candidate the index does not
    /// hold is read once, in a blocking task; one that resolves to nothing at
    /// all sends the candidates to `probe_schema_candidates`.
    async fn schema_source_for_document(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
    ) -> SchemaBinding {
        let planned = {
            self.index
                .read()
                .await
                .entry(uri)
                .map(|file| (file.candidates.clone(), file.resolved.clone()))
        };
        let (candidates, resolved) = match planned {
            Some(planned) => planned,
            // A document the index does not know yet still resolves from the
            // text in hand.
            None => (
                candidate_paths(uri, source, &self.workspace_paths().await),
                None,
            ),
        };

        if let Some(schema_uri) = resolved {
            return match self.document_source_for(&schema_uri).await {
                Some(schema_source) => SchemaBinding::Found(schema_source),
                // The candidate is present but its text cannot be read.
                None => SchemaBinding::Unreadable,
            };
        }

        self.probe_schema_candidates(&candidates).await
    }

    /// Ordered candidates of a config document, for a diagnostic that lists
    /// them. No filesystem call and no reparse: the index entry is the plan.
    async fn candidates_for_document(&self, uri: &Url, source: &Arc<SourceIndex>) -> Vec<PathBuf> {
        if let Some(file) = self.index.read().await.entry(uri) {
            return file.candidates.clone();
        }

        candidate_paths(uri, source, &self.workspace_paths().await)
    }

    /// Recompute the dependency state of every open config document against the
    /// final index state, and return the open configs that must be revalidated.
    ///
    /// A config is affected when it was forced, when its resolved schema URI
    /// changed, when one of the changed files is the schema it resolved to
    /// before or resolves to now, when one of the changed files is anywhere in
    /// its transitive gather set, or when the set of files it gathers changed.
    /// A change to the candidate list alone, or to a gather's own source text,
    /// records new state without republishing on its own, because the resolved
    /// schema and the gathered files are what the document is validated against.
    async fn refresh_dependencies_and_collect_affected(
        &self,
        changed_uris: &[Url],
        forced_documents: &[Url],
    ) -> Vec<Url> {
        let current: HashMap<Url, DocumentDependency> = {
            let index = self.index.read().await;
            let documents = self.documents.read().await;

            documents
                .keys()
                .filter_map(|uri| {
                    let file = index.entry(uri)?;
                    // Schema documents have no dependency: they are what configs
                    // depend on, so they never take part in discovery.
                    if file.is_schema {
                        return None;
                    }

                    let view = gather_view(&index, uri);
                    Some((
                        uri.clone(),
                        DocumentDependency {
                            schema_candidates: file.candidates.clone(),
                            resolved_schema: file.resolved.clone(),
                            gather_targets: view.reachable,
                            gather_bindings: view
                                .aliases
                                .iter()
                                .map(|binding| (binding.alias.clone(), binding.uri.clone()))
                                .collect(),
                        },
                    ))
                })
                .collect()
        };
        let previous_dependencies = self.document_dependencies.read().await.clone();

        let mut affected: Vec<Url> = Vec::new();

        for (uri, dependency) in &current {
            let previous = previous_dependencies.get(uri);

            let resolved_changed = previous.map(|dependency| &dependency.resolved_schema)
                != Some(&dependency.resolved_schema);
            let schema_file_changed = changed_uris.iter().any(|changed| {
                previous.and_then(|dependency| dependency.resolved_schema.as_ref()) == Some(changed)
                    || dependency.resolved_schema.as_ref() == Some(changed)
            });
            // A change to any file the document reaches, including one that is
            // not there yet, is a change to what it is validated against.
            let gather_file_changed = changed_uris.iter().any(|changed| {
                previous.is_some_and(|dependency| dependency.gather_targets.contains(changed))
                    || dependency.gather_targets.contains(changed)
            });
            let gather_bindings_changed = previous.is_some_and(|previous| {
                previous.gather_targets != dependency.gather_targets
                    || previous.gather_bindings != dependency.gather_bindings
            });

            if forced_documents.contains(uri)
                || resolved_changed
                || schema_file_changed
                || gather_file_changed
                || gather_bindings_changed
            {
                affected.push(uri.clone());
            }
        }

        // Record the recomputed dependency. A state change that does not alter
        // what the document is validated against is stored without publishing:
        // the next reconciliation tells what moved.
        *self.document_dependencies.write().await = current;

        affected
    }

    async fn diagnostics_for_document(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
    ) -> Vec<tower_lsp::lsp_types::Diagnostic> {
        if is_schema_document(uri, source) {
            let rune_diagnostics = match SchemaDocument::from_str(source.text()) {
                Ok(_) => Vec::new(),
                Err(error) => vec![diagnostic_from_error(error, source.lines())],
            };

            return rune_diagnostics
                .into_iter()
                .map(lsp_diagnostic_from_rune)
                .collect();
        }

        self.config_diagnostics(uri, source).await
    }

    /// The diagnostics of one open config, built from the same gather view the
    /// runtime loads for it.
    ///
    /// The root buffer and every file its `gather` statements reach are loaded
    /// by the loader the runtime uses, with open buffers winning over disk, so
    /// what the editor validates is what the program would read. Only an open
    /// document is ever published for.
    async fn config_diagnostics(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
    ) -> Vec<tower_lsp::lsp_types::Diagnostic> {
        self.ensure_gather_closures(std::slice::from_ref(uri)).await;
        let context = self.gather_context(uri).await;

        let Some(loaded) = self.load_config(uri, source, &context).await else {
            return Vec::new();
        };

        let loaded = match loaded {
            Ok(loaded) => loaded,
            // The buffer itself does not parse: its diagnostics are about the
            // buffer, exactly as they always were.
            Err(error) => {
                let mut diagnostics = vec![diagnostic_from_error(error, source.lines())];
                diagnostics.extend(recovery_diagnostics(source));
                dedupe_diagnostics(&mut diagnostics);
                return diagnostics
                    .into_iter()
                    .map(lsp_diagnostic_from_rune)
                    .collect();
            }
        };

        // One import that could not be loaded is what stops the runtime, and it
        // is reported on the root's own statement that reached it.
        if let Some(failure) = loaded.failures.into_iter().next() {
            return vec![
                self.gather_failure_diagnostic(source, &context, failure)
                    .await,
            ];
        }

        let config = loaded.config;

        let schema_source = match self.schema_source_for_document(uri, source).await {
            SchemaBinding::Found(schema_source) => schema_source,
            binding => {
                return self
                    .schema_binding_diagnostics(uri, source, &binding)
                    .await
                    .into_iter()
                    .map(lsp_diagnostic_from_rune)
                    .collect();
            }
        };

        let schema = match SchemaDocument::from_str(schema_source.text()) {
            Ok(schema) => schema,
            Err(error) => {
                return vec![lsp_diagnostic_from_rune(diagnostic_from_error(
                    error,
                    source.lines(),
                ))];
            }
        };

        config
            .validate_schema_with_source(&schema, source)
            .into_iter()
            .map(|diagnostic| import_aware_diagnostic(source, &context, diagnostic))
            .collect()
    }

    /// The gather view of one root document, with the files that merge into it
    /// and the buffers a config load prefers over disk.
    async fn gather_context(&self, uri: &Url) -> GatherContext {
        let (view, sources, open) = {
            let index = self.index.read().await;
            let view = gather_view(&index, uri);
            let sources = effective_sources(&index, &view);
            let open: Vec<(Url, Arc<SourceIndex>)> = index
                .entries()
                .filter(|(_, file)| file.open)
                .map(|(uri, file)| (uri.clone(), Arc::clone(&file.source)))
                .collect();

            (view, sources, open)
        };

        // Locating an editor buffer touches the filesystem, so it happens in a
        // blocking task like every other read.
        let overlay = tokio::task::spawn_blocking(move || buffer_overlay(&open))
            .await
            .unwrap_or_default();

        GatherContext {
            view,
            sources,
            overlay,
        }
    }

    /// Load one config the way the runtime loads it: the buffer in hand, plus
    /// every import its `gather` statements reach.
    ///
    /// `None` only when the blocking task itself could not run, which is when
    /// nothing can be said about the document at all.
    async fn load_config(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
        context: &GatherContext,
    ) -> Option<Result<LoadedConfig, RuneError>> {
        let text = source.text().to_string();
        let base_dir = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));
        let overlay = context.overlay.clone();

        tokio::task::spawn_blocking(move || {
            RuneConfig::from_text_with_provider(&text, &base_dir, &mut |import| {
                import_source(import, &overlay)
            })
        })
        .await
        .ok()
    }

    /// The diagnostic one failed `gather` produces on the open root.
    ///
    /// It is anchored on the root's own statement that led to the failing file,
    /// which is the first hop for a failure anywhere down the chain. The
    /// failing file is named in the message and linked through
    /// `relatedInformation`.
    async fn gather_failure_diagnostic(
        &self,
        source: &Arc<SourceIndex>,
        context: &GatherContext,
        failure: ImportFailure,
    ) -> tower_lsp::lsp_types::Diagnostic {
        let directive = gather_anchor(source, &context.view, &failure);
        let display = failure.resolved_path.display();
        let (detail, position) = error_detail(&failure.error);

        let message = match position {
            Some((line, column)) => format!(
                "Gathered file '{}' has a syntax error: {} (line {}, column {})",
                display, detail, line, column
            ),
            None => format!(
                "Gathered file '{}' could not be loaded: {}",
                display, detail
            ),
        };

        let range = directive
            .map(|directive| source.lines().range(directive.path_span))
            .unwrap_or_else(default_range);

        // The imported buffer answers the related range when the index holds it,
        // which is where an editor sees the failing file itself. The index keys
        // every target lexically normalized, so the loader's spelling of the
        // path is normalized before it becomes a URI.
        let imported_uri = Url::from_file_path(normalize_path(&failure.resolved_path)).ok();
        let imported = match &imported_uri {
            Some(uri) => self.index.read().await.source(uri),
            None => None,
        };
        let related = imported_uri.map(|uri| {
            vec![tower_lsp::lsp_types::DiagnosticRelatedInformation {
                location: Location {
                    uri,
                    range: error_range(imported.as_deref(), position),
                },
                message: format!("'{}' is gathered here", failure.raw_path),
            }]
        });

        tower_lsp::lsp_types::Diagnostic {
            range,
            severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
            code: Some(tower_lsp::lsp_types::NumberOrString::Number(
                GATHER_LOAD_CODE,
            )),
            code_description: None,
            source: Some("rune-cfg".into()),
            message,
            related_information: related,
            tags: None,
            data: None,
        }
    }

    /// The diagnostics a config gets when its schema did not resolve: a
    /// directive that found nothing is reported on its quoted reference, with
    /// every candidate that was checked; a directive whose candidate could not
    /// be read lists none. A config with no `@schema` directive is simply
    /// unvalidated and has no diagnostic at all.
    async fn schema_binding_diagnostics(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
        binding: &SchemaBinding,
    ) -> Vec<RuneDiagnostic> {
        let Some(directive) = schema_directive(source) else {
            return Vec::new();
        };

        let candidates = match binding {
            SchemaBinding::Missing => self.candidates_for_document(uri, source).await,
            SchemaBinding::Found(_) | SchemaBinding::Unreadable => Vec::new(),
        };

        vec![schema_reference_diagnostic(
            source.lines(),
            &directive,
            &candidates,
        )]
    }

    async fn schema_text_for(&self, uri: &Url) -> Option<String> {
        let source = self.document_source_for(uri).await?;
        match self.schema_source_for_document(uri, &source).await {
            SchemaBinding::Found(schema_source) => Some(schema_source.text().to_string()),
            SchemaBinding::Missing | SchemaBinding::Unreadable => None,
        }
    }

    async fn schema_for(&self, uri: &Url) -> Option<SchemaDocument> {
        let schema_text = self.schema_text_for(uri).await?;
        SchemaDocument::from_str(&schema_text).ok()
    }

    /// Resolve the schema file URI backing a config document, following an
    /// explicit `@schema` directive when present and otherwise discovering
    /// `schema.rune` upward from the config directory.
    async fn schema_uri_for_document(&self, uri: &Url, source: &Arc<SourceIndex>) -> Option<Url> {
        if let Some(file) = self.index.read().await.entry(uri) {
            if file.is_schema {
                return None;
            }

            return file.resolved.clone();
        }

        let candidates = candidate_paths(uri, source, &self.workspace_paths().await);
        let indexed = {
            let index = self.index.read().await;
            candidates
                .iter()
                .map(|candidate| index.contains_path(candidate))
                .collect::<Vec<_>>()
        };
        tokio::task::spawn_blocking(move || resolve_candidates(&candidates, &indexed))
            .await
            .ok()
            .flatten()
    }

    /// Resolve the rename/references target at a position: the field path plus
    /// the schema it belongs to. The schema URI is `None` when the document has
    /// no schema, signalling single-file behavior.
    async fn rename_target(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
        position: Position,
    ) -> Option<(Vec<String>, Option<Url>)> {
        if is_schema_document(uri, source) {
            let schema = SchemaDocument::from_str(source.text()).ok()?;
            let path = schema_path_at_position(source.text(), &schema, position)?;
            return Some((path, Some(uri.clone())));
        }

        let path = source.field_key_at(position)?.path.clone();
        let schema_uri = self.schema_uri_for_document(uri, source).await;
        Some((path, schema_uri))
    }

    /// Config files bound to `schema_uri` (via `@schema` or discovery), drawn
    /// from workspace members and currently open documents, and never from an
    /// on-demand cache. Excludes the schema itself.
    ///
    /// This is answered from the index's cached resolutions: no directory is
    /// walked and no file is read.
    async fn related_config_uris(&self, schema_uri: &Url) -> Vec<Url> {
        self.index.read().await.configs_bound_to(schema_uri)
    }

    /// Every occurrence of `path` to rename together: the schema field
    /// declaration (marked) plus its key usages in each bound config file.
    async fn cross_file_occurrences(&self, schema_uri: &Url, path: &[String]) -> Vec<Occurrence> {
        let mut occurrences = Vec::new();

        if let Some(schema_text) = self.document_text_for(schema_uri).await
            && let Ok(schema) = SchemaDocument::from_str(&schema_text)
            && let Some(range) = schema_definition_range(&schema_text, &schema, path)
        {
            occurrences.push(Occurrence {
                uri: schema_uri.clone(),
                range,
                is_declaration: true,
            });
        }

        for config_uri in self.related_config_uris(schema_uri).await {
            if let Some(source) = self.document_source_for(&config_uri).await {
                for range in references_in_document(&source, path) {
                    occurrences.push(Occurrence {
                        uri: config_uri.clone(),
                        range,
                        is_declaration: false,
                    });
                }
            }
        }

        occurrences
    }

    /// True when a field at `candidate` already exists in one document, which
    /// is what turns a rename into a sibling collision.
    ///
    /// A schema document is read as a schema and resolved through its parsed
    /// fields; every other document is a config, read through its index.
    async fn field_exists_in(&self, uri: &Url, is_schema: bool, candidate: &[String]) -> bool {
        let Some(source) = self.document_source_for(uri).await else {
            return false;
        };

        if is_schema {
            let Ok(schema) = SchemaDocument::from_str(source.text()) else {
                return false;
            };
            return find_field_by_path(&schema, candidate).is_some();
        }

        !source.entries_with_path(candidate).is_empty()
    }

    /// The workspace edit for a set of per-document rename edits.
    ///
    /// A client that supports `workspace.workspaceEdit.documentChanges` gets
    /// versioned document changes: one `TextDocumentEdit` per document, sorted
    /// by URI string, each carrying the version of the open buffer it applies
    /// to or `null` for a file that only exists on disk. Every other client
    /// gets the `changes` map. The two are never filled together.
    async fn rename_workspace_edit(
        &self,
        edits: Vec<(Url, Vec<Range>)>,
        new_name: &str,
    ) -> WorkspaceEdit {
        if !*self.supports_document_changes.read().await {
            let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

            for (uri, ranges) in edits {
                changes
                    .entry(uri)
                    .or_default()
                    .extend(ranges.into_iter().map(|range| TextEdit {
                        range,
                        new_text: new_name.to_string(),
                    }));
            }

            return WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            };
        }

        let versions: HashMap<Url, i32> = self
            .documents
            .read()
            .await
            .iter()
            .map(|(uri, document)| (uri.clone(), document.version))
            .collect();

        let mut ordered = edits;
        ordered.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));

        let document_changes = ordered
            .into_iter()
            .map(|(uri, ranges)| TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    version: versions.get(&uri).copied(),
                    uri,
                },
                edits: ranges
                    .into_iter()
                    .map(|range| {
                        OneOf::Left(TextEdit {
                            range,
                            new_text: new_name.to_string(),
                        })
                    })
                    .collect(),
            })
            .collect();

        WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(document_changes)),
            change_annotations: None,
        }
    }

    /// The effective sources of one document, which is what tells a local field
    /// from one a `gather` supplies.
    async fn effective_sources_for(&self, uri: &Url) -> Vec<EffectiveSource> {
        let index = self.index.read().await;
        let view = gather_view(&index, uri);

        effective_sources(&index, &view)
    }

    /// What one completion needs from a document's gather view: the children of
    /// an alias prefix, and the keys the unaliased gathers already supply.
    async fn gather_completion(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
        position: Position,
    ) -> Option<GatherCompletion> {
        self.ensure_gather_closures(std::slice::from_ref(uri)).await;

        let index = self.index.read().await;
        let view = gather_view(&index, uri);
        if view.direct.is_empty() {
            return None;
        }
        let sources = effective_sources(&index, &view);

        // `alias.` and `alias.object.` complete from the imported document the
        // alias resolves to. A `gather` alias whose target is not there has no
        // children at all, so nothing else is offered after it.
        let prefix_keys = source.reference_prefix_at(position).and_then(|prefix| {
            let (alias, rest) = prefix.split_first()?;

            match view.alias(alias) {
                Some(binding) => {
                    let imported = index.source(&binding.uri)?;
                    Some(child_keys(&imported, rest))
                }
                None if view.alias_names.contains(alias) => Some(Vec::new()),
                None => None,
            }
        });

        let supplied = gathered_scope_keys(&sources, &source.scope_at(position));
        let reference_position = source.assignment_before_cursor(position).is_some()
            || source.reference_at(position).is_some();

        Some(GatherCompletion {
            prefix_keys,
            supplied,
            reference_position,
        })
    }

    /// The definition of a `gather` statement: the file it names, when that
    /// file is really there.
    async fn gather_target_definition(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
        directive: &GatherDirective,
    ) -> Option<GotoDefinitionResponse> {
        self.ensure_gather_closures(std::slice::from_ref(uri)).await;

        let index = self.index.read().await;
        let view = gather_view(&index, uri);
        let edge =
            gather_position(source, directive).and_then(|position| view.direct.get(position))?;
        let target = edge.uri.clone()?;

        // A target that is not there, or cannot be read, is not a link.
        index.entry(&target)?;

        Some(GotoDefinitionResponse::Scalar(Location {
            uri: target,
            range: zero_range(),
        }))
    }

    /// The hover of one `gather` statement: what it resolves to, the alias it
    /// binds, and whether it includes or namespaces the file.
    async fn gather_hover(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
        directive: &GatherDirective,
    ) -> Option<Hover> {
        self.ensure_gather_closures(std::slice::from_ref(uri)).await;

        let index = self.index.read().await;
        let view = gather_view(&index, uri);
        let edge =
            gather_position(source, directive).and_then(|position| view.direct.get(position))?;
        let resolved = edge
            .uri
            .as_ref()
            .is_some_and(|target| index.entry(target).is_some());

        Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(gather_hover_text(
                edge, resolved,
            ))),
            range: None,
        })
    }

    /// Resolve the plain reference the cursor is on through the root's own
    /// gather view.
    async fn resolve_reference_at(&self, root: &Url, run: &ReferenceRun) -> Option<ResolvedSite> {
        let index = self.index.read().await;
        let view = gather_view(&index, root);
        let sources = effective_sources(&index, &view);
        let resolved = resolve_reference(&index, &view, &sources, &run.names())?;
        let target = index.source(&resolved.uri);

        Some(ResolvedSite {
            range: resolved
                .declaration
                .and_then(|(span, _)| target.as_ref().map(|source| source.lines().range(span))),
            key: resolved.declaration.map(|(span, _)| DeclarationKey {
                uri: resolved.uri.clone(),
                span,
            }),
            path: resolved.path,
            kind: resolved.declaration.map(|(_, kind)| kind),
            imported: resolved.imported,
            uri: resolved.uri,
        })
    }

    /// The gather-aware uses of the field under the cursor, when the file the
    /// cursor is in is itself gathered by another root.
    ///
    /// The declaration is the key itself, and its uses are the references that
    /// resolve to it, in every workspace-member or open config that reaches this
    /// file.
    async fn imported_uses(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
        position: Position,
        include_declaration: bool,
    ) -> Vec<Location> {
        let Some(entry) = source.field_key_at(position) else {
            return Vec::new();
        };
        let key = DeclarationKey {
            uri: uri.clone(),
            span: entry.key_span,
        };
        let declaration = source.key_range(entry);

        self.ensure_gather_closures(std::slice::from_ref(uri)).await;
        let index = self.index.read().await;
        let mut locations: Vec<Location> = Vec::new();

        if include_declaration {
            locations.push(Location {
                uri: uri.clone(),
                range: declaration,
            });
        }

        for root in index.roots_depending_on(uri) {
            let Some(root_source) = index.source(&root) else {
                continue;
            };
            for span in references_to(&index, &root, &key) {
                locations.push(Location {
                    uri: root.clone(),
                    range: root_source.lines().range(span),
                });
            }
        }

        locations
    }

    /// The references of a `gather` link or of a plain reference, or `None`
    /// when the cursor is on neither.
    ///
    /// A gather statement's link is identified by the file it names, so two
    /// statements that write the same alias for different files are never
    /// conflated. A reference is identified by the declaration it resolves to,
    /// exactly where that declaration is written, and its uses are searched for
    /// in the configs that reach that file.
    async fn gather_references(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
        position: Position,
        include_declaration: bool,
    ) -> Option<Vec<Location>> {
        self.ensure_gather_closures(std::slice::from_ref(uri)).await;

        if let Some(directive) = source.gather_at(position) {
            let index = self.index.read().await;
            let view = gather_view(&index, uri);
            let statement = gather_position(source, directive)?;
            let target = view.direct.get(statement)?.uri.clone()?;
            let current = (uri.clone(), statement);

            let locations = gather_occurrences(&index, &target)
                .into_iter()
                .filter(|(file, edge)| {
                    include_declaration || (file.clone(), edge.directive) != current
                })
                .filter_map(|(file, edge)| {
                    let source = index.source(&file)?;
                    let directive = source.gathers().get(edge.directive)?;

                    Some(Location {
                        uri: file,
                        range: source.lines().range(directive.path_span),
                    })
                })
                .collect();

            return Some(locations);
        }

        let run = source.reference_at(position)?;
        let site = self.resolve_reference_at(uri, run).await?;

        // A reference that names a whole imported file has no declaration to
        // search uses of.
        let key = site.key.clone()?;
        let index = self.index.read().await;
        let mut locations: Vec<Location> = Vec::new();

        if include_declaration && let Some(declaration) = index.source(&key.uri) {
            locations.push(Location {
                uri: key.uri.clone(),
                range: declaration.lines().range(key.span),
            });
        }

        // A use can only live in a config that reaches the declaring file, or
        // in the file itself.
        for root in index.roots_depending_on(&key.uri) {
            let Some(source) = index.source(&root) else {
                continue;
            };
            for span in references_to(&index, &root, &key) {
                locations.push(Location {
                    uri: root.clone(),
                    range: source.lines().range(span),
                });
            }
        }

        Some(locations)
    }

    async fn document_source_for(&self, uri: &Url) -> Option<Arc<SourceIndex>> {
        if let Some(source) = self.index.read().await.source(uri) {
            return Some(source);
        }

        let path = uri.to_file_path().ok()?;
        let text = tokio::task::spawn_blocking(move || read_text(&path))
            .await
            .ok()
            .flatten()?;

        self.cache_read(uri, &text).await
    }

    /// Text of a document, whether or not it is open in the editor.
    async fn document_text_for(&self, uri: &Url) -> Option<String> {
        self.document_source_for(uri)
            .await
            .map(|source| source.text().to_string())
    }

    async fn schema_source_label_for_document(
        &self,
        uri: &Url,
        source: &Arc<SourceIndex>,
    ) -> Option<String> {
        if let Some(directive) = schema_directive(source) {
            return Some(format!("@schema \"{}\"", directive.reference));
        }

        let schema_uri = self.schema_uri_for_document(uri, source).await?;
        schema_uri
            .to_file_path()
            .ok()
            .map(|path| path.display().to_string())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for RuneLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        *self.workspace_folders.write().await = workspace_folders_from_initialize(&params);
        *self.supports_document_changes.write().await = supports_document_changes(&params);

        // The first full scan runs once the folders and the client's exclusions
        // are known, and it is the only thing that ever reads the whole
        // workspace.
        let excludes =
            ScanExcludes::from_initialize_options(params.initialization_options.as_ref());
        {
            let _gate = self.index_gate.lock().await;
            self.rebuild_index(excludes).await;
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["$".into(), ".".into(), "\"".into()]),
                    ..CompletionOptions::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(tower_lsp::lsp_types::ServerInfo {
                name: "runecfg-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "runecfg-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let Some(source) = self.document_source_for(&uri).await else {
            return Ok(None);
        };

        let items = if is_schema_document(&uri, &source) {
            schema_completion_items()
        } else {
            let config_dir = uri
                .to_file_path()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf));

            // Only a cursor inside an `@schema` value reads a schema
            // directory, and it does so in a blocking task like every other
            // filesystem call.
            let schema_files = if source.metadata_string_context("schema", position) {
                let config_dir = config_dir.clone();
                tokio::task::spawn_blocking(move || schema_reference_items(config_dir.as_deref()))
                    .await
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let schema = self.schema_for(&uri).await;
            let gather = self.gather_completion(&uri, &source, position).await;
            config_completion_items(
                schema.as_ref(),
                &source,
                position,
                &schema_files,
                gather.as_ref(),
            )
        };

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let Some(source) = self.document_source_for(&uri).await else {
            return Ok(None);
        };
        if is_schema_document(&uri, &source) {
            return Ok(None);
        }

        let position = params.text_document_position_params.position;

        // A `gather` statement explains the link it declares, resolved or not.
        if let Some(directive) = source.gather_at(position) {
            return Ok(self.gather_hover(&uri, &source, directive).await);
        }

        // A plain reference explains where its value really comes from.
        if let Some(run) = source.reference_at(position) {
            self.ensure_gather_closures(std::slice::from_ref(&uri))
                .await;

            return Ok(self
                .resolve_reference_at(&uri, run)
                .await
                .map(|site| Hover {
                    contents: HoverContents::Scalar(MarkedString::String(reference_hover_text(
                        &site,
                    ))),
                    range: None,
                }));
        }

        let Some(schema) = self.schema_for(&uri).await else {
            return Ok(None);
        };
        let Some(path) = source
            .field_key_at(position)
            .map(|entry| entry.path.clone())
        else {
            return Ok(None);
        };
        let Some(field) = find_field_by_path(&schema, &path) else {
            return Ok(None);
        };
        let schema_source = self.schema_source_label_for_document(&uri, &source).await;

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(field_hover(
                &path,
                field,
                schema_source.as_deref(),
            ))),
            range: None,
        }))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let Some(source) = self.document_source_for(&uri).await else {
            return Ok(None);
        };

        Ok(Some(DocumentSymbolResponse::Nested(document_symbols(
            &source,
        ))))
    }

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let mut actions = Vec::new();
        let source = self.document_source_for(&uri).await;
        let schema = self.schema_for(&uri).await;
        let sources = self.effective_sources_for(&uri).await;

        for diagnostic in params.context.diagnostics {
            // A value that really lives in a gathered file is not edited by a
            // fix that only writes this buffer.
            if let Some(path) = diagnostic_path(&diagnostic.message)
                && effective_origin(&sources, &path).is_some_and(|origin| origin.imported)
            {
                continue;
            }

            if diagnostic.message.contains("Unclosed object block") {
                // The fix belongs at the end of the buffer. A diagnostic may be
                // anchored on the block's opening key, and inserting there
                // would put `end` before the block's own fields.
                let insert = source
                    .as_deref()
                    .map(|source| source.lines().full_range().end)
                    .unwrap_or(diagnostic.range.start);

                actions.push(text_edit_action(
                    uri.clone(),
                    "Insert missing end",
                    TextEdit {
                        range: Range {
                            start: insert,
                            end: insert,
                        },
                        new_text: "end\n".into(),
                    },
                    diagnostic,
                    true,
                ));
                continue;
            }

            if let Some((path, values)) = enum_values_from_message(&diagnostic.message) {
                if let Some(range) = value_range_for_path(source.as_deref(), &path) {
                    for (index, value) in values.iter().enumerate() {
                        actions.push(text_edit_action(
                            uri.clone(),
                            format!("Replace with \"{}\"", value),
                            TextEdit {
                                range,
                                new_text: format!("\"{}\"", value),
                            },
                            diagnostic.clone(),
                            index == 0,
                        ));
                    }
                }
                continue;
            }

            if let Some((path, replacement)) = source
                .as_deref()
                .and_then(|source| type_fix_from_message(source, &diagnostic.message))
            {
                if let Some(range) = value_range_for_path(source.as_deref(), &path) {
                    actions.push(text_edit_action(
                        uri.clone(),
                        replacement.title,
                        TextEdit {
                            range,
                            new_text: replacement.new_text,
                        },
                        diagnostic.clone(),
                        true,
                    ));
                }
                continue;
            }

            if let (Some(schema), Some((parent_path, field_name))) = (
                schema.as_ref(),
                missing_required_field_from_message(&diagnostic.message),
            ) {
                let mut path = split_path(&parent_path);
                path.push(field_name.clone());
                let insert = source.as_deref().and_then(|source| {
                    insert_position_for_object(source, &split_path(&parent_path))
                });
                if let (Some(field), Some(insert)) = (find_field_by_path(schema, &path), insert) {
                    actions.push(text_edit_action(
                        uri.clone(),
                        format!("Insert missing field '{}'", field_name),
                        TextEdit {
                            range: Range {
                                start: insert.position,
                                end: insert.position,
                            },
                            new_text: format!(
                                "{}{} {}\n",
                                insert.indent,
                                field_name,
                                sample_value_for_field(field)
                            ),
                        },
                        diagnostic,
                        true,
                    ));
                }
                continue;
            }

            if let Some(reference) = missing_schema_from_message(&diagnostic.message) {
                let path = if is_schema_path_reference(&reference) {
                    expand_schema_path(
                        &reference,
                        uri.to_file_path()
                            .ok()
                            .as_deref()
                            .and_then(Path::parent)
                            .unwrap_or_else(|| Path::new(".")),
                    )
                } else {
                    uri.to_file_path()
                        .ok()
                        .as_deref()
                        .and_then(Path::parent)
                        .unwrap_or_else(|| Path::new("."))
                        .join("schemas")
                        .join(format!("{}.rune", reference))
                };

                if let Ok(schema_uri) = Url::from_file_path(path) {
                    let mut changes = HashMap::new();
                    changes.insert(
                        schema_uri,
                        vec![TextEdit {
                            range: Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 0),
                            },
                            new_text: "schema app:\n  name string required\nend\n".into(),
                        }],
                    );

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: format!("Create schema '{}'", reference),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diagnostic]),
                        edit: Some(WorkspaceEdit {
                            changes: Some(changes),
                            document_changes: None,
                            change_annotations: None,
                        }),
                        command: None,
                        is_preferred: Some(false),
                        disabled: None,
                        data: None,
                    }));
                }
            }
        }

        Ok(Some(actions))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let Some(source) = self.document_source_for(&uri).await else {
            return Ok(None);
        };
        if is_schema_document(&uri, &source) {
            return Ok(None);
        }

        let position = params.text_document_position_params.position;

        // 1. A `gather` statement links to the file it names.
        if let Some(directive) = source.gather_at(position) {
            return Ok(self
                .gather_target_definition(&uri, &source, directive)
                .await);
        }

        // 2. A plain reference links to the declaration it resolves to,
        // through the root's own gather view.
        if let Some(run) = source.reference_at(position) {
            self.ensure_gather_closures(std::slice::from_ref(&uri))
                .await;

            return Ok(self.resolve_reference_at(&uri, run).await.map(|site| {
                GotoDefinitionResponse::Scalar(Location {
                    uri: site.uri,
                    range: site.range.unwrap_or_else(zero_range),
                })
            }));
        }

        // 3. A `@schema "..."` directive jumps to the top of the schema file,
        // but only from the quoted reference itself: the `@schema` keyword and
        // the rest of that line are not the link.
        if schema_directive(&source).is_some_and(|directive| {
            source
                .offset_at(position)
                .is_some_and(|offset| directive.reference_span.touches(offset))
        }) {
            return Ok(self
                .schema_uri_for_document(&uri, &source)
                .await
                .map(|schema_uri| {
                    GotoDefinitionResponse::Scalar(Location {
                        uri: schema_uri,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    })
                }));
        }

        // 4. Otherwise jump from a config key to its schema field/block
        // definition.
        let Some(path) = source
            .field_key_at(position)
            .map(|entry| entry.path.clone())
        else {
            return Ok(None);
        };
        let Some(schema_text) = self.schema_text_for(&uri).await else {
            return Ok(None);
        };
        let Ok(schema) = SchemaDocument::from_str(&schema_text) else {
            return Ok(None);
        };
        let Some(schema_uri) = self.schema_uri_for_document(&uri, &source).await else {
            return Ok(None);
        };
        let Some(range) = schema_definition_range(&schema_text, &schema, &path) else {
            return Ok(None);
        };
        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri: schema_uri,
            range,
        })))
    }

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;
        let Some(source) = self.document_source_for(&uri).await else {
            return Ok(None);
        };

        // A `gather` link or a plain reference is answered from the gather
        // view, which is the only thing that knows what an alias resolves to.
        if let Some(locations) = self
            .gather_references(&uri, &source, position, include_declaration)
            .await
        {
            return Ok(Some(locations));
        }

        let Some((path, schema_uri)) = self.rename_target(&uri, &source, position).await else {
            return Ok(None);
        };

        // No schema: fall back to single-file references within this document.
        // Every indexed occurrence of the path is a usage, so
        // `includeDeclaration` removes none of them. A file that is itself
        // gathered answers with the uses through those gathers as well.
        let Some(schema_uri) = schema_uri else {
            let mut locations: Vec<Location> = references_in_document(&source, &path)
                .into_iter()
                .map(|range| Location {
                    uri: uri.clone(),
                    range,
                })
                .collect();
            locations.extend(
                self.imported_uses(&uri, &source, position, include_declaration)
                    .await,
            );

            return Ok(Some(dedupe_locations(locations)));
        };

        // Schema-scoped: declaration in the schema + usages across bound configs.
        let mut locations: Vec<Location> = self
            .cross_file_occurrences(&schema_uri, &path)
            .await
            .into_iter()
            .filter(|occurrence| include_declaration || !occurrence.is_declaration)
            .map(|occurrence| Location {
                uri: occurrence.uri,
                range: occurrence.range,
            })
            .collect();
        locations.extend(
            self.imported_uses(&uri, &source, position, include_declaration)
                .await,
        );

        Ok(Some(dedupe_locations(locations)))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> LspResult<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let position = params.position;
        let Some(source) = self.document_source_for(&uri).await else {
            return Ok(None);
        };

        // Inside a schema document, allow renaming a field declaration - but
        // only when the cursor is on the exact field-name token.
        if is_schema_document(&uri, &source) {
            return Ok(
                schema_field_name_range(source.text(), position).map(PrepareRenameResponse::Range)
            );
        }

        Ok(source
            .field_key_at(position)
            .map(|entry| PrepareRenameResponse::Range(source.key_range(entry))))
    }

    async fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        let new_name = params.new_name;
        if !is_identifier(&new_name) {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "New name must be a valid RUNE identifier",
            ));
        }

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let Some(source) = self.document_source_for(&uri).await else {
            return Ok(None);
        };
        let Some((path, schema_uri)) = self.rename_target(&uri, &source, position).await else {
            return Ok(None);
        };

        // The ranges to rewrite, one group per document, in the order they are
        // discovered. Nothing is edited when the request is rejected.
        let mut edits: Vec<(Url, Vec<Range>)> = Vec::new();

        match schema_uri {
            // No schema: single-file rename within this document.
            None => {
                let ranges = references_in_document(&source, &path);
                if ranges.is_empty() {
                    return Ok(None);
                }

                if let Some(candidate) = sibling_candidate(&path, &new_name)
                    && !source.entries_with_path(&candidate).is_empty()
                {
                    return Err(rename_collision(&path, &candidate));
                }

                edits.push((uri.clone(), ranges));
            }
            // Schema-scoped: update the schema declaration and every bound config.
            Some(schema_uri) => {
                let occurrences = self.cross_file_occurrences(&schema_uri, &path).await;
                if occurrences.is_empty() {
                    return Ok(None);
                }

                if let Some(candidate) = sibling_candidate(&path, &new_name) {
                    // The schema owns the name even where the rename would not
                    // edit its declaration, so it is always checked. Every
                    // other document is checked only where the rename would
                    // really edit it.
                    if self.field_exists_in(&schema_uri, true, &candidate).await {
                        return Err(rename_collision(&path, &candidate));
                    }

                    let mut checked: Vec<Url> = Vec::new();

                    for occurrence in &occurrences {
                        if occurrence.is_declaration || checked.contains(&occurrence.uri) {
                            continue;
                        }
                        checked.push(occurrence.uri.clone());

                        if self
                            .field_exists_in(&occurrence.uri, false, &candidate)
                            .await
                        {
                            return Err(rename_collision(&path, &candidate));
                        }
                    }
                }

                for occurrence in occurrences {
                    match edits
                        .iter_mut()
                        .find(|(existing, _)| *existing == occurrence.uri)
                    {
                        Some((_, ranges)) => ranges.push(occurrence.range),
                        None => edits.push((occurrence.uri, vec![occurrence.range])),
                    }
                }
            }
        }

        Ok(Some(self.rename_workspace_edit(edits, &new_name).await))
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> LspResult<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let Some(source) = self.document_source_for(&uri).await else {
            return Ok(None);
        };
        let Some(formatted) = format_document(&source) else {
            return Ok(None);
        };

        Ok(Some(vec![TextEdit {
            range: source.lines().full_range(),
            new_text: formatted,
        }]))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = params.text_document;
        let uri = document.uri.clone();

        // The opened buffer becomes the entry for its file, and any config
        // that was waiting for that file is re-resolved against it - an open
        // `schema.rune` with no disk file can rebind configs that were opened
        // before it.
        {
            let _gate = self.index_gate.lock().await;
            self.index_open_document(&uri, &document.text, document.version)
                .await;
            self.reresolve_dependents(std::slice::from_ref(&uri)).await;
        }

        // The opened buffer may be what its own imports load, so its closure is
        // established before it is validated.
        self.ensure_gather_closures(std::slice::from_ref(&uri))
            .await;

        // The opened document is the one that changed.
        let changed = std::slice::from_ref(&uri);
        let affected = self
            .refresh_dependencies_and_collect_affected(changed, changed)
            .await;
        let targets = union_of_targets(changed, &affected);
        self.validate_open_documents(&targets).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // A change without full text carries nothing to revalidate.
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };

        let uri = params.text_document.uri.clone();

        {
            let _gate = self.index_gate.lock().await;
            self.index_open_document(&uri, &change.text, params.text_document.version)
                .await;
            self.reresolve_dependents(std::slice::from_ref(&uri)).await;
        }
        self.ensure_gather_closures(std::slice::from_ref(&uri))
            .await;

        let changed = std::slice::from_ref(&uri);
        let affected = self
            .refresh_dependencies_and_collect_affected(changed, changed)
            .await;
        let targets = union_of_targets(changed, &affected);
        self.validate_open_documents(&targets).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        {
            let _gate = self.index_gate.lock().await;
            self.close_document(&uri).await;
            self.reresolve_dependents(std::slice::from_ref(&uri)).await;
        }
        self.document_dependencies.write().await.remove(&uri);

        self.client
            .publish_diagnostics(uri.clone(), Vec::new(), None)
            .await;

        // The buffer that is gone was the text its importers loaded, so their
        // own closures are refreshed before they are revalidated.
        let dependents = self.open_dependents_of(std::slice::from_ref(&uri)).await;
        self.ensure_gather_closures(&dependents).await;

        // Dependents now fall back to the on-disk text, the next candidate, or
        // none at all. No other document is revalidated.
        let affected = self
            .refresh_dependencies_and_collect_affected(std::slice::from_ref(&uri), &[])
            .await;
        self.validate_open_documents(&affected).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let mut changed: Vec<Url> = Vec::new();
        let mut removed: Vec<Url> = Vec::new();
        let mut availability: Vec<Url> = Vec::new();

        {
            // The whole batch is classified and applied under one gate hold.
            // The open buffers and the index membership the classification
            // reads are only stable in here: tower-lsp dispatches messages
            // concurrently, so a `didOpen` or `didChange` racing this batch
            // must never have its buffer replaced with disk text, and a delete
            // must never drop an entry the editor just opened.
            let _gate = self.index_gate.lock().await;
            let folders = self.workspace_paths().await;
            let open: HashSet<Url> = self.documents.read().await.keys().cloned().collect();

            for event in params.changes {
                if event.typ != FileChangeType::CREATED
                    && event.typ != FileChangeType::CHANGED
                    && event.typ != FileChangeType::DELETED
                {
                    continue;
                }

                // A file the editor holds a buffer for is the editor's business
                // alone, and the batch is deduplicated before it is reconciled.
                if !is_rune_file(&event.uri)
                    || open.contains(&event.uri)
                    || changed.contains(&event.uri)
                    || removed.contains(&event.uri)
                {
                    continue;
                }

                if event.typ == FileChangeType::DELETED {
                    removed.push(event.uri);
                    continue;
                }

                // Only a file the workspace owns, or one the index already
                // holds, is re-read: an event for a file inside an excluded
                // directory does not turn bulk indexing on for it.
                let eligible = match event.uri.to_file_path() {
                    Ok(path) => self.index.read().await.indexes_event(&path, &folders),
                    Err(_) => false,
                };
                if eligible {
                    changed.push(event.uri);
                }
            }

            if changed.is_empty() && removed.is_empty() {
                return;
            }

            availability.extend(changed.iter().chain(&removed).cloned());

            // Created and changed files replace exactly their own entry; no
            // parent directory is rescanned, so a file nobody reported stays
            // invisible.
            self.reindex_disk_files(&changed).await;
            {
                let mut index = self.index.write().await;
                for uri in &removed {
                    index.remove(uri);
                }
            }
            self.reresolve_dependents(&availability).await;
        }

        // One reconciliation against the final state, then only the open
        // configs the change affects. The unopened file publishes nothing
        // itself.
        let dependents = self.open_dependents_of(&availability).await;
        self.ensure_gather_closures(&dependents).await;

        let affected = self
            .refresh_dependencies_and_collect_affected(&availability, &[])
            .await;
        self.validate_open_documents(&affected).await;
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let mut folders = self.workspace_folders.read().await.clone();

        for removed in &params.event.removed {
            folders.retain(|folder| folder != &removed.uri);
        }
        for added in &params.event.added {
            if added.uri.scheme() != "file" || folders.contains(&added.uri) {
                continue;
            }
            folders.push(added.uri.clone());
        }

        *self.workspace_folders.write().await = folders;

        // The folders move the discovery boundary, so the index is scanned
        // again from scratch, with the client's exclusions carried on and the
        // open buffers overlaid.
        {
            let _gate = self.index_gate.lock().await;
            let excludes = self.index.read().await.excludes().clone();
            self.rebuild_index(excludes).await;
        }

        // A rebuild can drop a cached import a folder moved away from, so every
        // open config re-establishes its own closure first.
        let open: Vec<Url> = self.documents.read().await.keys().cloned().collect();
        self.ensure_gather_closures(&open).await;

        // Only a config whose resolved schema URI or gather view changed is
        // revalidated.
        let affected = self
            .refresh_dependencies_and_collect_affected(&[], &[])
            .await;
        self.validate_open_documents(&affected).await;
    }
}

fn is_rune_file(uri: &Url) -> bool {
    uri.to_file_path()
        .ok()
        .and_then(|path| path.extension().map(|extension| extension == "rune"))
        .unwrap_or(false)
}

/// The union of the forced documents and the configs a change affects, in that
/// order and without duplicates: forced schema documents publish their own
/// diagnostics, and forced configs are already part of the affected set.
fn union_of_targets(forced: &[Url], affected: &[Url]) -> Vec<Url> {
    let mut targets: Vec<Url> = Vec::new();

    for uri in forced.iter().chain(affected) {
        if !targets.contains(uri) {
            targets.push(uri.clone());
        }
    }

    targets
}

/// One location per document and range, in the order they were discovered.
fn dedupe_locations(locations: Vec<Location>) -> Vec<Location> {
    let mut unique: Vec<Location> = Vec::new();

    for location in locations {
        let duplicate = unique.iter().any(|existing| {
            existing.uri == location.uri && ranges_match(&existing.range, &location.range)
        });
        if !duplicate {
            unique.push(location);
        }
    }

    unique
}

/// True when two ranges cover the same text.
fn ranges_match(left: &Range, right: &Range) -> bool {
    left.start == right.start && left.end == right.end
}

/// Workspace folders from `initialize`, applying the LSP fallback rules.
///
/// Folders the client sent win, even an explicit empty list, because the client
/// is stating the workspace it has. A client that supports workspace folders
/// but sent none has a workspace without folders, so nothing may be discovered
/// from `rootUri`. Only a client that does not use workspace folders at all
/// falls back to `rootUri` as a single-folder workspace. Non-file URIs are
/// ignored everywhere.
fn workspace_folders_from_initialize(params: &InitializeParams) -> Vec<Url> {
    if let Some(folders) = &params.workspace_folders {
        return folders
            .iter()
            .map(|folder| folder.uri.clone())
            .filter(|uri| uri.scheme() == "file")
            .collect();
    }

    let client_uses_workspace_folders = params
        .capabilities
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.workspace_folders)
        == Some(true);
    if client_uses_workspace_folders {
        return Vec::new();
    }

    params
        .root_uri
        .iter()
        .filter(|uri| uri.scheme() == "file")
        .cloned()
        .collect()
}

/// True when the client declared `workspace.workspaceEdit.documentChanges`,
/// which is what makes a versioned document-changes edit acceptable.
fn supports_document_changes(params: &InitializeParams) -> bool {
    params
        .capabilities
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.workspace_edit.as_ref())
        .and_then(|workspace_edit| workspace_edit.document_changes)
        == Some(true)
}

/// The `@schema` value completion: the schema files found in the schema
/// directories, then the two fixed relative entries.
fn schema_reference_completion_items(schema_files: &[(String, PathBuf)]) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = schema_files
        .iter()
        .map(|(label, path)| CompletionItem {
            label: label.clone(),
            kind: Some(CompletionItemKind::FILE),
            detail: Some(format!("schema: {}", path.display())),
            insert_text: Some(label.clone()),
            ..CompletionItem::default()
        })
        .collect();

    items.push(CompletionItem {
        label: "./schema.rune".into(),
        kind: Some(CompletionItemKind::FILE),
        detail: Some("relative schema path".into()),
        ..CompletionItem::default()
    });
    items.push(CompletionItem {
        label: "./schemas/".into(),
        kind: Some(CompletionItemKind::FOLDER),
        detail: Some("relative schema directory".into()),
        ..CompletionItem::default()
    });

    items
}

fn schema_reference_diagnostic(
    lines: &LineIndex,
    directive: &SchemaDirective,
    candidates: &[PathBuf],
) -> RuneDiagnostic {
    let hint = if candidates.is_empty() {
        "Check the @schema path or install the named schema".to_string()
    } else {
        format!(
            "Checked: {}",
            candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    // The range is the quoted reference token itself, counted in UTF-16 code
    // units so a non-ASCII reference is underlined exactly.
    let range = lines.rune_range(directive.reference_span);

    RuneDiagnostic::error(format!("Schema '{}' was not found", directive.reference))
        .with_range(range.start.line, range.start.column, range.end.column)
        .with_hint(hint)
        .with_code(701)
}

/// Diagnostics for a buffer whose strict parse failed, derived from the same
/// indexed entries the rest of the server uses.
///
/// The messages are the ones the server has always published: a key without a
/// value, a closing keyword with nothing to close, and an object still open at
/// the end of the buffer.
fn recovery_diagnostics(source: &SourceIndex) -> Vec<RuneDiagnostic> {
    let lines = source.lines();
    let mut positional: Vec<(Span, RuneDiagnostic)> = Vec::new();

    for entry in source.missing_value_entries() {
        let name = entry.name.clone().unwrap_or_default();
        let diagnostic = RuneDiagnostic::error(format!("Missing value for '{}'", name))
            .with_hint("Add a value after the key");
        positional.push((entry.key_span, diagnostic));
    }

    for closer in source.stray_closers() {
        let keyword = closer.kind.keyword();
        let diagnostic =
            RuneDiagnostic::error(format!("Unexpected '{}' without open block", keyword))
                .with_hint("Remove this closing keyword or add a matching block above");
        positional.push((closer.span, diagnostic));
    }

    for entry in source.entries() {
        if entry.kind != SourceEntryKind::Metadata || entry.name.as_deref() != Some("schema") {
            continue;
        }

        let reference = entry
            .value_span
            .and_then(|span| source.text().get(span.start..span.end))
            .and_then(|raw| parse_quoted_string(raw.trim_start()));

        if reference.is_none() {
            let diagnostic = RuneDiagnostic::error("Malformed @schema directive")
                .with_hint("Use: @schema \"name\" or @schema \"./schema.rune\"");
            positional.push((entry.key_span, diagnostic));
        }
    }

    positional.sort_by_key(|(span, _)| span.start);
    let mut diagnostics: Vec<RuneDiagnostic> = positional
        .into_iter()
        .map(|(span, diagnostic)| diagnostic_at(lines, span, diagnostic))
        .collect();

    for entry in source.unclosed_objects() {
        let name = entry.name.clone().unwrap_or_default();
        let diagnostic =
            RuneDiagnostic::error(format!("Unclosed object block '{}'; expected 'end'", name))
                .with_hint(format!("Add 'end' to close the '{}' block", name));
        diagnostics.push(diagnostic_at(lines, entry.key_span, diagnostic));
    }

    diagnostics
}

/// Attach the 1-based, UTF-16 range of `span` to a diagnostic.
fn diagnostic_at(lines: &LineIndex, span: Span, diagnostic: RuneDiagnostic) -> RuneDiagnostic {
    let range = lines.rune_range(span);
    diagnostic.with_range(range.start.line, range.start.column, range.end.column)
}

fn dedupe_diagnostics(diagnostics: &mut Vec<RuneDiagnostic>) {
    let mut seen = Vec::new();
    diagnostics.retain(|diagnostic| {
        let key = diagnostic.message.clone();
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}

fn text_edit_action(
    uri: Url,
    title: impl Into<String>,
    edit: TextEdit,
    diagnostic: tower_lsp::lsp_types::Diagnostic,
    is_preferred: bool,
) -> CodeActionOrCommand {
    let mut changes = HashMap::new();
    changes.insert(uri, vec![edit]);

    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.into(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(is_preferred),
        disabled: None,
        data: None,
    })
}

fn enum_values_from_message(message: &str) -> Option<(Vec<String>, Vec<String>)> {
    let path_start = message.find('\'')? + 1;
    let path_end = message[path_start..].find('\'')? + path_start;
    let path = split_path(&message[path_start..path_end]);
    let values = message
        .split_once("must be one of:")?
        .1
        .lines()
        .next()
        .unwrap_or_default()
        .split(',')
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    (!path.is_empty() && !values.is_empty()).then_some((path, values))
}

fn missing_required_field_from_message(message: &str) -> Option<(String, String)> {
    let field_start = message.find('\'')? + 1;
    let field_end = message[field_start..].find('\'')? + field_start;
    let after_field = &message[(field_end + 1)..];
    let parent_marker = "inside '";
    let parent_start = after_field.find(parent_marker)? + parent_marker.len();
    let parent_end = after_field[parent_start..].find('\'')? + parent_start;

    Some((
        after_field[parent_start..parent_end].to_string(),
        message[field_start..field_end].to_string(),
    ))
}

fn missing_schema_from_message(message: &str) -> Option<String> {
    let rest = message.strip_prefix("Schema '")?;
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

#[derive(Debug, Clone)]
struct TypeReplacement {
    title: String,
    new_text: String,
}

fn type_fix_from_message(
    source: &SourceIndex,
    message: &str,
) -> Option<(Vec<String>, TypeReplacement)> {
    let path_start = message.find('\'')? + 1;
    let path_end = message[path_start..].find('\'')? + path_start;
    let path = split_path(&message[path_start..path_end]);
    let expected = message
        .split_once(" expected ")?
        .1
        .split(',')
        .next()?
        .trim();
    let got = message.split_once(" got ")?.1.lines().next()?.trim();
    let range = value_range_for_path(Some(source), &path)?;
    let value = source.text_in_range(range)?.trim().to_string();

    if matches!(expected, "int" | "float" | "number" | "bool") && got == "string" {
        let unquoted = value.trim_matches('"').trim_matches('\'').to_string();
        if !unquoted.is_empty() {
            return Some((
                path,
                TypeReplacement {
                    title: format!("Remove quotes to make {}", expected),
                    new_text: unquoted,
                },
            ));
        }
    }

    if expected == "string" && got != "string" {
        return Some((
            path,
            TypeReplacement {
                title: "Add quotes to make string".into(),
                new_text: format!("\"{}\"", value.trim_matches('"').trim_matches('\'')),
            },
        ));
    }

    None
}

fn split_path(path: &str) -> Vec<String> {
    path.split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

#[derive(Debug, Clone)]
struct InsertPosition {
    position: Position,
    indent: String,
}

fn insert_position_for_object(source: &SourceIndex, path: &[String]) -> Option<InsertPosition> {
    let (position, indent) = source.object_body_insert(path)?;

    Some(InsertPosition { position, indent })
}

/// Range of the value tokens of the assignment at `path`.
fn value_range_for_path(source: Option<&SourceIndex>, path: &[String]) -> Option<Range> {
    let source = source?;
    let span = source.value_span_for_path(path)?;

    Some(source.lines().range(span))
}

fn sample_value_for_field(field: &SchemaField) -> String {
    if let Some(default) = &field.default {
        return value_literal(default);
    }

    match &field.kind {
        SchemaType::String => "\"\"".into(),
        SchemaType::Int | SchemaType::Float | SchemaType::Number => "0".into(),
        SchemaType::Bool => "false".into(),
        SchemaType::Regex => "r\"\"".into(),
        SchemaType::Null => "null".into(),
        SchemaType::Any => "null".into(),
        SchemaType::Array(_) => "[]".into(),
        SchemaType::Enum(values) => values
            .first()
            .map(|value| format!("\"{}\"", value))
            .unwrap_or_else(|| "\"\"".into()),
        SchemaType::Object => "".into(),
    }
}

fn value_literal(value: &crate::Value) -> String {
    match value {
        crate::Value::String(value) => format!("\"{}\"", value),
        crate::Value::Number(value) => value.to_string(),
        crate::Value::Bool(value) => value.to_string(),
        crate::Value::Null => "null".into(),
        crate::Value::Array(_) => "[]".into(),
        crate::Value::Object(_) => "".into(),
        crate::Value::Regex(pattern) => format!("r\"{}\"", pattern.as_str()),
        crate::Value::Reference(reference) => reference.join("."),
        crate::Value::Interpolated(_) => "\"\"".into(),
        crate::Value::Conditional(_) => "null".into(),
        crate::Value::Annotated(value) => value_literal(&value.value),
    }
}

/// Convert a library error into a diagnostic whose columns are UTF-16 code
/// units: the lexer and parser count columns in `char`s, so the line is needed
/// to translate them.
fn diagnostic_from_error(error: RuneError, lines: &LineIndex) -> RuneDiagnostic {
    match error {
        RuneError::SyntaxError {
            message,
            line,
            column,
            hint,
            code,
        }
        | RuneError::InvalidToken {
            token: message,
            line,
            column,
            hint,
            code,
        }
        | RuneError::UnexpectedEof {
            message,
            line,
            column,
            hint,
            code,
        }
        | RuneError::TypeError {
            message,
            line,
            column,
            hint,
            code,
        } => diagnostic_with_location(message, line, lines.utf16_column(line, column), hint, code),
        RuneError::UnclosedString {
            quote: _,
            line,
            column,
            hint,
            code,
        } => diagnostic_with_location(
            "Unclosed string literal",
            line,
            lines.utf16_column(line, column),
            hint,
            code,
        ),
        RuneError::UnexpectedCharacter {
            character,
            line,
            column,
            hint,
            code,
        } => diagnostic_with_location(
            format!("Unexpected character '{}'", character),
            line,
            lines.utf16_column(line, column),
            hint,
            code,
        ),
        RuneError::ValidationError {
            message,
            line,
            column,
            hint,
            code,
        } => diagnostic_with_location(message, line, lines.utf16_column(line, column), hint, code),
        RuneError::FileError {
            message,
            path,
            hint,
            code,
        } => {
            let mut diagnostic = RuneDiagnostic::error(format!("{}: {}", path, message));
            if let Some(code) = code {
                diagnostic = diagnostic.with_code(code);
            }
            if let Some(hint) = hint {
                diagnostic = diagnostic.with_hint(hint);
            }
            diagnostic
        }
        RuneError::RuntimeError {
            message,
            hint,
            code,
        } => {
            let mut diagnostic = RuneDiagnostic::error(message);
            if let Some(code) = code {
                diagnostic = diagnostic.with_code(code);
            }
            if let Some(hint) = hint {
                diagnostic = diagnostic.with_hint(hint);
            }
            diagnostic
        }
    }
}

fn diagnostic_with_location(
    message: impl Into<String>,
    line: usize,
    column: usize,
    hint: Option<String>,
    code: Option<u32>,
) -> RuneDiagnostic {
    let message = message.into();
    let message = if message.is_empty() {
        "RUNE syntax error".to_string()
    } else {
        message
    };

    let mut diagnostic = RuneDiagnostic::error(message).with_line(line, column);
    if let Some(code) = code {
        diagnostic = diagnostic.with_code(code);
    }
    if let Some(hint) = hint {
        diagnostic = diagnostic.with_hint(hint);
    }
    diagnostic
}

fn lsp_diagnostic_from_rune(diagnostic: RuneDiagnostic) -> tower_lsp::lsp_types::Diagnostic {
    let range = diagnostic.range.map(lsp_range).unwrap_or_else(|| Range {
        start: Position::new(0, 0),
        end: Position::new(0, 1),
    });

    let message = if let Some(hint) = diagnostic.hint {
        format!("{}\nHint: {}", diagnostic.message, hint)
    } else {
        diagnostic.message
    };

    tower_lsp::lsp_types::Diagnostic {
        range,
        severity: Some(match diagnostic.severity {
            DiagnosticSeverity::Error => tower_lsp::lsp_types::DiagnosticSeverity::ERROR,
            DiagnosticSeverity::Warning => tower_lsp::lsp_types::DiagnosticSeverity::WARNING,
            DiagnosticSeverity::Information => {
                tower_lsp::lsp_types::DiagnosticSeverity::INFORMATION
            }
            DiagnosticSeverity::Hint => tower_lsp::lsp_types::DiagnosticSeverity::HINT,
        }),
        code: diagnostic
            .code
            .map(|code| tower_lsp::lsp_types::NumberOrString::Number(code as i32)),
        code_description: None,
        source: Some("rune-cfg".into()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// The diagnostic code an import that could not be loaded is reported with.
const GATHER_LOAD_CODE: i32 = 702;

/// Everything one root document's diagnostics and requests need about its
/// `gather` statements.
struct GatherContext {
    /// The view the root's own buffer produces, root-relative as the loader is.
    view: GatherView,
    /// The files that merge into the root's effective document, lowest
    /// precedence first.
    sources: Vec<EffectiveSource>,
    /// The buffers a config load prefers over disk.
    overlay: Vec<(PathBuf, Arc<SourceIndex>)>,
}

/// The range a diagnostic without an anchor gets, which is the shape the server
/// has always used.
fn default_range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 1))
}

/// The message and the parser position of an error, when it has one.
fn error_detail(error: &RuneError) -> (String, Option<(usize, usize)>) {
    match error {
        RuneError::SyntaxError {
            message,
            line,
            column,
            ..
        }
        | RuneError::InvalidToken {
            token: message,
            line,
            column,
            ..
        }
        | RuneError::UnexpectedEof {
            message,
            line,
            column,
            ..
        }
        | RuneError::TypeError {
            message,
            line,
            column,
            ..
        }
        | RuneError::ValidationError {
            message,
            line,
            column,
            ..
        } => (message.clone(), Some((*line, *column))),
        RuneError::UnclosedString { line, column, .. } => {
            ("Unclosed string literal".into(), Some((*line, *column)))
        }
        RuneError::UnexpectedCharacter {
            character,
            line,
            column,
            ..
        } => (
            format!("Unexpected character '{}'", character),
            Some((*line, *column)),
        ),
        RuneError::FileError { message, .. } => (message.clone(), None),
        RuneError::RuntimeError { message, .. } => (message.clone(), None),
    }
}

/// The range of a parser position inside an imported file.
///
/// The position is 1-based and `char`-counted, as the lexer and parser report
/// it, so the imported buffer is what converts it when it is known.
fn error_range(source: Option<&SourceIndex>, position: Option<(usize, usize)>) -> Range {
    let Some((line, column)) = position else {
        return Range::new(Position::new(0, 0), Position::new(0, 0));
    };

    let character = match source {
        Some(source) => source.lines().utf16_column(line, column),
        None => column,
    };
    let line = line.saturating_sub(1) as u32;
    let start = character.saturating_sub(1) as u32;

    Range::new(Position::new(line, start), Position::new(line, start + 1))
}

/// The root's own `gather` statement a failure on the imported side belongs to:
/// the first hop that reached the failing file, or, when the failure came from
/// the path itself, the statement that wrote it.
fn gather_anchor<'a>(
    source: &'a SourceIndex,
    view: &GatherView,
    failure: &ImportFailure,
) -> Option<&'a GatherDirective> {
    // The loader resolves a path without lexical normalization, while the view
    // keys its first hops by the normalized spelling: normalize here so `..`
    // and `.` spellings still find the hop that reached the failing file.
    let target = Url::from_file_path(normalize_path(&failure.resolved_path)).ok();
    let hop = target
        .as_ref()
        .and_then(|target| view.first_hop.get(target))
        .copied();
    let edge = hop.and_then(|hop| view.direct.get(hop)).or_else(|| {
        view.direct
            .iter()
            .find(|edge| edge.raw_path == failure.raw_path)
    });

    match edge {
        Some(edge) => source.gathers().get(edge.directive),
        // A failure the view cannot attribute still belongs to a `gather`
        // statement, and the first one is the closest anchor there is.
        None => source.gathers().first(),
    }
}

/// Re-anchor a schema diagnostic whose effective value lives in a gathered
/// file, and link the imported declaration it really describes.
///
/// A diagnostic about the root's own fields keeps exactly the range, hint and
/// quick fixes it always had.
fn import_aware_diagnostic(
    source: &SourceIndex,
    context: &GatherContext,
    diagnostic: RuneDiagnostic,
) -> tower_lsp::lsp_types::Diagnostic {
    let Some(origin) = diagnostic_path(&diagnostic.message)
        .and_then(|path| effective_origin(&context.sources, &path))
        .filter(|origin| origin.imported)
    else {
        return lsp_diagnostic_from_rune(diagnostic);
    };

    let mut rune_diagnostic = diagnostic;

    // The value is not written in this buffer, so the statement that brought it
    // in is what the diagnostic belongs to, and the marker keeps a quick fix
    // from editing the root for a field it does not own.
    if let Some(statement) = origin
        .first_hop
        .and_then(|hop| context.view.direct.get(hop))
        .and_then(|edge| source.gathers().get(edge.directive))
    {
        let range = source.lines().rune_range(statement.path_span);
        rune_diagnostic =
            rune_diagnostic.with_range(range.start.line, range.start.column, range.end.column);

        let marker = format!("Provided by gather \"{}\"", statement.raw_path);
        rune_diagnostic.hint = Some(match rune_diagnostic.hint.take() {
            Some(hint) => format!("{hint}\n{marker}"),
            None => marker,
        });
    }

    let related = imported_source(&context.sources, &origin.uri).map(|imported| {
        vec![tower_lsp::lsp_types::DiagnosticRelatedInformation {
            location: Location {
                uri: origin.uri.clone(),
                range: imported.lines().range(origin.key_span),
            },
            message: format!("'{}' is declared here", origin.path.join(".")),
        }]
    });

    let mut diagnostic = lsp_diagnostic_from_rune(rune_diagnostic);
    diagnostic.related_information = related;
    diagnostic
}

/// The source of one effective file, when it takes part in the merge.
fn imported_source<'a>(sources: &'a [EffectiveSource], uri: &Url) -> Option<&'a Arc<SourceIndex>> {
    sources
        .iter()
        .find(|source| source.uri == *uri)
        .map(|source| &source.source)
}

/// The config path a validation message is about.
///
/// The messages are the ones `validate_schema_with_source` produces, read the
/// same way the quick-fix parsers already read them. An array item message
/// names the item (`app.plugins[1]`), which is still supplied by the field that
/// declares the array, so the index notation is dropped here: the path has to
/// find the declaration that really provides the value.
fn diagnostic_path(message: &str) -> Option<Vec<String>> {
    let path = message_path(message)?;
    Some(
        path.iter()
            .map(|segment| strip_item_indexes(segment).to_string())
            .collect(),
    )
}

fn message_path(message: &str) -> Option<Vec<String>> {
    if let Some(rest) = message.strip_prefix('\'') {
        let end = rest.find('\'')?;
        return Some(split_path(&rest[..end]));
    }

    if let Some((parent, field)) = missing_required_field_from_message(message) {
        let mut path = split_path(&parent);
        path.push(field);
        return Some(path);
    }

    let rest = message.strip_prefix("Missing required config path '")?;
    let end = rest.find('\'')?;
    Some(split_path(&rest[..end]))
}

/// `plugins[1]` names the same declared field as `plugins`, and `grid[1][2]`
/// the same field as `grid`: only trailing numeric item indexes are removed.
fn strip_item_indexes(segment: &str) -> &str {
    let mut segment = segment;

    while let Some(rest) = segment.strip_suffix(']') {
        let Some(open) = rest.rfind('[') else {
            break;
        };
        let inner = &rest[open + 1..];
        if inner.is_empty() || !inner.bytes().all(|byte| byte.is_ascii_digit()) {
            break;
        }
        segment = &rest[..open];
    }

    segment
}

/// What a document's gather view adds to one completion.
struct GatherCompletion {
    /// Direct child keys of the reference prefix, when the cursor follows the
    /// `.` of an alias-qualified reference.
    prefix_keys: Option<Vec<(String, SourceEntryKind)>>,
    /// The keys the unaliased gathers supply at the cursor's scope, which the
    /// document has even though it does not write them.
    supplied: Vec<String>,
    /// True when the cursor sits where a reference is being typed.
    reference_position: bool,
}

/// A reference the cursor is on, resolved through a root's gather view.
struct ResolvedSite {
    uri: Url,
    /// Where the declaration is written, when the reference points at a field.
    key: Option<DeclarationKey>,
    /// Range of that declaration inside `uri`.
    range: Option<Range>,
    /// The key path inside `uri`; empty for a whole imported file.
    path: Vec<String>,
    /// What kind of entry declares the value.
    kind: Option<SourceEntryKind>,
    /// True when the declaration lives in a gathered file.
    imported: bool,
}

/// The zero-width range at the top of a file.
fn zero_range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 0))
}

/// The position of one gather statement among its own file's statements.
fn gather_position(source: &SourceIndex, directive: &GatherDirective) -> Option<usize> {
    source
        .gathers()
        .iter()
        .position(|gather| gather.path_span == directive.path_span)
}

/// The hover text of one `gather` statement: what it resolves to, the alias it
/// binds, and whether it includes or namespaces the file.
fn gather_hover_text(edge: &GatherEdge, resolved: bool) -> String {
    let target = match &edge.uri {
        Some(uri) => uri.to_string(),
        None => edge.path.display().to_string(),
    };
    let alias = if edge.explicit_alias {
        "explicit"
    } else {
        "default"
    };
    let mode = if edge.explicit_alias {
        "namespaced import"
    } else {
        "include"
    };

    let mut lines = vec![
        format!("gather: {}", target),
        format!("alias: {} ({})", edge.alias, alias),
        format!("mode: {}", mode),
    ];

    if !resolved {
        lines.push("The gathered file is not there, or cannot be read".into());
    }

    lines.join("\n")
}

/// The hover text of one resolved reference: where the value comes from and
/// what kind of entry declares it. A value is never shown.
fn reference_hover_text(site: &ResolvedSite) -> String {
    let key = if site.path.is_empty() {
        "(the file itself)".to_string()
    } else {
        site.path.join(".")
    };

    let mut lines = vec![format!("reference: {}", site.uri), format!("key: {}", key)];

    if let Some(kind) = site.kind {
        lines.push(format!("kind: {}", entry_kind_label(kind)));
    }
    lines.push(if site.imported {
        "mode: gathered".into()
    } else {
        "mode: local".into()
    });

    lines.join("\n")
}

/// How one structural entry kind is described in a hover.
fn entry_kind_label(kind: SourceEntryKind) -> &'static str {
    match kind {
        SourceEntryKind::Assignment => "field",
        SourceEntryKind::Object => "object",
        SourceEntryKind::Gather => "gather",
        SourceEntryKind::Metadata => "metadata",
        SourceEntryKind::ConditionalHeader | SourceEntryKind::ConditionalBranch => "conditional",
        SourceEntryKind::ConditionalEnd => "conditional end",
    }
}

fn lsp_range(range: crate::SourceRange) -> Range {
    Range {
        start: lsp_position(range.start.line, range.start.column),
        end: lsp_position(range.end.line, range.end.column),
    }
}

fn lsp_position(line: usize, column: usize) -> Position {
    Position::new(
        line.saturating_sub(1) as u32,
        column.saturating_sub(1) as u32,
    )
}

fn schema_completion_items() -> Vec<CompletionItem> {
    [
        ("schema", CompletionItemKind::KEYWORD),
        ("string", CompletionItemKind::TYPE_PARAMETER),
        ("int", CompletionItemKind::TYPE_PARAMETER),
        ("float", CompletionItemKind::TYPE_PARAMETER),
        ("number", CompletionItemKind::TYPE_PARAMETER),
        ("bool", CompletionItemKind::TYPE_PARAMETER),
        ("regex", CompletionItemKind::TYPE_PARAMETER),
        ("any", CompletionItemKind::TYPE_PARAMETER),
        ("object", CompletionItemKind::TYPE_PARAMETER),
        ("required", CompletionItemKind::KEYWORD),
        ("default", CompletionItemKind::KEYWORD),
        ("range", CompletionItemKind::KEYWORD),
        ("end", CompletionItemKind::KEYWORD),
    ]
    .into_iter()
    .map(|(label, kind)| keyword_completion(label, kind))
    .collect()
}

fn config_completion_items(
    schema: Option<&SchemaDocument>,
    source: &SourceIndex,
    position: Position,
    schema_files: &[(String, PathBuf)],
    gather: Option<&GatherCompletion>,
) -> Vec<CompletionItem> {
    // Only a positively special context replaces the normal completion: a
    // cursor on a `$...` reference run, or inside the quoted value of the
    // `@schema` directive. Anything else is a field, enum, or keyword.
    if source.metadata_string_context("schema", position) {
        return schema_reference_completion_items(schema_files);
    }

    if source.dollar_reference_context(position) {
        return dollar_reference_completion_items();
    }

    // Right after the `.` of an alias-qualified reference the answer is the
    // imported document's own children, and nothing else is suggested.
    if let Some(keys) = gather.and_then(|gather| gather.prefix_keys.as_ref()) {
        return keys
            .iter()
            .map(|(name, kind)| child_completion_item(name, *kind))
            .collect();
    }

    let scope = source.scope_at(position);

    if let Some(schema) = schema
        && let Some(field_path) = source
            .assignment_before_cursor(position)
            .map(|entry| entry.path.clone())
        && let Some(field) = find_field_by_path(schema, &field_path)
        && let SchemaType::Enum(values) = &field.kind
    {
        return values
            .iter()
            .map(|value| CompletionItem {
                label: format!("\"{}\"", value),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some("enum value".into()),
                insert_text: Some(format!("\"{}\"", value)),
                ..CompletionItem::default()
            })
            .collect();
    }

    // A key an unaliased gather already supplies is present in this document,
    // so the schema fields it satisfies are not offered a second time.
    let mut used = source.used_keys_in_scope(&scope, position.line);
    if let Some(gather) = gather {
        for name in &gather.supplied {
            if !used.contains(name) {
                used.push(name.clone());
            }
        }
    }

    let mut items = if let Some(schema) = schema {
        schema_field_completion_items(schema, &scope, &used)
    } else {
        Vec::new()
    };

    // A reference is completed with the keys the gathers supply, which the
    // schema never names.
    if let Some(gather) = gather
        && gather.reference_position
    {
        items.extend(
            gather
                .supplied
                .iter()
                .map(|name| child_completion_item(name, SourceEntryKind::Assignment)),
        );
    }

    items.extend(
        ["end", "if", "else", "endif", "gather"]
            .into_iter()
            .map(|label| keyword_completion(label, CompletionItemKind::KEYWORD)),
    );

    items
}

/// One completion of a key a gathered document declares.
fn child_completion_item(name: &str, kind: SourceEntryKind) -> CompletionItem {
    CompletionItem {
        label: name.to_string(),
        kind: Some(if kind == SourceEntryKind::Object {
            CompletionItemKind::STRUCT
        } else {
            CompletionItemKind::FIELD
        }),
        detail: Some("gathered".into()),
        ..CompletionItem::default()
    }
}

fn schema_field_completion_items(
    schema: &SchemaDocument,
    stack: &[String],
    used: &[String],
) -> Vec<CompletionItem> {
    if stack.is_empty() {
        return schema
            .blocks
            .iter()
            .map(|block| CompletionItem {
                label: block.root.clone(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("schema root".into()),
                insert_text: Some(format!("{}:\n  $0\nend", block.root)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..CompletionItem::default()
            })
            .collect();
    }

    fields_for_stack(schema, stack)
        .unwrap_or(&[])
        .iter()
        .filter(|field| !used.contains(&field.name))
        .map(field_completion_item)
        .collect()
}

fn field_completion_item(field: &SchemaField) -> CompletionItem {
    let is_object = matches!(field.kind, SchemaType::Object) || !field.fields.is_empty();

    CompletionItem {
        label: field.name.clone(),
        kind: Some(if is_object {
            CompletionItemKind::STRUCT
        } else {
            CompletionItemKind::FIELD
        }),
        detail: Some(schema_type_label(&field.kind)),
        documentation: Some(tower_lsp::lsp_types::Documentation::String(field_hover(
            std::slice::from_ref(&field.name),
            field,
            None,
        ))),
        insert_text: Some(if is_object {
            format!("{}:\n  $0\nend", field.name)
        } else {
            field_snippet(field)
        }),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..CompletionItem::default()
    }
}

fn field_snippet(field: &SchemaField) -> String {
    match &field.kind {
        SchemaType::String => format!("{} \"$1\"", field.name),
        SchemaType::Int | SchemaType::Float | SchemaType::Number => format!("{} $1", field.name),
        SchemaType::Bool => format!("{} ${{1|true,false|}}", field.name),
        SchemaType::Regex => format!("{} r\"$1\"", field.name),
        SchemaType::Null => format!("{} null", field.name),
        SchemaType::Any => format!("{} $1", field.name),
        SchemaType::Array(_) => format!("{} [$1]", field.name),
        SchemaType::Enum(values) if values.is_empty() => format!("{} \"$1\"", field.name),
        SchemaType::Enum(values) => format!(
            "{} ${{1|{}|}}",
            field.name,
            values
                .iter()
                .map(|value| format!("\"{}\"", value))
                .collect::<Vec<_>>()
                .join(",")
        ),
        SchemaType::Object => format!("{}:\n  $0\nend", field.name),
    }
}

fn keyword_completion(label: &str, kind: CompletionItemKind) -> CompletionItem {
    CompletionItem {
        label: label.into(),
        kind: Some(kind),
        ..CompletionItem::default()
    }
}

fn dollar_reference_completion_items() -> Vec<CompletionItem> {
    [
        "$env.",
        "$sys.hostname",
        "$sys.os",
        "$sys.arch",
        "$sys.cpu_count",
        "$sys.memory_total",
        "$runtime.",
        "$var.",
    ]
    .into_iter()
    .map(|label| CompletionItem {
        label: label.into(),
        kind: Some(CompletionItemKind::VARIABLE),
        detail: Some("RUNE reference".into()),
        ..CompletionItem::default()
    })
    .collect()
}

fn field_hover(path: &[String], field: &SchemaField, schema_source: Option<&str>) -> String {
    let mut lines = vec![format!(
        "{}: {}",
        path.join("."),
        schema_type_label(&field.kind)
    )];

    if let Some(description) = &field.description {
        lines.push(String::new());
        lines.push(description.clone());
    }

    if field.required {
        lines.push("required".into());
    }
    if let Some((min, max)) = field.range {
        lines.push(format!("range: {}..{}", min, max));
    }
    if let Some(default) = &field.default {
        lines.push(format!("default: {:?}", default));
    }
    if let SchemaType::Enum(values) = &field.kind {
        lines.push(format!("values: {}", values.join(", ")));
    }
    if let Some(schema_source) = schema_source {
        lines.push(format!("schema: {}", schema_source));
    }

    lines.join("\n")
}

fn schema_type_label(kind: &SchemaType) -> String {
    match kind {
        SchemaType::String => "string".into(),
        SchemaType::Int => "int".into(),
        SchemaType::Float => "float".into(),
        SchemaType::Number => "number".into(),
        SchemaType::Bool => "bool".into(),
        SchemaType::Regex => "regex".into(),
        SchemaType::Null => "null".into(),
        SchemaType::Any => "any".into(),
        SchemaType::Array(inner) => format!("[{}]", schema_type_label(inner)),
        SchemaType::Enum(values) => format!("enum [{}]", values.join(", ")),
        SchemaType::Object => "object".into(),
    }
}

fn find_field_by_path<'a>(schema: &'a SchemaDocument, path: &[String]) -> Option<&'a SchemaField> {
    let (root, rest) = path.split_first()?;
    let block = schema.blocks.iter().find(|block| block.root == *root)?;
    find_nested_field(&block.fields, rest)
}

fn find_nested_field<'a>(fields: &'a [SchemaField], path: &[String]) -> Option<&'a SchemaField> {
    let (name, rest) = path.split_first()?;
    let field = fields.iter().find(|field| field.name == *name)?;

    if rest.is_empty() {
        Some(field)
    } else {
        find_nested_field(&field.fields, rest)
    }
}

fn fields_for_stack<'a>(schema: &'a SchemaDocument, stack: &[String]) -> Option<&'a [SchemaField]> {
    let (root, rest) = stack.split_first()?;
    let block = schema.blocks.iter().find(|block| block.root == *root)?;

    let mut fields = block.fields.as_slice();
    for segment in rest {
        let field = fields.iter().find(|field| field.name == *segment)?;
        fields = field.fields.as_slice();
    }

    Some(fields)
}

/// A valid RUNE identifier: one `Ident` token that spans the whole name.
///
/// Lexing the name instead of testing an ASCII character class is what makes
/// `näme` and other Unicode identifiers valid, keeps `_` and `-` valid after
/// an alphabetic first character, and rejects an empty name, a leading digit,
/// a leading `_`, an embedded space or quote, and a lexer keyword such as
/// `if`, `end`, `true`, or `null`, none of which lex to `Ident`.
fn is_identifier(value: &str) -> bool {
    let mut lexer = Lexer::new(value);

    let Ok(first) = lexer.next_token_spanned() else {
        return false;
    };
    if first.span.start != 0 || first.span.end != value.len() {
        return false;
    }
    let Token::Ident(name) = &first.token else {
        return false;
    };
    if name != value {
        return false;
    }

    matches!(lexer.next_token_spanned(), Ok(next) if next.token == Token::Eof)
}

/// Document symbols come straight from the indexed entries: objects are
/// containers, assignments are fields, a `gather` is a module of its own, and
/// conditional or metadata entries are layout and therefore never symbols.
///
/// A gathered file's own fields are never inlined: the importer shows the
/// statement that brings them in, and nothing else.
#[allow(deprecated)]
fn document_symbols(source: &SourceIndex) -> Vec<DocumentSymbol> {
    let mut positioned: Vec<(usize, DocumentSymbol)> = Vec::new();

    for gather in source.gathers() {
        let selection = source
            .lines()
            .range(gather.alias_span.unwrap_or(gather.path_content_span));

        positioned.push((
            gather.statement_span.start,
            DocumentSymbol {
                name: gather.alias.clone(),
                detail: Some(gather.raw_path.clone()),
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range: source.lines().range(gather.statement_span),
                selection_range: selection,
                children: None,
            },
        ));
    }

    for entry in source.entries() {
        let kind = match entry.kind {
            SourceEntryKind::Object => SymbolKind::OBJECT,
            SourceEntryKind::Assignment => SymbolKind::FIELD,
            _ => continue,
        };
        let key = source.key_range(entry);

        positioned.push((
            entry.key_span.start,
            DocumentSymbol {
                name: entry.path.join("."),
                detail: None,
                kind,
                tags: None,
                deprecated: None,
                range: Range::new(Position::new(key.start.line, 0), key.end),
                selection_range: key,
                children: None,
            },
        ));
    }

    // One order for the whole document, however it is spelled.
    positioned.sort_by_key(|(offset, _)| *offset);
    positioned.into_iter().map(|(_, symbol)| symbol).collect()
}

/// Range of the schema field-name token at `position`, when the cursor is on
/// it. Schema paths are selected from the parsed tree, but only after the
/// cursor is proven to touch the declaration's actual name token.
fn schema_field_name_range(schema_text: &str, position: Position) -> Option<Range> {
    let schema = SchemaDocument::from_str(schema_text).ok()?;
    let (_, span) = schema_field_at_position(schema_text, &schema, position)?;
    Some(LineIndex::new(schema_text).range(span))
}

/// Resolve the schema declaration under a cursor without treating every token
/// on the declaration's line as the field name. This is used by requests such
/// as `rename` and `references`, which clients may send without first calling
/// `prepareRename`.
fn schema_field_at_position(
    schema_text: &str,
    schema: &SchemaDocument,
    position: Position,
) -> Option<(Vec<String>, Span)> {
    let lines = LineIndex::new(schema_text);
    let offset = lines.position_to_byte(position)?;

    for block in &schema.blocks {
        if block.name_span.touches(offset) {
            return Some((vec![block.root.clone()], block.name_span));
        }

        let mut path = vec![block.root.clone()];
        if let Some(found) = schema_field_at_offset(&block.fields, &mut path, offset) {
            return Some(found);
        }
    }

    None
}

fn schema_field_at_offset(
    fields: &[SchemaField],
    path: &mut Vec<String>,
    offset: usize,
) -> Option<(Vec<String>, Span)> {
    for field in fields {
        path.push(field.name.clone());
        if field.name_span.touches(offset) {
            return Some((path.clone(), field.name_span));
        }
        if let Some(found) = schema_field_at_offset(&field.fields, path, offset) {
            return Some(found);
        }
        path.pop();
    }

    None
}

fn schema_definition_range(
    schema_text: &str,
    schema: &SchemaDocument,
    path: &[String],
) -> Option<Range> {
    let span = if path.len() == 1 {
        schema
            .blocks
            .iter()
            .find(|block| block.root == path[0])?
            .name_span
    } else {
        find_field_by_path(schema, path)?.name_span
    };

    Some(LineIndex::new(schema_text).range(span))
}

/// Paths of a rename target and of the sibling a rename would create.
///
/// The candidate is the same parent plus the new leaf, or `None` when the leaf
/// does not change, which cannot collide with anything.
fn sibling_candidate(path: &[String], new_name: &str) -> Option<Vec<String>> {
    let (leaf, parent) = path.split_last()?;
    if leaf == new_name {
        return None;
    }

    let mut candidate = parent.to_vec();
    candidate.push(new_name.to_string());
    Some(candidate)
}

/// The error a sibling collision is reported with: JSON-RPC invalid params,
/// with the current path, the new name, and the sibling that already owns it.
fn rename_collision(path: &[String], candidate: &[String]) -> tower_lsp::jsonrpc::Error {
    tower_lsp::jsonrpc::Error::invalid_params(format!(
        "Cannot rename '{}' to '{}': sibling '{}' already exists",
        path.join("."),
        candidate.last().cloned().unwrap_or_default(),
        candidate.join("."),
    ))
}

/// Every occurrence of `target_path` (key ranges) in one document.
///
/// Paths come from the structural index, so a key nested in a conditional is
/// matched at its enclosing object path and a same-named key in another object
/// is not matched at all.
fn references_in_document(source: &SourceIndex, target_path: &[String]) -> Vec<Range> {
    source
        .entries_with_path(target_path)
        .into_iter()
        .map(|entry| source.key_range(entry))
        .collect()
}

/// Resolve a schema field only when the cursor intersects its stored name span.
fn schema_path_at_position(
    schema_text: &str,
    schema: &SchemaDocument,
    position: Position,
) -> Option<Vec<String>> {
    schema_field_at_position(schema_text, schema, position).map(|(path, _)| path)
}

/// How one line takes part in the document's nesting.
enum LineRole<'a> {
    /// The line starts this statement.
    Entry(&'a SourceEntry),
    /// The line holds the `end` that closes an object, or a stray closer.
    Closer,
}

/// Re-indent a document to two spaces per nesting level.
///
/// Object and conditional blocks both open with a trailing `:` and close with
/// `end`/`endif`; `else`/`elseif` branches dedent for their own line and indent
/// what follows. Only leading indentation changes — trailing content (including
/// inline comments) and blank lines are preserved, as is the final-newline state.
///
/// Every line is classified through the index, so a line only affects the
/// nesting level when it really is a block keyword.
fn format_document(source: &SourceIndex) -> Option<String> {
    let text = source.text();
    let lines = source.lines();
    let mut depth: usize = 0;
    let mut out = String::new();

    // One role per line: the statement it starts, or the `end` closing an
    // object on it. The first statement on a line owns that line.
    let mut roles: HashMap<usize, LineRole> = HashMap::new();
    for entry in source.entries() {
        roles
            .entry(lines.line_of(entry.header_span.start))
            .or_insert(LineRole::Entry(entry));
    }
    for entry in source.entries() {
        if let Some(close) = entry.close_span {
            roles
                .entry(lines.line_of(close.start))
                .or_insert(LineRole::Closer);
        }
    }
    for closer in source.stray_closers() {
        roles
            .entry(lines.line_of(closer.span.start))
            .or_insert(LineRole::Closer);
    }

    for index in 0..text.lines().count() {
        let line = lines.line_text(index).unwrap_or("");
        if line.trim().is_empty() {
            out.push('\n');
            continue;
        }

        let entry = match roles.get(&index) {
            Some(LineRole::Entry(entry)) => Some(*entry),
            _ => None,
        };
        let closes_here = matches!(roles.get(&index), Some(LineRole::Closer));

        if entry.is_some_and(|entry| entry.kind.dedents()) || closes_here {
            depth = depth.saturating_sub(1);
        }

        out.push_str(&"  ".repeat(depth));
        out.push_str(line.trim_start());
        out.push('\n');

        let opens = entry.is_some_and(|entry| {
            entry.kind.opens_block() || entry.kind == SourceEntryKind::ConditionalBranch
        });
        if opens {
            depth += 1;
        }
    }

    if !text.ends_with('\n') {
        out.pop();
    }

    (out != text).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_diagnostics_find_missing_values_and_unclosed_blocks() {
        let diagnostics = recovery_diagnostics(&SourceIndex::new(
            r#"
app:
  name
  server:
    host "localhost"
"#,
        ));

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "Missing value for 'name'")
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("Unclosed object block 'app'"))
        );
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("Unclosed object block 'server'")
        }));
    }

    #[test]
    fn completion_filters_fields_already_used_in_current_object() {
        let schema = SchemaDocument::from_str(
            r#"
schema app:
  name string required
  debug bool
end
"#,
        )
        .unwrap();
        let completions = config_completion_items(
            Some(&schema),
            &SourceIndex::new("app:\n  name \"RuneApp\"\n  "),
            Position::new(2, 2),
            &[],
            None,
        );

        assert!(!completions.iter().any(|item| item.label == "name"));
        assert!(completions.iter().any(|item| item.label == "debug"));
    }

    #[test]
    fn enum_and_missing_required_messages_are_parsed_for_actions() {
        let (path, values) =
            enum_values_from_message("'app.environment' must be one of: dev, staging, production")
                .unwrap();
        assert_eq!(path, vec!["app", "environment"]);
        assert_eq!(values, vec!["dev", "staging", "production"]);

        let missing =
            missing_required_field_from_message("Missing required field 'version' inside 'app'")
                .unwrap();
        assert_eq!(missing, ("app".into(), "version".into()));
    }

    /// An array item message is about the field that supplies the array, so the
    /// item indexes are dropped from the path a declaration is looked up with.
    #[test]
    fn diagnostic_paths_drop_array_item_indexes() {
        assert_eq!(
            diagnostic_path("'app.plugins[1]' expected string, got int"),
            Some(vec!["app".to_string(), "plugins".to_string()])
        );
        assert_eq!(
            diagnostic_path("'app.grid[1][2]' expected string, got int"),
            Some(vec!["app".to_string(), "grid".to_string()])
        );
        assert_eq!(
            diagnostic_path("'app.plugins' expected array, got string"),
            Some(vec!["app".to_string(), "plugins".to_string()])
        );
    }

    #[test]
    fn schema_directive_completion_suggests_relative_paths() {
        let completions = config_completion_items(
            None,
            &SourceIndex::new("@schema \""),
            Position::new(0, 9),
            &[],
            None,
        );

        assert!(completions.iter().any(|item| item.label == "./schema.rune"));
        assert!(completions.iter().any(|item| item.label == "./schemas/"));
    }

    /// A started but unclosed value still offers enum members. The indexer
    /// stops at the unterminated string, so this cannot wait for a value token.
    #[test]
    fn enum_completion_is_offered_for_an_unclosed_value() {
        let schema = SchemaDocument::from_str(
            r#"
schema app:
  environment enum ["dev", "prod"]
end
"#,
        )
        .unwrap();
        let source = SourceIndex::new("app:\n  environment \"dev");
        let completions =
            config_completion_items(Some(&schema), &source, Position::new(1, 16), &[], None);

        assert!(
            completions
                .iter()
                .any(|item| item.label == "\"dev\"" || item.label.contains("dev")),
            "typing an unclosed enum value must still offer enum members: {:?}",
            completions
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>()
        );
    }

    /// A field completion after `endif` still knows it is inside the object the
    /// conditional was nested in.
    #[test]
    fn completion_after_endif_keeps_the_enclosing_object_scope() {
        let schema = SchemaDocument::from_str(
            r#"
schema app:
  name string required
  server:
    port int
  end
  debug bool
end
"#,
        )
        .unwrap();
        let source = SourceIndex::new("app:\n  if debug:\n    port 8080\n  endif\n  ");

        let completions =
            config_completion_items(Some(&schema), &source, Position::new(4, 2), &[], None);
        let labels: Vec<&str> = completions.iter().map(|item| item.label.as_str()).collect();

        // `port` was written inside the conditional, so it counts as used at
        // the enclosing object path... but only `server.port` belongs there;
        // the object's own fields are the ones that must be offered.
        assert!(labels.contains(&"name"), "expected app.name: {labels:?}");
        assert!(labels.contains(&"debug"), "expected app.debug: {labels:?}");
        assert!(
            labels.contains(&"server"),
            "expected app.server: {labels:?}"
        );

        // The conditional header never became a field of `app`.
        assert!(!labels.contains(&"debug:"), "{labels:?}");
    }

    #[test]
    fn type_fix_removes_quotes_for_numeric_values() {
        let text = "app:\n  port \"8080\"\nend\n";
        let (path, replacement) = type_fix_from_message(
            &SourceIndex::new(text),
            "'app.port' expected int, got string",
        )
        .unwrap();

        assert_eq!(path, vec!["app", "port"]);
        assert_eq!(replacement.new_text, "8080");
    }

    #[test]
    fn type_fix_handles_nested_numeric_values() {
        let text = "app:\n  server:\n    port \"8080\"\n  end\nend\n";
        let (path, replacement) = type_fix_from_message(
            &SourceIndex::new(text),
            "'app.server.port' expected int, got string",
        )
        .unwrap();

        assert_eq!(path, vec!["app", "server", "port"]);
        assert_eq!(replacement.new_text, "8080");
    }

    #[test]
    fn type_fix_handles_lsp_diagnostic_hint_text() {
        let text = "app:\n  name \"RuneApp\"\n  environment \"prod\"\n\n  server:\n    host \"localhost\"\n    port \"8080\"\n  end\n\n  plugins [\"auth\", 42]\nend\n";
        let (_, replacement) = type_fix_from_message(
            &SourceIndex::new(text),
            "'app.server.port' expected int, got string\nHint: Check around: port \"8080\"",
        )
        .unwrap();

        assert_eq!(replacement.title, "Remove quotes to make int");
        assert_eq!(replacement.new_text, "8080");
    }

    #[test]
    fn references_find_all_uses_of_a_scoped_path() {
        let text = "app:\n  server:\n    port 8080\n  end\n  port 9090\nend\n";
        let ranges = references_in_document(
            &SourceIndex::new(text),
            &["app".into(), "server".into(), "port".into()],
        );

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start.line, 2);
        // The unscoped `app.port` on line 4 must not be matched.
        assert!(ranges.iter().all(|range| range.start.line != 4));
    }

    #[test]
    fn references_locate_the_key_identifier_range() {
        let text = "app:\n  name \"RuneApp\"\nend\n";
        let ranges =
            references_in_document(&SourceIndex::new(text), &["app".into(), "name".into()]);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start, Position::new(1, 2));
        assert_eq!(ranges[0].end, Position::new(1, 6));
    }

    #[test]
    fn format_document_reindents_nested_blocks_and_conditionals() {
        let messy = "app:\nname \"RuneApp\"\nserver:\nport 8080\nif debug:\nlevel \"high\"\nelse:\nlevel \"low\"\nendif\nend\nend\n";
        let formatted =
            format_document(&SourceIndex::new(messy)).expect("formatting should change the text");

        let expected = "app:\n  name \"RuneApp\"\n  server:\n    port 8080\n    if debug:\n      level \"high\"\n    else:\n      level \"low\"\n    endif\n  end\nend\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn format_document_is_idempotent_and_preserves_comments_and_blanks() {
        let source =
            "app:\n  name \"RuneApp\" # the app name\n\n  server:\n    port 8080\n  end\nend\n";
        // Already well-formed: no change.
        assert_eq!(format_document(&SourceIndex::new(source)), None);

        let messy = "app:\nname \"RuneApp\" # the app name\n\nend\n";
        let once = format_document(&SourceIndex::new(messy)).unwrap();
        // Formatting the result again is a no-op.
        assert_eq!(format_document(&SourceIndex::new(&once)), None);
        assert!(once.contains("# the app name"));
        assert!(once.contains("\n\n"));
    }

    #[test]
    fn key_range_only_matches_when_cursor_is_on_the_key() {
        let source = SourceIndex::new("app:\n  name \"RuneApp\"\nend\n");
        // Cursor on the `name` key.
        assert_eq!(
            source
                .field_key_at(Position::new(1, 3))
                .map(|entry| source.key_range(entry)),
            Some(Range::new(Position::new(1, 2), Position::new(1, 6)))
        );
        // Cursor on the value: nothing renameable.
        assert!(source.field_key_at(Position::new(1, 9)).is_none());
    }

    /// The schema rename range comes from the field-name token, never from the
    /// line the cursor happens to be on.
    #[test]
    fn schema_rename_range_requires_the_field_name_token() {
        let text = "schema app:\n  name string\nend\n";

        // Inside the `string` type: no rename.
        assert_eq!(schema_field_name_range(text, Position::new(1, 9)), None);
        // On `name`: the exact identifier range.
        assert_eq!(
            schema_field_name_range(text, Position::new(1, 3)),
            Some(Range::new(Position::new(1, 2), Position::new(1, 6)))
        );
        // On the `schema` keyword rather than the block name: no rename.
        assert_eq!(schema_field_name_range(text, Position::new(0, 2)), None);
        // On the block name itself: the block's declaration range.
        assert_eq!(
            schema_field_name_range(text, Position::new(0, 8)),
            Some(Range::new(Position::new(0, 7), Position::new(0, 10)))
        );
    }

    /// Quick-fix value ranges come from real value tokens, so a literal holding
    /// a `#` keeps its whole span.
    #[test]
    fn value_range_covers_literals_containing_a_comment_marker() {
        let source = SourceIndex::new("app:\n  name \"a#b\"\nend\n");
        let range = value_range_for_path(Some(&source), &["app".into(), "name".into()]).unwrap();

        assert_eq!(
            range,
            Range::new(Position::new(1, 7), Position::new(1, 12)),
            "the value span must not stop at the `#` inside the string"
        );
        assert_eq!(source.text_in_range(range), Some("\"a#b\""));
    }

    /// Document symbols follow the object stack through conditionals.
    #[test]
    fn document_symbols_skip_conditionals_and_keep_scope() {
        let source = SourceIndex::new(
            "app:\n  if debug:\n    feature true\n  endif\n  name \"Rune\"\nend\n",
        );
        let names: Vec<String> = document_symbols(&source)
            .into_iter()
            .map(|symbol| symbol.name)
            .collect();

        assert_eq!(names, vec!["app", "app.feature", "app.name"]);
    }

    #[test]
    fn schema_path_requires_the_declaration_name_token() {
        let text = "schema app:\n  name string required\n  server:\n    port int\n  end\nend\n";
        let schema = SchemaDocument::from_str(text).unwrap();

        // Cursor on `schema app:` (0-based line 0) -> the block path.
        assert_eq!(
            schema_path_at_position(text, &schema, Position::new(0, 9)),
            Some(vec!["app".into()])
        );
        // Cursor on `port int` (0-based line 3) -> the nested field path.
        assert_eq!(
            schema_path_at_position(text, &schema, Position::new(3, 4)),
            Some(vec!["app".into(), "server".into(), "port".into()])
        );
        // A blank/unrelated line resolves to nothing.
        assert_eq!(
            schema_path_at_position(text, &schema, Position::new(5, 0)),
            None
        );
    }

    /// A new name is validated by lexing it, so a Unicode identifier is valid
    /// and a keyword - which never lexes to `Ident` - is not.
    #[test]
    fn identifiers_are_decided_by_the_lexer() {
        for valid in ["name", "näme", "schema", "a-b", "name_1", "x"] {
            assert!(is_identifier(valid), "{valid:?} must be a valid identifier");
        }

        for invalid in [
            "", "1name", "_name", "-name", "na me", "\"name\"", "if", "end", "true", "null",
        ] {
            assert!(
                !is_identifier(invalid),
                "{invalid:?} must not be a valid identifier"
            );
        }
    }

    /// A rename collides with a sibling of the same parent only, never with a
    /// leaf under another parent and never with the leaf it already has.
    #[test]
    fn sibling_candidates_share_the_parent_and_skip_the_current_leaf() {
        let path = vec!["server".to_string(), "port".to_string()];

        assert_eq!(
            sibling_candidate(&path, "host"),
            Some(vec!["server".to_string(), "host".to_string()])
        );
        assert_eq!(sibling_candidate(&path, "port"), None);

        let candidate = sibling_candidate(&path, "host").expect("a sibling candidate");
        assert_eq!(
            rename_collision(&path, &candidate).message.as_ref(),
            "Cannot rename 'server.port' to 'host': sibling 'server.host' already exists"
        );
    }
}
