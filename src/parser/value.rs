use super::*;
use crate::resolver::{expand_dollar_string, parse_dollar_reference};
use regex::Regex;

pub(super) fn parse_assignment(parser: &mut Parser) -> Result<(String, Value), RuneError> {
    let key = if let Token::Ident(k) = parser.bump()? { 
        k 
    } else {
        return Err(RuneError::SyntaxError {
            message: "Expected identifier for assignment".into(),
            line: parser.line(),
            column: parser.column(),
            hint: None,
            code: Some(208),
        });
    };

    match parser.peek() {
        Some(Token::Colon) => {
            parser.bump()?;
            let mut items = Vec::new();
            
            while let Some(tok) = parser.peek() {
                match tok {
                    Token::Ident(_) => { 
                        items.push(parse_assignment(parser)?); 
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
                            hint: Some("Expected key or 'end'".into()),
                            code: Some(207),
                        }); 
                    }
                }
            }
            return Ok((key, Value::Object(items)));
        }
        Some(Token::Equals) => { 
            parser.bump()?; 
        }
        _ => {
        }
    }

    let value = parse_value(parser)?;
    Ok((key, value))
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
        Some(Token::If) => conditional::parse_conditional(parser),
        _ => {
            let token = parser.bump()?;
            Err(RuneError::InvalidToken {
                token: format!("{:?}", token),
                line: parser.line(),
                column: parser.column(),
                hint: Some("Unexpected token in value position".into()),
                code: Some(210),
            })
        }
    }
}

fn parse_string_value(parser: &mut Parser) -> Result<Value, RuneError> {
    if let Token::String(s) = parser.bump()? {
        expand_dollar_string(&s)
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
    parser.bump()?;

    let namespace = if let Token::Ident(name) = parser.bump()? {
        if name != "env" && name != "sys" && name != "runtime" {
            return Err(RuneError::SyntaxError {
                message: format!("Unknown namespace ${}", name),
                line: parser.line(),
                column: parser.column(),
                hint: Some("Use $env, $sys, or $runtime".into()),
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

    parse_dollar_reference(path)
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
    parser.bump()?;
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
