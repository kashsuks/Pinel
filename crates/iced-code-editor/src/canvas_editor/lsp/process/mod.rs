//! LSP (Language Server Protocol) Process Client implementation.
//!
//! This module provides a client for communicating with LSP servers via stdio.
//! It handles document synchronization, hover requests, and completion requests.
//!
//! Enable with the `lsp-process` Cargo feature. Not available on WASM targets.
//!
//! The transport itself lives in [`protocol`]: framing, bounded reads, message
//! dispatch, and response parsing. [`text_model`] holds the per-document mirror
//! used for UTF-16 position translation, and [`pending`] tracks in-flight
//! requests. What remains here is the process lifecycle and the [`LspClient`]
//! implementation the editor talks to.

pub mod config;
pub mod overlay;

mod pending;
mod protocol;
mod text_model;

use self::config::{
    LspCommand, ensure_rust_analyzer_config, lsp_server_config, resolve_lsp_command,
};
use self::pending::{LspRequestKind, PendingRequest, evict_expired_requests};
use self::protocol::{
    frame_message, handle_client_response, handle_server_notification, handle_server_request,
    read_log_line, read_message,
};
use self::text_model::{DocumentState, TextModel, apply_changes_to_document};
use crate::canvas_editor::lsp::{LspClient, LspDocument, LspPosition, LspRange, LspTextChange};
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Instant;
// =============================================================================
// LSP Events - Events sent back to the main application
// =============================================================================

/// Events that can be sent from the LSP client to the application.
///
/// Receive these by polling the `mpsc::Receiver` you pass to
/// [`LspProcessClient::new_with_server`]. Requests are fire-and-forget, so
/// every server reply arrives here rather than as a return value — drain the
/// receiver on a timer and fold the results into the application state.
///
/// # Examples
///
/// ```
/// use std::sync::mpsc;
///
/// use iced_code_editor::LspEvent;
///
/// let (tx, rx) = mpsc::channel::<LspEvent>();
/// tx.send(LspEvent::Hover { text: "fn main()".to_string() })
///     .expect("the receiver is still alive");
///
/// // Drain everything queued since the last tick.
/// while let Ok(event) = rx.try_recv() {
///     match event {
///         LspEvent::Hover { text } => assert_eq!(text, "fn main()"),
///         LspEvent::Completion { items } => drop(items),
///         LspEvent::Definition { uri, .. } => drop(uri),
///         LspEvent::Progress { done, .. } => drop(done),
///         LspEvent::DocumentSymbols { symbols, .. } => drop(symbols),
///         LspEvent::Diagnostics { diagnostics, .. } => drop(diagnostics),
///         LspEvent::Log { message, .. } => drop(message),
///     }
/// }
/// ```
pub enum LspEvent {
    /// Hover information received from the LSP server.
    Hover {
        /// Markdown or plain-text hover content.
        text: String,
    },
    /// Completion items received from the LSP server.
    Completion {
        /// List of completion label strings.
        items: Vec<String>,
    },
    /// Definition location received from the LSP server.
    Definition {
        /// Target document URI.
        uri: String,
        /// Target range within that document.
        range: crate::canvas_editor::lsp::LspRange,
    },
    /// Progress notification from the LSP server.
    Progress {
        /// Progress token identifier.
        token: String,
        /// Key of the server that sent this notification.
        server_key: String,
        /// Human-readable title for the progress operation.
        title: String,
        /// Optional status message.
        message: Option<String>,
        /// Optional percentage complete (0–100).
        percentage: Option<u32>,
        /// `true` when this is the final progress notification.
        done: bool,
    },
    /// Document outline received from the LSP server.
    DocumentSymbols {
        /// URI of the document the symbols belong to.
        uri: String,
        /// Top-level symbols; nested symbols are in each symbol's `children`.
        symbols: Vec<crate::canvas_editor::lsp::LspSymbol>,
    },
    /// The latest diagnostics published for a document. An empty list clears them.
    Diagnostics {
        /// URI of the file being diagnosed.
        uri: String,
        /// Full replacement for the previous diagnostics for this URI.
        diagnostics: Vec<LspDiagnostic>,
    },
    /// Log message from the LSP server's stderr.
    Log {
        /// Key of the server that sent this message.
        server_key: String,
        /// The log line.
        message: String,
    },
}

/// A diagnostic published by a language server.
pub struct LspDiagnostic {
    /// Zero-based start line.
    pub line: usize,
    /// LSP severity from 1 (error) through 4 (hint), when provided.
    pub severity: Option<u64>,
    /// Human-readable description of the problem.
    pub message: String,
}

// =============================================================================
// LSP Process Client - Main client implementation
// =============================================================================

/// Client for communicating with an LSP server process.
///
/// Manages the lifecycle of the server process and handles all communication.
/// Implements [`LspClient`] so it can be plugged directly into a [`CodeEditor`].
///
/// # Examples
///
/// ```no_run
/// use std::sync::mpsc;
/// use iced_code_editor::{LspEvent, LspProcessClient, LspWorkspaceRoot};
///
/// let (tx, rx) = mpsc::channel::<LspEvent>();
/// let root = LspWorkspaceRoot {
///     uri: "file:///home/user/project".to_string(),
///     path: "/home/user/project".into(),
/// };
/// let client = LspProcessClient::new_with_server(Some(&root), tx, "lua-language-server");
/// ```
///
/// [`CodeEditor`]: crate::CodeEditor
pub struct LspProcessClient {
    /// The child process running the LSP server
    child: Child,
    /// Channel for sending messages to the writer thread
    writer: mpsc::Sender<Vec<u8>>,
    /// Map of URI to document state for all open documents
    documents: Arc<Mutex<HashMap<String, DocumentState>>>,
    /// Channel used to report a document mirror going out of sync (see
    /// [`Self::apply_change_and_convert`]); the reader/stderr threads carry
    /// their own clone of the same sender for server-pushed events.
    events: mpsc::Sender<LspEvent>,
    /// Key identifying the connected server, used to label events emitted
    /// directly by the client rather than by the reader/stderr threads.
    server_key: String,
    /// Counter for generating unique request IDs
    request_id: AtomicU64,
    /// Map of pending request IDs to their types and send time (for
    /// response routing and expiry of abandoned requests)
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    /// Handle to the writer thread (kept alive for the client's lifetime)
    _writer_thread: thread::JoinHandle<()>,
    /// Handle to the reader thread (kept alive for the client's lifetime)
    _reader_thread: thread::JoinHandle<()>,
    /// Handle to the stderr thread (kept alive for the client's lifetime)
    _stderr_thread: thread::JoinHandle<()>,
}

/// The folder a language server treats as its workspace.
///
/// The server process runs with `path` as its working directory, and the
/// `initialize` request names the folder in `rootUri`, `rootPath` and
/// `workspaceFolders`, so no server falls back to the editor's own working
/// directory (which is `/` when a macOS app is launched from the Dock).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspWorkspaceRoot {
    /// `file://` URI of the folder, such as `"file:///home/user/project"`.
    pub uri: String,
    /// Local path of the same folder.
    pub path: PathBuf,
}

impl LspProcessClient {
    /// Creates a new LSP client connected to the specified server.
    ///
    /// # Arguments
    ///
    /// * `root` — the workspace folder, or `None` when there is none. Without
    ///   one the server gets a null `rootUri` and keeps the editor's working
    ///   directory.
    /// * `events` — channel to send [`LspEvent`]s back to the application
    /// * `server_key` — key identifying the LSP server (e.g. `"lua-language-server"`)
    ///
    /// # Errors
    ///
    /// Returns an error string when the server key is not recognised, when the
    /// server binary cannot be found, or when the process cannot be spawned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::mpsc;
    /// use iced_code_editor::{LspEvent, LspProcessClient, LspWorkspaceRoot};
    ///
    /// let (tx, _rx) = mpsc::channel::<LspEvent>();
    /// let root = LspWorkspaceRoot {
    ///     uri: "file:///tmp/project".to_string(),
    ///     path: "/tmp/project".into(),
    /// };
    /// let client = LspProcessClient::new_with_server(Some(&root), tx, "lua-language-server");
    /// assert!(client.is_ok());
    /// ```
    pub fn new_with_server(
        root: Option<&LspWorkspaceRoot>,
        events: mpsc::Sender<LspEvent>,
        server_key: &str,
    ) -> Result<Self, String> {
        let config = lsp_server_config(server_key)
            .ok_or_else(|| format!("Unsupported LSP server: {}", server_key))?;

        if server_key == "rust-analyzer" {
            ensure_rust_analyzer_config();
        }

        let command = resolve_lsp_command(config)?;
        Self::new_with_command(root, events, &command, server_key)
    }

    /// Creates a new LSP client with a specific command.
    ///
    /// This is the internal implementation that spawns the process.
    ///
    /// # Errors
    ///
    /// Returns an error string if the process cannot be spawned or if stdio
    /// handles cannot be acquired.
    fn new_with_command(
        root: Option<&LspWorkspaceRoot>,
        events: mpsc::Sender<LspEvent>,
        command: &LspCommand,
        server_key: &str,
    ) -> Result<Self, String> {
        let mut child = server_command(command, root.map(|root| root.path.as_path()))
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    if command.program == "rust-analyzer" {
                        "LSP server program rust-analyzer not found. Please install rust-analyzer or set RUST_ANALYZER/RUST_ANALYZER_PATH environment variable".to_string()
                    } else {
                        format!("LSP server program {} not found", command.program)
                    }
                } else {
                    e.to_string()
                }
            })?;

        let stdin = child.stdin.take().ok_or("stdin unavailable")?;
        let stdout = child.stdout.take().ok_or("stdout unavailable")?;
        let stderr = child.stderr.take().ok_or("stderr unavailable")?;

        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let pending_reader = pending_requests.clone();
        let events_reader = events.clone();
        let events_log = events.clone();
        let events_field = events;
        let server_key = server_key.to_string();
        let server_key_reader = server_key.clone();
        let server_key_log = server_key.clone();
        let tx_reader = tx.clone();

        let writer_thread = thread::spawn(move || {
            let mut input = stdin;
            for bytes in rx {
                if input.write_all(&bytes).is_err() {
                    break;
                }
                let _ = input.flush();
            }
        });

        let reader_thread = thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Some(buf) = read_message(&mut reader) {
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&buf) {
                    if let Some(id) = value.get("id").and_then(|v| v.as_u64()) {
                        if let Some(method) = value.get("method").and_then(|m| m.as_str()) {
                            handle_server_request(id, method, &tx_reader);
                        } else {
                            handle_client_response(id, &value, &pending_reader, &events_reader);
                        }
                    } else if let Some(method) = value.get("method").and_then(|m| m.as_str())
                        && let Some(params) = value.get("params")
                    {
                        handle_server_notification(
                            method,
                            params,
                            &events_reader,
                            &server_key_reader,
                        );
                    }
                }
            }
        });

        let stderr_thread = thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            while let Some(line) = read_log_line(&mut reader) {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = events_log.send(LspEvent::Log {
                    server_key: server_key_log.clone(),
                    message: line.to_string(),
                });
            }
        });

        let client = Self {
            child,
            writer: tx,
            documents: Arc::new(Mutex::new(HashMap::new())),
            events: events_field,
            server_key,
            request_id: AtomicU64::new(1),
            pending_requests,
            _writer_thread: writer_thread,
            _reader_thread: reader_thread,
            _stderr_thread: stderr_thread,
        };

        let initialize = json!({
            "jsonrpc": "2.0",
            "id": client.next_id(),
            "method": "initialize",
            "params": initialize_params(root, std::process::id())
        });
        client.send_message(&initialize);

        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });
        client.send_message(&initialized);

        Ok(client)
    }

    /// Generates the next unique request ID using atomic operations.
    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Sends a JSON-RPC message to the LSP server.
    ///
    /// Formats the message with the required `Content-Length` header.
    fn send_message(&self, value: &serde_json::Value) {
        if let Some(bytes) = frame_message(value) {
            let _ = self.writer.send(bytes);
        }
    }

    /// Applies text changes to a document and converts them to JSON format.
    ///
    /// Also converts positions to UTF-16 as required by LSP.
    ///
    /// If the local document mirror desynchronizes partway through `changes`
    /// (see [`apply_changes_to_document`]), the document is dropped from
    /// [`Self::documents`] — so a later `did_open` reseeds it instead of
    /// this client continuing to serve hover/completion/definition
    /// positions computed from a copy known to be stale — and an
    /// [`LspEvent::Log`] is emitted so the desync is diagnosable instead of
    /// surfacing only as "the language server gives nonsense answers".
    fn apply_change_and_convert(
        &self,
        uri: &str,
        changes: &[LspTextChange],
    ) -> Vec<serde_json::Value> {
        let mut docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        let Some(state) = docs.get_mut(uri) else {
            return Vec::new();
        };

        match apply_changes_to_document(state, changes) {
            Some(out) => out,
            None => {
                docs.remove(uri);
                let _ = self.events.send(LspEvent::Log {
                    server_key: self.server_key.clone(),
                    message: format!(
                        "Local document mirror for {uri} desynchronized \
                         (a change referenced a line outside the tracked \
                         document); dropping it until the next open."
                    ),
                });
                Vec::new()
            },
        }
    }
}

/// Builds the command that starts a language server, with piped stdio.
///
/// When `working_dir` is given the server runs there, instead of inheriting
/// the editor's working directory.
fn server_command(command: &LspCommand, working_dir: Option<&Path>) -> Command {
    let mut cmd = Command::new(&command.program);
    cmd.args(&command.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }
    cmd
}

/// Builds the `params` of the `initialize` request sent to every server.
///
/// `root` is sent as `rootUri`, `rootPath` and the single entry of
/// `workspaceFolders`. Servers differ in which of these they read, and one
/// that finds none of them falls back to its working directory. Without a
/// root, all three are null.
///
/// `textDocument.hover.contentFormat` prefers markdown: without it, servers
/// such as pyright, rust-analyzer and clangd answer hovers with plaintext, so
/// the signature arrives without a code fence and renders as prose.
fn initialize_params(root: Option<&LspWorkspaceRoot>, process_id: u32) -> serde_json::Value {
    let workspace_folders = root.map(|root| {
        let name = root
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.path.display().to_string());
        json!([{ "uri": root.uri, "name": name }])
    });
    json!({
        "processId": process_id,
        "rootUri": root.map(|root| root.uri.as_str()),
        "rootPath": root.map(|root| root.path.display().to_string()),
        "capabilities": {
            "textDocument": {
                "synchronization": {
                    "dynamicRegistration": false,
                    "willSave": false,
                    "didSave": true
                },
                "hover": {
                    "contentFormat": ["markdown", "plaintext"]
                },
                "documentSymbol": {
                    "hierarchicalDocumentSymbolSupport": true
                }
            },
            "window": {
                "workDoneProgress": true
            }
        },
        "workspaceFolders": workspace_folders
    })
}

// =============================================================================
// Reader thread helper functions
// =============================================================================

/// Sends shutdown/exit notifications and kills the process on drop.
impl Drop for LspProcessClient {
    fn drop(&mut self) {
        let shutdown = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "shutdown",
            "params": null
        });
        self.send_message(&shutdown);

        let exit = json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": {}
        });
        self.send_message(&exit);

        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
    }
}
// =============================================================================
// LspClient Trait Implementation
// =============================================================================

impl LspClient for LspProcessClient {
    fn did_open(&mut self, document: &LspDocument, text: &str) {
        let mut docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        docs.insert(
            document.uri.clone(),
            DocumentState {
                text: TextModel::from_text(text),
            },
        );

        let msg = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": document.uri,
                    "languageId": document.language_id,
                    "version": document.version,
                    "text": text
                }
            }
        });
        self.send_message(&msg);
    }

    fn did_change(&mut self, document: &LspDocument, changes: &[LspTextChange]) {
        let content_changes = self.apply_change_and_convert(&document.uri, changes);
        if content_changes.is_empty() {
            return;
        }

        let msg = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": document.uri,
                    "version": document.version
                },
                "contentChanges": content_changes
            }
        });
        self.send_message(&msg);
    }

    fn did_save(&mut self, document: &LspDocument, text: &str) {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didSave",
            "params": {
                "textDocument": { "uri": document.uri },
                "text": text
            }
        });
        self.send_message(&msg);
    }

    fn did_close(&mut self, document: &LspDocument) {
        let mut docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        docs.remove(&document.uri);

        let msg = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {
                "textDocument": { "uri": document.uri }
            }
        });
        self.send_message(&msg);
    }

    fn request_hover(&mut self, document: &LspDocument, position: LspPosition) {
        let docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        let Some(state) = docs.get(&document.uri) else {
            return;
        };
        let pos = state.text.to_utf16_position(position);

        let id = self.next_id();
        {
            let mut pending = self.pending_requests.lock().unwrap_or_else(|e| e.into_inner());
            evict_expired_requests(&mut pending);
            pending.insert(
                id,
                PendingRequest {
                    kind: LspRequestKind::Hover,
                    requested_at: Instant::now(),
                },
            );
        }

        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": document.uri },
                "position": { "line": pos.line, "character": pos.character }
            }
        });
        self.send_message(&msg);
    }

    fn request_completion(&mut self, document: &LspDocument, position: LspPosition) {
        let docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        let Some(state) = docs.get(&document.uri) else {
            return;
        };
        let pos = state.text.to_utf16_position(position);

        let id = self.next_id();
        {
            let mut pending = self.pending_requests.lock().unwrap_or_else(|e| e.into_inner());
            evict_expired_requests(&mut pending);
            pending.insert(
                id,
                PendingRequest {
                    kind: LspRequestKind::Completion,
                    requested_at: Instant::now(),
                },
            );
        }

        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": document.uri },
                "position": { "line": pos.line, "character": pos.character },
                "context": { "triggerKind": 1 }
            }
        });
        self.send_message(&msg);
    }

    fn request_definition(&mut self, document: &LspDocument, position: LspPosition) {
        let docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        let Some(state) = docs.get(&document.uri) else {
            return;
        };
        let pos = state.text.to_utf16_position(position);

        let id = self.next_id();
        {
            let mut pending = self.pending_requests.lock().unwrap_or_else(|e| e.into_inner());
            evict_expired_requests(&mut pending);
            pending.insert(
                id,
                PendingRequest {
                    kind: LspRequestKind::Definition,
                    requested_at: Instant::now(),
                },
            );
        }

        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": document.uri },
                "position": { "line": pos.line, "character": pos.character }
            }
        });
        self.send_message(&msg);
    }

    fn request_document_symbols(&mut self, document: &LspDocument) {
        let id = self.next_id();
        {
            let mut pending = self.pending_requests.lock().unwrap_or_else(|e| e.into_inner());
            evict_expired_requests(&mut pending);
            pending.insert(
                id,
                PendingRequest {
                    kind: LspRequestKind::DocumentSymbols {
                        uri: document.uri.clone(),
                    },
                    requested_at: Instant::now(),
                },
            );
        }

        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/documentSymbol",
            "params": {
                "textDocument": { "uri": document.uri }
            }
        });
        self.send_message(&msg);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    // Several helpers and tests below carry an `#[allow]` for `expect_used`,
    // `unwrap_used`, or `panic`. In test code a panic *is* the failure
    // report, so these mirror the existing per-test allows in `protocol.rs`.
    use super::*;
    use crate::canvas_editor::lsp::{LspPosition, LspRange, LspTextChange};

    /// Builds an `LspProcessClient` without a real LSP server: `child` still
    /// needs a genuine `Child`, so a process that exits immediately stands in
    /// for it, and the writer/reader/stderr threads are no-ops since these
    /// tests exercise the client's own methods directly rather than the wire
    /// protocol those background threads drive.
    #[allow(clippy::expect_used)]
    fn test_client() -> (
        LspProcessClient,
        mpsc::Receiver<Vec<u8>>,
        mpsc::Receiver<LspEvent>,
    ) {
        let child = Command::new("true").spawn().expect("spawn stub process");
        let (writer_tx, writer_rx) = mpsc::channel::<Vec<u8>>();
        let (events_tx, events_rx) = mpsc::channel::<LspEvent>();
        let client = LspProcessClient {
            child,
            writer: writer_tx,
            documents: Arc::new(Mutex::new(HashMap::new())),
            events: events_tx,
            server_key: "test-server".to_string(),
            request_id: AtomicU64::new(1),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            _writer_thread: thread::spawn(|| {}),
            _reader_thread: thread::spawn(|| {}),
            _stderr_thread: thread::spawn(|| {}),
        };
        (client, writer_rx, events_rx)
    }

    /// Returns the JSON body of a `Content-Length`-framed message taken from
    /// the writer channel.
    #[allow(clippy::expect_used)]
    fn decode_sent(data: &[u8]) -> serde_json::Value {
        let header_end = data
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .expect("missing header separator");
        let body = &data[header_end + 4..];
        serde_json::from_slice(body).expect("invalid JSON body")
    }

    fn document(uri: &str) -> LspDocument {
        LspDocument {
            uri: uri.to_string(),
            language_id: "rust".to_string(),
            version: 1,
        }
    }

    fn range(sl: u32, sc: u32, el: u32, ec: u32) -> LspRange {
        LspRange {
            start: LspPosition {
                line: sl,
                character: sc,
            },
            end: LspPosition {
                line: el,
                character: ec,
            },
        }
    }

    // -------------------------------------------------------------------------
    // initialize_params
    // -------------------------------------------------------------------------

    fn project_root() -> LspWorkspaceRoot {
        LspWorkspaceRoot {
            uri: "file:///tmp/my%20project".to_string(),
            path: PathBuf::from("/tmp/my project"),
        }
    }

    #[test]
    fn test_initialize_params_requests_markdown_hover() {
        let params = initialize_params(Some(&project_root()), 42);

        let hover = &params["capabilities"]["textDocument"]["hover"];
        assert_eq!(hover["contentFormat"][0], "markdown");
        assert_eq!(hover["contentFormat"][1], "plaintext");

        // The rest of the request is unchanged by the hover capability.
        assert_eq!(params["processId"], 42);
        assert_eq!(
            params["capabilities"]["textDocument"]["documentSymbol"]["hierarchicalDocumentSymbolSupport"],
            true
        );
        assert_eq!(params["capabilities"]["window"]["workDoneProgress"], true);
    }

    #[test]
    fn test_initialize_params_with_root_names_it_everywhere() {
        let params = initialize_params(Some(&project_root()), 42);

        assert_eq!(params["rootUri"], "file:///tmp/my%20project");
        assert_eq!(params["rootPath"], "/tmp/my project");
        assert_eq!(
            params["workspaceFolders"][0]["uri"],
            "file:///tmp/my%20project"
        );
        assert_eq!(params["workspaceFolders"][0]["name"], "my project");
        assert_eq!(params["workspaceFolders"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn test_initialize_params_without_root_is_null() {
        let params = initialize_params(None, 42);

        assert!(params["rootUri"].is_null());
        assert!(params["rootPath"].is_null());
        assert!(params["workspaceFolders"].is_null());
    }

    // -------------------------------------------------------------------------
    // server_command
    // -------------------------------------------------------------------------

    fn stub_command() -> LspCommand {
        LspCommand {
            program: "stub-server".to_string(),
            args: vec!["--stdio".to_string()],
        }
    }

    #[test]
    fn test_server_command_runs_in_root() {
        let cmd = server_command(&stub_command(), Some(Path::new("/tmp/my project")));

        assert_eq!(cmd.get_current_dir(), Some(Path::new("/tmp/my project")));
        assert_eq!(cmd.get_program(), "stub-server");
    }

    #[test]
    fn test_server_command_without_root_keeps_working_dir() {
        let cmd = server_command(&stub_command(), None);

        assert_eq!(cmd.get_current_dir(), None);
    }

    // -------------------------------------------------------------------------
    // next_id / send_message
    // -------------------------------------------------------------------------

    #[test]
    fn test_next_id_increments() {
        let (client, _writer_rx, _events_rx) = test_client();
        assert_eq!(client.next_id(), 1);
        assert_eq!(client.next_id(), 2);
        assert_eq!(client.next_id(), 3);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_send_message_frames_and_forwards() {
        let (client, writer_rx, _events_rx) = test_client();
        client.send_message(&json!({"jsonrpc": "2.0", "method": "ping"}));

        let bytes = writer_rx.try_recv().expect("message forwarded to writer");
        assert_eq!(decode_sent(&bytes)["method"], "ping");
    }

    // -------------------------------------------------------------------------
    // did_open / did_change / did_save / did_close
    // -------------------------------------------------------------------------

    #[test]
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    fn test_did_open_registers_document_and_notifies_server() {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///a.rs");

        client.did_open(&doc, "fn main() {}");

        assert!(client.documents.lock().unwrap().contains_key("file:///a.rs"));
        let bytes = writer_rx.try_recv().expect("didOpen sent");
        let value = decode_sent(&bytes);
        assert_eq!(value["method"], "textDocument/didOpen");
        assert_eq!(value["params"]["textDocument"]["text"], "fn main() {}");
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_did_change_converts_and_sends_content_changes() {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///a.rs");
        client.did_open(&doc, "hello");
        writer_rx.try_recv().expect("drain didOpen");

        let changes = vec![LspTextChange {
            range: range(0, 0, 0, 5),
            text: "hi".to_string(),
        }];
        client.did_change(&doc, &changes);

        let bytes = writer_rx.try_recv().expect("didChange sent");
        let value = decode_sent(&bytes);
        assert_eq!(value["method"], "textDocument/didChange");
        assert_eq!(value["params"]["contentChanges"][0]["text"], "hi");
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_did_change_without_open_document_sends_nothing() {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///missing.rs");
        let changes = vec![LspTextChange {
            range: range(0, 0, 0, 0),
            text: "x".to_string(),
        }];

        client.did_change(&doc, &changes);

        assert!(writer_rx.try_recv().is_err());
    }

    #[test]
    #[allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::panic,
        clippy::assertions_on_result_states
    )]
    fn test_did_change_desync_drops_document_and_logs() {
        let (mut client, writer_rx, events_rx) = test_client();
        let doc = document("file:///a.rs");
        client.did_open(&doc, "hello");
        writer_rx.try_recv().expect("drain didOpen");

        // Line 5 doesn't exist in a single-line document, forcing a desync.
        let changes = vec![LspTextChange {
            range: range(5, 0, 5, 0),
            text: "x".to_string(),
        }];
        client.did_change(&doc, &changes);

        assert!(writer_rx.try_recv().is_err(), "no didChange after desync");
        assert!(!client.documents.lock().unwrap().contains_key("file:///a.rs"));

        match events_rx.try_recv().expect("desync logged") {
            LspEvent::Log { message, .. } => {
                assert!(message.contains("desynchronized"));
            },
            _ => panic!("expected LspEvent::Log"),
        }
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_did_save_sends_notification() {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///a.rs");

        client.did_save(&doc, "fn main() {}");

        let bytes = writer_rx.try_recv().expect("didSave sent");
        let value = decode_sent(&bytes);
        assert_eq!(value["method"], "textDocument/didSave");
        assert_eq!(value["params"]["text"], "fn main() {}");
    }

    #[test]
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    fn test_did_close_removes_document_and_notifies_server() {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///a.rs");
        client.did_open(&doc, "hello");
        writer_rx.try_recv().expect("drain didOpen");

        client.did_close(&doc);

        assert!(!client.documents.lock().unwrap().contains_key("file:///a.rs"));
        let bytes = writer_rx.try_recv().expect("didClose sent");
        assert_eq!(decode_sent(&bytes)["method"], "textDocument/didClose");
    }

    // -------------------------------------------------------------------------
    // request_hover / request_completion / request_definition
    // -------------------------------------------------------------------------

    #[test]
    #[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
    fn test_request_hover_registers_pending_and_sends_request() {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///a.rs");
        client.did_open(&doc, "hello");
        writer_rx.try_recv().expect("drain didOpen");

        client.request_hover(
            &doc,
            LspPosition {
                line: 0,
                character: 2,
            },
        );

        let bytes = writer_rx.try_recv().expect("hover request sent");
        let value = decode_sent(&bytes);
        assert_eq!(value["method"], "textDocument/hover");
        let id = value["id"].as_u64().expect("id present");

        let pending = client.pending_requests.lock().unwrap();
        match pending.get(&id).map(|p| &p.kind) {
            Some(LspRequestKind::Hover) => {},
            _ => panic!("expected a pending Hover request"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::assertions_on_result_states)]
    fn test_request_hover_without_open_document_does_nothing() {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///missing.rs");

        client.request_hover(
            &doc,
            LspPosition {
                line: 0,
                character: 0,
            },
        );

        assert!(writer_rx.try_recv().is_err());
        assert!(client.pending_requests.lock().unwrap().is_empty());
    }

    #[test]
    #[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
    fn test_request_completion_registers_pending_and_sends_request() {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///a.rs");
        client.did_open(&doc, "hello");
        writer_rx.try_recv().expect("drain didOpen");

        client.request_completion(
            &doc,
            LspPosition {
                line: 0,
                character: 2,
            },
        );

        let bytes = writer_rx.try_recv().expect("completion request sent");
        let value = decode_sent(&bytes);
        assert_eq!(value["method"], "textDocument/completion");
        let id = value["id"].as_u64().expect("id present");

        let pending = client.pending_requests.lock().unwrap();
        match pending.get(&id).map(|p| &p.kind) {
            Some(LspRequestKind::Completion) => {},
            _ => panic!("expected a pending Completion request"),
        }
    }

    #[test]
    fn test_request_document_symbols_registers_pending_and_sends_request()
    -> Result<(), Box<dyn std::error::Error>> {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///a.rs");

        client.request_document_symbols(&doc);

        let value = decode_sent(&writer_rx.try_recv()?);
        assert_eq!(value["method"], "textDocument/documentSymbol");
        assert_eq!(value["params"]["textDocument"]["uri"], "file:///a.rs");
        let id = value["id"].as_u64().ok_or("id missing")?;

        let pending = client.pending_requests.lock().unwrap_or_else(|e| e.into_inner());
        match pending.get(&id).map(|p| &p.kind) {
            Some(LspRequestKind::DocumentSymbols { uri }) => {
                assert_eq!(uri, "file:///a.rs");
                Ok(())
            },
            _ => Err("expected a pending DocumentSymbols request".into()),
        }
    }

    #[test]
    #[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
    fn test_request_definition_registers_pending_and_sends_request() {
        let (mut client, writer_rx, _events_rx) = test_client();
        let doc = document("file:///a.rs");
        client.did_open(&doc, "hello");
        writer_rx.try_recv().expect("drain didOpen");

        client.request_definition(
            &doc,
            LspPosition {
                line: 0,
                character: 2,
            },
        );

        let bytes = writer_rx.try_recv().expect("definition request sent");
        let value = decode_sent(&bytes);
        assert_eq!(value["method"], "textDocument/definition");
        let id = value["id"].as_u64().expect("id present");

        let pending = client.pending_requests.lock().unwrap();
        match pending.get(&id).map(|p| &p.kind) {
            Some(LspRequestKind::Definition) => {},
            _ => panic!("expected a pending Definition request"),
        }
    }

    // -------------------------------------------------------------------------
    // Drop
    // -------------------------------------------------------------------------

    #[test]
    #[allow(clippy::expect_used)]
    fn test_drop_sends_shutdown_then_exit() {
        let (client, writer_rx, _events_rx) = test_client();
        drop(client);

        let shutdown = writer_rx.try_recv().expect("shutdown sent");
        assert_eq!(decode_sent(&shutdown)["method"], "shutdown");

        let exit = writer_rx.try_recv().expect("exit sent");
        assert_eq!(decode_sent(&exit)["method"], "exit");
    }
}
