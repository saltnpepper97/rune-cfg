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
- document symbols for object blocks and keys, plus one symbol per `gather` statement
- go-to-definition from a config key to its schema field, from a `gather` path to the gathered file, and from `@schema` to the schema file
- go-to-definition through a reference such as `shared.name` or an include-supplied key
- find-references for a key within a document, across schema-bound configs, and across `gather` links
- rename of a key within a document or across schema-bound configs
- diagnostics and navigation for `gather` imports, resolved the way the runtime loads them
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

## Gather Imports

`gather` statements are resolved the way the runtime resolves them, so the editor sees the config the program would load rather than the open buffer alone.

```rune
# an include: the gathered file's keys are merged into this document
# and it also stays reachable under its default alias
#
gather "defaults.rune"

# a namespaced import: the keys stay behind the alias
gather "shared.rune" as shared

app:
  theme defaults.theme
  name shared.name
end
```

- Paths resolve relative to the file that writes the statement, with `~/` expanded, and nested gathers resolve relative to the file that declares them.
- An unaliased gather is an **include**: its keys merge into the importing document and override it, in source order, exactly as the runtime merges them.
- An explicitly aliased gather is a **namespace** only. Its keys are never merged into the importing document, and a bare reference to one of them is not resolved.
- A gathered file that is not there is skipped silently, exactly as the runtime skips it: no "file not found" diagnostic is published, though the schema consequences of the missing keys are.
- A gathered file that is there but cannot be read is reported as a load error on the importing document's own `gather` path.
- An imported file that does not parse is reported on the `gather` path that reached it, as the first hop from the importing document, and the message names the failing file and its parser position.
- Cycles terminate: every resolved path is visited once, and no diagnostic is published about them.

Diagnostics follow the values. A schema diagnostic about a key an include supplies is anchored on the statement that brought the key in, marked with `Provided by gather "…"`, and carries the imported declaration as related information. Quick fixes that would edit this buffer are not offered for those diagnostics, because the value is not written here.

Navigation follows the same view:

- Definition on a `gather` path opens the gathered file; a path that resolves to nothing answers `null`.
- Definition on `alias.key` or `alias.object.key` opens the exact key in the gathered file.
- Definition on a bare reference opens the declaration that supplies it, whether that is a local key or one an include provides.
- Hover on a `gather` statement reports the resolved file, the alias, and whether it includes or namespaces the file. Hover on an imported reference reports the resolved path, the source file, and the kind of declaration — never the value.
- Completion after `alias.` suggests the gathered file's own keys, and keys an unaliased gather supplies are treated as already present so they are not suggested twice.
- Document symbols include one module symbol per `gather` statement, named by its alias, with the target path as detail. The gathered file's keys are never inlined into the importing document.

### Open Buffers and Watched Files

An open buffer always wins over the copy on disk, for the file it belongs to and for anything that gathers it. Opening, changing, or closing the buffer of a gathered file revalidates the documents that gather it.

Disk changes are followed transitively: editing, deleting, or recreating a file any number of `gather` hops away republishes the configs that reach it, and no config that does not reach it is republished. A `gather` that names a file outside the workspace, or below an excluded directory, is followed too — the import is what makes that file part of the project, and no directory is scanned because of it.

## Installation

Install the released server binary with:

```sh
cargo install rune-cfg --version 0.8.0
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
vim.lsp.config("runecfg_lsp", {
  cmd = { "rune-lsp" },
  filetypes = { "runecfg" },
  root_markers = { "schema.rune", ".rune", ".git" },
})

vim.lsp.enable("runecfg_lsp")
```

For one-off testing without a named config:

```lua
vim.lsp.start({
  name = "runecfg_lsp",
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

Run the extension from VS Code's Extension Development Host. By default it launches `rune-lsp` from `PATH`; set `runecfg.server.path` to an absolute binary path when testing a local development build. See [`editors/vscode/README.md`](../editors/vscode/README.md) for details.

## Packaging and Releases

The VS Code extension does not bundle `rune-lsp`. It starts `rune-lsp` over stdio, resolving the binary from `PATH` by default, or from the `runecfg.server.path` setting when that is set.

The extension and the crate are released together. Both are `0.8.0`: extension 0.8.0 is built for the `rune-lsp` from crate 0.8.0. Mismatched versions are not guaranteed to work together.

Before a release, run the full checklist from the repository root, in order:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets

npm --prefix editors/vscode ci
npm --prefix editors/vscode run compile
npm --prefix editors/vscode run package:vsix
unzip -Z1 editors/vscode/rune-cfg.vsix | grep -Fx 'extension/out/extension.js'
```

The final command must print `extension/out/extension.js`, which confirms the packaged VSIX contains the extension entry point.

After those checks pass, bump the versions together in a single commit:

- `Cargo.toml`
- the root package entry in `Cargo.lock`
- `editors/vscode/package.json`
- the root versions in `editors/vscode/package-lock.json`
- the versioned install examples in `README.md`, `docs/language-server.md`, and the editor READMEs

CI runs the Rust checks and the VS Code packaging checks on every push and pull request, but it does not publish anything. Publishing and tagging stay manual.

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

Gather imports work in both modes: an unaliased include is validated when a schema is available, and a gathered file is parsed and reported on even when no schema exists.
