use super::{
    ast::LetBinding,
    ground::{
        GroundArgv, GroundEffect, GroundExpr, GroundFile, GroundLiteral, GroundNode,
        GroundTypeKind, TailOutput,
    },
    span::Span,
};

#[derive(Debug, Clone, PartialEq)]
pub struct IrProgram {
    pub items: Vec<IrItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrItem {
    Method(IrMethod),
    Statement(IrStatement),
    Error { span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrMethod {
    pub name: String,
    pub frame: IrFrame,
    pub tail: IrTailOutput,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFrame {
    pub kind: IrFrameKind,
    pub body: Vec<IrStatement>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrFrameKind {
    Method,
    Lambda,
    Handler,
    Process,
    Tool,
    Builtin,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrStatement {
    Let {
        name: String,
        binding: IrBinding,
        source: Option<IrValueSource>,
        span: Span,
    },
    Expr {
        expr: Option<IrExpr>,
        span: Span,
    },
    Effect {
        effect: Option<IrEffect>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Error {
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrBinding {
    Value,
    StreamView,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrValueSource {
    Pure(IrExpr),
    StreamView(IrExpr),
    TypeAbsorb {
        kind: IrTypeKind,
        call: Box<IrCall>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrExpr {
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
        value: IrLiteral,
        span: Span,
    },
    Array {
        items: Vec<IrExpr>,
        span: Span,
    },
    Map {
        entries: Vec<(String, IrExpr)>,
        span: Span,
    },
    Call(Box<IrCall>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrLiteral {
    String(String),
    Int(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrCall {
    pub kind: IrCallKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrCallKind {
    Forward {
        callable: Box<IrExpr>,
        args: Vec<IrExpr>,
    },
    Process {
        program: IrArgv,
        args: Vec<IrArgv>,
    },
    Receiver {
        receiver: Box<IrExpr>,
        method: String,
        args: Vec<IrExpr>,
    },
    Named {
        path: Vec<String>,
        args: Vec<IrArg>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrArg {
    pub label: Option<String>,
    pub value: IrExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrArgv {
    Text { value: String, span: Span },
    Expr { expr: IrExpr, span: Span },
    Spread { expr: IrExpr, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrEffect {
    Call(Box<IrCall>),
    Flow {
        op: IrStreamOp,
        left: Box<IrExpr>,
        right: Box<IrExpr>,
        span: Span,
    },
    Pipeline {
        stages: Vec<IrEffect>,
        span: Span,
    },
    TypeAbsorb {
        kind: IrTypeKind,
        call: Box<IrCall>,
        span: Span,
    },
    StreamDupe {
        call: Box<IrCall>,
        span: Span,
    },
    Exit {
        value: Option<IrExpr>,
        event: Option<IrExpr>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrStreamOp {
    To,
    From,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrTypeKind {
    String,
    Bytes,
    Array,
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrTailOutput {
    Implicit { span: Span },
    DisabledByStdout { span: Span },
    None,
}

pub fn lower(file: &GroundFile) -> IrProgram {
    IrProgram {
        items: file.nodes.iter().map(lower_node).collect(),
        span: file.span,
    }
}

fn lower_node(node: &GroundNode) -> IrItem {
    match node {
        GroundNode::Method { name, tail, span } => IrItem::Method(IrMethod {
            name: name.clone(),
            frame: IrFrame {
                kind: IrFrameKind::Method,
                body: Vec::new(),
                span: *span,
            },
            tail: IrTailOutput::from_ground(tail),
            span: *span,
        }),
        GroundNode::Let {
            name,
            binding,
            source,
            span,
        } => IrItem::Statement(IrStatement::Let {
            name: name.clone(),
            binding: IrBinding::from_raw(*binding),
            source: source.as_ref().map(IrValueSource::from_ground),
            span: *span,
        }),
        GroundNode::Expr { expr, span } => IrItem::Statement(IrStatement::Expr {
            expr: expr.as_ref().map(IrExpr::from_ground),
            span: *span,
        }),
        GroundNode::Effect { effect, span } => IrItem::Statement(IrStatement::Effect {
            effect: effect.as_ref().map(IrEffect::from_ground),
            span: *span,
        }),
        GroundNode::Error { span } => IrItem::Error { span: *span },
    }
}

impl IrTailOutput {
    pub fn from_ground(tail: &TailOutput) -> Self {
        match tail {
            TailOutput::Implicit { span } => Self::Implicit { span: *span },
            TailOutput::DisabledByStdout { span } => Self::DisabledByStdout { span: *span },
            TailOutput::None => Self::None,
        }
    }
}

impl IrBinding {
    fn from_raw(binding: LetBinding) -> Self {
        match binding {
            LetBinding::Value => Self::Value,
            LetBinding::Stream => Self::StreamView,
        }
    }
}

impl IrExpr {
    pub fn local(name: impl Into<String>, span: Span) -> Self {
        Self::Local {
            name: name.into(),
            span,
        }
    }

    fn from_ground(expr: &GroundExpr) -> Self {
        match expr {
            GroundExpr::Local { name, span } => Self::Local {
                name: name.clone(),
                span: *span,
            },
            GroundExpr::Env { name, span } => Self::Env {
                name: name.clone(),
                span: *span,
            },
            GroundExpr::Channel { name, span } => Self::Channel {
                name: name.clone(),
                span: *span,
            },
            GroundExpr::Literal { value, span } => Self::Literal {
                value: IrLiteral::from_ground(value),
                span: *span,
            },
            GroundExpr::Array { items, span } => Self::Array {
                items: items.iter().map(Self::from_ground).collect(),
                span: *span,
            },
            GroundExpr::Map { entries, span } => Self::Map {
                entries: entries
                    .iter()
                    .map(|(key, value)| (key.clone(), Self::from_ground(value)))
                    .collect(),
                span: *span,
            },
            GroundExpr::Process {
                program,
                args,
                span,
            } => Self::Call(Box::new(IrCall::process(
                IrArgv::from_ground(program),
                args.iter().map(IrArgv::from_ground).collect(),
                *span,
            ))),
        }
    }
}

impl IrValueSource {
    fn from_ground(source: &super::ground::GroundValueSource) -> Self {
        match source {
            super::ground::GroundValueSource::Pure(expr) => Self::Pure(IrExpr::from_ground(expr)),
            super::ground::GroundValueSource::StreamView(expr) => {
                Self::StreamView(IrExpr::from_ground(expr))
            }
            super::ground::GroundValueSource::TypeAbsorb { kind, call, span } => {
                let IrExpr::Call(call) = IrExpr::from_ground(call) else {
                    unreachable!("@type.* value sources currently lower call expressions only");
                };
                Self::TypeAbsorb {
                    kind: IrTypeKind::from_ground(*kind),
                    call,
                    span: *span,
                }
            }
        }
    }
}

impl IrLiteral {
    fn from_ground(literal: &GroundLiteral) -> Self {
        match literal {
            GroundLiteral::String(value) => Self::String(value.clone()),
            GroundLiteral::Int(value) => Self::Int(value.clone()),
            GroundLiteral::Bool(value) => Self::Bool(*value),
            GroundLiteral::Null => Self::Null,
        }
    }
}

impl IrArgv {
    fn from_ground(arg: &GroundArgv) -> Self {
        match arg {
            GroundArgv::Text { value, span } => Self::Text {
                value: value.clone(),
                span: *span,
            },
            GroundArgv::Expr { expr, span } => Self::Expr {
                expr: IrExpr::from_ground(expr),
                span: *span,
            },
            GroundArgv::Spread { expr, span } => Self::Spread {
                expr: IrExpr::from_ground(expr),
                span: *span,
            },
        }
    }
}

impl IrEffect {
    fn from_ground(effect: &GroundEffect) -> Self {
        match effect {
            GroundEffect::Call { expr, .. } => {
                let IrExpr::Call(call) = IrExpr::from_ground(expr) else {
                    unreachable!("ground call effects only contain call expressions");
                };
                Self::Call(call)
            }
            GroundEffect::Flow {
                op,
                left,
                right,
                span,
            } => Self::Flow {
                op: IrStreamOp::from_ground(*op),
                left: Box::new(IrExpr::from_ground(left)),
                right: Box::new(IrExpr::from_ground(right)),
                span: *span,
            },
        }
    }
}

impl IrStreamOp {
    fn from_ground(op: super::ast::StreamOp) -> Self {
        match op {
            super::ast::StreamOp::To => Self::To,
            super::ast::StreamOp::From => Self::From,
        }
    }
}

impl IrTypeKind {
    fn from_ground(kind: GroundTypeKind) -> Self {
        match kind {
            GroundTypeKind::String => Self::String,
            GroundTypeKind::Bytes => Self::Bytes,
            GroundTypeKind::Array => Self::Array,
            GroundTypeKind::Map => Self::Map,
        }
    }
}

impl IrCall {
    pub fn forward(callable: IrExpr, args: Vec<IrExpr>, span: Span) -> Self {
        Self {
            kind: IrCallKind::Forward {
                callable: Box::new(callable),
                args,
            },
            span,
        }
    }

    pub fn process(program: IrArgv, args: Vec<IrArgv>, span: Span) -> Self {
        Self {
            kind: IrCallKind::Process { program, args },
            span,
        }
    }

    pub fn receiver(
        receiver: IrExpr,
        method: impl Into<String>,
        args: Vec<IrExpr>,
        span: Span,
    ) -> Self {
        Self {
            kind: IrCallKind::Receiver {
                receiver: Box::new(receiver),
                method: method.into(),
                args,
            },
            span,
        }
    }
}
