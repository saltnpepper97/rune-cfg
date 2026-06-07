// Author: Dustin Pilgrim
// License: MIT

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, Position, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer};

use crate::diagnostic::{DiagnosticSeverity, RuneDiagnostic};
use crate::{RuneConfig, RuneError, SchemaDocument};

#[derive(Debug, Clone)]
struct OpenDocument {
    text: String,
    version: i32,
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
            Err(error) => return vec![diagnostic_from_error(error)],
        };

        let Some(schema_text) = self.schema_text_for(uri).await else {
            return Vec::new();
        };

        let schema = match SchemaDocument::from_str(&schema_text) {
            Ok(schema) => schema,
            Err(error) => return vec![diagnostic_from_error(error)],
        };

        config.validate_schema(&schema)
    }

    async fn schema_text_for(&self, uri: &Url) -> Option<String> {
        let schema_uri = self.schema_uri_for(uri).await?;

        if let Some(document) = self.documents.read().await.get(&schema_uri) {
            return Some(document.text.clone());
        }

        let path = schema_uri.to_file_path().ok()?;
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
