use super::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriviaKind {
    Whitespace,
    LineComment,
    BlockComment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub text: String,
    pub span: Span,
}

impl Trivia {
    pub fn new(kind: TriviaKind, text: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            text: text.into(),
            span,
        }
    }

    pub fn is_comment(&self) -> bool {
        matches!(
            self.kind,
            TriviaKind::LineComment | TriviaKind::BlockComment
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Method,
    Let,
    If,
    Else,
    Match,
    For,
    In,
    While,
    Break,
    Continue,
    With,
    Env,
    True,
    False,
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Newline,
    Ident,
    Number,
    String,
    TextBlock,
    Keyword(Keyword),
    At,
    Dollar,
    Hash,
    Pipe,
    ShiftRight,
    ShiftLeft,
    OrOr,
    AndAnd,
    FatArrow,
    ColonEq,
    EqEq,
    BangEq,
    LessEq,
    GreaterEq,
    QuestionQuestion,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Eq,
    Less,
    Greater,
    Question,
    Dot,
    Comma,
    Colon,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Underscore,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
    pub leading: Vec<Trivia>,
}

impl Token {
    pub fn new(kind: TokenKind, text: impl Into<String>, span: Span, leading: Vec<Trivia>) -> Self {
        Self {
            kind,
            text: text.into(),
            span,
            leading,
        }
    }

    pub fn has_leading_whitespace(&self) -> bool {
        self.leading
            .iter()
            .any(|trivia| matches!(trivia.kind, TriviaKind::Whitespace))
    }

    pub fn leading_comments(&self) -> impl Iterator<Item = &Trivia> {
        self.leading.iter().filter(|trivia| trivia.is_comment())
    }
}
