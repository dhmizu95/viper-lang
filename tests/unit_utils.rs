//! Unit tests for the Viper utils module
//! Tests for: Span (new, empty, merge, Display), mangle_function_name

use viper_lang::ast::Type;
use viper_lang::utils::{mangle_function_name, Span};

// ============================================================================
// Span Tests
// ============================================================================

#[test]
fn test_span_new() {
    let span = Span::new(10, 20, 5, 3);
    assert_eq!(span.start, 10);
    assert_eq!(span.end, 20);
    assert_eq!(span.line, 5);
    assert_eq!(span.column, 3);
}

#[test]
fn test_span_empty() {
    let span = Span::empty(5, 3);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
    assert_eq!(span.line, 5);
    assert_eq!(span.column, 3);
}

#[test]
fn test_span_merge() {
    let span1 = Span::new(10, 20, 5, 3);
    let span2 = Span::new(15, 30, 6, 1);
    
    let merged = span1.merge(span2);
    
    assert_eq!(merged.start, 10); // min start
    assert_eq!(merged.end, 30);   // max end
    assert_eq!(merged.line, 5);   // min line
    assert_eq!(merged.column, 1); // min column
}

#[test]
fn test_span_merge_same() {
    let span1 = Span::new(10, 20, 5, 3);
    let span2 = Span::new(10, 20, 5, 3);
    
    let merged = span1.merge(span2);
    
    assert_eq!(merged.start, 10);
    assert_eq!(merged.end, 20);
    assert_eq!(merged.line, 5);
    assert_eq!(merged.column, 3);
}

#[test]
fn test_span_merge_reverse() {
    let span1 = Span::new(15, 30, 6, 1);
    let span2 = Span::new(10, 20, 5, 3);
    
    let merged = span1.merge(span2);
    
    assert_eq!(merged.start, 10); // min start
    assert_eq!(merged.end, 30);   // max end
    assert_eq!(merged.line, 5);   // min line
    assert_eq!(merged.column, 1); // min column
}

#[test]
fn test_span_display() {
    let span = Span::new(10, 20, 5, 3);
    let display = format!("{}", span);
    assert_eq!(display, "5:3:10"); // line:column:length
}

#[test]
fn test_span_display_empty() {
    let span = Span::empty(5, 3);
    let display = format!("{}", span);
    assert_eq!(display, "5:3:0");
}

#[test]
fn test_span_default() {
    let span = Span::default();
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
    assert_eq!(span.line, 0);
    assert_eq!(span.column, 0);
}

#[test]
fn test_span_copy_clone() {
    let span1 = Span::new(10, 20, 5, 3);
    let span2 = span1; // Copy
    let span3 = span1.clone(); // Clone
    
    assert_eq!(span1.start, span2.start);
    assert_eq!(span1.start, span3.start);
}

#[test]
fn test_span_partial_eq() {
    let span1 = Span::new(10, 20, 5, 3);
    let span2 = Span::new(10, 20, 5, 3);
    let span3 = Span::new(10, 21, 5, 3);
    
    assert_eq!(span1, span2);
    assert_ne!(span1, span3);
}

// ============================================================================
// Name Mangling Tests
// ============================================================================

#[test]
fn test_mangle_no_params() {
    assert_eq!(mangle_function_name("foo", &[]), "foo");
    assert_eq!(mangle_function_name("main", &[]), "main");
    assert_eq!(mangle_function_name("print_value", &[]), "print_value");
}

#[test]
fn test_mangle_single_param() {
    assert_eq!(mangle_function_name("foo", &[Type::I64]), "foo_i64");
    assert_eq!(mangle_function_name("print", &[Type::I64]), "print_i64");
    assert_eq!(mangle_function_name("square", &[Type::F64]), "square_f64");
}

#[test]
fn test_mangle_multiple_params() {
    assert_eq!(mangle_function_name("add", &[Type::I64, Type::I64]), "add_i64_i64");
    assert_eq!(mangle_function_name("foo", &[Type::I64, Type::F64]), "foo_i64_f64");
    assert_eq!(mangle_function_name("bar", &[Type::Bool, Type::Str]), "bar_bool_str");
}

#[test]
fn test_mangle_all_primitive_types() {
    assert_eq!(mangle_function_name("fn", &[Type::I8]), "fn_i8");
    assert_eq!(mangle_function_name("fn", &[Type::I16]), "fn_i16");
    assert_eq!(mangle_function_name("fn", &[Type::I32]), "fn_i32");
    assert_eq!(mangle_function_name("fn", &[Type::I64]), "fn_i64");
    assert_eq!(mangle_function_name("fn", &[Type::BigInt]), "fn_bigint");
    assert_eq!(mangle_function_name("fn", &[Type::F32]), "fn_f32");
    assert_eq!(mangle_function_name("fn", &[Type::F64]), "fn_f64");
    assert_eq!(mangle_function_name("fn", &[Type::Bool]), "fn_bool");
    assert_eq!(mangle_function_name("fn", &[Type::Str]), "fn_str");
    assert_eq!(mangle_function_name("fn", &[Type::None]), "fn_none");
}

#[test]
fn test_mangle_list_type() {
    assert_eq!(
        mangle_function_name("process", &[Type::List(Box::new(Type::I64))]),
        "process_list_i64"
    );
    assert_eq!(
        mangle_function_name("process", &[Type::List(Box::new(Type::Str))]),
        "process_list_str"
    );
}

#[test]
fn test_mangle_dict_type() {
    assert_eq!(
        mangle_function_name("lookup", &[Type::Dict(Box::new(Type::Str), Box::new(Type::I64))]),
        "lookup_dict_str_i64"
    );
}

#[test]
fn test_mangle_tuple_type() {
    assert_eq!(
        mangle_function_name("unpack", &[Type::Tuple(vec![Type::I64, Type::Str])]),
        "unpack_tuplei64_str"
    );
    assert_eq!(
        mangle_function_name("unpack", &[Type::Tuple(vec![Type::I64])]),
        "unpack_tuplei64"
    );
}

#[test]
fn test_mangle_array_type() {
    assert_eq!(
        mangle_function_name("process", &[Type::Array(Box::new(Type::I64), 5)]),
        "process_array_i64_5"
    );
    assert_eq!(
        mangle_function_name("process", &[Type::Array(Box::new(Type::F64), 10)]),
        "process_array_f64_10"
    );
}

#[test]
fn test_mangle_fn_type() {
    assert_eq!(
        mangle_function_name("apply", &[Type::Fn(vec![Type::I64], Box::new(Type::I64))]),
        "apply_fn_i64_ret_i64"
    );
    assert_eq!(
        mangle_function_name("apply", &[Type::Fn(vec![Type::I64, Type::I64], Box::new(Type::I64))]),
        "apply_fn_i64_i64_ret_i64"
    );
}

#[test]
fn test_mangle_chan_type() {
    assert_eq!(
        mangle_function_name("send", &[Type::Chan(Box::new(Type::I64))]),
        "send_chan_i64"
    );
    assert_eq!(
        mangle_function_name("recv", &[Type::Chan(Box::new(Type::Str))]),
        "recv_chan_str"
    );
}

#[test]
fn test_mangle_optional_type() {
    assert_eq!(
        mangle_function_name("unwrap", &[Type::Optional(Box::new(Type::I64))]),
        "unwrap_opt_i64"
    );
    assert_eq!(
        mangle_function_name("unwrap", &[Type::Optional(Box::new(Type::Str))]),
        "unwrap_opt_str"
    );
}

#[test]
fn test_mangle_waitgroup_type() {
    assert_eq!(
        mangle_function_name("wait", &[Type::WaitGroup]),
        "wait_waitgroup"
    );
}

#[test]
fn test_mangle_struct_type() {
    assert_eq!(
        mangle_function_name("process", &[Type::Struct {
            name: "Person".to_string(),
            fields: vec![("name".to_string(), Type::Str)]
        }]),
        "process_struct_Person"
    );
}

#[test]
fn test_mangle_future_type() {
    assert_eq!(
        mangle_function_name("await", &[Type::Future(Box::new(Type::I64))]),
        "await_future_i64"
    );
}

#[test]
fn test_mangle_var_type() {
    assert_eq!(
        mangle_function_name("generic", &[Type::Var("T".to_string())]),
        "generic_var_T"
    );
}

#[test]
fn test_mangle_infer_type() {
    assert_eq!(
        mangle_function_name("infer", &[Type::Infer]),
        "infer_infer"
    );
}

#[test]
fn test_mangle_error_type() {
    assert_eq!(
        mangle_function_name("error_fn", &[Type::Error]),
        "error_fn_error"
    );
}

#[test]
fn test_mangle_complex_mixed_params() {
    let params = vec![
        Type::I64,
        Type::List(Box::new(Type::Str)),
        Type::Dict(Box::new(Type::Str), Box::new(Type::I64)),
        Type::Optional(Box::new(Type::Bool)),
    ];
    assert_eq!(
        mangle_function_name("complex", &params),
        "complex_i64_list_str_dict_str_i64_opt_bool"
    );
}

#[test]
fn test_mangle_nested_types() {
    let params = vec![
        Type::List(Box::new(Type::List(Box::new(Type::I64)))),
        Type::Dict(
            Box::new(Type::Str),
            Box::new(Type::List(Box::new(Type::I64))),
        ),
    ];
    assert_eq!(
        mangle_function_name("nested", &params),
        "nested_list_list_i64_dict_str_list_i64"
    );
}
