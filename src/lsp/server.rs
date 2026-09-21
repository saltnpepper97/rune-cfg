// Author: Dustin Pilgrim
// License: MIT

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, CompletionItem, CompletionItemKind,
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, InitializedParams, InsertTextFormat, Location,
    MarkedString, MessageType, OneOf, Position, PrepareRenameResponse, Range, ReferenceParams,
    RenameOptions, RenameParams, ServerCapabilities, SymbolKind, TextDocumentPositionParams,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url, WorkDoneProgressOptions,
    WorkspaceEdit,
};
use tower_lsp::{Client, LanguageServer};

use crate::diagnostic::{DiagnosticSeverity, RuneDiagnostic};
use crate::source::{
    LineIndex, SourceEntry, SourceEntryKind, SourceIndex, Span, starts_with_schema_block,
};
use crate::{RuneConfig, RuneError, SchemaDocument, SchemaField, SchemaType};

/// An open buffer together with the structural index built from its latest
/// text, so each request re-uses one tokenization instead of re-deriving the
/// document layout from raw lines.
#[derive(Debug, Clone)]
struct OpenDocument {
    version: i32,
    source: Arc<SourceIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SchemaDirective {
    reference: String,
    line: usize,
    column: usize,
}

/// A single key occurrence for cross-file references/rename. The schema field
/// declaration is flagged so `references` can honor `include_declaration`.
struct Occurrence {
    uri: Url,
    range: Range,
    is_declaration: bool,
}

pub struct RuneLanguageServer {
    client: Client,
    root_uri: RwLock<Option<Url>>,
    documents: RwLock<HashMap<Url, OpenDocument>>,
}

impl RuneLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            root_uri: RwLock::new(None),
            documents: RwLock::new(HashMap::new()),
        }
    }

    async fn validate_all_open_documents(&self) {
        let documents = self.documents.read().await.clone();

        for (uri, document) in documents {
            if !is_rune_file(&uri) {
                continue;
            }

            let diagnostics = self.diagnostics_for_document(&uri, &document.source).await;
            self.client
                .publish_diagnostics(uri, diagnostics, Some(document.version))
                .await;
        }
    }

    async fn diagnostics_for_document(
        &self,
        uri: &Url,
        source: &SourceIndex,
    ) -> Vec<tower_lsp::lsp_types::Diagnostic> {
        let text = source.text();
        let rune_diagnostics = if is_schema_document(uri, text) {
            match SchemaDocument::from_str(text) {
                Ok(_) => Vec::new(),
                Err(error) => vec![diagnostic_from_error(error, source.lines())],
            }
        } else {
            self.config_diagnostics(uri, source).await
        };

        rune_diagnostics
            .into_iter()
            .map(lsp_diagnostic_from_rune)
            .collect()
    }

    async fn config_diagnostics(&self, uri: &Url, source: &SourceIndex) -> Vec<RuneDiagnostic> {
        let text = source.text();
        let config = match RuneConfig::from_str(text) {
            Ok(config) => config,
            Err(error) => {
                let mut diagnostics = vec![diagnostic_from_error(error, source.lines())];
                diagnostics.extend(recovery_diagnostics(source));
                dedupe_diagnostics(&mut diagnostics);
                return diagnostics;
            }
        };

        let schema_text = match self.schema_text_for_document(uri, source).await {
            Ok(Some(schema_text)) => schema_text,
            Ok(None) => return Vec::new(),
            Err(diagnostic) => return vec![diagnostic],
        };

        let schema = match SchemaDocument::from_str(&schema_text) {
            Ok(schema) => schema,
            Err(error) => return vec![diagnostic_from_error(error, source.lines())],
        };

        config.validate_schema(&schema)
    }

    async fn schema_text_for(&self, uri: &Url) -> Option<String> {
        let source = self.document_source_for(uri).await?;
        self.schema_text_for_document(uri, &source)
            .await
            .ok()
            .flatten()
    }

    async fn schema_text_for_document(
        &self,
        uri: &Url,
        source: &SourceIndex,
    ) -> Result<Option<String>, RuneDiagnostic> {
        if let Some(directive) = schema_directive(source) {
            let schema_uri = self.resolve_schema_directive_uri(uri, &directive).await?;
            return self
                .schema_text_for_uri(&schema_uri)
                .await
                .map(Some)
                .ok_or_else(|| schema_reference_diagnostic(&directive, &[]));
        }

        let Some(schema_uri) = self.schema_uri_for(uri).await else {
            return Ok(None);
        };

        Ok(self.schema_text_for_uri(&schema_uri).await)
    }

    async fn schema_text_for_uri(&self, schema_uri: &Url) -> Option<String> {
        if let Some(document) = self.documents.read().await.get(&schema_uri) {
            return Some(document.source.text().to_string());
        }

        let path = schema_uri.to_file_path().ok()?;
        std::fs::read_to_string(path).ok()
    }

    async fn resolve_schema_directive_uri(
        &self,
        uri: &Url,
        directive: &SchemaDirective,
    ) -> Result<Url, RuneDiagnostic> {
        let path = uri
            .to_file_path()
            .map_err(|_| schema_reference_diagnostic(directive, &[]))?;
        let config_dir = path
            .parent()
            .ok_or_else(|| schema_reference_diagnostic(directive, &[]))?;

        let candidates = schema_candidates(&directive.reference, config_dir);
        for candidate in &candidates {
            if candidate.exists() || self.is_open_uri_for_path(&candidate).await {
                if let Ok(uri) = Url::from_file_path(candidate.clone()) {
                    return Ok(uri);
                }
            }
        }

        Err(schema_reference_diagnostic(directive, &candidates))
    }

    async fn schema_for(&self, uri: &Url) -> Option<SchemaDocument> {
        let schema_text = self.schema_text_for(uri).await?;
        SchemaDocument::from_str(&schema_text).ok()
    }

    /// Resolve the schema file URI backing a config document, following an
    /// explicit `@schema` directive when present and otherwise discovering
    /// `schema.rune` upward from the config directory.
    async fn schema_uri_for_document(&self, uri: &Url, source: &SourceIndex) -> Option<Url> {
        if let Some(directive) = schema_directive(source) {
            return self
                .resolve_schema_directive_uri(uri, &directive)
                .await
                .ok();
        }

        self.schema_uri_for(uri).await
    }

    /// Resolve the rename/references target at a position: the field path plus
    /// the schema it belongs to. The schema URI is `None` when the document has
    /// no schema, signalling single-file behavior.
    async fn rename_target(
        &self,
        uri: &Url,
        source: &SourceIndex,
        position: Position,
    ) -> Option<(Vec<String>, Option<Url>)> {
        if is_schema_document(uri, source.text()) {
            let schema = SchemaDocument::from_str(source.text()).ok()?;
            let path = schema_path_at_position(&schema, position)?;
            return Some((path, Some(uri.clone())));
        }

        let path = source.field_on_line(position.line)?.path.clone();
        let schema_uri = self.schema_uri_for_document(uri, source).await;
        Some((path, schema_uri))
    }

    /// Config files bound to `schema_uri` (via `@schema` or discovery), drawn
    /// from open documents and the workspace tree. Excludes the schema itself.
    async fn related_config_uris(&self, schema_uri: &Url) -> Vec<Url> {
        let mut candidates: Vec<Url> = self.documents.read().await.keys().cloned().collect();

        if let Some(root) = self.root_path().await {
            let mut paths = Vec::new();
            collect_rune_files(&root, &mut paths);
            for path in paths {
                if let Ok(uri) = Url::from_file_path(path) {
                    candidates.push(uri);
                }
            }
        }

        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for uri in candidates {
            if !seen.insert(uri.clone()) {
                continue;
            }
            if uri == *schema_uri || is_schema_file(&uri) || !is_rune_file(&uri) {
                continue;
            }
            let Some(source) = self.document_source_for(&uri).await else {
                continue;
            };
            if looks_like_schema_text(source.text()) {
                continue;
            }
            if self.schema_uri_for_document(&uri, &source).await.as_ref() == Some(schema_uri) {
                result.push(uri);
            }
        }

        result
    }

    /// Every occurrence of `path` to rename together: the schema field
    /// declaration (marked) plus its key usages in each bound config file.
    async fn cross_file_occurrences(&self, schema_uri: &Url, path: &[String]) -> Vec<Occurrence> {
        let mut occurrences = Vec::new();

        if let Some(schema_text) = self.document_text_for(schema_uri).await
            && let Ok(schema) = SchemaDocument::from_str(&schema_text)
            && let Some(line) = definition_line_for_path(&schema, path)
        {
            let name = path.last().map(String::as_str).unwrap_or_default();
            occurrences.push(Occurrence {
                uri: schema_uri.clone(),
                range: identifier_range(&LineIndex::new(&schema_text), line, name),
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

    async fn document_source_for(&self, uri: &Url) -> Option<Arc<SourceIndex>> {
        if let Some(document) = self.documents.read().await.get(uri) {
            return Some(Arc::clone(&document.source));
        }

        let path = uri.to_file_path().ok()?;
        let text = std::fs::read_to_string(path).ok()?;
        Some(Arc::new(SourceIndex::new(&text)))
    }

    /// Text of a document, whether or not it is open in the editor.
    async fn document_text_for(&self, uri: &Url) -> Option<String> {
        self.document_source_for(uri)
            .await
            .map(|source| source.text().to_string())
    }

    async fn schema_uri_for(&self, uri: &Url) -> Option<Url> {
        let path = uri.to_file_path().ok()?;
        let mut directory = path.parent()?.to_path_buf();
        let root_path = self.root_path().await;

        loop {
            let candidate = directory.join("schema.rune");
            if candidate.exists() || self.is_open_uri_for_path(&candidate).await {
                if let Ok(uri) = Url::from_file_path(candidate) {
                    return Some(uri);
                }
            }

            if root_path.as_ref().is_some_and(|root| directory == *root) {
                break;
            }

            if !directory.pop() {
                break;
            }
        }

        None
    }

    async fn root_path(&self) -> Option<PathBuf> {
        self.root_uri
            .read()
            .await
            .as_ref()
            .and_then(|uri| uri.to_file_path().ok())
    }

    async fn is_open_uri_for_path(&self, path: &Path) -> bool {
        let Ok(uri) = Url::from_file_path(path) else {
            return false;
        };
        self.documents.read().await.contains_key(&uri)
    }

    async fn schema_source_label_for_document(
        &self,
        uri: &Url,
        source: &SourceIndex,
    ) -> Option<String> {
        if let Some(directive) = schema_directive(source) {
            return Some(format!("@schema \"{}\"", directive.reference));
        }

        let schema_uri = self.schema_uri_for(uri).await?;
        schema_uri
            .to_file_path()
            .ok()
            .map(|path| path.display().to_string())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for RuneLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        *self.root_uri.write().await = params.root_uri;

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

        let items = if is_schema_document(&uri, source.text()) {
            schema_completion_items()
        } else {
            let schema = self.schema_for(&uri).await;
            let config_dir = uri
                .to_file_path()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf));
            config_completion_items(schema.as_ref(), &source, position, config_dir.as_deref())
        };

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let Some(source) = self.document_source_for(&uri).await else {
            return Ok(None);
        };
        if is_schema_document(&uri, source.text()) {
            return Ok(None);
        }

        let position = params.text_document_position_params.position;
        let Some(schema) = self.schema_for(&uri).await else {
            return Ok(None);
        };
        let Some(path) = source
            .field_on_line(position.line)
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

        for diagnostic in params.context.diagnostics {
            if diagnostic.message.contains("Unclosed object block") {
                actions.push(text_edit_action(
                    uri.clone(),
                    "Insert missing end",
                    TextEdit {
                        range: Range {
                            start: diagnostic.range.start,
                            end: diagnostic.range.start,
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
        let text = source.text();
        if is_schema_document(&uri, text) {
            return Ok(None);
        }

        let position = params.text_document_position_params.position;

        // A `@schema "..."` directive jumps to the top of the schema file.
        if schema_directive(&source)
            .is_some_and(|directive| directive.line == position.line as usize + 1)
        {
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

        // Otherwise jump from a config key to its schema field/block definition.
        let Some(path) = source
            .field_on_line(position.line)
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
        let Some(line) = definition_line_for_path(&schema, &path) else {
            return Ok(None);
        };

        let name = path.last().map(String::as_str).unwrap_or_default();
        let range = identifier_range(&LineIndex::new(&schema_text), line, name);
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
        let Some((path, schema_uri)) = self.rename_target(&uri, &source, position).await else {
            return Ok(None);
        };

        // No schema: fall back to single-file references within this document.
        let Some(schema_uri) = schema_uri else {
            let locations = references_in_document(&source, &path)
                .into_iter()
                .filter(|range| include_declaration || range.start.line != position.line)
                .map(|range| Location {
                    uri: uri.clone(),
                    range,
                })
                .collect();
            return Ok(Some(locations));
        };

        // Schema-scoped: declaration in the schema + usages across bound configs.
        let locations = self
            .cross_file_occurrences(&schema_uri, &path)
            .await
            .into_iter()
            .filter(|occurrence| include_declaration || !occurrence.is_declaration)
            .map(|occurrence| Location {
                uri: occurrence.uri,
                range: occurrence.range,
            })
            .collect();

        Ok(Some(locations))
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
        if is_schema_document(&uri, source.text()) {
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

        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

        match schema_uri {
            // No schema: single-file rename within this document.
            None => {
                let edits: Vec<TextEdit> = references_in_document(&source, &path)
                    .into_iter()
                    .map(|range| TextEdit {
                        range,
                        new_text: new_name.clone(),
                    })
                    .collect();
                if edits.is_empty() {
                    return Ok(None);
                }
                changes.insert(uri, edits);
            }
            // Schema-scoped: update the schema declaration and every bound config.
            Some(schema_uri) => {
                for occurrence in self.cross_file_occurrences(&schema_uri, &path).await {
                    changes.entry(occurrence.uri).or_default().push(TextEdit {
                        range: occurrence.range,
                        new_text: new_name.clone(),
                    });
                }
                if changes.is_empty() {
                    return Ok(None);
                }
            }
        }

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
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
        self.documents.write().await.insert(
            document.uri,
            OpenDocument {
                version: document.version,
                source: Arc::new(SourceIndex::new(&document.text)),
            },
        );
        self.validate_all_open_documents().await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            self.documents.write().await.insert(
                params.text_document.uri,
                OpenDocument {
                    version: params.text_document.version,
                    source: Arc::new(SourceIndex::new(&change.text)),
                },
            );
        }

        self.validate_all_open_documents().await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.write().await.remove(&uri);
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
        self.validate_all_open_documents().await;
    }
}

fn is_rune_file(uri: &Url) -> bool {
    uri.to_file_path()
        .ok()
        .and_then(|path| path.extension().map(|extension| extension == "rune"))
        .unwrap_or(false)
}

fn is_schema_file(uri: &Url) -> bool {
    uri.to_file_path()
        .ok()
        .and_then(|path| path.file_name().map(|name| name == "schema.rune"))
        .unwrap_or(false)
}

fn is_schema_document(uri: &Url, text: &str) -> bool {
    is_schema_file(uri) || looks_like_schema_text(text)
}

/// True for `schema <name>:` documents, decided from real tokens so a
/// commented-out or quoted `schema` never counts.
fn looks_like_schema_text(text: &str) -> bool {
    starts_with_schema_block(text)
}

/// The first `@schema "reference"` directive: the reference plus the 1-based
/// line and `char` column of its opening quote.
///
/// The directive is found through the indexed entries, so a `#` inside the
/// quoted reference never ends the directive early.
fn schema_directive(source: &SourceIndex) -> Option<SchemaDirective> {
    let entry = source.entries().iter().find(|entry| {
        entry.kind == SourceEntryKind::Metadata && entry.name.as_deref() == Some("schema")
    })?;

    let value_span = entry.value_span?;
    let raw = source
        .text()
        .get(value_span.start..value_span.end)?
        .trim_start();
    let reference = parse_quoted_string(raw)?;

    let lines = source.lines();
    let line = lines.line_of(value_span.start);
    let line_start = lines.line_span(line)?.start;
    let column = source
        .text()
        .get(line_start..value_span.start)?
        .chars()
        .count()
        + 1;

    Some(SchemaDirective {
        reference,
        line: line + 1,
        column,
    })
}

fn parse_quoted_string(input: &str) -> Option<String> {
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

fn schema_reference_completion_items(config_dir: Option<&Path>) -> Vec<CompletionItem> {
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

    let mut items = Vec::new();
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
            if items.iter().any(|item: &CompletionItem| item.label == stem) {
                continue;
            }

            items.push(CompletionItem {
                label: stem.into(),
                kind: Some(CompletionItemKind::FILE),
                detail: Some(format!("schema: {}", path.display())),
                insert_text: Some(stem.into()),
                ..CompletionItem::default()
            });
        }
    }

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

fn is_schema_directive_context(before_cursor: &str) -> bool {
    let trimmed = before_cursor.trim_start();
    trimmed.starts_with("@schema") && trimmed.contains('"')
}

fn is_schema_path_reference(reference: &str) -> bool {
    reference.starts_with('.')
        || reference.starts_with('/')
        || reference.starts_with('~')
        || reference.contains('/')
        || reference.contains('\\')
        || reference.ends_with(".rune")
}

fn expand_schema_path(reference: &str, config_dir: &Path) -> PathBuf {
    if let Some(rest) = reference.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }

    let path = PathBuf::from(reference);
    if path.is_absolute() {
        path
    } else {
        config_dir.join(path)
    }
}

fn schema_reference_diagnostic(
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

    RuneDiagnostic::error(format!("Schema '{}' was not found", directive.reference))
        .with_range(
            directive.line,
            directive.column,
            directive.column + directive.reference.len() + 2,
        )
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
    config_dir: Option<&Path>,
) -> Vec<CompletionItem> {
    let before_cursor = source.lines().text_before(position);
    if is_schema_directive_context(before_cursor) {
        return schema_reference_completion_items(config_dir);
    }

    if before_cursor.contains('$') {
        return dollar_reference_completion_items();
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

    let used = source.used_keys_in_scope(&scope, position.line);
    let mut items = if let Some(schema) = schema {
        schema_field_completion_items(schema, &scope, &used)
    } else {
        Vec::new()
    };

    items.extend(
        ["end", "if", "else", "endif", "gather"]
            .into_iter()
            .map(|label| keyword_completion(label, CompletionItemKind::KEYWORD)),
    );

    items
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
            &[field.name.clone()],
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

/// A bare RUNE identifier, which is what a rename may introduce.
fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

/// Document symbols come straight from the indexed entries: objects are
/// containers, assignments are fields, and conditional or metadata entries are
/// layout and therefore never symbols.
fn document_symbols(source: &SourceIndex) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    for entry in source.entries() {
        let kind = match entry.kind {
            SourceEntryKind::Object => SymbolKind::OBJECT,
            SourceEntryKind::Assignment => SymbolKind::FIELD,
            _ => continue,
        };
        let key = source.key_range(entry);

        #[allow(deprecated)]
        symbols.push(DocumentSymbol {
            name: entry.path.join("."),
            detail: None,
            kind,
            tags: None,
            deprecated: None,
            range: Range::new(Position::new(key.start.line, 0), key.end),
            selection_range: key,
            children: None,
        });
    }

    symbols
}

/// Range of the first token on a line that reads as `name`.
///
/// The token span the lexer produced wins over a raw byte search, and both are
/// reported through [`LineIndex`], so positions are UTF-16 code units.
fn identifier_range(lines: &LineIndex, line: u32, name: &str) -> Range {
    if let Some(span) = lines.identifier_span_on_line(line as usize, name) {
        return lines.range(span);
    }

    // Fall back to a byte search when the line's tokens cannot be lexed, for
    // instance because a lexeme is still unterminated.
    let start = lines
        .line_span(line as usize)
        .and_then(|span| {
            let text = lines.line_text(line as usize)?;
            let byte = text.find(name)?;
            Some(lines.byte_to_position(span.start + byte))
        })
        .unwrap_or(Position::new(line, 0));
    let width = name.encode_utf16().count() as u32;

    Range::new(start, Position::new(line, start.character + width))
}

/// Range of the schema field-name token at `position`, when the cursor is on
/// it.
///
/// A schema tree resolves a field from its 1-based line, and a line also holds
/// the field's type and options; a rename may only be offered when the cursor
/// intersects the name token itself, which is the text it would replace.
fn schema_field_name_range(schema_text: &str, position: Position) -> Option<Range> {
    let schema = SchemaDocument::from_str(schema_text).ok()?;
    let path = schema_path_at_position(&schema, position)?;
    let name = path.last()?;
    let lines = LineIndex::new(schema_text);
    let span = lines.identifier_span_on_line(position.line as usize, name)?;
    let offset = lines.position_to_byte(position)?;

    span.touches(offset).then(|| lines.range(span))
}

/// Resolve a config path to the 0-based line of its schema definition.
///
/// A single-segment path refers to a top-level block; deeper paths refer to a
/// nested field. Schema lines are stored 1-based, so they are converted here.
fn definition_line_for_path(schema: &SchemaDocument, path: &[String]) -> Option<u32> {
    let line = if path.len() == 1 {
        schema
            .blocks
            .iter()
            .find(|block| block.root == path[0])?
            .line
    } else {
        find_field_by_path(schema, path)?.line
    };

    Some(line.saturating_sub(1) as u32)
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

/// Recursively collect `*.rune` files under `dir`, skipping `target/`, `.git/`,
/// and hidden directories. Used to find configs bound to a schema.
fn collect_rune_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if path.is_dir() {
            if name.starts_with('.') || name == "target" {
                continue;
            }
            collect_rune_files(&path, out);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rune") {
            out.push(path);
        }
    }
}

/// The field path declared at `position` inside a schema file, walking the
/// parsed schema tree by 1-based source line.
fn schema_path_at_position(schema: &SchemaDocument, position: Position) -> Option<Vec<String>> {
    let target_line = position.line as usize + 1;

    for block in &schema.blocks {
        if block.line == target_line {
            return Some(vec![block.root.clone()]);
        }
        if let Some(path) = find_schema_path_by_line(
            &block.fields,
            std::slice::from_ref(&block.root),
            target_line,
        ) {
            return Some(path);
        }
    }

    None
}

fn find_schema_path_by_line(
    fields: &[SchemaField],
    prefix: &[String],
    target_line: usize,
) -> Option<Vec<String>> {
    for field in fields {
        let mut path = prefix.to_vec();
        path.push(field.name.clone());

        if field.line == target_line {
            return Some(path);
        }
        if let Some(found) = find_schema_path_by_line(&field.fields, &path, target_line) {
            return Some(found);
        }
    }

    None
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
    fn parses_schema_directive() {
        let directive = schema_directive(&SourceIndex::new(
            r#"
app:
  name "RuneApp"
end
@schema "stasis"
"#,
        ))
        .unwrap();

        assert_eq!(directive.reference, "stasis");
        assert_eq!(directive.line, 5);
        assert_eq!(directive.column, 9);
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
        let directive =
            schema_directive(&SourceIndex::new("@schema \"./schemas/app#1.rune\"")).unwrap();

        assert_eq!(directive.reference, "./schemas/app#1.rune");
        assert_eq!(directive.column, 9);
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
        let text = "# App schema\nschema app:\n  name string required\nend\n";

        assert!(is_schema_document(&uri, text));
    }

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

    #[test]
    fn schema_directive_completion_suggests_relative_paths() {
        let completions = config_completion_items(
            None,
            &SourceIndex::new("@schema \""),
            Position::new(0, 9),
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
            config_completion_items(Some(&schema), &source, Position::new(1, 16), None);

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
            config_completion_items(Some(&schema), &source, Position::new(4, 2), None);
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
    fn definition_line_resolves_blocks_and_fields() {
        let schema = SchemaDocument::from_str(
            "schema app:\n  name string required\n  server:\n    port int\n  end\nend\n",
        )
        .unwrap();

        // Top-level block (`schema app:` is line 1 -> 0-based 0).
        assert_eq!(definition_line_for_path(&schema, &["app".into()]), Some(0));
        // Nested field (`port int` is line 4 -> 0-based 3).
        assert_eq!(
            definition_line_for_path(&schema, &["app".into(), "server".into(), "port".into()]),
            Some(3)
        );
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
    fn schema_path_resolves_block_and_nested_field_by_line() {
        let schema = SchemaDocument::from_str(
            "schema app:\n  name string required\n  server:\n    port int\n  end\nend\n",
        )
        .unwrap();

        // Cursor on `schema app:` (0-based line 0) -> the block path.
        assert_eq!(
            schema_path_at_position(&schema, Position::new(0, 9)),
            Some(vec!["app".into()])
        );
        // Cursor on `port int` (0-based line 3) -> the nested field path.
        assert_eq!(
            schema_path_at_position(&schema, Position::new(3, 4)),
            Some(vec!["app".into(), "server".into(), "port".into()])
        );
        // A blank/unrelated line resolves to nothing.
        assert_eq!(schema_path_at_position(&schema, Position::new(5, 0)), None);
    }

    #[test]
    fn collect_rune_files_walks_tree_and_skips_target_and_hidden() {
        let base = std::env::temp_dir().join(format!("rune-walk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("nested")).unwrap();
        std::fs::create_dir_all(base.join("target")).unwrap();
        std::fs::create_dir_all(base.join(".hidden")).unwrap();
        std::fs::write(base.join("a.rune"), "app:\nend\n").unwrap();
        std::fs::write(base.join("nested/b.rune"), "app:\nend\n").unwrap();
        std::fs::write(base.join("notes.txt"), "ignore me").unwrap();
        std::fs::write(base.join("target/skip.rune"), "app:\nend\n").unwrap();
        std::fs::write(base.join(".hidden/skip.rune"), "app:\nend\n").unwrap();

        let mut found = Vec::new();
        collect_rune_files(&base, &mut found);
        let names: Vec<String> = found
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
            .collect();

        assert!(names.contains(&"a.rune".to_string()));
        assert!(names.contains(&"b.rune".to_string()));
        assert!(!names.iter().any(|n| n == "notes.txt"));
        assert!(!names.iter().any(|n| n == "skip.rune"));

        let _ = std::fs::remove_dir_all(&base);
    }
}
