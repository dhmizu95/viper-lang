//! Tests for utils::mangling module

use viper_lang::ast::Type;
use viper_lang::utils::mangle_function_name;

#[test]
fn test_mangle_no_params() {
    assert_eq!(mangle_function_name("foo", &[]), "foo");
}

#[test]
fn test_mangle_single_param() {
    assert_eq!(mangle_function_name("foo", &[Type::I64]), "foo_i64");
}

#[test]
fn test_mangle_multiple_params() {
    assert_eq!(mangle_function_name("add", &[Type::I64, Type::I64]), "add_i64_i64");
    assert_eq!(mangle_function_name("foo", &[Type::I64, Type::F64]), "foo_i64_f64");
}

#[test]
fn test_mangle_union_type() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert_eq!(mangle_function_name("process", &[union]), "process_union_int_str");
}

#[test]
fn test_mangle_union_multiple_variants() {
    let union = Type::Union(vec![Type::I64, Type::Str, Type::Bool]);
    assert_eq!(mangle_function_name("process", &[union]), "process_union_i64_str_bool");
}

#[test]
fn test_mangle_function_with_union_param() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert_eq!(mangle_function_name("process", &[union]), "process_union_int_str");
}
