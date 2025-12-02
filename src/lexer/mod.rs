use std::str::Chars;
use crate::RuneError;

mod scanner;
mod tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    String(String),
    Regex(String),
    Number(f64),
    Bool(bool),
    Null,

    Colon, Equals, LBracket, RBracket, End,
    Dollar, Dot, At, Gather, As,

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

    pub fn next_token(&mut self) -> Result<Token, RuneError> {
        tokenizer::next_token_with_flag(self, false)
    }

    pub fn next_token_in_array(&mut self) -> Result<Token, RuneError> {
        tokenizer::next_token_with_flag(self, true)
    }
}

#[cfg(test)]
mod tests;
