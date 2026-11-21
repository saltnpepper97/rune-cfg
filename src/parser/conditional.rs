// Author: Dustin Pilgrim
// License: GPL-3.0-or-later

use super::*;
use crate::ast::{Condition, IfBlock, ObjectItem};

/// Inline/value conditional:
///     key = if condition then_value else else_value
///
/// This produces a `Value::Conditional`, which gets resolved later by the resolver/helpers.
pub(super) fn parse_conditional(parser: &mut Parser) -> Result<Value, RuneError> {
    parser.bump()?; // consume 'if'

    let condition = parse_condition(parser)?;
    let then_value = value::parse_value(parser)?;

    let else_value = if let Some(Token::Else) = parser.peek() {
        parser.bump()?; // consume 'else'
        Some(value::parse_value(parser)?)
    } else {
        None
    };

    Ok(Value::Conditional(Box::new(crate::ast::ConditionalValue {
        condition,
        then_value,
        else_value,
    })))
}

/// Block conditional inside object blocks:
///
/// ```text
/// if condition:
///   key = value
///   nested:
///     ...
///   end
/// else:
///   ...
/// end
/// ```
///
/// Produces `ObjectItem::IfBlock`, which is later flattened by config/helpers.
pub(super) fn parse_if_block(parser: &mut Parser) -> Result<ObjectItem, RuneError> {
    parser.bump()?; // consume 'if'

    let condition = parse_condition(parser)?;

    // Require colon: if condition:
    match parser.bump()? {
        Token::Colon => {}
        other => {
            return Err(RuneError::SyntaxError {
                message: format!("Expected ':' after if condition, got {:?}", other),
                line: parser.line(),
                column: parser.column(),
                hint: Some("Use: if condition:".into()),
                code: Some(214),
            });
        }
    }

    // Parse the then-branch items until `else`, `elseif`, or `end`.
    let then_items = parse_object_items_until(parser, StopAt::BranchOrEnd)?;
    let else_items = parse_conditional_tail(parser)?;

    Ok(ObjectItem::IfBlock(Box::new(IfBlock {
        condition,
        then_items,
        else_items,
    })))
}

/// Parse the optional `else`/`elseif` chain and consume its single closing `end`.
///
/// An `elseif` is represented as an `IfBlock` nested inside the preceding
/// block's `else_items`. This keeps the public AST stable while making chained
/// branches evaluate in the same order as the source language.
fn parse_conditional_tail(parser: &mut Parser) -> Result<Option<Vec<ObjectItem>>, RuneError> {
    match parser.peek() {
        Some(Token::Else) => {
            parser.bump()?;
            match parser.bump()? {
                Token::Colon => {}
                other => {
                    return Err(RuneError::SyntaxError {
                        message: format!("Expected ':' after else, got {:?}", other),
                        line: parser.line(),
                        column: parser.column(),
                        hint: Some("Use: else:".into()),
                        code: Some(214),
                    });
                }
            }

            let items = parse_object_items_until(parser, StopAt::EndOnly)?;
            expect_end(parser)?;
            Ok(Some(items))
        }
        Some(Token::ElseIf) => {
            parser.bump()?;
            let condition = parse_condition(parser)?;
            match parser.bump()? {
                Token::Colon => {}
                other => {
                    return Err(RuneError::SyntaxError {
                        message: format!("Expected ':' after elseif condition, got {:?}", other),
                        line: parser.line(),
                        column: parser.column(),
                        hint: Some("Use: elseif condition:".into()),
                        code: Some(214),
                    });
                }
            }

            let then_items = parse_object_items_until(parser, StopAt::BranchOrEnd)?;
            let else_items = parse_conditional_tail(parser)?;
            Ok(Some(vec![ObjectItem::IfBlock(Box::new(IfBlock {
                condition,
                then_items,
                else_items,
            }))]))
        }
        _ => {
            expect_end(parser)?;
            Ok(None)
        }
    }
}

fn expect_end(parser: &mut Parser) -> Result<(), RuneError> {
    match parser.bump()? {
        Token::End => match parser.peek() {
            // Only named blocks take a label; an if-block has no name to repeat.
            Some(Token::Ident(label) | Token::String(label)) => {
                let label = label.clone();
                let (line, column) = parser.token_position();
                Err(RuneError::SyntaxError {
                    message: format!("'end {}' closes an if-block, which takes no label", label),
                    line,
                    column,
                    hint: Some("Close if-blocks with a bare 'end'".into()),
                    code: Some(219),
                })
            }
            _ => Ok(()),
        },
        Token::Eof => Err(RuneError::UnexpectedEof {
            message: "Unclosed if-block; expected 'end'".into(),
            line: parser.line(),
            column: parser.column(),
            hint: Some("Close if-blocks with 'end'".into()),
            code: Some(215),
        }),
        other => Err(RuneError::SyntaxError {
            message: format!("Expected 'end', got {:?}", other),
            line: parser.line(),
            column: parser.column(),
            hint: Some("Close if-blocks with 'end'".into()),
            code: Some(214),
        }),
    }
}

#[derive(Copy, Clone)]
enum StopAt {
    BranchOrEnd,
    EndOnly,
}

fn parse_object_items_until(
    parser: &mut Parser,
    stop: StopAt,
) -> Result<Vec<ObjectItem>, RuneError> {
    let mut items: Vec<ObjectItem> = Vec::new();

    while let Some(tok) = parser.peek() {
        match tok {
            Token::Newline => {
                parser.bump()?;
            }

            Token::Ident(_) | Token::String(_) => {
                let (k, v) = value::parse_assignment(parser)?;
                items.push(ObjectItem::Assign(k, v));
            }

            Token::If => {
                // nested if-block
                items.push(parse_if_block(parser)?);
            }

            Token::Else | Token::ElseIf => {
                if matches!(stop, StopAt::BranchOrEnd) {
                    break;
                }
                return Err(RuneError::InvalidToken {
                    token: tok.describe(),
                    line: parser.line(),
                    column: parser.column(),
                    hint: Some("Unexpected branch (no matching 'if'?)".into()),
                    code: Some(207),
                });
            }

            // Nested objects consume their own `end`, so an `end` reached here
            // closes the if-block.
            Token::End | Token::Eof => {
                break;
            }

            _ => {
                return Err(RuneError::InvalidToken {
                    token: tok.describe(),
                    line: parser.line(),
                    column: parser.column(),
                    hint: Some("Expected assignment, nested block, 'else', or 'end'".into()),
                    code: Some(207),
                });
            }
        }
    }

    // The caller consumes the closing `end` and reports a missing one.
    Ok(items)
}

/// Parse the test after `if` or `elseif`:
///
/// ```text
/// condition := "not" condition
///            | path [ ("=" | "!=") value | "in" "[" value, ... "]" ]
/// path      := ["$"] name ("." name)*
/// ```
///
/// `not` and `in` are contextual: `not` negates only when a path follows it,
/// and `in` is a membership test only when a list follows it, so keys named
/// `not` or `in` still work.
fn parse_condition(parser: &mut Parser) -> Result<Condition, RuneError> {
    if matches!(parser.peek(), Some(Token::Ident(word)) if word == "not") {
        parser.bump()?;
        if matches!(parser.peek(), Some(Token::Ident(_) | Token::Dollar)) {
            return Ok(Condition::Not(Box::new(parse_condition(parser)?)));
        }
        // A key named `not`.
        return parse_comparison(parser, "not".into());
    }

    let path = parse_condition_path(parser)?;
    parse_comparison(parser, path)
}

/// The comparison after a condition's path, if any.
fn parse_comparison(parser: &mut Parser, path: String) -> Result<Condition, RuneError> {
    match parser.peek() {
        Some(Token::Equals) => {
            parser.bump()?;
            Ok(Condition::Equals(path, value::parse_value(parser)?))
        }
        Some(Token::NotEquals) => {
            parser.bump()?;
            Ok(Condition::NotEquals(path, value::parse_value(parser)?))
        }
        Some(Token::Ident(word)) if word == "in" => {
            parser.bump()?;
            let (line, column) = parser.token_position();
            if parser.peek() != Some(&Token::LBracket) {
                return Err(RuneError::SyntaxError {
                    message: format!("Expected a list after '{} in'", path),
                    line,
                    column,
                    hint: Some(format!("Use: if {} in [\"a\", \"b\"]", path)),
                    code: Some(214),
                });
            }
            match value::parse_value(parser)? {
                Value::Array(options) => Ok(Condition::In(path, options)),
                _ => unreachable!("a value starting with '[' is an array"),
            }
        }
        _ => Ok(Condition::Truthy(path)),
    }
}

/// A condition's path: `name`, `a.b.c`, or `$env.NAME`, joined with dots.
/// `$env`, `$sys`, `$runtime` and `$var` read the same as their bare forms.
fn parse_condition_path(parser: &mut Parser) -> Result<String, RuneError> {
    let (line, column) = parser.token_position();
    let dollar = parser.peek() == Some(&Token::Dollar);
    if dollar {
        parser.bump()?;
    }

    let mut segments = vec![expect_path_segment(parser, line, column)?];
    while parser.peek() == Some(&Token::Dot) {
        parser.bump()?;
        let (line, column) = parser.token_position();
        segments.push(expect_path_segment(parser, line, column)?);
    }

    if dollar && !matches!(segments[0].as_str(), "env" | "sys" | "runtime" | "var") {
        return Err(RuneError::SyntaxError {
            message: format!("Unknown namespace ${}", segments[0]),
            line,
            column,
            hint: Some("Use $env, $sys, $runtime, or $var".into()),
            code: Some(214),
        });
    }

    Ok(segments.join("."))
}

fn expect_path_segment(
    parser: &mut Parser,
    line: usize,
    column: usize,
) -> Result<String, RuneError> {
    match parser.peek() {
        Some(Token::Ident(name)) => {
            let name = name.clone();
            parser.bump()?;
            Ok(name)
        }
        other => Err(RuneError::SyntaxError {
            message: format!(
                "Expected a key name in condition, got {}",
                other
                    .map(Token::describe)
                    .unwrap_or_else(|| "end of input".into())
            ),
            line,
            column,
            hint: Some("Use: if key, if app.key, or if key = value".into()),
            code: Some(214),
        }),
    }
}
