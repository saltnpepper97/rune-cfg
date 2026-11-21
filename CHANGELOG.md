# Changelog

## Unreleased

- Allow an optional block name after `end`, as in `server:` ... `end server`,
  for object blocks, schema blocks, and nested schema objects. The name must
  match the block being closed; a mismatch is a syntax error (code 219), and
  if-blocks keep a bare `end`. `rune-lsp` treats the label as part of its
  block (hover, definition, references, and rename all work from it, and
  rename updates it), and the formatter keeps labels as written. The
  tree-sitter grammar adds an `end_name` field, Vim, Neovim, Emacs, and VS
  Code highlight the label, and the Emacs grammar pin moves to the grammar
  revision that parses it. An identifier written after `end` on the same line
  is now always read as its label.
- **Breaking:** block conditionals now close with `end`, like every other
  block, instead of `endif`. `endif` is no longer a keyword, so existing
  configs must replace each `endif` with `end`. Updated the parser, LSP
  (diagnostics, completion, formatting), tree-sitter grammar, and the Vim,
  Neovim, Emacs, and VS Code integrations to match.
- Pin the Emacs mode's grammar install recipe to the `end`-closing grammar
  revision, so `rune-cfg-ts-mode-install-grammar` no longer builds a grammar
  that still expects `endif`.
- Report the correct line for keys inside or after block conditionals in
  type, validation, and lookup errors. `if`/`else`/`elseif` headers no longer
  count as part of a key's path, and an exact path match takes priority over
  an earlier field that merely shares the last segment.
- Add an Emacs Tree-sitter major mode (`rune-cfg-ts-mode`) with highlighting,
  indentation, Imenu, section navigation, and optional Eglot registration for
  `rune-lsp`, plus an Emacs CI workflow.
- Make `rune-lsp` exit as soon as it handles the `exit` notification instead
  of waiting for stdin to close, using exit code 1 when `shutdown` was never
  requested.

## 0.8.0

- Add `ParseOptions`, opt-in literal quoted references, and strict environment
  resolution to parser, string, and file-loading APIs, including nested gathers.
- Make `\$` prevent expansion in literal quoted-string mode, including arrays.
- Report duplicate keys as LSP warnings and schema validation errors without
  changing lookup behavior; exclusive conditional branches remain valid.
- Correct source indexing for multiple sibling assignments on one line and
  expose `RuneConfig::key_ranges` for consumer diagnostics.
- Preserve shell punctuation after dollar variables instead of reporting an
  identifier error. Anchor resolver diagnostics at the actual value location.
- Keep legacy quoted references and unset-variable behavior in existing APIs.
- Strict string interpolation preserves hyphen suffixes after environment names.
- Include the existing gather-aware LSP diagnostics and navigation fixes.
- Align source license headers with the GPL-3.0-or-later package license.
- Keep VS Code extension metadata in step with the crate version.
