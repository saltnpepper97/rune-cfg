// Author: Dustin Pilgrim
// License: MIT

use crate::RuneError;
use std::str::Chars;

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

impl Token {
    /// Human-readable label for diagnostics, avoiding `Debug` output like
    /// `String("..")` leaking into user-facing error messages.
    pub(crate) fn describe(&self) -> String {
        match self {
            Token::Ident(name) => format!("identifier '{}'", name),
            Token::String(value) => format!("string \"{}\"", value),
            Token::Regex(value) => format!("regex r\"{}\"", value),
            Token::Number(number) => format!("number {}", number),
            Token::Bool(value) => format!("boolean {}", value),
            Token::Null => "null".into(),
            Token::Colon => "':'".into(),
            Token::Equals => "'='".into(),
            Token::LBracket => "'['".into(),
            Token::RBracket => "']'".into(),
            Token::End => "'end'".into(),
            Token::EndIf => "'endif'".into(),
            Token::Dollar => "'$'".into(),
            Token::Dot => "'.'".into(),
            Token::At => "'@'".into(),
            Token::Gather => "'gather'".into(),
            Token::As => "'as'".into(),
            Token::If => "'if'".into(),
            Token::Else => "'else'".into(),
            Token::ElseIf => "'elseif'".into(),
            Token::Newline => "newline".into(),
            Token::Eof => "end of input".into(),
        }
    }
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
