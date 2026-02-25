#![allow(dead_code)]
use crate::ast::types::Type;
use crate::utils::Span;

/// Abstract Syntax Tree node for expressions
#[derive(Debug, Clone)]
pub enum Expr {
    /// Integer literal
    Int(i64, Span),
    /// Float literal
    Float(f64, Span),
    /// String literal
    Str(String, Span),
    /// FString literal
    FString(Vec<Expr>, Span),
    /// Boolean literal
    Bool(bool, Span),
    /// None literal
    None(Span),
    /// Identifier/variable reference
    Ident(String, Span),
    /// Binary operation
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        span: Span,
    },
    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    /// Function call
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    /// Index access (list[i])
    Index {
        obj: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    /// Attribute access (obj.attr)
    Attribute {
        obj: Box<Expr>,
        attr: String,
        span: Span,
    },
    /// List literal
    List { elements: Vec<Expr>, span: Span },
    /// Tuple literal
    Tuple { elements: Vec<Expr>, span: Span },
    /// Dictionary literal
    Dict {
        pairs: Vec<(Expr, Expr)>,
        span: Span,
    },
    /// Array literal (fixed-size): [value; size] or [elements...]
    Array {
        elements: Vec<Expr>,
        size: Option<usize>,
        span: Span,
    },
    /// Lambda expression
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
        span: Span,
    },
    /// List comprehension: [expr for var in iter]
    ListComprehension {
        element: Box<Expr>,
        var: String,
        iter: Box<Expr>,
        span: Span,
    },
    /// Conditional expression (a if cond else b)
    Conditional {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
        span: Span,
    },
    /// Await expression (await future)
    Await { future: Box<Expr>, span: Span },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, s) => *s,
            Expr::Float(_, s) => *s,
            Expr::Str(_, s) => *s,
            Expr::FString(_, s) => *s,
            Expr::Bool(_, s) => *s,
            Expr::None(s) => *s,
            Expr::Ident(_, s) => *s,
            Expr::BinOp { span, .. } => *span,
            Expr::UnaryOp { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::Attribute { span, .. } => *span,
            Expr::List { span, .. } => *span,
            Expr::Tuple { span, .. } => *span,
            Expr::Dict { span, .. } => *span,
            Expr::Array { span, .. } => *span,
            Expr::Lambda { span, .. } => *span,
            Expr::ListComprehension { span, .. } => *span,
            Expr::Conditional { span, .. } => *span,
            Expr::Await { span, .. } => *span,
        }
    }

    /// Try to get the identifier name if this is an Ident expression
    pub fn as_ident(&self) -> Option<&String> {
        match self {
            Expr::Ident(name, _) => Some(name),
            _ => None,
        }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    FloorDiv,
    Pow,
    // Comparison
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    // Logical
    And,
    Or,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    // Identity (Phase 2)
    Is,
    IsNot,
    // Membership (Phase 2)
    In,
    NotIn,
}

impl BinOp {
    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::Pow => 14,
            BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::FloorDiv => 13,
            BinOp::Add | BinOp::Sub => 12,
            BinOp::LShift | BinOp::RShift => 11,
            BinOp::BitAnd => 10,
            BinOp::BitXor => 9,
            BinOp::BitOr => 8,
            BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => 7,
            BinOp::Eq | BinOp::NotEq | BinOp::Is | BinOp::IsNot | BinOp::In | BinOp::NotIn => 6,
            BinOp::And => 5,
            BinOp::Or => 4,
        }
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, BinOp::Pow)
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
    Pos,
    Invert,
}

/// Abstract Syntax Tree node for statements
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Expression statement (expression as statement)
    Expr(Expr),
    /// Variable assignment: x = expr
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    /// Augmented assignment: x += expr
    AugAssign {
        target: Box<Expr>,
        op: BinOp,
        value: Box<Expr>,
        span: Span,
    },
    /// Variable declaration with type: x: i64 = expr
    Declare {
        name: String,
        type_ann: Option<Type>,
        value: Option<Expr>,
        mutable: bool,
        span: Span,
    },
    /// If statement
    If {
        condition: Expr,
        body: Vec<Stmt>,
        elif_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    /// While loop
    While {
        condition: Expr,
        body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    /// For loop
    For {
        target: Box<Expr>,
        iter: Box<Expr>,
        body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
        is_async: bool,
        span: Span,
    },
    /// Function definition
    Function {
        name: String,
        type_params: Vec<String>,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
        span: Span,
        is_async: bool,
    },
    /// External C function declaration: extern "C" fn name(args...) -> ret
    Extern {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        span: Span,
    },
    /// Return statement
    Return { value: Option<Expr>, span: Span },
    /// Break statement
    Break(Span),
    /// Continue statement
    Continue(Span),
    /// Pass statement
    Pass(Span),
    /// Import statement
    Import {
        module: String,
        alias: Option<String>,
        span: Span,
    },
    /// From import
    FromImport {
        module: String,
        names: Vec<(String, Option<String>)>,
        span: Span,
    },
    /// Class definition
    Class {
        name: String,
        bases: Vec<Expr>,
        body: Vec<Stmt>,
        span: Span,
    },
    /// Struct definition
    Struct {
        name: String,
        fields: Vec<(String, Type)>,
        span: Span,
    },
    /// Try-except block
    Try {
        body: Vec<Stmt>,
        handlers: Vec<ExceptHandler>,
        else_body: Option<Vec<Stmt>>,
        finally_body: Option<Vec<Stmt>>,
        span: Span,
    },
    /// Sync block (concurrency)
    Sync { body: Vec<Stmt>, span: Span },
    /// Task spawn
    Task { call: Expr, span: Span },
    /// Channel creation: chan(size)
    Chan { size: Expr, span: Span },
    /// Channel send: send(chan, value)
    Send {
        chan: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    /// Channel receive: recv(chan)
    Recv { chan: Box<Expr>, span: Span },
    /// WaitGroup creation
    WaitGroup { span: Span },
    /// WaitGroup add: add(wg, n)
    WgAdd {
        wg: Box<Expr>,
        n: Box<Expr>,
        span: Span,
    },
    /// WaitGroup done: done(wg)
    WgDone { wg: Box<Expr>, span: Span },
    /// WaitGroup wait: wait(wg)
    WgWait { wg: Box<Expr>, span: Span },
    /// Match statement: match value { case pattern: ... }
    Match {
        subject: Box<Expr>,
        cases: Vec<MatchCase>,
        span: Span,
    },
    /// Select statement for channels: select { case recv(c1): ... case send(c2, v): ... }
    Select { cases: Vec<SelectCase>, span: Span },
}

/// A single case in a match statement
#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: MatchPattern,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Patterns for match statement
#[derive(Debug, Clone)]
pub enum MatchPattern {
    /// Wildcard pattern: _
    Wildcard,
    /// Constant value: 42, "hello", True
    Constant(Expr),
    /// Variable binding: x
    Variable(String),
    /// Tuple pattern: (a, b)
    Tuple(Vec<MatchPattern>),
    /// List pattern: [a, b, ...rest]
    List {
        elements: Vec<MatchPattern>,
        rest: Option<String>,
    },
    /// Type check pattern: Type(value)
    TypeCheck {
        type_name: String,
        binding: Option<String>,
    },
    /// Range pattern: 1..5
    Range {
        start: Option<i64>,
        end: Option<i64>,
    },
}

/// A single case in a select statement
#[derive(Debug, Clone)]
pub struct SelectCase {
    pub kind: SelectCaseKind,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Kind of select case
#[derive(Debug, Clone)]
pub enum SelectCaseKind {
    /// Receive from channel: case x = recv(chan):
    Recv {
        chan: Box<Expr>,
        var: Option<String>,
    },
    /// Send to channel: case send(chan, value):
    Send { chan: Box<Expr>, value: Box<Expr> },
    /// Default case: case default:
    Default,
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Expr(e) => e.span(),
            Stmt::Assign { span, .. } => *span,
            Stmt::AugAssign { span, .. } => *span,
            Stmt::Declare { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::While { span, .. } => *span,
            Stmt::For { span, .. } => *span,
            Stmt::Function { span, .. } => *span,
            Stmt::Extern { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
            Stmt::Break(s) => *s,
            Stmt::Continue(s) => *s,
            Stmt::Pass(s) => *s,
            Stmt::Import { span, .. } => *span,
            Stmt::FromImport { span, .. } => *span,
            Stmt::Class { span, .. } => *span,
            Stmt::Struct { span, .. } => *span,
            Stmt::Try { span, .. } => *span,
            Stmt::Sync { span, .. } => *span,
            Stmt::Task { span, .. } => *span,
            Stmt::Chan { span, .. } => *span,
            Stmt::Send { span, .. } => *span,
            Stmt::Recv { span, .. } => *span,
            Stmt::WaitGroup { span } => *span,
            Stmt::WgAdd { span, .. } => *span,
            Stmt::WgDone { span, .. } => *span,
            Stmt::WgWait { span, .. } => *span,
            Stmt::Match { span, .. } => *span,
            Stmt::Select { span, .. } => *span,
        }
    }
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<Type>,
    pub default: Option<Expr>,
    pub span: Span,
}

/// Exception handler
#[derive(Debug, Clone)]
pub struct ExceptHandler {
    pub type_ann: Option<Type>,
    pub name: Option<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// A complete Viper module/program
#[derive(Debug, Default)]
pub struct Module {
    pub statements: Vec<Stmt>,
    pub span: Span,
}
