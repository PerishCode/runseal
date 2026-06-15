use super::{
    ast::*,
    diag::Diagnostic,
    lexer,
    span::Span,
    token::{Keyword, Token, TokenKind, TriviaKind},
};

mod expr;
mod process;
mod statement;

#[derive(Debug, Clone)]
pub struct ParseOutput {
    pub file: SourceFile,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(source: &str) -> ParseOutput {
    let lexed = lexer::lex(source);
    let mut parser = Parser::new(lexed.tokens, lexed.diagnostics);
    parser.parse_source_file()
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
    comments: Vec<Comment>,
    allow_block_call: bool,
}

impl Parser {
    fn new(tokens: Vec<Token>, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            tokens,
            cursor: 0,
            diagnostics,
            comments: Vec::new(),
            allow_block_call: true,
        }
    }

    fn parse_source_file(&mut self) -> ParseOutput {
        let start = self.current().span.start;
        let mut items = Vec::new();
        self.consume_separators_as_items(&mut items);
        while !self.at(TokenKind::Eof) {
            let mut item = self.parse_item_or_recover();
            item.trailing_comments
                .extend(self.consume_trailing_comments());
            items.push(item);
            self.consume_separators_as_items(&mut items);
        }
        let end = self.current().span.end;
        ParseOutput {
            file: SourceFile {
                items,
                comments: std::mem::take(&mut self.comments),
                span: Span::new(start, end),
            },
            diagnostics: std::mem::take(&mut self.diagnostics),
        }
    }

    fn parse_item_or_recover(&mut self) -> RawItem {
        let leading_comments = self.take_leading_comments();
        if self.at_keyword(Keyword::Method) {
            return self.parse_method(leading_comments);
        }
        let statement = self.parse_statement();
        RawItem {
            span: statement.span,
            kind: RawItemKind::Statement(statement),
            leading_comments,
            trailing_comments: Vec::new(),
        }
    }

    fn parse_method(&mut self, leading_comments: Vec<CommentId>) -> RawItem {
        let start = self
            .expect(TokenKind::Keyword(Keyword::Method), "expected method")
            .span;
        let name = self.expect_ident("expected method name");
        self.expect(TokenKind::LParen, "expected '(' after method name");
        let mut params = Vec::new();
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            let param_start = self.current().span;
            let name = self.expect_ident("expected parameter name");
            let default = if self.eat(TokenKind::Eq).is_some() {
                Some(self.parse_expr())
            } else {
                None
            };
            let end = default
                .as_ref()
                .map_or(param_start.end, |expr| expr.span.end);
            params.push(RawParam {
                name,
                default,
                span: Span::new(param_start.start, end),
            });
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(TokenKind::RParen, "expected ')' after method parameters");
        let body = self.parse_block();
        let span = start.join(body.span);
        RawItem {
            span,
            kind: RawItemKind::Method(RawMethod { name, params, body }),
            leading_comments,
            trailing_comments: Vec::new(),
        }
    }

    fn parse_block(&mut self) -> RawBlock {
        let open = self.expect(TokenKind::LBrace, "expected '{' before block");
        let mut items = Vec::new();
        self.consume_separators_as_items(&mut items);
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let mut item = self.parse_item_or_recover();
            item.trailing_comments
                .extend(self.consume_trailing_comments());
            items.push(item);
            self.consume_separators_as_items(&mut items);
        }
        let close = self.expect(TokenKind::RBrace, "expected '}' after block");
        RawBlock {
            items,
            span: open.span.join(close.span),
        }
    }
    fn current_binary_op(&self) -> Option<(BinaryOp, u8, u8)> {
        let op = match self.current().kind {
            TokenKind::Star => (BinaryOp::Multiply, 10, 11),
            TokenKind::Slash => (BinaryOp::Divide, 10, 11),
            TokenKind::Percent => (BinaryOp::Remainder, 10, 11),
            TokenKind::Plus => (BinaryOp::Add, 9, 10),
            TokenKind::Minus => (BinaryOp::Subtract, 9, 10),
            TokenKind::Less => (BinaryOp::Less, 8, 9),
            TokenKind::LessEq => (BinaryOp::LessEq, 8, 9),
            TokenKind::Greater => (BinaryOp::Greater, 8, 9),
            TokenKind::GreaterEq => (BinaryOp::GreaterEq, 8, 9),
            TokenKind::Keyword(Keyword::In) => (BinaryOp::In, 8, 9),
            TokenKind::EqEq => (BinaryOp::Eq, 7, 8),
            TokenKind::BangEq => (BinaryOp::NotEq, 7, 8),
            TokenKind::AndAnd => (BinaryOp::And, 6, 7),
            TokenKind::OrOr => (BinaryOp::Or, 5, 6),
            TokenKind::QuestionQuestion => (BinaryOp::NullCoalesce, 4, 5),
            _ => return None,
        };
        Some(op)
    }

    fn consume_separators_as_items(&mut self, items: &mut Vec<RawItem>) {
        while self.at(TokenKind::Newline) || self.at(TokenKind::Semicolon) {
            let token = self.bump();
            for comment in self.comments_from_token(&token) {
                items.push(RawItem {
                    span: self.comments[comment].span,
                    kind: RawItemKind::Comment(comment),
                    leading_comments: Vec::new(),
                    trailing_comments: Vec::new(),
                });
            }
        }
    }

    fn consume_trailing_comments(&mut self) -> Vec<CommentId> {
        let mut comments = Vec::new();
        while self.at(TokenKind::Newline) || self.at(TokenKind::Semicolon) {
            let token = self.bump();
            comments.extend(self.comments_from_token(&token));
            if token.kind == TokenKind::Newline {
                break;
            }
        }
        comments
    }

    fn consume_soft_separators(&mut self) {
        while self.at(TokenKind::Newline) || self.at(TokenKind::Semicolon) {
            self.bump();
        }
    }

    fn take_leading_comments(&mut self) -> Vec<CommentId> {
        let token = self.current().clone();
        self.comments_from_token(&token)
    }

    fn comments_from_token(&mut self, token: &Token) -> Vec<CommentId> {
        let mut ids = Vec::new();
        for trivia in token.leading_comments() {
            let kind = match trivia.kind {
                TriviaKind::LineComment => CommentKind::Line,
                TriviaKind::BlockComment => CommentKind::Block,
                TriviaKind::Whitespace => continue,
            };
            let id = self.comments.len();
            self.comments.push(Comment {
                span: trivia.span,
                text: trivia.text.clone(),
                kind,
            });
            ids.push(id);
        }
        ids
    }

    fn recover_until_expr_boundary(&mut self) {
        while !self.at(TokenKind::Eof)
            && !self.at(TokenKind::Newline)
            && !self.at(TokenKind::Semicolon)
            && !self.at(TokenKind::Comma)
            && !self.at(TokenKind::RParen)
            && !self.at(TokenKind::RBracket)
            && !self.at(TokenKind::RBrace)
        {
            self.bump();
        }
    }

    fn at_statement_boundary(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Eof | TokenKind::Newline | TokenKind::Semicolon | TokenKind::RBrace
        )
    }

    fn at_process_boundary(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Eof
                | TokenKind::Newline
                | TokenKind::Semicolon
                | TokenKind::RBrace
                | TokenKind::ShiftRight
                | TokenKind::ShiftLeft
        )
    }

    fn expect_ident(&mut self, message: &str) -> String {
        if self.at(TokenKind::Ident) {
            self.bump().text
        } else {
            let token = self.current().clone();
            self.diagnostics.push(Diagnostic::new(token.span, message));
            String::new()
        }
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Token {
        if self.current().kind == kind {
            self.bump()
        } else {
            let token = self.current().clone();
            self.diagnostics.push(Diagnostic::new(token.span, message));
            Token {
                kind,
                text: String::new(),
                span: Span::empty(token.span.start),
                leading: Vec::new(),
            }
        }
    }

    fn eat(&mut self, kind: TokenKind) -> Option<Token> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        self.current().kind == TokenKind::Keyword(keyword)
    }

    fn current(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.cursor.saturating_sub(1)]
    }

    fn peek_kind(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens
            .get(self.cursor + offset)
            .map(|token| &token.kind)
    }

    fn bump(&mut self) -> Token {
        let token = self.current().clone();
        if token.kind != TokenKind::Eof {
            self.cursor += 1;
        }
        token
    }
}
