# RUNE VS Code Support

This directory contains an experimental VS Code extension for RUNE configuration files.

It provides:

- `*.rune` language detection
- basic TextMate syntax highlighting
- comment, bracket, and folding configuration
- LSP integration through `rune-lsp`

Install the language server binary first:

```sh
cargo install rune-cfg --version 0.5.0
```

Then install extension dependencies and compile the client:

```sh
cd editors/vscode
npm install
npm run compile
```

Open this directory in VS Code and run the extension from the Extension Development Host.

By default, the extension launches `rune-lsp` from `PATH`. To test a local development build, set:

```json
{
  "rune.server.path": "/path/to/rune-cfg/target/debug/rune-lsp"
}
```

The server speaks LSP over stdio and provides diagnostics, completion, hover, document symbols, code actions, go-to-definition, references, rename, and formatting.
