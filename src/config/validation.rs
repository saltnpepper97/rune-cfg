// Author: Dustin Pilgrim
// License: MIT

use super::*;
use std::collections::HashMap;

use crate::diagnostic::{RuneDiagnostic, SourceRange};
use crate::schema::{SchemaDocument, SchemaField, SchemaType};
use crate::source::{SourceEntryKind, SourceIndex, Span};

impl RuneConfig {
    pub fn get_validated<T, F>(
        &self,
        path: &str,
        validator: F,
        valid_values: &str,
    ) -> Result<T, RuneError>
    where
        T: TryFrom<Value, Error = RuneError>,
        F: FnOnce(&T) -> bool,
    {
        let value = self.get_value(path)?;
        let typed_value = T::try_from(value)?;

        if !validator(&typed_value) {
            let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
            return Err(RuneError::ValidationError {
                message: format!("Invalid value for `{}`\nExpected: {}", path, valid_values),
                line,
                column: 0,
                hint: Some(format!(
                    "Valid values are: {}\n  → {}",
                    valid_values, snippet
                )),
                code: Some(450),
            });
        }

        Ok(typed_value)
    }

    pub fn get_string_enum(
        &self,
        path: &str,
        allowed_values: &[&str],
    ) -> Result<String, RuneError> {
        let value = self.get_value(path)?;

        let string_value = match value.into_primary() {
            Value::String(s) => s,
            value => {
                return Err(RuneError::TypeError {
                    message: format!("Expected string for `{}`, got {:?}", path, value),
                    line: 0,
                    column: 0,
                    hint: Some("Use a string value in your config".into()),
                    code: Some(401),
                });
            }
        };

        let lower_value = string_value.to_lowercase();

        if !allowed_values
            .iter()
            .any(|&v| v.to_lowercase() == lower_value)
        {
            let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
            return Err(RuneError::ValidationError {
                message: format!("Invalid value '{}' for `{}`", string_value, path),
                line,
                column: 0,
                hint: Some(format!(
                    "Expected one of: {}\n  → {}",
                    allowed_values.join(", "),
                    snippet
                )),
                code: Some(451),
            });
        }

        Ok(string_value)
    }

    pub fn path_exists_in_content(&self, path: &str) -> bool {
        let (line, _) = helpers::find_config_line(path, &self.raw_content);
        line > 0
    }

    /// Validate `schema` against this config.
    ///
    /// Diagnostics carry the exact key spans of `raw_content`; the language
    /// server uses [`RuneConfig::validate_schema_with_source`] so the index it
    /// already built for the buffer is the one the ranges come from.
    pub fn validate_schema(&self, schema: &SchemaDocument) -> Vec<RuneDiagnostic> {
        let source = SourceIndex::new(&self.raw_content);
        self.validate_schema_with_source(schema, &source)
    }

    /// Validate `schema` against this config, locating every diagnostic with
    /// the key spans of `source`.
    ///
    /// Contract: `source.text()` is the exact content this config was built
    /// from. The language server indexes a buffer once and parses the same text
    /// into a `RuneConfig`, so both describe one document; mixing an index with
    /// other text would report spans of the wrong buffer, which debug builds
    /// assert against.
    pub(crate) fn validate_schema_with_source(
        &self,
        schema: &SchemaDocument,
        source: &SourceIndex,
    ) -> Vec<RuneDiagnostic> {
        debug_assert_eq!(
            source.text(),
            self.raw_content.as_str(),
            "schema validation must use the source index of the config's own text"
        );

        let snapshot = ValidationSnapshot::build(self, source);
        let mut diagnostics = Vec::new();

        for block in &schema.blocks {
            let root = [block.root.clone()];

            let Some(value) = snapshot.value(&root) else {
                diagnostics.push(
                    RuneDiagnostic::error(format!(
                        "Required schema root '{}' is missing",
                        block.root
                    ))
                    .with_code(650)
                    .with_hint(format!("Add an '{}' object to the config", block.root)),
                );
                continue;
            };

            if !matches!(value, Value::Object(_)) {
                diagnostics.push(type_diagnostic(
                    &root,
                    "object",
                    &value_type_name(value),
                    snapshot.location(&root),
                ));
                continue;
            }

            validate_fields(&snapshot, &root, &block.fields, &mut diagnostics);
        }

        diagnostics
    }
}

fn validate_fields(
    snapshot: &ValidationSnapshot,
    parent_path: &[String],
    fields: &[SchemaField],
    diagnostics: &mut Vec<RuneDiagnostic>,
) {
    for field in fields {
        let path = child_path(parent_path, &field.name);

        match snapshot.value(&path) {
            Some(value) => validate_value(snapshot, &path, field, value, diagnostics),
            None if field.required || has_required_descendant(field) => {
                diagnostics.push(missing_diagnostic(snapshot, &path));
            }
            None => {}
        }
    }
}

fn validate_value(
    snapshot: &ValidationSnapshot,
    path: &[String],
    field: &SchemaField,
    value: &Value,
    diagnostics: &mut Vec<RuneDiagnostic>,
) {
    let value = value.primary();
    let location = snapshot.location(path);

    if !type_matches(&field.kind, value) {
        diagnostics.push(type_diagnostic(
            path,
            &field.kind.name(),
            &value_type_name(value),
            location,
        ));
        return;
    }

    if let (Some((min, max)), Value::Number(number)) = (field.range, value)
        && (*number < min || *number > max)
    {
        diagnostics.push(
            located_diagnostic(
                format!("'{}' must be between {} and {}", join_path(path), min, max),
                location.clone(),
            )
            .with_code(653)
            .with_hint(format!("Use a value in the range {}..{}", min, max)),
        );
    }

    match (&field.kind, value) {
        (SchemaType::Enum(allowed), Value::String(actual)) => {
            if !allowed.iter().any(|value| value == actual) {
                diagnostics.push(
                    located_diagnostic(
                        format!(
                            "'{}' must be one of: {}",
                            join_path(path),
                            allowed.join(", ")
                        ),
                        location,
                    )
                    .with_code(654)
                    .with_hint(format!(
                        "Replace '{}' with one of the allowed values",
                        actual
                    )),
                );
            }
        }
        (SchemaType::Array(inner), Value::Array(items)) => {
            for (index, item) in items.iter().enumerate() {
                if !type_matches(inner, item) {
                    diagnostics.push(
                        located_diagnostic(
                            format!(
                                "'{}[{}]' expected {}, got {}",
                                join_path(path),
                                index,
                                inner.name(),
                                value_type_name(item)
                            ),
                            location.clone(),
                        )
                        .with_code(652),
                    );
                }
            }
        }
        (SchemaType::Object, Value::Object(_)) => {
            validate_fields(snapshot, path, &field.fields, diagnostics);
        }
        _ => {}
    }
}

fn type_matches(kind: &SchemaType, value: &Value) -> bool {
    let value = value.primary();
    match (kind, value) {
        (SchemaType::Any, _) => true,
        (SchemaType::String, Value::String(_)) => true,
        (SchemaType::Int, Value::Number(number)) => number.fract() == 0.0,
        (SchemaType::Float | SchemaType::Number, Value::Number(_)) => true,
        (SchemaType::Bool, Value::Bool(_)) => true,
        (SchemaType::Regex, Value::Regex(_)) => true,
        (SchemaType::Null, Value::Null) => true,
        (SchemaType::Array(_), Value::Array(_)) => true,
        (SchemaType::Enum(_), Value::String(_)) => true,
        (SchemaType::Object, Value::Object(_)) => true,
        _ => false,
    }
}

fn has_required_descendant(field: &SchemaField) -> bool {
    field
        .fields
        .iter()
        .any(|child| child.required || has_required_descendant(child))
}

fn missing_diagnostic(snapshot: &ValidationSnapshot, path: &[String]) -> RuneDiagnostic {
    let parent_path = &path[..path.len().saturating_sub(1)];
    let field = path.last().map(String::as_str).unwrap_or_default();
    let parent = join_path(parent_path);
    let joined = join_path(path);

    let mut diagnostic = RuneDiagnostic::error(if parent.is_empty() {
        format!("Missing required config path '{}'", joined)
    } else {
        format!("Missing required field '{}' inside '{}'", field, parent)
    })
    .with_code(651)
    .with_hint(if parent.is_empty() {
        format!("Add '{}' to satisfy the schema", joined)
    } else {
        format!("Add '{}' inside '{}' to satisfy the schema", field, parent)
    });

    // The deepest existing, effective, source-backed parent is what gets
    // underlined: a missing `app.server.port` points at the `app.server`
    // occurrence value resolution selected, not at a same-named key elsewhere.
    if let Some(location) = snapshot.location(parent_path) {
        let near = if location.snippet.is_empty() {
            parent.as_str()
        } else {
            location.snippet.as_str()
        };

        diagnostic.range = Some(location.range);
        diagnostic = diagnostic.with_hint(format!("Add '{}' near: {}", field, near));
    }

    diagnostic
}

fn type_diagnostic(
    path: &[String],
    expected: &str,
    actual: &str,
    location: Option<Location>,
) -> RuneDiagnostic {
    located_diagnostic(
        format!(
            "'{}' expected {}, got {}",
            join_path(path),
            expected,
            actual
        ),
        location,
    )
    .with_code(652)
}

/// The diagnostic shape validation has always produced: the message, a range
/// covering the field's key token, and a `Check around:` hint on that key's
/// line. A field with no source-backed anchor carries neither range nor hint.
fn located_diagnostic(message: String, location: Option<Location>) -> RuneDiagnostic {
    match location {
        Some(location) => RuneDiagnostic {
            range: Some(location.range),
            hint: Some(format!("Check around: {}", location.snippet)),
            ..RuneDiagnostic::error(message)
        },
        None => RuneDiagnostic::error(message),
    }
}

/// Segment paths are only joined where a message has to print one; nothing
/// about locating a diagnostic goes through a dotted string.
fn join_path(path: &[String]) -> String {
    path.join(".")
}

fn child_path(parent: &[String], name: &str) -> Vec<String> {
    let mut path = parent.to_vec();
    path.push(name.to_string());
    path
}

fn value_type_name(value: &Value) -> String {
    match value {
        Value::Annotated(value) => value_type_name(&value.value),
        Value::String(_) => "string".into(),
        Value::Number(number) if number.fract() == 0.0 => "int".into(),
        Value::Number(_) => "number".into(),
        Value::Bool(_) => "bool".into(),
        Value::Regex(_) => "regex".into(),
        Value::Array(_) => "array".into(),
        Value::Object(_) => "object".into(),
        Value::Reference(_) => "reference".into(),
        Value::Interpolated(_) => "interpolated".into(),
        Value::Conditional(_) => "conditional".into(),
        Value::Null => "null".into(),
    }
}

/// A location-aware view of the fields a config actually provides.
///
/// The tree mirrors what `RuneConfig::resolved_root` resolves: globals before
/// document items, the selected branch of every conditional flattened in
/// place, and the first assignment of a name winning at each object level. It
/// adds the one thing that resolution cannot carry, the exact [`SourceIndex`]
/// occurrence each key was written at.
struct ValidationSnapshot<'a> {
    source: &'a SourceIndex,
    root: Vec<LocatedField>,
}

/// One effective field, paired with the source occurrence that declares it.
struct LocatedField {
    /// Full segment path, compared as a whole: a leaf name alone never selects
    /// an occurrence from another object.
    path: Vec<String>,
    /// Key token of the paired occurrence; `None` when the AST node has no
    /// compatible indexed entry.
    span: Option<Span>,
    /// Trimmed source line holding the key, for hint text only. It never takes
    /// part in locating the diagnostic.
    snippet: Option<String>,
    /// The field's own AST value with references and conditionals resolved,
    /// which is what `get_value` reports for the same path.
    value: Option<Value>,
    /// Located fields of a native object value, in document order.
    children: Vec<LocatedField>,
}

/// A diagnostic anchor: the exact range to underline plus the trimmed source
/// line the hints mention.
#[derive(Clone)]
struct Location {
    range: SourceRange,
    snippet: String,
}

impl<'a> ValidationSnapshot<'a> {
    fn build(config: &RuneConfig, source: &'a SourceIndex) -> Self {
        // Values are only reported when the config's own root resolution
        // succeeds. When it fails, `get_value` reports every path as missing,
        // and validation has to keep reporting exactly that rather than the
        // fields that happen to resolve on their own.
        let resolution =
            Resolution::new(config).filter(|resolution| resolution.resolves_root(config));
        let root = SnapshotBuilder::new(source, resolution).build(config);

        Self { source, root }
    }

    /// Resolved value at `path`, or `None` when the config has no such path.
    ///
    /// The located field supplies the value when the index holds the path. The
    /// segments below it are then resolved inside that value, which is how
    /// reference- and import-produced fields and the attributes of an
    /// annotated value are reached: they have no indexed occurrence of their
    /// own, but they do live inside a located value.
    fn value(&self, path: &[String]) -> Option<&Value> {
        let mut best: Option<(&LocatedField, usize)> = None;
        deepest_value_prefix(&self.root, path, &mut best);
        let (field, depth) = best?;

        let mut current = field.value.as_ref()?;
        for segment in &path[depth..] {
            current = child_value(current, segment)?;
        }

        Some(current)
    }

    /// Where `path` is underlined: its own indexed occurrence, or the deepest
    /// source-backed ancestor when the field itself has no indexed entry (it is
    /// missing, produced by a reference, or an inline attribute).
    fn location(&self, path: &[String]) -> Option<Location> {
        let field = deepest_anchor(&self.root, path)?;
        let span = field.span?;

        Some(Location {
            range: self.source.lines().rune_range(span),
            snippet: field.snippet.clone().unwrap_or_default(),
        })
    }
}

/// The located field with the longest path that is a prefix of `path` and
/// carries a resolved value.
fn deepest_value_prefix<'s>(
    fields: &'s [LocatedField],
    path: &[String],
    best: &mut Option<(&'s LocatedField, usize)>,
) {
    for field in fields {
        if !path.starts_with(field.path.as_slice()) {
            continue;
        }

        let longest = match best {
            Some((_, depth)) => *depth,
            None => 0,
        };
        if field.value.is_some() && field.path.len() > longest {
            *best = Some((field, field.path.len()));
        }

        deepest_value_prefix(&field.children, path, best);
    }
}

/// The deepest located field whose path is a prefix of `path` and whose key has
/// an indexed span.
fn deepest_anchor<'s>(fields: &'s [LocatedField], path: &[String]) -> Option<&'s LocatedField> {
    let mut best = None;
    collect_anchors(fields, path, &mut best);
    best
}

fn collect_anchors<'s>(
    fields: &'s [LocatedField],
    path: &[String],
    best: &mut Option<&'s LocatedField>,
) {
    for field in fields {
        if !path.starts_with(field.path.as_slice()) {
            continue;
        }

        if field.span.is_some() {
            *best = Some(field);
        }

        collect_anchors(&field.children, path, best);
    }
}

/// One step of the traversal `get_value` performs: object items first-match by
/// key, then the attributes of an annotated value.
fn child_value<'s>(value: &'s Value, segment: &str) -> Option<&'s Value> {
    match value {
        Value::Object(items) => items.iter().find_map(|item| match item {
            ObjectItem::Assign(key, value) if key == segment => Some(value),
            _ => None,
        }),
        Value::Annotated(annotated) => annotated
            .attributes
            .iter()
            .find_map(|(key, value)| (key == segment).then_some(value)),
        _ => None,
    }
}

/// Builds the located tree by pairing the AST with indexed occurrences while
/// walking the document in source order.
struct SnapshotBuilder<'a> {
    source: &'a SourceIndex,
    /// Key spans of every indexed field, in document order, per exact path and
    /// entry kind.
    occurrences: HashMap<(Vec<String>, FieldKind), Vec<Span>>,
    /// How many occurrences of a key the walk has paired so far.
    paired: HashMap<(Vec<String>, FieldKind), usize>,
    resolution: Option<Resolution>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum FieldKind {
    Assignment,
    Object,
}

impl FieldKind {
    fn of(value: &Value) -> Self {
        match value {
            Value::Object(_) => Self::Object,
            _ => Self::Assignment,
        }
    }
}

impl<'a> SnapshotBuilder<'a> {
    fn new(source: &'a SourceIndex, resolution: Option<Resolution>) -> Self {
        let mut occurrences: HashMap<(Vec<String>, FieldKind), Vec<Span>> = HashMap::new();

        for entry in source.entries() {
            let kind = match entry.kind {
                SourceEntryKind::Assignment => FieldKind::Assignment,
                SourceEntryKind::Object => FieldKind::Object,
                // Conditional, metadata and gather entries describe layout
                // rather than fields, and no schema diagnostic points at them.
                _ => continue,
            };

            occurrences
                .entry((entry.path.clone(), kind))
                .or_default()
                .push(entry.key_span);
        }

        Self {
            source,
            occurrences,
            paired: HashMap::new(),
            resolution,
        }
    }

    fn build(mut self, config: &RuneConfig) -> Vec<LocatedField> {
        let mut root = Vec::new();
        let mut seen = Vec::new();

        if let Some(document) = config.document() {
            // `resolved_root` builds its root as globals followed by document
            // items, so the two views agree on which occurrence wins.
            for (name, value) in document.globals.iter().chain(document.items.iter()) {
                self.push_field(name, value, &[], true, &mut seen, &mut root);
            }
        }

        root
    }

    /// Walk one object body, flattening the selected branch of every
    /// conditional in place, exactly as `resolve_value_recursively` does.
    fn walk_object(
        &mut self,
        items: &[ObjectItem],
        scope: &[String],
        emit: bool,
        out: &mut Vec<LocatedField>,
    ) {
        let mut seen = Vec::new();
        self.walk_items(items, scope, emit, &mut seen, out);
    }

    fn walk_items(
        &mut self,
        items: &[ObjectItem],
        scope: &[String],
        emit: bool,
        seen: &mut Vec<String>,
        out: &mut Vec<LocatedField>,
    ) {
        for item in items {
            match item {
                ObjectItem::Assign(name, value) => {
                    self.push_field(name, value, scope, emit, seen, out);
                }

                // `elseif` is a nested `IfBlock` inside the preceding branch's
                // `else_items`, so walking the two branches in source order
                // pairs occurrences in document order, and only the selected
                // branch is emitted while the other still advances the pairing.
                ObjectItem::IfBlock(block) => {
                    let take_then = self.condition_is_met(&block.condition);

                    if take_then {
                        self.walk_items(&block.then_items, scope, emit, seen, out);
                        if let Some(else_items) = &block.else_items {
                            self.walk_items(else_items, scope, false, seen, out);
                        }
                    } else {
                        self.walk_items(&block.then_items, scope, false, seen, out);
                        if let Some(else_items) = &block.else_items {
                            self.walk_items(else_items, scope, emit, seen, out);
                        }
                    }
                }
            }
        }
    }

    fn push_field(
        &mut self,
        name: &str,
        value: &Value,
        scope: &[String],
        emit: bool,
        seen: &mut Vec<String>,
        out: &mut Vec<LocatedField>,
    ) {
        let path = child_path(scope, name);
        let span = self.pair(&path, FieldKind::of(value));

        // The first assignment of a name wins at each object level, in the
        // order `resolve_value_recursively` flattens items; a later assignment
        // is shadowed by it and never gets a diagnostic of its own.
        let shadowed = seen.iter().any(|existing| existing == name);
        if !emit || shadowed {
            // Inactive branches and shadowed duplicates still advance the
            // pairing so later live fields keep their own spans.
            if let Value::Object(items) = value {
                self.walk_object(items, &path, false, out);
            }
            return;
        }

        seen.push(name.to_string());

        let mut children = Vec::new();
        if let Value::Object(items) = value {
            self.walk_object(items, &path, true, &mut children);
        }

        out.push(LocatedField {
            path,
            span,
            snippet: span.map(|span| self.snippet_of(span).to_string()),
            value: self.resolved_value(value),
            children,
        });
    }

    /// The next indexed occurrence of exactly `path`, consuming it.
    fn pair(&mut self, path: &[String], kind: FieldKind) -> Option<Span> {
        let key = (path.to_vec(), kind);
        let index = {
            let paired = self.paired.entry(key.clone()).or_insert(0);
            let index = *paired;
            *paired += 1;
            index
        };

        self.occurrences.get(&key)?.get(index).copied()
    }

    /// Trimmed text of the line a key token sits on.
    fn snippet_of(&self, span: Span) -> &str {
        let lines = self.source.lines();
        lines
            .line_text(lines.line_of(span.start))
            .unwrap_or("")
            .trim()
    }

    fn condition_is_met(&self, condition: &crate::ast::Condition) -> bool {
        let Some(resolution) = &self.resolution else {
            // Without a resolvable root no value is reported at all, so the
            // branch choice can only affect occurrences no diagnostic reaches.
            return false;
        };

        helpers::condition_is_met(condition, &resolution.parser, &resolution.document)
    }

    fn resolved_value(&self, value: &Value) -> Option<Value> {
        let resolution = self.resolution.as_ref()?;

        helpers::resolve_value_recursively(value, &resolution.parser, &resolution.document).ok()
    }
}

/// The parser and main document `RuneConfig::resolved_root` resolves against.
struct Resolution {
    parser: parser::Parser<'static>,
    document: Document,
}

impl Resolution {
    /// Every imported document is reachable under its alias and references
    /// resolve against the main document, exactly as in `resolved_root`.
    fn new(config: &RuneConfig) -> Option<Self> {
        let document = config.document()?.clone();
        let mut parser = parser::Parser::new("").ok()?;

        for (alias, imported) in config.all_documents() {
            if alias != &config.main_doc_key {
                parser.inject_import(alias.clone(), imported.clone());
            }
        }

        Some(Self { parser, document })
    }

    /// True when resolving the whole root succeeds. A root that does not
    /// resolve makes `get_value` report every path as missing, which is what
    /// validation reports as well.
    fn resolves_root(&self, config: &RuneConfig) -> bool {
        let Some(document) = config.document() else {
            return false;
        };

        let mut items: Vec<ObjectItem> = Vec::new();
        for (key, value) in document.globals.iter().chain(document.items.iter()) {
            items.push(ObjectItem::Assign(key.clone(), value.clone()));
        }

        helpers::resolve_value_recursively(&Value::Object(items), &self.parser, &self.document)
            .is_ok()
    }
}
