use ginto_diag::{Diagnostic, DiagnosticConvertible, FileId, Label, Severity, Span};

use crate::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum LexerError {
    UnexpectedCharacter {
        ch: char,
        span: Span,
        file_id: FileId,
    },
    InvalidNumber {
        text: String,
        span: Span,
        file_id: FileId,
    },
    NumberTooLarge {
        span: Span,
        file_id: FileId,
    },
    InvalidIndentation {
        span: Span,
        file_id: FileId,
    },
}

impl DiagnosticConvertible for LexerError {
    fn into_diagnostic(self) -> Diagnostic {
        match self {
            LexerError::UnexpectedCharacter { ch, span, file_id } => {
                Diagnostic::new(Severity::Error)
                    .with_message(format!("unexpected character `{}`", ch))
                    .with_label(
                        Label::primary(file_id, span)
                            .with_message("this character is not valid here"),
                    )
                    .with_help("remove this character or replace it with a valid token")
            }

            LexerError::InvalidNumber {
                text,
                span,
                file_id,
            } => Diagnostic::new(Severity::Error)
                .with_message("invalid number literal")
                .with_label(
                    Label::primary(file_id, span)
                        .with_message(format!("`{}` is not a valid number", text)),
                )
                .with_help(
                    "check that the number uses valid digits and does not contain extra symbols",
                ),

            LexerError::NumberTooLarge { span, file_id } => Diagnostic::new(Severity::Error)
                .with_message("number literal is too large")
                .with_label(
                    Label::primary(file_id, span)
                        .with_message("this value does not fit in the target integer type"),
                )
                .with_help("try using a smaller value or a wider integer type if available"),

            LexerError::InvalidIndentation { span, file_id } => Diagnostic::new(Severity::Error)
                .with_message("invalid indentation")
                .with_label(
                    Label::primary(file_id, span)
                        .with_message("this indentation does not match any previous block level"),
                )
                .with_note("indentation must match the indentation of a previous block exactly")
                .with_help("align this line with a previous block or fix inconsistent spaces/tabs"),
        }
    }
}

#[derive(Clone)]
pub struct Lexer {
    pos: usize,
    file_id: FileId,
    input: Vec<char>,
    indent_stack: Vec<usize>,
    pending: Vec<TokenKind>,
}

impl Lexer {
    pub fn new(file_id: FileId, input: &str) -> Self {
        Self {
            pos: 0,
            file_id,
            input: input.chars().collect(),
            indent_stack: vec![0],
            pending: Vec::new(),
        }
    }

    fn current(&self) -> char {
        self.input.get(self.pos).copied().unwrap_or('\0')
    }

    fn peek(&self) -> char {
        self.input.get(self.pos + 1).copied().unwrap_or('\0')
    }

    fn advance(&mut self) {
        self.pos += 1
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.current(), ' ' | '\t' | '\r') {
            self.advance();
        }
    }

    fn lex_ident(&mut self) -> Result<String, LexerError> {
        let start = self.pos;
        while self.current().is_alphanumeric() || self.current() == '_' {
            self.advance();
        }
        Ok(self.input[start..self.pos].iter().collect())
    }

    fn lex_num(&mut self) -> Result<u64, LexerError> {
        let start = self.pos;
        while self.current().is_ascii_digit() {
            self.advance();
        }
        let text: String = self.input[start..self.pos].iter().collect();
        text.parse::<u64>().map_err(|_| {
            if text.chars().all(|c| c.is_ascii_digit()) {
                LexerError::NumberTooLarge {
                    file_id: self.file_id,
                    span: Span::from_range(start..self.pos),
                }
            } else {
                LexerError::InvalidNumber {
                    file_id: self.file_id,
                    text,
                    span: Span::from_range(start..self.pos),
                }
            }
        })
    }

    fn count_indent(&mut self) -> usize {
        let mut count = 0;
        while self.current() == ' ' {
            count += 1;
            self.advance();
        }
        count
    }

    fn handle_newline(&mut self) -> Result<TokenKind, LexerError> {
        self.advance();

        let indent = self.count_indent();
        let current = *self.indent_stack.last().unwrap();

        if indent > current {
            self.indent_stack.push(indent);
            self.pending.push(TokenKind::Indent);
        } else if indent < current {
            while let Some(&top) = self.indent_stack.last() {
                if top > indent {
                    self.indent_stack.pop();
                    self.pending.push(TokenKind::Dedent);
                } else {
                    break;
                }
            }

            if *self.indent_stack.last().unwrap() != indent {
                return Err(LexerError::InvalidIndentation {
                    span: Span::from_range(self.pos..self.pos),
                    file_id: self.file_id,
                });
            }
        }

        Ok(TokenKind::Newline)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();
        let pos = self.pos;
        let ch = self.current();

        let kind = match ch {
            '\0' => {
                self.advance();
                TokenKind::Eof
            }
            '\n' => self.handle_newline()?,
            '(' => {
                self.advance();
                TokenKind::LParen
            }
            ')' => {
                self.advance();
                TokenKind::RParen
            }
            '.' => {
                self.advance();
                TokenKind::Dot
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            ':' => {
                self.advance();
                TokenKind::Colon
            }
            '+' => {
                self.advance();
                TokenKind::Plus
            }
            '-' => {
                self.advance();
                if self.peek() == '>' {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                self.advance();
                TokenKind::Star
            }
            '/' => {
                self.advance();
                TokenKind::Slash
            }
            '=' => {
                self.advance();
                TokenKind::Equal
            }
            '<' => {
                self.advance();
                if self.peek() == '>' {
                    self.advance();
                    TokenKind::NotEqual
                } else if self.peek() == '=' {
                    self.advance();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '&' if self.peek() == '&' => {
                self.advance();
                self.advance();
                TokenKind::And
            }
            '|' if self.peek() == '|' => {
                self.advance();
                self.advance();
                TokenKind::Or
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let ident = self.lex_ident()?;
                match ident.as_str() {
                    "let" => TokenKind::Let,
                    "mod" => TokenKind::Mod,
                    "fn" => TokenKind::Fn,
                    "not" => TokenKind::Not,
                    "u64" => TokenKind::U64,
                    "i64" => TokenKind::I64,
                    "true" => TokenKind::BoolLiteral(true),
                    "false" => TokenKind::BoolLiteral(false),
                    _ => TokenKind::Ident(ident),
                }
            }
            '0'..='9' => self.lex_num().map(TokenKind::IntLiteral)?,
            _ => {
                return Err(LexerError::UnexpectedCharacter {
                    ch,
                    file_id: self.file_id,
                    span: Span::from_range(pos..pos),
                });
            }
        };

        Ok(Token {
            inner: kind,
            span: Span::from_range(pos..self.pos),
        })
    }

    pub fn lex_all(&mut self) -> Result<Vec<Token>, Vec<LexerError>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        loop {
            match self.next_token() {
                Ok(tok) => {
                    if tok.inner == TokenKind::Eof {
                        let eof_span = tok.span;
                        while self.indent_stack.len() > 1 {
                            self.indent_stack.pop();
                            tokens.push(Token {
                                inner: TokenKind::Dedent,
                                span: eof_span,
                            });
                        }
                        tokens.push(tok);
                        break;
                    }
                    tokens.push(tok);
                }
                Err(err) => {
                    errors.push(err);
                    self.advance();
                }
            }
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }
}
