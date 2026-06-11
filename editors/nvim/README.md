# Neovim Support

This directory contains lightweight Neovim runtime files for RUNE.

An experimental Tree-sitter grammar also exists at `editors/tree-sitter-rune/`. Use these Vim runtime files as the stable fallback while the Tree-sitter grammar matures.

They provide:

- `*.rune` filetype detection
- regex-based syntax highlighting
- 2-space RUNE indentation

Install them by copying the runtime directories into your Neovim config:

```sh
cp -r editors/nvim/ftdetect ~/.config/nvim/
cp -r editors/nvim/syntax ~/.config/nvim/
cp -r editors/nvim/ftplugin ~/.config/nvim/
```

The syntax file distinguishes:

- top-level object blocks, such as `app:`
- nested object blocks, such as `server:`
- assignment keys, such as `port`
- closing block keywords, such as `end` and `endif`
- strings, regex literals, numbers, booleans, nulls, metadata, schema keywords, and `$env`/`$sys` references

For LSP diagnostics, completion, hover, navigation, rename, and formatting, install `rune-lsp` and configure Neovim to launch it:

See [Language Server](../../docs/language-server.md) for the full capability list and schema-aware behavior.

```sh
cargo install rune-cfg --version 0.5.0
```

For local development, build the binary from this repository:

```sh
cargo build --bin rune-lsp
```

The resulting binary is:

```text
target/debug/rune-lsp
```

Neovim 0.11+ can launch the installed binary directly:

```lua
vim.lsp.config("rune_lsp", {
  cmd = { "rune-lsp" },
  filetypes = { "rune" },
  root_markers = { "schema.rune", ".rune", ".git" },
})

vim.lsp.enable("rune_lsp")
```
