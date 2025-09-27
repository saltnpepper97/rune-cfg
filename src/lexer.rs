use std::str::Chars;
use crate::RuneError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    String(String),
    Regex(String),
    Number(f64),
    Bool(bool),
    Null,

    Colon, Equals, LBracket, RBracket, End,
    Dollar, Dot, At, Gather, As,

    Newline,
    Eof,
}

pub struct Lexer<'a> {
    input: Chars<'a>,
    peek: Option<char>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input: input.chars(),
            peek: None,
            line: 1,
            column: 0,
        };
        lexer.peek = lexer.input.next();
        lexer
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    fn bump(&mut self) -> Option<char> {
        let curr = self.peek;
        if let Some(c) = curr {
            if c == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
        self.peek = self.input.next();
        curr
    }

    fn skip_whitespace_and_comments(&mut self, skip_newlines: bool) {
        while let Some(c) = self.peek {
            match c {
                ' ' | '\t' => { self.bump(); }
                '\n' if skip_newlines => { self.bump(); }
                '\n' => break,
                '#' => { while let Some(ch) = self.bump() { if ch == '\n' { break; } } }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token, RuneError> {
        self.next_token_with_flag(false)
    }

    pub fn next_token_in_array(&mut self) -> Result<Token, RuneError> {
        self.next_token_with_flag(true)
    }

    fn next_token_with_flag(&mut self, skip_newlines: bool) -> Result<Token, RuneError> {
        self.skip_whitespace_and_comments(skip_newlines);

        let token = match self.peek {
            Some('\n') => { self.bump(); Ok(Token::Newline) }
            Some(':') => { self.bump(); Ok(Token::Colon) }
            Some('=') => { self.bump(); Ok(Token::Equals) }
            Some('[') => { self.bump(); Ok(Token::LBracket) }
            Some(']') => { self.bump(); Ok(Token::RBracket) }
            Some(',') => { self.bump(); return self.next_token_with_flag(skip_newlines); } // skip commas            
            Some('$') => {
                self.bump(); // consume '$'
                Ok(Token::Dollar) // Just return Dollar token, let parser handle validation
            }
            Some('.') => { self.bump(); Ok(Token::Dot) }
            Some('@') => { self.bump(); Ok(Token::At) }

            // regex literal r"..."
            Some('r') => {
                let mut clone_iter = self.input.clone();
                let next_char = clone_iter.next();

                if next_char == Some('"') {
                    self.bump(); // consume 'r'
                    self.bump(); // consume opening '"'

                    let mut content = String::new();
                    while let Some(ch) = self.bump() {
                        if ch == '"' {
                            break; // closing quote
                        }

                        if ch == '\\' {
                            // preserve the backslash literally
                            content.push('\\');
                            if let Some(next_ch) = self.bump() {
                                content.push(next_ch);
                            } else {
                                return Err(RuneError::UnclosedString {
                                    quote: '"',
                                    line: self.line,
                                    column: self.column,
                                    hint: Some("Trailing backslash in regex".into()),
                                    code: Some(103),
                                });
                            }
                        } else {
                            content.push(ch);
                        }
                    }

                    if self.peek.is_none() {
                        // nothing to do, regex already closed properly
                    }

                    Ok(Token::Regex(content))
                } else {
                    // just a normal identifier starting with 'r'
                    let mut ident = String::new();
                    ident.push(self.bump().unwrap()); // consume 'r'
                    while let Some(ch) = self.peek {
                        if ch.is_alphanumeric() || ch == '_' || ch == '-' { 
                            ident.push(ch); 
                            self.bump(); 
                        } else { break; }
                    }
                    Ok(Token::Ident(ident))
                }
            }

            // normal double-quoted string
            Some('"') | Some('\'') => {
                let quote = self.bump().unwrap();
                let mut content = String::new();                
                while let Some(ch) = self.peek {
                    if ch == quote { 
                        self.bump(); // consume the closing quote
                        break;
                    }

                    if ch == '\\' {
                        self.bump(); // consume '\'
                        if let Some(next_ch) = self.bump() {
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
                                line: self.line,
                                column: self.column,
                                hint: Some("Trailing backslash in string".into()),
                                code: Some(103),
                            });
                        }
                    } else {
                        content.push(ch);
                        self.bump();
                    }
                }

                if self.peek.is_none() && content.ends_with(quote) == false {
                    return Err(RuneError::UnclosedString {
                        quote,
                        line: self.line,
                        column: self.column,
                        hint: Some("String literal not closed".into()),
                        code: Some(103),
                    });
                }
                Ok(Token::String(content))
            }

            // numbers
            Some(c) if c.is_digit(10) => {
                let mut num = String::new();
                while let Some(ch) = self.peek {
                    if ch.is_digit(10) || ch == '.' { num.push(ch); self.bump(); } 
                    else { break; }
                }
                num.parse::<f64>()
                    .map(Token::Number)
                    .map_err(|_| RuneError::TypeError {
                        message: format!("Invalid number '{}'", num),
                        line: self.line,
                        column: self.column,
                        hint: None,
                        code: Some(102),
                    })
            }

            // identifiers & keywords
            Some(c) if c.is_alphabetic() => {
                let mut ident = String::new();
                while let Some(ch) = self.peek {
                    if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                        ident.push(ch);
                        self.bump();
                    } else { break; }
                }

                // map both "null" and "None" to Token::Null
                match ident.as_str() {
                    "true" => Ok(Token::Bool(true)),
                    "false" => Ok(Token::Bool(false)),
                    "end" => Ok(Token::End),
                    "gather" => Ok(Token::Gather),
                    "as" => Ok(Token::As),
                    "null" | "None" => Ok(Token::Null),
                    _ => Ok(Token::Ident(ident)),
                }
            }
                
            // unexpected characters
            Some(ch) => {
                self.bump();
                Err(RuneError::UnexpectedCharacter {
                    character: ch,
                    line: self.line,
                    column: self.column,
                    hint: Some("Unexpected character in input".into()),
                    code: Some(104),
                })
            }

            None => Ok(Token::Eof),
        };

        if let Ok(ref _t) = token {
            //println!("DEBUG: peek={:?}, token={:?}", self.peek, t);
        }

        token
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_rune_example() {
        let input = r#"
gather "defaults.rune" as defaults
name "RuneApp"
app:
  name name
  version "1.0.0"
  description None
end
"#;

        let mut lexer = Lexer::new(input);

        let mut expected_tokens = vec![
            Token::Newline,
            Token::Gather,
            Token::String("defaults.rune".into()),
            Token::As,
            Token::Ident("defaults".into()),
            Token::Newline,
            Token::Ident("name".into()),
            Token::String("RuneApp".into()),
            Token::Newline,
            Token::Ident("app".into()),
            Token::Colon,
            Token::Newline,
            Token::Ident("name".into()),
            Token::Ident("name".into()),
            Token::Newline,
            Token::Ident("version".into()),
            Token::String("1.0.0".into()),
            Token::Newline,
            Token::Ident("description".into()),
            Token::Null,
            Token::Newline,
            Token::End,
            Token::Newline,
            Token::Eof,
        ];

        while !expected_tokens.is_empty() {
            let expected = expected_tokens.remove(0);
            let tok = if expected == Token::String("defaults.rune".into()) {
                lexer.next_token_in_array()
            } else {
                lexer.next_token()
            };
            println!("{:?}", tok); // debug output
            assert_eq!(tok, Ok(expected));
        }
    }

    #[test]
    fn test_dollar_namespace_tokens() {
        let input = r#"$env $sys $runtime"#;
        let mut lexer = Lexer::new(input);

        let expected_tokens = vec![
            Token::Dollar,
            Token::Ident("env".into()),
            Token::Dollar,
            Token::Ident("sys".into()),
            Token::Dollar,
            Token::Ident("runtime".into()),
            Token::Eof,
        ];

        for expected in expected_tokens {
            let tok = lexer.next_token();
            println!("{:?}", tok); // debug output
            assert_eq!(tok, Ok(expected));
        }
    }

    #[test]
    fn test_invalid_raw_string_error() {
        let input = r#"rhello"#; // missing quotes
        let mut lexer = Lexer::new(input);
        let result = lexer.next_token();

        // This should now just be parsed as a regular identifier
        assert_eq!(result, Ok(Token::Ident("rhello".into())));
    }
}

#[test]
fn test_empty_array() {
    let input = r#"plugins []"#;
    let mut lexer = Lexer::new(input);

    let expected_tokens = vec![
        Token::Ident("plugins".into()),
        Token::LBracket,
        Token::RBracket,
        Token::Eof,
    ];

    for expected in expected_tokens {
        let tok = lexer.next_token();
        assert_eq!(tok, Ok(expected));
    }
}

#[cfg(test)]
mod escape_tests {
    use super::*;

    #[test]
    fn test_string_escapes() {
        let input = r#"
escaped "\n\t\\\"\'\$"
normal "hello"
"#;

        let mut lexer = Lexer::new(input);

        let expected_tokens = vec![
            Token::Newline,
            Token::Ident("escaped".into()),
            Token::String("\n\t\\\"\'$".into()), // escapes expanded
            Token::Newline,
            Token::Ident("normal".into()),
            Token::String("hello".into()),
            Token::Newline,
            Token::Eof,
        ];

        for expected in expected_tokens {
            let tok = lexer.next_token().expect("Failed to get token");
            assert_eq!(tok, expected);
        }
    }
}

#[test]
fn test_regex_literal() {
    let input = r#"r"^foo.*bar$""#;
    let mut lexer = Lexer::new(input);
    let tok = lexer.next_token();
    assert_eq!(tok, Ok(Token::Regex("^foo.*bar$".into())));
}

#[test]
fn test_regex_with_command_like_content() {
    let input = r#"r"notify-send 'Test'""#;
    let mut lexer = Lexer::new(input);
    let tok = lexer.next_token();
    assert_eq!(tok, Ok(Token::Regex("notify-send 'Test'".into())));
}

#[test]
fn test_regex_with_escape() {
    let input = r#"r"\d{3}-\d{2}-\d{4}""#;
    let mut lexer = Lexer::new(input);
    let tok = lexer.next_token();
    assert_eq!(tok, Ok(Token::Regex("\\d{3}-\\d{2}-\\d{4}".into())));
}

#[test]
fn test_hyphen_and_underscore_identifiers() {
    let input = "foo-bar qux123";
    let mut lexer = Lexer::new(input);

    let expected_tokens = vec![
        Token::Ident("foo-bar".into()),
        Token::Ident("qux123".into()),
        Token::Eof,
    ];

    for expected in expected_tokens {
        let tok = lexer.next_token().unwrap();
        assert_eq!(tok, expected);
    }
}
