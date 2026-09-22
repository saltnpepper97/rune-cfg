// Author: Dustin Pilgrim
// License: MIT

//! Source coordinates and a tolerant structural index of RUNE buffers.
//!
//! Two things live here:
//!
//! * [`LineIndex`] is the only place that translates between UTF-8 byte offsets
//!   and zero-based LSP positions (`line` plus a UTF-16 `character`).
//! * [`SourceIndex`] is a span-aware structural view of a buffer, built from the
//!   real [`Lexer`] token stream. It never re-implements the language's lexical
//!   rules and it never rejects input: a half-written editor buffer simply
//!   yields the entries recognised so far.
//!
//! Conditional blocks (`if` / `else` / `elseif` / `endif`) describe layout and
//! control flow. They are recorded as entries of their own, they never become
//! fields, and they never change the object path of the statements inside them.

use tower_lsp::lsp_types::{Position, Range};

use crate::diagnostic::{SourcePosition, SourceRange};
use crate::lexer::{Lexer, SpannedToken, Token};

/// Half-open UTF-8 byte range into a source buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    pub(crate) fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(crate) fn empty(at: usize) -> Self {
        Self { start: at, end: at }
    }

    pub(crate) fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// True when `offset` lies inside the span or exactly at its `end`, which is
    /// how a cursor sitting right after a typed-out identifier behaves.
    pub(crate) fn touches(&self, offset: usize) -> bool {
        self.start <= offset && offset <= self.end
    }

    /// The smallest span covering both spans.
    pub(crate) fn merge(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }

    /// The span with one byte removed from each side, when it is long enough.
    /// Used to narrow a quoted token to its content.
    pub(crate) fn inner(self) -> Span {
        if self.len() >= 2 {
            Span::new(self.start + 1, self.end - 1)
        } else {
            self
        }
    }
}

/// Byte geometry of a text buffer, plus every byte<->LSP conversion.
#[derive(Debug, Clone)]
pub(crate) struct LineIndex {
    text: String,
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub(crate) fn new(text: &str) -> Self {
        let mut line_starts = vec![0usize];
        for (offset, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(offset + 1);
            }
        }

        Self {
            text: text.to_string(),
            line_starts,
        }
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Byte span of one zero-based line, excluding its line terminator (`\n`,
    /// and the `\r` of a CRLF pair).
    pub(crate) fn line_span(&self, line: usize) -> Option<Span> {
        let start = *self.line_starts.get(line)?;
        let mut end = match self.line_starts.get(line + 1) {
            Some(next) => next.saturating_sub(1),
            None => self.text.len(),
        };
        end = end.max(start);

        if self.text.get(start..end).is_some_and(|s| s.ends_with('\r')) {
            end -= 1;
        }

        Some(Span::new(start, end))
    }

    pub(crate) fn line_text(&self, line: usize) -> Option<&str> {
        let span = self.line_span(line)?;
        self.text.get(span.start..span.end)
    }

    /// Zero-based line containing `offset`.
    pub(crate) fn line_of(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(next) => next.saturating_sub(1),
        }
    }

    /// Leading whitespace of one zero-based line.
    pub(crate) fn indent(&self, line: usize) -> &str {
        let Some(text) = self.line_text(line) else {
            return "";
        };
        let end = text.len() - text.trim_start().len();
        text.get(..end).unwrap_or("")
    }

    /// LSP position of a byte offset, counted in UTF-16 code units.
    pub(crate) fn byte_to_position(&self, offset: usize) -> Position {
        let offset = offset.min(self.text.len());
        let line = self.line_of(offset);
        let span = self.line_span(line).unwrap_or_else(|| Span::empty(offset));
        let capped = offset.clamp(span.start, span.end);
        let character = self
            .text
            .get(span.start..capped)
            .map(|text| text.encode_utf16().count())
            .unwrap_or(0);

        Position::new(line as u32, character as u32)
    }

    /// Byte offset of an LSP position. A character past the end of its line
    /// clamps to the line end, as LSP clients expect.
    pub(crate) fn position_to_byte(&self, position: Position) -> Option<usize> {
        let span = self.line_span(position.line as usize)?;
        let text = self.text.get(span.start..span.end)?;

        if position.character == 0 {
            return Some(span.start);
        }

        let mut utf16 = 0u32;
        for (byte, ch) in text.char_indices() {
            if utf16 >= position.character {
                return Some(span.start + byte);
            }
            utf16 += ch.len_utf16() as u32;
        }

        Some(span.end)
    }

    pub(crate) fn range(&self, span: Span) -> Range {
        Range::new(
            self.byte_to_position(span.start),
            self.byte_to_position(span.end),
        )
    }

    /// Range spanning the whole buffer, for a full-buffer replacement edit.
    pub(crate) fn full_range(&self) -> Range {
        let line = self.line_count().saturating_sub(1);
        let end = self
            .line_span(line)
            .map(|span| self.byte_to_position(span.end))
            .unwrap_or_else(|| Position::new(line as u32, 0));

        Range::new(Position::new(0, 0), end)
    }

    /// A 1-based, UTF-16-columned range, which is the shape
    /// [`crate::RuneDiagnostic`] carries.
    pub(crate) fn rune_range(&self, span: Span) -> SourceRange {
        SourceRange {
            start: self.rune_position(span.start),
            end: self.rune_position(span.end),
        }
    }

    fn rune_position(&self, offset: usize) -> SourcePosition {
        let position = self.byte_to_position(offset);
        SourcePosition {
            line: position.line as usize + 1,
            column: position.character as usize + 1,
        }
    }

    /// Convert a 1-based, `char`-counted column (the convention the lexer and
    /// parser report errors in) into a 1-based UTF-16 column for the same line.
    pub(crate) fn utf16_column(&self, line: usize, column: usize) -> usize {
        if column <= 1 {
            return column.max(1);
        }

        let Some(text) = self.line_text(line.saturating_sub(1)) else {
            return column;
        };

        let mut utf16 = 1usize;
        for ch in text.chars().take(column - 1) {
            utf16 += ch.len_utf16();
        }

        utf16.min(text.encode_utf16().count() + 1)
    }
}

/// What a structural entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceEntryKind {
    /// `key value` or `key = value`
    Assignment,
    /// `key:` ... `end`
    Object,
    /// `if <condition>:`
    ConditionalHeader,
    /// `else:` or `elseif <condition>:`
    ConditionalBranch,
    /// `endif`
    ConditionalEnd,
    /// `@name value`
    Metadata,
    /// `gather "file" as alias`
    Gather,
}

impl SourceEntryKind {
    /// Fields are the entries an editor can rename, jump to, or complete;
    /// conditional, metadata and gather entries only describe layout.
    pub(crate) fn is_field(self) -> bool {
        matches!(self, Self::Assignment | Self::Object)
    }

    /// True for the entries that open a block, which indents what follows.
    pub(crate) fn opens_block(self) -> bool {
        matches!(self, Self::Object | Self::ConditionalHeader)
    }

    /// True for the entries that are written one level out from the statements
    /// they close or continue.
    pub(crate) fn dedents(self) -> bool {
        matches!(self, Self::ConditionalEnd | Self::ConditionalBranch)
    }
}

/// A closing keyword with nothing to close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StrayCloser {
    End,
    EndIf,
}

impl StrayCloser {
    pub(crate) fn keyword(self) -> &'static str {
        match self {
            Self::End => "end",
            Self::EndIf => "endif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StrayCloserSpan {
    pub(crate) kind: StrayCloser,
    pub(crate) span: Span,
}

/// One structural entry of a buffer.
#[derive(Debug, Clone)]
pub(crate) struct SourceEntry {
    pub(crate) kind: SourceEntryKind,
    /// Object path enclosing this entry; empty at top level.
    pub(crate) scope: Vec<String>,
    /// Full path of the entry: its `scope` plus its own name for fields, and
    /// its `scope` for conditional, metadata and gather entries.
    pub(crate) path: Vec<String>,
    pub(crate) name: Option<String>,
    /// Bytes of the key token. A quoted key narrows to the quoted content,
    /// which is exactly the text a rename should replace.
    pub(crate) key_span: Span,
    /// Bytes of the entry itself: key plus value for a statement, the whole
    /// header for objects and conditionals.
    pub(crate) header_span: Span,
    /// Bytes of an assignment's value tokens; `None` while the key has none.
    pub(crate) value_span: Option<Span>,
    /// Bytes of the `end` closing an object; `None` while it is unclosed.
    pub(crate) close_span: Option<Span>,
}

/// A tolerant, span-aware view of one RUNE buffer.
///
/// The token stream the index was built from is kept, because a cursor question
/// that is about a token rather than about structure - a quoted `@schema`
/// value, or a `$...` reference run - is answered from real tokens instead of
/// from raw line text.
#[derive(Debug, Clone)]
pub(crate) struct SourceIndex {
    lines: LineIndex,
    tokens: Vec<SpannedToken>,
    entries: Vec<SourceEntry>,
    stray_closers: Vec<StrayCloserSpan>,
}

impl SourceIndex {
    pub(crate) fn new(text: &str) -> Self {
        let lines = LineIndex::new(text);
        let tokens = spanned_tokens(lines.text());
        let (entries, stray_closers) = IndexBuilder::new(&tokens).build();

        Self {
            lines,
            tokens,
            entries,
            stray_closers,
        }
    }

    pub(crate) fn text(&self) -> &str {
        self.lines.text()
    }

    pub(crate) fn lines(&self) -> &LineIndex {
        &self.lines
    }

    pub(crate) fn entries(&self) -> &[SourceEntry] {
        &self.entries
    }

    pub(crate) fn stray_closers(&self) -> &[StrayCloserSpan] {
        &self.stray_closers
    }

    /// Byte offset of an LSP position.
    pub(crate) fn offset_at(&self, position: Position) -> Option<usize> {
        self.lines.position_to_byte(position)
    }

    /// The field (object or assignment) whose key token sits on `line`.
    pub(crate) fn field_on_line(&self, line: u32) -> Option<&SourceEntry> {
        self.entries.iter().find(|entry| {
            entry.kind.is_field() && self.lines.line_of(entry.key_span.start) as u32 == line
        })
    }

    /// The field whose key token the cursor intersects.
    pub(crate) fn field_key_at(&self, position: Position) -> Option<&SourceEntry> {
        let offset = self.offset_at(position)?;
        self.entries
            .iter()
            .find(|entry| entry.kind.is_field() && entry.key_span.touches(offset))
    }

    /// The full token span of an entry's quoted value: the string token that
    /// starts at the value's first byte, quotes included.
    ///
    /// `None` when the entry has no value at all, or when its value does not
    /// open with a string literal, such as an array or a regex. A still-open
    /// string yields no token either: the indexer stops at the unterminated
    /// literal.
    pub(crate) fn quoted_value_span(&self, entry: &SourceEntry) -> Option<Span> {
        let start = entry.value_span?.start;

        self.tokens
            .iter()
            .find(|token| token.span.start == start)
            .filter(|token| matches!(token.token, Token::String(_)))
            .map(|token| token.span)
    }

    /// True when the cursor is inside the quoted value of the named metadata
    /// directive, such as `@schema "name"`.
    ///
    /// An unterminated `@schema "` has no string token, so it is recognised
    /// from the directive's own line alone: the cursor must sit after the
    /// opening quote on that line. No other line and no wider prefix takes
    /// part.
    pub(crate) fn metadata_string_context(&self, name: &str, position: Position) -> bool {
        let Some(offset) = self.offset_at(position) else {
            return false;
        };
        let Some(entry) = self.entries.iter().find(|entry| {
            entry.kind == SourceEntryKind::Metadata && entry.name.as_deref() == Some(name)
        }) else {
            return false;
        };

        match self.quoted_value_span(entry) {
            Some(span) => span.touches(offset),
            None => entry.value_span.is_none() && self.after_open_quote(entry, offset),
        }
    }

    /// True when `offset` lies after the opening quote of a still-open string
    /// value on the directive's own line.
    fn after_open_quote(&self, entry: &SourceEntry, offset: usize) -> bool {
        let line = self.lines.line_of(entry.key_span.start);
        let Some(span) = self.lines.line_span(line) else {
            return false;
        };
        if offset > span.end {
            return false;
        }

        let Some(rest) = self.text().get(entry.key_span.end..span.end) else {
            return false;
        };
        rest.find('"')
            .is_some_and(|quote| offset > entry.key_span.end + quote)
    }

    /// True when the cursor touches a `$...` reference run: a `Dollar` token
    /// followed by the contiguous components of the same reference. The cursor
    /// may sit anywhere on the run, exactly at its end included.
    ///
    /// A `$` inside a string literal is part of that string token and never a
    /// `Dollar` token, so it never forms a run; a finished `$...` somewhere
    /// else is a different run and does not qualify.
    pub(crate) fn dollar_reference_context(&self, position: Position) -> bool {
        let Some(offset) = self.offset_at(position) else {
            return false;
        };

        self.dollar_reference_spans()
            .into_iter()
            .any(|span| span.touches(offset))
    }

    /// Byte spans of the `$...` reference runs of the buffer.
    fn dollar_reference_spans(&self) -> Vec<Span> {
        let mut spans = Vec::new();
        let mut index = 0;

        while let Some(token) = self.tokens.get(index) {
            if token.token != Token::Dollar {
                index += 1;
                continue;
            }

            let start = token.span.start;
            let mut end = token.span.end;
            let mut next = index + 1;

            while let Some(component) = self.tokens.get(next) {
                let is_component = matches!(component.token, Token::Ident(_) | Token::Dot);
                if !is_component || component.span.start != end {
                    break;
                }
                end = component.span.end;
                next += 1;
            }

            spans.push(Span::new(start, end));
            index = next;
        }

        spans
    }

    /// The assignment on the cursor's line once its key is complete and the
    /// cursor has reached the value, which is when a value can be completed.
    ///
    /// A started but still unlexable value (an unclosed string, for example)
    /// counts: the indexer stops at the first lexical error, so there is no
    /// value token yet, but the cursor is already in value position.
    pub(crate) fn assignment_before_cursor(&self, position: Position) -> Option<&SourceEntry> {
        let offset = self.offset_at(position)?;
        let entry = self.field_on_line(position.line)?;
        if entry.kind != SourceEntryKind::Assignment || offset <= entry.key_span.end {
            return None;
        }

        if entry.value_span.is_some() || self.attempted_value_after_key(entry) {
            Some(entry)
        } else {
            None
        }
    }

    /// True when the assignment's line has non-comment content after the key
    /// even though no value token was produced, which is what an incomplete
    /// buffer looks like while a value is being typed.
    fn attempted_value_after_key(&self, entry: &SourceEntry) -> bool {
        let line = self.lines.line_of(entry.key_span.start);
        let Some(span) = self.lines.line_span(line) else {
            return false;
        };
        let Some(rest) = self.text().get(entry.key_span.end..span.end) else {
            return false;
        };
        let rest = rest.trim_start();
        !rest.is_empty() && !rest.starts_with('#')
    }

    /// Object path of the innermost block open at `position`.
    pub(crate) fn scope_at(&self, position: Position) -> Vec<String> {
        self.offset_at(position)
            .map(|offset| self.scope_at_offset(offset))
            .unwrap_or_default()
    }

    pub(crate) fn scope_at_offset(&self, offset: usize) -> Vec<String> {
        let mut scope = Vec::new();

        for entry in &self.entries {
            if entry.kind != SourceEntryKind::Object {
                continue;
            }
            if entry.header_span.start > offset {
                break;
            }
            if entry.close_span.is_some_and(|close| close.start < offset) {
                continue;
            }
            scope = entry.path.clone();
        }

        scope
    }

    /// Field names already written in `scope` before `before_line`, so
    /// completions do not offer them twice.
    pub(crate) fn used_keys_in_scope(&self, scope: &[String], before_line: u32) -> Vec<String> {
        let mut used = Vec::new();

        for entry in &self.entries {
            if !entry.kind.is_field() || entry.scope.as_slice() != scope {
                continue;
            }
            if self.lines.line_of(entry.key_span.start) as u32 >= before_line {
                continue;
            }
            if let Some(name) = &entry.name
                && !used.contains(name)
            {
                used.push(name.clone());
            }
        }

        used
    }

    /// Every field entry whose full path is `path`, in document order.
    pub(crate) fn entries_with_path(&self, path: &[String]) -> Vec<&SourceEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.kind.is_field() && entry.path == path)
            .collect()
    }

    /// Value span of the assignment at `path`.
    pub(crate) fn value_span_for_path(&self, path: &[String]) -> Option<Span> {
        self.entries
            .iter()
            .find(|entry| entry.kind == SourceEntryKind::Assignment && entry.path == path)?
            .value_span
    }

    /// Where a new field goes inside the object at `path`: the line after its
    /// header, plus the indentation of that object's children.
    pub(crate) fn object_body_insert(&self, path: &[String]) -> Option<(Position, String)> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.kind == SourceEntryKind::Object && entry.path.as_slice() == path)?;
        let line = self.lines.line_of(entry.header_span.end);
        let indent = format!("{}  ", self.lines.indent(line));

        Some((Position::new(line as u32 + 1, 0), indent))
    }

    /// Objects with no `end` of their own.
    pub(crate) fn unclosed_objects(&self) -> Vec<&SourceEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.kind == SourceEntryKind::Object && entry.close_span.is_none())
            .collect()
    }

    /// Assignments whose key has no value and no started-but-incomplete value.
    pub(crate) fn missing_value_entries(&self) -> Vec<&SourceEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.kind == SourceEntryKind::Assignment
                    && entry.value_span.is_none()
                    && !self.attempted_value_after_key(entry)
            })
            .collect()
    }

    /// LSP range of an entry's key token.
    pub(crate) fn key_range(&self, entry: &SourceEntry) -> Range {
        self.lines.range(entry.key_span)
    }

    /// Source text covered by a single-line LSP range.
    pub(crate) fn text_in_range(&self, range: Range) -> Option<&str> {
        if range.start.line != range.end.line {
            return None;
        }

        let start = self.lines.position_to_byte(range.start)?;
        let end = self.lines.position_to_byte(range.end)?;
        self.text().get(start..end)
    }
}

/// True when the buffer opens with a `schema <name>:` block, which is how the
/// language server tells schema documents apart from configs.
pub(crate) fn starts_with_schema_block(text: &str) -> bool {
    let mut lexer = Lexer::new(text);
    let mut seen_keyword = false;

    while let Some(token) = next_token_or_stop(&mut lexer) {
        match token.token {
            Token::Newline => {}
            Token::Ident(name) if !seen_keyword && name == "schema" => seen_keyword = true,
            Token::Ident(_) | Token::String(_) if seen_keyword => {}
            Token::Colon if seen_keyword => return true,
            _ => return false,
        }
    }

    false
}

/// The name a key or value token carries.
fn token_name(token: &Token) -> Option<String> {
    match token {
        Token::Ident(name) | Token::String(name) => Some(name.clone()),
        _ => None,
    }
}

/// Span of a key token as an edit should see it: a quoted key narrows to the
/// quoted content so a rename does not eat the quotes.
fn key_span_of(token: &SpannedToken) -> Span {
    match &token.token {
        Token::String(_) => token.span.inner(),
        _ => token.span,
    }
}

fn field_path(scope: &[String], name: &str) -> Vec<String> {
    let mut path = scope.to_vec();
    path.push(name.to_string());
    path
}

/// Every token of `text`, stopped at the first lexical error so an incomplete
/// buffer still yields the tokens that are complete.
fn spanned_tokens(text: &str) -> Vec<SpannedToken> {
    let mut lexer = Lexer::new(text);

    std::iter::from_fn(|| next_token_or_stop(&mut lexer)).collect()
}

/// The next spanned token, or `None` once the input is exhausted or the lexer
/// reaches a lexeme it cannot finish, such as an unterminated string.
fn next_token_or_stop(lexer: &mut Lexer) -> Option<SpannedToken> {
    match lexer.next_token_spanned() {
        Ok(token) if token.token == Token::Eof => None,
        Ok(token) => Some(token),
        Err(_) => None,
    }
}

/// A block that is open while the indexer walks the token stream.
enum Frame {
    Object { entry: usize },
    Conditional,
}

/// Walks real tokens far enough to describe the document's structure.
///
/// Only the grammar the strict parser accepts is modelled; anything else is
/// skipped to the end of its statement rather than rejected.
struct IndexBuilder<'a> {
    tokens: &'a [SpannedToken],
    cursor: usize,
    entries: Vec<SourceEntry>,
    frames: Vec<Frame>,
    stray_closers: Vec<StrayCloserSpan>,
}

impl<'a> IndexBuilder<'a> {
    fn new(tokens: &'a [SpannedToken]) -> Self {
        Self {
            tokens,
            cursor: 0,
            entries: Vec::new(),
            frames: Vec::new(),
            stray_closers: Vec::new(),
        }
    }

    fn build(mut self) -> (Vec<SourceEntry>, Vec<StrayCloserSpan>) {
        self.run();
        (self.entries, self.stray_closers)
    }

    fn run(&mut self) {
        while let Some(token) = self.peek_token() {
            match token {
                Token::Eof => break,
                Token::Newline => self.cursor += 1,
                Token::At => self.metadata(),
                Token::Gather => self.gather_statement(),
                Token::End => self.close_object(),
                Token::EndIf => self.close_conditional(),
                Token::Else | Token::ElseIf => self.conditional_branch(),
                Token::If if self.if_starts_block() => self.conditional_header(),
                Token::Ident(_) | Token::String(_) => self.assignment_or_object(),
                // Tolerated: skip the statement and keep indexing.
                _ => {
                    self.scan_statement();
                }
            }
        }
    }

    fn peek_token(&self) -> Option<Token> {
        self.tokens
            .get(self.cursor)
            .map(|token| token.token.clone())
    }

    fn bump(&mut self) -> Option<&'a SpannedToken> {
        let tokens = self.tokens;
        let token = tokens.get(self.cursor)?;
        self.cursor += 1;
        Some(token)
    }

    /// Consume tokens up to the end of the current statement, returning the
    /// span covering them. Newlines inside brackets do not end a statement.
    fn scan_statement(&mut self) -> Option<Span> {
        let mut span: Option<Span> = None;
        let mut depth: usize = 0;

        while let Some(token) = self.peek_token() {
            match token {
                Token::Eof => break,
                Token::Newline if depth == 0 => break,
                Token::LBracket => depth += 1,
                Token::RBracket => depth = depth.saturating_sub(1),
                _ => {}
            }

            if let Some(token) = self.bump() {
                span = Some(match span {
                    Some(existing) => existing.merge(token.span),
                    None => token.span,
                });
            }
        }

        span
    }

    /// An `if` opens a conditional block when its line ends the header with a
    /// `:`; otherwise it is an inline value conditional and belongs to the
    /// statement it appears in.
    fn if_starts_block(&self) -> bool {
        let mut depth: usize = 0;

        for token in self.tokens.iter().skip(self.cursor + 1) {
            match &token.token {
                Token::LBracket => depth += 1,
                Token::RBracket => depth = depth.saturating_sub(1),
                Token::Colon if depth == 0 => return true,
                Token::Newline | Token::Eof if depth == 0 => return false,
                _ => {}
            }
        }

        false
    }

    fn current_scope(&self) -> Vec<String> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| match frame {
                Frame::Object { entry } => Some(self.entries[*entry].path.clone()),
                Frame::Conditional => None,
            })
            .unwrap_or_default()
    }

    fn push_entry(&mut self, entry: SourceEntry) {
        self.entries.push(entry);
    }

    fn assignment_or_object(&mut self) {
        let Some(key) = self.bump() else {
            return;
        };
        let Some(name) = token_name(&key.token) else {
            return;
        };

        let scope = self.current_scope();
        let key_span = key_span_of(key);

        match self.peek_token() {
            Some(Token::Colon) => {
                let Some(colon) = self.bump() else {
                    return;
                };
                let header_span = key.span.merge(colon.span);
                let path = field_path(&scope, &name);
                self.push_entry(SourceEntry {
                    kind: SourceEntryKind::Object,
                    scope,
                    path,
                    name: Some(name),
                    key_span,
                    header_span,
                    value_span: None,
                    close_span: None,
                });
                self.frames.push(Frame::Object {
                    entry: self.entries.len() - 1,
                });
            }
            Some(Token::Equals) => {
                let Some(equals) = self.bump() else {
                    return;
                };
                let value_span = self.scan_statement();
                let header_span = match value_span {
                    Some(value) => key.span.merge(value),
                    None => key.span.merge(equals.span),
                };
                self.push_assignment(scope, name, key_span, header_span, value_span);
            }
            _ => {
                let value_span = self.scan_statement();
                let header_span = match value_span {
                    Some(value) => key.span.merge(value),
                    None => key.span,
                };
                self.push_assignment(scope, name, key_span, header_span, value_span);
            }
        }
    }

    fn push_assignment(
        &mut self,
        scope: Vec<String>,
        name: String,
        key_span: Span,
        header_span: Span,
        value_span: Option<Span>,
    ) {
        let path = field_path(&scope, &name);
        self.push_entry(SourceEntry {
            kind: SourceEntryKind::Assignment,
            scope,
            path,
            name: Some(name),
            key_span,
            header_span,
            value_span,
            close_span: None,
        });
    }

    fn metadata(&mut self) {
        let Some(at) = self.bump() else {
            return;
        };

        let (key_span, name) = match self.peek_token() {
            // A directive's key span covers `@name`, which is the text a
            // diagnostic about it should underline.
            Some(Token::Ident(name)) => {
                let span = self.bump().map(key_span_of).unwrap_or(at.span);
                (at.span.merge(span), Some(name))
            }
            Some(Token::String(name)) => {
                let span = self.bump().map(key_span_of).unwrap_or(at.span);
                (at.span.merge(span), Some(name))
            }
            _ => (at.span, None),
        };

        let scope = self.current_scope();
        let value_span = self.scan_statement();
        let header_span = match value_span {
            Some(value) => at.span.merge(value),
            None => at.span,
        };

        self.push_entry(SourceEntry {
            kind: SourceEntryKind::Metadata,
            scope: scope.clone(),
            path: scope,
            name,
            key_span,
            header_span,
            value_span,
            close_span: None,
        });
    }

    fn gather_statement(&mut self) {
        let Some(keyword) = self.bump() else {
            return;
        };

        let scope = self.current_scope();
        let value_span = self.scan_statement();
        let header_span = match value_span {
            Some(value) => keyword.span.merge(value),
            None => keyword.span,
        };

        self.push_entry(SourceEntry {
            kind: SourceEntryKind::Gather,
            scope: scope.clone(),
            path: scope,
            name: None,
            key_span: keyword.span,
            header_span,
            value_span,
            close_span: None,
        });
    }

    fn conditional_header(&mut self) {
        let Some(keyword) = self.bump() else {
            return;
        };

        let scope = self.current_scope();
        let condition_span = self.scan_statement();
        let header_span = match condition_span {
            Some(span) => keyword.span.merge(span),
            None => keyword.span,
        };

        self.push_entry(SourceEntry {
            kind: SourceEntryKind::ConditionalHeader,
            scope: scope.clone(),
            path: scope,
            name: None,
            key_span: keyword.span,
            header_span,
            value_span: None,
            close_span: None,
        });
        self.frames.push(Frame::Conditional);
    }

    fn conditional_branch(&mut self) {
        let Some(keyword) = self.bump() else {
            return;
        };

        let scope = self.current_scope();
        let rest = self.scan_statement();
        let header_span = match rest {
            Some(span) => keyword.span.merge(span),
            None => keyword.span,
        };

        self.push_entry(SourceEntry {
            kind: SourceEntryKind::ConditionalBranch,
            scope: scope.clone(),
            path: scope,
            name: None,
            key_span: keyword.span,
            header_span,
            value_span: None,
            close_span: None,
        });
    }

    fn close_object(&mut self) {
        let Some(keyword) = self.bump() else {
            return;
        };

        match self.frames.pop() {
            Some(Frame::Object { entry }) => self.entries[entry].close_span = Some(keyword.span),
            // An `end` where a conditional is still open is a half-written
            // buffer rather than an object close; closing the conditional keeps
            // the enclosing object path intact for everything that follows.
            Some(Frame::Conditional) => {}
            None => self.stray_closers.push(StrayCloserSpan {
                kind: StrayCloser::End,
                span: keyword.span,
            }),
        }
    }

    fn close_conditional(&mut self) {
        let Some(keyword) = self.bump() else {
            return;
        };

        let scope = self.current_scope();
        self.push_entry(SourceEntry {
            kind: SourceEntryKind::ConditionalEnd,
            scope: scope.clone(),
            path: scope,
            name: None,
            key_span: keyword.span,
            header_span: keyword.span,
            value_span: None,
            close_span: None,
        });

        if !self
            .frames
            .iter()
            .any(|frame| matches!(frame, Frame::Conditional))
        {
            self.stray_closers.push(StrayCloserSpan {
                kind: StrayCloser::EndIf,
                span: keyword.span,
            });
            return;
        }

        // Close the innermost conditional, tolerating a buffer where an object
        // opened inside it was never closed.
        while let Some(frame) = self.frames.pop() {
            if matches!(frame, Frame::Conditional) {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position(line: u32, character: u32) -> Position {
        Position::new(line, character)
    }

    /// `app:` with a conditional block, then an assignment that belongs to
    /// `app` rather than to the conditional.
    const CONDITIONAL_CONFIG: &str = "app:\n  if debug:\n    feature true\n  else:\n    feature false\n  endif\n  name \"Rune\"\nend\n";

    #[test]
    fn line_index_counts_utf16_units() {
        let index = LineIndex::new("app:\nname \"😀\"\n");

        // The emoji is one code point, two UTF-16 units and four bytes.
        assert_eq!(index.line_count(), 3);
        assert_eq!(index.byte_to_position(0), position(0, 0));
        assert_eq!(index.byte_to_position(16), position(1, 9));
        assert_eq!(index.position_to_byte(position(1, 9)), Some(16));
        // A character past the end of a line clamps to its end.
        assert_eq!(index.position_to_byte(position(1, 99)), Some(16));
        // A character inside the surrogate pair clamps to the next boundary,
        // which is where the closing quote starts.
        assert_eq!(index.position_to_byte(position(1, 7)), Some(15));
        assert_eq!(index.position_to_byte(position(1, 8)), Some(15));
        assert_eq!(index.byte_to_position(15), position(1, 8));
        assert_eq!(
            index.full_range(),
            Range::new(position(0, 0), position(2, 0))
        );
    }

    #[test]
    fn line_index_handles_crlf_and_missing_final_newline() {
        let index = LineIndex::new("app:\r\nname \"x\"");

        assert_eq!(index.line_text(0), Some("app:"));
        assert_eq!(index.line_text(1), Some("name \"x\""));
        assert_eq!(
            index.full_range(),
            Range::new(position(0, 0), position(1, 8))
        );
        assert_eq!(index.indent(1), "");
    }

    #[test]
    fn conditional_headers_and_endif_are_not_fields() {
        let index = SourceIndex::new(CONDITIONAL_CONFIG);
        let names: Vec<String> = index
            .entries()
            .iter()
            .filter(|entry| entry.kind.is_field())
            .map(|entry| entry.path.join("."))
            .collect();

        assert_eq!(names, vec!["app", "app.feature", "app.feature", "app.name"]);
        assert!(
            !names.iter().any(|name| name.contains("if")),
            "the conditional header must never become a field: {names:?}"
        );
    }

    #[test]
    fn scope_survives_endif_and_branches() {
        let index = SourceIndex::new(CONDITIONAL_CONFIG);

        // `name "Rune"` is line 6 and still belongs to `app`.
        let field = index.field_on_line(6).expect("the trailing assignment");
        assert_eq!(field.path, vec!["app", "name"]);
        assert_eq!(field.scope, vec!["app"]);
        assert_eq!(index.scope_at(position(6, 3)), vec!["app"]);

        // Both branches keep the enclosing object as their scope.
        assert_eq!(index.scope_at(position(2, 5)), vec!["app"]);
        assert_eq!(index.scope_at(position(4, 5)), vec!["app"]);
        // A blank line inside the conditional is still inside `app`.
        assert_eq!(index.scope_at(position(3, 0)), vec!["app"]);
    }

    #[test]
    fn scope_tracks_nested_objects_and_their_closers() {
        let index = SourceIndex::new("app:\n  server:\n    port 8080\n  end\n  name \"x\"\nend\n");

        assert_eq!(index.scope_at(position(2, 4)), vec!["app", "server"]);
        // Between `end` and the next statement the object is closed again.
        assert_eq!(index.scope_at(position(4, 2)), vec!["app"]);
        // On the closing `end` itself the object is still the innermost scope.
        assert_eq!(index.scope_at(position(3, 2)), vec!["app", "server"]);
        assert_eq!(index.unclosed_objects().len(), 0);
    }

    #[test]
    fn quoted_keys_use_their_decoded_name_and_content_span() {
        let index = SourceIndex::new("\"$var.mod+r\" \"reload\"\n");

        let entry = index.entries().first().expect("one entry");
        assert_eq!(entry.kind, SourceEntryKind::Assignment);
        assert_eq!(entry.name.as_deref(), Some("$var.mod+r"));
        assert_eq!(
            &"\"$var.mod+r\" \"reload\"\n"[entry.key_span.start..entry.key_span.end],
            "$var.mod+r"
        );
        assert_eq!(
            &"\"$var.mod+r\" \"reload\"\n"
                [entry.value_span.unwrap().start..entry.value_span.unwrap().end],
            "\"reload\""
        );
    }

    #[test]
    fn comment_marker_inside_literals_does_not_start_a_comment() {
        let index = SourceIndex::new("app:\n  name \"a#b\" # the name\n  color '#fff'\nend\n");

        let name = index.field_on_line(1).expect("a field on line 1");
        assert_eq!(name.path, vec!["app", "name"]);
        assert_eq!(
            &index.text()[name.value_span.unwrap().start..name.value_span.unwrap().end],
            "\"a#b\""
        );

        // A comment-only line produces no entry at all.
        assert!(
            index
                .field_on_line(2)
                .is_some_and(|entry| entry.name.as_deref() == Some("color"))
        );

        let commented = SourceIndex::new("app:\n  # name \"hidden\"\nend\n");
        assert!(
            commented
                .entries()
                .iter()
                .all(|entry| entry.name.as_deref() != Some("name")),
            "a commented-out assignment must not be indexed"
        );
    }

    #[test]
    fn regex_and_multiline_arrays_keep_their_value_spans() {
        let text = "app:\n  pattern r\"a#\\d+\"\n  plugins [\n    \"a#b\"\n  ]\nend\n";
        let index = SourceIndex::new(text);

        let pattern = index.field_on_line(1).expect("a field on line 1");
        assert_eq!(pattern.name.as_deref(), Some("pattern"));
        assert_eq!(
            &text[pattern.value_span.unwrap().start..pattern.value_span.unwrap().end],
            "r\"a#\\d+\""
        );

        let plugins = index.field_on_line(2).expect("a field on line 2");
        let value = plugins.value_span.expect("an array value");
        assert!(text[value.start..value.end].starts_with('['));
        assert_eq!(&text[value.start..value.end], "[\n    \"a#b\"\n  ]");
    }

    #[test]
    fn inline_conditionals_belong_to_their_statement() {
        let index = SourceIndex::new("app:\n  level if debug \"high\" else \"low\"\nend\n");

        assert_eq!(index.entries().len(), 2, "{:#?}", index.entries());
        let level = index.field_on_line(1).expect("an assignment on line 1");
        assert_eq!(level.kind, SourceEntryKind::Assignment);
        assert_eq!(level.path, vec!["app", "level"]);
        assert!(
            index
                .entries()
                .iter()
                .all(|entry| entry.kind != SourceEntryKind::ConditionalHeader),
            "an inline conditional is not a block"
        );
    }

    #[test]
    fn unclosed_value_is_value_context_not_a_missing_value() {
        let index = SourceIndex::new("app:\n  environment \"dev\nend\n");

        assert!(
            index.missing_value_entries().is_empty(),
            "an unclosed string is an incomplete value, not a missing one"
        );
        assert_eq!(
            index
                .assignment_before_cursor(position(1, 16))
                .and_then(|entry| entry.name.clone()),
            Some("environment".into()),
            "a cursor inside the unclosed value must still see the assignment"
        );
        assert!(
            index.assignment_before_cursor(position(1, 12)).is_none(),
            "a cursor still on the key is not value context"
        );
    }

    #[test]
    fn missing_values_and_stray_closers_are_recorded() {
        let index = SourceIndex::new("app:\n  name\n  server:\n    host \"localhost\"\n");

        let missing: Vec<&str> = index
            .missing_value_entries()
            .iter()
            .filter_map(|entry| entry.name.as_deref())
            .collect();
        assert_eq!(missing, vec!["name"]);

        let unclosed: Vec<&str> = index
            .unclosed_objects()
            .iter()
            .filter_map(|entry| entry.name.as_deref())
            .collect();
        assert_eq!(unclosed, vec!["app", "server"]);
        assert!(index.stray_closers().is_empty());

        let stray = SourceIndex::new("name \"x\"\nend\nendif\n");
        let keywords: Vec<&str> = stray
            .stray_closers()
            .iter()
            .map(|closer| closer.kind.keyword())
            .collect();
        assert_eq!(keywords, vec!["end", "endif"]);
    }

    #[test]
    fn end_of_file_keeps_the_recognized_entries() {
        let index = SourceIndex::new("app:\n  name \"Rune\"\n");

        assert_eq!(index.entries().len(), 2);
        assert_eq!(index.unclosed_objects().len(), 1);
        assert_eq!(
            index.field_on_line(1).map(|entry| entry.path.clone()),
            Some(vec!["app".to_string(), "name".to_string()])
        );
    }

    #[test]
    fn incomplete_buffers_keep_a_partial_index() {
        // The trailing quote is still open: everything before it is indexed.
        let index = SourceIndex::new("app:\n  name \"Rune\n");

        assert_eq!(index.entries().len(), 2);
        assert_eq!(index.scope_at(position(1, 2)), vec!["app"]);
    }

    #[test]
    fn key_spans_back_rename_ranges() {
        let index = SourceIndex::new("app:\n  name \"Rune\"\nend\n");

        let entry = index
            .field_key_at(position(1, 3))
            .expect("cursor on the key");
        assert_eq!(
            index.key_range(entry),
            Range::new(position(1, 2), position(1, 6))
        );

        // A cursor on the value belongs to no key token.
        assert!(index.field_key_at(position(1, 9)).is_none());
        // A cursor on a conditional header belongs to no field.
        let conditional = SourceIndex::new(CONDITIONAL_CONFIG);
        assert!(conditional.field_key_at(position(1, 5)).is_none());
    }

    #[test]
    fn used_keys_and_insert_points_are_scoped() {
        let index =
            SourceIndex::new("app:\n  name \"RuneApp\"\n  server:\n    port 8080\n  end\nend\n");
        let app = vec!["app".to_string()];
        let server = vec!["app".to_string(), "server".to_string()];

        assert_eq!(index.used_keys_in_scope(&app, 4), vec!["name", "server"]);
        assert_eq!(index.used_keys_in_scope(&server, 4), vec!["port"]);
        // The cursor's own line is not counted as already used.
        assert!(index.used_keys_in_scope(&app, 1).is_empty());
        assert_eq!(
            index.used_keys_in_scope(&app, 2),
            vec!["name"],
            "the object header on the cursor's line is excluded"
        );

        let (position, indent) = index.object_body_insert(&server).expect("an insert point");
        assert_eq!(position, Position::new(3, 0));
        assert_eq!(indent, "    ");
    }

    #[test]
    fn value_spans_back_quick_fixes() {
        let index = SourceIndex::new("app:\n  port \"8080\"\nend\n");

        let span = index
            .value_span_for_path(&["app".to_string(), "port".to_string()])
            .expect("a value span");
        let range = index.lines().range(span);
        assert_eq!(index.text_in_range(range), Some("\"8080\""));
        assert!(
            index
                .value_span_for_path(&["app".to_string(), "missing".to_string()])
                .is_none()
        );
    }

    #[test]
    fn metadata_and_gather_are_not_fields() {
        let index = SourceIndex::new(
            "@author \"Dustin\"\ngather \"defaults.rune\" as defaults\napp:\n  name \"x\"\nend\n",
        );

        let kinds: Vec<SourceEntryKind> = index.entries().iter().map(|entry| entry.kind).collect();
        assert_eq!(
            kinds,
            vec![
                SourceEntryKind::Metadata,
                SourceEntryKind::Gather,
                SourceEntryKind::Object,
                SourceEntryKind::Assignment
            ]
        );
        assert!(index.field_on_line(0).is_none());
        assert!(index.field_on_line(1).is_none());
        // Before the first object opens, statements are at top level.
        assert!(index.scope_at(position(1, 0)).is_empty());
        assert_eq!(index.scope_at(position(3, 2)), vec!["app"]);
    }

    #[test]
    fn schema_documents_are_recognized_from_tokens() {
        assert!(starts_with_schema_block(
            "# comment\nschema app:\n  name string\nend\n"
        ));
        assert!(starts_with_schema_block("schema \"app\":\nend\n"));
        assert!(!starts_with_schema_block("app:\n  name \"x\"\nend\n"));
        assert!(!starts_with_schema_block(
            "@author \"x\"\nschema app:\nend\n"
        ));
        assert!(!starts_with_schema_block(""));
    }
}
