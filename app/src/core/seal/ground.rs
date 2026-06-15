use super::{
    ast::{RawExpr, RawExprKind, RawItemKind, RawStatementKind, SourceFile},
    diag::Diagnostic,
    span::Span,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GroundOutput {
    pub file: GroundFile,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroundFile {
    pub nodes: Vec<GroundNode>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroundNode {
    Method { name: String, span: Span },
    Let { name: String, span: Span },
    Expr { span: Span },
    Effect { span: Span },
    Error { span: Span },
}

pub fn ground(file: &SourceFile) -> GroundOutput {
    let mut diagnostics = Vec::new();
    let mut nodes = Vec::new();

    for item in &file.items {
        match &item.kind {
            RawItemKind::Comment(_) => {}
            RawItemKind::Method(method) => {
                nodes.push(GroundNode::Method {
                    name: method.name.clone(),
                    span: item.span,
                });
            }
            RawItemKind::Statement(statement) => {
                nodes.push(ground_statement(statement, &mut diagnostics));
            }
            RawItemKind::Error => nodes.push(GroundNode::Error { span: item.span }),
        }
    }

    GroundOutput {
        file: GroundFile {
            nodes,
            span: file.span,
        },
        diagnostics,
    }
}

fn ground_statement(
    statement: &super::ast::RawStatement,
    diagnostics: &mut Vec<Diagnostic>,
) -> GroundNode {
    match &statement.kind {
        RawStatementKind::Let { name, value, .. } => {
            reject_comparison_chain(value, diagnostics);
            GroundNode::Let {
                name: name.clone(),
                span: statement.span,
            }
        }
        RawStatementKind::Assign { target, value } => {
            reject_comparison_chain(target, diagnostics);
            reject_comparison_chain(value, diagnostics);
            GroundNode::Expr {
                span: statement.span,
            }
        }
        RawStatementKind::If { .. }
        | RawStatementKind::For { .. }
        | RawStatementKind::While { .. }
        | RawStatementKind::WithEnv { .. } => {
            reject_statement_comparison_chains(statement, diagnostics);
            GroundNode::Expr {
                span: statement.span,
            }
        }
        RawStatementKind::Effect(expr) => {
            reject_comparison_chain(expr, diagnostics);
            GroundNode::Effect {
                span: statement.span,
            }
        }
        RawStatementKind::Expr(expr) => {
            reject_comparison_chain(expr, diagnostics);
            GroundNode::Expr {
                span: statement.span,
            }
        }
        RawStatementKind::Break | RawStatementKind::Continue => GroundNode::Expr {
            span: statement.span,
        },
        RawStatementKind::Error => GroundNode::Error {
            span: statement.span,
        },
    }
}

fn reject_statement_comparison_chains(
    statement: &super::ast::RawStatement,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match &statement.kind {
        RawStatementKind::Let { value, .. } => reject_comparison_chain(value, diagnostics),
        RawStatementKind::Assign { target, value } => {
            reject_comparison_chain(target, diagnostics);
            reject_comparison_chain(value, diagnostics);
        }
        RawStatementKind::Expr(expr) | RawStatementKind::Effect(expr) => {
            reject_comparison_chain(expr, diagnostics);
        }
        RawStatementKind::If {
            branches,
            else_branch,
        } => {
            for branch in branches {
                reject_comparison_chain(&branch.condition, diagnostics);
                for item in &branch.body.items {
                    if let RawItemKind::Statement(nested) = &item.kind {
                        reject_statement_comparison_chains(nested, diagnostics);
                    }
                }
            }
            if let Some(block) = else_branch {
                for item in &block.items {
                    if let RawItemKind::Statement(nested) = &item.kind {
                        reject_statement_comparison_chains(nested, diagnostics);
                    }
                }
            }
        }
        RawStatementKind::For { iterable, body, .. } => {
            reject_comparison_chain(iterable, diagnostics);
            for item in &body.items {
                if let RawItemKind::Statement(nested) = &item.kind {
                    reject_statement_comparison_chains(nested, diagnostics);
                }
            }
        }
        RawStatementKind::While { condition, body } => {
            reject_comparison_chain(condition, diagnostics);
            for item in &body.items {
                if let RawItemKind::Statement(nested) = &item.kind {
                    reject_statement_comparison_chains(nested, diagnostics);
                }
            }
        }
        RawStatementKind::WithEnv { bindings, body } => {
            for binding in bindings {
                reject_comparison_chain(&binding.value, diagnostics);
            }
            for item in &body.items {
                if let RawItemKind::Statement(nested) = &item.kind {
                    reject_statement_comparison_chains(nested, diagnostics);
                }
            }
        }
        RawStatementKind::Break | RawStatementKind::Continue | RawStatementKind::Error => {}
    }
}

fn reject_comparison_chain(expr: &RawExpr, diagnostics: &mut Vec<Diagnostic>) {
    match &expr.kind {
        RawExprKind::Binary { op, left, right } => {
            if op.is_comparison()
                && (matches!(&left.kind, RawExprKind::Binary { op: left_op, .. } if left_op.is_comparison())
                    || matches!(&right.kind, RawExprKind::Binary { op: right_op, .. } if right_op.is_comparison()))
            {
                diagnostics.push(Diagnostic::new(
                    expr.span,
                    "comparison operators cannot be chained",
                ));
            }
            reject_comparison_chain(left, diagnostics);
            reject_comparison_chain(right, diagnostics);
        }
        RawExprKind::Match(match_expr) => {
            reject_comparison_chain(&match_expr.scrutinee, diagnostics);
            for arm in &match_expr.arms {
                for pattern in &arm.patterns {
                    if let super::ast::RawPatternKind::Expr(expr) = &pattern.kind {
                        reject_comparison_chain(expr, diagnostics);
                    }
                }
                reject_comparison_chain(&arm.value, diagnostics);
            }
        }
        RawExprKind::Unary { expr, .. } | RawExprKind::Group(expr) => {
            reject_comparison_chain(expr, diagnostics);
        }
        RawExprKind::Call { callee, args } => {
            reject_comparison_chain(callee, diagnostics);
            for arg in args {
                reject_comparison_chain(&arg.value, diagnostics);
            }
        }
        RawExprKind::BlockCall { callee, block } => {
            reject_comparison_chain(callee, diagnostics);
            for item in &block.items {
                if let RawItemKind::Statement(statement) = &item.kind {
                    reject_statement_comparison_chains(statement, diagnostics);
                }
            }
        }
        RawExprKind::ReceiverCall { receiver, args, .. } => {
            reject_comparison_chain(receiver, diagnostics);
            for arg in args {
                reject_comparison_chain(&arg.value, diagnostics);
            }
        }
        RawExprKind::Array(items) => {
            for item in items {
                reject_comparison_chain(item, diagnostics);
            }
        }
        RawExprKind::Map(entries) => {
            for entry in entries {
                reject_comparison_chain(&entry.value, diagnostics);
            }
        }
        RawExprKind::StreamFlow { left, right, .. } => {
            reject_comparison_chain(left, diagnostics);
            reject_comparison_chain(right, diagnostics);
        }
        RawExprKind::Process(process) => {
            for arg in process.program.iter().chain(process.args.iter()) {
                match &arg.kind {
                    super::ast::RawProcessArgKind::Spread(expr) => {
                        reject_comparison_chain(expr, diagnostics);
                    }
                    super::ast::RawProcessArgKind::Word(parts) => {
                        for part in parts {
                            if let super::ast::RawProcessPart::Interpolation(expr) = part {
                                reject_comparison_chain(expr, diagnostics);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
