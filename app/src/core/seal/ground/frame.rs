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
    validate_event_expr(left, diagnostics);
}

pub(super) fn validate_event_expr(expr: &RawExpr, diagnostics: &mut Vec<Diagnostic>) {
    let RawExprKind::Map(entries) = &expr.kind else {
        return;
    };
    let Some(type_value) = field(entries, "type") else {
        diagnostics.push(Diagnostic::new(
            expr.span,
            "frame event map must include field 'type'",
        ));
        return;
    };
    let Some(event_type) = string_literal(type_value) else {
        diagnostics.push(Diagnostic::new(
            type_value.span,
            "frame event field 'type' must be a string literal",
        ));
        return;
    };

    match event_type {
        "ok" => {}
        "failed" => require_field(entries, "exit", expr.span, diagnostics),
        "fault" => require_field(entries, "fault", expr.span, diagnostics),
        "cancelled" => {
            require_field(entries, "source", expr.span, diagnostics);
            require_field(entries, "signal", expr.span, diagnostics);
        }
        "cleanup" => validate_cleanup(entries, expr.span, diagnostics),
        _ => diagnostics.push(Diagnostic::new(
            expr.span,
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

fn string_literal(expr: &RawExpr) -> Option<&str> {
    let RawExprKind::Literal(RawLiteral::String(value)) = &expr.kind else {
        return None;
    };
    Some(value)
}

fn field<'a>(entries: &'a [RawMapEntry], key: &str) -> Option<&'a RawExpr> {
    entries
        .iter()
        .find(|entry| entry.key == key)
        .map(|entry| &entry.value)
}
