use super::*;
use crate::source::SourceIndex;
use crate::{DiagnosticSeverity, RuneDiagnostic};
use std::collections::HashMap;

impl RuneConfig {
    /// Source ranges of each definition of a dotted key in the root text.
    pub fn key_ranges(&self, path: &str) -> Vec<crate::SourceRange> {
        let source = SourceIndex::new(&self.raw_content);
        let path = path.split('.').map(str::to_owned).collect::<Vec<_>>();
        source
            .entries_with_path(&path)
            .iter()
            .map(|entry| source.lines().rune_range(entry.key_span))
            .collect()
    }

    /// Warn about repeated keys within the same object or conditional branch.
    /// Includes the root document only; gathered files can be checked individually.
    /// Existing first-key-wins lookup behavior is unchanged.
    pub fn duplicate_key_diagnostics(&self) -> Vec<RuneDiagnostic> {
        self.duplicate_key_diagnostics_with_source(&SourceIndex::new(&self.raw_content))
    }

    pub(crate) fn duplicate_key_diagnostics_with_source(
        &self,
        source: &SourceIndex,
    ) -> Vec<RuneDiagnostic> {
        let Ok(mut parser) = parser::Parser::with_options(&self.raw_content, self.options) else {
            return Vec::new();
        };
        let Ok(document) = parser.parse_document() else {
            return Vec::new();
        };
        let mut top = document.globals;
        top.extend(document.items);
        let mut diagnostics = Vec::new();
        walk(&top, &[], source, &mut HashMap::new(), &mut diagnostics);
        diagnostics
    }
}

fn walk(
    items: &[(String, Value)],
    scope: &[String],
    source: &SourceIndex,
    occurrences: &mut HashMap<Vec<String>, usize>,
    diagnostics: &mut Vec<RuneDiagnostic>,
) {
    let mut seen = std::collections::HashSet::new();
    for (key, value) in items {
        walk_value(
            key,
            value,
            &mut seen,
            scope,
            source,
            occurrences,
            diagnostics,
        );
    }
}
fn walk_value(
    key: &str,
    value: &Value,
    seen: &mut std::collections::HashSet<String>,
    scope: &[String],
    source: &SourceIndex,
    occurrences: &mut HashMap<Vec<String>, usize>,
    diagnostics: &mut Vec<RuneDiagnostic>,
) {
    let mut path = scope.to_vec();
    path.push(key.to_owned());
    let index = occurrences.entry(path.clone()).or_default();
    let entry = source
        .entries_with_path(&path)
        .into_iter()
        .filter(|e| e.kind.is_field())
        .nth(*index);
    *index += 1;
    if !seen.insert(key.to_owned()) {
        let mut diagnostic = RuneDiagnostic::error(format!("Duplicate key '{}'", path.join(".")))
            .with_code(660)
            .with_hint("Remove or rename the repeated key; the first value is used");
        diagnostic.severity = DiagnosticSeverity::Warning;
        if let Some(entry) = entry {
            diagnostic.range = Some(source.lines().rune_range(entry.key_span));
        }
        diagnostics.push(diagnostic);
    }
    if let Value::Object(items) = value {
        walk_object(items, &path, source, occurrences, diagnostics);
    }
}
fn walk_object(
    items: &[ObjectItem],
    scope: &[String],
    source: &SourceIndex,
    occurrences: &mut HashMap<Vec<String>, usize>,
    diagnostics: &mut Vec<RuneDiagnostic>,
) {
    let mut seen = std::collections::HashSet::new();
    for item in items {
        match item {
            ObjectItem::Assign(key, value) => walk_value(
                key,
                value,
                &mut seen,
                scope,
                source,
                occurrences,
                diagnostics,
            ),
            ObjectItem::IfBlock(block) => {
                // Branches have separate namespaces: mutually exclusive definitions are valid.
                walk_object(&block.then_items, scope, source, occurrences, diagnostics);
                if let Some(items) = &block.else_items {
                    walk_object(items, scope, source, occurrences, diagnostics);
                }
            }
        }
    }
}
