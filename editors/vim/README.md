# Vim Support

This directory contains lightweight Vim runtime files for RUNE.

An experimental Tree-sitter grammar also exists at `editors/tree-sitter-rune/`, and Neovim runtime files live at `editors/nvim/`. Use these Vim runtime files for classic Vim (or Neovim) as the stable fallback while the Tree-sitter grammar matures.

They provide:

- `*.rune` filetype detection
- regex-based syntax highlighting
- 2-space RUNE indentation

Install them by copying the runtime directories into your Vim config:

```sh
cp -r editors/vim/ftdetect ~/.vim/
cp -r editors/vim/syntax ~/.vim/
cp -r editors/vim/ftplugin ~/.vim/
```

The syntax file distinguishes:

- top-level object blocks, such as `app:`
- nested object blocks, such as `server:`
- assignment keys, such as `port`
- closing block keywords, such as `end` and `endif`
- strings, regex literals, numbers, booleans, nulls, metadata, schema keywords, and `$env`/`$sys` references

For LSP diagnostics, completion, hover, navigation, rename, and formatting, install `rune-lsp` and configure your editor to launch it:

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

In classic Vim, wire the binary up through an LSP client such as
[vim-lsp](https://github.com/prabirshrestha/vim-lsp) or
[ALE](https://github.com/dense-analysis/ale). For example, with vim-lsp:

```vim
if executable('rune-lsp')
  autocmd User lsp_setup call lsp#register_server({
    \ 'name': 'rune-lsp',
    \ 'cmd': {server_info->['rune-lsp']},
    \ 'allowlist': ['rune'],
    \ })
endif
```

The server speaks LSP over stdio. Use `target/debug/rune-lsp` in the `cmd` value when testing a local development build instead of an installed binary.
