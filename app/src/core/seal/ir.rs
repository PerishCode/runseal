use super::{
    ground::{GroundFile, GroundNode, TailOutput},
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
        value: Option<IrExpr>,
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
pub enum IrExpr {
    Local {
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
        left: Box<IrEffect>,
        right: Box<IrEffect>,
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
        GroundNode::Let { name, span } => IrItem::Statement(IrStatement::Let {
            name: name.clone(),
            binding: IrBinding::Value,
            value: None,
            span: *span,
        }),
        GroundNode::Expr { span } => IrItem::Statement(IrStatement::Expr {
            expr: None,
            span: *span,
        }),
        GroundNode::Effect { span } => IrItem::Statement(IrStatement::Effect {
            effect: None,
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

impl IrExpr {
    pub fn local(name: impl Into<String>, span: Span) -> Self {
        Self::Local {
            name: name.into(),
            span,
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
