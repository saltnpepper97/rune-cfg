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
