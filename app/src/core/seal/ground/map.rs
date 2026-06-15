use std::collections::BTreeMap;

use crate::core::seal::{
    ast::{RawMapEntry, RawPatternEntry},
    diag::Diagnostic,
};

pub(super) fn validate_expr_entries(entries: &[RawMapEntry], diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = BTreeMap::new();
    for entry in entries {
        let key = key_name(&entry.key);
        if seen.insert(key, entry.span).is_some() {
            diagnostics.push(Diagnostic::new(
                entry.span,
                format!("duplicate map key '{key}'"),
            ));
        }
    }
}

pub(super) fn validate_pattern_entries(
    entries: &[RawPatternEntry],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = BTreeMap::new();
    for entry in entries {
        if seen.insert(entry.key.as_str(), entry.span).is_some() {
            diagnostics.push(Diagnostic::new(
                entry.span,
                format!("duplicate map pattern key '{}'", entry.key),
            ));
        }
    }
}

fn key_name(key: &str) -> &str {
    key.strip_prefix('"')
        .and_then(|key| key.strip_suffix('"'))
        .unwrap_or(key)
}
