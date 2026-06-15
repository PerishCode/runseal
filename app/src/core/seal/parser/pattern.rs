use super::*;

impl Parser {
    pub(super) fn parse_pattern(&mut self) -> RawPattern {
        if self.at(TokenKind::Underscore) {
            let token = self.bump();
            return RawPattern {
                span: token.span,
                kind: RawPatternKind::Wildcard,
            };
        }
        if self.at(TokenKind::LBrace) {
            return self.parse_map_pattern();
        }
        if self.at(TokenKind::LBracket) {
            return self.parse_array_pattern();
        }
        let expr = self.parse_expr_no_block();
        RawPattern {
            span: expr.span,
            kind: RawPatternKind::Expr(expr),
        }
    }

    fn parse_map_pattern(&mut self) -> RawPattern {
        let open = self.expect(TokenKind::LBrace, "expected '{' before map pattern");
        let mut entries = Vec::new();
        self.consume_soft_separators();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let key_token = self.current().clone();
            let key = self.expect_ident("expected map pattern key");
            self.expect(TokenKind::Colon, "expected ':' after map pattern key");
            let pattern = self.parse_pattern();
            entries.push(RawPatternEntry {
                key,
                span: key_token.span.join(pattern.span),
                pattern,
            });
            self.consume_soft_separators();
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
            self.consume_soft_separators();
        }
        let close = self.expect(TokenKind::RBrace, "expected '}' after map pattern");
        RawPattern {
            span: open.span.join(close.span),
            kind: RawPatternKind::Map(entries),
        }
    }

    fn parse_array_pattern(&mut self) -> RawPattern {
        let open = self.expect(TokenKind::LBracket, "expected '[' before array pattern");
        let mut items = Vec::new();
        self.consume_soft_separators();
        while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
            items.push(self.parse_pattern());
            self.consume_soft_separators();
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
            self.consume_soft_separators();
        }
        let close = self.expect(TokenKind::RBracket, "expected ']' after array pattern");
        RawPattern {
            span: open.span.join(close.span),
            kind: RawPatternKind::Array(items),
        }
    }
}
