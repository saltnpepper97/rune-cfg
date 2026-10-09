// Author: Dustin Pilgrim
// License: GPL-3.0-or-later

use std::collections::HashMap;

use crate::RuneError;
use crate::ast::{Document, Value};
use crate::lexer::{Lexer, Token};

mod conditional;
mod document;
mod reference;
mod value;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peek: Option<Token>,
    peek_start: usize,
    peek_escaped_dollars: Vec<usize>,
    lines: crate::source::LineIndex,
    pub(crate) options: crate::ParseOptions,
    pub imports: HashMap<String, Document>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, RuneError> {
        Self::with_options(input, crate::ParseOptions::default())
    }

    pub fn with_options(input: &'a str, options: crate::ParseOptions) -> Result<Self, RuneError> {
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token_spanned()?;
        let peek_escaped_dollars = lexer.escaped_dollars.clone();
        let peek_start = token.span.start;
        let peek = Some(token.token);
        Ok(Self {
            lexer,
            peek,
            peek_start,
            peek_escaped_dollars,
            lines: crate::source::LineIndex::new(input),
            options,
            imports: HashMap::new(),
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
        let token = self.lexer.next_token_spanned()?;
        self.peek_start = token.span.start;
        self.peek_escaped_dollars = self.lexer.escaped_dollars.clone();
        self.peek = Some(token.token);
        Ok(curr)
    }

    pub(crate) fn token_position(&self) -> (usize, usize) {
        let position = self.lines.byte_to_position(self.peek_start);
        (position.line as usize + 1, position.character as usize + 1)
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

    pub fn resolve_reference<'b>(
        &'b self,
        path: &[String],
        doc: &'b Document,
    ) -> Option<&'b Value> {
        reference::resolve_reference(self, path, doc)
    }
}

#[cfg(test)]
mod tests;
