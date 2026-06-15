use std::collections::BTreeSet;

use crate::core::seal::{
    ast::{RawArg, RawExpr, RawExprKind},
    diag::Diagnostic,
};

pub(super) fn validate_args(callee: &RawExpr, args: &[RawArg], diagnostics: &mut Vec<Diagnostic>) {
    reject_duplicate_labels(args, diagnostics);

    let labeled = args.iter().filter(|arg| arg.label.is_some());
    if callee_is_call_forward(callee) {
        for arg in labeled {
            diagnostics.push(Diagnostic::new(
                arg.span,
                "@call.forward arguments are positional-only",
            ));
        }
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
