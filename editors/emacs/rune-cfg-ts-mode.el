;;; rune-cfg-ts-mode.el --- RUNE configuration files -*- lexical-binding: t; -*-

;; Copyright (C) 2026 Ev Dolzhenko
;; Author: Ev Dolzhenko <dolzenko@gmail.com>
;; Version: 0.1.0
;; Package-Requires: ((emacs "31.1"))
;; Keywords: languages, tools
;; URL: https://github.com/saltnpepper97/rune-cfg
;; SPDX-License-Identifier: GPL-3.0-or-later
;;
;; This program is free software: you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.
;;
;; This program is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU General Public License for more details.
;;
;; You should have received a copy of the GNU General Public License
;; along with this program.  If not, see <https://www.gnu.org/licenses/>.

;;; Commentary:
;; A Tree-sitter major mode for the rune-cfg configuration language.
;; Install the grammar separately with `rune-cfg-ts-mode-install-grammar'.
;; Optional Eglot integration uses rune-lsp; run `eglot' to connect.
;; Opening a file never downloads a grammar or starts a language server.

;;; Code:

(require 'treesit)

(defvar eglot-server-programs)

(defgroup rune-cfg nil
  "Editing RUNE configuration files."
  :group 'languages)

(defcustom rune-cfg-ts-mode-indent-offset 2
  "Number of spaces per RUNE nesting level."
  :type 'natnum
  :safe 'natnump
  :group 'rune-cfg)

;; Update this tested grammar revision together with the mode tests.
(add-to-list 'treesit-language-source-alist
             '(runecfg "https://github.com/saltnpepper97/rune-cfg"
                       :commit "321c7d133640c1851b5c53d0188b1b5bcc1b9a15"
                       :source-dir "editors/tree-sitter-rune/src")
             t)

(defun rune-cfg-ts-mode-install-grammar ()
  "Build the pinned upstream RUNE grammar for this Emacs installation."
  (interactive)
  (treesit-install-language-grammar 'runecfg))

(defvar rune-cfg-ts-mode-syntax-table
  (let ((table (make-syntax-table)))
    (modify-syntax-entry ?# "<" table)
    (modify-syntax-entry ?\n ">" table)
    (modify-syntax-entry ?\' "\"" table)
    (modify-syntax-entry ?_ "w" table)
    (modify-syntax-entry ?- "_" table)
    table)
  "Syntax table for `rune-cfg-ts-mode'.")

(defun rune-cfg-ts-mode--section-name (node)
  "Return the name of the RUNE section NODE."
  (treesit-node-text (treesit-node-child-by-field-name node "name") t))

;;;###autoload
(define-derived-mode rune-cfg-ts-mode prog-mode "RUNE"
  "Edit rune-cfg files with structural highlighting and indentation."
  :group 'rune-cfg
  :syntax-table rune-cfg-ts-mode-syntax-table
  (unless (treesit-ready-p 'runecfg t)
    (user-error "Install the grammar with M-x rune-cfg-ts-mode-install-grammar"))
  (setq treesit-primary-parser (treesit-parser-create 'runecfg))
  (setq-local indent-tabs-mode nil)
  (setq-local comment-start "# ")
  (setq-local comment-end "")
  (setq-local comment-start-skip "#+[ \t]*")
  (setq-local treesit-simple-indent-rules
              '((runecfg
                 ((parent-is "source_file") column-0 0)
                 ((node-is "end\\|endif\\|else\\|elseif\\|else-if\\|]") parent-bol 0)
                 ((parent-is "object_block\\|schema_block\\|schema_object\\|if_block\\|array")
                  parent-bol rune-cfg-ts-mode-indent-offset))))
  (setq-local treesit-font-lock-settings
              (treesit-font-lock-rules
               :language 'runecfg :feature 'comment
               '((comment) @font-lock-comment-face)
               :language 'runecfg :feature 'string
               '([(string) (regex)] @font-lock-string-face)
               :language 'runecfg :feature 'keyword
               '(["end" "if" "else" "elseif" "else-if" "endif"
                  "gather" "as" "with" "schema" "required" "default" "range" "enum"]
                 @font-lock-keyword-face)
               :language 'runecfg :feature 'constant
               '([(boolean) (null) (number) (range)] @font-lock-constant-face)
               :language 'runecfg :feature 'name
               '((metadata name: (identifier) @font-lock-preprocessor-face)
                 (object_block name: (identifier) @font-lock-type-face)
                 (schema_block name: (identifier) @font-lock-type-face)
                 (schema_object name: (identifier) @font-lock-type-face)
                 (assignment key: (identifier) @font-lock-property-name-face)
                 (schema_field name: (identifier) @font-lock-property-name-face)
                 (schema_primitive) @font-lock-type-face
                 (reference) @font-lock-variable-use-face)))
  (setq-local treesit-font-lock-feature-list
              '((comment string) (keyword constant) (name)))
  (setq-local treesit-defun-type-regexp "\\`\\(?:object_block\\|schema_block\\)\\'")
  (setq-local treesit-defun-name-function #'rune-cfg-ts-mode--section-name)
  (setq-local treesit-simple-imenu-settings
              '(("Section" "\\`\\(?:object_block\\|schema_block\\|schema_object\\)\\'"
                 nil rune-cfg-ts-mode--section-name)))
  (treesit-major-mode-setup))

;; Register the server without loading Eglot or starting any processes.
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '((rune-cfg-ts-mode :language-id "runecfg") . ("rune-lsp")) t))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.rune\\'" . rune-cfg-ts-mode))

(provide 'rune-cfg-ts-mode)
;;; rune-cfg-ts-mode.el ends here
