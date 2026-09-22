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
        let path = self.workspace.path().join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent directory");
        }
        std::fs::write(path, text).expect("write file");
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
    let mut harness = LspHarness::start_with_initializer(|root| {
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

    // One schema, bound from the first folder by a relative path and from the
    // second by its absolute path, so both configs resolve to the same schema
    // URI and both folders have to be scanned for the answer to be complete.
    let shared_schema = harness.workspace_path().join("one/shared.rune");
    harness.write_file("one/shared.rune", SCHEMA_STRING_FIELD);
    harness.write_file("one/config.rune", CONFIG_BOUND_TO_SHARED_SCHEMA);
    harness.write_file(
        "two/config.rune",
        &format!(
            "@schema \"{}\"\napp:\n  name \"Rune\"\nend\n",
            shared_schema.display()
        ),
    );

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
