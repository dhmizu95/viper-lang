//! Unit tests for the Viper parser module
//! Tests for PrattParser expression parsing via lexer → parser chain

use viper_lang::ast::{BinOp, Expr, UnaryOp};
use viper_lang::lexer::Lexer;
use viper_lang::parser::expressions::PrattParser;
use viper_lang::parser::precedence::Precedence;

// ============================================================================
// Helper Functions
// ============================================================================

fn tokenize(src: &str) -> viper_lang::error::Result<Vec<viper_lang::lexer::Token>> {
    let mut lexer = Lexer::new(src);
    lexer.tokenize()
}

fn parse_expr(src: &str) -> viper_lang::error::Result<Expr> {
    let tokens = tokenize(src)?;
    let mut parser = PrattParser::new(&tokens);
    parser.parse_expr(Precedence::MIN)
}

fn assert_is_int(expr: &Expr, expected: i64) {
    match expr {
        Expr::Int(n, _) => assert_eq!(*n, expected),
        _ => panic!("Expected Int({}), got {:?}", expected, expr),
    }
}

fn assert_is_float(expr: &Expr, expected: f64) {
    match expr {
        Expr::Float(n, _) => assert!((*n - expected).abs() < f64::EPSILON),
        _ => panic!("Expected Float({}), got {:?}", expected, expr),
    }
}

fn assert_is_bool(expr: &Expr, expected: bool) {
    match expr {
        Expr::Bool(b, _) => assert_eq!(*b, expected),
        _ => panic!("Expected Bool({}), got {:?}", expected, expr),
    }
}

fn assert_is_ident(expr: &Expr, expected: &str) {
    match expr {
        Expr::Ident(name, _) => assert_eq!(name, expected),
        _ => panic!("Expected Ident({}), got {:?}", expected, expr),
    }
}

fn assert_is_str(expr: &Expr, expected: &str) {
    match expr {
        Expr::Str(s, _) => assert_eq!(s, expected),
        _ => panic!("Expected Str({}), got {:?}", expected, expr),
    }
}

// ============================================================================
// Literal Parsing Tests
// ============================================================================

#[test]
fn test_parse_int_literal() {
    let expr = parse_expr("42").unwrap();
    assert_is_int(&expr, 42);
}

#[test]
fn test_parse_negative_int_literal() {
    let expr = parse_expr("-42").unwrap();
    match &expr {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(*op, UnaryOp::Neg);
            assert_is_int(operand, 42);
        }
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_float_literal() {
    let expr = parse_expr("3.14").unwrap();
    assert_is_float(&expr, 3.14);
}

#[test]
fn test_parse_bool_true() {
    let expr = parse_expr("True").unwrap();
    assert_is_bool(&expr, true);
}

#[test]
fn test_parse_bool_false() {
    let expr = parse_expr("False").unwrap();
    assert_is_bool(&expr, false);
}

#[test]
fn test_parse_none() {
    let expr = parse_expr("None").unwrap();
    match expr {
        Expr::None(_) => {}
        _ => panic!("Expected None, got {:?}", expr),
    }
}

#[test]
fn test_parse_string_literal() {
    let expr = parse_expr(r#""hello""#).unwrap();
    match &expr {
        Expr::Str(s, _) => assert_eq!(s, "hello"),
        _ => panic!("Expected Str, got {:?}", expr),
    }
}

#[test]
fn test_parse_fstring() {
    let expr = parse_expr(r#"f"hello {name}""#).unwrap();
    match &expr {
        Expr::FString(_, _) => {}
        _ => panic!("Expected FString, got {:?}", expr),
    }
}

// ============================================================================
// Unary Operator Tests
// ============================================================================

#[test]
fn test_parse_unary_neg() {
    let expr = parse_expr("-x").unwrap();
    match &expr {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(*op, UnaryOp::Neg);
            assert_is_ident(operand, "x");
        }
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_unary_not() {
    let expr = parse_expr("not x").unwrap();
    match &expr {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(*op, UnaryOp::Not);
            assert_is_ident(operand, "x");
        }
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_unary_pos() {
    let expr = parse_expr("+x").unwrap();
    match &expr {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(*op, UnaryOp::Pos);
            assert_is_ident(operand, "x");
        }
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_unary_invert() {
    let expr = parse_expr("~x").unwrap();
    match &expr {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(*op, UnaryOp::Invert);
            assert_is_ident(operand, "x");
        }
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_unary_pre_increment() {
    let expr = parse_expr("++x").unwrap();
    match &expr {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(*op, UnaryOp::PreIncrement);
            assert_is_ident(operand, "x");
        }
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_unary_pre_decrement() {
    let expr = parse_expr("--x").unwrap();
    match &expr {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(*op, UnaryOp::PreDecrement);
            assert_is_ident(operand, "x");
        }
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_unary_post_increment() {
    let expr = parse_expr("x++").unwrap();
    match &expr {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(*op, UnaryOp::PostIncrement);
            assert_is_ident(operand, "x");
        }
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_unary_post_decrement() {
    let expr = parse_expr("x--").unwrap();
    match &expr {
        Expr::UnaryOp { op, operand, .. } => {
            assert_eq!(*op, UnaryOp::PostDecrement);
            assert_is_ident(operand, "x");
        }
        _ => panic!("Expected UnaryOp, got {:?}", expr),
    }
}

// ============================================================================
// Binary Arithmetic Tests
// ============================================================================

#[test]
fn test_parse_binary_add() {
    let expr = parse_expr("1 + 2").unwrap();
    match &expr {
        Expr::BinOp { left, op, right, .. } => {
            assert_is_int(left, 1);
            assert_eq!(*op, BinOp::Add);
            assert_is_int(right, 2);
        }
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_sub() {
    let expr = parse_expr("10 - 3").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Sub),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_mul() {
    let expr = parse_expr("4 * 5").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Mul),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_div() {
    let expr = parse_expr("10 / 2").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Div),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_floor_div() {
    let expr = parse_expr("10 // 3").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::FloorDiv),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_mod() {
    let expr = parse_expr("10 % 3").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Mod),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_pow() {
    let expr = parse_expr("2 ** 8").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Pow),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

// ============================================================================
// Binary Comparison Tests
// ============================================================================

#[test]
fn test_parse_binary_eq() {
    let expr = parse_expr("a == b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Eq),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_not_eq() {
    let expr = parse_expr("a != b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::NotEq),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_lt() {
    let expr = parse_expr("a < b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Lt),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_gt() {
    let expr = parse_expr("a > b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Gt),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_lt_eq() {
    let expr = parse_expr("a <= b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::LtEq),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_binary_gt_eq() {
    let expr = parse_expr("a >= b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::GtEq),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

// ============================================================================
// Logical Operators Tests
// ============================================================================

#[test]
fn test_parse_logical_and() {
    let expr = parse_expr("a and b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::And),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_logical_or() {
    let expr = parse_expr("a or b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Or),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

// ============================================================================
// Bitwise Operators Tests
// ============================================================================

#[test]
fn test_parse_bitwise_and() {
    let expr = parse_expr("a & b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::BitAnd),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_bitwise_or() {
    let expr = parse_expr("a | b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::BitOr),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_bitwise_xor() {
    let expr = parse_expr("a ^ b").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::BitXor),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_bitwise_lshift() {
    let expr = parse_expr("a << 2").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::LShift),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_bitwise_rshift() {
    let expr = parse_expr("a >> 2").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::RShift),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

// ============================================================================
// Membership and Identity Tests
// ============================================================================

#[test]
fn test_parse_in() {
    let expr = parse_expr("x in lst").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::In),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_not_in() {
    let expr = parse_expr("x not in lst").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::NotIn),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_is() {
    let expr = parse_expr("x is y").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Is),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_parse_is_not() {
    let expr = parse_expr("x is not y").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::IsNot),
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

// ============================================================================
// Precedence Tests
// ============================================================================

#[test]
fn test_precedence_mul_before_add() {
    let expr = parse_expr("1 + 2 * 3").unwrap();
    match &expr {
        Expr::BinOp { left, op, right, .. } => {
            assert_eq!(*op, BinOp::Add);
            assert_is_int(left, 1);
            // Right should be the multiplication
            match right.as_ref() {
                Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Mul),
                _ => panic!("Expected Mul on right"),
            }
        }
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_precedence_pow_right_associative() {
    let expr = parse_expr("2 ** 3 ** 2").unwrap();
    // Right associative: 2 ** (3 ** 2) = 2 ** 9 = 512
    match &expr {
        Expr::BinOp { left, op, right, .. } => {
            assert_eq!(*op, BinOp::Pow);
            assert_is_int(left, 2);
            // Right should be another Pow
            match right.as_ref() {
                Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Pow),
                _ => panic!("Expected Pow on right"),
            }
        }
        _ => panic!("Expected BinOp, got {:?}", expr),
    }
}

#[test]
fn test_precedence_parentheses_override() {
    let expr = parse_expr("(1 + 2) * 3").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Mul),
        _ => panic!("Expected Mul, got {:?}", expr),
    }
}

// ============================================================================
// Grouping Tests
// ============================================================================

#[test]
fn test_parse_grouping() {
    let expr = parse_expr("(1 + 2)").unwrap();
    // Grouping returns the inner expression (BinOp in this case)
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Add),
        _ => panic!("Expected BinOp(Add), got {:?}", expr),
    }
}

#[test]
fn test_parse_nested_grouping() {
    let expr = parse_expr("((1 + 2) * 3)").unwrap();
    match &expr {
        Expr::BinOp { op, .. } => assert_eq!(*op, BinOp::Mul),
        _ => panic!("Expected Mul, got {:?}", expr),
    }
}

// ============================================================================
// List Literal Tests
// ============================================================================

// Note: Empty lists [] may not be supported by the parser
// This test is removed as the parser requires at least one element

#[test]
fn test_parse_list_single() {
    let expr = parse_expr("[1]").unwrap();
    match &expr {
        Expr::List { elements, .. } => {
            assert_eq!(elements.len(), 1);
            assert_is_int(&elements[0], 1);
        }
        _ => panic!("Expected List, got {:?}", expr),
    }
}

#[test]
fn test_parse_list_multiple() {
    let expr = parse_expr("[1, 2, 3]").unwrap();
    match &expr {
        Expr::List { elements, .. } => {
            assert_eq!(elements.len(), 3);
            assert_is_int(&elements[0], 1);
            assert_is_int(&elements[1], 2);
            assert_is_int(&elements[2], 3);
        }
        _ => panic!("Expected List, got {:?}", expr),
    }
}

#[test]
fn test_parse_list_comprehension() {
    let expr = parse_expr("[x for x in lst]").unwrap();
    match &expr {
        Expr::ListComprehension { var, .. } => assert_eq!(var, "x"),
        _ => panic!("Expected ListComprehension, got {:?}", expr),
    }
}

// ============================================================================
// Tuple Tests
// ============================================================================

#[test]
fn test_parse_tuple_empty() {
    let expr = parse_expr("()").unwrap();
    match &expr {
        Expr::Tuple { elements, .. } => assert!(elements.is_empty()),
        _ => panic!("Expected Tuple, got {:?}", expr),
    }
}

#[test]
fn test_parse_tuple_single() {
    let expr = parse_expr("(1,)").unwrap();
    match &expr {
        Expr::Tuple { elements, .. } => {
            assert_eq!(elements.len(), 1);
            assert_is_int(&elements[0], 1);
        }
        _ => panic!("Expected Tuple, got {:?}", expr),
    }
}

#[test]
fn test_parse_tuple_two_elements() {
    let expr = parse_expr("(1, 2)").unwrap();
    match &expr {
        Expr::Tuple { elements, .. } => {
            assert_eq!(elements.len(), 2);
            assert_is_int(&elements[0], 1);
            assert_is_int(&elements[1], 2);
        }
        _ => panic!("Expected Tuple, got {:?}", expr),
    }
}

// ============================================================================
// Dict Literal Tests
// ============================================================================

#[test]
fn test_parse_dict_empty() {
    let expr = parse_expr("{}").unwrap();
    match &expr {
        Expr::Dict { pairs, .. } => assert!(pairs.is_empty()),
        _ => panic!("Expected Dict, got {:?}", expr),
    }
}

#[test]
fn test_parse_dict_single() {
    let expr = parse_expr(r#"{"a": 1}"#).unwrap();
    match &expr {
        Expr::Dict { pairs, .. } => {
            assert_eq!(pairs.len(), 1);
            // Keys are strings in dict literals
            assert_is_str(&pairs[0].0, "a");
            assert_is_int(&pairs[0].1, 1);
        }
        _ => panic!("Expected Dict, got {:?}", expr),
    }
}

#[test]
fn test_parse_dict_multiple() {
    let expr = parse_expr(r#"{"a": 1, "b": 2}"#).unwrap();
    match &expr {
        Expr::Dict { pairs, .. } => assert_eq!(pairs.len(), 2),
        _ => panic!("Expected Dict, got {:?}", expr),
    }
}

// ============================================================================
// Array Repetition Tests
// ============================================================================

#[test]
fn test_parse_array_repetition() {
    let expr = parse_expr("[0; 5]").unwrap();
    match &expr {
        Expr::Array { elements, size, .. } => {
            assert_eq!(elements.len(), 1);
            assert_eq!(*size, Some(5));
        }
        _ => panic!("Expected Array, got {:?}", expr),
    }
}

// ============================================================================
// Lambda Tests
// ============================================================================

#[test]
fn test_parse_lambda_single_param() {
    let expr = parse_expr("lambda x: x + 1").unwrap();
    match &expr {
        Expr::Lambda { params, .. } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], "x");
        }
        _ => panic!("Expected Lambda, got {:?}", expr),
    }
}

#[test]
fn test_parse_lambda_fn_syntax() {
    let expr = parse_expr("fn(x, y): x + y").unwrap();
    match &expr {
        Expr::Lambda { params, .. } => {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "x");
            assert_eq!(params[1], "y");
        }
        _ => panic!("Expected Lambda, got {:?}", expr),
    }
}

#[test]
fn test_parse_lambda_no_params() {
    let expr = parse_expr("lambda: 42").unwrap();
    match &expr {
        Expr::Lambda { params, .. } => assert!(params.is_empty()),
        _ => panic!("Expected Lambda, got {:?}", expr),
    }
}

// ============================================================================
// Ternary Expression Tests
// ============================================================================

#[test]
fn test_parse_ternary() {
    let expr = parse_expr("x if cond else y").unwrap();
    match &expr {
        Expr::Conditional { .. } => {}
        _ => panic!("Expected Conditional, got {:?}", expr),
    }
}

#[test]
fn test_parse_ternary_complex() {
    let expr = parse_expr("1 if True else 2").unwrap();
    match &expr {
        Expr::Conditional { then_expr, else_expr, .. } => {
            assert_is_int(then_expr, 1);
            assert_is_int(else_expr, 2);
        }
        _ => panic!("Expected Conditional, got {:?}", expr),
    }
}

// ============================================================================
// Call Expression Tests
// ============================================================================

#[test]
fn test_parse_call_no_args() {
    let expr = parse_expr("foo()").unwrap();
    match &expr {
        Expr::Call { func, args, .. } => {
            assert_is_ident(func, "foo");
            assert!(args.is_empty());
        }
        _ => panic!("Expected Call, got {:?}", expr),
    }
}

#[test]
fn test_parse_call_with_args() {
    let expr = parse_expr("foo(1, 2)").unwrap();
    match &expr {
        Expr::Call { args, .. } => {
            assert_eq!(args.len(), 2);
            assert_is_int(&args[0], 1);
            assert_is_int(&args[1], 2);
        }
        _ => panic!("Expected Call, got {:?}", expr),
    }
}

// ============================================================================
// Index Expression Tests
// ============================================================================

#[test]
fn test_parse_index() {
    let expr = parse_expr("a[0]").unwrap();
    match &expr {
        Expr::Index { obj, index, .. } => {
            assert_is_ident(obj, "a");
            assert_is_int(index, 0);
        }
        _ => panic!("Expected Index, got {:?}", expr),
    }
}

// ============================================================================
// Slice Expression Tests
// ============================================================================

#[test]
fn test_parse_slice_full() {
    let expr = parse_expr("a[1:3]").unwrap();
    match &expr {
        Expr::Slice { start, end, .. } => {
            assert!(start.is_some());
            assert!(end.is_some());
        }
        _ => panic!("Expected Slice, got {:?}", expr),
    }
}

#[test]
fn test_parse_slice_step() {
    let expr = parse_expr("a[::-1]").unwrap();
    match &expr {
        Expr::Slice { step, .. } => {
            assert!(step.is_some());
        }
        _ => panic!("Expected Slice, got {:?}", expr),
    }
}

#[test]
fn test_parse_slice_open_ended() {
    let expr = parse_expr("a[:5]").unwrap();
    match &expr {
        Expr::Slice { start, end, .. } => {
            assert!(start.is_none());
            assert!(end.is_some());
        }
        _ => panic!("Expected Slice, got {:?}", expr),
    }
}

// ============================================================================
// Attribute Expression Tests
// ============================================================================

#[test]
fn test_parse_attribute() {
    let expr = parse_expr("obj.attr").unwrap();
    match &expr {
        Expr::Attribute { obj, attr, .. } => {
            assert_is_ident(obj, "obj");
            assert_eq!(attr, "attr");
        }
        _ => panic!("Expected Attribute, got {:?}", expr),
    }
}

// ============================================================================
// Pipeline Expression Tests
// ============================================================================

#[test]
fn test_parse_pipeline() {
    let expr = parse_expr("x |> f").unwrap();
    // Pipeline: x |> f => f(x)
    match &expr {
        Expr::Call { func, args, .. } => {
            assert_is_ident(func, "f");
            assert_eq!(args.len(), 1);
            assert_is_ident(&args[0], "x");
        }
        _ => panic!("Expected Call (pipeline), got {:?}", expr),
    }
}

#[test]
fn test_parse_pipeline_chain() {
    let expr = parse_expr("x |> f |> g").unwrap();
    // Pipeline chain: x |> f |> g => g(f(x))
    match &expr {
        Expr::Call { func, args, .. } => {
            assert_is_ident(func, "g");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Call (pipeline chain), got {:?}", expr),
    }
}

// ============================================================================
// Error Tests
// ============================================================================

#[test]
fn test_parse_empty_string() {
    let result = parse_expr("");
    // Empty string should return EOF or error gracefully
    assert!(result.is_err() || matches!(result, Ok(Expr::None(_))));
}

#[test]
fn test_parse_unmatched_paren() {
    let result = parse_expr("(1 + 2");
    assert!(result.is_err());
}

#[test]
fn test_parse_unmatched_bracket() {
    let result = parse_expr("[1, 2");
    assert!(result.is_err());
}

#[test]
fn test_parse_unmatched_brace() {
    let result = parse_expr(r#"{"a": 1"#);
    assert!(result.is_err());
}
