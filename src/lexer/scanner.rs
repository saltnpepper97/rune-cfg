use super::*;

/// Advance the character iterator and update line/column tracking
pub(super) fn bump(lexer: &mut Lexer) -> Option<char> {
    let curr = lexer.peek;
    if let Some(c) = curr {
        if c == '\n' {
            lexer.line += 1;
            lexer.column = 0;
        } else {
            lexer.column += 1;
        }
    }
    lexer.peek = lexer.input.next();
    curr
}

/// Skip whitespace and comments
pub(super) fn skip_whitespace_and_comments(lexer: &mut Lexer, skip_newlines: bool) {
    while let Some(c) = lexer.peek {
        match c {
            ' ' | '\t' => { 
                bump(lexer); 
            }
            '\n' if skip_newlines => { 
                bump(lexer); 
            }
            '\n' => break,
            '#' => { 
                // Skip comment until end of line
                while let Some(ch) = bump(lexer) { 
                    if ch == '\n' { 
                        break; 
                    } 
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
