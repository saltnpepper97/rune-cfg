// Author: Dustin Pilgrim
// License: MIT

//! Black-box LSP regression tests for the `rune-lsp` language server.
//!
//! These tests drive [`RuneLanguageServer`] the way an editor client does:
//! through `tower_lsp::LspService` with JSON-RPC requests and notifications
//! only. Nothing private to the server crate is imported, so every assertion
//! describes observable protocol behavior.
//!
//! The tests below were originally written against a line-based server
//! implementation that reported `app.if`, dropped fields out of their object
//! after `endif`, answered `prepareRename` from the cursor's line alone, and
//! measured positions in UTF-8 bytes. They are regression tests now: each one
//! names the defect it pins down in its doc comment, and none of them shape
//! their assertions around any particular implementation.

use std::path::Path;
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use rune_cfg::lsp::RuneLanguageServer;
use serde_json::{Value, json};
use tokio::time::timeout;
use tower::{Service, ServiceExt};
use tower_lsp::jsonrpc::Request as JsonRpcRequest;
use tower_lsp::lsp_types::Url;
use tower_lsp::{ClientSocket, LspService};

/// `app:` with a conditional block followed by an assignment that belongs to
/// `app`, not to the conditional.
const CONDITIONAL_CONFIG: &str = r#"app:
  if debug:
    feature true
  endif
  name "Rune"
end
"#;

/// A schema declaring one string field. Line 1 is `  name string`.
const SCHEMA_STRING_FIELD: &str = r#"schema app:
  name string
end
"#;

/// The same schema with `app.name` widened to `int`, used to prove that an open
/// config is revalidated when its open schema changes.
const SCHEMA_INT_FIELD: &str = r#"schema app:
  name int
end
"#;

/// A config that satisfies [`SCHEMA_STRING_FIELD`].
const CONFIG_STRING_VALUE: &str = r#"app:
  name "Rune"
end
"#;

/// Deliberately unformatted and without a final newline. The emoji is one code
/// point, two UTF-16 code units, and four UTF-8 bytes.
const UNFORMATTED_CONFIG: &str = "app:\nname \"😀\"";

/// Budget for one server-to-client message. Every message this server sends is
/// produced while the triggering request or notification is being handled, so
/// this only bounds the wait when an expectation is wrong.
const MESSAGE_BUDGET: Duration = Duration::from_secs(5);

/// Scheduler turns used to hand over a message that is still sitting in the
/// server's capacity-1 socket channel when a call returns.
const FORWARDER_TURNS: usize = 32;

/// One server-to-client JSON-RPC message observed on the client socket.
#[derive(Debug, Clone)]
struct ServerMessage {
    method: String,
    params: Value,
}

/// The payload of a `textDocument/publishDiagnostics` notification.
#[derive(Debug, Clone)]
struct PublishedDiagnostics {
    uri: Url,
    version: Option<i32>,
    diagnostics: Vec<Value>,
}

impl ServerMessage {
    /// Turns the message into the diagnostics payload it carries, or `None`
    /// for unrelated messages such as `window/logMessage`.
    fn into_publish_diagnostics(self) -> Option<PublishedDiagnostics> {
        if self.method != "textDocument/publishDiagnostics" {
            return None;
        }

        Some(PublishedDiagnostics {
            uri: self.params.get("uri")?.as_str()?.parse().ok()?,
            version: self
                .params
                .get("version")
                .and_then(Value::as_i64)
                .map(|version| version as i32),
            diagnostics: self.params.get("diagnostics")?.as_array()?.clone(),
        })
    }
}

/// Looks up the notifications published for one document version, panicking
/// with everything that was collected when it is missing.
fn diagnostics_for<'a>(
    collected: &'a [PublishedDiagnostics],
    uri: &Url,
    version: Option<i32>,
) -> &'a PublishedDiagnostics {
    collected
        .iter()
        .find(|notification| notification.uri == *uri && notification.version == version)
        .unwrap_or_else(|| {
            panic!("no publishDiagnostics for {uri} version {version:?}; collected: {collected:#?}")
        })
}

/// Document symbol names in the order the server reported them.
fn symbol_names(result: &Value) -> Vec<String> {
    result
        .as_array()
        .unwrap_or_else(|| panic!("documentSymbol must return a symbol array, got {result}"))
        .iter()
        .map(|symbol| {
            symbol
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("document symbol without a name: {symbol}"))
                .to_string()
        })
        .collect()
}

/// A `rune-lsp` instance driven over JSON-RPC, plus a temporary workspace root.
struct LspHarness {
    service: LspService<RuneLanguageServer>,
    incoming: UnboundedReceiver<ServerMessage>,
    next_request_id: i64,
    forwarder: tokio::task::JoinHandle<()>,
    workspace: tempfile::TempDir,
}

impl Drop for LspHarness {
    fn drop(&mut self) {
        self.forwarder.abort();
    }
}

impl LspHarness {
    /// Boots the same `LspService`/`ClientSocket` pair the `rune-lsp` binary
    /// uses, points it at a fresh temporary workspace, and completes the LSP
    /// handshake.
    async fn start() -> Self {
        let workspace = tempfile::tempdir().expect("temporary workspace");
        let (service, socket) = LspService::new(RuneLanguageServer::new);
        let (sender, incoming) = unbounded();
        let forwarder = tokio::spawn(forward_client_messages(socket, sender));

        let mut harness = Self {
            service,
            incoming,
            next_request_id: 0,
            forwarder,
            workspace,
        };

        let root_uri =
            Url::from_directory_path(harness.workspace.path()).expect("workspace root uri");
        let result = harness
            .request(
                "initialize",
                json!({
                    "processId": Value::Null,
                    "rootUri": root_uri,
                    "capabilities": {},
                }),
            )
            .await;
        assert!(
            result.get("capabilities").is_some(),
            "initialize must report server capabilities, got {result}"
        );

        harness.notify("initialized", json!({})).await;
        // The `initialized` handler logs to the window; nothing else should be
        // on the wire before a document is opened.
        harness.discard_server_messages().await;

        harness
    }

    /// Path of the temporary workspace, which stays empty: documents are only
    /// ever opened in memory.
    fn workspace_path(&self) -> &Path {
        self.workspace.path()
    }

    /// File URI for a document in the temporary workspace.
    fn document_uri(&self, file_name: &str) -> Url {
        Url::from_file_path(self.workspace.path().join(file_name)).expect("document uri")
    }

    /// Sends a JSON-RPC request and returns its `result` payload.
    async fn request(&mut self, method: &'static str, params: Value) -> Value {
        let id = self.next_request_id;
        self.next_request_id += 1;

        let request = JsonRpcRequest::build(method).params(params).id(id).finish();
        let response = self
            .service
            .ready()
            .await
            .expect("service accepts the request")
            .call(request)
            .await
            .expect("the server handles the request")
            .expect("a request receives a response");

        let (_, result) = response.into_parts();
        result.unwrap_or_else(|error| panic!("{method} failed: {error:?}"))
    }

    /// Sends a JSON-RPC notification. Notifications produce no response, but the
    /// handler still runs to completion before this returns, so server-side work
    /// such as revalidation is finished here.
    async fn notify(&mut self, method: &'static str, params: Value) {
        let request = JsonRpcRequest::build(method).params(params).finish();
        let _ = self
            .service
            .ready()
            .await
            .expect("service accepts the notification")
            .call(request)
            .await
            .expect("the server handles the notification");
    }

    /// Opens a document with full-text sync, which is the sync kind the server
    /// advertises.
    async fn did_open(&mut self, uri: &Url, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "runecfg",
                    "version": 1,
                    "text": text,
                }
            }),
        )
        .await;
    }

    /// Replaces the whole buffer of an already open document.
    async fn replace_document(&mut self, uri: &Url, version: i32, text: &str) {
        self.notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": version },
                "contentChanges": [{ "text": text }],
            }),
        )
        .await;
    }

    /// Hands over every message the server has already produced.
    ///
    /// A call only returns once its handler finished, which leaves at most one
    /// message in the server's capacity-1 socket channel. Yielding a bounded
    /// number of times lets the forwarder move it across without any sleeps.
    async fn take_server_messages(&mut self) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        let mut quiet_turns = 0;

        while quiet_turns < FORWARDER_TURNS {
            while let Ok(message) = self.incoming.try_recv() {
                messages.push(message);
                quiet_turns = 0;
            }

            tokio::task::yield_now().await;
            quiet_turns += 1;
        }

        messages
    }

    /// Drops everything the server published up to this point, so a later
    /// assertion only observes notifications triggered by the next change.
    async fn discard_server_messages(&mut self) {
        self.take_server_messages().await;
    }

    /// Collects at least `needed` `textDocument/publishDiagnostics`
    /// notifications, ignoring unrelated notifications such as
    /// `window/logMessage`.
    ///
    /// Each notification is awaited with a timeout and the collection is
    /// returned as-is when it runs out, so a wrong expectation fails an
    /// assertion that prints what was actually published instead of hanging.
    async fn collect_diagnostics(&mut self, needed: usize) -> Vec<PublishedDiagnostics> {
        let mut collected: Vec<PublishedDiagnostics> = self
            .take_server_messages()
            .await
            .into_iter()
            .filter_map(ServerMessage::into_publish_diagnostics)
            .collect();

        while collected.len() < needed {
            match timeout(MESSAGE_BUDGET, self.incoming.next()).await {
                Ok(Some(message)) => {
                    if let Some(diagnostics) = message.into_publish_diagnostics() {
                        collected.push(diagnostics);
                    }
                }
                // The socket closed, or nothing else is coming.
                Ok(None) | Err(_) => break,
            }
        }

        collected
    }
}

/// Continuously drains the client socket into the test's unbounded channel.
///
/// The server sends every `window/logMessage` and `publishDiagnostics` through
/// a channel with room for a single message, so a server handler blocks until
/// the socket is drained. Without this task, opening a document would deadlock.
async fn forward_client_messages(socket: ClientSocket, sender: UnboundedSender<ServerMessage>) {
    let mut socket = socket;

    while let Some(request) = socket.next().await {
        let message = ServerMessage {
            method: request.method().to_string(),
            params: request.params().cloned().unwrap_or(Value::Null),
        };

        if sender.unbounded_send(message).is_err() {
            break;
        }
    }
}

/// `endif` closes the conditional block, not the enclosing object, and the
/// `if <condition>:` header is not a document symbol.
///
/// Regression guard: the earlier line-based symbol walker read the `if debug:`
/// line as an assignment to a key named `if` and popped the object stack when
/// it reached `endif`, so the trailing assignment was reported as a top-level
/// `name` symbol instead of `app.name`.
#[tokio::test]
async fn document_symbols_preserve_object_scope_across_endif() {
    let mut harness = LspHarness::start().await;
    let uri = harness.document_uri("config.rune");

    harness.did_open(&uri, CONDITIONAL_CONFIG).await;
    harness.discard_server_messages().await;

    let result = harness
        .request(
            "textDocument/documentSymbol",
            json!({ "textDocument": { "uri": uri } }),
        )
        .await;
    let names = symbol_names(&result);

    assert_eq!(
        names,
        vec!["app", "app.feature", "app.name"],
        "`endif` must close the conditional and leave `app` as the scope of what follows"
    );
    assert!(
        !names.iter().any(|name| name == "app.if"),
        "the conditional header must not be reported as a field: {names:?}"
    );
}

/// `prepareRename` only offers a rename when the cursor sits on the identifier
/// that would be renamed.
///
/// Regression guard: resolving the field from the cursor's line alone answered
/// a cursor inside the `string` type with the `name` range (2..6) instead of
/// JSON null.
#[tokio::test]
async fn prepare_rename_returns_null_off_the_identifier() {
    let mut harness = LspHarness::start().await;
    let uri = harness.document_uri("schema.rune");

    harness.did_open(&uri, SCHEMA_STRING_FIELD).await;
    harness.discard_server_messages().await;

    // Line 1 is `  name string`; character 9 sits inside the `string` type.
    let off_identifier = harness
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 1, "character": 9 },
            }),
        )
        .await;
    assert_eq!(
        off_identifier,
        Value::Null,
        "a cursor inside the field type must not offer a rename"
    );

    // Control: a cursor inside `name` reports the exact identifier range.
    let on_identifier = harness
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 1, "character": 3 },
            }),
        )
        .await;
    assert_eq!(
        on_identifier,
        json!({
            "start": { "line": 1, "character": 2 },
            "end": { "line": 1, "character": 6 },
        }),
        "a cursor on `name` must report its exact range"
    );
}

/// Document formatting reports the replaced range in UTF-16 code units, which
/// is what LSP clients count positions in.
///
/// Regression guard: measuring the last line with `str::len()` (UTF-8 bytes)
/// reported character 11 for `name "😀"` instead of 9.
#[tokio::test]
async fn formatting_reports_utf16_end_position() {
    let mut harness = LspHarness::start().await;
    let uri = harness.document_uri("config.rune");

    harness.did_open(&uri, UNFORMATTED_CONFIG).await;
    harness.discard_server_messages().await;

    let result = harness
        .request(
            "textDocument/formatting",
            json!({
                "textDocument": { "uri": uri },
                "options": { "tabSize": 2, "insertSpaces": true },
            }),
        )
        .await;

    let edits = result
        .as_array()
        .unwrap_or_else(|| panic!("formatting must return an edit array, got {result}"));
    assert_eq!(
        edits.len(),
        1,
        "formatting must return a single full-document edit: {result}"
    );

    let edit = &edits[0];
    assert_eq!(
        edit["newText"],
        json!("app:\n  name \"😀\""),
        "the replacement must re-indent the assignment inside `app:`"
    );
    assert_eq!(
        edit["range"],
        json!({
            "start": { "line": 0, "character": 0 },
            // Line 1 is 11 UTF-8 bytes but 9 UTF-16 code units.
            "end": { "line": 1, "character": 9 },
        }),
        "the full-document range must be counted in UTF-16 code units"
    );
}

/// A schema change revalidates every open config bound to that open schema
/// document, republishing diagnostics for both documents.
///
/// The workspace stays empty: the config can only see the changes because the
/// schema is open in memory, so this proves the server uses open dependency
/// state rather than file contents. The order of the two notifications is not
/// assumed, because the server walks its open documents through a `HashMap`.
#[tokio::test]
async fn schema_change_republishes_diagnostics_for_open_dependents() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    harness.did_open(&schema_uri, SCHEMA_STRING_FIELD).await;
    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;

    // Opening each document revalidates every open document: one publication
    // for the schema after the first open, then the schema and the config after
    // the second.
    let published = harness.collect_diagnostics(3).await;
    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert!(
        config.diagnostics.is_empty(),
        "a config matching the open schema has no diagnostics: {published:#?}"
    );

    harness.discard_server_messages().await;

    // Neither file exists, so the config's schema can only be the open buffer.
    assert!(!harness.workspace_path().join("schema.rune").exists());
    assert!(!harness.workspace_path().join("config.rune").exists());

    // Widen `app.name` to int while the config stays open.
    harness
        .replace_document(&schema_uri, 2, SCHEMA_INT_FIELD)
        .await;
    let published = harness.collect_diagnostics(2).await;

    let schema = diagnostics_for(&published, &schema_uri, Some(2));
    assert!(
        schema.diagnostics.is_empty(),
        "the updated schema parses cleanly: {published:#?}"
    );

    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert_eq!(
        config.diagnostics.len(),
        1,
        "the open config must be revalidated against the new schema: {published:#?}"
    );

    let message = config.diagnostics[0]
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        message.contains("app.name")
            && message.contains("expected int")
            && message.contains("got string"),
        "unexpected diagnostic message: {message}"
    );
}

/// Completion after `endif` offers the fields of the object the conditional was
/// nested in, not the document's top level.
///
/// Regression guard: the earlier line-based completion walker popped the object
/// stack at `endif` and then treated the `if debug:` header as a key, so the
/// scope after `endif` was wrong.
#[tokio::test]
async fn completion_after_endif_uses_the_enclosing_object_scope() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    harness.did_open(&schema_uri, SCHEMA_STRING_FIELD).await;
    harness.did_open(&config_uri, CONDITIONAL_CONFIG).await;
    harness.discard_server_messages().await;

    // Line 4 is `  name "Rune"`, which still belongs to `app`.
    let result = harness
        .request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 4, "character": 2 },
            }),
        )
        .await;

    let labels: Vec<&str> = result
        .as_array()
        .unwrap_or_else(|| panic!("completion must return an item array, got {result}"))
        .iter()
        .filter_map(|item| item.get("label").and_then(Value::as_str))
        .collect();

    assert!(
        labels.contains(&"name"),
        "the field of the enclosing object must be offered: {labels:?}"
    );
    assert!(
        !labels.contains(&"app"),
        "a top-level scope would offer the `app` block instead: {labels:?}"
    );
}

/// The schema-scoped requests all agree on the field path the cursor resolves
/// to, and the schema declaration range is the field-name token itself.
#[tokio::test]
async fn schema_scoped_navigation_resolves_indexed_field_paths() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    harness.did_open(&schema_uri, SCHEMA_STRING_FIELD).await;
    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;
    harness.discard_server_messages().await;

    // `CONFIG_STRING_VALUE` is `app:\n  name "Rune"\nend\n`; line 1 holds the
    // `name` key, so character 3 sits inside that key token.
    let definition = harness
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 1, "character": 3 },
            }),
        )
        .await;
    assert_eq!(definition["uri"], json!(schema_uri.as_str()));
    assert_eq!(
        definition["range"],
        json!({
            "start": { "line": 1, "character": 2 },
            "end": { "line": 1, "character": 6 },
        }),
        "the declaration range must be the indexed field-name token"
    );

    let hover = harness
        .request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 1, "character": 3 },
            }),
        )
        .await;
    let hover_text = hover["contents"].as_str().unwrap_or_default();
    assert!(
        hover_text.contains("app.name"),
        "hover must describe app.name: {hover}"
    );

    let references = harness
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 1, "character": 3 },
                "context": { "includeDeclaration": true },
            }),
        )
        .await;
    let locations = references
        .as_array()
        .unwrap_or_else(|| panic!("references must return locations, got {references}"));
    assert_eq!(
        locations.len(),
        2,
        "one schema declaration plus one usage: {references}"
    );

    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 1, "character": 3 },
                "newName": "title",
            }),
        )
        .await;
    let changes = rename["changes"]
        .as_object()
        .unwrap_or_else(|| panic!("rename must return a workspace edit, got {rename}"));
    assert_eq!(
        changes.len(),
        2,
        "the schema declaration and the config usage are both renamed: {rename}"
    );
    assert_eq!(
        changes[config_uri.as_str()][0]["newText"],
        json!("title"),
        "the usage is replaced in place: {rename}"
    );
}
