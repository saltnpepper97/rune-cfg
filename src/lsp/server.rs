// Author: Dustin Pilgrim
// License: MIT

use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
use crate::{RuneConfig, RuneError, SchemaDocument, SchemaField, SchemaType};

#[derive(Debug, Clone)]
struct OpenDocument {
    text: String,
    version: i32,
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

            let diagnostics = self.diagnostics_for_document(&uri, &document.text).await;
            self.client
                .publish_diagnostics(uri, diagnostics, Some(document.version))
                .await;
        }
    }

    async fn diagnostics_for_document(
        &self,
        uri: &Url,
        text: &str,
    ) -> Vec<tower_lsp::lsp_types::Diagnostic> {
        let rune_diagnostics = if is_schema_document(uri, text) {
            match SchemaDocument::from_str(text) {
                Ok(_) => Vec::new(),
                Err(error) => vec![diagnostic_from_error(error)],
            }
        } else {
            self.config_diagnostics(uri, text).await
        };

        rune_diagnostics
            .into_iter()
            .map(lsp_diagnostic_from_rune)
            .collect()
    }

    async fn config_diagnostics(&self, uri: &Url, text: &str) -> Vec<RuneDiagnostic> {
        let config = match RuneConfig::from_str(text) {
            Ok(config) => config,
            Err(error) => {
                let mut diagnostics = vec![diagnostic_from_error(error)];
                diagnostics.extend(recovery_diagnostics(text));
                dedupe_diagnostics(&mut diagnostics);
                return diagnostics;
            }
        };

        let schema_text = match self.schema_text_for_document(uri, text).await {
            Ok(Some(schema_text)) => schema_text,
            Ok(None) => return Vec::new(),
            Err(diagnostic) => return vec![diagnostic],
        };

        let schema = match SchemaDocument::from_str(&schema_text) {
            Ok(schema) => schema,
            Err(error) => return vec![diagnostic_from_error(error)],
        };

        config.validate_schema(&schema)
    }

    async fn schema_text_for(&self, uri: &Url) -> Option<String> {
        let text = self.document_text_for(uri).await?;
        self.schema_text_for_document(uri, &text)
            .await
            .ok()
            .flatten()
    }

    async fn schema_text_for_document(
        &self,
        uri: &Url,
        text: &str,
    ) -> Result<Option<String>, RuneDiagnostic> {
        if let Some(directive) = schema_directive(text) {
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
            return Some(document.text.clone());
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
    async fn schema_uri_for_document(&self, uri: &Url, text: &str) -> Option<Url> {
        if let Some(directive) = schema_directive(text) {
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
        text: &str,
        position: Position,
    ) -> Option<(Vec<String>, Option<Url>)> {
        if is_schema_document(uri, text) {
            let schema = SchemaDocument::from_str(text).ok()?;
            let path = schema_path_at_position(&schema, position)?;
            return Some((path, Some(uri.clone())));
        }

        let path = path_at_position(text, position)?;
        let schema_uri = self.schema_uri_for_document(uri, text).await;
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
            let Some(text) = self.document_text_for(&uri).await else {
                continue;
            };
            if looks_like_schema_text(&text) {
                continue;
            }
            if self.schema_uri_for_document(&uri, &text).await.as_ref() == Some(schema_uri) {
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
                range: identifier_range(&schema_text, line, name),
                is_declaration: true,
            });
        }

        for config_uri in self.related_config_uris(schema_uri).await {
            if let Some(text) = self.document_text_for(&config_uri).await {
                for range in references_in_document(&text, path) {
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

    async fn document_text_for(&self, uri: &Url) -> Option<String> {
        if let Some(document) = self.documents.read().await.get(uri) {
            return Some(document.text.clone());
        }

        let path = uri.to_file_path().ok()?;
        std::fs::read_to_string(path).ok()
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

    async fn schema_source_label_for_document(&self, uri: &Url, text: &str) -> Option<String> {
        if let Some(directive) = schema_directive(text) {
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
                name: "rune-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "rune-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let Some(text) = self.document_text_for(&uri).await else {
            return Ok(None);
        };

        let items = if is_schema_document(&uri, &text) {
            schema_completion_items()
        } else {
            let schema = self.schema_for(&uri).await;
            let config_dir = uri
                .to_file_path()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf));
            config_completion_items(schema.as_ref(), &text, position, config_dir.as_deref())
        };

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let Some(text) = self.document_text_for(&uri).await else {
            return Ok(None);
        };
        if is_schema_document(&uri, &text) {
            return Ok(None);
        }

        let position = params.text_document_position_params.position;
        let Some(schema) = self.schema_for(&uri).await else {
            return Ok(None);
        };
        let Some(path) = path_at_position(&text, position) else {
            return Ok(None);
        };
        let Some(field) = find_field_by_path(&schema, &path) else {
            return Ok(None);
        };
        let schema_source = self.schema_source_label_for_document(&uri, &text).await;

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
        let Some(text) = self.document_text_for(&uri).await else {
            return Ok(None);
        };

        Ok(Some(DocumentSymbolResponse::Nested(document_symbols(
            &text,
        ))))
    }

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let mut actions = Vec::new();
        let text = self.document_text_for(&uri).await.unwrap_or_default();
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
                if let Some(range) = value_range_for_path(&text, &path) {
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

            if let Some((path, replacement)) = type_fix_from_message(&text, &diagnostic.message) {
                if let Some(range) = value_range_for_path(&text, &path) {
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
                if let (Some(field), Some(insert)) = (
                    find_field_by_path(schema, &path),
                    insert_position_for_object(&text, &split_path(&parent_path)),
                ) {
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
        let Some(text) = self.document_text_for(&uri).await else {
            return Ok(None);
        };
        if is_schema_document(&uri, &text) {
            return Ok(None);
        }

        let position = params.text_document_position_params.position;

        // A `@schema "..."` directive jumps to the top of the schema file.
        if schema_directive(&text)
            .is_some_and(|directive| directive.line == position.line as usize + 1)
        {
            return Ok(self
                .schema_uri_for_document(&uri, &text)
                .await
                .map(|schema_uri| {
                    GotoDefinitionResponse::Scalar(Location {
                        uri: schema_uri,
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    })
                }));
        }

        // Otherwise jump from a config key to its schema field/block definition.
        let Some(path) = path_at_position(&text, position) else {
            return Ok(None);
        };
        let Some(schema_text) = self.schema_text_for(&uri).await else {
            return Ok(None);
        };
        let Ok(schema) = SchemaDocument::from_str(&schema_text) else {
            return Ok(None);
        };
        let Some(schema_uri) = self.schema_uri_for_document(&uri, &text).await else {
            return Ok(None);
        };
        let Some(line) = definition_line_for_path(&schema, &path) else {
            return Ok(None);
        };

        let name = path.last().map(String::as_str).unwrap_or_default();
        let range = identifier_range(&schema_text, line, name);
        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri: schema_uri,
            range,
        })))
    }

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;
        let Some(text) = self.document_text_for(&uri).await else {
            return Ok(None);
        };
        let Some((path, schema_uri)) = self.rename_target(&uri, &text, position).await else {
            return Ok(None);
        };

        // No schema: fall back to single-file references within this document.
        let Some(schema_uri) = schema_uri else {
            let locations = references_in_document(&text, &path)
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
        let Some(text) = self.document_text_for(&uri).await else {
            return Ok(None);
        };

        // Inside a schema document, allow renaming a field declaration.
        if is_schema_document(&uri, &text) {
            let Ok(schema) = SchemaDocument::from_str(&text) else {
                return Ok(None);
            };
            let Some(path) = schema_path_at_position(&schema, position) else {
                return Ok(None);
            };
            let name = path.last().map(String::as_str).unwrap_or_default();
            return Ok(Some(PrepareRenameResponse::Range(identifier_range(
                &text,
                position.line,
                name,
            ))));
        }

        Ok(key_range_at_position(&text, position).map(PrepareRenameResponse::Range))
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
        let Some(text) = self.document_text_for(&uri).await else {
            return Ok(None);
        };
        let Some((path, schema_uri)) = self.rename_target(&uri, &text, position).await else {
            return Ok(None);
        };

        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

        match schema_uri {
            // No schema: single-file rename within this document.
            None => {
                let edits: Vec<TextEdit> = references_in_document(&text, &path)
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
        let Some(text) = self.document_text_for(&uri).await else {
            return Ok(None);
        };
        let Some(formatted) = format_document(&text) else {
            return Ok(None);
        };

        Ok(Some(vec![TextEdit {
            range: full_document_range(&text),
            new_text: formatted,
        }]))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = params.text_document;
        self.documents.write().await.insert(
            document.uri,
            OpenDocument {
                text: document.text,
                version: document.version,
            },
        );
        self.validate_all_open_documents().await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            self.documents.write().await.insert(
                params.text_document.uri,
                OpenDocument {
                    text: change.text,
                    version: params.text_document.version,
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

fn looks_like_schema_text(text: &str) -> bool {
    text.lines()
        .map(|line| code_part(line).trim())
        .find(|line| !line.is_empty())
        .is_some_and(|line| line.starts_with("schema ") && line.ends_with(':'))
}

fn schema_directive(text: &str) -> Option<SchemaDirective> {
    for (index, line) in text.lines().enumerate() {
        let code = code_part(line).trim_start();
        let leading_whitespace = line.len().saturating_sub(line.trim_start().len());
        let Some(after_schema) = code.strip_prefix("@schema") else {
            continue;
        };

        let after_schema = after_schema.trim_start();
        let quote_offset = code.find('"')?;
        let Some(reference) = parse_quoted_string(after_schema) else {
            continue;
        };

        return Some(SchemaDirective {
            reference,
            line: index + 1,
            column: leading_whitespace + quote_offset + 1,
        });
    }

    None
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

fn recovery_diagnostics(text: &str) -> Vec<RuneDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut stack: Vec<(String, usize, usize)> = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let line_no = index + 1;
        let code = code_part(line);
        let trimmed = code.trim();
        if trimmed.is_empty() || trimmed.starts_with("gather ") {
            continue;
        }

        if trimmed.starts_with("@schema") && schema_directive(line).is_none() {
            let column = line.find("@schema").unwrap_or(0) + 1;
            diagnostics.push(
                RuneDiagnostic::error("Malformed @schema directive")
                    .with_range(line_no, column, column + "@schema".len())
                    .with_hint("Use: @schema \"name\" or @schema \"./schema.rune\""),
            );
            continue;
        }

        if trimmed.starts_with('@') {
            continue;
        }

        if trimmed == "end" || trimmed == "endif" {
            if stack.pop().is_none() {
                let column = line.find(trimmed).unwrap_or(0) + 1;
                diagnostics.push(
                    RuneDiagnostic::error(format!("Unexpected '{}' without open block", trimmed))
                        .with_range(line_no, column, column + trimmed.len())
                        .with_hint("Remove this closing keyword or add a matching block above"),
                );
            }
            continue;
        }

        if let Some(name) = block_name(trimmed) {
            let column = line.find(&name).unwrap_or(0) + 1;
            stack.push((name, line_no, column));
            continue;
        }

        if let Some(key) = assignment_key(trimmed) {
            let rest = trimmed.get(key.len()..).unwrap_or_default().trim();
            if rest.is_empty() {
                let column = line.find(&key).unwrap_or(0) + 1;
                diagnostics.push(
                    RuneDiagnostic::error(format!("Missing value for '{}'", key))
                        .with_range(line_no, column, column + key.len())
                        .with_hint("Add a value after the key"),
                );
            }
        }
    }

    for (name, line_no, column) in stack {
        diagnostics.push(
            RuneDiagnostic::error(format!("Unclosed object block '{}'; expected 'end'", name))
                .with_range(line_no, column, column + name.len())
                .with_hint(format!("Add 'end' to close the '{}' block", name)),
        );
    }

    diagnostics
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

fn type_fix_from_message(text: &str, message: &str) -> Option<(Vec<String>, TypeReplacement)> {
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
    let range = value_range_for_path(text, &path)?;
    let value = value_text_for_range(text, range)?.trim().to_string();

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

fn insert_position_for_object(text: &str, path: &[String]) -> Option<InsertPosition> {
    let mut stack = Vec::<String>::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = code_part(line).trim();
        if trimmed == "end" || trimmed == "endif" {
            stack.pop();
            continue;
        }

        if let Some(name) = block_name(trimmed) {
            stack.push(name);
            if stack == path {
                let indent = line
                    .chars()
                    .take_while(|ch| ch.is_whitespace())
                    .collect::<String>()
                    + "  ";
                return Some(InsertPosition {
                    position: Position::new((index + 1) as u32, 0),
                    indent,
                });
            }
        }
    }

    None
}

fn value_range_for_path(text: &str, path: &[String]) -> Option<Range> {
    let field = path.last()?;

    for (index, line) in text.lines().enumerate() {
        let Some(line_path) = path_at_position(text, Position::new(index as u32, 0)) else {
            continue;
        };
        if line_path != path {
            continue;
        }

        let key_start = line.find(field)?;
        let after_key = key_start + field.len();
        let value_start = line[after_key..]
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
            .map(|(idx, _)| after_key + idx)?;
        let value_end = line[value_start..]
            .find('#')
            .map(|idx| value_start + idx)
            .unwrap_or(line.len());

        return Some(Range {
            start: Position::new(index as u32, value_start as u32),
            end: Position::new(index as u32, value_end as u32),
        });
    }

    None
}

fn value_text_for_range(text: &str, range: Range) -> Option<&str> {
    if range.start.line != range.end.line {
        return None;
    }

    let line = text.lines().nth(range.start.line as usize)?;
    line.get(range.start.character as usize..range.end.character as usize)
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
    }
}

fn diagnostic_from_error(error: RuneError) -> RuneDiagnostic {
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
        } => diagnostic_with_location(message, line, column, hint, code),
        RuneError::UnclosedString {
            quote: _,
            line,
            column,
            hint,
            code,
        } => diagnostic_with_location("Unclosed string literal", line, column, hint, code),
        RuneError::UnexpectedCharacter {
            character,
            line,
            column,
            hint,
            code,
        } => diagnostic_with_location(
            format!("Unexpected character '{}'", character),
            line,
            column,
            hint,
            code,
        ),
        RuneError::ValidationError {
            message,
            line,
            column,
            hint,
            code,
        } => diagnostic_with_location(message, line, column, hint, code),
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
    text: &str,
    position: Position,
    config_dir: Option<&Path>,
) -> Vec<CompletionItem> {
    let before_cursor = line_before_cursor(text, position);
    if is_schema_directive_context(&before_cursor) {
        return schema_reference_completion_items(config_dir);
    }

    if before_cursor.contains('$') {
        return dollar_reference_completion_items();
    }

    let stack = object_stack_before_line(text, position.line as usize);

    if let (Some(schema), Some(key)) = (schema, current_key_before_cursor(&before_cursor)) {
        let mut path = stack.clone();
        path.push(key);

        if let Some(field) = find_field_by_path(schema, &path) {
            if let SchemaType::Enum(values) = &field.kind {
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
        }
    }

    let used = used_keys_in_current_object(text, position.line as usize, &stack);
    let mut items = if let Some(schema) = schema {
        schema_field_completion_items(schema, &stack, &used)
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

fn path_at_position(text: &str, position: Position) -> Option<Vec<String>> {
    let line = text.lines().nth(position.line as usize)?;
    let trimmed = code_part(line).trim();
    if trimmed.is_empty() || trimmed == "end" || trimmed == "endif" {
        return None;
    }

    let mut path = object_stack_before_line(text, position.line as usize);
    let key = block_name(trimmed).or_else(|| assignment_key(trimmed))?;
    path.push(key);
    Some(path)
}

fn object_stack_before_line(text: &str, line_index: usize) -> Vec<String> {
    let mut stack = Vec::new();

    for line in text.lines().take(line_index) {
        let trimmed = code_part(line).trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "end" || trimmed == "endif" {
            stack.pop();
            continue;
        }

        if let Some(name) = block_name(trimmed) {
            stack.push(name);
        }
    }

    stack
}

fn used_keys_in_current_object(
    text: &str,
    line_index: usize,
    target_stack: &[String],
) -> Vec<String> {
    let mut stack = Vec::new();
    let mut used = Vec::new();

    for line in text.lines().take(line_index) {
        let trimmed = code_part(line).trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "end" || trimmed == "endif" {
            stack.pop();
            continue;
        }

        if stack.as_slice() == target_stack {
            if let Some(key) = assignment_key(trimmed).or_else(|| block_name(trimmed)) {
                if !used.contains(&key) {
                    used.push(key);
                }
            }
        }

        if let Some(name) = block_name(trimmed) {
            stack.push(name);
        }
    }

    used
}

fn line_before_cursor(text: &str, position: Position) -> String {
    text.lines()
        .nth(position.line as usize)
        .map(|line| line.chars().take(position.character as usize).collect())
        .unwrap_or_default()
}

fn current_key_before_cursor(before_cursor: &str) -> Option<String> {
    let trimmed = code_part(before_cursor).trim_start();
    let key = assignment_key(trimmed)?;

    let after_key = trimmed.get(key.len()..)?.trim_start();
    (!after_key.is_empty()).then_some(key)
}

fn block_name(trimmed: &str) -> Option<String> {
    let name = trimmed.strip_suffix(':')?.trim();
    if name.starts_with("schema ") || !is_identifier(name) {
        return None;
    }

    Some(name.into())
}

fn assignment_key(trimmed: &str) -> Option<String> {
    let key = trimmed.split_whitespace().next()?;
    if is_identifier(key) {
        Some(key.into())
    } else {
        None
    }
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn code_part(line: &str) -> &str {
    line.split_once('#').map(|(code, _)| code).unwrap_or(line)
}

fn document_symbols(text: &str) -> Vec<DocumentSymbol> {
    let mut stack: Vec<String> = Vec::new();
    let mut symbols = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = code_part(line).trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "end" || trimmed == "endif" {
            stack.pop();
            continue;
        }

        if let Some(name) = block_name(trimmed) {
            let mut path = stack.clone();
            path.push(name.clone());
            symbols.push(line_symbol(
                path.join("."),
                SymbolKind::OBJECT,
                index,
                line,
                &name,
            ));
            stack.push(name);
            continue;
        }

        if let Some(key) = assignment_key(trimmed) {
            let mut path = stack.clone();
            path.push(key.clone());
            symbols.push(line_symbol(
                path.join("."),
                SymbolKind::FIELD,
                index,
                line,
                &key,
            ));
        }
    }

    symbols
}

fn line_symbol(
    name: String,
    kind: SymbolKind,
    index: usize,
    line: &str,
    selection: &str,
) -> DocumentSymbol {
    let start = line.find(selection).unwrap_or(0) as u32;
    let end = start + selection.len() as u32;
    let line = index as u32;

    #[allow(deprecated)]
    DocumentSymbol {
        name,
        detail: None,
        kind,
        tags: None,
        deprecated: None,
        range: Range {
            start: Position::new(line, 0),
            end: Position::new(line, end),
        },
        selection_range: Range {
            start: Position::new(line, start),
            end: Position::new(line, end),
        },
        children: None,
    }
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

/// Char range of the first occurrence of `name` on a given 0-based line.
///
/// Positions use the LSP default UTF-16 encoding; RUNE identifiers are ASCII,
/// so byte offsets and UTF-16 offsets coincide here.
fn identifier_range(text: &str, line_index: u32, name: &str) -> Range {
    let line = text.lines().nth(line_index as usize).unwrap_or("");
    let start = line.find(name).unwrap_or(0) as u32;
    let end = start + name.len() as u32;
    Range::new(
        Position::new(line_index, start),
        Position::new(line_index, end),
    )
}

/// Range of the renameable key on the cursor line, when the cursor sits on it.
fn key_range_at_position(text: &str, position: Position) -> Option<Range> {
    let line = text.lines().nth(position.line as usize)?;
    let trimmed = code_part(line).trim();
    let key = block_name(trimmed).or_else(|| assignment_key(trimmed))?;
    let range = identifier_range(text, position.line, &key);

    (position.character >= range.start.character && position.character <= range.end.character)
        .then_some(range)
}

/// Every occurrence of `target_path` (key ranges) within a single document.
///
/// Walks the document tracking the block stack so same-named keys in other
/// objects are not matched.
fn references_in_document(text: &str, target_path: &[String]) -> Vec<Range> {
    let mut stack: Vec<String> = Vec::new();
    let mut ranges = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = code_part(line).trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "end" || trimmed == "endif" {
            stack.pop();
            continue;
        }

        if let Some(name) = block_name(trimmed) {
            let mut path = stack.clone();
            path.push(name.clone());
            if path == target_path {
                ranges.push(identifier_range(text, index as u32, &name));
            }
            stack.push(name);
            continue;
        }

        if let Some(key) = assignment_key(trimmed) {
            let mut path = stack.clone();
            path.push(key.clone());
            if path == target_path {
                ranges.push(identifier_range(text, index as u32, &key));
            }
        }
    }

    ranges
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

/// Re-indent a document to two spaces per nesting level.
///
/// Object and conditional blocks both open with a trailing `:` and close with
/// `end`/`endif`; `else`/`elseif` branches dedent for their own line and indent
/// what follows. Only leading indentation changes — trailing content (including
/// inline comments) and blank lines are preserved, as is the final-newline state.
fn format_document(text: &str) -> Option<String> {
    let mut depth: usize = 0;
    let mut out = String::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            out.push('\n');
            continue;
        }

        let code = code_part(line).trim();
        if is_block_closer(code) || is_branch_continuation(code) {
            depth = depth.saturating_sub(1);
        }

        out.push_str(&"  ".repeat(depth));
        out.push_str(line.trim_start());
        out.push('\n');

        if is_block_opener(code) || is_branch_continuation(code) {
            depth += 1;
        }
    }

    if !text.ends_with('\n') {
        out.pop();
    }

    (out != text).then_some(out)
}

fn is_block_closer(code: &str) -> bool {
    code == "end" || code == "endif"
}

fn is_branch_continuation(code: &str) -> bool {
    let head = code.strip_suffix(':').unwrap_or(code).trim();
    matches!(
        head.split_whitespace().next(),
        Some("else") | Some("elseif") | Some("else-if")
    )
}

fn is_block_opener(code: &str) -> bool {
    code.ends_with(':') && !is_branch_continuation(code)
}

/// Range spanning the whole document, for a full-buffer replacement edit.
fn full_document_range(text: &str) -> Range {
    let line_count = text.lines().count() as u32;
    let trailing_newline = text.ends_with('\n');
    let end = if trailing_newline {
        Position::new(line_count, 0)
    } else {
        let last_len = text.lines().last().map(str::len).unwrap_or(0) as u32;
        Position::new(line_count.saturating_sub(1), last_len)
    };

    Range::new(Position::new(0, 0), end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_schema_directive() {
        let directive = schema_directive(
            r#"
app:
  name "RuneApp"
end
@schema "stasis"
"#,
        )
        .unwrap();

        assert_eq!(directive.reference, "stasis");
        assert_eq!(directive.line, 5);
        assert_eq!(directive.column, 9);
    }

    #[test]
    fn schema_directive_allows_path_references() {
        let directive = schema_directive(r#"@schema "./schemas/app.rune""#).unwrap();
        assert_eq!(directive.reference, "./schemas/app.rune");
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
        let diagnostics = recovery_diagnostics(
            r#"
app:
  name
  server:
    host "localhost"
"#,
        );

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
            "app:\n  name \"RuneApp\"\n  ",
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
        let completions = config_completion_items(None, "@schema \"", Position::new(0, 9), None);

        assert!(completions.iter().any(|item| item.label == "./schema.rune"));
        assert!(completions.iter().any(|item| item.label == "./schemas/"));
    }

    #[test]
    fn type_fix_removes_quotes_for_numeric_values() {
        let text = "app:\n  port \"8080\"\nend\n";
        let (path, replacement) =
            type_fix_from_message(text, "'app.port' expected int, got string").unwrap();

        assert_eq!(path, vec!["app", "port"]);
        assert_eq!(replacement.new_text, "8080");
    }

    #[test]
    fn type_fix_handles_nested_numeric_values() {
        let text = "app:\n  server:\n    port \"8080\"\n  end\nend\n";
        let (path, replacement) =
            type_fix_from_message(text, "'app.server.port' expected int, got string").unwrap();

        assert_eq!(path, vec!["app", "server", "port"]);
        assert_eq!(replacement.new_text, "8080");
    }

    #[test]
    fn type_fix_handles_lsp_diagnostic_hint_text() {
        let text = "app:\n  name \"RuneApp\"\n  environment \"prod\"\n\n  server:\n    host \"localhost\"\n    port \"8080\"\n  end\n\n  plugins [\"auth\", 42]\nend\n";
        let (_, replacement) = type_fix_from_message(
            text,
            "'app.server.port' expected int, got string\nHint: Check around: port \"8080\"",
        )
        .unwrap();

        assert_eq!(replacement.title, "Remove quotes to make int");
        assert_eq!(replacement.new_text, "8080");
    }

    #[test]
    fn references_find_all_uses_of_a_scoped_path() {
        let text = "app:\n  server:\n    port 8080\n  end\n  port 9090\nend\n";
        let ranges = references_in_document(&text, &["app".into(), "server".into(), "port".into()]);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start.line, 2);
        // The unscoped `app.port` on line 4 must not be matched.
        assert!(ranges.iter().all(|range| range.start.line != 4));
    }

    #[test]
    fn references_locate_the_key_identifier_range() {
        let text = "app:\n  name \"RuneApp\"\nend\n";
        let ranges = references_in_document(&text, &["app".into(), "name".into()]);

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
        let formatted = format_document(messy).expect("formatting should change the text");

        let expected = "app:\n  name \"RuneApp\"\n  server:\n    port 8080\n    if debug:\n      level \"high\"\n    else:\n      level \"low\"\n    endif\n  end\nend\n";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn format_document_is_idempotent_and_preserves_comments_and_blanks() {
        let source =
            "app:\n  name \"RuneApp\" # the app name\n\n  server:\n    port 8080\n  end\nend\n";
        // Already well-formed: no change.
        assert_eq!(format_document(source), None);

        let messy = "app:\nname \"RuneApp\" # the app name\n\nend\n";
        let once = format_document(messy).unwrap();
        // Formatting the result again is a no-op.
        assert_eq!(format_document(&once), None);
        assert!(once.contains("# the app name"));
        assert!(once.contains("\n\n"));
    }

    #[test]
    fn key_range_only_matches_when_cursor_is_on_the_key() {
        let text = "app:\n  name \"RuneApp\"\nend\n";
        // Cursor on the `name` key.
        assert_eq!(
            key_range_at_position(&text, Position::new(1, 3)),
            Some(Range::new(Position::new(1, 2), Position::new(1, 6)))
        );
        // Cursor on the value: nothing renameable.
        assert_eq!(key_range_at_position(&text, Position::new(1, 9)), None);
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
