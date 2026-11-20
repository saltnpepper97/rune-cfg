// Author: Dustin Pilgrim
// License: GPL-3.0-or-later

//! Process-level tests for how the `rune-lsp` binary leaves after `exit`.
//!
//! Editors keep the server's stdin open after sending `exit` and expect the
//! process to stop on its own. tower-lsp 0.20 only noticed `exit` when the next
//! message arrived, so the server lingered until the editor killed it. These
//! tests keep stdin open and require the process to stop within a few seconds.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const EXIT_TIMEOUT: Duration = Duration::from_secs(10);

struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Server {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rune-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("rune-lsp starts");
        let stdin = child.stdin.take().expect("stdin is piped");
        let stdout = BufReader::new(child.stdout.take().expect("stdout is piped"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, message: Value) {
        let body = message.to_string();
        write!(self.stdin, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("write message");
        self.stdin.flush().expect("flush message");
    }

    /// Reads messages until the response with `id` arrives.
    fn response(&mut self, id: u64) -> Value {
        loop {
            let message = self.read_message();
            if message.get("id") == Some(&json!(id)) && message.get("method").is_none() {
                return message;
            }
        }
    }

    fn read_message(&mut self) -> Value {
        let mut length = None;
        loop {
            let mut line = String::new();
            self.stdout.read_line(&mut line).expect("read header");
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(value) = line.strip_prefix("Content-Length: ") {
                length = Some(value.parse::<usize>().expect("numeric Content-Length"));
            }
        }
        let mut body = vec![0; length.expect("Content-Length header")];
        self.stdout.read_exact(&mut body).expect("read body");
        serde_json::from_slice(&body).expect("JSON body")
    }

    fn initialize(&mut self) {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "capabilities": {} }
        }));
        let response = self.response(1);
        assert!(
            response.get("result").is_some(),
            "initialize failed: {response}"
        );
        self.send(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
    }

    /// Waits for the process to stop while stdin stays open.
    fn wait_for_exit(&mut self) -> ExitStatus {
        let deadline = Instant::now() + EXIT_TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait().expect("poll rune-lsp") {
                return status;
            }
            if Instant::now() >= deadline {
                let _ = self.child.kill();
                panic!("rune-lsp did not exit within {EXIT_TIMEOUT:?} after `exit`");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// After `shutdown` and `exit`, the server stops with code 0 even though the
/// client has not closed stdin.
#[test]
fn exits_with_success_after_shutdown_and_exit() {
    let mut server = Server::start();
    server.initialize();

    server.send(json!({ "jsonrpc": "2.0", "id": 2, "method": "shutdown" }));
    let response = server.response(2);
    assert_eq!(response.get("result"), Some(&Value::Null), "{response}");

    server.send(json!({ "jsonrpc": "2.0", "method": "exit" }));
    let status = server.wait_for_exit();
    assert_eq!(status.code(), Some(0));
}

/// An `exit` without a preceding `shutdown` still stops the server, with the
/// error code the LSP specification requires.
#[test]
fn exits_with_error_when_exit_arrives_without_shutdown() {
    let mut server = Server::start();
    server.initialize();

    server.send(json!({ "jsonrpc": "2.0", "method": "exit" }));
    let status = server.wait_for_exit();
    assert_eq!(status.code(), Some(1));
}
