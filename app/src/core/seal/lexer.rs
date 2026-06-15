use super::{
    diag::Diagnostic,
    span::Span,
    token::{Keyword, Token, TokenKind, Trivia, TriviaKind},
};

#[derive(Debug, Clone)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &str) -> LexOutput {
    Lexer::new(source).lex()
}

struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    trivia_boundary: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
            trivia_boundary: true,
        }
    }

    fn lex(mut self) -> LexOutput {
        while !self.is_eof() {
            let leading = self.collect_leading_trivia();
            let start = self.pos;
            let Some(ch) = self.peek_char() else {
                break;
            };

            if ch == '\n' {
                self.bump_char();
                self.push(TokenKind::Newline, start, self.pos, leading);
                self.trivia_boundary = true;
                continue;
            }
            if ch == '\r' {
                self.bump_char();
                if self.peek_char() == Some('\n') {
                    self.bump_char();
                }
                self.push(TokenKind::Newline, start, self.pos, leading);
                self.trivia_boundary = true;
                continue;
            }

            if is_ident_start(ch) {
                self.lex_ident_or_keyword(start, leading);
                self.trivia_boundary = false;
                continue;
            }
            if ch.is_ascii_digit() {
                self.lex_number(start, leading);
                self.trivia_boundary = false;
                continue;
            }

            let kind = match ch {
                '"' => {
                    self.lex_string(start, leading);
                    self.trivia_boundary = false;
                    continue;
                }
                '`' => {
                    self.lex_text_block(start, leading);
                    self.trivia_boundary = false;
                    continue;
                }
                '@' => single(&mut self, TokenKind::At),
                '$' => single(&mut self, TokenKind::Dollar),
                '#' => single(&mut self, TokenKind::Hash),
                '|' => {
                    if self.starts_with("||") {
                        double(&mut self, TokenKind::OrOr)
                    } else {
                        single(&mut self, TokenKind::Pipe)
                    }
                }
                '>' => {
                    if self.starts_with(">>") {
                        double(&mut self, TokenKind::ShiftRight)
                    } else if self.starts_with(">=") {
                        double(&mut self, TokenKind::GreaterEq)
                    } else {
                        single(&mut self, TokenKind::Greater)
                    }
                }
                '<' => {
                    if self.starts_with("<<") {
                        double(&mut self, TokenKind::ShiftLeft)
                    } else if self.starts_with("<=") {
                        double(&mut self, TokenKind::LessEq)
                    } else {
                        single(&mut self, TokenKind::Less)
                    }
                }
                '&' if self.starts_with("&&") => double(&mut self, TokenKind::AndAnd),
                '=' => {
                    if self.starts_with("=>") {
                        double(&mut self, TokenKind::FatArrow)
                    } else if self.starts_with("==") {
                        double(&mut self, TokenKind::EqEq)
                    } else {
                        single(&mut self, TokenKind::Eq)
                    }
                }
                ':' => {
                    if self.starts_with(":=") {
                        double(&mut self, TokenKind::ColonEq)
                    } else {
                        single(&mut self, TokenKind::Colon)
                    }
                }
                '!' => {
                    if self.starts_with("!=") {
                        double(&mut self, TokenKind::BangEq)
                    } else {
                        single(&mut self, TokenKind::Bang)
                    }
                }
                '?' => {
                    if self.starts_with("??") {
                        double(&mut self, TokenKind::QuestionQuestion)
                    } else {
                        single(&mut self, TokenKind::Question)
                    }
                }
                '+' => single(&mut self, TokenKind::Plus),
                '-' => single(&mut self, TokenKind::Minus),
                '*' => single(&mut self, TokenKind::Star),
                '/' => single(&mut self, TokenKind::Slash),
                '%' => single(&mut self, TokenKind::Percent),
                '.' => single(&mut self, TokenKind::Dot),
                ',' => single(&mut self, TokenKind::Comma),
                ';' => single(&mut self, TokenKind::Semicolon),
                '(' => single(&mut self, TokenKind::LParen),
                ')' => single(&mut self, TokenKind::RParen),
                '{' => single(&mut self, TokenKind::LBrace),
                '}' => single(&mut self, TokenKind::RBrace),
                '[' => single(&mut self, TokenKind::LBracket),
                ']' => single(&mut self, TokenKind::RBracket),
                '_' => single(&mut self, TokenKind::Underscore),
                _ => {
                    self.bump_char();
                    self.diagnostics.push(Diagnostic::new(
                        Span::new(start, self.pos),
                        format!("unexpected character {ch:?}"),
                    ));
                    TokenKind::Unknown
                }
            };

            self.push(kind, start, self.pos, leading);
            self.trivia_boundary = false;
        }

        let leading = self.collect_leading_trivia();
        self.tokens.push(Token::new(
            TokenKind::Eof,
            "",
            Span::empty(self.pos),
            leading,
        ));

        LexOutput {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn collect_leading_trivia(&mut self) -> Vec<Trivia> {
        let mut trivia = Vec::new();
        loop {
            let start = self.pos;
            let Some(ch) = self.peek_char() else {
                break;
            };
            if matches!(ch, ' ' | '\t') {
                while matches!(self.peek_char(), Some(' ' | '\t')) {
                    self.bump_char();
                }
                trivia.push(Trivia::new(
                    TriviaKind::Whitespace,
                    &self.source[start..self.pos],
                    Span::new(start, self.pos),
                ));
                self.trivia_boundary = true;
                continue;
            }
            if self.trivia_boundary && self.starts_with("//") {
                self.pos += 2;
                while let Some(next) = self.peek_char() {
                    if matches!(next, '\n' | '\r') {
                        break;
                    }
                    self.bump_char();
                }
                trivia.push(Trivia::new(
                    TriviaKind::LineComment,
                    &self.source[start..self.pos],
                    Span::new(start, self.pos),
                ));
                self.trivia_boundary = true;
                continue;
            }
            if self.trivia_boundary && self.starts_with("/*") {
                self.pos += 2;
                while !self.is_eof() && !self.starts_with("*/") {
                    self.bump_char();
                }
                if self.starts_with("*/") {
                    self.pos += 2;
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        Span::new(start, self.pos),
                        "unterminated block comment",
                    ));
                }
                trivia.push(Trivia::new(
                    TriviaKind::BlockComment,
                    &self.source[start..self.pos],
                    Span::new(start, self.pos),
                ));
                self.trivia_boundary = true;
                continue;
            }
            break;
        }
        trivia
    }

    fn lex_ident_or_keyword(&mut self, start: usize, leading: Vec<Trivia>) {
        self.bump_char();
        while matches!(self.peek_char(), Some(ch) if is_ident_continue(ch)) {
            self.bump_char();
        }
        let text = &self.source[start..self.pos];
        let kind = match text {
            "method" => TokenKind::Keyword(Keyword::Method),
            "let" => TokenKind::Keyword(Keyword::Let),
            "if" => TokenKind::Keyword(Keyword::If),
            "else" => TokenKind::Keyword(Keyword::Else),
            "match" => TokenKind::Keyword(Keyword::Match),
            "for" => TokenKind::Keyword(Keyword::For),
            "in" => TokenKind::Keyword(Keyword::In),
            "while" => TokenKind::Keyword(Keyword::While),
            "break" => TokenKind::Keyword(Keyword::Break),
            "continue" => TokenKind::Keyword(Keyword::Continue),
            "with" => TokenKind::Keyword(Keyword::With),
            "env" => TokenKind::Keyword(Keyword::Env),
            "true" => TokenKind::Keyword(Keyword::True),
            "false" => TokenKind::Keyword(Keyword::False),
            "null" => TokenKind::Keyword(Keyword::Null),
            _ => TokenKind::Ident,
        };
        self.push(kind, start, self.pos, leading);
    }

    fn lex_number(&mut self, start: usize, leading: Vec<Trivia>) {
        self.bump_char();
        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
            self.bump_char();
        }
        self.push(TokenKind::Number, start, self.pos, leading);
    }

    fn lex_string(&mut self, start: usize, leading: Vec<Trivia>) {
        self.bump_char();
        let mut escaped = false;
        while let Some(ch) = self.peek_char() {
            self.bump_char();
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                self.push(TokenKind::String, start, self.pos, leading);
                return;
            }
            if matches!(ch, '\n' | '\r') {
                break;
            }
        }
        self.diagnostics.push(Diagnostic::new(
            Span::new(start, self.pos),
            "unterminated string literal",
        ));
        self.push(TokenKind::String, start, self.pos, leading);
    }

    fn lex_text_block(&mut self, start: usize, leading: Vec<Trivia>) {
        self.bump_char();
        while let Some(ch) = self.peek_char() {
            self.bump_char();
            if ch == '`' {
                self.push(TokenKind::TextBlock, start, self.pos, leading);
                return;
            }
        }
        self.diagnostics.push(Diagnostic::new(
            Span::new(start, self.pos),
            "unterminated text block",
        ));
        self.push(TokenKind::TextBlock, start, self.pos, leading);
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize, leading: Vec<Trivia>) {
        self.tokens.push(Token::new(
            kind,
            &self.source[start..end],
            Span::new(start, end),
            leading,
        ));
    }

    fn starts_with(&self, needle: &str) -> bool {
        self.source[self.pos..].starts_with(needle)
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }
}

fn single(lexer: &mut Lexer<'_>, kind: TokenKind) -> TokenKind {
    lexer.bump_char();
    kind
}

fn double(lexer: &mut Lexer<'_>, kind: TokenKind) -> TokenKind {
    lexer.pos += 2;
    kind
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
