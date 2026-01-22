// Author: Dustin Pilgrim
// License: MIT

use std::str::Chars;
use crate::RuneError;

mod scanner;
mod tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // --- literals ---
    Ident(String),
    String(String),
    Regex(String),
    Number(f64),
    Bool(bool),
    Null,

    // --- structure ---
    Colon,
    Equals,
    LBracket,
    RBracket,

    End,
    EndIf,

    // --- symbols ---
    Dollar,
    Dot,
    At,

    // --- keywords ---
    Gather,
    As,
    If,
    Else,
    ElseIf,

    // --- layout ---
    Newline,
    Eof,
}

pub struct Lexer<'a> {
    input: Chars<'a>,
    peek: Option<char>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input: input.chars(),
            peek: None,
            line: 1,
            column: 0,
        };
        lexer.peek = lexer.input.next();
        lexer
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    /// Normal tokenization (newlines are significant)
    pub fn next_token(&mut self) -> Result<Token, RuneError> {
        tokenizer::next_token_with_flag(self, false)
    }

    /// Tokenization inside arrays (newlines ignored)
    pub fn next_token_in_array(&mut self) -> Result<Token, RuneError> {
        tokenizer::next_token_with_flag(self, true)
    }
}

#[cfg(test)]
mod tests;
