# Changelog

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
