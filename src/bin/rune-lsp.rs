use rune_cfg::lsp::RuneLanguageServer;
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
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn print_help() {
    println!(
        "rune-lsp {version}\n\nLanguage server for RUNE configuration files.\n\nUsage:\n  rune-lsp [OPTIONS]\n\nOptions:\n  -h, --help     Print help\n  -V, --version  Print version\n\nWhen started without options, rune-lsp speaks LSP over stdio.",
        version = env!("CARGO_PKG_VERSION")
    );
}
