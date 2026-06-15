use super::*;

impl Parser {
    pub(super) fn parse_expr(&mut self) -> RawExpr {
        self.parse_expr_bp(0)
    }

    pub(super) fn parse_expr_no_block(&mut self) -> RawExpr {
        let previous = self.allow_block_call;
        self.allow_block_call = false;
        let expr = self.parse_expr();
        self.allow_block_call = previous;
        expr
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> RawExpr {
        let mut left = self.parse_prefix_expr();
        while let Some((op, left_bp, right_bp)) = self.current_binary_op() {
            if left_bp < min_bp {
                break;
            }
            self.bump();
            let right = self.parse_expr_bp(right_bp);
            let span = left.span.join(right.span);
            left = RawExpr {
                span,
                kind: RawExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            };
        }
        left
    }

    fn parse_prefix_expr(&mut self) -> RawExpr {
        if self.at(TokenKind::Bang) || self.at(TokenKind::Minus) {
            let token = self.bump();
            let op = if token.kind == TokenKind::Bang {
                UnaryOp::Not
            } else {
                UnaryOp::Neg
            };
            let expr = self.parse_expr_bp(11);
            return RawExpr {
                span: token.span.join(expr.span),
                kind: RawExprKind::Unary {
                    op,
                    expr: Box::new(expr),
                },
            };
        }
        self.parse_postfix_expr()
    }

    pub(super) fn parse_postfix_expr(&mut self) -> RawExpr {
        let mut expr = self.parse_primary_expr();
        loop {
            if !self.at(TokenKind::Dot) {
                let checkpoint = self.cursor;
                while self.at(TokenKind::Newline) {
                    self.bump();
                }
                if !self.at(TokenKind::Dot) {
                    self.cursor = checkpoint;
                }
            }
            if self.at(TokenKind::LParen) {
                let args = self.parse_call_args();
                let span = expr
                    .span
                    .join(args.last().map_or(self.previous().span, |arg| arg.span));
                expr = RawExpr {
                    span,
                    kind: RawExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                };
                continue;
            }
            if self.allow_block_call && self.at(TokenKind::LBrace) {
                let block = self.parse_block();
                let span = expr.span.join(block.span);
                expr = RawExpr {
                    span,
                    kind: RawExprKind::BlockCall {
                        callee: Box::new(expr),
                        block,
                    },
                };
                continue;
            }
            if self.at(TokenKind::Dot) && self.peek_kind(1) == Some(&TokenKind::Ident) {
                self.bump();
                let method = self.bump();
                if self.at(TokenKind::LParen) {
                    let args = self.parse_call_args();
                    let span = expr
                        .span
                        .join(args.last().map_or(method.span, |arg| arg.span));
                    expr = RawExpr {
                        span,
                        kind: RawExprKind::ReceiverCall {
                            receiver: Box::new(expr),
                            method: method.text,
                            args,
                        },
                    };
                } else {
                    let span = expr.span.join(method.span);
                    expr = RawExpr {
                        span,
                        kind: RawExprKind::ReceiverCall {
                            receiver: Box::new(expr),
                            method: method.text,
                            args: Vec::new(),
                        },
                    };
                }
                continue;
            }
            break;
        }
        expr
    }

    fn parse_primary_expr(&mut self) -> RawExpr {
        let token = self.current().clone();
        match token.kind {
            TokenKind::Ident => {
                self.bump();
                RawExpr {
                    span: token.span,
                    kind: RawExprKind::Ident(token.text),
                }
            }
            TokenKind::Number => {
                self.bump();
                RawExpr {
                    span: token.span,
                    kind: RawExprKind::Literal(RawLiteral::Int(token.text)),
                }
            }
            TokenKind::String => {
                self.bump();
                RawExpr {
                    span: token.span,
                    kind: RawExprKind::Literal(RawLiteral::String(token.text)),
                }
            }
            TokenKind::TextBlock => {
                self.bump();
                RawExpr {
                    span: token.span,
                    kind: RawExprKind::Literal(RawLiteral::TextBlock(token.text)),
                }
            }
            TokenKind::Keyword(Keyword::True) | TokenKind::Keyword(Keyword::False) => {
                self.bump();
                RawExpr {
                    span: token.span,
                    kind: RawExprKind::Literal(RawLiteral::Bool(
                        token.kind == TokenKind::Keyword(Keyword::True),
                    )),
                }
            }
            TokenKind::Keyword(Keyword::Null) => {
                self.bump();
                RawExpr {
                    span: token.span,
                    kind: RawExprKind::Literal(RawLiteral::Null),
                }
            }
            TokenKind::Keyword(Keyword::Match) => self.parse_match_expr(),
            TokenKind::At => self.parse_at_name(),
            TokenKind::Dollar => self.parse_prefixed_name(TokenKind::Dollar),
            TokenKind::Hash => self.parse_prefixed_name(TokenKind::Hash),
            TokenKind::Pipe => self.parse_process(),
            TokenKind::LParen if self.at_lambda_start() => self.parse_lambda(),
            TokenKind::LParen => self.parse_group(),
            TokenKind::LBracket => self.parse_array(),
            TokenKind::LBrace => self.parse_map(),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    token.span,
                    format!("expected expression, found {:?}", token.kind),
                ));
                if !self.at_statement_boundary() {
                    self.bump();
                }
                RawExpr::error(token.span)
            }
        }
    }

    fn parse_at_name(&mut self) -> RawExpr {
        let start = self.expect(TokenKind::At, "expected '@'").span;
        let mut parts = Vec::new();
        parts.push(self.expect_ident("expected identifier after '@'"));
        while self.eat(TokenKind::Dot).is_some() {
            parts.push(self.expect_ident("expected identifier after '.'"));
        }
        let end = self.previous().span;
        RawExpr {
            span: start.join(end),
            kind: RawExprKind::AtName(parts),
        }
    }

    fn parse_prefixed_name(&mut self, prefix: TokenKind) -> RawExpr {
        let start = self.expect(prefix.clone(), "expected prefix").span;
        let name = self.expect_ident("expected identifier after prefix");
        let span = start.join(self.previous().span);
        let kind = if prefix == TokenKind::Dollar {
            RawExprKind::Env(name)
        } else {
            RawExprKind::Channel(name)
        };
        RawExpr { span, kind }
    }

    fn parse_group(&mut self) -> RawExpr {
        let open = self.expect(TokenKind::LParen, "expected '('").span;
        self.consume_soft_separators();
        let expr = self.parse_expr();
        self.consume_soft_separators();
        let close = self.expect(TokenKind::RParen, "expected ')' after expression");
        RawExpr {
            span: open.join(close.span),
            kind: RawExprKind::Group(Box::new(expr)),
        }
    }

    fn at_lambda_start(&self) -> bool {
        if !self.at(TokenKind::LParen) {
            return false;
        }

        let mut cursor = self.cursor + 1;
        let mut depth = 1usize;
        while let Some(token) = self.tokens.get(cursor) {
            match token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        cursor += 1;
                        while matches!(
                            self.tokens.get(cursor).map(|token| &token.kind),
                            Some(TokenKind::Newline | TokenKind::Semicolon)
                        ) {
                            cursor += 1;
                        }
                        return matches!(
                            self.tokens.get(cursor).map(|token| &token.kind),
                            Some(TokenKind::FatArrow)
                        );
                    }
                }
                TokenKind::Eof => return false,
                _ => {}
            }
            cursor += 1;
        }
        false
    }

    fn parse_lambda(&mut self) -> RawExpr {
        let open = self.expect(TokenKind::LParen, "expected '(' before lambda parameters");
        let mut params = Vec::new();
        self.consume_soft_separators();
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            let param_start = self.current().span;
            let name = self.expect_ident("expected lambda parameter name");
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
            self.consume_soft_separators();
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
            self.consume_soft_separators();
        }
        self.expect(TokenKind::RParen, "expected ')' after lambda parameters");
        self.consume_soft_separators();
        self.expect(TokenKind::FatArrow, "expected '=>' after lambda parameters");
        let body = self.parse_block();
        RawExpr {
            span: open.span.join(body.span),
            kind: RawExprKind::Lambda(RawLambda { params, body }),
        }
    }

    fn parse_array(&mut self) -> RawExpr {
        let open = self.expect(TokenKind::LBracket, "expected '['").span;
        let mut items = Vec::new();
        self.consume_soft_separators();
        while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
            items.push(self.parse_expr());
            self.consume_soft_separators();
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
            self.consume_soft_separators();
        }
        let close = self.expect(TokenKind::RBracket, "expected ']' after array");
        RawExpr {
            span: open.join(close.span),
            kind: RawExprKind::Array(items),
        }
    }

    fn parse_map(&mut self) -> RawExpr {
        let open = self.expect(TokenKind::LBrace, "expected '{'").span;
        let mut entries = Vec::new();
        self.consume_soft_separators();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let key_token = self.current().clone();
            let key = match key_token.kind {
                TokenKind::Ident | TokenKind::String => {
                    self.bump();
                    key_token.text
                }
                _ => {
                    self.diagnostics
                        .push(Diagnostic::new(key_token.span, "expected map key"));
                    self.recover_until_expr_boundary();
                    break;
                }
            };
            self.expect(TokenKind::Colon, "expected ':' after map key");
            let value = self.parse_expr();
            let span = key_token.span.join(value.span);
            entries.push(RawMapEntry { key, value, span });
            self.consume_soft_separators();
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
            self.consume_soft_separators();
        }
        let close = self.expect(TokenKind::RBrace, "expected '}' after map");
        RawExpr {
            span: open.join(close.span),
            kind: RawExprKind::Map(entries),
        }
    }

    fn parse_call_args(&mut self) -> Vec<RawArg> {
        self.expect(TokenKind::LParen, "expected '(' before arguments");
        let mut args = Vec::new();
        self.consume_soft_separators();
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            let start = self.current().span;
            let label = if self.at(TokenKind::Ident) && self.peek_kind(1) == Some(&TokenKind::Colon)
            {
                let label = self.bump().text;
                self.bump();
                Some(label)
            } else {
                None
            };
            let value = self.parse_expr();
            args.push(RawArg {
                label,
                span: start.join(value.span),
                value,
            });
            self.consume_soft_separators();
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
            self.consume_soft_separators();
        }
        self.expect(TokenKind::RParen, "expected ')' after arguments");
        args
    }

    fn parse_match_expr(&mut self) -> RawExpr {
        let start = self
            .expect(TokenKind::Keyword(Keyword::Match), "expected match")
            .span;
        let scrutinee = self.parse_expr_no_block();
        self.expect(TokenKind::LBrace, "expected '{' before match arms");
        let mut arms = Vec::new();
        self.consume_soft_separators();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            arms.push(self.parse_match_arm());
            self.consume_soft_separators();
            self.eat(TokenKind::Comma);
            self.consume_soft_separators();
        }
        let close = self.expect(TokenKind::RBrace, "expected '}' after match");
        RawExpr {
            span: start.join(close.span),
            kind: RawExprKind::Match(RawMatch {
                scrutinee: Box::new(scrutinee),
                arms,
            }),
        }
    }

    fn parse_match_arm(&mut self) -> RawMatchArm {
        let start = self.current().span;
        let mut patterns = vec![self.parse_pattern()];
        while self.at(TokenKind::Pipe) {
            self.bump();
            patterns.push(self.parse_pattern());
        }
        self.expect(TokenKind::FatArrow, "expected '=>' after match pattern");
        let body = if self.at(TokenKind::LBrace) && !self.brace_starts_map() {
            RawMatchArmBody::Block(self.parse_block())
        } else {
            RawMatchArmBody::Expr(self.parse_stream_expr())
        };
        RawMatchArm {
            span: start.join(body.span()),
            patterns,
            body,
        }
    }

    fn brace_starts_map(&self) -> bool {
        if !self.at(TokenKind::LBrace) {
            return false;
        }

        let Some(first) = self.next_significant_index(self.cursor + 1) else {
            return false;
        };
        if !matches!(
            self.tokens[first].kind,
            TokenKind::Ident | TokenKind::String
        ) {
            return false;
        }

        matches!(
            self.next_significant_index(first + 1)
                .map(|index| &self.tokens[index].kind),
            Some(TokenKind::Colon)
        )
    }

    fn next_significant_index(&self, mut cursor: usize) -> Option<usize> {
        while matches!(
            self.tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Newline | TokenKind::Semicolon)
        ) {
            cursor += 1;
        }
        self.tokens.get(cursor).map(|_| cursor)
    }

    fn parse_pattern(&mut self) -> RawPattern {
        if self.at(TokenKind::Underscore) {
            let token = self.bump();
            return RawPattern {
                span: token.span,
                kind: RawPatternKind::Wildcard,
            };
        }
        let expr = self.parse_expr_no_block();
        RawPattern {
            span: expr.span,
            kind: RawPatternKind::Expr(expr),
        }
    }
}
