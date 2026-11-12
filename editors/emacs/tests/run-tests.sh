#!/bin/sh
set -eu

editor_dir=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
grammar_source="$editor_dir/../tree-sitter-rune/src"
test_dir=$(mktemp -d "${TMPDIR:-/tmp}/runecfg-emacs.XXXXXX")
trap 'rm -rf -- "$test_dir"' EXIT HUP INT TERM
export RUNE_TEST_GRAMMAR_DIR="$test_dir"

"${CC:-cc}" -fPIC -shared -I "$grammar_source" "$grammar_source/parser.c" \
  -o "$test_dir/libtree-sitter-runecfg.so"

"${EMACS:-emacs}" -Q --batch \
  --eval '(setq byte-compile-error-on-warn t
                byte-compile-dest-file-function
                (lambda (_file)
                  (expand-file-name "rune-cfg-ts-mode.elc"
                                    (getenv "RUNE_TEST_GRAMMAR_DIR"))))' \
  -f batch-byte-compile "$editor_dir/rune-cfg-ts-mode.el"

"${EMACS:-emacs}" -Q --batch \
  --eval '(setq treesit-extra-load-path (list (getenv "RUNE_TEST_GRAMMAR_DIR")))' \
  -L "$editor_dir" -l "$editor_dir/rune-cfg-ts-mode.el" \
  -l "$editor_dir/tests/rune-cfg-ts-mode-tests.el" \
  -f ert-run-tests-batch-and-exit
