# Emacs support

`rune-cfg-ts-mode` is a Tree-sitter major mode for RUNE configuration files.
It requires Emacs 31.1 or later with Tree-sitter support, Git, and a C compiler.

The mode provides:

- `*.rune` file detection
- structural highlighting for comments, metadata, keys, sections, values,
  references, and schema declarations
- configurable indentation (two spaces by default), including arrays and
  conditional branches
- `#` comment commands
- section navigation with `C-M-a` / `C-M-e` and Imenu
- optional Eglot registration for `rune-lsp`

It uses the [upstream RUNE grammar](../tree-sitter-rune/), which is experimental.
Opening files does not download grammars or start a language server.

## Installation

Add this directory to `load-path`, autoload the mode, and associate `.rune`:

```elisp
(add-to-list 'load-path "/path/to/rune-cfg/editors/emacs")
(autoload 'rune-cfg-ts-mode "rune-cfg-ts-mode" nil t)
(add-to-list 'auto-mode-alist '("\\.rune\\'" . rune-cfg-ts-mode))
```

Once, run `M-x load-library RET rune-cfg-ts-mode RET`, then
`M-x rune-cfg-ts-mode-install-grammar`. This builds the pinned grammar revision
with Emacs's native installer into `tree-sitter/` below `user-emacs-directory`.
It does not require Node.js or the Tree-sitter CLI: generated parser sources
are included in the repository.

Use `M-x rune-cfg-ts-mode` to activate the mode in an already-open buffer.
Customize `rune-cfg-ts-mode-indent-offset` to change the indentation width.

The grammar recipe pins the tested upstream revision. To test a different
grammar, customize `treesit-language-source-alist` **after** loading the mode,
then rerun `M-x treesit-install-language-grammar RET runecfg RET` and restart
Emacs to load the rebuilt library.

## Language server

Install the matching server release:

```sh
cargo install rune-cfg --version 0.8.0
```

With `rune-lsp` on `PATH`, run `M-x eglot` in a RUNE buffer. The mode registers
the server and its `runecfg` language ID without requiring Eglot at startup.
To connect automatically, opt in with:

```elisp
(add-hook 'rune-cfg-ts-mode-hook #'eglot-ensure)
```

Eglot supplies diagnostics, completion, hover, definitions, references, rename,
formatting, and code actions through standard Emacs commands. Schema-aware
completion and validation require a discovered or explicitly selected schema;
syntax diagnostics and import navigation also work without one. See
[Language Server](../../docs/language-server.md) for capabilities and schemas.

## Tests

From the repository root:

```sh
editors/emacs/tests/run-tests.sh
```

The runner builds the bundled grammar and byte-compiles the mode with warnings
treated as errors in a temporary directory, then runs ERT. No network access or
previous grammar installation is needed. Override `EMACS`, `CC`, or `TMPDIR`
when needed. Tests use their own fixtures and cover parsing, highlighting,
indentation, Imenu, automatic detection, and Eglot registration. They do not
load personal Emacs configuration, access a running daemon, or start a language
server. Generated libraries and bytecode are removed when the runner exits.

To also test a real Eglot connection and document-symbol request, build the
server and opt in with its absolute path:

```sh
cargo build --locked --bin rune-lsp
RUNE_LSP_TEST_BINARY="$PWD/target/debug/rune-lsp" editors/emacs/tests/run-tests.sh
```

With `rune-lsp` 0.8.0, the opt-in test can report an Eglot shutdown warning:
the server stays alive after the `exit` notification until stdin closes, so
Eglot forcefully terminates it. This does not affect the connection or symbol
request checked by the test.
