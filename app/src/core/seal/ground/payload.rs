use crate::core::seal::{
    ast::{
        LetBinding, RawExpr, RawExprKind, RawItemKind, RawLiteral, RawProcessArg,
        RawProcessArgKind, RawProcessPart, RawStatementKind, StreamOp,
    },
    span::Span,
};

#[derive(Debug, Clone, PartialEq)]
pub enum GroundExpr {
    Local {
        name: String,
        span: Span,
    },
    Env {
        name: String,
        span: Span,
    },
    Channel {
        name: String,
        span: Span,
    },
    Literal {
        value: GroundLiteral,
        span: Span,
    },
    Array {
        items: Vec<GroundExpr>,
        span: Span,
    },
    Map {
        entries: Vec<(String, GroundExpr)>,
        span: Span,
    },
    Process {
        program: GroundArgv,
        args: Vec<GroundArgv>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroundValueSource {
    Pure(GroundExpr),
    StreamView(GroundExpr),
    TypeAbsorb {
        kind: GroundTypeKind,
        call: GroundExpr,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroundLiteral {
    String(String),
    Int(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroundTypeKind {
    String,
    Bytes,
    Array,
    Map,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroundArgv {
    Text { value: String, span: Span },
    Expr { expr: Box<GroundExpr>, span: Span },
    Spread { expr: Box<GroundExpr>, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroundEffect {
    Call {
        expr: GroundExpr,
        span: Span,
    },
    Flow {
        op: StreamOp,
        left: GroundExpr,
        right: GroundExpr,
        span: Span,
    },
}

pub(super) fn ground_value_source(
    binding: LetBinding,
    value: &RawExpr,
) -> Option<GroundValueSource> {
    match binding {
        LetBinding::Value => {
            ground_type_absorb(value).or_else(|| Some(GroundValueSource::Pure(ground_expr(value)?)))
        }
        LetBinding::Stream => Some(GroundValueSource::StreamView(ground_expr(value)?)),
    }
}

pub(super) fn ground_expr(expr: &RawExpr) -> Option<GroundExpr> {
    match &expr.kind {
        RawExprKind::Ident(name) => Some(GroundExpr::Local {
            name: name.clone(),
            span: expr.span,
        }),
        RawExprKind::Env(name) => Some(GroundExpr::Env {
            name: name.clone(),
            span: expr.span,
        }),
        RawExprKind::Channel(name) => Some(GroundExpr::Channel {
            name: name.clone(),
            span: expr.span,
        }),
        RawExprKind::Literal(literal) => Some(GroundExpr::Literal {
            value: ground_literal(literal),
            span: expr.span,
        }),
        RawExprKind::Array(items) => Some(GroundExpr::Array {
            items: items.iter().filter_map(ground_expr).collect(),
            span: expr.span,
        }),
        RawExprKind::Map(entries) => Some(GroundExpr::Map {
            entries: entries
                .iter()
                .filter_map(|entry| Some((entry.key.clone(), ground_expr(&entry.value)?)))
                .collect(),
            span: expr.span,
        }),
        RawExprKind::Process(process) => {
            let program = process.program.as_ref().and_then(ground_argv)?;
            Some(GroundExpr::Process {
                program,
                args: process.args.iter().filter_map(ground_argv).collect(),
                span: expr.span,
            })
        }
        RawExprKind::StreamFlow { .. } => None,
        RawExprKind::Group(inner) => ground_expr(inner),
        _ => None,
    }
}

pub(super) fn ground_effect(expr: &RawExpr) -> Option<GroundEffect> {
    match &expr.kind {
        RawExprKind::Process(_) => Some(GroundEffect::Call {
            expr: ground_expr(expr)?,
            span: expr.span,
        }),
        RawExprKind::StreamFlow { op, left, right } => Some(GroundEffect::Flow {
            op: *op,
            left: ground_expr(left)?,
            right: ground_expr(right)?,
            span: expr.span,
        }),
        RawExprKind::Group(inner) => ground_effect(inner),
        _ => None,
    }
}

fn ground_type_absorb(expr: &RawExpr) -> Option<GroundValueSource> {
    match &expr.kind {
        RawExprKind::Call { callee, args } => {
            let kind = type_absorb_kind(callee)?;
            let [arg] = args.as_slice() else {
                return None;
            };
            if arg.label.is_some() {
                return None;
            }
            let call = ground_type_absorb_call(&arg.value)?;
            Some(GroundValueSource::TypeAbsorb {
                kind,
                call,
                span: expr.span,
            })
        }
        RawExprKind::BlockCall { callee, block } => {
            let kind = type_absorb_kind(callee)?;
            let call = ground_type_absorb_block(block)?;
            Some(GroundValueSource::TypeAbsorb {
                kind,
                call,
                span: expr.span,
            })
        }
        RawExprKind::Group(inner) => ground_type_absorb(inner),
        _ => None,
    }
}

fn ground_type_absorb_block(block: &crate::core::seal::ast::RawBlock) -> Option<GroundExpr> {
    let mut statements = block.items.iter().filter_map(|item| match &item.kind {
        RawItemKind::Statement(statement) => Some(statement),
        RawItemKind::Comment(_) | RawItemKind::Method(_) | RawItemKind::Error => None,
    });
    let statement = statements.next()?;
    if statements.next().is_some() {
        return None;
    }
    let RawStatementKind::Effect(expr) = &statement.kind else {
        return None;
    };
    ground_type_absorb_call(expr)
}

fn ground_type_absorb_call(expr: &RawExpr) -> Option<GroundExpr> {
    match &expr.kind {
        RawExprKind::Process(_) => ground_expr(expr),
        RawExprKind::Group(inner) => ground_type_absorb_call(inner),
        _ => None,
    }
}

fn type_absorb_kind(callee: &RawExpr) -> Option<GroundTypeKind> {
    let RawExprKind::AtName(parts) = &callee.kind else {
        return None;
    };
    let [namespace, name] = parts.as_slice() else {
        return None;
    };
    if namespace != "type" {
        return None;
    }
    match name.as_str() {
        "string" => Some(GroundTypeKind::String),
        "bytes" => Some(GroundTypeKind::Bytes),
        "array" => Some(GroundTypeKind::Array),
        "map" => Some(GroundTypeKind::Map),
        _ => None,
    }
}

fn ground_literal(literal: &RawLiteral) -> GroundLiteral {
    match literal {
        RawLiteral::String(value) | RawLiteral::TextBlock(value) => {
            GroundLiteral::String(value.clone())
        }
        RawLiteral::Int(value) => GroundLiteral::Int(value.clone()),
        RawLiteral::Bool(value) => GroundLiteral::Bool(*value),
        RawLiteral::Null => GroundLiteral::Null,
    }
}

fn ground_argv(arg: &RawProcessArg) -> Option<GroundArgv> {
    match &arg.kind {
        RawProcessArgKind::Word(parts) => ground_word_argv(parts, arg.span),
        RawProcessArgKind::String(value) | RawProcessArgKind::TextBlock(value) => {
            Some(GroundArgv::Text {
                value: value.clone(),
                span: arg.span,
            })
        }
        RawProcessArgKind::Spread(expr) => Some(GroundArgv::Spread {
            expr: Box::new(ground_expr(expr)?),
            span: arg.span,
        }),
        RawProcessArgKind::Error => None,
    }
}

fn ground_word_argv(parts: &[RawProcessPart], span: Span) -> Option<GroundArgv> {
    match parts {
        [RawProcessPart::Text(value)] => Some(GroundArgv::Text {
            value: value.clone(),
            span,
        }),
        [RawProcessPart::Interpolation(expr)] => Some(GroundArgv::Expr {
            expr: Box::new(ground_expr(expr)?),
            span,
        }),
        _ => None,
    }
}
