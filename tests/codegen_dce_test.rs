//! Tests for codegen::dce module

use viper_lang::codegen::DeadCodeEliminator;
use viper_lang::ast::{Expr, Stmt, Type, BinOp, Module};
use viper_lang::utils::Span;

fn span() -> Span {
    Span::empty(1, 0)
}

#[test]
fn test_eliminate_unused_var() {
    let mut dce = DeadCodeEliminator::new();

    let module = Module {
        statements: vec![
            // Unused variable
            Stmt::Declare {
                name: "x".to_string(),
                type_ann: Some(Type::I64),
                value: Some(Expr::Int(42, span())),
                mutable: false,
                span: span(),
            },
        ],
        span: span(),
    };

    let optimized = dce.optimize(&module);

    // Should eliminate the unused variable declaration
    assert_eq!(optimized.statements.len(), 0);
}

#[test]
fn test_keep_side_effects() {
    let mut dce = DeadCodeEliminator::new();

    let module = Module {
        statements: vec![
            // Function call with side effects
            Stmt::Expr(Expr::Call {
                func: Box::new(Expr::Ident("print".to_string(), span())),
                args: vec![Expr::Int(42, span())],
                span: span(),
            }),
        ],
        span: span(),
    };

    let optimized = dce.optimize(&module);

    // Should keep the function call (side effects)
    assert_eq!(optimized.statements.len(), 1);
}

#[test]
fn test_dead_store_elimination() {
    let mut dce = DeadCodeEliminator::new();

    let module = Module {
        statements: vec![
            // First assignment (dead store)
            Stmt::Declare {
                name: "x".to_string(),
                type_ann: Some(Type::I64),
                value: Some(Expr::Int(5, span())),
                mutable: false,
                span: span(),
            },
            // Second assignment (overwrites without reading)
            Stmt::Assign {
                target: Box::new(Expr::Ident("x".to_string(), span())),
                value: Box::new(Expr::Int(10, span())),
                span: span(),
            },
            // Use x
            Stmt::Expr(Expr::Call {
                func: Box::new(Expr::Ident("print".to_string(), span())),
                args: vec![Expr::Ident("x".to_string(), span())],
                span: span(),
            }),
        ],
        span: span(),
    };

    let optimized = dce.optimize(&module);

    // Should eliminate the first assignment (dead store)
    // Keep only the last assignment and the print
    assert_eq!(optimized.statements.len(), 2);
}
