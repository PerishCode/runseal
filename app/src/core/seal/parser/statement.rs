use super::*;

impl Parser {
    pub(super) fn parse_statement(&mut self) -> RawStatement {
        if self.at_keyword(Keyword::Let) {
            return self.parse_let_statement();
        }
        if self.at_keyword(Keyword::If) {
            return self.parse_if_statement();
        }
        if self.at_keyword(Keyword::For) {
            return self.parse_for_statement();
        }
        if self.at_keyword(Keyword::While) {
            return self.parse_while_statement();
        }
        if self.at_keyword(Keyword::With) {
            return self.parse_with_env_statement();
        }
        if self.at_keyword(Keyword::Break) {
            let token = self.bump();
            return RawStatement {
                span: token.span,
                kind: RawStatementKind::Break,
            };
        }
        if self.at_keyword(Keyword::Continue) {
            let token = self.bump();
            return RawStatement {
                span: token.span,
                kind: RawStatementKind::Continue,
            };
        }

        let expr = self.parse_stream_expr();
        if self.at(TokenKind::Eq) {
            let eq = self.bump();
            let value = self.parse_stream_expr();
            let span = expr.span.join(value.span).join(eq.span);
            return RawStatement {
                span,
                kind: RawStatementKind::Assign {
                    target: expr,
                    value,
                },
            };
        }
        let kind = if matches!(
            expr.kind,
            RawExprKind::Process(_) | RawExprKind::StreamFlow { .. }
        ) {
            RawStatementKind::Effect(expr)
        } else {
            RawStatementKind::Expr(expr)
        };
        let span = match &kind {
            RawStatementKind::Effect(expr) | RawStatementKind::Expr(expr) => expr.span,
            _ => Span::empty(self.current().span.start),
        };
        RawStatement { kind, span }
    }

    fn parse_let_statement(&mut self) -> RawStatement {
        let start = self
            .expect(TokenKind::Keyword(Keyword::Let), "expected let")
            .span;
        let name = self.expect_ident("expected binding name");
        let binding = if self.eat(TokenKind::ColonEq).is_some() {
            LetBinding::Stream
        } else {
            self.expect(TokenKind::Eq, "expected '=' or ':=' after binding name");
            LetBinding::Value
        };
        let value = self.parse_stream_expr();
        let span = start.join(value.span);
        RawStatement {
            span,
            kind: RawStatementKind::Let {
                name,
                binding,
                value,
            },
        }
    }

    fn parse_if_statement(&mut self) -> RawStatement {
        let start = self
            .expect(TokenKind::Keyword(Keyword::If), "expected if")
            .span;
        let condition = self.parse_expr();
        let body = self.parse_block();
        let mut span = start.join(body.span);
        let mut branches = vec![RawIfBranch {
            span,
            condition,
            body,
        }];
        let mut else_branch = None;

        while self.at_keyword(Keyword::Else) {
            self.bump();
            if self.at_keyword(Keyword::If) {
                self.bump();
                let condition = self.parse_expr();
                let body = self.parse_block();
                span = span.join(body.span);
                branches.push(RawIfBranch {
                    span: condition.span.join(body.span),
                    condition,
                    body,
                });
            } else {
                let body = self.parse_block();
                span = span.join(body.span);
                else_branch = Some(body);
                break;
            }
        }

        RawStatement {
            span,
            kind: RawStatementKind::If {
                branches,
                else_branch,
            },
        }
    }

    fn parse_for_statement(&mut self) -> RawStatement {
        let start = self
            .expect(TokenKind::Keyword(Keyword::For), "expected for")
            .span;
        let binding = self.expect_ident("expected for binding name");
        self.expect(
            TokenKind::Keyword(Keyword::In),
            "expected 'in' after for binding",
        );
        let iterable = self.parse_expr();
        let body = self.parse_block();
        RawStatement {
            span: start.join(body.span),
            kind: RawStatementKind::For {
                binding,
                iterable,
                body,
            },
        }
    }

    fn parse_while_statement(&mut self) -> RawStatement {
        let start = self
            .expect(TokenKind::Keyword(Keyword::While), "expected while")
            .span;
        let condition = self.parse_expr();
        let body = self.parse_block();
        RawStatement {
            span: start.join(body.span),
            kind: RawStatementKind::While { condition, body },
        }
    }

    fn parse_with_env_statement(&mut self) -> RawStatement {
        let start = self
            .expect(TokenKind::Keyword(Keyword::With), "expected with")
            .span;
        self.expect(
            TokenKind::Keyword(Keyword::Env),
            "expected 'env' after with",
        );
        let bindings = self.parse_env_bindings();
        let body = self.parse_block();
        RawStatement {
            span: start.join(body.span),
            kind: RawStatementKind::WithEnv { bindings, body },
        }
    }

    fn parse_env_bindings(&mut self) -> Vec<RawEnvBinding> {
        self.expect(TokenKind::LBrace, "expected '{' before env bindings");
        let mut bindings = Vec::new();
        self.consume_soft_separators();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let start = self.current().span;
            let name = self.expect_ident("expected env binding name");
            self.expect(TokenKind::Eq, "expected '=' after env binding name");
            let value = self.parse_expr();
            bindings.push(RawEnvBinding {
                name,
                span: start.join(value.span),
                value,
            });
            self.consume_soft_separators();
            self.eat(TokenKind::Comma);
            self.consume_soft_separators();
        }
        self.expect(TokenKind::RBrace, "expected '}' after env bindings");
        bindings
    }

    fn parse_stream_expr(&mut self) -> RawExpr {
        let mut left = self.parse_effect_atom();
        while self.at(TokenKind::ShiftRight) || self.at(TokenKind::ShiftLeft) {
            let token = self.bump();
            let op = if token.kind == TokenKind::ShiftRight {
                StreamOp::To
            } else {
                StreamOp::From
            };
            self.consume_soft_separators();
            let right = self.parse_effect_atom();
            let span = left.span.join(right.span);
            left = RawExpr {
                span,
                kind: RawExprKind::StreamFlow {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            };
        }
        left
    }

    fn parse_effect_atom(&mut self) -> RawExpr {
        if self.at(TokenKind::Pipe) {
            return self.parse_process();
        }
        self.parse_expr()
    }
}
