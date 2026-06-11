# RUNE Language Server

RUNE includes an experimental language server binary named `rune-lsp`. It speaks LSP over stdio and can be launched by any editor client that supports custom language servers.

Editor integrations are included under [`editors/nvim/`](../editors/nvim/) for Neovim, [`editors/vim/`](../editors/vim/) for classic Vim, and [`editors/vscode/`](../editors/vscode/) for VS Code. The Vim and Neovim runtime files provide highlighting, filetype detection, and 2-space indentation. The VS Code extension provides language registration, basic TextMate highlighting, language configuration, and a client that launches `rune-lsp`.

An experimental Tree-sitter grammar is included under [`editors/tree-sitter-rune/`](../editors/tree-sitter-rune/) for higher quality highlighting, indentation, and folding. The Vim syntax files remain the stable fallback while the grammar matures.

## Capabilities

`rune-lsp` currently supports:

- full-document sync
- syntax diagnostics for opened `.rune` files
- schema parsing diagnostics for schema files
- schema validation diagnostics for config files
- schema-aware key and enum-value completion
- hover text for schema-backed fields
- document symbols for object blocks and keys
- go-to-definition from a config key to its schema field, and from `@schema` to the schema file
- find-references for a key within a document or across schema-bound configs
- rename of a key within a document or across schema-bound configs
- document formatting with 2-space block indentation
- a quickfix code action for missing `end` diagnostics
- quickfix code actions for invalid enum values and missing required fields
- quickfix code actions for simple schema type mismatches, such as quoted numbers
- schema field descriptions from leading comments in schema files
- `@schema` name/path completion from project, user, and system schema directories
- optional `@schema` references for app-provided schemas
- automatic `schema.rune` discovery from the config file directory upward to the workspace root

## Schema Support

Schemas are optional. `rune-lsp` supports three levels of editor intelligence:

- Plain RUNE: no schema required; syntax diagnostics still work.
- Local schema: place `schema.rune` beside a config file or in a parent directory for validation, completion, and hover.
- App-provided schema: add `@schema "name"` or `@schema "./path/to/schema.rune"` to use a named or explicit schema.

Explicit schema references are resolved before local discovery. Path references may be relative to the config file directory or absolute:

```rune
@schema "./schemas/app.rune"
@schema "../schemas/app.rune"
@schema "/usr/share/rune/schemas/app.rune"
```

Named schemas use the schema name plus `.rune` and search these locations:

```text
./schemas/<name>.rune
./.rune/schemas/<name>.rune
~/.config/rune/schemas/<name>.rune
/usr/local/share/rune/schemas/<name>.rune
/usr/share/rune/schemas/<name>.rune
```

For example:

```rune
@schema "stasis"
```

If an explicit schema reference cannot be resolved, diagnostics include the locations that were searched. Completion inside `@schema "..."` suggests discovered schema names plus common relative paths such as `./schema.rune` and `./schemas/`.

Schema comments immediately before fields become editor hover/completion documentation:

```rune
schema app:
  # Deployment environment for this application.
  environment enum ["dev", "staging", "production"] required

  # Server listener settings.
  server:
    # Public host name or IP address.
    host string required
    port int range 1..65535 default 8080
  end
end
```

Completion uses the active schema to suggest only fields that belong in the current object, filters fields already present in that object, and uses snippets for common value shapes such as booleans, arrays, enums, and object blocks. Hover text includes the schema source so app-provided schemas are visible from the editor.

## Installation

Install the released server binary with:

```sh
cargo install rune-cfg --version 0.5.0
```

Or run the server directly from this repository with:

```sh
cargo run --bin rune-lsp
```

Build a reusable local development binary with:

```sh
cargo build --bin rune-lsp
```

The binary will be available at:

```text
target/debug/rune-lsp
```

Check the binary with:

```sh
rune-lsp --version
rune-lsp --help
```

Use `target/debug/rune-lsp --version` when checking a local development build.

## Neovim

Install the optional syntax files:

```sh
cp -r editors/nvim/ftdetect ~/.config/nvim/
cp -r editors/nvim/syntax ~/.config/nvim/
cp -r editors/nvim/ftplugin ~/.config/nvim/
```

Then configure the LSP in Neovim 0.11+:

```lua
vim.lsp.config("rune_lsp", {
  cmd = { "rune-lsp" },
  filetypes = { "rune" },
  root_markers = { "schema.rune", ".rune", ".git" },
})

vim.lsp.enable("rune_lsp")
```

For one-off testing without a named config:

```lua
vim.lsp.start({
  name = "rune_lsp",
  cmd = { "/path/to/rune-cfg/target/debug/rune-lsp" },
  root_dir = vim.fs.root(0, { "schema.rune", ".rune", ".git" }),
})
```

## Vim

Install the optional syntax files:

```sh
cp -r editors/vim/ftdetect ~/.vim/
cp -r editors/vim/syntax ~/.vim/
cp -r editors/vim/ftplugin ~/.vim/
```

Then wire up the LSP through a client such as [vim-lsp](https://github.com/prabirshrestha/vim-lsp) or [ALE](https://github.com/dense-analysis/ale). See [`editors/vim/README.md`](../editors/vim/README.md) for a vim-lsp example.

## VS Code

Compile the extension client:

```sh
cd editors/vscode
npm install
npm run compile
```

Run the extension from VS Code's Extension Development Host. By default it launches `rune-lsp` from `PATH`; set `rune.server.path` to an absolute binary path when testing a local development build. See [`editors/vscode/README.md`](../editors/vscode/README.md) for details.

## Tree-sitter

Optional Tree-sitter grammar development:

```sh
cd editors/tree-sitter-rune
npm install
npm run generate
npm test
```

## Plain vs Schema-Aware Mode

Place a `schema.rune` next to your config file or in a parent directory, or use `@schema` to point at an app-provided schema. When a schema is available, `rune-lsp` validates the config and uses the schema for completion and hover. Without a schema, `rune-lsp` stays in plain RUNE mode and only reports syntax-level diagnostics.
