use crate::core::seal::{
    ast::{RawBlock, RawExpr, RawExprKind, RawItemKind, RawStatement, RawStatementKind},
    diag::Diagnostic,
};

pub(super) fn validate_block(block: &RawBlock, in_loop: bool, diagnostics: &mut Vec<Diagnostic>) {
    for item in &block.items {
        match &item.kind {
            RawItemKind::Statement(statement) => {
                validate_statement(statement, in_loop, diagnostics)
            }
            RawItemKind::Method(method) => validate_block(&method.body, false, diagnostics),
            RawItemKind::Comment(_) | RawItemKind::Error => {}
        }
    }
}

pub(super) fn validate_statement(
    statement: &RawStatement,
    in_loop: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match &statement.kind {
        RawStatementKind::Break if !in_loop => {
            diagnostics.push(Diagnostic::new(statement.span, "'break' outside loop"));
        }
        RawStatementKind::Continue if !in_loop => {
            diagnostics.push(Diagnostic::new(statement.span, "'continue' outside loop"));
        }
        RawStatementKind::For { iterable, body, .. } => {
            validate_expr(iterable, in_loop, diagnostics);
            validate_block(body, true, diagnostics);
        }
        RawStatementKind::While { condition, body } => {
            validate_expr(condition, in_loop, diagnostics);
            validate_block(body, true, diagnostics);
        }
        RawStatementKind::If {
            branches,
            else_branch,
        } => {
            for branch in branches {
                validate_expr(&branch.condition, in_loop, diagnostics);
                validate_block(&branch.body, in_loop, diagnostics);
            }
            if let Some(block) = else_branch {
                validate_block(block, in_loop, diagnostics);
            }
        }
        RawStatementKind::WithEnv { bindings, body } => {
            for binding in bindings {
                validate_expr(&binding.value, in_loop, diagnostics);
            }
            validate_block(body, in_loop, diagnostics);
        }
        RawStatementKind::Let { value, .. } => validate_expr(value, in_loop, diagnostics),
        RawStatementKind::Assign { target, value } => {
            validate_expr(target, in_loop, diagnostics);
            validate_expr(value, in_loop, diagnostics);
        }
        RawStatementKind::Expr(expr) | RawStatementKind::Effect(expr) => {
            validate_expr(expr, in_loop, diagnostics);
        }
        RawStatementKind::Break | RawStatementKind::Continue | RawStatementKind::Error => {}
    }
}

fn validate_expr(expr: &RawExpr, in_loop: bool, diagnostics: &mut Vec<Diagnostic>) {
    match &expr.kind {
        RawExprKind::Unary { expr, .. } | RawExprKind::Group(expr) => {
            validate_expr(expr, in_loop, diagnostics);
        }
        RawExprKind::Binary { left, right, .. } | RawExprKind::StreamFlow { left, right, .. } => {
            validate_expr(left, in_loop, diagnostics);
            validate_expr(right, in_loop, diagnostics);
        }
        RawExprKind::Call { callee, args } => {
            validate_expr(callee, in_loop, diagnostics);
            for arg in args {
                validate_expr(&arg.value, in_loop, diagnostics);
            }
        }
        RawExprKind::BlockCall { callee, block } => {
            validate_expr(callee, in_loop, diagnostics);
            validate_block(block, in_loop, diagnostics);
        }
        RawExprKind::ReceiverCall { receiver, args, .. } => {
            validate_expr(receiver, in_loop, diagnostics);
            for arg in args {
                validate_expr(&arg.value, in_loop, diagnostics);
            }
        }
        RawExprKind::Lambda(lambda) => {
            for param in &lambda.params {
                if let Some(default) = &param.default {
                    validate_expr(default, in_loop, diagnostics);
                }
            }
            validate_block(&lambda.body, false, diagnostics);
        }
        RawExprKind::Array(items) => {
            for item in items {
                validate_expr(item, in_loop, diagnostics);
            }
        }
        RawExprKind::Map(entries) => {
            for entry in entries {
                validate_expr(&entry.value, in_loop, diagnostics);
            }
        }
        RawExprKind::Match(match_expr) => {
            validate_expr(&match_expr.scrutinee, in_loop, diagnostics);
            for arm in &match_expr.arms {
                match &arm.body {
                    crate::core::seal::ast::RawMatchArmBody::Expr(expr) => {
                        validate_expr(expr, in_loop, diagnostics);
                    }
                    crate::core::seal::ast::RawMatchArmBody::Block(block) => {
                        validate_block(block, in_loop, diagnostics);
                    }
                }
            }
        }
        RawExprKind::Process(process) => {
            for arg in process.program.iter().chain(process.args.iter()) {
                if let crate::core::seal::ast::RawProcessArgKind::Spread(expr) = &arg.kind {
                    validate_expr(expr, in_loop, diagnostics);
                }
                if let crate::core::seal::ast::RawProcessArgKind::Word(parts) = &arg.kind {
                    for part in parts {
                        if let crate::core::seal::ast::RawProcessPart::Interpolation(expr) = part {
                            validate_expr(expr, in_loop, diagnostics);
                        }
                    }
                }
            }
        }
        RawExprKind::Ident(_)
        | RawExprKind::Literal(_)
        | RawExprKind::AtName(_)
        | RawExprKind::Env(_)
        | RawExprKind::Channel(_)
        | RawExprKind::Error => {}
    }
}
