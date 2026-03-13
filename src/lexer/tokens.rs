#![allow(dead_code)]
use crate::utils::Span;

/// All token types in the Viper language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Int(i128),
    BigInt(String), // BigInt literal (e.g., 123n or large integers)
    Float(f64),
    Str(String),
    FString(String),
    Bytes(Vec<u8>), // Byte literal b"bytes"
    Bool(bool),
    Ident(String),

    // Decorator
    At, // @

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
    None,
    Void,
    Sync,
    Task,
    Try,
    Except,
    Finally,
    As,
    Class,
    Import,
    From,
    Async,
    Await,
    Struct,
    Extern,
    Match,
    Case,
    Underscore,
    Select,
    Recv,
    Send,
    Unless,
    Pipeline,
    DotDot,
    Global,
    Nonlocal, // nonlocal keyword for closures
    Const,
    Type,     // type keyword for type aliases
    Tuple,    // tuple keyword for tuple types
    Optional, // Optional keyword for optional types
    Result,   // Result keyword for result types
    Assert,   // assert keyword for assertions
    Del,      // del keyword for deletion
    Raise,    // raise keyword for exceptions
    With,     // with keyword for context managers
    Yield,    // yield keyword for generators
    Super,    // super() builtin

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    DoubleStar,
    DoubleSlash,
    PlusPlus,      // ++
    MinusMinus,    // --
    PlusEq,        // +=
    MinusEq,       // -=
    StarEq,        // *=
    SlashEq,       // /=
    PercentEq,     // %=
    DoubleStarEq,  // **=
    DoubleSlashEq, // //=
    Eq,
    EqEq,
    NotEq,
    ColonEq, // := (walrus operator)
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    Not,
    // Bitwise operators (Phase 2)
    Ampersand,   // &
    Pipe,        // |
    Caret,       // ^
    Tilde,       // ~
    LtLt,        // <<
    GtGt,        // >>
    AmpersandEq, // &=
    PipeEq,      // |=
    CaretEq,     // ^=
    LtLtEq,      // <<=
    GtGtEq,      // >>=
    // Identity and Membership (Phase 2)
    Is,
    IsNot,
    NotIn,
    // Ternary (Phase 2)
    Question,       // ?
    DoubleQuestion, // ?? - Null coalescing (Phase 3)
    Lambda,
    Fn, // fn as alternative to lambda

    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Semi, // ;
    Dot,
    Arrow,

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
            TokenKind::BigInt(s) => write!(f, "BigInt({})", s),
            TokenKind::Float(n) => write!(f, "Float({})", n),
            TokenKind::Str(s) => write!(f, "Str({})", s),
            TokenKind::FString(s) => write!(f, "FString({})", s),
            TokenKind::Bytes(b) => write!(f, "Bytes({:?})", String::from_utf8_lossy(b)),
            TokenKind::Bool(b) => write!(f, "Bool({})", b),
            TokenKind::Ident(s) => write!(f, "Ident({})", s),
            TokenKind::At => write!(f, "@"),
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
            TokenKind::None => write!(f, "None"),
            TokenKind::Void => write!(f, "void"),
            TokenKind::Sync => write!(f, "sync"),
            TokenKind::Task => write!(f, "task"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Except => write!(f, "except"),
            TokenKind::Finally => write!(f, "finally"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::From => write!(f, "from"),
            TokenKind::Async => write!(f, "async"),
            TokenKind::Await => write!(f, "await"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Extern => write!(f, "extern"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Case => write!(f, "case"),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::Select => write!(f, "select"),
            TokenKind::Recv => write!(f, "recv"),
            TokenKind::Send => write!(f, "send"),
            TokenKind::Unless => write!(f, "unless"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Pipeline => write!(f, "|>"),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::Global => write!(f, "global"),
            TokenKind::Nonlocal => write!(f, "nonlocal"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::DoubleStar => write!(f, "**"),
            TokenKind::DoubleSlash => write!(f, "//"),
            TokenKind::PlusPlus => write!(f, "++"),
            TokenKind::MinusMinus => write!(f, "--"),
            TokenKind::PlusEq => write!(f, "+="),
            TokenKind::MinusEq => write!(f, "-="),
            TokenKind::StarEq => write!(f, "*="),
            TokenKind::SlashEq => write!(f, "/="),
            TokenKind::PercentEq => write!(f, "%="),
            TokenKind::DoubleStarEq => write!(f, "**="),
            TokenKind::DoubleSlashEq => write!(f, "//="),
            TokenKind::Eq => write!(f, "="),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::ColonEq => write!(f, ":="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::LtLt => write!(f, "<<"),
            TokenKind::GtGt => write!(f, ">>"),
            TokenKind::AmpersandEq => write!(f, "&="),
            TokenKind::PipeEq => write!(f, "|="),
            TokenKind::CaretEq => write!(f, "^="),
            TokenKind::LtLtEq => write!(f, "<<="),
            TokenKind::GtGtEq => write!(f, ">>="),
            TokenKind::Is => write!(f, "is"),
            TokenKind::IsNot => write!(f, "is not"),
            TokenKind::NotIn => write!(f, "not in"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::DoubleQuestion => write!(f, "??"),
            TokenKind::Lambda => write!(f, "lambda"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Tuple => write!(f, "tuple"),
            TokenKind::Optional => write!(f, "Optional"),
            TokenKind::Result => write!(f, "Result"),
            TokenKind::Assert => write!(f, "assert"),
            TokenKind::Del => write!(f, "del"),
            TokenKind::Raise => write!(f, "raise"),
            TokenKind::With => write!(f, "with"),
            TokenKind::Yield => write!(f, "yield"),
            TokenKind::Super => write!(f, "super"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Indent => write!(f, "<INDENT>"),
            TokenKind::Dedent => write!(f, "<DEDENT>"),
            TokenKind::Newline => write!(f, "<NEWLINE>"),
            TokenKind::Eof => write!(f, "<EOF>"),
            TokenKind::Error(msg) => write!(f, "<ERROR: {}>", msg),
        }
    }
}
