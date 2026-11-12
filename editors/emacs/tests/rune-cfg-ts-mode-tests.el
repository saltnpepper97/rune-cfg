;;; rune-cfg-ts-mode-tests.el --- RUNE checks -*- lexical-binding: t; -*-

(require 'ert)
(require 'rune-cfg-ts-mode)

(defconst rune-cfg-test--fixture-file
  (expand-file-name "fixtures/idle.rune"
                    (file-name-directory (or load-file-name buffer-file-name))))

(ert-deftest rune-cfg-test-fixture-parses-and-keeps-indentation ()
  (with-temp-buffer
    (insert-file-contents rune-cfg-test--fixture-file)
    (rune-cfg-ts-mode)
    (should-not (treesit-node-check (treesit-buffer-root-node) 'has-error))
    (let ((original (buffer-string)))
      (indent-region (point-min) (point-max))
      (should (equal (buffer-string) original)))))

(ert-deftest rune-cfg-test-indentation-respects-strings-comments-and-branches ()
  (with-temp-buffer
    (insert "app:\n"
            "# comment with end: [\n"
            "if environment = \"home\":\n"
            "command \"echo end: [ # not a comment\"\n"
            "patterns [\n"
            "r\"steam_app_.*\"\n"
            "[\"nested\"]\n"
            "]\n"
            "else:\n"
            "debug false\n"
            "endif\n"
            "child:\n"
            "path '/tmp/#file'\n"
            "end\n"
            "end\n")
    (rune-cfg-ts-mode)
    (should-not (treesit-node-check (treesit-buffer-root-node) 'has-error))
    (indent-region (point-min) (point-max))
    (should (equal (buffer-string)
                   (concat "app:\n"
                           "  # comment with end: [\n"
                           "  if environment = \"home\":\n"
                           "    command \"echo end: [ # not a comment\"\n"
                           "    patterns [\n"
                           "      r\"steam_app_.*\"\n"
                           "      [\"nested\"]\n"
                           "    ]\n"
                           "  else:\n"
                           "    debug false\n"
                           "  endif\n"
                           "  child:\n"
                           "    path '/tmp/#file'\n"
                           "  end\n"
                           "end\n")))))

(ert-deftest rune-cfg-test-font-lock ()
  (with-temp-buffer
    (insert "@description \"Test\"\n"
            "# comment\n"
            "default:\n"
            "  command \"echo # true end\"\n"
            "  pattern r\"steam_app_.*\"\n"
            "  enabled true\n"
            "  timeout 300\n"
            "  fallback None\n"
            "  delay lock_after\n"
            "end\n")
    (rune-cfg-ts-mode)
    (font-lock-ensure)
    (dolist (entry '(("description" . font-lock-preprocessor-face)
                     ("# comment" . font-lock-comment-face)
                     ("default" . font-lock-type-face)
                     ("command" . font-lock-property-name-face)
                     ("# true end" . font-lock-string-face)
                     ("steam_app_.*" . font-lock-string-face)
                     ("enabled true" . font-lock-property-name-face)
                     ("true\n" . font-lock-constant-face)
                     ("300" . font-lock-constant-face)
                     ("None" . font-lock-constant-face)
                     ("lock_after" . font-lock-variable-use-face)
                     ("end\n" . font-lock-keyword-face)))
      (goto-char (point-min))
      (search-forward (car entry))
      (should (eq (get-text-property (match-beginning 0) 'face) (cdr entry))))))

(ert-deftest rune-cfg-test-imenu-sections ()
  (with-temp-buffer
    (insert-file-contents rune-cfg-test--fixture-file)
    (rune-cfg-ts-mode)
    (let ((index (treesit-simple-imenu)))
      (should (assoc "default" (cdr (assoc "Section" index)))))))

(ert-deftest rune-cfg-test-schema-parsing-and-indentation ()
  (with-temp-buffer
    (insert "schema app:\n"
            "  environment enum [\"home\", \"work\"] required\n"
            "  server:\n"
            "    port int range 1..65535 default 8080\n"
            "    labels [string]\n"
            "  end\n"
            "end\n")
    (rune-cfg-ts-mode)
    (font-lock-ensure)
    (should-not (treesit-node-check (treesit-buffer-root-node) 'has-error))
    (let ((original (buffer-string)))
      (indent-region (point-min) (point-max))
      (should (equal (buffer-string) original)))))

(ert-deftest rune-cfg-test-auto-mode ()
  (with-temp-buffer
    (let ((buffer-file-name (expand-file-name "example.rune")))
      (set-auto-mode)
      (should (eq major-mode 'rune-cfg-ts-mode)))))

(ert-deftest rune-cfg-test-eglot-registration ()
  (require 'eglot)
  (with-temp-buffer
    (rune-cfg-ts-mode)
    (let ((entry (eglot--lookup-mode major-mode)))
      (should (equal (car entry) '((rune-cfg-ts-mode . "runecfg"))))
      (should (equal (cdr entry) '("rune-lsp")))))
  ;; Loading the library again must not duplicate either registration.
  (load "rune-cfg-ts-mode" nil t)
  (should (= 1 (cl-count '("\\.rune\\'" . rune-cfg-ts-mode) auto-mode-alist
                         :test #'equal)))
  (should (= 1 (cl-count '((rune-cfg-ts-mode :language-id "runecfg") . ("rune-lsp"))
                         eglot-server-programs :test #'equal))))

(ert-deftest rune-cfg-test-reload-preserves-user-recipes ()
  (require 'eglot)
  (let ((treesit-language-source-alist '((runecfg "/custom/grammar")))
        (eglot-server-programs
         '(((rune-cfg-ts-mode :language-id "runecfg") . ("custom-rune-lsp")))))
    (load "rune-cfg-ts-mode" nil t)
    (should (equal (alist-get 'runecfg treesit-language-source-alist)
                   '("/custom/grammar")))
    (should (equal (cdr (eglot--lookup-mode 'rune-cfg-ts-mode))
                   '("custom-rune-lsp")))))

;; Opt-in integration test: keep the default suite independent of a server build.
(when (getenv "RUNE_LSP_TEST_BINARY")
  (ert-deftest rune-cfg-test-eglot-server-symbols ()
    (require 'eglot)
    (let* ((directory (make-temp-file "runecfg-eglot-" t))
           (file (expand-file-name "idle.rune" directory))
           (binary (getenv "RUNE_LSP_TEST_BINARY"))
           (eglot-server-programs
            `(((rune-cfg-ts-mode :language-id "runecfg") . (,binary))))
           (eglot-sync-connect 10)
           buffer server)
      (unwind-protect
          (progn
            (copy-file rune-cfg-test--fixture-file file)
            (setq buffer (find-file-noselect file))
            (with-current-buffer buffer
              (apply #'eglot (eglot--guess-contact))
              (setq server (eglot-current-server))
              (should (eglot-managed-p))
              (let ((symbols (jsonrpc-request
                              server :textDocument/documentSymbol
                              (list :textDocument (eglot--TextDocumentIdentifier))
                              :timeout 5)))
                (should (cl-find "default" symbols
                                 :key (lambda (symbol) (plist-get symbol :name))
                                 :test #'equal)))))
        (when server (eglot-shutdown server nil 5))
        (when (buffer-live-p buffer) (kill-buffer buffer))
        (delete-directory directory t)))))

;;; rune-cfg-ts-mode-tests.el ends here
