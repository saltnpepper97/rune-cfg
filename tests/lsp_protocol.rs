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
//!
//! The dependency and filesystem lifecycle tests at the end drive the same
//! server through the events an editor sends for buffers and for files on
//! disk: `didOpen`, `didChange`, `didClose`, `didChangeWatchedFiles`, and
//! `didChangeWorkspaceFolders`. Every one of them asserts the exact set of
//! published diagnostics, because the server's contract is that a document
//! unrelated to a change is never republished.

use std::path::Path;
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use rune_cfg::lsp::RuneLanguageServer;
use serde_json::{Value, json};
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

/// A config with an explicit relative schema reference and a string
/// `app.name`.
const CONFIG_WITH_CUSTOM_SCHEMA: &str = r#"@schema "./custom.rune"
app:
  name "Rune"
end
"#;

/// A config bound to `other.rune` next to it.
const CONFIG_BOUND_TO_OTHER_SCHEMA: &str = r#"@schema "./other.rune"
app:
  name "Rune"
end
"#;

/// A config bound to the `shared.rune` next to it.
const CONFIG_BOUND_TO_SHARED_SCHEMA: &str = r#"@schema "./shared.rune"
app:
  name "Rune"
end
"#;

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

/// Writes a file into a workspace root, creating any parent directories.
///
/// This is free-standing so a fixture can be written *before* `initialize`,
/// from the closure that builds the `initialize` params: the server indexes
/// the workspace once it knows the folders and the exclusions, and a file that
/// exists at that point is a workspace member from the start.
fn write_workspace_fixture(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent directory");
    }
    std::fs::write(path, text).expect("write file");
}

/// A `rune-lsp` instance driven over JSON-RPC, plus a temporary workspace root.
struct LspHarness {
    service: LspService<RuneLanguageServer>,
    incoming: UnboundedReceiver<ServerMessage>,
    next_request_id: i64,
    forwarder: tokio::task::JoinHandle<()>,
    workspace: tempfile::TempDir,
    initialize_result: Value,
}

impl Drop for LspHarness {
    fn drop(&mut self) {
        self.forwarder.abort();
    }
}

impl LspHarness {
    /// Boots the same `LspService`/`ClientSocket` pair the `rune-lsp` binary
    /// uses, points it at a fresh temporary workspace, and completes the LSP
    /// handshake with that workspace as `rootUri`.
    async fn start() -> Self {
        let mut harness = Self::boot().await;
        let root_uri =
            Url::from_directory_path(harness.workspace.path()).expect("workspace root uri");

        harness
            .initialize(json!({
                "processId": Value::Null,
                "rootUri": root_uri,
                "capabilities": {},
            }))
            .await;

        harness
    }

    /// Boots a harness and completes the handshake with the `initialize` params
    /// `build` derives from the temporary workspace path.
    ///
    /// This is how a test describes the workspace the client offers: no
    /// `rootUri` at all, one or more workspace folders, or a folder nested
    /// inside the temporary directory so its parent can hold a schema too.
    async fn start_with_initializer(build: impl FnOnce(&Path) -> Value) -> Self {
        let mut harness = Self::boot().await;
        let params = build(harness.workspace.path());

        harness.initialize(params).await;

        harness
    }

    /// Creates the service, the client socket and the forwarder task, without
    /// sending anything on the wire yet.
    async fn boot() -> Self {
        let workspace = tempfile::tempdir().expect("temporary workspace");
        let (service, socket) = LspService::new(RuneLanguageServer::new);
        let (sender, incoming) = unbounded();
        let forwarder = tokio::spawn(forward_client_messages(socket, sender));

        Self {
            service,
            incoming,
            next_request_id: 0,
            forwarder,
            workspace,
            initialize_result: Value::Null,
        }
    }

    /// Sends `initialize`, asserts the server reported capabilities, completes
    /// the handshake, and drops the log message `initialized` produces: nothing
    /// else is on the wire before a document is opened.
    async fn initialize(&mut self, params: Value) {
        let result = self.request("initialize", params).await;
        assert!(
            result.get("capabilities").is_some(),
            "initialize must report server capabilities, got {result}"
        );
        self.initialize_result = result;

        self.notify("initialized", json!({})).await;
        self.discard_server_messages().await;
    }

    /// The `InitializeResult` the server sent during the handshake.
    fn initialize_result(&self) -> &Value {
        &self.initialize_result
    }

    /// Path of the temporary workspace, which holds every file a test writes to
    /// disk: other documents are only ever opened in memory.
    fn workspace_path(&self) -> &Path {
        self.workspace.path()
    }

    /// File URI for a document in the temporary workspace.
    fn document_uri(&self, file_name: &str) -> Url {
        Url::from_file_path(self.workspace.path().join(file_name)).expect("document uri")
    }

    /// File URI for a path relative to the temporary workspace.
    fn file_uri(&self, relative: &str) -> Url {
        Url::from_file_path(self.workspace.path().join(relative)).expect("file uri")
    }

    /// Directory URI for a path relative to the temporary workspace, with the
    /// trailing separator a workspace folder carries.
    fn directory_uri(&self, relative: &str) -> Url {
        Url::from_directory_path(self.workspace.path().join(relative)).expect("directory uri")
    }

    /// Writes a file into the temporary workspace, creating parent directories.
    fn write_file(&self, relative: &str, text: &str) {
        write_workspace_fixture(self.workspace.path(), relative, text);
    }

    /// Removes a file from the temporary workspace.
    fn remove_file(&self, relative: &str) {
        std::fs::remove_file(self.workspace.path().join(relative)).expect("remove file");
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

    /// Sends a JSON-RPC request and returns either its `result` payload or the
    /// JSON-RPC error object the server replied with.
    ///
    /// Requests a test expects to be rejected use this instead of
    /// [`Self::request`], which panics on an error.
    async fn request_or_error(
        &mut self,
        method: &'static str,
        params: Value,
    ) -> Result<Value, Value> {
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
        result.map_err(|error| serde_json::to_value(error).expect("a JSON-RPC error is JSON"))
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

    /// Closes an open document.
    async fn did_close(&mut self, uri: &Url) {
        self.notify(
            "textDocument/didClose",
            json!({ "textDocument": { "uri": uri } }),
        )
        .await;
    }

    /// Reports file events the client's watcher observed, as `(uri, type)`
    /// pairs where the type is the LSP `FileChangeType` number: 1 created,
    /// 2 changed, 3 deleted.
    async fn did_change_watched_files(&mut self, events: &[(Url, i64)]) {
        let changes: Vec<Value> = events
            .iter()
            .map(|(uri, typ)| json!({ "uri": uri, "type": typ }))
            .collect();

        self.notify(
            "workspace/didChangeWatchedFiles",
            json!({ "changes": changes }),
        )
        .await;
    }

    /// Reports workspace folders the client added and removed.
    async fn did_change_workspace_folders(&mut self, added: &[Url], removed: &[Url]) {
        let folder = |uri: &Url| json!({ "uri": uri, "name": "workspace" });

        self.notify(
            "workspace/didChangeWorkspaceFolders",
            json!({
                "event": {
                    "added": added.iter().map(folder).collect::<Vec<_>>(),
                    "removed": removed.iter().map(folder).collect::<Vec<_>>(),
                }
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

    /// Every `textDocument/publishDiagnostics` notification the request or
    /// notification that just returned produced, in the order sent.
    ///
    /// Nothing here waits on a timeout, so asserting that a change publishes
    /// *nothing* costs no wall-clock time: a call only returns once its handler
    /// has finished, and the bounded number of scheduler turns is enough to
    /// forward the message the capacity-1 socket channel may still hold.
    async fn collect_diagnostics(&mut self) -> Vec<PublishedDiagnostics> {
        self.take_server_messages()
            .await
            .into_iter()
            .filter_map(ServerMessage::into_publish_diagnostics)
            .collect()
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

    // Clients are allowed to send rename directly without prepareRename. The
    // server must still reject a cursor on the type token rather than using the
    // declaration's line as a proxy for the field name.
    let direct_rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 1, "character": 9 },
                "newName": "title",
            }),
        )
        .await;
    assert_eq!(
        direct_rename,
        Value::Null,
        "direct rename must reject a cursor on the field type"
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
///
/// Opening a document only publishes for that document, so the two opens below
/// publish two notifications in total - the schema, then the config - and not
/// one per open document.
#[tokio::test]
async fn schema_change_republishes_diagnostics_for_open_dependents() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    harness.did_open(&schema_uri, SCHEMA_STRING_FIELD).await;
    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;

    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        2,
        "each open publishes only its own document: {published:#?}"
    );
    let schema = diagnostics_for(&published, &schema_uri, Some(1));
    assert!(
        schema.diagnostics.is_empty(),
        "a schema that parses cleanly has no diagnostics: {published:#?}"
    );
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
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        2,
        "the schema and the dependent config are republished: {published:#?}"
    );

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

/// A schema for the source-indexed diagnostic test, in CRLF form: `app.mode`
/// must be an int and `app.𐐀name` a string.
const SCHEMA_CONDITIONAL_FIELDS: &str =
    "schema app:\r\n  mode int required\r\n  \u{10400}name string required\r\nend\r\n";

/// A CRLF config whose `app.mode` is written in all three branches of an
/// `if`/`elseif`/`else` chain. The active branch is the `elseif`, so that is the
/// occurrence a diagnostic has to point at; the value carries a `#`, and the
/// second field is a quoted non-BMP key.
const CONDITIONAL_CRLF_CONFIG: &str = "first false\r\nsecond true\r\napp:\r\n  if first = true:\r\n    mode 100\r\n  elseif second = true:\r\n    mode \"beta#1\"\r\n  else:\r\n    mode 200\r\n  endif\r\n  \"\u{10400}name\" 42\r\nend\r\n";

/// The one published diagnostic whose message contains `needle`.
fn diagnostic_with_message<'a>(diagnostics: &'a [Value], needle: &str) -> &'a Value {
    let matching: Vec<&Value> = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains(needle)
        })
        .collect();

    assert_eq!(
        matching.len(),
        1,
        "expected exactly one diagnostic containing {needle:?}, got {diagnostics:#?}"
    );

    matching[0]
}

/// Schema diagnostics are reported at the exact source span of the key that
/// supplied the value: the active `elseif` occurrence, not an inactive sibling
/// writing the same path, and a quoted non-BMP key counted in UTF-16 code units
/// on a CRLF buffer.
#[tokio::test]
async fn schema_diagnostics_use_indexed_source_spans() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    harness
        .did_open(&schema_uri, SCHEMA_CONDITIONAL_FIELDS)
        .await;
    harness.did_open(&config_uri, CONDITIONAL_CRLF_CONFIG).await;

    // Each open publishes only its own document, so no unrelated document is
    // revalidated when a config is opened.
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        2,
        "each open publishes only its own document: {published:#?}"
    );
    assert!(
        diagnostics_for(&published, &schema_uri, Some(1))
            .diagnostics
            .is_empty(),
        "the CRLF schema parses cleanly: {published:#?}"
    );
    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert_eq!(
        config.diagnostics.len(),
        2,
        "each invalid field is reported once: {published:#?}"
    );

    // Line 6 is `    mode "beta#1"` - the elseif branch. The `#` inside the
    // value must not move the key, and the inactive `if`/`else` occurrences of
    // the same path must not be the ones underlined.
    let mode = diagnostic_with_message(&config.diagnostics, "app.mode");
    assert_eq!(mode["code"], json!(652));
    assert!(
        mode["message"]
            .as_str()
            .unwrap_or_default()
            .contains("expected int, got string"),
        "the selected elseif value is the one validated: {mode}"
    );
    assert_eq!(
        mode["range"],
        json!({
            "start": { "line": 6, "character": 4 },
            "end": { "line": 6, "character": 8 },
        })
    );

    // Line 10 is `  "𐐀name" 42`: the range covers the decoded key inside its
    // quotes, and 𐐀 is two UTF-16 code units, so the end column is 9 and not
    // the 11 UTF-8 bytes of the same text.
    let name = diagnostic_with_message(&config.diagnostics, "app.\u{10400}name");
    assert_eq!(name["code"], json!(652));
    assert_eq!(
        name["range"],
        json!({
            "start": { "line": 10, "character": 3 },
            "end": { "line": 10, "character": 9 },
        })
    );
}

/// A schema whose block root and field name are both quoted. The quotes are
/// not part of the names, so navigation and rename must use the span between
/// them.
const SCHEMA_QUOTED_KEYS: &str = "schema \"app\":\n  \"name\" string default \"x\"\nend\n";

/// A config exercising [`SCHEMA_QUOTED_KEYS`] with quoted keys of its own and a
/// `#` inside a value that must not shift the key position.
const CONFIG_QUOTED_KEYS: &str = "app:\n  \"name\" \"a#b\"\nend\n";

/// Navigation and rename on a quoted schema key use the inner name span: the
/// quotes survive a rename because the edit range never covers them, and a
/// cursor on a quote, on a type, or on a default is not a rename target.
#[tokio::test]
async fn quoted_schema_keys_use_inner_name_spans() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    harness.did_open(&schema_uri, SCHEMA_QUOTED_KEYS).await;
    harness.did_open(&config_uri, CONFIG_QUOTED_KEYS).await;
    harness.discard_server_messages().await;

    // `SCHEMA_QUOTED_KEYS` line 1 is `  "name" string default "x"`: the name
    // occupies characters 3..7 between the quotes.
    let name_range = json!({
        "start": { "line": 1, "character": 3 },
        "end": { "line": 1, "character": 7 },
    });

    // `CONFIG_QUOTED_KEYS` line 1 is `  "name" "a#b"`; character 4 sits inside
    // the quoted key, and the `#` in the value must not affect it.
    let definition = harness
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 1, "character": 4 },
            }),
        )
        .await;
    assert_eq!(definition["uri"], json!(schema_uri.as_str()));
    assert_eq!(
        definition["range"], name_range,
        "goto-definition must land on the decoded key, not on its quotes"
    );

    let references = harness
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 1, "character": 4 },
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
        "one schema declaration plus one config usage: {references}"
    );
    for location in locations {
        assert_eq!(
            location["range"], name_range,
            "every occurrence of a quoted key is its inner span: {references}"
        );
    }

    let prepare = harness
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 1, "character": 4 },
            }),
        )
        .await;
    assert_eq!(
        prepare, name_range,
        "prepareRename must report the inner name span"
    );

    // Character 2 is the opening quote, character 10 is inside the `string`
    // type, and character 25 is inside the `"x"` default: none of them may
    // offer a rename, and a direct rename must reject them as well.
    for character in [2, 10, 25] {
        let prepare = harness
            .request(
                "textDocument/prepareRename",
                json!({
                    "textDocument": { "uri": schema_uri },
                    "position": { "line": 1, "character": character },
                }),
            )
            .await;
        assert_eq!(
            prepare,
            Value::Null,
            "character {character} is not part of the name token"
        );

        let rename = harness
            .request(
                "textDocument/rename",
                json!({
                    "textDocument": { "uri": schema_uri },
                    "position": { "line": 1, "character": character },
                    "newName": "title",
                }),
            )
            .await;
        assert_eq!(
            rename,
            Value::Null,
            "a direct rename must reject character {character} too"
        );
    }

    // Renaming the field edits only the inner span on both sides, so the
    // schema's quotes remain in place around the new name.
    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 1, "character": 4 },
                "newName": "title",
            }),
        )
        .await;
    let changes = rename["changes"]
        .as_object()
        .unwrap_or_else(|| panic!("rename must return a workspace edit, got {rename}"));
    assert_eq!(changes.len(), 2, "the declaration and the usage: {rename}");
    assert_eq!(
        changes[schema_uri.as_str()],
        json!([{ "range": name_range, "newText": "title" }]),
        "the schema edit must not eat the quotes: {rename}"
    );
    assert_eq!(
        changes[config_uri.as_str()],
        json!([{ "range": name_range, "newText": "title" }]),
        "the quoted config key is edited at its inner span too: {rename}"
    );

    // Line 0 is `schema "app":`; the quoted root name sits at 8..11 and gets
    // the same treatment as the quoted field name.
    let root_range = json!({
        "start": { "line": 0, "character": 8 },
        "end": { "line": 0, "character": 11 },
    });
    let prepare = harness
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 0, "character": 9 },
            }),
        )
        .await;
    assert_eq!(prepare, root_range);

    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 0, "character": 9 },
                "newName": "main",
            }),
        )
        .await;
    let changes = rename["changes"]
        .as_object()
        .unwrap_or_else(|| panic!("rename must return a workspace edit, got {rename}"));
    assert_eq!(
        changes[schema_uri.as_str()],
        json!([{ "range": root_range, "newText": "main" }]),
        "the quoted root is renamed through its inner span: {rename}"
    );
    assert_eq!(
        changes[config_uri.as_str()],
        json!([{
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 3 },
            },
            "newText": "main",
        }]),
        "the config usage of the root is the `app` key: {rename}"
    );
}

/// A schema declaring the quoted non-BMP key `𐐀name`, which is one char but
/// two UTF-16 code units and four UTF-8 bytes wide.
const SCHEMA_NON_BMP_QUOTED_KEY: &str = "schema app:\n  \"\u{10400}name\" string\nend\n";

/// A config exercising [`SCHEMA_NON_BMP_QUOTED_KEY`] through the same quoted
/// non-BMP key.
const CONFIG_NON_BMP_QUOTED_KEY: &str = "app:\n  \"\u{10400}name\" \"Rune\"\nend\n";

/// Every range of a non-BMP quoted schema key is measured in UTF-16 code
/// units: the key `𐐀name` runs from character 3 to character 9 - two units
/// for the astral character - and never from chars or UTF-8 bytes.
#[tokio::test]
async fn non_bmp_quoted_schema_key_uses_utf16_ranges() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    harness
        .did_open(&schema_uri, SCHEMA_NON_BMP_QUOTED_KEY)
        .await;
    harness
        .did_open(&config_uri, CONFIG_NON_BMP_QUOTED_KEY)
        .await;
    harness.discard_server_messages().await;

    // Line 1 is `  "𐐀name" string`; character 5 sits inside the key, and
    // the decoded name spans UTF-16 characters 3..9.
    let name_range = json!({
        "start": { "line": 1, "character": 3 },
        "end": { "line": 1, "character": 9 },
    });

    let definition = harness
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 1, "character": 5 },
            }),
        )
        .await;
    assert_eq!(definition["uri"], json!(schema_uri.as_str()));
    assert_eq!(
        definition["range"], name_range,
        "the declaration range must count UTF-16 code units"
    );

    let prepare = harness
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 1, "character": 5 },
            }),
        )
        .await;
    assert_eq!(prepare, name_range);

    let references = harness
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 1, "character": 5 },
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
        "one schema declaration plus one config usage: {references}"
    );
    for location in locations {
        assert_eq!(
            location["range"], name_range,
            "every occurrence is the UTF-16 inner span: {references}"
        );
    }

    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 1, "character": 5 },
                "newName": "title",
            }),
        )
        .await;
    let changes = rename["changes"]
        .as_object()
        .unwrap_or_else(|| panic!("rename must return a workspace edit, got {rename}"));
    assert_eq!(changes.len(), 2, "the declaration and the usage: {rename}");
    assert_eq!(
        changes[schema_uri.as_str()],
        json!([{ "range": name_range, "newText": "title" }]),
        "the schema edit must replace exactly `𐐀name` in UTF-16 units: {rename}"
    );
    assert_eq!(
        changes[config_uri.as_str()],
        json!([{ "range": name_range, "newText": "title" }]),
        "the config edit is measured the same way: {rename}"
    );
}

/// A schema that breaks right after a quoted non-BMP key: `=` is no valid
/// schema token.
const SCHEMA_INVALID_AFTER_NON_BMP_KEY: &str = "schema app:\n  \"\u{10400}name\" =\nend\n";

/// Schema parse errors are published at UTF-16 columns. After the quoted
/// non-BMP key on line 1, the invalid `=` sits at character 11 in UTF-16 code
/// units - one more than the char count and two less than the byte count.
#[tokio::test]
async fn schema_error_after_non_bmp_key_publishes_utf16_column() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");

    harness
        .did_open(&schema_uri, SCHEMA_INVALID_AFTER_NON_BMP_KEY)
        .await;

    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "the schema publishes for itself: {published:#?}"
    );
    let schema = diagnostics_for(&published, &schema_uri, Some(1));
    assert_eq!(schema.diagnostics.len(), 1, "{published:#?}");

    let error = diagnostic_with_message(&schema.diagnostics, "Unknown schema type");
    assert_eq!(error["code"], json!(600));
    assert_eq!(
        error["range"],
        json!({
            "start": { "line": 1, "character": 11 },
            "end": { "line": 1, "character": 12 },
        }),
        "the column must count UTF-16 code units after the non-BMP key"
    );
}

/// `schema schema:` with a default that repeats the field name: each line
/// carries the name twice, and only the declaration's own token is the rename
/// target.
const SCHEMA_REPEATED_NAMES: &str = "schema schema:\n  name string default name\nend\n";

/// The stored name span - not the first or last name-looking token on the
/// line - decides what `schema schema:` and a repeated name select.
#[tokio::test]
async fn schema_declarations_select_the_exact_name_token() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");

    harness.did_open(&schema_uri, SCHEMA_REPEATED_NAMES).await;
    harness.discard_server_messages().await;

    // Line 0 is `schema schema:`; the keyword at character 2 is not the root
    // name, which sits at 7..13.
    let on_keyword = harness
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 0, "character": 2 },
            }),
        )
        .await;
    assert_eq!(
        on_keyword,
        Value::Null,
        "the `schema` keyword is not the root name"
    );

    let on_root = harness
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 0, "character": 9 },
            }),
        )
        .await;
    assert_eq!(
        on_root,
        json!({
            "start": { "line": 0, "character": 7 },
            "end": { "line": 0, "character": 13 },
        })
    );

    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 0, "character": 9 },
                "newName": "app",
            }),
        )
        .await;
    assert_eq!(
        rename["changes"][schema_uri.as_str()],
        json!([{
            "range": {
                "start": { "line": 0, "character": 7 },
                "end": { "line": 0, "character": 13 },
            },
            "newText": "app",
        }]),
        "renaming the root must edit the name token, not the keyword: {rename}"
    );

    // Line 1 is `  name string default name`; the trailing `name` repeats the
    // field name as its default and is not the declaration.
    let on_default = harness
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 1, "character": 24 },
            }),
        )
        .await;
    assert_eq!(
        on_default,
        Value::Null,
        "the repeated name is the default, not the declaration"
    );

    let on_field = harness
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": schema_uri },
                "position": { "line": 1, "character": 3 },
            }),
        )
        .await;
    assert_eq!(
        on_field,
        json!({
            "start": { "line": 1, "character": 2 },
            "end": { "line": 1, "character": 6 },
        })
    );
}

/// Two unrelated configs, with no schema anywhere: opening or changing one
/// publishes for that document alone, and closing it clears its diagnostics
/// with a `null` version.
///
/// Regression guard: the server used to revalidate every open document on
/// every open, change, and close, so each of these steps published for both
/// configs.
#[tokio::test]
async fn independent_configs_publish_only_the_document_that_changed() {
    let mut harness = LspHarness::start().await;
    let first = harness.document_uri("first.rune");
    let second = harness.document_uri("second.rune");

    // No schema exists, so each config has nothing to validate against.
    harness.did_open(&first, CONFIG_STRING_VALUE).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "one open publishes one document: {published:#?}"
    );
    assert_eq!(published[0].uri, first);
    assert_eq!(published[0].version, Some(1));
    assert!(
        published[0].diagnostics.is_empty(),
        "a config with no schema is clean: {published:#?}"
    );

    harness.did_open(&second, CONFIG_STRING_VALUE).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "the first config is not revalidated by the second open: {published:#?}"
    );
    assert_eq!(published[0].uri, second);
    assert_eq!(published[0].version, Some(1));
    assert!(published[0].diagnostics.is_empty(), "{published:#?}");

    harness
        .replace_document(&first, 2, CONFIG_STRING_VALUE)
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "only the changed buffer is republished: {published:#?}"
    );
    assert_eq!(published[0].uri, first);
    assert_eq!(published[0].version, Some(2));
    assert!(published[0].diagnostics.is_empty(), "{published:#?}");

    harness.did_close(&first).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "closing publishes only the closed document: {published:#?}"
    );
    assert_eq!(published[0].uri, first);
    assert_eq!(
        published[0].version, None,
        "a closed document is cleared without a version: {published:#?}"
    );
    assert!(published[0].diagnostics.is_empty(), "{published:#?}");
}

/// An open `schema.rune` overrides the file of the same name on disk while it
/// is open, and closing it hands the config back to the disk contents.
///
/// Regression guard: closing a document used to revalidate every other open
/// document and left the closed buffer's own dependency state behind.
#[tokio::test]
async fn closing_an_open_schema_falls_back_to_the_schema_on_disk() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    // The file on disk is the string schema; the buffer opens with the int one.
    harness.write_file("schema.rune", SCHEMA_STRING_FIELD);

    harness.did_open(&schema_uri, SCHEMA_INT_FIELD).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "the schema is the only open document: {published:#?}"
    );
    assert_eq!(published[0].uri, schema_uri);
    assert!(
        published[0].diagnostics.is_empty(),
        "the int schema parses cleanly: {published:#?}"
    );

    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "only the newly opened config is published: {published:#?}"
    );
    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert_eq!(
        config.diagnostics.len(),
        1,
        "the open int schema rejects the string value: {published:#?}"
    );
    assert!(
        config.diagnostics[0]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("expected int, got string"),
        "unexpected diagnostic: {published:#?}"
    );

    harness.did_close(&schema_uri).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        2,
        "the closed schema is cleared and the config is revalidated: {published:#?}"
    );

    let schema = diagnostics_for(&published, &schema_uri, None);
    assert!(
        schema.diagnostics.is_empty(),
        "a closed document is cleared: {published:#?}"
    );

    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert!(
        config.diagnostics.is_empty(),
        "the config now validates against the string schema on disk: {published:#?}"
    );
}

/// A watched change to the file a config resolved to republishes that config,
/// and nothing else: a config bound to a different schema is untouched, and an
/// unrelated `.rune` event publishes nothing at all.
#[tokio::test]
async fn watched_schema_change_republishes_only_bound_configs() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");
    let isolated_uri = harness.document_uri("isolated/config.rune");

    harness.write_file("schema.rune", SCHEMA_STRING_FIELD);
    harness.write_file("isolated/other.rune", SCHEMA_STRING_FIELD);

    // The root config discovers `schema.rune`; the isolated one is bound to
    // `other.rune` by an explicit directive and never sees the root schema.
    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert!(
        diagnostics_for(&published, &config_uri, Some(1))
            .diagnostics
            .is_empty(),
        "the config matches the string schema on disk: {published:#?}"
    );

    harness
        .did_open(&isolated_uri, CONFIG_BOUND_TO_OTHER_SCHEMA)
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert!(
        diagnostics_for(&published, &isolated_uri, Some(1))
            .diagnostics
            .is_empty(),
        "the isolated config matches its own schema: {published:#?}"
    );

    // The root schema becomes an int schema on disk.
    harness.write_file("schema.rune", SCHEMA_INT_FIELD);
    harness
        .did_change_watched_files(&[(schema_uri.clone(), 2)])
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "exactly the config bound to the changed schema is republished: {published:#?}"
    );
    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert_eq!(config.diagnostics.len(), 1, "{published:#?}");
    assert!(
        config.diagnostics[0]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("expected int, got string"),
        "unexpected diagnostic: {published:#?}"
    );
    assert!(
        !published
            .iter()
            .any(|notification| notification.uri == isolated_uri),
        "the isolated config is not republished: {published:#?}"
    );

    // An unrelated `.rune` file is not a schema for anybody.
    harness.write_file("unrelated.rune", CONFIG_STRING_VALUE);
    harness
        .did_change_watched_files(&[(harness.file_uri("unrelated.rune"), 1)])
        .await;
    let published = harness.collect_diagnostics().await;
    assert!(
        published.is_empty(),
        "an unrelated .rune event publishes nothing: {published:#?}"
    );
}

/// A file the editor has open is the editor's buffer: a watched event for it
/// is ignored, and the disk copy never replaces the buffer.
#[tokio::test]
async fn an_open_document_ignores_watched_events_for_its_file() {
    let mut harness = LspHarness::start().await;
    let config_uri = harness.document_uri("config.rune");

    harness.write_file("schema.rune", SCHEMA_STRING_FIELD);
    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert!(
        published[0].diagnostics.is_empty(),
        "the config matches the schema it discovered: {published:#?}"
    );

    // The disk copy of the open config becomes something else entirely.
    harness.write_file("config.rune", SCHEMA_INT_FIELD);
    harness
        .did_change_watched_files(&[(config_uri.clone(), 2)])
        .await;
    let published = harness.collect_diagnostics().await;
    assert!(
        published.is_empty(),
        "a watched event for an open document publishes nothing: {published:#?}"
    );

    // The buffer still wins: its symbols are the config's, not the schema's.
    let symbols = harness
        .request(
            "textDocument/documentSymbol",
            json!({ "textDocument": { "uri": config_uri } }),
        )
        .await;
    assert_eq!(
        symbol_names(&symbols),
        vec!["app", "app.name"],
        "the open buffer is still the config it was opened with: {symbols}"
    );
}

/// Config text that does not parse neither loses nor reports its schema
/// dependency: the dependency comes from the indexed `@schema` directive, and
/// the parse diagnostics stay exactly as they were.
#[tokio::test]
async fn invalid_config_text_keeps_its_schema_dependency() {
    let mut harness = LspHarness::start().await;
    let config_uri = harness.document_uri("config.rune");
    let custom_uri = harness.file_uri("custom.rune");

    // An unclosed `app` block, so this can never be validated against a schema.
    harness
        .did_open(
            &config_uri,
            "@schema \"./custom.rune\"\napp:\n  name \"Rune\"\n",
        )
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "opening publishes the config: {published:#?}"
    );
    let before = diagnostics_for(&published, &config_uri, Some(1))
        .diagnostics
        .clone();
    assert!(
        !before.is_empty(),
        "unparsable text is reported: {published:#?}"
    );

    // Creating the directive's target rebinds the config, which is only
    // possible because the invalid buffer still has a dependency.
    harness.write_file("custom.rune", SCHEMA_INT_FIELD);
    harness.did_change_watched_files(&[(custom_uri, 1)]).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "the config is rebound to the created schema: {published:#?}"
    );
    assert_eq!(published[0].uri, config_uri);
    assert_eq!(published[0].version, Some(1));
    assert_eq!(
        published[0].diagnostics, before,
        "the parse diagnostics stay exactly as they were: {published:#?}"
    );
}

/// A client that supports workspace folders but sends none has a workspace
/// without folders, so `rootUri` is not used as a discovery boundary.
#[tokio::test]
async fn initialize_without_folders_ignores_root_uri() {
    let mut harness = LspHarness::start_with_initializer(|root| {
        json!({
            "processId": Value::Null,
            "rootUri": Url::from_directory_path(root.join("child")).expect("root uri"),
            "capabilities": { "workspace": { "workspaceFolders": true } },
        })
    })
    .await;

    let config_uri = harness.file_uri("child/config.rune");

    // The schema sits above `rootUri`: an unbounded walk discovers it, while a
    // boundary at `rootUri` would hide it and leave the config unvalidated.
    harness.write_file("schema.rune", SCHEMA_INT_FIELD);
    harness.write_file("child/config.rune", CONFIG_STRING_VALUE);

    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert_eq!(published[0].uri, config_uri);
    assert_eq!(
        published[0].diagnostics.len(),
        1,
        "rootUri must not stand in for a workspace folder: {published:#?}"
    );
}

/// Discovery follows the nearest `schema.rune` once it appears or disappears on
/// disk, rebinding the open config without touching any other document.
#[tokio::test]
async fn watched_schema_discovery_rebinds_configs_at_the_nearest_candidate() {
    let mut harness = LspHarness::start().await;
    let root_schema = harness.file_uri("schema.rune");
    let nested_schema = harness.file_uri("nested/schema.rune");
    let config_uri = harness.file_uri("nested/config.rune");

    harness.write_file("schema.rune", SCHEMA_INT_FIELD);
    harness.write_file("nested/config.rune", CONFIG_STRING_VALUE);

    // With no nested schema, the walk reaches the parent int schema.
    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "opening the config publishes the config: {published:#?}"
    );
    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert_eq!(
        config.diagnostics.len(),
        1,
        "the parent int schema rejects the string value: {published:#?}"
    );

    // A nearer string schema appears.
    harness.write_file("nested/schema.rune", SCHEMA_STRING_FIELD);
    harness
        .did_change_watched_files(&[(nested_schema.clone(), 1)])
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "only the rebinding config is republished: {published:#?}"
    );
    assert_eq!(published[0].uri, config_uri);
    assert_eq!(published[0].version, Some(1));
    assert!(
        published[0].diagnostics.is_empty(),
        "the nearest string schema matches the config: {published:#?}"
    );

    // It disappears again, so the parent int schema takes over.
    harness.remove_file("nested/schema.rune");
    harness
        .did_change_watched_files(&[(nested_schema.clone(), 3)])
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert_eq!(published[0].uri, config_uri);
    assert_eq!(
        published[0].diagnostics.len(),
        1,
        "the parent int schema applies again: {published:#?}"
    );

    // With every candidate gone, the config has no schema to validate against.
    harness.remove_file("schema.rune");
    harness
        .did_change_watched_files(&[(root_schema.clone(), 3)])
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert_eq!(published[0].uri, config_uri);
    assert!(
        published[0].diagnostics.is_empty(),
        "a config with no schema is clean: {published:#?}"
    );
}

/// An `@schema` directive keeps a config bound to its target as that file is
/// created, changed, and deleted on disk, and reports the missing reference
/// again once it is gone.
#[tokio::test]
async fn watched_schema_directive_target_rebinds_and_clears_diagnostics() {
    let mut harness = LspHarness::start().await;
    let config_uri = harness.document_uri("config.rune");
    let custom_uri = harness.file_uri("custom.rune");

    // The referenced file does not exist yet, so the directive itself is the
    // only diagnostic the config has.
    harness
        .did_open(&config_uri, CONFIG_WITH_CUSTOM_SCHEMA)
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert_eq!(
        config.diagnostics.len(),
        1,
        "a missing @schema target is one diagnostic: {published:#?}"
    );
    assert_eq!(config.diagnostics[0]["code"], json!(701));
    assert!(
        config.diagnostics[0]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("was not found"),
        "unexpected diagnostic: {published:#?}"
    );

    // Creating the target binds the config to it.
    harness.write_file("custom.rune", SCHEMA_STRING_FIELD);
    harness
        .did_change_watched_files(&[(custom_uri.clone(), 1)])
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "only the bound config is republished: {published:#?}"
    );
    assert_eq!(published[0].uri, config_uri);
    assert!(
        published[0].diagnostics.is_empty(),
        "the created string schema matches the config: {published:#?}"
    );

    // Widening the target to int rejects the config's string value.
    harness.write_file("custom.rune", SCHEMA_INT_FIELD);
    harness
        .did_change_watched_files(&[(custom_uri.clone(), 2)])
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert_eq!(published[0].uri, config_uri);
    assert_eq!(
        published[0].diagnostics.len(),
        1,
        "the int schema rejects the string value: {published:#?}"
    );

    // Deleting it brings the missing reference back.
    harness.remove_file("custom.rune");
    harness.did_change_watched_files(&[(custom_uri, 3)]).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert_eq!(published[0].uri, config_uri);
    assert_eq!(
        published[0].diagnostics.len(),
        1,
        "the missing @schema target is reported again: {published:#?}"
    );
    assert_eq!(published[0].diagnostics[0]["code"], json!(701));
}

/// A client that offers workspace folders gets the capability advertised, and
/// the folder bounds discovery: a schema above it is never used, so deleting
/// the local schema leaves the config with no schema at all.
#[tokio::test]
async fn workspace_folder_root_bounds_schema_discovery() {
    let mut harness = LspHarness::start_with_initializer(|root| {
        json!({
            "processId": Value::Null,
            "rootUri": Value::Null,
            "workspaceFolders": [{
                "uri": Url::from_directory_path(root.join("workspace"))
                    .expect("workspace folder uri"),
                "name": "workspace",
            }],
            "capabilities": { "workspace": { "workspaceFolders": true } },
        })
    })
    .await;

    let capabilities =
        &harness.initialize_result()["capabilities"]["workspace"]["workspaceFolders"];
    assert_eq!(
        capabilities["supported"],
        json!(true),
        "the server must advertise workspace folder support: {capabilities}"
    );
    assert_eq!(
        capabilities["changeNotifications"],
        json!(true),
        "the server must ask for folder change notifications: {capabilities}"
    );

    let local_schema_uri = harness.file_uri("workspace/schema.rune");
    let config_uri = harness.file_uri("workspace/config.rune");

    // One int schema inside the folder and one above it, plus a config whose
    // string value only the local schema can reject.
    harness.write_file("workspace/schema.rune", SCHEMA_INT_FIELD);
    harness.write_file("workspace/config.rune", CONFIG_STRING_VALUE);
    harness.write_file("schema.rune", SCHEMA_INT_FIELD);

    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert_eq!(
        config.diagnostics.len(),
        1,
        "the workspace-local int schema rejects the string value: {published:#?}"
    );

    let definition = harness
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 1, "character": 3 },
            }),
        )
        .await;
    assert_eq!(
        definition["uri"],
        json!(local_schema_uri.as_str()),
        "the schema inside the workspace folder is the one that resolves: {definition}"
    );

    // Deleting the local schema leaves the config with nothing: the schema
    // above the workspace folder is out of discovery range.
    harness.remove_file("workspace/schema.rune");
    harness
        .did_change_watched_files(&[(local_schema_uri, 3)])
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "only the config is republished: {published:#?}"
    );
    assert_eq!(published[0].uri, config_uri);
    assert!(
        published[0].diagnostics.is_empty(),
        "discovery must not fall through to a schema above the folder: {published:#?}"
    );
}

/// Removing the only workspace folder makes schema discovery unbounded, and
/// adding it back restores the boundary.
#[tokio::test]
async fn removing_a_workspace_folder_extends_schema_discovery() {
    let mut harness = LspHarness::start_with_initializer(|root| {
        json!({
            "processId": Value::Null,
            "rootUri": Value::Null,
            "workspaceFolders": [{
                "uri": Url::from_directory_path(root.join("child"))
                    .expect("child folder uri"),
                "name": "child",
            }],
            "capabilities": { "workspace": { "workspaceFolders": true } },
        })
    })
    .await;

    let child_folder = harness.directory_uri("child");
    let config_uri = harness.file_uri("child/config.rune");

    // An int schema above the child folder, which the boundary hides.
    harness.write_file("schema.rune", SCHEMA_INT_FIELD);
    harness.write_file("child/config.rune", CONFIG_STRING_VALUE);

    harness.did_open(&config_uri, CONFIG_STRING_VALUE).await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert_eq!(published[0].uri, config_uri);
    assert!(
        published[0].diagnostics.is_empty(),
        "the workspace folder hides the schema above it: {published:#?}"
    );

    // With no folder left, the walk is unbounded and finds the parent schema.
    harness
        .did_change_workspace_folders(&[], std::slice::from_ref(&child_folder))
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "only the config whose schema changed is republished: {published:#?}"
    );
    assert_eq!(published[0].uri, config_uri);
    assert_eq!(
        published[0].diagnostics.len(),
        1,
        "the parent int schema rejects the string value: {published:#?}"
    );

    // Adding the folder back hides the schema again.
    harness
        .did_change_workspace_folders(std::slice::from_ref(&child_folder), &[])
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(published.len(), 1, "{published:#?}");
    assert_eq!(published[0].uri, config_uri);
    assert!(
        published[0].diagnostics.is_empty(),
        "the workspace folder bounds discovery again: {published:#?}"
    );
}

/// A schema-scoped rename or reference query reaches configs in every workspace
/// folder that is bound to the same schema.
#[tokio::test]
async fn schema_scoped_navigation_spans_every_workspace_folder() {
    // One schema, bound from the first folder by a relative path and from the
    // second by its absolute path, so both configs resolve to the same schema
    // URI and both folders have to be scanned for the answer to be complete.
    // The fixtures exist before `initialize`, because the workspace is indexed
    // once, at startup.
    let mut harness = LspHarness::start_with_initializer(|root| {
        let shared_schema = root.join("one/shared.rune");
        write_workspace_fixture(root, "one/shared.rune", SCHEMA_STRING_FIELD);
        write_workspace_fixture(root, "one/config.rune", CONFIG_BOUND_TO_SHARED_SCHEMA);
        write_workspace_fixture(
            root,
            "two/config.rune",
            &format!(
                "@schema \"{}\"\napp:\n  name \"Rune\"\nend\n",
                shared_schema.display()
            ),
        );

        json!({
            "processId": Value::Null,
            "rootUri": Value::Null,
            "workspaceFolders": [
                {
                    "uri": Url::from_directory_path(root.join("one")).expect("one folder uri"),
                    "name": "one",
                },
                {
                    "uri": Url::from_directory_path(root.join("two")).expect("two folder uri"),
                    "name": "two",
                },
            ],
            "capabilities": { "workspace": { "workspaceFolders": true } },
        })
    })
    .await;

    let schema_uri = harness.file_uri("one/shared.rune");
    let first = harness.file_uri("one/config.rune");
    let second = harness.file_uri("two/config.rune");

    let references = harness
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": first },
                "position": { "line": 2, "character": 3 },
                "context": { "includeDeclaration": true },
            }),
        )
        .await;
    let locations = references
        .as_array()
        .unwrap_or_else(|| panic!("references must return locations, got {references}"));
    assert_eq!(
        locations.len(),
        3,
        "the declaration and the usage in each folder: {references}"
    );

    let uris: Vec<&str> = locations
        .iter()
        .filter_map(|location| location["uri"].as_str())
        .collect();
    assert!(
        uris.contains(&schema_uri.as_str()),
        "the declaration in the shared schema: {references}"
    );
    assert!(
        uris.contains(&first.as_str()),
        "the usage in the first folder: {references}"
    );
    assert!(
        uris.contains(&second.as_str()),
        "the usage in the second folder: {references}"
    );

    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": first },
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
        3,
        "the schema declaration and the config in each folder are renamed: {rename}"
    );
}

/// Completion item labels at one position, so a test can describe which kind of
/// items a context has to offer.
async fn completion_labels(
    harness: &mut LspHarness,
    uri: &Url,
    line: u32,
    character: u32,
) -> Vec<String> {
    let result = harness
        .request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
            }),
        )
        .await;

    result
        .as_array()
        .unwrap_or_else(|| panic!("completion must return an item array, got {result}"))
        .iter()
        .filter_map(|item| {
            item.get("label")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect()
}

/// The range of a two-space-indented, four-character key on `line`.
fn indented_key_range(line: u32) -> Value {
    json!({
        "start": { "line": line, "character": 2 },
        "end": { "line": line, "character": 6 },
    })
}

/// A config whose `@schema` line carries the keyword, the quoted reference,
/// and a trailing comment after it.
const CONFIG_WITH_DIRECTIVE_COMMENT: &str = r#"@schema "./schema.rune" # the app schema
app:
  name "Rune"
end
"#;

/// A config whose `@schema` reference does not exist, and whose reference is
/// non-ASCII: the `ä` of `schemä` is two UTF-8 bytes but one UTF-16 code unit.
const CONFIG_WITH_MISSING_UNICODE_SCHEMA: &str = r#"@schema "schemä"
app:
  name "Rune"
end
"#;

/// Definition follows the schema link only from the quoted reference itself:
/// the `@schema` keyword and a trailing comment on that line are not links.
#[tokio::test]
async fn definition_follows_only_the_quoted_schema_reference() {
    let mut harness = LspHarness::start().await;
    let config_uri = harness.document_uri("config.rune");
    let schema_uri = harness.document_uri("schema.rune");

    harness.write_file("schema.rune", SCHEMA_STRING_FIELD);
    harness
        .did_open(&config_uri, CONFIG_WITH_DIRECTIVE_COMMENT)
        .await;
    harness.discard_server_messages().await;

    // Line 0 is `@schema "./schema.rune" # the app schema`: the quoted token
    // occupies characters 8..24, so character 12 is inside the reference.
    let definition = harness
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 0, "character": 12 },
            }),
        )
        .await;
    assert_eq!(definition["uri"], json!(schema_uri.as_str()));
    assert_eq!(
        definition["range"],
        json!({
            "start": { "line": 0, "character": 0 },
            "end": { "line": 0, "character": 0 },
        }),
        "a cursor on the quoted reference jumps to the top of the schema: {definition}"
    );

    // Character 3 is inside the `@schema` keyword and character 30 is inside
    // the trailing comment: both are on the directive's line, neither is the
    // reference.
    for character in [3, 30] {
        let definition = harness
            .request(
                "textDocument/definition",
                json!({
                    "textDocument": { "uri": config_uri },
                    "position": { "line": 0, "character": character },
                }),
            )
            .await;
        assert_eq!(
            definition,
            Value::Null,
            "character {character} of the directive line is not the reference"
        );
    }
}

/// A missing `@schema` reference is reported on the quoted token itself,
/// measured in UTF-16 code units rather than UTF-8 bytes.
#[tokio::test]
async fn missing_unicode_schema_reference_reports_the_quoted_token_range() {
    let mut harness = LspHarness::start().await;
    let config_uri = harness.document_uri("config.rune");

    harness
        .did_open(&config_uri, CONFIG_WITH_MISSING_UNICODE_SCHEMA)
        .await;
    let published = harness.collect_diagnostics().await;
    assert_eq!(
        published.len(),
        1,
        "opening publishes only the config: {published:#?}"
    );

    let config = diagnostics_for(&published, &config_uri, Some(1));
    assert_eq!(config.diagnostics.len(), 1, "{published:#?}");

    let missing = diagnostic_with_message(&config.diagnostics, "was not found");
    assert_eq!(missing["code"], json!(701));
    assert_eq!(
        missing["range"],
        json!({
            "start": { "line": 0, "character": 8 },
            "end": { "line": 0, "character": 16 },
        }),
        "the range is the quoted token in UTF-16 code units, not in bytes: {missing}"
    );
}

/// A config that writes the same key twice inside one object.
const CONFIG_TWO_USAGES: &str = r#"app:
  name "Rune"
  name "Other"
end
"#;

/// Without a schema every indexed occurrence is a usage, so a request that
/// excludes declarations still reports the occurrence on the cursor's line.
#[tokio::test]
async fn references_without_declaration_still_report_the_cursor_usage() {
    let mut harness = LspHarness::start().await;
    let uri = harness.document_uri("config.rune");

    harness.did_open(&uri, CONFIG_TWO_USAGES).await;
    harness.discard_server_messages().await;

    let references = harness
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 1, "character": 3 },
                "context": { "includeDeclaration": false },
            }),
        )
        .await;
    let locations = references
        .as_array()
        .unwrap_or_else(|| panic!("references must return locations, got {references}"));

    assert_eq!(
        locations.len(),
        2,
        "both occurrences are usages, the cursor's own included: {references}"
    );
    let ranges: Vec<Value> = locations
        .iter()
        .map(|location| {
            assert_eq!(location["uri"], json!(uri.as_str()));
            location["range"].clone()
        })
        .collect();
    assert_eq!(
        ranges,
        vec![indented_key_range(1), indented_key_range(2)],
        "the filter that dropped the cursor's line is gone: {references}"
    );
}

/// A config with a nested `app.server.port` assignment.
const CONFIG_APP_SERVER_PORT: &str = r#"app:
  server:
    port 8080
  end
end
"#;

/// A schema-scoped rename for a client that supports versioned document
/// changes answers with `documentChanges` alone: one entry per edited
/// document, sorted by URI, carrying the open buffer's version or `null` for a
/// file that only exists on disk.
#[tokio::test]
async fn rename_with_document_changes_reports_versions_and_sorted_documents() {
    // The unopened disk config exists before `initialize`, so it is a
    // workspace member from the start: only a file the scan saw, or a watcher
    // event named, is ever a rename target.
    let mut harness = LspHarness::start_with_initializer(|root| {
        write_workspace_fixture(root, "disk/config.rune", CONFIG_STRING_VALUE);

        json!({
            "processId": Value::Null,
            "rootUri": Url::from_directory_path(root).expect("workspace root uri"),
            "capabilities": { "workspace": { "workspaceEdit": { "documentChanges": true } } },
        })
    })
    .await;

    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");
    let disk_uri = harness.file_uri("disk/config.rune");

    harness.did_open(&schema_uri, SCHEMA_STRING_FIELD).await;
    harness
        .replace_document(&schema_uri, 3, SCHEMA_STRING_FIELD)
        .await;
    // The open config writes the key twice, so its own edits must stay in
    // source order.
    harness.did_open(&config_uri, CONFIG_TWO_USAGES).await;
    harness.discard_server_messages().await;

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

    assert!(
        rename.get("changes").is_none(),
        "a versioned rename must not also carry a changes map: {rename}"
    );

    let document_changes = rename["documentChanges"]
        .as_array()
        .unwrap_or_else(|| panic!("rename must return documentChanges, got {rename}"));
    let expected: Vec<Value> = [
        (
            config_uri.as_str(),
            json!(1),
            vec![indented_key_range(1), indented_key_range(2)],
        ),
        (disk_uri.as_str(), Value::Null, vec![indented_key_range(1)]),
        (schema_uri.as_str(), json!(3), vec![indented_key_range(1)]),
    ]
    .into_iter()
    .map(|(uri, version, ranges)| {
        json!({
            "textDocument": { "uri": uri, "version": version },
            "edits": ranges
                .into_iter()
                .map(|range| json!({ "range": range, "newText": "title" }))
                .collect::<Vec<Value>>(),
        })
    })
    .collect();
    assert_eq!(
        document_changes, &expected,
        "documents are sorted by URI and carry their version, or null on disk: {rename}"
    );

    // The same rename from a client without the capability still answers with
    // the `changes` map and no `documentChanges`.
    let mut plain = LspHarness::start().await;
    let plain_schema_uri = plain.document_uri("schema.rune");
    let plain_config_uri = plain.document_uri("config.rune");
    plain.did_open(&plain_schema_uri, SCHEMA_STRING_FIELD).await;
    plain.did_open(&plain_config_uri, CONFIG_STRING_VALUE).await;
    plain.discard_server_messages().await;

    let rename = plain
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": plain_config_uri },
                "position": { "line": 1, "character": 3 },
                "newName": "title",
            }),
        )
        .await;

    assert!(
        rename.get("documentChanges").is_none(),
        "a plain client must not receive documentChanges: {rename}"
    );
    let changes = rename["changes"]
        .as_object()
        .unwrap_or_else(|| panic!("a plain rename must return changes, got {rename}"));
    assert_eq!(changes.len(), 2, "the schema and the config: {rename}");
    assert_eq!(
        changes[plain_config_uri.as_str()],
        json!([{ "range": indented_key_range(1), "newText": "title" }]),
        "the usage is replaced in place: {rename}"
    );
}

/// A config where `server.port` already has a `server.host` sibling.
const CONFIG_WITH_SIBLING_HOST: &str = r#"server:
  port 8080
  host "localhost"
end
"#;

/// A config whose `host` sibling lives under a different parent.
const CONFIG_WITH_HOST_ELSEWHERE: &str = r#"server:
  port 8080
app:
  host "localhost"
end
"#;

/// A rename is rejected as JSON-RPC invalid params when the candidate name is
/// already taken by a sibling in the same document, and accepted when it is
/// not.
#[tokio::test]
async fn rename_rejects_a_sibling_that_already_exists_in_the_document() {
    let mut harness = LspHarness::start().await;
    let uri = harness.document_uri("config.rune");

    harness.did_open(&uri, CONFIG_WITH_SIBLING_HOST).await;
    harness.discard_server_messages().await;

    let error = harness
        .request_or_error(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 1, "character": 4 },
                "newName": "host",
            }),
        )
        .await
        .expect_err("an existing sibling must reject the whole request");

    assert_eq!(error["code"], json!(-32602), "{error}");
    assert_eq!(
        error["message"],
        json!("Cannot rename 'server.port' to 'host': sibling 'server.host' already exists"),
        "{error}"
    );

    // The same leaf under a different parent is not a sibling.
    let other_uri = harness.document_uri("other.rune");
    harness
        .did_open(&other_uri, CONFIG_WITH_HOST_ELSEWHERE)
        .await;
    harness.discard_server_messages().await;

    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": other_uri },
                "position": { "line": 1, "character": 4 },
                "newName": "host",
            }),
        )
        .await;
    assert_eq!(
        rename["changes"][other_uri.as_str()],
        json!([{ "range": indented_key_range(1), "newText": "host" }]),
        "`app.host` is not the sibling of `server.port`: {rename}"
    );

    // Renaming to the leaf the field already has cannot collide.
    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 1, "character": 4 },
                "newName": "port",
            }),
        )
        .await;
    assert_eq!(
        rename["changes"][uri.as_str()],
        json!([{ "range": indented_key_range(1), "newText": "port" }]),
        "renaming to the current leaf is not a collision: {rename}"
    );
}

/// A schema declaring `app.server.port`.
const SCHEMA_APP_SERVER_PORT: &str = r#"schema app:
  server:
    port int
  end
end
"#;

/// The same schema with the `app.server.host` sibling a rename could create.
const SCHEMA_APP_SERVER_PORT_AND_HOST: &str = r#"schema app:
  server:
    port int
    host string
  end
end
"#;

/// A config carrying the `host` sibling that only `app.server.port` renames
/// would collide with.
const CONFIG_APP_SERVER_PORT_AND_HOST: &str = r#"app:
  server:
    port 8080
    host "localhost"
  end
end
"#;

/// A schema-scoped rename is rejected when the schema itself declares the
/// sibling the new name would take.
#[tokio::test]
async fn rename_rejects_a_sibling_declared_by_the_schema() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    harness
        .did_open(&schema_uri, SCHEMA_APP_SERVER_PORT_AND_HOST)
        .await;
    harness.did_open(&config_uri, CONFIG_APP_SERVER_PORT).await;
    harness.discard_server_messages().await;

    // Line 2 of the config is `    port 8080`, so character 5 is on the key.
    let error = harness
        .request_or_error(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 2, "character": 5 },
                "newName": "host",
            }),
        )
        .await
        .expect_err("the schema declares app.server.host, so the rename must be rejected");

    assert_eq!(error["code"], json!(-32602), "{error}");
    assert_eq!(
        error["message"],
        json!(
            "Cannot rename 'app.server.port' to 'host': sibling 'app.server.host' already exists"
        ),
        "{error}"
    );
}

/// A schema declaring only the `app.server.host` sibling: the rename target
/// itself is undeclared, but the schema already owns the name the rename would
/// take.
const SCHEMA_APP_SERVER_HOST_ONLY: &str = r#"schema app:
  server:
    host string
  end
end
"#;

/// A schema-scoped rename is rejected when the schema declares the sibling
/// even though it does not declare the key being renamed, so the schema itself
/// would not be edited.
#[tokio::test]
async fn rename_rejects_a_sibling_the_schema_declares_without_the_renamed_key() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");

    harness
        .did_open(&schema_uri, SCHEMA_APP_SERVER_HOST_ONLY)
        .await;
    harness.did_open(&config_uri, CONFIG_APP_SERVER_PORT).await;
    harness.discard_server_messages().await;

    let error = harness
        .request_or_error(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 2, "character": 5 },
                "newName": "host",
            }),
        )
        .await
        .expect_err("the schema declares app.server.host, so the rename must be rejected");

    assert_eq!(error["code"], json!(-32602), "{error}");
    assert_eq!(
        error["message"],
        json!(
            "Cannot rename 'app.server.port' to 'host': sibling 'app.server.host' already exists"
        ),
        "{error}"
    );
}

/// A schema-scoped rename is rejected when a bound config that would be edited
/// already has the sibling, even though the schema itself does not declare it.
#[tokio::test]
async fn rename_rejects_a_sibling_in_a_bound_config() {
    let mut harness = LspHarness::start().await;
    let schema_uri = harness.document_uri("schema.rune");
    let config_uri = harness.document_uri("config.rune");
    let other_uri = harness.document_uri("other.rune");

    harness.did_open(&schema_uri, SCHEMA_APP_SERVER_PORT).await;
    harness.did_open(&config_uri, CONFIG_APP_SERVER_PORT).await;
    harness
        .did_open(&other_uri, CONFIG_APP_SERVER_PORT_AND_HOST)
        .await;
    harness.discard_server_messages().await;

    let error = harness
        .request_or_error(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": config_uri },
                "position": { "line": 2, "character": 5 },
                "newName": "host",
            }),
        )
        .await
        .expect_err("a bound config owns the sibling, so the rename must be rejected");

    assert_eq!(error["code"], json!(-32602), "{error}");
    assert_eq!(
        error["message"],
        json!(
            "Cannot rename 'app.server.port' to 'host': sibling 'app.server.host' already exists"
        ),
        "{error}"
    );
}

/// A new name is validated by lexing it: `näme` is an identifier, while an
/// empty name, a leading digit, a leading `_`, an embedded space, and a lexer
/// keyword are not.
#[tokio::test]
async fn rename_accepts_a_unicode_identifier_and_rejects_invalid_names() {
    let mut harness = LspHarness::start().await;
    let uri = harness.document_uri("config.rune");

    harness.did_open(&uri, CONFIG_STRING_VALUE).await;
    harness.discard_server_messages().await;

    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 1, "character": 3 },
                "newName": "näme",
            }),
        )
        .await;
    assert_eq!(
        rename["changes"][uri.as_str()],
        json!([{ "range": indented_key_range(1), "newText": "näme" }]),
        "a Unicode identifier is a valid new name: {rename}"
    );

    for invalid in ["", "1name", "_name", "na me", "if"] {
        let error = harness
            .request_or_error(
                "textDocument/rename",
                json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": 1, "character": 3 },
                    "newName": invalid,
                }),
            )
            .await
            .unwrap_err();

        assert_eq!(error["code"], json!(-32602), "newName {invalid:?}: {error}");
        assert_eq!(
            error["message"],
            json!("New name must be a valid RUNE identifier"),
            "newName {invalid:?}: {error}"
        );
    }
}

/// A config with a leading `@schema` line, a later key position, and a string
/// value holding a `$`.
const CONFIG_AFTER_SCHEMA_DIRECTIVE: &str = r#"@schema "./schema.rune"
app:
  name "$env.PATH"
  
end
"#;

/// Completion offers `$...` reference items only inside a reference run: a `$`
/// inside a string literal is not a run, and a finished run on another line is
/// not the cursor's run.
#[tokio::test]
async fn completion_offers_dollar_items_only_inside_a_reference_run() {
    let mut harness = LspHarness::start().await;
    let uri = harness.document_uri("config.rune");

    // Line 1 is `  host $env`, whose reference run is characters 7..11.
    harness.did_open(&uri, "app:\n  host $env\nend\n").await;
    harness.discard_server_messages().await;

    let labels = completion_labels(&mut harness, &uri, 1, 9).await;
    assert!(
        labels.contains(&"$env.".to_string()),
        "a cursor inside `$env` offers reference items: {labels:?}"
    );
    assert!(
        !labels.contains(&"end".to_string()),
        "reference items replace the normal completion: {labels:?}"
    );

    // The `$` of a string literal never becomes a reference token.
    harness
        .replace_document(&uri, 2, "app:\n  host \"$env.PATH\"\n  \nend\n")
        .await;
    harness.discard_server_messages().await;

    let labels = completion_labels(&mut harness, &uri, 2, 2).await;
    assert!(
        labels.contains(&"end".to_string()),
        "a `$` inside an earlier string is not a reference context: {labels:?}"
    );
    assert!(!labels.contains(&"$env.".to_string()), "{labels:?}");

    // A finished reference elsewhere on the buffer is a different run.
    harness
        .replace_document(&uri, 3, "app:\n  host $sys.hostname\n  \nend\n")
        .await;
    harness.discard_server_messages().await;

    let labels = completion_labels(&mut harness, &uri, 2, 2).await;
    assert!(
        labels.contains(&"end".to_string()),
        "a finished `$...` on another line does not qualify: {labels:?}"
    );
    assert!(!labels.contains(&"$env.".to_string()), "{labels:?}");
}

/// Completion offers schema references only inside the `@schema` value; a key
/// after that line keeps the normal field and keyword completion.
#[tokio::test]
async fn completion_offers_schema_references_only_inside_the_directive_value() {
    let mut harness = LspHarness::start().await;
    let uri = harness.document_uri("config.rune");

    harness.did_open(&uri, CONFIG_AFTER_SCHEMA_DIRECTIVE).await;
    harness.discard_server_messages().await;

    // Line 0 is `@schema "./schema.rune"`, so character 12 is inside the value.
    let labels = completion_labels(&mut harness, &uri, 0, 12).await;
    assert!(
        labels.contains(&"./schema.rune".to_string()),
        "the directive value offers schema references: {labels:?}"
    );
    assert!(labels.contains(&"./schemas/".to_string()), "{labels:?}");
    assert!(!labels.contains(&"end".to_string()), "{labels:?}");

    // Line 3 is the empty key position after the directive line.
    let labels = completion_labels(&mut harness, &uri, 3, 2).await;
    assert!(
        labels.contains(&"end".to_string()),
        "a later key is normal field and keyword completion: {labels:?}"
    );
    assert!(
        !labels.contains(&"./schema.rune".to_string()),
        "the directive value must not reach a later line: {labels:?}"
    );
    assert!(
        !labels.contains(&"$env.".to_string()),
        "a `$` inside an earlier string must not offer reference items here: {labels:?}"
    );

    // A still-open `@schema "` has no string token at all, and is still the
    // directive's value: the same line after the opening quote is the context.
    harness.replace_document(&uri, 2, "@schema \"").await;
    harness.discard_server_messages().await;

    let labels = completion_labels(&mut harness, &uri, 0, 9).await;
    assert!(
        labels.contains(&"./schema.rune".to_string()),
        "an unterminated directive value is still the directive's value: {labels:?}"
    );
    assert!(!labels.contains(&"end".to_string()), "{labels:?}");
}

/// A symlinked directory cycle and a symlinked file that points outside the
/// workspace are both skipped, so initialization and the request that follows
/// it finish, the real in-workspace config is reported once, and the linked
/// config is never a part of the workspace.
#[cfg(unix)]
#[tokio::test]
async fn symlinked_cycles_and_links_out_of_the_workspace_are_never_followed() {
    let outside = tempfile::tempdir().expect("outside directory");
    std::fs::write(outside.path().join("outside.rune"), CONFIG_STRING_VALUE)
        .expect("write the outside config");

    let mut harness = LspHarness::boot().await;
    let root = harness.workspace_path().to_path_buf();
    write_workspace_fixture(&root, "real/schema.rune", SCHEMA_STRING_FIELD);
    write_workspace_fixture(&root, "real/config.rune", CONFIG_STRING_VALUE);

    // A link to the workspace root and a link to the directory holding it: a
    // scan that followed either of them would walk the same tree forever.
    std::os::unix::fs::symlink(&root, root.join("real/up")).expect("cycle link");
    std::os::unix::fs::symlink(root.join("real"), root.join("real/self")).expect("self link");
    // A link to the `.rune` config outside the workspace: following it would
    // put a second copy of that config inside the workspace.
    std::os::unix::fs::symlink(
        outside.path().join("outside.rune"),
        root.join("real/outside.rune"),
    )
    .expect("outward link");

    let root_uri = Url::from_directory_path(&root).expect("workspace root uri");
    let schema_uri = harness.file_uri("real/schema.rune");
    let config_uri = harness.file_uri("real/config.rune");
    let linked_uri = harness.file_uri("real/outside.rune");

    // Initialization scans the workspace, so a followed cycle would hang here.
    let references = tokio::time::timeout(Duration::from_secs(10), async {
        harness
            .initialize(json!({
                "processId": Value::Null,
                "rootUri": root_uri,
                "capabilities": {},
            }))
            .await;

        harness
            .request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": config_uri },
                    "position": { "line": 1, "character": 3 },
                    "context": { "includeDeclaration": true },
                }),
            )
            .await
    })
    .await
    .expect("a symlink cycle must never be followed");

    let locations = references
        .as_array()
        .unwrap_or_else(|| panic!("references must return locations, got {references}"));
    assert_eq!(
        locations.len(),
        2,
        "the declaration and the one real config usage: {references}"
    );

    let uris: Vec<&str> = locations
        .iter()
        .filter_map(|location| location["uri"].as_str())
        .collect();
    assert_eq!(
        uris.iter()
            .filter(|uri| **uri == config_uri.as_str())
            .count(),
        1,
        "the real config is reported exactly once: {references}"
    );
    assert!(uris.contains(&schema_uri.as_str()), "{references}");
    assert!(
        !uris.contains(&linked_uri.as_str()),
        "a symlinked file is not a workspace member: {references}"
    );
    assert!(
        !uris.iter().any(|uri| uri.contains("outside.rune")),
        "the linked config outside the workspace never appears: {references}"
    );
}

/// `initializationOptions.exclude` extends the always-excluded directories: a
/// bound config under an excluded directory is not a reference or rename
/// target, while a normal sibling config is.
#[tokio::test]
async fn excluded_directories_are_not_reference_or_rename_targets() {
    let mut harness = LspHarness::start_with_initializer(|root| {
        write_workspace_fixture(root, "schema.rune", SCHEMA_STRING_FIELD);
        write_workspace_fixture(root, "config.rune", CONFIG_STRING_VALUE);
        write_workspace_fixture(root, "sibling.rune", CONFIG_STRING_VALUE);
        write_workspace_fixture(root, "vendor/vendored.rune", CONFIG_STRING_VALUE);
        write_workspace_fixture(root, "target/built.rune", CONFIG_STRING_VALUE);

        json!({
            "processId": Value::Null,
            "rootUri": Url::from_directory_path(root).expect("workspace root uri"),
            "initializationOptions": { "exclude": ["vendor"] },
            "capabilities": {},
        })
    })
    .await;

    let config_uri = harness.document_uri("config.rune");
    let schema_uri = harness.document_uri("schema.rune");
    let sibling_uri = harness.document_uri("sibling.rune");
    let vendored_uri = harness.file_uri("vendor/vendored.rune");
    let built_uri = harness.file_uri("target/built.rune");

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

    let uris: Vec<&str> = references
        .as_array()
        .unwrap_or_else(|| panic!("references must return locations, got {references}"))
        .iter()
        .filter_map(|location| location["uri"].as_str())
        .collect();
    assert_eq!(
        uris.len(),
        3,
        "the schema declaration and the two indexed configs: {references}"
    );
    assert!(uris.contains(&schema_uri.as_str()), "{references}");
    assert!(uris.contains(&config_uri.as_str()), "{references}");
    assert!(
        uris.contains(&sibling_uri.as_str()),
        "a normal sibling config is a target: {references}"
    );
    assert!(
        !uris.contains(&vendored_uri.as_str()),
        "a config under a client-excluded directory is not indexed: {references}"
    );
    assert!(
        !uris.contains(&built_uri.as_str()),
        "a config under `target` is not indexed: {references}"
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
        3,
        "only the indexed documents are renamed: {rename}"
    );
    assert!(!changes.contains_key(vendored_uri.as_str()), "{rename}");
    assert!(!changes.contains_key(built_uri.as_str()), "{rename}");
}

/// A create event for one file makes exactly that file visible to cross-file
/// requests: a file written to disk without an event and without being opened
/// stays invisible, which is what proves no second walk happened.
#[tokio::test]
async fn watched_creates_are_indexed_and_unreported_files_stay_invisible() {
    let mut harness = LspHarness::start_with_initializer(|root| {
        write_workspace_fixture(root, "schema.rune", SCHEMA_STRING_FIELD);
        write_workspace_fixture(root, "anchor.rune", CONFIG_STRING_VALUE);

        json!({
            "processId": Value::Null,
            "rootUri": Url::from_directory_path(root).expect("workspace root uri"),
            "capabilities": {},
        })
    })
    .await;

    let anchor_uri = harness.document_uri("anchor.rune");
    let schema_uri = harness.document_uri("schema.rune");
    let watched_uri = harness.document_uri("watched.rune");
    let silent_uri = harness.document_uri("silent.rune");

    // Both files exist on disk, but the client only reports one of them.
    harness.write_file("watched.rune", CONFIG_STRING_VALUE);
    harness.write_file("silent.rune", CONFIG_STRING_VALUE);
    harness
        .did_change_watched_files(&[(watched_uri.clone(), 1)])
        .await;
    harness.discard_server_messages().await;

    let references = harness
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": anchor_uri },
                "position": { "line": 1, "character": 3 },
                "context": { "includeDeclaration": true },
            }),
        )
        .await;

    let uris: Vec<&str> = references
        .as_array()
        .unwrap_or_else(|| panic!("references must return locations, got {references}"))
        .iter()
        .filter_map(|location| location["uri"].as_str())
        .collect();
    assert_eq!(
        uris.len(),
        3,
        "the schema declaration, the anchor and the watched config: {references}"
    );
    assert!(uris.contains(&schema_uri.as_str()), "{references}");
    assert!(uris.contains(&anchor_uri.as_str()), "{references}");
    assert!(
        uris.contains(&watched_uri.as_str()),
        "the watched create is indexed: {references}"
    );
    assert!(
        !uris.contains(&silent_uri.as_str()),
        "a file nobody reported stays invisible: {references}"
    );

    let rename = harness
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": anchor_uri },
                "position": { "line": 1, "character": 3 },
                "newName": "title",
            }),
        )
        .await;
    let changes = rename["changes"]
        .as_object()
        .unwrap_or_else(|| panic!("rename must return a workspace edit, got {rename}"));
    assert!(changes.contains_key(watched_uri.as_str()), "{rename}");
    assert!(
        !changes.contains_key(silent_uri.as_str()),
        "the unreported file is not renamed: {rename}"
    );
}

/// A buffer whose text differs from the indexed file on disk is what the
/// cross-file requests answer from, and a versioned rename keeps that buffer's
/// version.
#[tokio::test]
async fn an_open_buffer_wins_over_the_indexed_file_on_disk() {
    let mut harness = LspHarness::start_with_initializer(|root| {
        write_workspace_fixture(root, "schema.rune", SCHEMA_STRING_FIELD);
        // The disk copy writes the key once; the buffer opens with it twice.
        write_workspace_fixture(root, "config.rune", CONFIG_STRING_VALUE);

        json!({
            "processId": Value::Null,
            "rootUri": Url::from_directory_path(root).expect("workspace root uri"),
            "capabilities": { "workspace": { "workspaceEdit": { "documentChanges": true } } },
        })
    })
    .await;

    let config_uri = harness.document_uri("config.rune");
    let schema_uri = harness.document_uri("schema.rune");

    harness.did_open(&config_uri, CONFIG_TWO_USAGES).await;
    harness
        .replace_document(&config_uri, 7, CONFIG_TWO_USAGES)
        .await;
    harness.discard_server_messages().await;

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
        3,
        "the declaration plus both buffer usages: {references}"
    );
    let config_ranges: Vec<Value> = locations
        .iter()
        .filter(|location| location["uri"] == json!(config_uri.as_str()))
        .map(|location| location["range"].clone())
        .collect();
    assert_eq!(
        config_ranges,
        vec![indented_key_range(1), indented_key_range(2)],
        "the buffer's own two usages are the ones reported: {references}"
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
    let document_changes = rename["documentChanges"]
        .as_array()
        .unwrap_or_else(|| panic!("rename must return documentChanges, got {rename}"));
    let opened = document_changes
        .iter()
        .find(|change| change["textDocument"]["uri"] == json!(config_uri.as_str()))
        .unwrap_or_else(|| panic!("the open buffer is edited: {rename}"));
    assert_eq!(
        opened["textDocument"]["version"],
        json!(7),
        "the open buffer's version is the one kept: {rename}"
    );
    assert_eq!(
        opened["edits"],
        json!([
            { "range": indented_key_range(1), "newText": "title" },
            { "range": indented_key_range(2), "newText": "title" },
        ]),
        "the buffer's two usages are edited: {rename}"
    );
    assert_eq!(
        document_changes.len(),
        2,
        "the schema declaration and the open config: {rename}"
    );
    assert_eq!(
        document_changes
            .iter()
            .find(|change| change["textDocument"]["uri"] == json!(schema_uri.as_str()))
            .map(|change| change["textDocument"]["version"].clone()),
        Some(Value::Null),
        "the unopened schema carries no version: {rename}"
    );
}
