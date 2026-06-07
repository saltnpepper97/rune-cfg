# Neovim Support

This directory contains lightweight Neovim runtime files for RUNE.

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

For LSP diagnostics, build `rune-lsp` and configure your editor to launch it:

```sh
cargo build --bin rune-lsp
```

The resulting binary is:

```text
target/debug/rune-lsp
```
