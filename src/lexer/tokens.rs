#![allow(dead_code)]
use crate::utils::Span;

/// All token types in the Viper language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Ident(String),

    // Keywords
    Def,
    If,
    Else,
    Elif,
    While,
    For,
    In,
    Return,
    Pass,
    Break,
    Continue,
    True,
    False,
    Mut,
    Sync,
    Task,
    Try,
    Except,
    Finally,
    As,
    Class,
    Import,
    From,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    DoubleStar,
    DoubleSlash,
    Eq,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    Not,

    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Dot,
    Arrow,
    At,

    // Indentation (Python-style)
    Indent,
    Dedent,
    Newline,

    // Special
    Eof,
    Error(String),
}

/// A token with its source location
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn eof(span: Span) -> Self {
        Self { kind: TokenKind::Eof, span }
    }

    pub fn error(msg: String, span: Span) -> Self {
        Self { kind: TokenKind::Error(msg), span }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Int(n) => write!(f, "Int({})", n),
            TokenKind::Float(n) => write!(f, "Float({})", n),
            TokenKind::Str(s) => write!(f, "Str({})", s),
            TokenKind::Bool(b) => write!(f, "Bool({})", b),
            TokenKind::Ident(s) => write!(f, "Ident({})", s),
            TokenKind::Def => write!(f, "def"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Elif => write!(f, "elif"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Pass => write!(f, "pass"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::True => write!(f, "True"),
            TokenKind::False => write!(f, "False"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Sync => write!(f, "sync"),
            TokenKind::Task => write!(f, "task"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Except => write!(f, "except"),
            TokenKind::Finally => write!(f, "finally"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::From => write!(f, "from"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::DoubleStar => write!(f, "**"),
            TokenKind::DoubleSlash => write!(f, "//"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Indent => write!(f, "<INDENT>"),
            TokenKind::Dedent => write!(f, "<DEDENT>"),
            TokenKind::Newline => write!(f, "<NEWLINE>"),
            TokenKind::Eof => write!(f, "<EOF>"),
            TokenKind::Error(msg) => write!(f, "<ERROR: {}>", msg),
        }
    }
}
