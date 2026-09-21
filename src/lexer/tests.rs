// Author: Dustin Pilgrim
// License: MIT

#[cfg(test)]
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
        println!("{:?}", tok);
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
        println!("{:?}", tok);
        assert_eq!(tok, Ok(expected));
    }
}

#[test]
fn test_invalid_raw_string_error() {
    let input = r#"rhello"#;
    let mut lexer = Lexer::new(input);
    let result = lexer.next_token();

    assert_eq!(result, Ok(Token::Ident("rhello".into())));
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
        Token::String("\n\t\\\"\'$".into()),
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

#[test]
fn comments_preserve_the_newline_statement_boundary() {
    let input = "name \"Rune\" # inline comment\nenabled true\n";
    let mut lexer = Lexer::new(input);

    let expected_tokens = vec![
        Token::Ident("name".into()),
        Token::String("Rune".into()),
        Token::Newline,
        Token::Ident("enabled".into()),
        Token::Bool(true),
        Token::Newline,
        Token::Eof,
    ];

    for expected in expected_tokens {
        assert_eq!(lexer.next_token(), Ok(expected));
    }
}

#[test]
fn test_endif_token() {
    let input = r#"
if something:
endif
"#;

    let mut lexer = Lexer::new(input);

    let expected_tokens = vec![
        Token::Newline,
        Token::If,
        Token::Ident("something".into()),
        Token::Colon,
        Token::Newline,
        Token::EndIf,
        Token::Newline,
        Token::Eof,
    ];

    for expected in expected_tokens {
        let tok = lexer.next_token();
        println!("{:?}", tok);
        assert_eq!(tok, Ok(expected));
    }
}

#[test]
fn test_else_and_endif_tokens() {
    let input = r#"
if something:
else:
endif
"#;

    let mut lexer = Lexer::new(input);

    let expected_tokens = vec![
        Token::Newline,
        Token::If,
        Token::Ident("something".into()),
        Token::Colon,
        Token::Newline,
        Token::Else,
        Token::Colon,
        Token::Newline,
        Token::EndIf,
        Token::Newline,
        Token::Eof,
    ];

    for expected in expected_tokens {
        let tok = lexer.next_token();
        assert_eq!(tok, Ok(expected));
    }
}

#[test]
fn test_token_spans_cover_lexemes() {
    let input = "app:\n  name \"Rune\" # comment\n";
    let mut lexer = Lexer::new(input);

    let mut spanned = Vec::new();
    loop {
        let token = lexer.next_token_spanned().expect("token");
        let done = token.token == Token::Eof;
        spanned.push(token);
        if done {
            break;
        }
    }

    let spans: Vec<(Token, Span)> = spanned
        .into_iter()
        .map(|token| (token.token, token.span))
        .collect();

    assert_eq!(spans[0], (Token::Ident("app".into()), Span::new(0, 3)));
    assert_eq!(spans[1], (Token::Colon, Span::new(3, 4)));
    assert_eq!(spans[2], (Token::Newline, Span::new(4, 5)));
    // Whitespace and comments are not part of the lexeme that follows them.
    assert_eq!(spans[3], (Token::Ident("name".into()), Span::new(7, 11)));
    assert_eq!(spans[4], (Token::String("Rune".into()), Span::new(12, 18)));
    assert_eq!(spans[5], (Token::Newline, Span::new(28, 29)));
    assert_eq!(
        spans[6],
        (Token::Eof, Span::new(input.len(), input.len())),
        "end of input is an empty span at the end of the buffer"
    );
}

#[test]
fn test_regex_and_quoted_key_spans() {
    let input = "\"$var.mod+r\" r\"a#b\"";
    let mut lexer = Lexer::new(input);

    let key = lexer.next_token_spanned().expect("key");
    assert_eq!(key.token, Token::String("$var.mod+r".into()));
    assert_eq!(&input[key.span.start..key.span.end], "\"$var.mod+r\"");

    let regex = lexer.next_token_spanned().expect("regex");
    assert_eq!(regex.token, Token::Regex("a#b".into()));
    assert_eq!(&input[regex.span.start..regex.span.end], "r\"a#b\"");
}

/// A string closed by a quote right at the end of the buffer is closed: the
/// final line of an editor buffer often has no trailing newline.
#[test]
fn test_string_closed_at_end_of_input() {
    let mut lexer = Lexer::new("name \"x\"");

    assert_eq!(
        lexer.next_token_spanned().map(|token| token.token),
        Ok(Token::Ident("name".into()))
    );
    let value = lexer.next_token_spanned().expect("a closed string");
    assert_eq!(value.token, Token::String("x".into()));
    assert_eq!(value.span, Span::new(5, 8));
    assert_eq!(
        lexer.next_token_spanned().map(|token| token.token),
        Ok(Token::Eof)
    );
}

#[test]
fn test_crlf_is_layout_and_preserves_line_tokens() {
    let mut lexer = Lexer::new("app:\r\n  name \"Rune\"\r\n");
    let expected = [
        Token::Ident("app".into()),
        Token::Colon,
        Token::Newline,
        Token::Ident("name".into()),
        Token::String("Rune".into()),
        Token::Newline,
        Token::Eof,
    ];

    for expected in expected {
        assert_eq!(lexer.next_token(), Ok(expected));
    }
}

#[test]
fn test_unterminated_string_is_still_an_error() {
    let mut lexer = Lexer::new("name \"x");

    assert_eq!(lexer.next_token(), Ok(Token::Ident("name".into())));
    match lexer.next_token() {
        Err(RuneError::UnclosedString { .. }) => {}
        other => panic!("expected an unclosed string, got {other:?}"),
    }
}
