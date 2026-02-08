use crate::{BinOp, Expr, ExprKind, Token, TokenKind, UnaryOp};
use ginto_diag::{Diagnostic, DiagnosticConvertible, FileId, Label, Severity, Span, Spanned};

#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    UnexpectedToken {
        expected: Vec<TokenKind>,
        found: TokenKind,
        span: Span,
        file_id: FileId,
    },
    UnexpectedEof {
        expected: Vec<TokenKind>,
        span: Span,
        file_id: FileId,
    },
    MissingExpression {
        span: Span,
        file_id: FileId,
    },
    InvalidSyntax {
        message: String,
        span: Span,
        file_id: FileId,
    },
}

impl DiagnosticConvertible for ParserError {
    fn into_diagnostic(self) -> Diagnostic {
        match self {
            ParserError::UnexpectedToken {
                expected,
                found,
                span,
                file_id,
            } => {
                let expected_str = if expected.is_empty() {
                    "something else".to_string()
                } else if expected.len() == 1 {
                    format!("`{:?}`", expected[0])
                } else {
                    format!(
                        "one of {}",
                        expected
                            .iter()
                            .map(|k| format!("`{:?}`", k))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };

                Diagnostic::new(Severity::Error)
                    .with_message(format!("unexpected token `{:?}`", found))
                    .with_label(
                        Label::primary(file_id, span).with_message(format!(
                            "expected {}, found `{:?}`",
                            expected_str, found
                        )),
                    )
            }

            ParserError::UnexpectedEof {
                expected,
                span,
                file_id,
            } => {
                let expected_str = if expected.is_empty() {
                    "more input".to_string()
                } else {
                    expected
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                        .to_string()
                };

                Diagnostic::new(Severity::Error)
                    .with_message("unexpected end of file")
                    .with_label(
                        Label::primary(file_id, span)
                            .with_message(format!("expected {}", expected_str)),
                    )
            }

            ParserError::MissingExpression { span, file_id } => Diagnostic::new(Severity::Error)
                .with_message("expected expression")
                .with_label(Label::primary(file_id, span).with_message("expression expected here")),

            ParserError::InvalidSyntax {
                message,
                span,
                file_id,
            } => Diagnostic::new(Severity::Error)
                .with_message("invalid syntax")
                .with_label(Label::primary(file_id, span).with_message(message)),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    file_id: FileId,
    errors: Vec<ParserError>,
}

impl Parser {
    pub fn new(file_id: FileId, tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            file_id,
            errors: Vec::new(),
        }
    }

    fn current(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn peek(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.pos + offset)
            .unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn peek_is(&self, offset: usize, token: TokenKind) -> bool {
        self.peek(offset).inner == token
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.current().inner == TokenKind::Eof
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.current().inner == kind
    }

    fn check_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.iter().any(|k| self.check(k))
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.match_token(kind.clone()) {
                return true;
            }
        }
        false
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
        if self.check(&kind) {
            Ok(self.advance().clone())
        } else {
            let current = self.current();
            Err(ParserError::UnexpectedToken {
                expected: vec![kind],
                found: current.inner.clone(),
                span: current.span,
                file_id: self.file_id,
            })
        }
    }

    fn expect_with_recovery(
        &mut self,
        kind: TokenKind,
        sync_tokens: &[TokenKind],
    ) -> Option<Token> {
        match self.expect(kind) {
            Ok(tok) => Some(tok),
            Err(err) => {
                self.errors.push(err);
                self.synchronize(sync_tokens);
                None
            }
        }
    }

    fn synchronize(&mut self, sync_tokens: &[TokenKind]) {
        while !self.is_at_end() {
            if sync_tokens.iter().any(|k| self.check(k)) {
                return;
            }
            self.advance();
        }
    }

    fn synchronize_to_statement(&mut self) {
        self.synchronize(&[
            TokenKind::Let,
            TokenKind::Fn,
            TokenKind::Mod,
            TokenKind::Newline,
            TokenKind::Eof,
        ]);
    }

    fn synchronize_to_newline(&mut self) {
        self.synchronize(&[TokenKind::Newline, TokenKind::Eof]);
    }

    fn report_error(&mut self, error: ParserError) {
        self.errors.push(error);
    }

    fn error_and_recover<T>(&mut self, error: ParserError, sync_tokens: &[TokenKind]) -> Option<T> {
        self.report_error(error);
        self.synchronize(sync_tokens);
        None
    }

    fn skip_newlines(&mut self) {
        while self.match_token(TokenKind::Newline) {}
    }

    fn expect_newline_or_eof(&mut self) -> Result<(), ParserError> {
        if self.is_at_end() || self.check(&TokenKind::Newline) {
            if !self.is_at_end() {
                self.advance();
            }
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken {
                expected: vec![TokenKind::Newline, TokenKind::Eof],
                found: self.current().inner.clone(),
                span: self.current().span,
                file_id: self.file_id,
            })
        }
    }

    fn current_kind(&self) -> &TokenKind {
        &self.current().inner
    }

    pub fn errors(&self) -> &[ParserError] {
        &self.errors
    }

    pub fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_binary_expr(0)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.current_kind().clone() {
            TokenKind::IntLiteral(x) => {
                let span = self.advance().span;
                Some(Expr::new(ExprKind::Int(x), span))
            }
            TokenKind::BoolLiteral(v) => {
                let span = self.advance().span;
                Some(Expr::new(ExprKind::Bool(v), span))
            }
            TokenKind::Ident(v) => {
                let span = self.advance().span;
                Some(Expr::new(ExprKind::Var(v), span))
            }
            TokenKind::LParen => {
                let l_span = self.advance().span;
                if self.peek_is(1, TokenKind::RParen) {
                    let r_span = self.advance().span;
                    let span = l_span.merge(r_span);
                    Some(Expr::new(ExprKind::Unit, span))
                } else {
                    let Spanned { inner, span } = self.parse_expr()?;
                    let r_span = self.advance().span;
                    let span = l_span.merge(span).merge(r_span);
                    Some(Expr::new(inner, span))
                }
            }
            _ => {
                let current = self.current();
                self.report_error(ParserError::MissingExpression {
                    span: current.span,
                    file_id: self.file_id,
                });
                None
            }
        }
    }

    fn parse_unary_expr(&mut self) -> Option<Expr> {
        match self.current_kind() {
            TokenKind::Minus => {
                let start = self.advance().span;
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span);
                Some(Expr::new(
                    ExprKind::Unary {
                        op: Spanned::new(UnaryOp::Neg, start),
                        expr: Box::new(operand),
                    },
                    span,
                ))
            }
            TokenKind::Not => {
                let start = self.advance().span;
                let operand = self.parse_unary_expr()?;
                let span = start.merge(operand.span);
                Some(Expr::new(
                    ExprKind::Unary {
                        op: Spanned::new(UnaryOp::Not, start),
                        expr: Box::new(operand),
                    },
                    span,
                ))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_binary_expr(&mut self, min_bp: u8) -> Option<Expr> {
        let mut lhs = self.parse_unary_expr()?;
        loop {
            let op = match self.current_kind() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Greater => BinOp::Greater,
                TokenKind::GreaterEqual => BinOp::Ge,
                TokenKind::Less => BinOp::Less,
                TokenKind::LessEqual => BinOp::Le,
                TokenKind::Mod => BinOp::Mod,
                TokenKind::Equal => BinOp::Equal,
                TokenKind::NotEqual => BinOp::NotEq,
                TokenKind::And => BinOp::And,
                TokenKind::Or => BinOp::Or,
                _ => break,
            };
            let (left_bp, right_bp) = op.binding_power();
            if left_bp < min_bp {
                break;
            }
            let op_span = self.advance().span;
            let rhs = self.parse_binary_expr(right_bp)?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr::new(
                ExprKind::Binary {
                    op: Spanned::new(op, op_span),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            )
        }
        Some(lhs)
    }
}
