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
    /// FString element - either a string literal or an expression with optional format spec
    FStringElement { expr: Box<Expr>, format_spec: Option<String>, span: Span },
    /// FString literal (sequence of string literals and elements)
    FString(Vec<Expr>, Span),
    /// Bytes literal
    Bytes(Vec<u8>, Span),
    /// BigInt literal (arbitrary precision integer)
    BigInt(String, Span),
    /// Boolean literal
    Bool(bool, Span),
    /// None literal
    None(Span),
    /// Identifier/variable reference
    Ident(String, Span),
    /// Binary operation
    BinOp { left: Box<Expr>, op: BinOp, right: Box<Expr>, span: Span },
    /// Unary operation
    UnaryOp { op: UnaryOp, operand: Box<Expr>, span: Span },
    /// Function call
    Call { func: Box<Expr>, args: Vec<Expr>, span: Span },
    /// Index access (list[i])
    Index { obj: Box<Expr>, index: Box<Expr>, span: Span },
    /// Slice access (list[start:end] or list[start:end:step])
    Slice {
        obj: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
        span: Span,
    },
    /// Attribute access (obj.attr)
    Attribute { obj: Box<Expr>, attr: String, span: Span },
    /// List literal
    List { elements: Vec<Expr>, span: Span },
    /// Tuple literal
    Tuple { elements: Vec<Expr>, span: Span },
    /// Dictionary literal
    Dict { pairs: Vec<(Expr, Expr)>, span: Span },
    /// Array literal (fixed-size): [value; size] or [elements...]
    Array { elements: Vec<Expr>, size: Option<usize>, span: Span },
    /// Lambda expression
    Lambda { params: Vec<String>, body: Box<Expr>, span: Span },
    /// List comprehension: [expr for var in iter] or [expr for var1, var2 in iter if cond]
    ListComprehension {
        element: Box<Expr>,
        target: Box<Expr>,  // Can be Ident or Tuple for unpacking
        iter: Box<Expr>,
        ifs: Vec<Expr>,     // Optional filter conditions
        span: Span,
    },
    /// Conditional expression (a if cond else b)
    Conditional { condition: Box<Expr>, then_expr: Box<Expr>, else_expr: Box<Expr>, span: Span },
    /// Await expression (await future)
    Await { future: Box<Expr>, span: Span },
    /// Assignment expression (walrus operator: :=)
    AssignmentExpr { target: Box<Expr>, value: Box<Expr>, span: Span },
    /// Super call for inheritance: super()
    Super(Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, s) => *s,
            Expr::Float(_, s) => *s,
            Expr::Str(_, s) => *s,
            Expr::FStringElement { span, .. } => *span,
            Expr::FString(_, s) => *s,
            Expr::Bytes(_, s) => *s,
            Expr::BigInt(_, s) => *s,
            Expr::Bool(_, s) => *s,
            Expr::None(s) => *s,
            Expr::Ident(_, s) => *s,
            Expr::BinOp { span, .. } => *span,
            Expr::UnaryOp { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::Slice { span, .. } => *span,
            Expr::Attribute { span, .. } => *span,
            Expr::List { span, .. } => *span,
            Expr::Tuple { span, .. } => *span,
            Expr::Dict { span, .. } => *span,
            Expr::Array { span, .. } => *span,
            Expr::Lambda { span, .. } => *span,
            Expr::ListComprehension { span, .. } => *span,
            Expr::Conditional { span, .. } => *span,
            Expr::Await { span, .. } => *span,
            Expr::AssignmentExpr { span, .. } => *span,
            Expr::Super(s) => *s,
        }
    }

    /// Try to get the identifier name if this is an Ident expression
    pub fn as_ident(&self) -> Option<&String> {
        match self {
            Expr::Ident(name, _) => Some(name),
            _ => None,
        }
    }

    /// Substitute type variables in expression
    pub fn substitute(&self, substitution: &std::collections::HashMap<String, Type>) -> Expr {
        match self {
            Expr::BinOp { left, op, right, span } => Expr::BinOp {
                left: Box::new(left.substitute(substitution)),
                op: *op,
                right: Box::new(right.substitute(substitution)),
                span: *span,
            },
            Expr::UnaryOp { op, operand, span } => Expr::UnaryOp {
                op: *op,
                operand: Box::new(operand.substitute(substitution)),
                span: *span,
            },
            Expr::Call { func, args, span } => Expr::Call {
                func: Box::new(func.substitute(substitution)),
                args: args.iter().map(|a| a.substitute(substitution)).collect(),
                span: *span,
            },
            Expr::Index { obj, index, span } => Expr::Index {
                obj: Box::new(obj.substitute(substitution)),
                index: Box::new(index.substitute(substitution)),
                span: *span,
            },
            Expr::Attribute { obj, attr, span } => Expr::Attribute {
                obj: Box::new(obj.substitute(substitution)),
                attr: attr.clone(),
                span: *span,
            },
            Expr::List { elements, span } => Expr::List {
                elements: elements.iter().map(|e| e.substitute(substitution)).collect(),
                span: *span,
            },
            Expr::Tuple { elements, span } => Expr::Tuple {
                elements: elements.iter().map(|e| e.substitute(substitution)).collect(),
                span: *span,
            },
            Expr::Dict { pairs, span } => Expr::Dict {
                pairs: pairs
                    .iter()
                    .map(|(k, v)| (k.substitute(substitution), v.substitute(substitution)))
                    .collect(),
                span: *span,
            },
            Expr::ListComprehension { element, target, iter, ifs, span } => Expr::ListComprehension {
                element: Box::new(element.substitute(substitution)),
                target: Box::new(target.substitute(substitution)),
                iter: Box::new(iter.substitute(substitution)),
                ifs: ifs.iter().map(|e| e.substitute(substitution)).collect(),
                span: *span,
            },
            Expr::Conditional { condition, then_expr, else_expr, span } => Expr::Conditional {
                condition: Box::new(condition.substitute(substitution)),
                then_expr: Box::new(then_expr.substitute(substitution)),
                else_expr: Box::new(else_expr.substitute(substitution)),
                span: *span,
            },
            Expr::Await { future, span } => {
                Expr::Await { future: Box::new(future.substitute(substitution)), span: *span }
            }
            Expr::AssignmentExpr { target, value, span } => Expr::AssignmentExpr {
                target: Box::new(target.substitute(substitution)),
                value: Box::new(value.substitute(substitution)),
                span: *span,
            },
            Expr::FStringElement { expr, format_spec, span } => Expr::FStringElement {
                expr: Box::new(expr.substitute(substitution)),
                format_spec: format_spec.clone(),
                span: *span,
            },
            _ => self.clone(),
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
    // Null coalescing (Phase 3)
    NullCoalesce,
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
            BinOp::NullCoalesce => 5, // Same as 'and' - low precedence
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
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement,
    /// Unwrap operator: `?` - propagates errors from Result types
    Unwrap,
    /// Unwrap with message: `unwrap_or_default()` helper
    UnwrapOrDefault,
}

/// Abstract Syntax Tree node for statements
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Expression statement (expression as statement)
    Expr(Expr),
    /// Variable assignment: x = expr
    Assign { target: Box<Expr>, value: Box<Expr>, span: Span },
    /// Augmented assignment: x += expr
    AugAssign { target: Box<Expr>, op: BinOp, value: Box<Expr>, span: Span },
    /// Slice assignment: obj[start:end:step] = value
    SliceAssign {
        obj: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
        value: Box<Expr>,
        span: Span,
    },
    /// Variable declaration with type: x: i64 = expr
    Declare { name: String, type_ann: Option<Type>, value: Option<Expr>, mutable: bool, span: Span },
    /// Global variable declaration: global x, y, z (inside function)
    Global { names: Vec<String>, span: Span },
    /// Nonlocal variable declaration: nonlocal x, y (inside nested function)
    Nonlocal { names: Vec<String>, span: Span },
    /// Constant declaration: const PI = 3.14
    Const { name: String, value: Expr, span: Span },
    /// If statement
    If {
        condition: Expr,
        body: Vec<Stmt>,
        elif_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    /// While loop
    While { condition: Expr, body: Vec<Stmt>, else_body: Option<Vec<Stmt>>, span: Span },
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
        decorators: Vec<Decorator>,
    },
    /// External C function declaration: extern "C" fn name(args...) -> ret
    Extern { name: String, params: Vec<Param>, return_type: Option<Type>, span: Span },
    /// Return statement
    Return { value: Option<Expr>, span: Span },
    /// Break statement
    Break(Span),
    /// Continue statement
    Continue(Span),
    /// Pass statement
    Pass(Span),
    /// Import statement
    Import { module: String, alias: Option<String>, span: Span },
    /// From import
    FromImport { module: String, names: Vec<(String, Option<String>)>, span: Span },
    /// Class definition
    Class {
        name: String,
        bases: Vec<Expr>,
        body: Vec<Stmt>,
        span: Span,
        decorators: Vec<Decorator>,
        /// Fields declared in the class body (name, type, is_class_var)
        fields: Vec<(String, Option<Type>, bool)>,
        /// Methods defined in the class
        methods: Vec<String>,
    },
    /// Struct definition
    Struct { name: String, fields: Vec<(String, Type)>, span: Span },
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
    Send { chan: Box<Expr>, value: Box<Expr>, span: Span },
    /// Channel receive: recv(chan)
    Recv { chan: Box<Expr>, span: Span },
    /// WaitGroup creation
    WaitGroup { span: Span },
    /// WaitGroup add: add(wg, n)
    WgAdd { wg: Box<Expr>, n: Box<Expr>, span: Span },
    /// WaitGroup done: done(wg)
    WgDone { wg: Box<Expr>, span: Span },
    /// WaitGroup wait: wait(wg)
    WgWait { wg: Box<Expr>, span: Span },
    /// Type alias: type Name = Type
    TypeAlias { name: String, type_def: Type, span: Span },
    /// Match statement: match value { case pattern: ... }
    Match { subject: Box<Expr>, cases: Vec<MatchCase>, span: Span },
    /// Select statement for channels: select { case recv(c1): ... case send(c2, v): ... }
    Select { cases: Vec<SelectCase>, span: Span },
    /// Assert statement: assert condition, message
    Assert { condition: Box<Expr>, message: Option<Box<Expr>>, span: Span },
    /// Delete statement: del target1, target2, ...
    Delete { targets: Vec<Expr>, span: Span },
    /// Raise statement: raise Exception() or raise Exception() from cause
    Raise { exception: Option<Box<Expr>>, cause: Option<Box<Expr>>, span: Span },
    /// With statement: with expr as var: body
    With { items: Vec<WithItem>, body: Vec<Stmt>, is_async: bool, span: Span },
    /// Yield statement: yield expr or yield
    Yield { value: Option<Box<Expr>>, span: Span },
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
    List { elements: Vec<MatchPattern>, rest: Option<String> },
    /// Type check pattern: Type(value)
    TypeCheck { type_name: String, binding: Option<String> },
    /// Range pattern: 1..5
    Range { start: Option<i64>, end: Option<i64> },
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
    Recv { chan: Box<Expr>, var: Option<String> },
    /// Send to channel: case send(chan, value):
    Send { chan: Box<Expr>, value: Box<Expr> },
    /// Default case: case default:
    Default,
}

/// A single item in a with statement: expr as var
#[derive(Debug, Clone)]
pub struct WithItem {
    pub context_expr: Expr,
    pub optional_vars: Option<String>,
    pub span: Span,
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Expr(e) => e.span(),
            Stmt::Assign { span, .. } => *span,
            Stmt::AugAssign { span, .. } => *span,
            Stmt::SliceAssign { span, .. } => *span,
            Stmt::Declare { span, .. } => *span,
            Stmt::Global { span, .. } => *span,
            Stmt::Nonlocal { span, .. } => *span,
            Stmt::Const { span, .. } => *span,
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
            Stmt::TypeAlias { span, .. } => *span,
            Stmt::Assert { span, .. } => *span,
            Stmt::Delete { span, .. } => *span,
            Stmt::Raise { span, .. } => *span,
            Stmt::With { span, .. } => *span,
            Stmt::Yield { span, .. } => *span,
        }
    }

    /// Substitute type variables in statement
    pub fn substitute(&self, substitution: &std::collections::HashMap<String, Type>) -> Stmt {
        match self {
            Stmt::Expr(expr) => Stmt::Expr(expr.substitute(substitution)),
            Stmt::Assign { target, value, span } => Stmt::Assign {
                target: Box::new(target.substitute(substitution)),
                value: Box::new(value.substitute(substitution)),
                span: *span,
            },
            Stmt::SliceAssign { obj, start, end, step, value, span } => Stmt::SliceAssign {
                obj: Box::new(obj.substitute(substitution)),
                start: start.as_ref().map(|e| Box::new(e.substitute(substitution))),
                end: end.as_ref().map(|e| Box::new(e.substitute(substitution))),
                step: step.as_ref().map(|e| Box::new(e.substitute(substitution))),
                value: Box::new(value.substitute(substitution)),
                span: *span,
            },
            Stmt::Declare { name, type_ann, value, mutable, span } => Stmt::Declare {
                name: name.clone(),
                type_ann: type_ann.as_ref().map(|t| t.substitute(substitution)),
                value: value.as_ref().map(|v| v.substitute(substitution)),
                mutable: *mutable,
                span: *span,
            },
            Stmt::If { condition, body, elif_blocks, else_body, span } => Stmt::If {
                condition: condition.substitute(substitution),
                body: body.iter().map(|s| s.substitute(substitution)).collect(),
                elif_blocks: elif_blocks
                    .iter()
                    .map(|(c, b)| {
                        (c.substitute(substitution), b.iter().map(|s| s.substitute(substitution)).collect())
                    })
                    .collect(),
                else_body: else_body
                    .as_ref()
                    .map(|b| b.iter().map(|s| s.substitute(substitution)).collect()),
                span: *span,
            },
            Stmt::While { condition, body, else_body, span } => Stmt::While {
                condition: condition.substitute(substitution),
                body: body.iter().map(|s| s.substitute(substitution)).collect(),
                else_body: else_body
                    .as_ref()
                    .map(|b| b.iter().map(|s| s.substitute(substitution)).collect()),
                span: *span,
            },
            Stmt::For { target, iter, body, else_body, is_async, span } => Stmt::For {
                target: Box::new(target.substitute(substitution)),
                iter: Box::new(iter.substitute(substitution)),
                body: body.iter().map(|s| s.substitute(substitution)).collect(),
                else_body: else_body
                    .as_ref()
                    .map(|b| b.iter().map(|s| s.substitute(substitution)).collect()),
                is_async: *is_async,
                span: *span,
            },
            Stmt::Return { value, span } => Stmt::Return {
                value: value.as_ref().map(|v| v.substitute(substitution)),
                span: *span,
            },
            Stmt::Try { body, handlers, else_body, finally_body, span } => Stmt::Try {
                body: body.iter().map(|s| s.substitute(substitution)).collect(),
                handlers: handlers
                    .iter()
                    .map(|h| ExceptHandler {
                        type_ann: h.type_ann.as_ref().map(|t| t.substitute(substitution)),
                        name: h.name.clone(),
                        body: h.body.iter().map(|s| s.substitute(substitution)).collect(),
                        span: h.span,
                    })
                    .collect(),
                else_body: else_body
                    .as_ref()
                    .map(|b| b.iter().map(|s| s.substitute(substitution)).collect()),
                finally_body: finally_body
                    .as_ref()
                    .map(|b| b.iter().map(|s| s.substitute(substitution)).collect()),
                span: *span,
            },
            Stmt::With { items, body, is_async, span } => Stmt::With {
                items: items
                    .iter()
                    .map(|i| WithItem {
                        context_expr: i.context_expr.substitute(substitution),
                        optional_vars: i.optional_vars.clone(),
                        span: i.span,
                    })
                    .collect(),
                body: body.iter().map(|s| s.substitute(substitution)).collect(),
                is_async: *is_async,
                span: *span,
            },
            Stmt::Function {
                name,
                type_params,
                params,
                return_type,
                body,
                span,
                is_async,
                decorators,
            } => Stmt::Function {
                name: name.clone(),
                type_params: type_params.clone(),
                params: params
                    .iter()
                    .map(|p| Param {
                        name: p.name.clone(),
                        type_ann: p.type_ann.as_ref().map(|t| t.substitute(substitution)),
                        default: p.default.as_ref().map(|d| d.substitute(substitution)),
                        span: p.span,
                        is_variadic: p.is_variadic,
                        is_kw_variadic: p.is_kw_variadic,
                    })
                    .collect(),
                return_type: return_type.as_ref().map(|t| t.substitute(substitution)),
                body: body.iter().map(|s| s.substitute(substitution)).collect(),
                span: *span,
                is_async: *is_async,
                decorators: decorators.clone(), // Substituted if needed
            },
            _ => self.clone(),
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
    pub is_variadic: bool,    // *args
    pub is_kw_variadic: bool, // **kwargs
}

/// Decorator for functions, classes, and methods
#[derive(Debug, Clone)]
pub struct Decorator {
    pub name: String,
    pub args: Vec<Expr>,
    pub keywords: Vec<(String, Expr)>,
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
