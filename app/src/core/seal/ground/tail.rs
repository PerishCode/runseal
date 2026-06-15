use crate::core::seal::{
    ast::{RawBlock, RawExpr, RawExprKind, RawItemKind, RawStatement, RawStatementKind},
    diag::Diagnostic,
    span::Span,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TailOutput {
    Implicit { span: Span },
    DisabledByStdout { span: Span },
    None,
}

pub(super) fn method_tail_output(body: &RawBlock, diagnostics: &mut Vec<Diagnostic>) -> TailOutput {
    if let Some(span) = find_current_stdout_block(body) {
        return TailOutput::DisabledByStdout { span };
    }

    for item in body.items.iter().rev() {
        match &item.kind {
            RawItemKind::Comment(_) => continue,
            RawItemKind::Statement(statement) => {
                super::reject_statement_comparison_chains(statement, diagnostics);
                return match &statement.kind {
                    RawStatementKind::Expr(expr) => TailOutput::Implicit { span: expr.span },
                    _ => TailOutput::None,
                };
            }
            RawItemKind::Method(_) | RawItemKind::Error => return TailOutput::None,
        }
    }
    TailOutput::None
}

fn find_current_stdout_block(block: &RawBlock) -> Option<Span> {
    block.items.iter().find_map(|item| match &item.kind {
        RawItemKind::Statement(statement) => find_current_stdout_statement(statement),
        RawItemKind::Comment(_) | RawItemKind::Method(_) | RawItemKind::Error => None,
    })
}

fn find_current_stdout_statement(statement: &RawStatement) -> Option<Span> {
    match &statement.kind {
        RawStatementKind::Let { value, .. } => find_current_stdout_expr(value),
        RawStatementKind::Assign { target, value } => {
            find_current_stdout_expr(target).or_else(|| find_current_stdout_expr(value))
        }
        RawStatementKind::If {
            branches,
            else_branch,
        } => {
            for branch in branches {
                if let Some(span) = find_current_stdout_expr(&branch.condition)
                    .or_else(|| find_current_stdout_block(&branch.body))
                {
                    return Some(span);
                }
            }
            else_branch.as_ref().and_then(find_current_stdout_block)
        }
        RawStatementKind::For { iterable, body, .. } => {
            find_current_stdout_expr(iterable).or_else(|| find_current_stdout_block(body))
        }
        RawStatementKind::While { condition, body } => {
            find_current_stdout_expr(condition).or_else(|| find_current_stdout_block(body))
        }
        RawStatementKind::WithEnv { bindings, body } => bindings
            .iter()
            .find_map(|binding| find_current_stdout_expr(&binding.value))
            .or_else(|| find_current_stdout_block(body)),
        RawStatementKind::Expr(expr) | RawStatementKind::Effect(expr) => {
            find_current_stdout_expr(expr)
        }
        RawStatementKind::Break | RawStatementKind::Continue | RawStatementKind::Error => None,
    }
}

fn find_current_stdout_expr(expr: &RawExpr) -> Option<Span> {
    match &expr.kind {
        RawExprKind::Channel(name) if name == "stdout" => Some(expr.span),
        RawExprKind::Binary { left, right, .. } | RawExprKind::StreamFlow { left, right, .. } => {
            find_current_stdout_expr(left).or_else(|| find_current_stdout_expr(right))
        }
        RawExprKind::Unary { expr, .. } | RawExprKind::Group(expr) => {
            find_current_stdout_expr(expr)
        }
        RawExprKind::Call { callee, args } => find_current_stdout_expr(callee).or_else(|| {
            args.iter()
                .find_map(|arg| find_current_stdout_expr(&arg.value))
        }),
        RawExprKind::BlockCall { callee, block } => {
            find_current_stdout_expr(callee).or_else(|| find_current_stdout_block(block))
        }
        RawExprKind::Lambda(_) => None,
        RawExprKind::ReceiverCall { receiver, args, .. } => find_current_stdout_expr(receiver)
            .or_else(|| {
                args.iter()
                    .find_map(|arg| find_current_stdout_expr(&arg.value))
            }),
        RawExprKind::Array(items) => items.iter().find_map(find_current_stdout_expr),
        RawExprKind::Map(entries) => entries
            .iter()
            .find_map(|entry| find_current_stdout_expr(&entry.value)),
        RawExprKind::Match(match_expr) => {
            find_current_stdout_expr(&match_expr.scrutinee).or_else(|| {
                match_expr.arms.iter().find_map(|arm| {
                    arm.patterns
                        .iter()
                        .find_map(find_stdout_pattern)
                        .or_else(|| find_stdout_arm_body(&arm.body))
                })
            })
        }
        RawExprKind::Process(process) => process
            .program
            .iter()
            .chain(process.args.iter())
            .find_map(find_stdout_arg),
        _ => None,
    }
}

fn find_stdout_arm_body(body: &crate::core::seal::ast::RawMatchArmBody) -> Option<Span> {
    match body {
        crate::core::seal::ast::RawMatchArmBody::Expr(expr) => find_current_stdout_expr(expr),
        crate::core::seal::ast::RawMatchArmBody::Block(block) => find_current_stdout_block(block),
    }
}

fn find_stdout_pattern(pattern: &crate::core::seal::ast::RawPattern) -> Option<Span> {
    match &pattern.kind {
        crate::core::seal::ast::RawPatternKind::Expr(expr) => find_current_stdout_expr(expr),
        crate::core::seal::ast::RawPatternKind::Map(entries) => entries
            .iter()
            .find_map(|entry| find_stdout_pattern(&entry.pattern)),
        crate::core::seal::ast::RawPatternKind::Array(items) => {
            items.iter().find_map(find_stdout_pattern)
        }
        crate::core::seal::ast::RawPatternKind::Wildcard => None,
    }
}

fn find_stdout_arg(arg: &crate::core::seal::ast::RawProcessArg) -> Option<Span> {
    match &arg.kind {
        crate::core::seal::ast::RawProcessArgKind::Spread(expr) => find_current_stdout_expr(expr),
        crate::core::seal::ast::RawProcessArgKind::Word(parts) => {
            parts.iter().find_map(|part| match part {
                crate::core::seal::ast::RawProcessPart::Interpolation(expr) => {
                    find_current_stdout_expr(expr)
                }
                crate::core::seal::ast::RawProcessPart::Text(_) => None,
            })
        }
        crate::core::seal::ast::RawProcessArgKind::String(_)
        | crate::core::seal::ast::RawProcessArgKind::TextBlock(_)
        | crate::core::seal::ast::RawProcessArgKind::Error => None,
    }
}
