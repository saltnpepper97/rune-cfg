// Author: Dustin Pilgrim
// License: MIT

use super::*;

/// NOTE:
/// `gather` statements are handled in the config loader (`RuneConfig::from_file_with_base`)
/// which reads the file(s) from disk and injects imported documents.
/// The parser only needs to record the alias so references like `alias.key.path` can resolve
/// after imports are injected.
///
/// This parser intentionally inserts a placeholder Document for the alias. The loader will
/// overwrite it with the real parsed Document.
pub(super) fn parse_document(parser: &mut Parser) -> Result<Document, RuneError> {
    let mut metadata = Vec::new();
    let mut globals = Vec::new();
    let mut items = Vec::new();

    while let Some(tok) = parser.peek() {
        match tok {
            Token::Newline => {
                parser.bump()?;
            }
            Token::Eof => {
                break;
            }
            Token::At => {
                parse_metadata(parser, &mut metadata)?;
            }
            Token::Ident(_) => {
                parse_top_level_item(parser, &mut globals, &mut items)?;
            }
            Token::Gather => {
                parse_gather_statement(parser)?;
            }
            Token::Dollar => {
                return Err(RuneError::SyntaxError {
                    message: "Dollar variables ($env, $sys, $runtime) cannot be assigned at top level".into(),
                    line: parser.line(),
                    column: parser.column(),
                    hint: Some(
                        "Dollar variables can only be used as values, not as top-level definitions".into(),
                    ),
                    code: Some(213),
                });
            }
            _ => {
                return Err(RuneError::InvalidToken {
                    token: format!("{:?}", tok),
                    line: parser.line(),
                    column: parser.column(),
                    hint: Some("Unexpected token at top-level".into()),
                    code: Some(205),
                });
            }
        }
    }

    Ok(Document { metadata, globals, items })
}

fn parse_metadata(parser: &mut Parser, metadata: &mut Vec<(String, Value)>) -> Result<(), RuneError> {
    parser.bump()?;

    if let Token::Ident(key) = parser.bump()? {
        let value = value::parse_value(parser)?;
        metadata.push((key, value));
        Ok(())
    } else {
        Err(RuneError::SyntaxError {
            message: "Expected identifier after @".into(),
            line: parser.line(),
            column: parser.column(),
            hint: None,
            code: Some(203),
        })
    }
}

fn parse_top_level_item(
    parser: &mut Parser,
    globals: &mut Vec<(String, Value)>,
    items: &mut Vec<(String, Value)>,
) -> Result<(), RuneError> {
    let key = if let Token::Ident(k) = parser.bump()? { k } else { unreachable!() };

    match parser.peek() {
        Some(Token::Colon) => {
            parser.bump()?;
            let mut object_items: Vec<crate::ast::ObjectItem> = Vec::new();

            while let Some(tok) = parser.peek() {
                match tok {
                    Token::Ident(_) => {
                        let (k, v) = value::parse_assignment(parser)?;
                        object_items.push(crate::ast::ObjectItem::Assign(k, v));
                    }
                    Token::If => {
                        // block if: if condition: ... endif
                        object_items.push(conditional::parse_if_block(parser)?);
                    }
                    Token::End => {
                        parser.bump()?;
                        break;
                    }
                    Token::Newline => {
                        parser.bump()?;
                    }
                    _ => {
                        return Err(RuneError::InvalidToken {
                            token: format!("{:?}", tok),
                            line: parser.line(),
                            column: parser.column(),
                            hint: Some("Expected key, 'if', or 'end'".into()),
                            code: Some(207),
                        });
                    }
                }
            }

            items.push((key, Value::Object(object_items)));
        }
        Some(Token::Equals) => {
            // Explicit assignment with =
            parser.bump()?;
            let value = value::parse_value(parser)?;
            globals.push((key, value));
        }
        _ => {
            // Implicit assignment (no = needed)
            let value = value::parse_value(parser)?;
            globals.push((key, value));
        }
    }

    Ok(())
}

fn parse_gather_statement(parser: &mut Parser) -> Result<(), RuneError> {
    parser.bump()?; // consume `gather`

    let filename = if let Token::String(f) = parser.bump()? {
        f
    } else {
        return Err(RuneError::SyntaxError {
            message: "Expected string after gather".into(),
            line: parser.line(),
            column: parser.column(),
            hint: None,
            code: Some(211),
        });
    };

    let alias = if let Some(Token::As) = parser.peek() {
        parser.bump()?; // consume `as`
        if let Token::Ident(a) = parser.bump()? {
            a
        } else {
            return Err(RuneError::SyntaxError {
                message: "Expected identifier after 'as'".into(),
                line: parser.line(),
                column: parser.column(),
                hint: None,
                code: Some(212),
            });
        }
    } else {
        // Default alias: file stem (no extension)
        let stem = std::path::Path::new(&filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported");
        stem.to_string()
    };

    // IMPORTANT:
    // Insert a placeholder import so reference resolution can treat the first segment as an alias.
    // The loader (RuneConfig::from_file_with_base) will overwrite this with the real document.
    if !parser.imports.contains_key(&alias) {
        parser.imports.insert(
            alias,
            Document {
                metadata: vec![],
                globals: vec![],
                items: vec![],
            },
        );
    }

    Ok(())
}
