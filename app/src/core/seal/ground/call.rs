use std::collections::BTreeSet;

use crate::core::seal::{
    ast::{RawArg, RawExpr, RawExprKind},
    diag::Diagnostic,
    span::Span,
};

pub(super) fn validate_args(
    call_span: Span,
    callee: &RawExpr,
    args: &[RawArg],
    diagnostics: &mut Vec<Diagnostic>,
) {
    reject_duplicate_labels(args, diagnostics);

    let labeled = args.iter().filter(|arg| arg.label.is_some());
    if callee_is_call_forward(callee) {
        for arg in labeled {
            diagnostics.push(Diagnostic::new(
                arg.span,
                "@call.forward arguments are positional-only",
            ));
        }
        validate_call_forward_shape(call_span, args, diagnostics);
        return;
    }

    if callee_accepts_labels(callee) {
        return;
    }

    for arg in labeled {
        diagnostics.push(Diagnostic::new(
            arg.span,
            "labeled call arguments require a static method or @ helper callee",
        ));
    }
}

fn validate_call_forward_shape(
    call_span: Span,
    args: &[RawArg],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if args.len() != 2 {
        diagnostics.push(Diagnostic::new(
            call_span,
            "@call.forward expects exactly 2 arguments",
        ));
    }

    let Some(bundle) = args.get(1) else {
        return;
    };
    if static_non_array(&bundle.value) {
        diagnostics.push(Diagnostic::new(
            bundle.value.span,
            "@call.forward second argument must be an array bundle",
        ));
    }
}

fn reject_duplicate_labels(args: &[RawArg], diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = BTreeSet::new();
    for arg in args {
        let Some(label) = &arg.label else {
            continue;
        };
        if !seen.insert(label) {
            diagnostics.push(Diagnostic::new(
                arg.span,
                format!("duplicate labeled argument '{label}'"),
            ));
        }
    }
}

fn callee_accepts_labels(callee: &RawExpr) -> bool {
    matches!(&callee.kind, RawExprKind::Ident(_) | RawExprKind::AtName(_))
}

fn callee_is_call_forward(callee: &RawExpr) -> bool {
    matches!(
        &callee.kind,
        RawExprKind::AtName(parts) if parts.len() == 2 && parts[0] == "call" && parts[1] == "forward"
    )
}

fn static_non_array(expr: &RawExpr) -> bool {
    match &expr.kind {
        RawExprKind::Array(_) => false,
        RawExprKind::Group(expr) => static_non_array(expr),
        RawExprKind::Literal(_)
        | RawExprKind::Map(_)
        | RawExprKind::Lambda(_)
        | RawExprKind::Env(_)
        | RawExprKind::Channel(_)
        | RawExprKind::Process(_) => true,
        _ => false,
    }
}
