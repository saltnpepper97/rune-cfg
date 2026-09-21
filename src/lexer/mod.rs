// Author: Dustin Pilgrim
// License: MIT

use crate::RuneError;
use crate::source::Span;
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
    /// Byte offset of `peek` inside the original input; it reaches the input
    /// length once the lexer is exhausted.
    offset: usize,
    line: usize,
    column: usize,
}

/// A token together with the byte span of its own lexeme.
///
/// The span excludes the whitespace and comments the lexer skipped before the
/// token, and covers exactly the bytes that produced it, so it can be handed to
/// an editor as a rename/edit range or converted into an LSP position.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpannedToken {
    pub(crate) token: Token,
    pub(crate) span: Span,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input: input.chars(),
            peek: None,
            offset: 0,
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

    /// Byte offset of the next unconsumed character within the lexer input.
    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

    /// Normal tokenization (newlines are significant)
    pub fn next_token(&mut self) -> Result<Token, RuneError> {
        self.next_token_spanned().map(|spanned| spanned.token)
    }

    /// Tokenization inside arrays (newlines ignored)
    pub fn next_token_in_array(&mut self) -> Result<Token, RuneError> {
        self.next_token_in_array_spanned()
            .map(|spanned| spanned.token)
    }

    /// Like [`Lexer::next_token`], but keeps the token's byte span.
    pub(crate) fn next_token_spanned(&mut self) -> Result<SpannedToken, RuneError> {
        tokenizer::next_token_with_flag(self, false)
    }

    /// Like [`Lexer::next_token_in_array`], but keeps the token's byte span.
    pub(crate) fn next_token_in_array_spanned(&mut self) -> Result<SpannedToken, RuneError> {
        tokenizer::next_token_with_flag(self, true)
    }
}

#[cfg(test)]
mod tests;
