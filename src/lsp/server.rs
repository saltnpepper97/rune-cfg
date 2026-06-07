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
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentSymbol, DocumentSymbolParams,
    DocumentSymbolResponse, Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, InitializedParams, InsertTextFormat, MarkedString,
    MessageType, OneOf, Position, Range, ServerCapabilities, SymbolKind,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url, WorkspaceEdit,
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
        let rune_diagnostics = if is_schema_file(uri) {
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
                .ok_or_else(|| schema_reference_diagnostic(&directive));
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
            .map_err(|_| schema_reference_diagnostic(directive))?;
        let config_dir = path
            .parent()
            .ok_or_else(|| schema_reference_diagnostic(directive))?;

        for candidate in schema_candidates(&directive.reference, config_dir) {
            if candidate.exists() || self.is_open_uri_for_path(&candidate).await {
                if let Ok(uri) = Url::from_file_path(candidate) {
                    return Ok(uri);
                }
            }
        }

        Err(schema_reference_diagnostic(directive))
    }

    async fn schema_for(&self, uri: &Url) -> Option<SchemaDocument> {
        let schema_text = self.schema_text_for(uri).await?;
        SchemaDocument::from_str(&schema_text).ok()
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

        let items = if is_schema_file(&uri) {
            schema_completion_items()
        } else {
            let schema = self.schema_for(&uri).await;
            config_completion_items(schema.as_ref(), &text, position)
        };

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        if is_schema_file(&uri) {
            return Ok(None);
        }

        let position = params.text_document_position_params.position;
        let Some(text) = self.document_text_for(&uri).await else {
            return Ok(None);
        };
        let Some(schema) = self.schema_for(&uri).await else {
            return Ok(None);
        };
        let Some(path) = path_at_position(&text, position) else {
            return Ok(None);
        };
        let Some(field) = find_field_by_path(&schema, &path) else {
            return Ok(None);
        };

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(field_hover(&path, field))),
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

fn schema_reference_diagnostic(directive: &SchemaDirective) -> RuneDiagnostic {
    RuneDiagnostic::error(format!("Schema '{}' was not found", directive.reference))
        .with_range(
            directive.line,
            directive.column,
            directive.column + directive.reference.len() + 2,
        )
        .with_hint("Check the @schema path or install the named schema")
        .with_code(701)
}

fn recovery_diagnostics(text: &str) -> Vec<RuneDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut stack: Vec<(String, usize, usize)> = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let line_no = index + 1;
        let code = code_part(line);
        let trimmed = code.trim();
        if trimmed.is_empty() || trimmed.starts_with('@') || trimmed.starts_with("gather ") {
            continue;
        }

        if trimmed == "end" || trimmed == "endif" {
            stack.pop();
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
        let line_path = path_at_position(text, Position::new(index as u32, 0))?;
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
) -> Vec<CompletionItem> {
    let before_cursor = line_before_cursor(text, position);
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

fn field_hover(path: &[String], field: &SchemaField) -> String {
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
}
