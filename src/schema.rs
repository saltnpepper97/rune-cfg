// Author: Dustin Pilgrim
// License: MIT

use crate::RuneError;
use crate::ast::Value;
use crate::lexer::{Lexer, SpannedToken, Token};
use crate::source::{LineIndex, Span};

#[derive(Debug, Clone)]
pub struct SchemaDocument {
    pub blocks: Vec<SchemaBlock>,
}

#[derive(Debug, Clone)]
pub struct SchemaBlock {
    pub root: String,
    pub fields: Vec<SchemaField>,
    pub line: usize,
    pub(crate) name_span: Span,
}

#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub kind: SchemaType,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<Value>,
    pub range: Option<(f64, f64)>,
    pub fields: Vec<SchemaField>,
    pub line: usize,
    pub(crate) name_span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    String,
    Int,
    Float,
    Number,
    Bool,
    Regex,
    Null,
    Any,
    Array(Box<SchemaType>),
    Enum(Vec<String>),
    Object,
}

impl PartialEq for SchemaDocument {
    fn eq(&self, other: &Self) -> bool {
        self.blocks == other.blocks
    }
}

impl PartialEq for SchemaBlock {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.fields == other.fields && self.line == other.line
    }
}

impl PartialEq for SchemaField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.kind == other.kind
            && self.description == other.description
            && self.required == other.required
            && self.default == other.default
            && self.range == other.range
            && self.fields == other.fields
            && self.line == other.line
    }
}

impl SchemaDocument {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, RuneError> {
        let content = std::fs::read_to_string(&path).map_err(|e| RuneError::FileError {
            message: format!("Failed to read schema file: {}", e),
            path: path.as_ref().to_string_lossy().to_string(),
            hint: Some("Check that the schema file exists and is readable".into()),
            code: Some(601),
        })?;
        Self::from_str(&content)
    }

    // Retain the established inherent API alongside the FromStr impl below.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> Result<Self, RuneError> {
        SchemaParser::new(content).parse()
    }
}

impl std::str::FromStr for SchemaDocument {
    type Err = RuneError;

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        SchemaDocument::from_str(content)
    }
}

struct SchemaParser<'a> {
    source: &'a str,
    lines: LineIndex,
    lexer: Lexer<'a>,
    current: Option<SpannedToken>,
    leading_start: usize,
    statement_spans: Vec<Span>,
}

impl<'a> SchemaParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            lines: LineIndex::new(source),
            lexer: Lexer::new(source),
            current: None,
            leading_start: 0,
            statement_spans: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<SchemaDocument, RuneError> {
        let mut blocks = Vec::new();

        loop {
            let comments = self.skip_layout()?;
            let token = self.peek()?.clone();
            match token.token {
                Token::Eof => break,
                Token::Ident(ref keyword) if keyword == "schema" => {
                    self.take();
                    let (root, name_span) =
                        self.expect_name("Expected schema name", "Use: schema app:")?;
                    self.expect_token(
                        Token::Colon,
                        "Expected ':' after schema name",
                        "Use: schema app:",
                    )?;
                    self.expect_boundary("Expected end of schema declaration", "Use: schema app:")?;
                    let line = self.line_of(name_span);
                    let fields = self.parse_fields()?;
                    blocks.push(SchemaBlock {
                        root,
                        fields,
                        line,
                        name_span,
                    });
                }
                _ => {
                    return Err(self.error_at(
                        &token,
                        format!("Expected schema block, got {}", token_label(&token.token)),
                        "Use: schema <name>:",
                    ));
                }
            }

            // Comments before a schema block are deliberately ignored.
            let _ = comments;
        }

        Ok(SchemaDocument { blocks })
    }

    fn parse_fields(&mut self) -> Result<Vec<SchemaField>, RuneError> {
        let mut fields = Vec::new();

        loop {
            let comments = self.skip_layout()?;
            let token = self.peek()?.clone();

            match token.token {
                Token::End => {
                    self.take();
                    self.expect_boundary(
                        "Expected end of schema block",
                        "Close schema blocks with 'end'",
                    )?;
                    return Ok(fields);
                }
                Token::Eof => {
                    return Err(self.error_at(
                        &token,
                        "Unclosed schema block",
                        "Close schema blocks and nested objects with 'end'",
                    ));
                }
                _ => {
                    let mut field = self.parse_field()?;
                    field.description = take_description(comments);
                    fields.push(field);
                }
            }
        }
    }

    fn parse_field(&mut self) -> Result<SchemaField, RuneError> {
        let (name, name_span) =
            self.expect_name("Expected schema field name", "Use: name string required")?;
        let line = self.line_of(name_span);

        if self.peek()?.token == Token::Colon {
            self.take();
            self.expect_boundary("Expected newline after object field ':'", "Use: server:")?;
            let fields = self.parse_fields()?;
            return Ok(SchemaField {
                name,
                kind: SchemaType::Object,
                description: None,
                required: false,
                default: None,
                range: None,
                fields,
                line,
                name_span,
            });
        }

        let next = self.peek()?.clone();
        if matches!(next.token, Token::Newline | Token::Eof) {
            return Err(self.error_at(
                &next,
                format!("Expected type for schema field '{}'", name),
                "Use: name string required",
            ));
        }

        let kind = self.parse_type()?;
        let mut required = false;
        let mut default = None;
        let mut range = None;

        loop {
            let token = self.peek()?.clone();
            match token.token {
                Token::Newline | Token::Eof => {
                    self.consume_boundary();
                    break;
                }
                Token::Ident(ref modifier) if modifier == "required" => {
                    self.take();
                    required = true;
                }
                Token::Ident(ref modifier) if modifier == "range" => {
                    self.take();
                    range = Some(self.parse_range()?);
                }
                Token::Ident(ref modifier) if modifier == "default" => {
                    self.take();
                    default = Some(self.parse_default()?);
                }
                _ => {
                    return Err(self.error_at(
                        &token,
                        format!("Unexpected schema modifier {}", token_label(&token.token)),
                        "Use modifiers like required, range min..max, or default value",
                    ));
                }
            }
        }

        Ok(SchemaField {
            name,
            kind,
            description: None,
            required,
            default,
            range,
            fields: Vec::new(),
            line,
            name_span,
        })
    }

    fn parse_type(&mut self) -> Result<SchemaType, RuneError> {
        let token = self.peek()?.clone();
        match token.token {
            Token::Ident(ref word) if word == "enum" => {
                self.take();
                self.expect_token(
                    Token::LBracket,
                    "Expected enum values",
                    "Use: environment enum [\"dev\", \"prod\"]",
                )?;
                let mut values = Vec::new();
                loop {
                    let value = self.peek()?.clone();
                    match value.token {
                        Token::RBracket => {
                            self.take();
                            break;
                        }
                        Token::Newline => {
                            self.take();
                        }
                        Token::Eof => {
                            return Err(self.error_at(
                                &value,
                                "Expected enum values",
                                "Use: environment enum [\"dev\", \"prod\"]",
                            ));
                        }
                        _ => {
                            let Some(value) = enum_value(&value.token) else {
                                return Err(self.error_at(
                                    &value,
                                    "Invalid enum value",
                                    "Use strings, numbers, booleans, or null in enum brackets",
                                ));
                            };
                            self.take();
                            values.push(value);
                        }
                    }
                }
                Ok(SchemaType::Enum(values))
            }
            Token::LBracket => {
                self.take();
                if self.peek()?.token == Token::RBracket {
                    let token = self.peek()?.clone();
                    return Err(self.error_at(
                        &token,
                        "Expected array type",
                        "Use: plugins [string]",
                    ));
                }
                let inner = self.parse_type()?;
                self.expect_token(
                    Token::RBracket,
                    "Expected array type",
                    "Use: plugins [string]",
                )?;
                Ok(SchemaType::Array(Box::new(inner)))
            }
            Token::Null => {
                self.take();
                Ok(SchemaType::Null)
            }
            Token::Ident(ref word) => {
                self.take();
                let kind = match word.as_str() {
                    "string" | "str" => SchemaType::String,
                    "int" | "integer" => SchemaType::Int,
                    "float" => SchemaType::Float,
                    "number" => SchemaType::Number,
                    "bool" | "boolean" => SchemaType::Bool,
                    "regex" => SchemaType::Regex,
                    "null" => SchemaType::Null,
                    "any" => SchemaType::Any,
                    "object" => SchemaType::Object,
                    _ => {
                        return Err(self.error_at(
                            &token,
                            format!("Unknown schema type '{}'", word),
                            "Use string, int, float, number, bool, regex, null, any, enum, object, or [type]",
                        ));
                    }
                };
                Ok(kind)
            }
            _ => Err(self.error_at(
                &token,
                format!("Unknown schema type {}", token_label(&token.token)),
                "Use string, int, float, number, bool, regex, null, any, enum, object, or [type]",
            )),
        }
    }

    fn parse_range(&mut self) -> Result<(f64, f64), RuneError> {
        let minimum = self.expect_number(
            "Invalid range minimum",
            "Use numeric range bounds like 1..65535",
        )?;
        self.expect_token(
            Token::Dot,
            "Expected range in min..max form",
            "Use: port int range 1..65535",
        )?;
        self.expect_token(
            Token::Dot,
            "Expected range in min..max form",
            "Use: port int range 1..65535",
        )?;
        let maximum = self.expect_number(
            "Invalid range maximum",
            "Use numeric range bounds like 1..65535",
        )?;
        Ok((minimum, maximum))
    }

    fn parse_default(&mut self) -> Result<Value, RuneError> {
        let token = self.peek()?.clone();
        let value = match token.token {
            Token::String(value) => Value::String(value),
            Token::Number(value) => Value::Number(value),
            Token::Bool(value) => Value::Bool(value),
            Token::Null => Value::Null,
            Token::Ident(value) => Value::String(value),
            _ => {
                return Err(self.error_at(
                    &token,
                    "Expected value after default",
                    "Use: debug bool default false",
                ));
            }
        };
        self.take();
        Ok(value)
    }

    fn expect_number(&mut self, message: &str, hint: &str) -> Result<f64, RuneError> {
        let token = self.peek()?.clone();
        if let Token::Number(number) = token.token {
            self.take();
            Ok(number)
        } else {
            Err(self.error_at(&token, message, hint))
        }
    }

    fn expect_name(&mut self, message: &str, hint: &str) -> Result<(String, Span), RuneError> {
        let token = self.peek()?.clone();
        let span = token.span;
        let quoted = matches!(&token.token, Token::String(_));
        let name = match token.token {
            Token::Ident(name) | Token::String(name) => {
                self.take();
                name
            }
            _ => return Err(self.error_at(&token, message, hint)),
        };
        let name_span = if quoted { span.inner() } else { span };
        Ok((name, name_span))
    }

    fn expect_token(
        &mut self,
        expected: Token,
        message: &str,
        hint: &str,
    ) -> Result<(), RuneError> {
        let token = self.peek()?.clone();
        if token.token == expected {
            self.take();
            Ok(())
        } else {
            Err(self.error_at(&token, message, hint))
        }
    }

    fn expect_boundary(&mut self, message: &str, hint: &str) -> Result<(), RuneError> {
        let token = self.peek()?.clone();
        match token.token {
            Token::Newline => {
                self.take();
                Ok(())
            }
            Token::Eof => Ok(()),
            _ => Err(self.error_at(&token, message, hint)),
        }
    }

    fn consume_boundary(&mut self) {
        if matches!(
            self.current.as_ref().map(|token| &token.token),
            Some(Token::Newline)
        ) {
            self.take();
        }
    }

    fn skip_layout(&mut self) -> Result<Vec<String>, RuneError> {
        let mut comments = Vec::new();
        loop {
            let token = self.peek()?.clone();
            comments.extend(leading_comments(
                self.source,
                self.leading_start,
                token.span.start,
                &self.statement_spans,
            ));
            if token.token == Token::Newline {
                self.take();
                continue;
            }
            return Ok(comments);
        }
    }

    fn peek(&mut self) -> Result<&SpannedToken, RuneError> {
        if self.current.is_none() {
            self.current = Some(self.lexer.next_token_spanned()?);
        }
        Ok(self.current.as_ref().expect("token is present"))
    }

    fn take(&mut self) -> SpannedToken {
        let token = self.current.take().expect("peeked token");
        if matches!(token.token, Token::Newline | Token::Eof) {
            self.leading_start = token.span.end;
            self.statement_spans.clear();
        } else {
            self.statement_spans.push(token.span);
        }
        token
    }

    fn line_of(&self, span: Span) -> usize {
        self.lines.line_of(span.start) + 1
    }

    fn error_at(
        &self,
        token: &SpannedToken,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> RuneError {
        let line = self.lines.line_of(token.span.start);
        let line_start = self
            .lines
            .line_span(line)
            .map(|span| span.start)
            .unwrap_or(token.span.start);
        let column = self
            .source
            .get(line_start..token.span.start)
            .map(|text| text.chars().count() + 1)
            .unwrap_or(1);
        schema_error(message, line + 1, column, hint)
    }
}

fn enum_value(token: &Token) -> Option<String> {
    match token {
        Token::String(value) | Token::Ident(value) | Token::Regex(value) => Some(value.clone()),
        Token::Number(value) => Some(value.to_string()),
        Token::Bool(value) => Some(value.to_string()),
        Token::Null => Some("null".into()),
        _ => None,
    }
}

fn leading_comments(source: &str, start: usize, end: usize, protected: &[Span]) -> Vec<String> {
    if start >= end {
        return Vec::new();
    }

    let mut comments = Vec::new();
    let Some(text) = source.get(start..end) else {
        return comments;
    };

    let mut line_start = 0;
    for line_with_terminator in text.split_inclusive('\n') {
        let line = line_with_terminator
            .strip_suffix('\n')
            .unwrap_or(line_with_terminator)
            .strip_suffix('\r')
            .unwrap_or_else(|| {
                line_with_terminator
                    .strip_suffix('\n')
                    .unwrap_or(line_with_terminator)
            });
        let leading = line.len() - line.trim_start().len();
        let marker = start + line_start + leading;
        let trimmed = &line[leading..];

        if let Some(comment) = trimmed.strip_prefix('#')
            && !protected.iter().any(|span| span.touches(marker))
        {
            let comment = comment.trim();
            if !comment.is_empty() {
                comments.push(comment.to_string());
            }
        }

        line_start += line_with_terminator.len();
    }

    comments
}

fn take_description(comments: Vec<String>) -> Option<String> {
    (!comments.is_empty()).then(|| comments.join("\n"))
}

fn token_label(token: &Token) -> String {
    token.describe()
}

fn schema_error(
    message: impl Into<String>,
    line: usize,
    column: usize,
    hint: impl Into<String>,
) -> RuneError {
    RuneError::SyntaxError {
        message: message.into(),
        line,
        column,
        hint: Some(hint.into()),
        code: Some(600),
    }
}

impl SchemaType {
    pub fn name(&self) -> String {
        match self {
            SchemaType::String => "string".into(),
            SchemaType::Int => "int".into(),
            SchemaType::Float => "float".into(),
            SchemaType::Number => "number".into(),
            SchemaType::Bool => "bool".into(),
            SchemaType::Regex => "regex".into(),
            SchemaType::Null => "null".into(),
            SchemaType::Any => "any".into(),
            SchemaType::Array(inner) => format!("[{}]", inner.name()),
            SchemaType::Enum(values) => format!("enum [{}]", values.join(", ")),
            SchemaType::Object => "object".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_schema_blocks() {
        let schema = SchemaDocument::from_str(
            r#"
schema app:
  name string required
  debug bool default false
  environment enum ["dev", "staging", "production"]
  server:
    host string required
    port int range 1..65535 default 8080
  end
  plugins [string]
end
"#,
        )
        .expect("schema should parse");

        assert_eq!(schema.blocks.len(), 1);
        assert_eq!(schema.blocks[0].root, "app");
        assert_eq!(schema.blocks[0].fields.len(), 5);
        assert_eq!(schema.blocks[0].fields[3].kind, SchemaType::Object);
    }

    #[test]
    fn leading_comments_become_field_descriptions() {
        let schema = SchemaDocument::from_str(
            r#"
schema app:
  # Environment shown in editor hover.
  # Usually production for deployed apps.
  environment enum ["dev", "production"] required
  # Server settings.
  server:
    # Public hostname.
    host string required
  end
end
"#,
        )
        .expect("schema should parse");

        let environment = &schema.blocks[0].fields[0];
        assert_eq!(
            environment.description.as_deref(),
            Some("Environment shown in editor hover.\nUsually production for deployed apps.")
        );

        let server = &schema.blocks[0].fields[1];
        assert_eq!(server.description.as_deref(), Some("Server settings."));
        assert_eq!(
            server.fields[0].description.as_deref(),
            Some("Public hostname.")
        );
    }

    #[test]
    fn parser_supports_quoted_names_and_exact_columns() {
        let schema = SchemaDocument::from_str(
            "schema \"app.name\":\n  \"na\\me\" string default \"a#b\"\nend\n",
        )
        .unwrap();

        assert_eq!(schema.blocks[0].root, "app.name");
        assert_eq!(schema.blocks[0].fields[0].name, "name");
        assert_eq!(schema.blocks[0].fields[0].fields.len(), 0);
        assert_eq!(schema.blocks[0].name_span, Span::new(8, 16));
        assert_eq!(schema.blocks[0].fields[0].name_span, Span::new(22, 27));
        assert_eq!(
            schema.blocks[0].fields[0].default,
            Some(Value::String("a#b".into()))
        );
    }

    #[test]
    fn comments_and_literals_follow_statement_boundaries() {
        let schema = SchemaDocument::from_str(
            "schema app:\n  # The value description.\n  value string default \"a#b\" # inline\n  # The null description.\n  nothing null\nend\n",
        )
        .unwrap();

        assert_eq!(
            schema.blocks[0].fields[0].description.as_deref(),
            Some("The value description.")
        );
        assert_eq!(
            schema.blocks[0].fields[0].default,
            Some(Value::String("a#b".into()))
        );
        assert_eq!(
            schema.blocks[0].fields[1].description.as_deref(),
            Some("The null description.")
        );
        assert_eq!(schema.blocks[0].fields[1].kind, SchemaType::Null);
    }

    #[test]
    fn parser_preserves_semantics_across_line_endings_and_ranges() {
        let lf = SchemaDocument::from_str(
            "schema app:\n  port int range -10..10\n  ratio number range 0.5..1.5\nend\n",
        )
        .unwrap();
        let crlf = SchemaDocument::from_str(
            "schema app:\r\n  port int range -10..10\r\n  ratio number range 0.5..1.5\r\nend\r\n",
        )
        .unwrap();

        assert_eq!(lf, crlf);
        assert_eq!(lf.blocks[0].fields[0].line, 2);
        assert_eq!(crlf.blocks[0].fields[0].line, 2);
        assert_eq!(lf.blocks[0].fields[0].range, Some((-10.0, 10.0)));
    }

    #[test]
    fn parser_reports_real_error_columns() {
        let error = SchemaDocument::from_str("schema app:\n  value mystery\nend\n").unwrap_err();
        assert_eq!(
            error,
            RuneError::SyntaxError {
                message: "Unknown schema type 'mystery'".into(),
                line: 2,
                column: 9,
                hint: Some("Use string, int, float, number, bool, regex, null, any, enum, object, or [type]".into()),
                code: Some(600),
            }
        );

        let error =
            SchemaDocument::from_str("schema app:\n  \u{10400} mystery\nend\n").unwrap_err();
        assert!(matches!(
            error,
            RuneError::SyntaxError {
                line: 2,
                column: 5,
                code: Some(600),
                ..
            }
        ));
    }

    #[test]
    fn comments_before_blocks_are_ignored_and_hashes_survive_enum_values() {
        let schema = SchemaDocument::from_str(
            "schema app:\n  \"weird\" enum [\"a#b\", plain] required\nend\n# Not a description.\nschema other:\n  value string\nend\n",
        )
        .unwrap();

        assert_eq!(schema.blocks.len(), 2);
        assert_eq!(
            schema.blocks[0].fields[0].kind,
            SchemaType::Enum(vec!["a#b".into(), "plain".into()]),
            "a '#' inside an enum value must not truncate it"
        );
        assert_eq!(
            schema.blocks[1].fields[0].description, None,
            "comments before a schema block must not become descriptions"
        );
    }
}
