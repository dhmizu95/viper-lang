//! Unit tests for the Viper AST module
//! Tests for: Type methods, BinOp precedence/associativity, Expr::span/as_ident, Stmt::span

use viper_lang::ast::{BinOp, Expr, Stmt, Type, UnaryOp};
use viper_lang::utils::Span;

// ============================================================================
// Type Tests
// ============================================================================

#[test]
fn test_type_is_numeric() {
    assert!(Type::I8.is_numeric());
    assert!(Type::I16.is_numeric());
    assert!(Type::I32.is_numeric());
    assert!(Type::I64.is_numeric());
    assert!(Type::F32.is_numeric());
    assert!(Type::F64.is_numeric());

    assert!(!Type::Bool.is_numeric());
    assert!(!Type::Str.is_numeric());
    assert!(!Type::None.is_numeric());
}

#[test]
fn test_type_is_integer() {
    assert!(Type::I8.is_integer());
    assert!(Type::I16.is_integer());
    assert!(Type::I32.is_integer());
    assert!(Type::I64.is_integer());

    assert!(!Type::F32.is_integer());
    assert!(!Type::F64.is_integer());
    assert!(!Type::Bool.is_integer());
}

#[test]
fn test_type_is_float() {
    assert!(Type::F32.is_float());
    assert!(Type::F64.is_float());

    assert!(!Type::I64.is_float());
}

#[test]
fn test_type_is_infer() {
    assert!(Type::Infer.is_infer());
    assert!(!Type::I64.is_infer());
    assert!(!Type::Str.is_infer());
}

#[test]
fn test_type_is_error() {
    assert!(Type::Error.is_error());
    assert!(!Type::I64.is_error());
    assert!(!Type::Str.is_error());
}

#[test]
fn test_type_is_hashable() {
    assert!(Type::I8.is_hashable());
    assert!(Type::I16.is_hashable());
    assert!(Type::I32.is_hashable());
    assert!(Type::I64.is_hashable());
    assert!(Type::F32.is_hashable());
    assert!(Type::F64.is_hashable());
    assert!(Type::Bool.is_hashable());
    assert!(Type::Str.is_hashable());

    assert!(!Type::List(Box::new(Type::I64)).is_hashable());
    assert!(!Type::Dict(Box::new(Type::I64), Box::new(Type::I64)).is_hashable());
}

#[test]
fn test_type_is_hashable_tuple() {
    let hashable_tuple = Type::Tuple(vec![Type::I64, Type::Bool, Type::Str]);
    assert!(hashable_tuple.is_hashable_tuple());

    let non_hashable_tuple = Type::Tuple(vec![Type::I64, Type::List(Box::new(Type::I64))]);
    assert!(!non_hashable_tuple.is_hashable_tuple());

    assert!(!Type::I64.is_hashable_tuple());
}

#[test]
fn test_type_is_fully_hashable() {
    assert!(Type::I64.is_fully_hashable());
    assert!(Type::Str.is_fully_hashable());

    let hashable_tuple = Type::Tuple(vec![Type::I64, Type::Bool]);
    assert!(hashable_tuple.is_fully_hashable());

    assert!(!Type::List(Box::new(Type::I64)).is_fully_hashable());
    assert!(!Type::Dict(Box::new(Type::I64), Box::new(Type::I64)).is_fully_hashable());
    assert!(!Type::Array(Box::new(Type::I64), 5).is_fully_hashable());
}

#[test]
fn test_type_display_primitives() {
    assert_eq!(format!("{}", Type::I8), "i8");
    assert_eq!(format!("{}", Type::I16), "i16");
    assert_eq!(format!("{}", Type::I32), "i32");
    assert_eq!(format!("{}", Type::I64), "i64");
    assert_eq!(format!("{}", Type::F32), "f32");
    assert_eq!(format!("{}", Type::F64), "f64");
    assert_eq!(format!("{}", Type::Bool), "bool");
    assert_eq!(format!("{}", Type::Str), "str");
    assert_eq!(format!("{}", Type::None), "None");
    assert_eq!(format!("{}", Type::Infer), "_");
    assert_eq!(format!("{}", Type::Error), "<error>");
}

#[test]
fn test_type_display_list() {
    let list = Type::List(Box::new(Type::I64));
    assert_eq!(format!("{}", list), "[i64]");

    let nested = Type::List(Box::new(Type::List(Box::new(Type::I64))));
    assert_eq!(format!("{}", nested), "[[i64]]");
}

#[test]
fn test_type_display_dict() {
    let dict = Type::Dict(Box::new(Type::Str), Box::new(Type::I64));
    assert_eq!(format!("{}", dict), "{str: i64}");
}

#[test]
fn test_type_display_tuple() {
    let tuple = Type::Tuple(vec![Type::I64, Type::Str]);
    assert_eq!(format!("{}", tuple), "(i64, str)");

    let empty: Type = Type::Tuple(vec![]);
    assert_eq!(format!("{}", empty), "()");
}

#[test]
fn test_type_display_array() {
    let array = Type::Array(Box::new(Type::I64), 5);
    assert_eq!(format!("{}", array), "[i64; 5]");
}

#[test]
fn test_type_display_fn() {
    let fn_type = Type::Fn(vec![Type::I64, Type::I64], Box::new(Type::I64));
    assert_eq!(format!("{}", fn_type), "fn(i64, i64) -> i64");
}

#[test]
fn test_type_display_chan() {
    let chan = Type::Chan(Box::new(Type::I64));
    assert_eq!(format!("{}", chan), "chan[i64]");
}

#[test]
fn test_type_display_optional() {
    let opt = Type::Optional(Box::new(Type::I64));
    assert_eq!(format!("{}", opt), "i64?");
}

#[test]
fn test_type_display_waitgroup() {
    assert_eq!(format!("{}", Type::WaitGroup), "WaitGroup");
}

#[test]
fn test_type_display_struct() {
    let struct_type = Type::Struct {
        name: "Person".to_string(),
        fields: vec![("name".to_string(), Type::Str), ("age".to_string(), Type::I64)],
    };
    assert_eq!(format!("{}", struct_type), "struct Person { name: str, age: i64 }");
}

#[test]
fn test_type_display_future() {
    let future = Type::Future(Box::new(Type::I64));
    assert_eq!(format!("{}", future), "Future[i64]");
}

#[test]
fn test_type_display_var() {
    let var = Type::Var("T".to_string());
    assert_eq!(format!("{}", var), "T");
}

// ============================================================================
// BinOp Tests
// ============================================================================

#[test]
fn test_binop_precedence_pow() {
    assert_eq!(BinOp::Pow.precedence(), 14);
}

#[test]
fn test_binop_precedence_mult() {
    assert_eq!(BinOp::Mul.precedence(), 13);
    assert_eq!(BinOp::Div.precedence(), 13);
    assert_eq!(BinOp::Mod.precedence(), 13);
    assert_eq!(BinOp::FloorDiv.precedence(), 13);
}

#[test]
fn test_binop_precedence_add() {
    assert_eq!(BinOp::Add.precedence(), 12);
    assert_eq!(BinOp::Sub.precedence(), 12);
}

#[test]
fn test_binop_precedence_shift() {
    assert_eq!(BinOp::LShift.precedence(), 11);
    assert_eq!(BinOp::RShift.precedence(), 11);
}

#[test]
fn test_binop_precedence_bitwise() {
    assert_eq!(BinOp::BitAnd.precedence(), 10);
    assert_eq!(BinOp::BitXor.precedence(), 9);
    assert_eq!(BinOp::BitOr.precedence(), 8);
}

#[test]
fn test_binop_precedence_comparison() {
    assert_eq!(BinOp::Lt.precedence(), 7);
    assert_eq!(BinOp::LtEq.precedence(), 7);
    assert_eq!(BinOp::Gt.precedence(), 7);
    assert_eq!(BinOp::GtEq.precedence(), 7);
}

#[test]
fn test_binop_precedence_equality() {
    assert_eq!(BinOp::Eq.precedence(), 6);
    assert_eq!(BinOp::NotEq.precedence(), 6);
    assert_eq!(BinOp::Is.precedence(), 6);
    assert_eq!(BinOp::IsNot.precedence(), 6);
    assert_eq!(BinOp::In.precedence(), 6);
    assert_eq!(BinOp::NotIn.precedence(), 6);
}

#[test]
fn test_binop_precedence_logical() {
    assert_eq!(BinOp::And.precedence(), 5);
    assert_eq!(BinOp::Or.precedence(), 4);
}

#[test]
fn test_binop_is_right_associative() {
    assert!(BinOp::Pow.is_right_associative());
    assert!(!BinOp::Add.is_right_associative());
    assert!(!BinOp::Mul.is_right_associative());
    assert!(!BinOp::Eq.is_right_associative());
}

// ============================================================================
// Expr::span Tests
// ============================================================================

fn test_span() -> Span {
    Span::new(0, 5, 1, 1)
}

#[test]
fn test_expr_span_int() {
    let expr = Expr::Int(42, test_span());
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_float() {
    let expr = Expr::Float(3.14, test_span());
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_str() {
    let expr = Expr::Str("hello".to_string(), test_span());
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_bool() {
    let expr = Expr::Bool(true, test_span());
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_none() {
    let expr = Expr::None(test_span());
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_ident() {
    let expr = Expr::Ident("x".to_string(), test_span());
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_binop() {
    let left = Box::new(Expr::Int(1, test_span()));
    let right = Box::new(Expr::Int(2, test_span()));
    let expr = Expr::BinOp { left, op: BinOp::Add, right, span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_unaryop() {
    let operand = Box::new(Expr::Int(42, test_span()));
    let expr = Expr::UnaryOp { op: UnaryOp::Neg, operand, span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_call() {
    let func = Box::new(Expr::Ident("foo".to_string(), test_span()));
    let expr = Expr::Call { func, args: vec![], span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_index() {
    let obj = Box::new(Expr::Ident("lst".to_string(), test_span()));
    let index = Box::new(Expr::Int(0, test_span()));
    let expr = Expr::Index { obj, index, span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_slice() {
    let obj = Box::new(Expr::Ident("lst".to_string(), test_span()));
    let expr = Expr::Slice { obj, start: None, end: None, step: None, span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_attribute() {
    let obj = Box::new(Expr::Ident("obj".to_string(), test_span()));
    let expr = Expr::Attribute { obj, attr: "field".to_string(), span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_list() {
    let expr = Expr::List { elements: vec![], span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_tuple() {
    let expr = Expr::Tuple { elements: vec![], span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_dict() {
    let expr = Expr::Dict { pairs: vec![], span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_array() {
    let expr = Expr::Array { elements: vec![], size: None, span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_lambda() {
    let body = Box::new(Expr::Int(42, test_span()));
    let expr = Expr::Lambda { params: vec![], body, span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_list_comprehension() {
    let element = Box::new(Expr::Int(1, test_span()));
    let iter = Box::new(Expr::Ident("lst".to_string(), test_span()));
    let target = Box::new(Expr::Ident("x".to_string(), test_span()));
    let expr = Expr::ListComprehension { element, target, iter, ifs: vec![], span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_conditional() {
    let cond = Box::new(Expr::Bool(true, test_span()));
    let then_expr = Box::new(Expr::Int(1, test_span()));
    let else_expr = Box::new(Expr::Int(2, test_span()));
    let expr = Expr::Conditional { condition: cond, then_expr, else_expr, span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_await() {
    let future = Box::new(Expr::Ident("fut".to_string(), test_span()));
    let expr = Expr::Await { future, span: test_span() };
    assert_eq!(expr.span(), test_span());
}

#[test]
fn test_expr_span_fstring() {
    let expr = Expr::FString(vec![], test_span());
    assert_eq!(expr.span(), test_span());
}

// ============================================================================
// Expr::as_ident Tests
// ============================================================================

#[test]
fn test_expr_as_ident_ident() {
    let expr = Expr::Ident("x".to_string(), test_span());
    assert_eq!(expr.as_ident(), Some(&"x".to_string()));
}

#[test]
fn test_expr_as_ident_int() {
    let expr = Expr::Int(42, test_span());
    assert_eq!(expr.as_ident(), None);
}

#[test]
fn test_expr_as_ident_binop() {
    let left = Box::new(Expr::Int(1, test_span()));
    let right = Box::new(Expr::Int(2, test_span()));
    let expr = Expr::BinOp { left, op: BinOp::Add, right, span: test_span() };
    assert_eq!(expr.as_ident(), None);
}

#[test]
fn test_expr_as_ident_call() {
    let func = Box::new(Expr::Ident("foo".to_string(), test_span()));
    let expr = Expr::Call { func, args: vec![], span: test_span() };
    assert_eq!(expr.as_ident(), None);
}

#[test]
fn test_expr_as_ident_attribute() {
    let obj = Box::new(Expr::Ident("obj".to_string(), test_span()));
    let expr = Expr::Attribute { obj, attr: "field".to_string(), span: test_span() };
    assert_eq!(expr.as_ident(), None);
}

// ============================================================================
// Stmt::span Tests
// ============================================================================

#[test]
fn test_stmt_span_expr() {
    let expr = Expr::Int(42, test_span());
    let stmt = Stmt::Expr(expr);
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_assign() {
    let target = Box::new(Expr::Ident("x".to_string(), test_span()));
    let value = Box::new(Expr::Int(42, test_span()));
    let stmt = Stmt::Assign { target, value, span: test_span() };
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_aug_assign() {
    let target = Box::new(Expr::Ident("x".to_string(), test_span()));
    let value = Box::new(Expr::Int(42, test_span()));
    let stmt = Stmt::AugAssign { target, op: BinOp::Add, value, span: test_span() };
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_declare() {
    let stmt = Stmt::Declare {
        name: "x".to_string(),
        type_ann: None,
        value: None,
        mutable: false,
        span: test_span(),
    };
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_if() {
    let condition = Expr::Bool(true, test_span());
    let stmt = Stmt::If {
        condition,
        body: vec![],
        elif_blocks: vec![],
        else_body: None,
        span: test_span(),
    };
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_while() {
    let condition = Expr::Bool(true, test_span());
    let stmt = Stmt::While { condition, body: vec![], else_body: None, span: test_span() };
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_for() {
    let target = Box::new(Expr::Ident("x".to_string(), test_span()));
    let iter = Box::new(Expr::Ident("lst".to_string(), test_span()));
    let stmt = Stmt::For {
        target,
        iter,
        body: vec![],
        else_body: None,
        is_async: false,
        span: test_span(),
    };
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_function() {
    let stmt = Stmt::Function {
        name: "foo".to_string(),
        type_params: vec![],
        params: vec![],
        return_type: None,
        body: vec![],
        span: test_span(),
        is_async: false,
        decorators: vec![],
    };
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_return() {
    let stmt = Stmt::Return { value: None, span: test_span() };
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_break() {
    let stmt = Stmt::Break(test_span());
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_continue() {
    let stmt = Stmt::Continue(test_span());
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_pass() {
    let stmt = Stmt::Pass(test_span());
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_match() {
    let subject = Box::new(Expr::Int(42, test_span()));
    let stmt = Stmt::Match { subject, cases: vec![], span: test_span() };
    assert_eq!(stmt.span(), test_span());
}

#[test]
fn test_stmt_span_select() {
    let stmt = Stmt::Select { cases: vec![], span: test_span() };
    assert_eq!(stmt.span(), test_span());
}
