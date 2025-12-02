use super::*;
use super::scanner::{bump, skip_whitespace_and_comments};

pub(super) fn next_token_with_flag(lexer: &mut Lexer, skip_newlines: bool) -> Result<Token, RuneError> {
    skip_whitespace_and_comments(lexer, skip_newlines);

    let token = match lexer.peek {
        Some('\n') => tokenize_newline(lexer),
        Some(':') => tokenize_symbol(lexer, Token::Colon),
        Some('=') => tokenize_symbol(lexer, Token::Equals),
        Some('[') => tokenize_symbol(lexer, Token::LBracket),
        Some(']') => tokenize_symbol(lexer, Token::RBracket),
        Some(',') => {
            bump(lexer);
            return next_token_with_flag(lexer, skip_newlines); // skip commas
        }
        Some('$') => tokenize_symbol(lexer, Token::Dollar),
        Some('.') => tokenize_symbol(lexer, Token::Dot),
        Some('@') => tokenize_symbol(lexer, Token::At),
        Some('r') => tokenize_regex_or_ident(lexer),
        Some('"') | Some('\'') => tokenize_string(lexer),
        Some(c) if c.is_digit(10) => tokenize_number(lexer),
        Some(c) if c.is_alphabetic() => tokenize_identifier_or_keyword(lexer),
        Some(ch) => tokenize_unexpected_char(lexer, ch),
        None => Ok(Token::Eof),
    };

    token
}

fn tokenize_newline(lexer: &mut Lexer) -> Result<Token, RuneError> {
    bump(lexer);
    Ok(Token::Newline)
}

fn tokenize_symbol(lexer: &mut Lexer, token: Token) -> Result<Token, RuneError> {
    bump(lexer);
    Ok(token)
}

fn tokenize_regex_or_ident(lexer: &mut Lexer) -> Result<Token, RuneError> {
    // Check if this is a regex literal r"..."
    let mut clone_iter = lexer.input.clone();
    let next_char = clone_iter.next();

    if next_char == Some('"') {
        tokenize_regex_literal(lexer)
    } else {
        tokenize_identifier_starting_with_r(lexer)
    }
}

fn tokenize_regex_literal(lexer: &mut Lexer) -> Result<Token, RuneError> {
    bump(lexer); // consume 'r'
    bump(lexer); // consume opening '"'

    let mut content = String::new();
    while let Some(ch) = bump(lexer) {
        if ch == '"' {
            break; // closing quote
        }

        if ch == '\\' {
            // Preserve the backslash literally in regex
            content.push('\\');
            if let Some(next_ch) = bump(lexer) {
                content.push(next_ch);
            } else {
                return Err(RuneError::UnclosedString {
                    quote: '"',
                    line: lexer.line,
                    column: lexer.column,
                    hint: Some("Trailing backslash in regex".into()),
                    code: Some(103),
                });
            }
        } else {
            content.push(ch);
        }
    }

    Ok(Token::Regex(content))
}

fn tokenize_identifier_starting_with_r(lexer: &mut Lexer) -> Result<Token, RuneError> {
    let mut ident = String::new();
    ident.push(bump(lexer).unwrap()); // consume 'r'
    
    while let Some(ch) = lexer.peek {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' { 
            ident.push(ch); 
            bump(lexer); 
        } else { 
            break; 
        }
    }
    
    Ok(Token::Ident(ident))
}

fn tokenize_string(lexer: &mut Lexer) -> Result<Token, RuneError> {
    let quote = bump(lexer).unwrap();
    let mut content = String::new();

    while let Some(ch) = lexer.peek {
        if ch == quote { 
            bump(lexer); // consume the closing quote
            break;
        }

        if ch == '\\' {
            bump(lexer); // consume '\'
            if let Some(next_ch) = bump(lexer) {
                let escaped = match next_ch {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    '$' => '$',
                    '{' => '{',
                    '}' => '}',
                    other => other,
                };
                content.push(escaped);
            } else {
                return Err(RuneError::UnclosedString {
                    quote,
                    line: lexer.line,
                    column: lexer.column,
                    hint: Some("Trailing backslash in string".into()),
                    code: Some(103),
                });
            }
        } else {
            content.push(ch);
            bump(lexer);
        }
    }

    // Check if string was properly closed
    if lexer.peek.is_none() && !content.ends_with(quote) {
        return Err(RuneError::UnclosedString {
            quote,
            line: lexer.line,
            column: lexer.column,
            hint: Some("String literal not closed".into()),
            code: Some(103),
        });
    }

    Ok(Token::String(content))
}

fn tokenize_number(lexer: &mut Lexer) -> Result<Token, RuneError> {
    let mut num = String::new();
    
    while let Some(ch) = lexer.peek {
        if ch.is_digit(10) || ch == '.' { 
            num.push(ch); 
            bump(lexer); 
        } else { 
            break; 
        }
    }
    
    num.parse::<f64>()
        .map(Token::Number)
        .map_err(|_| RuneError::TypeError {
            message: format!("Invalid number '{}'", num),
            line: lexer.line,
            column: lexer.column,
            hint: None,
            code: Some(102),
        })
}

fn tokenize_identifier_or_keyword(lexer: &mut Lexer) -> Result<Token, RuneError> {
    let mut ident = String::new();
    
    while let Some(ch) = lexer.peek {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            ident.push(ch);
            bump(lexer);
        } else { 
            break; 
        }
    }

    // Map keywords to their respective tokens
    let token = match ident.as_str() {
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "end" => Token::End,
        "gather" => Token::Gather,
        "as" => Token::As,
        "null" | "None" => Token::Null,
        _ => Token::Ident(ident),
    };

    Ok(token)
}

fn tokenize_unexpected_char(lexer: &mut Lexer, ch: char) -> Result<Token, RuneError> {
    bump(lexer);
    Err(RuneError::UnexpectedCharacter {
        character: ch,
        line: lexer.line,
        column: lexer.column,
        hint: Some("Unexpected character in input".into()),
        code: Some(104),
    })
}
