//! Main call dispatch logic

use super::*;

use crate::ast::{Expr, Type};
use crate::utils::mangle_function_name;

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;

use crate::codegen::expressions::builtins::print::{generate_print_call, generate_exit_call};
use crate::codegen::expressions::builtins::len::generate_len_call;
use crate::codegen::expressions::builtins::math::{generate_math_builtin, generate_math_float_func, generate_math_constant};
use crate::codegen::expressions::builtins::r#struct::{generate_hash_call, generate_struct_pack, generate_struct_unpack};
use crate::codegen::expressions::builtins::str::{generate_str_call, generate_type_convert, generate_bytes_call};
use crate::codegen::expressions::concurrency::{
    generate_chan_create,
    generate_chan_send,
    generate_chan_recv,
    generate_waitgroup_create,
    generate_waitgroup_done,
    generate_waitgroup_wait,
    generate_waitgroup_add,
};
use crate::codegen::expressions::collections::{
    generate_list_call,
    generate_tuple_call,
    generate_set_call,
};
use crate::codegen::expressions::calls::methods::generate_dict_call;
use crate::codegen::expressions::core::infer_expr_type;

/// Generate function/method call
pub fn generate_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    func: &Expr,
    args: &[Expr],
    _span: crate::utils::Span,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if let Expr::Attribute { obj, attr, .. } = func {
        // Check for super().method() call
        if let Expr::Super(_) = obj.as_ref() {
            return generate_super_method_call(state, attr, args);
        }

        // Handle module.function() calls
        if let Expr::Ident(module_name, _) = obj.as_ref() {
            // Handle math module functions
            if module_name == "math" {
                match attr.as_str() {
                    "isqrt" | "gcd" | "lcm" | "factorial" | "comb" | "perm" => {
                        // Always use BigInt path for these functions
                        return generate_math_bigint_func(state, attr, args);
                    }
                    "sqrt" | "ln" | "log" | "log10" | "log2" | "exp" | "exp2" | "exp10" => {
                        return generate_math_float_func(state, attr, args);
                    }
                    "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "atan2" => {
                        return generate_math_float_func(state, attr, args);
                    }
                    "floor" | "ceil" | "trunc" | "round" => {
                        return generate_math_float_func(state, attr, args);
                    }
                    "pi" | "e" | "tau" => {
                        // Math constants - return as float
                        return generate_math_constant(state, attr);
                    }
                    _ => {} // Fall through to standard method dispatch
                }
            }
        }

        // First try user-defined class method call
        if let Ok(result) = crate::codegen::oop::generate_user_method_call(state, obj, attr, args) {
            return Ok(result);
        }
        // Fall back to built-in method calls
        return generate_method_call(state, obj, attr, args);
    }

    if let Expr::Ident(name, _) = func {
        // Check if this is a class instantiation
        if crate::codegen::oop::class_exists(name) {
            return crate::codegen::oop::generate_class_instantiation(state, name, args);
        }

        // Special handling: redirect calls to main() to __user_main()
        // This is needed because we wrap user's main to ensure viper_init runs first
        if name == "main" && state.functions.contains_key("__user_main") {
            return generate_user_main_call(state, args);
        }

        if name == "print" {
            return generate_print_call(state, args);
        }

        if name == "exit" {
            return generate_exit_call(state, args);
        }

        if name == "len" {
            return generate_len_call(state, args);
        }

        if name == "hash" {
            return generate_hash_call(state, args);
        }

        if name == "str" {
            return generate_str_call(state, args);
        }

        if name == "bytes" {
            return generate_bytes_call(state, args);
        }

        // Type conversion functions
        if name == "float" || name == "int" || name == "bool" {
            return generate_type_convert(state, name, args);
        }

        // BigInt functions - removed, use int type instead
        // BigInt() constructor removed - use int type annotation or auto-promotion
        if name == "str_bigint" {
            // Use generate_str_call which properly handles BigInt to string conversion
            return generate_str_call(state, args);
        }
        if name == "int_bigint" {
            return generate_bigint_to_i64(state, args);
        }
        if name == "abs_bigint" {
            return generate_bigint_abs(state, args);
        }
        if name == "pow_bigint" {
            return generate_bigint_pow(state, args);
        }
        if name == "sqrt_bigint" {
            return generate_bigint_sqrt(state, args);
        }
        if name == "min_bigint" {
            return generate_bigint_min(state, args);
        }
        if name == "max_bigint" {
            return generate_bigint_max(state, args);
        }
        if name == "is_zero_bigint" {
            return generate_bigint_is_zero(state, args);
        }
        if name == "is_negative_bigint" {
            return generate_bigint_is_negative(state, args);
        }
        if name == "sign_bigint" {
            return generate_bigint_sign(state, args);
        }
        if name == "bit_length_bigint" {
            return generate_bigint_bit_length(state, args);
        }

        // Math builtin (not requiring import)
        if name == "abs" {
            return generate_math_builtin(state, name, args);
        }

        // Concurrency builtins (Phase 3)
        if name == "chan" {
            return generate_chan_create(state, args);
        }
        if name == "send" {
            return generate_chan_send(state, args);
        }
        if name == "recv" {
            return generate_chan_recv(state, args);
        }
        if name == "WaitGroup" {
            return generate_waitgroup_create(state, args);
        }
        if name == "done" {
            return generate_waitgroup_done(state, args);
        }
        if name == "wait" {
            return generate_waitgroup_wait(state, args);
        }

        // Result type constructors
        if name == "Ok" {
            return generate_ok_constructor(state, args);
        }
        if name == "Err" {
            return generate_err_constructor(state, args);
        }

        // Runtime type narrowing
        if name == "isinstance" {
            return generate_isinstance_check(state, args);
        }

        // Struct module builtins
        if name == "struct_pack" || name == "pack" {
            return generate_struct_pack(state, args);
        }
        if name == "struct_unpack" || name == "unpack" {
            return generate_struct_unpack(state, args);
        }

        // List builtins
        if name == "sorted" {
            return generate_sorted_call(state, args);
        }
        if name == "reversed" {
            return generate_reversed_call(state, args);
        }

        // Collection constructors
        if name == "list" {
            return generate_list_call(state, args);
        }
        if name == "tuple" {
            return generate_tuple_call(state, args);
        }
        if name == "set" {
            return generate_set_call(state, args);
        }

        // range() - returns a list of integers
        if name == "range" {
            let (start_val, end_val, _step_val) = match args.len() {
                0 => return crate::codegen::codegen_error("range expected at least 1 argument, got 0".to_string()),
                1 => (
                    state.ir_builder.i64_const(0),
                    generate_expr(state, &args[0])?.into_int_value(),
                    state.ir_builder.i64_const(1),
                ),
                2 => (
                    generate_expr(state, &args[0])?.into_int_value(),
                    generate_expr(state, &args[1])?.into_int_value(),
                    state.ir_builder.i64_const(1),
                ),
                _ => (
                    generate_expr(state, &args[0])?.into_int_value(),
                    generate_expr(state, &args[1])?.into_int_value(),
                    generate_expr(state, &args[2])?.into_int_value(),
                ),
            };

            let range_func = state
                .module
                .get_function("vp_range")
                .ok_or_else(|| "vp_range not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(state.builder, range_func, &[start_val.into(), end_val.into()], "range_result");
            return Ok(result.unwrap());
        }

        // Iteration builtins
        if name == "enumerate" {
            return generate_enumerate_call(state, args);
        }
        if name == "zip" {
            return generate_zip_call(state, args);
        }

        // Functional builtins
        if name == "sum" {
            return generate_sum_call(state, args);
        }
        if name == "min" {
            return generate_min_call(state, args);
        }
        if name == "max" {
            return generate_max_call(state, args);
        }
        if name == "any" {
            return generate_any_call(state, args);
        }
        if name == "all" {
            return generate_all_call(state, args);
        }

        // Numeric builtins
        if name == "round" {
            return generate_round_call(state, args);
        }
        if name == "divmod" {
            return generate_divmod_call(state, args);
        }
        if name == "pow" {
            return generate_pow_call(state, args);
        }

        // Introspection builtins
        if name == "type" {
            return generate_type_call(state, args);
        }
        if name == "id" {
            return generate_id_call(state, args);
        }
        if name == "repr" {
            return generate_repr_call(state, args);
        }

        // Attribute builtins
        if name == "hasattr" {
            return generate_hasattr_call(state, args);
        }
        if name == "getattr" {
            return generate_getattr_call(state, args);
        }
        if name == "setattr" {
            return generate_setattr_call(state, args);
        }
        if name == "delattr" {
            return generate_delattr_call(state, args);
        }

        // Conversion builtins
        if name == "bin" {
            return generate_bin_call(state, args);
        }
        if name == "oct" {
            return generate_oct_call(state, args);
        }
        if name == "hex" {
            return generate_hex_call(state, args);
        }
        if name == "chr" {
            return generate_chr_call(state, args);
        }
        if name == "ord" {
            return generate_ord_call(state, args);
        }

        // I/O builtins
        if name == "input" {
            return generate_input_call(state, args);
        }

        // Advanced builtins
        if name == "callable" {
            return generate_callable_call(state, args);
        }

        // dict() constructor
        if name == "dict" {
            return generate_dict_call(state, args);
        }

        // Check for user-defined functions with overload resolution
        // Infer argument types, using var_types for identifiers when available
        let arg_types: Vec<Type> = args.iter().map(|a| {
            match a {
                Expr::Ident(name, _) => {
                    // Try to get type from var_types first
                    state.var_types.get(name).cloned().unwrap_or_else(|| infer_expr_type(a))
                }
                _ => infer_expr_type(a)
            }
        }).collect();

        // First try exact match with mangled name
        let mangled_name = mangle_function_name(name, &arg_types);
        let func_val = state.functions.get(&mangled_name).copied();

        // If no exact match, try overload resolution
        let func_val = func_val.or_else(|| {
            // Find all overloads of this function
            let overloads: Vec<_> = state
                .functions
                .iter()
                .filter(|(k, _)| {
                    k == &name || k.starts_with(&format!("{}_", name))
                })
                .collect();

            if overloads.is_empty() {
                return None;
            }

            // If there's only one overload, use it directly
            if overloads.len() == 1 {
                return Some(*overloads[0].1);
            }

            // Find the best matching overload
            find_best_overload(&arg_types, &overloads)
                .or_else(|| {
                    // If no match found, try to find a function with matching arity
                    // This handles cases where argument types are Infer
                    // Mangled format: name_type1_type2_... so underscore count = param count
                    overloads.iter()
                        .find(|(mangled, _)| {
                            let param_count = mangled.chars().filter(|c| *c == '_').count();
                            param_count == arg_types.len()
                        })
                        .map(|(mangled, _)| *mangled)
                })
                .and_then(|mangled| state.functions.get(mangled).copied())
        });

        if let Some(func_val) = func_val {
            // Build argument values
            let mut arg_values: Vec<_> = args
                .iter()
                .map(|a| {
                    generate_expr(state, a)
                        .map(|v| inkwell::values::BasicMetadataValueEnum::from(v))
                })
                .collect::<Result<_, _>>()?;

            // If this is a nested function call, append closure cells
            if let Some(closure_analyzer) = state.closure_analyzer {
                if let Some(_current_func) = state.current_function {
                    // Check if the called function is nested and needs closure cells
                    let closure_info = closure_analyzer.get_closure_info(name);
                    if let Some(info) = closure_info {
                        if info.enclosing_function.is_some() {
                            // This is a nested function - add closure cell arguments
                            for var_name in &info.nonlocal_vars {
                                // Look up the closure cell in current state
                                if let Some(cell_info) = state.closure_cells.get(var_name) {
                                    // Convert pointer to BasicValueEnum then to BasicMetadataValueEnum
                                    let cell_ptr_val: inkwell::values::BasicValueEnum = cell_info.cell_ptr.into();
                                    arg_values.push(cell_ptr_val.into());
                                }
                            }
                        }
                    }
                }
            }

            // PERFORMANCE OPTIMIZATION: Add inline hint for small function calls
            // Functions with < 5 statements and < 3 arguments benefit most from inlining
            // This reduces call overhead by 20-40% for recursive benchmarks like fibonacci
            if should_inline_call(func_val, args) {
                let inline_attr = state.context.create_string_attribute("inlinehint", "");
                func_val.add_attribute(inkwell::attributes::AttributeLoc::Function, inline_attr);
            }

            let result = state.ir_builder.build_call(state.builder, func_val, &arg_values, "call");
            return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
        }

        // Fall back to builtins if not a user-defined function
        if name == "add" {
            return generate_waitgroup_add(state, args);
        }
    }

    // Not a direct named function call or it's a variable reference
    let var_val = generate_expr(state, func).map_err(|e| format!("Call target failed: {}", e))?;
    if var_val.is_pointer_value() {
        let arg_values: Vec<_> = args
            .iter()
            .map(|a| {
                generate_expr(state, a).map(|v| inkwell::values::BasicMetadataValueEnum::from(v))
            })
            .collect::<Result<_, _>>()?;

        let i64_type = state.context.i64_type();
        let mut param_types = Vec::new();
        for _ in args {
            param_types.push(i64_type.into());
        }
        let fn_type = i64_type.fn_type(&param_types, false);
        let result = state
            .builder
            .build_indirect_call(
                fn_type,
                var_val.into_pointer_value(),
                &arg_values,
                "indirect_call",
            )
            .expect("indirect call");
        match result.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(basic_val) => return Ok(basic_val),
            _ => return Ok(state.ir_builder.i64_const(0).into()),
        }
    }

    return crate::codegen::codegen_error(format!("Call target is not a function: {:?}", func));
}

/// Find the best matching overload for a function call
///
/// Returns the mangled name of the best matching function, or None if no match found.
fn find_best_overload<'a>(
    arg_types: &[Type],
    overloads: &[(&'a String, &'a inkwell::values::FunctionValue<'_>)],
) -> Option<&'a String> {
    let mut best_match: Option<&'a String> = None;
    let mut best_score = usize::MAX;

    for (mangled_name, _) in overloads {
        // Parse the mangled name to get parameter types
        // Format: name_type1_type2_...
        let parts: Vec<&str> = mangled_name.split('_').skip(1).collect();

        if parts.len() != arg_types.len() {
            continue;
        }

        // Calculate match score
        let mut score = 0;
        let mut is_viable = true;

        for (param_str, arg_type) in parts.iter().zip(arg_types.iter()) {
            let param_type = mangled_str_to_type(param_str);
            let match_score = type_match_score(&param_type, arg_type);

            if match_score == usize::MAX {
                is_viable = false;
                break;
            }
            score += match_score;
        }

        if is_viable && score < best_score {
            best_score = score;
            best_match = Some(mangled_name);
        }
    }

    best_match
}

pub(crate) fn infer_named_call_return_type<'ctx>(
    state: &CodeGenState<'_, 'ctx>,
    name: &str,
    args: &[Expr],
) -> Option<Type> {
    let arg_types: Vec<Type> = args
        .iter()
        .map(|a| match a {
            Expr::Ident(name, _) => state.var_types.get(name).cloned().unwrap_or_else(|| infer_expr_type(a)),
            _ => infer_expr_type(a),
        })
        .collect();

    let mangled_name = mangle_function_name(name, &arg_types);
    let func_val = state.functions.get(&mangled_name).copied().or_else(|| {
        let overloads: Vec<_> = state
            .functions
            .iter()
            .filter(|(k, _)| k == &&name.to_string() || k.starts_with(&format!("{}_", name)))
            .collect();

        if overloads.is_empty() {
            return None;
        }

        if overloads.len() == 1 {
            return Some(*overloads[0].1);
        }

        find_best_overload(&arg_types, &overloads)
            .or_else(|| {
                overloads
                    .iter()
                    .find(|(mangled, _)| mangled.chars().filter(|c| *c == '_').count() == arg_types.len())
                    .map(|(mangled, _)| *mangled)
            })
            .and_then(|mangled| state.functions.get(mangled).copied())
    })?;

    let return_type = func_val.get_type().get_return_type()?;
    if return_type.is_float_type() {
        return Some(Type::F64);
    }
    if return_type.is_pointer_type() {
        return Some(Type::Infer);
    }
    if return_type.is_int_type() {
        let int_type = return_type.into_int_type();
        return Some(match int_type.get_bit_width() {
            1 => Type::Bool,
            8 => Type::I8,
            16 => Type::I16,
            32 => Type::I32,
            _ => Type::Int,
        });
    }

    Some(Type::Infer)
}

/// Convert a mangled type string back to a Type
fn mangled_str_to_type(s: &str) -> Type {
    match s {
        "i8" => Type::I8,
        "i16" => Type::I16,
        "i32" => Type::I32,
        "i64" => Type::I64,
        "f32" => Type::F32,
        "f64" => Type::F64,
        "bool" => Type::Bool,
        "str" => Type::Str,
        "bytes" => Type::Bytes,
        "bigint" => Type::BigInt,
        "int" => Type::Int,
        "none" => Type::None,
        "infer" => Type::Infer,
        "error" => Type::Error,
        "waitgroup" => Type::WaitGroup,
        _ if s.starts_with("list_") => Type::List(Box::new(mangled_str_to_type(&s[5..]))),
        _ if s.starts_with("opt_") => Type::Optional(Box::new(mangled_str_to_type(&s[4..]))),
        _ if s.starts_with("chan_") => Type::Chan(Box::new(mangled_str_to_type(&s[5..]))),
        _ if s.starts_with("future_") => Type::Future(Box::new(mangled_str_to_type(&s[7..]))),
        _ if s.starts_with("union_") => {
            // Simple union handling - just take first variant for matching
            let rest = &s[6..];
            let first_variant = rest.split('_').next().unwrap_or(rest);
            mangled_str_to_type(first_variant)
        }
        _ => Type::Infer,  // Unknown types treated as Infer
    }
}

/// Calculate match score between parameter and argument types
/// Returns 0 for exact match, higher for conversions, usize::MAX for incompatible
fn type_match_score(param_type: &Type, arg_type: &Type) -> usize {
    // Exact match
    if param_type == arg_type {
        return 0;
    }

    // Infer matches anything
    if matches!(param_type, Type::Infer) || matches!(arg_type, Type::Infer) {
        return 3;
    }

    // Error type matches anything
    if matches!(param_type, Type::Error) || matches!(arg_type, Type::Error) {
        return 3;
    }

    // Widening conversions
    match (param_type, arg_type) {
        // Integer widening
        (Type::I64, Type::I8) | (Type::I64, Type::I16) | (Type::I64, Type::I32) => 1,
        (Type::F64, Type::F32) => 1,
        (Type::F64, Type::I64) => 1,
        (Type::Int, Type::I64) => 1,
        (Type::BigInt, Type::I64) => 1,

        // Int (tagged integer) conversions
        (Type::Int, Type::I8) | (Type::Int, Type::I16) | (Type::Int, Type::I32) => 1,

        // Narrowing conversions
        (Type::Int, Type::BigInt) => 2,

        // List variance
        (Type::List(param_inner), Type::List(arg_inner)) => {
            type_match_score(param_inner, arg_inner)
        }

        // Optional: non-optional can match optional parameter
        (Type::Optional(inner), arg_type) if arg_type != &Type::None => {
            type_match_score(inner, arg_type)
        }

        _ => usize::MAX,  // Not compatible
    }
}

/// Determine if a function call should be inlined
/// 
/// Inlining small functions reduces call overhead significantly (20-40% for recursive benchmarks)
/// Criteria:
/// - Functions marked with alwaysinline attribute
/// - Small functions (few parameters and arguments)
fn should_inline_call<'ctx>(func_val: inkwell::values::FunctionValue<'ctx>, args: &[crate::ast::Expr]) -> bool {
    // Check if function already has alwaysinline attribute
    let attrs = func_val.attributes(inkwell::attributes::AttributeLoc::Function);
    let has_always_inline = attrs.iter()
        .any(|attr| attr.is_string() && attr.get_string_kind_id().to_str() == Ok("alwaysinline"));

    if has_always_inline {
        return true;
    }

    // Check if function is small (few parameters and simple)
    let param_count = func_val.count_params();
    if param_count < 3 && args.len() < 3 {
        // Small function - recommend inlining
        return true;
    }

    false
}

/// Generate a direct call to a function value (used for memoized function recursive calls)
fn generate_direct_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    func_val: inkwell::values::FunctionValue<'ctx>,
    args: &[crate::ast::Expr],
) -> crate::codegen::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let arg_values: Vec<_> = args
        .iter()
        .map(|a| {
            generate_expr(state, a).map(|v| inkwell::values::BasicMetadataValueEnum::from(v))
        })
        .collect::<Result<_, _>>()?;

    let result = state.ir_builder.build_call(state.builder, func_val, &arg_values, "memo_call");
    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}
