// Author: Dustin Pilgrim
// License: MIT

use super::*;

/// Advance the character iterator and update byte/line/column tracking
pub(super) fn bump(lexer: &mut Lexer) -> Option<char> {
    let curr = lexer.peek;
    if let Some(c) = curr {
        if c == '\n' {
            lexer.line += 1;
            lexer.column = 0;
        } else {
            lexer.column += 1;
        }
        lexer.offset += c.len_utf8();
    }
    lexer.peek = lexer.input.next();
    curr
}

/// Skip whitespace and comments
pub(super) fn skip_whitespace_and_comments(lexer: &mut Lexer, skip_newlines: bool) {
    while let Some(c) = lexer.peek {
        match c {
            ' ' | '\t' | '\r' => {
                // Treat the carriage return in CRLF as layout; the following
                // `\\n` remains the single significant line boundary.
                bump(lexer);
            }
            '\n' if skip_newlines => {
                bump(lexer);
            }
            '\n' => break,
            '#' => {
                // Preserve the newline as a statement boundary outside arrays.
                while lexer.peek.is_some_and(|ch| ch != '\n') {
                    bump(lexer);
                }
            }
            _ => break,
        }
    }
}

/// Peek at the current character without consuming it
#[allow(dead_code)]
pub(super) fn peek_char(lexer: &Lexer) -> Option<char> {
    lexer.peek
}
