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
}

impl fmt::Display for RuneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuneError::SyntaxError { message, line, column, hint, code } =>
                write!(f, "[RUNE] Syntax Error at {}:{}: {}{}{}", 
                    line, column, message,
                    hint.as_ref().map_or(String::new(), |h| format!(" Hint: {}", h)),
                    code.map_or(String::new(), |c| format!(" Code: {}", c))
                ),
            RuneError::InvalidToken { token, line, column, hint, code } =>
                write!(f, "[RUNE] Invalid Token '{}' at {}:{}{}{}", 
                    token, line, column,
                    hint.as_ref().map_or(String::new(), |h| format!(" Hint: {}", h)),
                    code.map_or(String::new(), |c| format!(" Code: {}", c))
                ),
            RuneError::UnexpectedEof { message, line, column, hint, code } =>
                write!(f, "[RUNE] Unexpected EOF at {}:{}: {}{}{}", 
                    line, column, message,
                    hint.as_ref().map_or(String::new(), |h| format!(" Hint: {}", h)),
                    code.map_or(String::new(), |c| format!(" Code: {}", c))
                ),
            RuneError::TypeError { message, line, column, hint, code } =>
                write!(f, "[RUNE] Type Error at {}:{}: {}{}{}", 
                    line, column, message,
                    hint.as_ref().map_or(String::new(), |h| format!(" Hint: {}", h)),
                    code.map_or(String::new(), |c| format!(" Code: {}", c))
                ),
            RuneError::UnclosedString { quote, line, column, hint, code } =>
                write!(f, "[RUNE] Unclosed string starting with '{}' at {}:{}{}{}", 
                    quote, line, column,
                    hint.as_ref().map_or(String::new(), |h| format!(" Hint: {}", h)),
                    code.map_or(String::new(), |c| format!(" Code: {}", c))
                ),
            RuneError::UnexpectedCharacter { character, line, column, hint, code } =>
                write!(f, "[RUNE] Unexpected character '{}' at {}:{}{}{}", 
                    character, line, column,
                    hint.as_ref().map_or(String::new(), |h| format!(" Hint: {}", h)),
                    code.map_or(String::new(), |c| format!(" Code: {}", c))
                ),
            RuneError::FileError { message, path, hint, code } =>
                write!(f, "[RUNE] File Error '{}': {}{}{}", 
                    path, message,
                    hint.as_ref().map_or(String::new(), |h| format!(" Hint: {}", h)),
                    code.map_or(String::new(), |c| format!(" Code: {}", c))
                ),
            RuneError::RuntimeError { message, hint, code } =>
                write!(f, "[RUNE] Runtime Error: {}{}{}", 
                    message,
                    hint.as_ref().map_or(String::new(), |h| format!(" Hint: {}", h)),
                    code.map_or(String::new(), |c| format!(" Code: {}", c))
                ),
        }
    }
}

impl std::error::Error for RuneError {}
