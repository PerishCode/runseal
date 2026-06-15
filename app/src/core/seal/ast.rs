use super::span::Span;

pub type CommentId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub span: Span,
    pub text: String,
    pub kind: CommentKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentKind {
    Line,
    Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFile {
    pub items: Vec<RawItem>,
    pub comments: Vec<Comment>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawItem {
    pub kind: RawItemKind,
    pub span: Span,
    pub leading_comments: Vec<CommentId>,
    pub trailing_comments: Vec<CommentId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawItemKind {
    Method(RawMethod),
    Statement(RawStatement),
    Comment(CommentId),
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawMethod {
    pub name: String,
    pub params: Vec<RawParam>,
    pub body: RawBlock,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawParam {
    pub name: String,
    pub default: Option<RawExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawBlock {
    pub items: Vec<RawItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawStatement {
    pub kind: RawStatementKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawStatementKind {
    Let {
        name: String,
        binding: LetBinding,
        value: RawExpr,
    },
    Assign {
        target: RawExpr,
        value: RawExpr,
    },
    If {
        branches: Vec<RawIfBranch>,
        else_branch: Option<RawBlock>,
    },
    For {
        binding: String,
        iterable: RawExpr,
        body: RawBlock,
    },
    While {
        condition: RawExpr,
        body: RawBlock,
    },
    WithEnv {
        bindings: Vec<RawEnvBinding>,
        body: RawBlock,
    },
    Expr(RawExpr),
    Effect(RawExpr),
    Break,
    Continue,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LetBinding {
    Value,
    Stream,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawIfBranch {
    pub condition: RawExpr,
    pub body: RawBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawEnvBinding {
    pub name: String,
    pub value: RawExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawExpr {
    pub kind: RawExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawExprKind {
    Ident(String),
    Literal(RawLiteral),
    AtName(Vec<String>),
    Env(String),
    Channel(String),
    Array(Vec<RawExpr>),
    Map(Vec<RawMapEntry>),
    Call {
        callee: Box<RawExpr>,
        args: Vec<RawArg>,
    },
    BlockCall {
        callee: Box<RawExpr>,
        block: RawBlock,
    },
    ReceiverCall {
        receiver: Box<RawExpr>,
        method: String,
        args: Vec<RawArg>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<RawExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<RawExpr>,
        right: Box<RawExpr>,
    },
    Match(RawMatch),
    Process(RawProcess),
    StreamFlow {
        op: StreamOp,
        left: Box<RawExpr>,
        right: Box<RawExpr>,
    },
    Group(Box<RawExpr>),
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawLiteral {
    String(String),
    TextBlock(String),
    Int(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawMapEntry {
    pub key: String,
    pub value: RawExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawArg {
    pub label: Option<String>,
    pub value: RawExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawMatch {
    pub scrutinee: Box<RawExpr>,
    pub arms: Vec<RawMatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawMatchArm {
    pub patterns: Vec<RawPattern>,
    pub value: RawExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawPattern {
    pub kind: RawPatternKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawPatternKind {
    Wildcard,
    Expr(RawExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawProcess {
    pub program: Option<RawProcessArg>,
    pub args: Vec<RawProcessArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawProcessArg {
    pub kind: RawProcessArgKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawProcessArgKind {
    Word(Vec<RawProcessPart>),
    String(String),
    TextBlock(String),
    Spread(Box<RawExpr>),
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawProcessPart {
    Text(String),
    Interpolation(RawExpr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    In,
    Eq,
    NotEq,
    And,
    Or,
    NullCoalesce,
}

impl BinaryOp {
    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Less | Self::LessEq | Self::Greater | Self::GreaterEq | Self::In
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamOp {
    To,
    From,
}

impl RawExpr {
    pub fn error(span: Span) -> Self {
        Self {
            kind: RawExprKind::Error,
            span,
        }
    }
}
