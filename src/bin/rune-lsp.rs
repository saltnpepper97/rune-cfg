use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

use rune_cfg::lsp::RuneLanguageServer;
use tokio::sync::oneshot;
use tower::Service;
use tower_lsp::jsonrpc::{Request, Response};
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("rune-lsp {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(RuneLanguageServer::new);
    let (service, exit_rx, shutdown_requested) = ExitWatcher::new(service);

    // tower-lsp 0.20 only notices the `exit` notification when it reads the
    // next message, so `serve` keeps blocking on stdin after an editor asks the
    // server to exit. Stop as soon as `exit` has been handled instead.
    let exited = tokio::select! {
        () = Server::new(stdin, stdout, socket).serve(service) => false,
        _ = exit_rx => true,
    };

    // Tokio's stdin uses a blocking read that cannot be cancelled, so leaving
    // `main` normally could still wait for input. Exit the process directly,
    // using the code the LSP specification asks for after `exit`.
    let code = if exited && !shutdown_requested.load(Ordering::SeqCst) {
        1
    } else {
        0
    };
    std::process::exit(code);
}

/// Forwards every message to the language service and reports when the
/// client's `exit` notification has been processed.
struct ExitWatcher<S> {
    inner: S,
    exit_tx: Option<oneshot::Sender<()>>,
    shutdown_requested: Arc<AtomicBool>,
}

impl<S> ExitWatcher<S> {
    fn new(inner: S) -> (Self, oneshot::Receiver<()>, Arc<AtomicBool>) {
        let (exit_tx, exit_rx) = oneshot::channel();
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let watcher = Self {
            inner,
            exit_tx: Some(exit_tx),
            shutdown_requested: Arc::clone(&shutdown_requested),
        };
        (watcher, exit_rx, shutdown_requested)
    }
}

impl<S> Service<Request> for ExitWatcher<S>
where
    S: Service<Request, Response = Option<Response>>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let method = request.method().to_owned();
        if method == "shutdown" {
            self.shutdown_requested.store(true, Ordering::SeqCst);
        }

        let response = self.inner.call(request);
        let exit_tx = if method == "exit" {
            self.exit_tx.take()
        } else {
            None
        };

        Box::pin(async move {
            let result = response.await;
            if let Some(exit_tx) = exit_tx {
                let _ = exit_tx.send(());
            }
            result
        })
    }
}

fn print_help() {
    println!(
        "rune-lsp {version}\n\nLanguage server for RUNE configuration files.\n\nUsage:\n  rune-lsp [OPTIONS]\n\nOptions:\n  -h, --help     Print help\n  -V, --version  Print version\n\nWhen started without options, rune-lsp speaks LSP over stdio.",
        version = env!("CARGO_PKG_VERSION")
    );
}
