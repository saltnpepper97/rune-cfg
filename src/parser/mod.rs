use std::collections::HashMap;

use crate::lexer::{Lexer, Token};
use crate::RuneError;
use crate::ast::{Document, Value};

mod conditional;
mod document;
mod value;
mod reference;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peek: Option<Token>,
    pub imports: HashMap<String, Document>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, RuneError> {
        let mut lexer = Lexer::new(input);
        let peek = Some(lexer.next_token()?);
        Ok(Self { 
            lexer, 
            peek, 
            imports: HashMap::new() 
        })
    }

    pub fn inject_import(&mut self, alias: String, document: Document) {
        self.imports.insert(alias, document);
    }

    pub(crate) fn bump(&mut self) -> Result<Token, RuneError> {
        let curr = self.peek.take().ok_or(RuneError::UnexpectedEof {
            message: "Unexpected end of input".into(),
            line: self.lexer.line(),
            column: self.lexer.column(),
            hint: None,
            code: Some(201),
        })?;
        self.peek = Some(self.lexer.next_token()?);
        Ok(curr)
    }

    pub(crate) fn peek(&self) -> Option<&Token> {
        self.peek.as_ref()
    }

    #[allow(dead_code)]
    pub(crate) fn expect(&mut self, expected: Token) -> Result<Token, RuneError> {
        let token = self.bump()?;
        if token != expected {
            return Err(RuneError::SyntaxError {
                message: format!("Expected {:?}, got {:?}", expected, token),
                line: self.lexer.line(),
                column: self.lexer.column(),
                hint: Some("Check your syntax".into()),
                code: Some(202),
            });
        }
        Ok(token)
    }

    pub(crate) fn line(&self) -> usize {
        self.lexer.line()
    }

    pub(crate) fn column(&self) -> usize {
        self.lexer.column()
    }

    // Re-export main public methods
    pub fn parse_document(&mut self) -> Result<Document, RuneError> {
        document::parse_document(self)
    }

    pub fn resolve_reference<'b>(&'b self, path: &[String], doc: &'b Document) -> Option<&'b Value> {
        reference::resolve_reference(self, path, doc)
    }
}

#[cfg(test)]
mod tests;
