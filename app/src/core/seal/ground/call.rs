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

    if let Some(call_name) = callee_call_name(callee) {
        match call_name {
            "forward" => {
                for arg in args.iter().filter(|arg| arg.label.is_some()) {
                    diagnostics.push(Diagnostic::new(
                        arg.span,
                        "@call.forward arguments are positional-only",
                    ));
                }
                validate_call_forward(call_span, args, diagnostics);
            }
            "stdio" => validate_io_call("@call.stdio", 3, call_span, args, diagnostics),
            "completion" => validate_io_call("@call.completion", 4, call_span, args, diagnostics),
            _ => {}
        }
    }

    if callee_accepts_labels(callee) {
        return;
    }

    for arg in args.iter().filter(|arg| arg.label.is_some()) {
        diagnostics.push(Diagnostic::new(
            arg.span,
            "labeled call arguments require a static method or @ helper callee",
        ));
    }
}

fn validate_call_forward(call_span: Span, args: &[RawArg], diagnostics: &mut Vec<Diagnostic>) {
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

fn validate_io_call(
    name: &str,
    params: usize,
    call_span: Span,
    args: &[RawArg],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if args.len() != 2 {
        diagnostics.push(Diagnostic::new(
            call_span,
            format!("{name} expects exactly 2 arguments"),
        ));
    }

    let Some(handler) = args.get(1) else {
        return;
    };
    let RawExprKind::Lambda(lambda) = &handler.value.kind else {
        if static_non_lambda(&handler.value) {
            diagnostics.push(Diagnostic::new(
                handler.value.span,
                format!("{name} second argument must be a handler lambda"),
            ));
        }
        return;
    };
    if lambda.params.len() != params {
        diagnostics.push(Diagnostic::new(
            handler.value.span,
            format!("{name} handler must accept exactly {params} parameters"),
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

fn callee_call_name(callee: &RawExpr) -> Option<&str> {
    match &callee.kind {
        RawExprKind::AtName(parts) if parts.len() == 2 && parts[0] == "call" => {
            Some(parts[1].as_str())
        }
        _ => None,
    }
}

fn static_non_lambda(expr: &RawExpr) -> bool {
    match &expr.kind {
        RawExprKind::Lambda(_) => false,
        RawExprKind::Group(expr) => static_non_lambda(expr),
        RawExprKind::Ident(_)
        | RawExprKind::AtName(_)
        | RawExprKind::Call { .. }
        | RawExprKind::ReceiverCall { .. } => false,
        _ => true,
    }
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
