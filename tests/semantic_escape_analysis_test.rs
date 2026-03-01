//! Tests for semantic::escape_analysis module

use viper_lang::semantic::{EscapeAnalyzer, EscapeState};
use viper_lang::ast::{Expr, Stmt, Type, BinOp};
use viper_lang::utils::Span;

#[test]
fn test_escape_state_merge() {
    assert_eq!(EscapeState::None.merge(EscapeState::None), EscapeState::None);
    assert_eq!(EscapeState::None.merge(EscapeState::MayEscape), EscapeState::MayEscape);
    assert_eq!(EscapeState::None.merge(EscapeState::Shared), EscapeState::Shared);
    assert_eq!(EscapeState::MayEscape.merge(EscapeState::Shared), EscapeState::Shared);
    assert_eq!(EscapeState::Shared.merge(EscapeState::None), EscapeState::Shared);
}

#[test]
fn test_can_stack_allocate() {
    assert!(EscapeState::None.can_stack_allocate());
    assert!(!EscapeState::Returned.can_stack_allocate());
    assert!(!EscapeState::MayEscape.can_stack_allocate());
    assert!(!EscapeState::Shared.can_stack_allocate());
}

#[test]
fn test_simple_function_no_escape() {
    let mut analyzer = EscapeAnalyzer::new();

    // Create a simple function: def foo(): x = 5; return x
    let body = vec![
        Stmt::Declare {
            name: "x".to_string(),
            type_ann: Some(Type::I64),
            value: Some(Expr::Int(5, Span::empty(1, 0))),
            mutable: false,
            span: Span::empty(1, 0),
        },
        Stmt::Return {
            value: Some(Expr::Ident("x".to_string(), Span::empty(2, 0))),
            span: Span::empty(2, 0),
        },
    ];

    analyzer.analyze_function("foo", &body);

    // Variable x escapes because it's returned
    let info = analyzer.get_variable_escape_info("foo", "x").unwrap();
    assert_eq!(info.escape_state, EscapeState::Returned);
}

#[test]
fn test_local_variable_no_escape() {
    let mut analyzer = EscapeAnalyzer::new();

    // Function with local variable that doesn't escape: def foo(): x = 5; y = x + 1; return y
    let body = vec![
        Stmt::Declare {
            name: "x".to_string(),
            type_ann: Some(Type::I64),
            value: Some(Expr::Int(5, Span::empty(1, 0))),
            mutable: false,
            span: Span::empty(1, 0),
        },
        Stmt::Declare {
            name: "y".to_string(),
            type_ann: Some(Type::I64),
            value: Some(Expr::BinOp {
                left: Box::new(Expr::Ident("x".to_string(), Span::empty(2, 0))),
                op: BinOp::Add,
                right: Box::new(Expr::Int(1, Span::empty(2, 0))),
                span: Span::empty(2, 0),
            }),
            mutable: false,
            span: Span::empty(2, 0),
        },
        Stmt::Return {
            value: Some(Expr::Ident("y".to_string(), Span::empty(3, 0))),
            span: Span::empty(3, 0),
        },
    ];

    analyzer.analyze_function("foo", &body);

    // x is used locally but not returned directly
    let x_info = analyzer.get_variable_escape_info("foo", "x").unwrap();
    assert_eq!(x_info.escape_state, EscapeState::None);

    // y escapes because it's returned
    let y_info = analyzer.get_variable_escape_info("foo", "y").unwrap();
    assert_eq!(y_info.escape_state, EscapeState::Returned);
}
