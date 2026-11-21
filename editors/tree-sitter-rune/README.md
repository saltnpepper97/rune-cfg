# tree-sitter-runecfg

Experimental Tree-sitter grammar for RUNE configuration files.

This grammar is intentionally kept under `editors/tree-sitter-rune/` while RUNE's syntax and editor tooling are still evolving. It can be split into a dedicated `tree-sitter-runecfg` repository later.

Current scope:

- metadata declarations like `@schema "app"`
- `gather "file.rune" as alias`
- assignments with optional `=`
- object and schema blocks closed by `end`, optionally labeled `end <name>`
- block conditionals closed by `end`
- schema blocks, schema objects, schema fields, enum types, ranges, defaults, and required fields
- strings, regex literals, numbers, booleans, nulls, arrays, references, and comments

Generate and test with the Tree-sitter CLI:

```sh
npm install
npm run generate
npm test
```

Neovim users can keep using the Vim syntax files under `editors/nvim/` as a fallback. Once this grammar is generated and installed locally, it can provide higher quality highlighting, indentation, and folding.

For broader editor and LSP setup, see [Language Server](../../docs/language-server.md).
