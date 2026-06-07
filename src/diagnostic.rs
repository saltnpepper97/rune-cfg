// Author: Dustin Pilgrim
// License: MIT

/// Severity values are intentionally LSP-shaped so diagnostics can be mapped
/// directly when a language server is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuneDiagnostic {
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub range: Option<SourceRange>,
    pub code: Option<u32>,
    pub hint: Option<String>,
}

impl RuneDiagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: DiagnosticSeverity::Error,
            range: None,
            code: None,
            hint: None,
        }
    }

    pub fn with_line(mut self, line: usize, column: usize) -> Self {
        if line > 0 {
            self = self.with_range(line, column, column.saturating_add(1));
        }
        self
    }

    pub fn with_range(mut self, line: usize, start_column: usize, end_column: usize) -> Self {
        if line > 0 {
            self.range = Some(SourceRange {
                start: SourcePosition {
                    line,
                    column: start_column,
                },
                end: SourcePosition {
                    line,
                    column: end_column.max(start_column.saturating_add(1)),
                },
            });
        }
        self
    }

    pub fn with_code(mut self, code: u32) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}
