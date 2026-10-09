// Author: Dustin Pilgrim
// License: GPL-3.0-or-later

use super::*;
use crate::ast::{AnnotatedValue, ObjectItem};
use crate::resolver::{expand_dollar_string_with_escapes, parse_dollar_reference_with_options};
use regex::Regex;

pub(super) fn parse_assignment(parser: &mut Parser) -> Result<(String, Value), RuneError> {
    let key = match parser.bump()? {
        // Bare identifier keys (`name "value"`) and quoted-string keys
        // (`"$var.mod+r" "reload"`) are both accepted; string keys are stored
        // literally with no interpolation.
        Token::Ident(k) | Token::String(k) => k,
        _ => {
            return Err(RuneError::SyntaxError {
                message: "Expected identifier or string for assignment".into(),
                line: parser.line(),
                column: parser.column(),
                hint: None,
                code: Some(208),
            });
        }
    };

    match parser.peek() {
        Some(Token::Colon) => {
            parser.bump()?;
            let mut items: Vec<ObjectItem> = Vec::new();
            let mut closed = false;

            while let Some(tok) = parser.peek() {
                match tok {
                    Token::Ident(_) | Token::String(_) => {
                        let (k, v) = parse_assignment(parser)?;
                        items.push(ObjectItem::Assign(k, v));
                    }
                    Token::If => {
                        // block if: if condition: ... endif
                        items.push(conditional::parse_if_block(parser)?);
                    }
                    Token::End => {
                        parser.bump()?;
                        closed = true;
                        break;
                    }
                    Token::Eof => break,
                    Token::Newline => {
                        parser.bump()?;
                    }
                    _ => {
                        return Err(RuneError::InvalidToken {
                            token: tok.describe(),
                            line: parser.line(),
                            column: parser.column(),
                            hint: Some("Expected key, 'if', or 'end'".into()),
                            code: Some(207),
                        });
                    }
                }
            }

            if !closed {
                return Err(RuneError::UnexpectedEof {
                    message: format!("Unclosed object block '{}'; expected 'end'", key),
                    line: parser.line(),
                    column: parser.column(),
                    hint: Some(format!("Add another 'end' to close the '{}' block", key)),
                    code: Some(215),
                });
            }

            return Ok((key, Value::Object(items)));
        }
        Some(Token::Equals) => {
            parser.bump()?;
        }
        _ => {}
    }

    let value = parse_value_with_attributes(parser)?;
    Ok((key, value))
}

pub(super) fn parse_value_with_attributes(parser: &mut Parser) -> Result<Value, RuneError> {
    let value = parse_value(parser)?;
    let Some(Token::Ident(keyword)) = parser.peek() else {
        return Ok(value);
    };
    if keyword != "with" {
        return Ok(value);
    }
    parser.bump()?;

    let mut attributes = Vec::new();
    loop {
        match parser.peek() {
            Some(
                Token::Newline
                | Token::Eof
                | Token::End
                | Token::EndIf
                | Token::Else
                | Token::ElseIf,
            ) => break,
            Some(Token::Ident(_) | Token::String(_)) => {
                let name = match parser.bump()? {
                    Token::Ident(name) | Token::String(name) => name,
                    _ => unreachable!(),
                };
                if matches!(
                    parser.peek(),
                    Some(
                        Token::Newline
                            | Token::Eof
                            | Token::End
                            | Token::EndIf
                            | Token::Else
                            | Token::ElseIf
                    )
                ) {
                    return Err(RuneError::SyntaxError {
                        message: format!("Missing value for inline attribute '{name}'"),
                        line: parser.line(),
                        column: parser.column(),
                        hint: Some(format!("Add a value after '{name}'")),
                        code: Some(216),
                    });
                }
                attributes.push((name, parse_value(parser)?));
            }
            Some(token) => {
                return Err(RuneError::InvalidToken {
                    token: token.describe(),
                    line: parser.line(),
                    column: parser.column(),
                    hint: Some(
                        "Expected an attribute name or the end of the line after 'with'".into(),
                    ),
                    code: Some(216),
                });
            }
            None => break,
        }
    }

    if attributes.is_empty() {
        return Err(RuneError::SyntaxError {
            message: "Expected at least one named attribute after 'with'".into(),
            line: parser.line(),
            column: parser.column(),
            hint: Some("Use `with name value`, or remove `with`".into()),
            code: Some(216),
        });
    }

    Ok(Value::Annotated(Box::new(AnnotatedValue {
        value: Box::new(value),
        attributes,
    })))
}

pub(super) fn parse_value(parser: &mut Parser) -> Result<Value, RuneError> {
    match parser.peek() {
        Some(Token::String(_)) => parse_string_value(parser),
        Some(Token::Number(_)) => parse_number_value(parser),
        Some(Token::Bool(_)) => parse_bool_value(parser),
        Some(Token::Regex(_)) => parse_regex_value(parser),
        Some(Token::Dollar) => parse_dollar_reference_value(parser),
        Some(Token::Ident(_)) => parse_reference_value(parser),
        Some(Token::LBracket) => parse_array_value(parser),
        Some(Token::Null) => parse_null_value(parser),
        Some(Token::If) => {
            // IMPORTANT:
            // - In value position, `if` means inline conditional:
            //     x = if cond a else b
            // - Block ifs are only parsed inside objects:
            //     if cond: ... endif
            conditional::parse_conditional(parser)
        }
        _ => {
            let token = parser.bump()?;
            Err(RuneError::InvalidToken {
                token: token.describe(),
                line: parser.line(),
                column: parser.column(),
                hint: Some("Unexpected token in value position".into()),
                code: Some(210),
            })
        }
    }
}

fn parse_string_value(parser: &mut Parser) -> Result<Value, RuneError> {
    let position = parser.token_position();
    let escaped = if parser.options.quoted_references {
        Vec::new()
    } else {
        parser.peek_escaped_dollars.clone()
    };
    if let Token::String(s) = parser.bump()? {
        expand_dollar_string_with_escapes(&s, parser.options, &escaped)
            .map_err(|error| at_position(error, position))
    } else {
        unreachable!()
    }
}

fn parse_number_value(parser: &mut Parser) -> Result<Value, RuneError> {
    if let Token::Number(n) = parser.bump()? {
        Ok(Value::Number(n))
    } else {
        unreachable!()
    }
}

fn parse_bool_value(parser: &mut Parser) -> Result<Value, RuneError> {
    if let Token::Bool(b) = parser.bump()? {
        Ok(Value::Bool(b))
    } else {
        unreachable!()
    }
}

fn parse_regex_value(parser: &mut Parser) -> Result<Value, RuneError> {
    if let Token::Regex(pattern) = parser.bump()? {
        let regex = Regex::new(&pattern).map_err(|e| RuneError::TypeError {
            message: format!("Invalid regex pattern: {}", e),
            line: parser.line(),
            column: parser.column(),
            hint: Some("Check your regex syntax".into()),
            code: Some(211),
        })?;
        Ok(Value::Regex(regex))
    } else {
        unreachable!()
    }
}

fn parse_null_value(parser: &mut Parser) -> Result<Value, RuneError> {
    parser.bump()?;
    Ok(Value::Null)
}

fn parse_dollar_reference_value(parser: &mut Parser) -> Result<Value, RuneError> {
    let position = parser.token_position();
    parser.bump()?; // $

    let namespace = if let Token::Ident(name) = parser.bump()? {
        if name != "env" && name != "sys" && name != "runtime" && name != "var" {
            return Err(RuneError::SyntaxError {
                message: format!("Unknown namespace ${}", name),
                line: parser.line(),
                column: parser.column(),
                hint: Some("Use $env, $sys, $runtime, or $var".into()),
                code: Some(209),
            });
        }
        name
    } else {
        return Err(RuneError::SyntaxError {
            message: "Expected identifier after $".into(),
            line: parser.line(),
            column: parser.column(),
            hint: None,
            code: Some(209),
        });
    };

    let mut path = vec![namespace];

    while let Some(Token::Dot) = parser.peek() {
        parser.bump()?;
        if let Token::Ident(name) = parser.bump()? {
            path.push(name);
        } else {
            return Err(RuneError::SyntaxError {
                message: "Expected identifier after '.'".into(),
                line: parser.line(),
                column: parser.column(),
                hint: None,
                code: Some(210),
            });
        }
    }

    parse_dollar_reference_with_options(path, parser.options)
        .map_err(|error| at_position(error, position))
}

fn parse_reference_value(parser: &mut Parser) -> Result<Value, RuneError> {
    let mut path = Vec::new();

    if let Token::Ident(name) = parser.bump()? {
        path.push(name);
    } else {
        unreachable!()
    }

    while let Some(Token::Dot) = parser.peek() {
        parser.bump()?;
        if let Token::Ident(name) = parser.bump()? {
            path.push(name);
        } else {
            return Err(RuneError::SyntaxError {
                message: "Expected identifier after '.'".into(),
                line: parser.line(),
                column: parser.column(),
                hint: None,
                code: Some(210),
            });
        }
    }

    Ok(Value::Reference(path))
}

fn parse_array_value(parser: &mut Parser) -> Result<Value, RuneError> {
    parser.bump()?; // '['
    let mut arr = Vec::new();

    while let Some(tok) = parser.peek() {
        match tok {
            Token::RBracket => {
                parser.bump()?;
                break;
            }
            Token::Newline => {
                parser.bump()?;
            }
            _ => {
                arr.push(parse_value(parser)?);
            }
        }
    }

    Ok(Value::Array(arr))
}

fn at_position(mut error: RuneError, position: (usize, usize)) -> RuneError {
    if let RuneError::SyntaxError { line, column, .. } = &mut error
        && *line == 0
    {
        *line = position.0;
        *column = position.1;
    }
    error
}
