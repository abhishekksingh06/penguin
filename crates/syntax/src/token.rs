use core::fmt;

use ginto_diag::Spanned;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TokenKind {
    // Literals
    IntLiteral(u64),
    BoolLiteral(bool),

    // Identifiers
    Ident(String),

    // Keywords
    Let,
    Mod,
    Not,
    Fn,
    U64,
    I64,

    // Operators
    Plus,  // +
    Minus, // -
    Star,  // *
    Slash, // /

    Equal,        // =
    NotEqual,     // <>
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    And, // &&
    Or,  // ||

    // Delimiters
    LParen, // (
    RParen, // )

    Comma, // ,
    Dot,   // .
    Colon, // :

    // end of file
    Eof,
    Arrow,

    // layout
    Indent,
    Dedent,
    Newline,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::IntLiteral(_) => write!(f, "integer literal"),
            TokenKind::BoolLiteral(_) => write!(f, "boolean literal"),
            TokenKind::Ident(_) => write!(f, "identifier"),
            TokenKind::Let => write!(f, "`let`"),
            TokenKind::Mod => write!(f, "`mod`"),
            TokenKind::Not => write!(f, "`not`"),
            TokenKind::Plus => write!(f, "`+`"),
            TokenKind::Minus => write!(f, "`-`"),
            TokenKind::Star => write!(f, "`*`"),
            TokenKind::Slash => write!(f, "`/`"),
            TokenKind::Equal => write!(f, "`=`"),
            TokenKind::NotEqual => write!(f, "`<>`"),
            TokenKind::Less => write!(f, "`<`"),
            TokenKind::LessEqual => write!(f, "`<=`"),
            TokenKind::Greater => write!(f, "`>`"),
            TokenKind::GreaterEqual => write!(f, "`>=`"),
            TokenKind::And => write!(f, "`&&`"),
            TokenKind::Or => write!(f, "`||`"),
            TokenKind::LParen => write!(f, "`(`"),
            TokenKind::RParen => write!(f, "`)`"),
            TokenKind::Comma => write!(f, "`,`"),
            TokenKind::Dot => write!(f, "`.`"),
            TokenKind::Colon => write!(f, "`:`"),
            TokenKind::Newline => write!(f, "newline"),
            TokenKind::Eof => write!(f, "end of file"),
            TokenKind::Fn => write!(f, "`fn`"),
            TokenKind::Indent => write!(f, "`indent`"),
            TokenKind::Dedent => write!(f, "dedent"),
            TokenKind::Arrow => write!(f, "`->`"),
            TokenKind::U64 => write!(f, "u64"),
            TokenKind::I64 => write!(f, "i64"),
        }
    }
}

pub type Token = Spanned<TokenKind>;
