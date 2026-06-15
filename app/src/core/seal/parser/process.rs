use super::*;

impl Parser {
    pub(super) fn parse_process(&mut self) -> RawExpr {
        let start = self
            .expect(TokenKind::Pipe, "expected process marker '|'")
            .span;
        if !self.at_process_boundary() && !self.current().has_leading_whitespace() {
            self.diagnostics.push(Diagnostic::new(
                self.current().span,
                "expected whitespace after process marker '|'",
            ));
        }
        let mut args = Vec::new();
        while !self.at_process_boundary() {
            if self.current().has_leading_whitespace() && !args.is_empty() {
                // Whitespace separates argv atoms; the next loop iteration parses
                // the next argument from the current token.
            }
            args.push(self.parse_process_arg());
        }
        let end = args.last().map_or(start, |arg| arg.span);
        if args.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                start,
                "expected command word after process marker '|'",
            ));
        }
        let mut iter = args.into_iter();
        let program = iter.next();
        RawExpr {
            span: start.join(end),
            kind: RawExprKind::Process(RawProcess {
                program,
                args: iter.collect(),
            }),
        }
    }

    pub(super) fn parse_process_arg(&mut self) -> RawProcessArg {
        let token = self.current().clone();
        match token.kind {
            TokenKind::Star => self.parse_process_spread(),
            TokenKind::String => {
                self.bump();
                RawProcessArg {
                    span: token.span,
                    kind: RawProcessArgKind::String(token.text),
                }
            }
            TokenKind::TextBlock => {
                self.bump();
                RawProcessArg {
                    span: token.span,
                    kind: RawProcessArgKind::TextBlock(token.text),
                }
            }
            _ => self.parse_process_word(),
        }
    }

    pub(super) fn parse_process_spread(&mut self) -> RawProcessArg {
        let star = self.expect(TokenKind::Star, "expected '*'");
        let expr = if self.at(TokenKind::LBrace) {
            self.bump();
            let expr = self.parse_expr();
            self.expect(TokenKind::RBrace, "expected '}' after process spread");
            expr
        } else {
            self.parse_postfix_expr()
        };
        RawProcessArg {
            span: star.span.join(expr.span),
            kind: RawProcessArgKind::Spread(Box::new(expr)),
        }
    }

    pub(super) fn parse_process_word(&mut self) -> RawProcessArg {
        let start = self.current().span;
        let mut span = start;
        let mut parts = Vec::new();
        let mut text = String::new();
        let mut first = true;

        while !self.at_process_boundary() {
            if !first && self.current().has_leading_whitespace() {
                break;
            }
            let token = self.current().clone();
            match token.kind {
                TokenKind::String
                | TokenKind::TextBlock
                | TokenKind::LParen
                | TokenKind::RParen
                | TokenKind::LBracket
                | TokenKind::RBracket => break,
                TokenKind::LBrace => {
                    if !text.is_empty() {
                        parts.push(RawProcessPart::Text(std::mem::take(&mut text)));
                    }
                    self.bump();
                    let expr = self.parse_expr();
                    self.expect(TokenKind::RBrace, "expected '}' after argv interpolation");
                    span = span.join(expr.span);
                    parts.push(RawProcessPart::Interpolation(expr));
                }
                _ => {
                    text.push_str(&token.text);
                    span = span.join(token.span);
                    self.bump();
                }
            }
            first = false;
        }

        if !text.is_empty() {
            parts.push(RawProcessPart::Text(text));
        }
        if parts.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                self.current().span,
                "expected process argv word",
            ));
            if !self.at_process_boundary() {
                let bad = self.bump();
                return RawProcessArg {
                    span: bad.span,
                    kind: RawProcessArgKind::Error,
                };
            }
        }

        RawProcessArg {
            span,
            kind: RawProcessArgKind::Word(parts),
        }
    }
}
