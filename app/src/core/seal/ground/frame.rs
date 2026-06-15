use crate::core::seal::{
    ast::{RawExpr, RawExprKind, RawLiteral, RawMapEntry},
    diag::Diagnostic,
    span::Span,
};

pub(super) fn validate_frame_event(expr: &RawExpr, diagnostics: &mut Vec<Diagnostic>) {
    let RawExprKind::StreamFlow { left, right, .. } = &expr.kind else {
        return;
    };
    if !matches!(&right.kind, RawExprKind::Channel(name) if name == "frame") {
        return;
    }
    let RawExprKind::Map(entries) = &left.kind else {
        return;
    };

    let Some(event_type) = string_field(entries, "type") else {
        diagnostics.push(Diagnostic::new(
            left.span,
            "frame event map must include string literal field 'type'",
        ));
        return;
    };

    match event_type {
        "ok" => {}
        "failed" => require_field(entries, "exit", left.span, diagnostics),
        "fault" => require_field(entries, "fault", left.span, diagnostics),
        "cancelled" => {
            require_field(entries, "source", left.span, diagnostics);
            require_field(entries, "signal", left.span, diagnostics);
        }
        "cleanup" => validate_cleanup(entries, left.span, diagnostics),
        _ => diagnostics.push(Diagnostic::new(
            left.span,
            format!("unknown frame event type '{event_type}'"),
        )),
    }
}

fn validate_cleanup(entries: &[RawMapEntry], span: Span, diagnostics: &mut Vec<Diagnostic>) {
    let Some(run) = field(entries, "run") else {
        diagnostics.push(Diagnostic::new(
            span,
            "cleanup frame event requires field 'run'",
        ));
        return;
    };
    if !matches!(run.kind, RawExprKind::Lambda(_)) {
        diagnostics.push(Diagnostic::new(
            run.span,
            "cleanup frame event field 'run' must be a lambda",
        ));
    }
}

fn require_field(
    entries: &[RawMapEntry],
    key: &str,
    span: Span,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if field(entries, key).is_none() {
        diagnostics.push(Diagnostic::new(
            span,
            format!("frame event type requires field '{key}'"),
        ));
    }
}

fn string_field<'a>(entries: &'a [RawMapEntry], key: &str) -> Option<&'a str> {
    let field = field(entries, key)?;
    let RawExprKind::Literal(RawLiteral::String(value)) = &field.kind else {
        return None;
    };
    Some(string_content(value))
}

fn string_content(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

fn field<'a>(entries: &'a [RawMapEntry], key: &str) -> Option<&'a RawExpr> {
    entries
        .iter()
        .find(|entry| entry.key == key)
        .map(|entry| &entry.value)
}
