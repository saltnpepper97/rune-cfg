// Author: Dustin Pilgrim
// License: MIT

use std::fmt;

/// The main error type for RUNE parsing and lexing.
#[derive(Debug, Clone, PartialEq)]
pub enum RuneError {
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
        hint: Option<String>,
        code: Option<u32>,
    },
    InvalidToken {
        token: String,
        line: usize,
        column: usize,
        hint: Option<String>,
        code: Option<u32>,
    },
    UnexpectedEof {
        message: String,
        line: usize,
        column: usize,
        hint: Option<String>,
        code: Option<u32>,
    },
    TypeError {
        message: String,
        line: usize,
        column: usize,
        hint: Option<String>,
        code: Option<u32>,
    },
    /// Raised when a string literal is not closed.
    UnclosedString {
        quote: char,
        line: usize,
        column: usize,
        hint: Option<String>,
        code: Option<u32>,
    },
    /// Raised for unexpected characters or tokens.
    UnexpectedCharacter {
        character: char,
        line: usize,
        column: usize,
        hint: Option<String>,
        code: Option<u32>,
    },
    FileError {
        message: String,
        path: String,
        hint: Option<String>,
        code: Option<u32>,
    },
    /// Raised for runtime issues, such as missing environment variables.
    RuntimeError {
        message: String,
        hint: Option<String>,
        code: Option<u32>,
    },
    ValidationError {
        message: String,
        line: usize,
        column: usize,
        hint: Option<String>,
        code: Option<u32>,
    },
}

impl fmt::Display for RuneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuneError::SyntaxError { message, line, hint, code, .. } => {
                if *line > 0 {
                    write!(f, "[RUNE] Syntax Error at line {}: {}", line, message)?;
                } else {
                    write!(f, "[RUNE] Syntax Error: {}", message)?;
                }
                if let Some(h) = hint {
                    write!(f, " Hint: {}", h)?;
                }
                if let Some(c) = code {
                    write!(f, " Code: {}", c)?;
                }
                Ok(())
            }
            RuneError::InvalidToken { token, line, hint, code, .. } => {
                if *line > 0 {
                    write!(f, "[RUNE] Invalid Token '{}' at line {}", token, line)?;
                } else {
                    write!(f, "[RUNE] Invalid Token '{}'", token)?;
                }
                if let Some(h) = hint {
                    write!(f, " Hint: {}", h)?;
                }
                if let Some(c) = code {
                    write!(f, " Code: {}", c)?;
                }
                Ok(())
            }
            RuneError::UnexpectedEof { message, line, hint, code, .. } => {
                if *line > 0 {
                    write!(f, "[RUNE] Unexpected EOF at line {}: {}", line, message)?;
                } else {
                    write!(f, "[RUNE] Unexpected EOF: {}", message)?;
                }
                if let Some(h) = hint {
                    write!(f, " Hint: {}", h)?;
                }
                if let Some(c) = code {
                    write!(f, " Code: {}", c)?;
                }
                Ok(())
            }
            RuneError::TypeError { message, line, hint, code, .. } => {
                if *line > 0 {
                    write!(f, "[RUNE] Type Error at line {}: {}", line, message)?;
                } else {
                    write!(f, "[RUNE] Type Error: {}", message)?;
                }
                if let Some(h) = hint {
                    write!(f, " Hint: {}", h)?;
                }
                if let Some(c) = code {
                    write!(f, " Code: {}", c)?;
                }
                Ok(())
            }
            RuneError::UnclosedString { quote, line, hint, code, .. } => {
                if *line > 0 {
                    write!(f, "[RUNE] Unclosed string starting with '{}' at line {}", quote, line)?;
                } else {
                    write!(f, "[RUNE] Unclosed string starting with '{}'", quote)?;
                }
                if let Some(h) = hint {
                    write!(f, " Hint: {}", h)?;
                }
                if let Some(c) = code {
                    write!(f, " Code: {}", c)?;
                }
                Ok(())
            }
            RuneError::UnexpectedCharacter { character, line, hint, code, .. } => {
                if *line > 0 {
                    write!(f, "[RUNE] Unexpected character '{}' at line {}", character, line)?;
                } else {
                    write!(f, "[RUNE] Unexpected character '{}'", character)?;
                }
                if let Some(h) = hint {
                    write!(f, " Hint: {}", h)?;
                }
                if let Some(c) = code {
                    write!(f, " Code: {}", c)?;
                }
                Ok(())
            }
            RuneError::FileError { message, path, hint, code } => {
                write!(f, "[RUNE] File Error '{}': {}", path, message)?;
                if let Some(h) = hint {
                    write!(f, " Hint: {}", h)?;
                }
                if let Some(c) = code {
                    write!(f, " Code: {}", c)?;
                }
                Ok(())
            }
            RuneError::RuntimeError { message, hint, code } => {
                write!(f, "[RUNE] Runtime Error: {}", message)?;
                if let Some(h) = hint {
                    write!(f, " Hint: {}", h)?;
                }
                if let Some(c) = code {
                    write!(f, " Code: {}", c)?;
                }
                Ok(())
            }
            RuneError::ValidationError { message, hint, code, .. } => {
                write!(f, "{}", message)?;
                if let Some(h) = hint {
                    write!(f, "\nHint: {}", h)?;
                }
                if let Some(c) = code {
                    write!(f, " [E{}]", c)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for RuneError {}
