use super::*;
use crate::ast::Condition;

pub(super) fn parse_conditional(parser: &mut Parser) -> Result<Value, RuneError> {
    parser.bump()?;
    
    let condition = parse_condition(parser)?;
    
    let then_value = value::parse_value(parser)?;
    
    let else_value = if let Some(Token::Else) = parser.peek() {
        parser.bump()?;
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
