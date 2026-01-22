// Author: Dustin Pilgrim
// License: MIT

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
/// endif
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

    // Parse the then-branch items until `else` or `endif`
    let then_items = parse_object_items_until(parser, StopAt::ElseOrEndIf)?;

    // Optional else-branch
    let else_items = if let Some(Token::Else) = parser.peek() {
        parser.bump()?; // consume 'else'

        // Require colon: else:
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

        Some(parse_object_items_until(parser, StopAt::EndIfOnly)?)
    } else {
        None
    };

    // Must close with `endif`
    match parser.bump()? {
        Token::EndIf => {}
        other => {
            return Err(RuneError::SyntaxError {
                message: format!("Expected 'endif', got {:?}", other),
                line: parser.line(),
                column: parser.column(),
                hint: Some("Close if-blocks with 'endif'".into()),
                code: Some(214),
            });
        }
    }

    Ok(ObjectItem::IfBlock(Box::new(IfBlock {
        condition,
        then_items,
        else_items,
    })))
}

#[derive(Copy, Clone)]
enum StopAt {
    ElseOrEndIf,
    EndIfOnly,
}

fn parse_object_items_until(parser: &mut Parser, stop: StopAt) -> Result<Vec<ObjectItem>, RuneError> {
    let mut items: Vec<ObjectItem> = Vec::new();

    while let Some(tok) = parser.peek() {
        match tok {
            Token::Newline => {
                parser.bump()?;
            }

            Token::Ident(_) => {
                let (k, v) = value::parse_assignment(parser)?;
                items.push(ObjectItem::Assign(k, v));
            }

            Token::If => {
                // nested if-block
                items.push(parse_if_block(parser)?);
            }

            Token::Else => {
                if matches!(stop, StopAt::ElseOrEndIf) {
                    break;
                }
                return Err(RuneError::InvalidToken {
                    token: format!("{:?}", tok),
                    line: parser.line(),
                    column: parser.column(),
                    hint: Some("Unexpected 'else' (no matching 'if'?)".into()),
                    code: Some(207),
                });
            }

            Token::EndIf => {
                break;
            }

            Token::End => {
                return Err(RuneError::SyntaxError {
                    message: "Found 'end' while parsing an if-block; did you mean 'endif'?".into(),
                    line: parser.line(),
                    column: parser.column(),
                    hint: Some("Use 'endif' to close if-blocks".into()),
                    code: Some(214),
                });
            }

            _ => {
                return Err(RuneError::InvalidToken {
                    token: format!("{:?}", tok),
                    line: parser.line(),
                    column: parser.column(),
                    hint: Some("Expected assignment, nested block, 'else', or 'endif'".into()),
                    code: Some(207),
                });
            }
        }
    }

    // If stop == EndIfOnly and we reached EOF without endif, error will occur when caller expects EndIf.
    Ok(items)
}

fn parse_condition(parser: &mut Parser) -> Result<Condition, RuneError> {
    let path = if let Token::Ident(name) = parser.bump()? {
        name
    } else {
        return Err(RuneError::SyntaxError {
            message: "Expected identifier in condition".into(),
            line: parser.line(),
            column: parser.column(),
            hint: None,
            code: Some(214),
        });
    };

    match parser.peek() {
        Some(Token::Equals) => {
            parser.bump()?;
            let value = value::parse_value(parser)?;
            Ok(Condition::Equals(path, value))
        }
        _ => Ok(Condition::Exists(path)),
    }
}
