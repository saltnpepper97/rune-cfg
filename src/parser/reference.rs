// Author: Dustin Pilgrim
// License: MIT

use super::*;

pub(super) fn resolve_reference<'b>(
    parser: &'b Parser,
    path: &[String],
    doc: &'b Document,
) -> Option<&'b Value> {
    if path.is_empty() {
        return None;
    }

    // Check if first segment is an import alias
    let (current_doc, remaining_path): (&Document, &[String]) = {
        if let Some(import_doc) = parser.imports.get(&path[0]) {
            // First segment is an import alias, use imported doc and skip first segment
            (import_doc, &path[1..])
        } else {
            // Not an import alias, use current doc and full path
            (doc, path)
        }
    };

    if remaining_path.is_empty() {
        return None;
    }

    // Find the first segment in the current document
    let mut current: &Value = {
        let first_segment = &remaining_path[0];

        // First check items (top-level blocks/assignments)
        if let Some((_, v)) = current_doc.items.iter().find(|(k, _)| k == first_segment) {
            v
        }
        // Then check globals
        else if let Some((_, v)) = current_doc.globals.iter().find(|(k, _)| k == first_segment) {
            v
        }
        // Not found
        else {
            return None;
        }
    };

    // Traverse the remaining path segments
    for seg in &remaining_path[1..] {
        match current {
            Value::Object(items) => {
                // NOTE:
                // Objects now contain `ObjectItem` (assignments + if-blocks).
                // Reference resolution does NOT attempt to evaluate if-blocks here.
                // The config layer resolves/flatten objects before typical access.
                //
                // For references, we only traverse explicit assignments present in the AST.
                if let Some(v) = items.iter().find_map(|item| match item {
                    crate::ast::ObjectItem::Assign(k, v) if k == seg => Some(v),
                    _ => None,
                }) {
                    current = v;
                } else {
                    return None;
                }
            }
            _ => {
                return None;
            }
        }
    }

    Some(current)
}
