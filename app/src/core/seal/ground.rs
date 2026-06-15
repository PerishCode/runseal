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
    Method {
        name: String,
        tail: TailOutput,
        span: Span,
    },
    Let {
        name: String,
        span: Span,
    },
    Expr {
        span: Span,
    },
    Effect {
        span: Span,
    },
    Error {
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TailOutput {
    Implicit { span: Span },
    DisabledByStdout { span: Span },
    None,
}

pub fn ground(file: &SourceFile) -> GroundOutput {
    let mut diagnostics = Vec::new();
    let mut nodes = Vec::new();

    for item in &file.items {
        match &item.kind {
            RawItemKind::Comment(_) => {}
            RawItemKind::Method(method) => {
                let tail = method_tail_output(&method.body, &mut diagnostics);
                nodes.push(GroundNode::Method {
                    name: method.name.clone(),
                    tail,
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

fn method_tail_output(
    body: &super::ast::RawBlock,
    diagnostics: &mut Vec<Diagnostic>,
) -> TailOutput {
    if let Some(span) = find_current_stdout_block(body) {
        return TailOutput::DisabledByStdout { span };
    }

    for item in body.items.iter().rev() {
        match &item.kind {
            RawItemKind::Comment(_) => continue,
            RawItemKind::Statement(statement) => {
                reject_statement_comparison_chains(statement, diagnostics);
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

fn find_current_stdout_block(block: &super::ast::RawBlock) -> Option<Span> {
    block.items.iter().find_map(|item| match &item.kind {
        RawItemKind::Statement(statement) => find_current_stdout_statement(statement),
        RawItemKind::Comment(_) | RawItemKind::Method(_) | RawItemKind::Error => None,
    })
}

fn find_current_stdout_statement(statement: &super::ast::RawStatement) -> Option<Span> {
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
                        .find_map(|pattern| match &pattern.kind {
                            super::ast::RawPatternKind::Expr(expr) => {
                                find_current_stdout_expr(expr)
                            }
                            super::ast::RawPatternKind::Wildcard => None,
                        })
                        .or_else(|| find_current_stdout_expr(&arm.value))
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

fn find_stdout_arg(arg: &super::ast::RawProcessArg) -> Option<Span> {
    match &arg.kind {
        super::ast::RawProcessArgKind::Spread(expr) => find_current_stdout_expr(expr),
        super::ast::RawProcessArgKind::Word(parts) => parts.iter().find_map(|part| match part {
            super::ast::RawProcessPart::Interpolation(expr) => find_current_stdout_expr(expr),
            super::ast::RawProcessPart::Text(_) => None,
        }),
        super::ast::RawProcessArgKind::String(_)
        | super::ast::RawProcessArgKind::TextBlock(_)
        | super::ast::RawProcessArgKind::Error => None,
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
            validate_effect_block(block, diagnostics);
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

fn validate_effect_block(block: &super::ast::RawBlock, diagnostics: &mut Vec<Diagnostic>) {
    let statements = block
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            RawItemKind::Statement(statement) => Some(statement),
            RawItemKind::Comment(_) => None,
            RawItemKind::Method(_) | RawItemKind::Error => None,
        })
        .collect::<Vec<_>>();

    let valid = matches!(
        statements.as_slice(),
        [statement] if matches!(statement.kind, RawStatementKind::Effect(_))
    );
    if !valid {
        diagnostics.push(Diagnostic::new(
            block.span,
            "effect block must contain exactly one stream graph",
        ));
    }
}
