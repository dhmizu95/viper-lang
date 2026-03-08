//! String and type conversion code generation for Viper

use crate::ast::{Expr, Type};
use crate::codegen::state::CodeGenState;

use inkwell::values::BasicValueEnum;

use crate::codegen::expressions::calls::generate_bigint_to_str;
use crate::codegen::expressions::core::generate_expr;

/// Generate type conversion calls (float(), int(), bool())
pub fn generate_type_convert<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("{}() takes exactly 1 argument, got {}", name, args.len()));
    }

    let arg_val = generate_expr(state, &args[0])?;

    match name {
        "float" => {
            // Convert to float
            if arg_val.is_float_value() {
                Ok(arg_val)
            } else if arg_val.is_int_value() {
                let int_val = arg_val.into_int_value();
                let result = state
                    .builder
                    .build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
                    .expect("int to float conversion");
                Ok(result.into())
            } else if arg_val.is_pointer_value() {
                // Try string to float conversion
                let str_to_f64 = state
                    .module
                    .get_function("vp_f64_from_str")
                    .ok_or_else(|| "vp_f64_from_str not declared".to_string())?;
                let result = state
                    .ir_builder
                    .build_call(state.builder, str_to_f64, &[arg_val.into()], "str_to_f64")
                    .unwrap()
                    .into_float_value();
                Ok(result.into())
            } else {
                Err("Cannot convert to float".to_string())
            }
        }
        "int" => {
            // Convert to int (Python-style: arbitrary precision using tagged ints)
            if arg_val.is_int_value() {
                // Already an i64, convert to tagged int
                let from_i64_func = state
                    .module
                    .get_function("tagged_int_from_i64")
                    .ok_or_else(|| "tagged_int_from_i64 not declared".to_string())?;
                let result = state
                    .ir_builder
                    .build_call(state.builder, from_i64_func, &[arg_val.into()], "int_from_i64")
                    .unwrap();
                Ok(result)
            } else if arg_val.is_float_value() {
                // Float to int: first convert to i64, then to tagged int
                let float_val = arg_val.into_float_value();
                let int_val = state
                    .builder
                    .build_float_to_signed_int(float_val, state.context.i64_type(), "float_to_int")
                    .expect("float to int conversion");
                let from_i64_func = state
                    .module
                    .get_function("tagged_int_from_i64")
                    .ok_or_else(|| "tagged_int_from_i64 not declared".to_string())?;
                let result = state
                    .ir_builder
                    .build_call(state.builder, from_i64_func, &[int_val.into()], "int_from_float")
                    .unwrap();
                Ok(result)
            } else if arg_val.is_pointer_value() {
                // String to int (arbitrary precision) using tagged int
                let str_to_int = state
                    .module
                    .get_function("tagged_int_from_str")
                    .ok_or_else(|| "tagged_int_from_str not declared".to_string())?;
                let result = state
                    .ir_builder
                    .build_call(state.builder, str_to_int, &[arg_val.into()], "str_to_int")
                    .unwrap();
                Ok(result)
            } else {
                Err("Cannot convert to int".to_string())
            }
        }
        "bool" => {
            // Convert to bool (i1)
            if arg_val.is_int_value() {
                let int_val = arg_val.into_int_value();
                // Non-zero becomes true, zero becomes false
                let zero = state.context.i64_type().const_int(0, false);
                let result = state
                    .builder
                    .build_int_compare(inkwell::IntPredicate::NE, int_val, zero, "to_bool")
                    .expect("int to bool comparison");
                Ok(result.into())
            } else if arg_val.is_float_value() {
                let float_val = arg_val.into_float_value();
                // Non-zero becomes true, zero becomes false
                let zero = state.context.f64_type().const_float(0.0);
                let result = state
                    .builder
                    .build_float_compare(inkwell::FloatPredicate::ONE, float_val, zero, "to_bool")
                    .expect("float to bool comparison");
                Ok(result.into())
            } else if arg_val.is_pointer_value() {
                // For pointers: null is false, non-null is true
                let null_ptr =
                    state.context.ptr_type(inkwell::AddressSpace::default()).const_null();
                let ptr_as_int = state
                    .builder
                    .build_ptr_to_int(
                        arg_val.into_pointer_value(),
                        state.context.i64_type(),
                        "ptr_to_int",
                    )
                    .expect("ptr to int");
                let null_as_int = state
                    .builder
                    .build_ptr_to_int(null_ptr, state.context.i64_type(), "null_to_int")
                    .expect("null to int");
                let result = state
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::NE,
                        ptr_as_int,
                        null_as_int,
                        "ptr_to_bool",
                    )
                    .expect("ptr to bool comparison");
                Ok(result.into())
            } else {
                Err("Cannot convert to bool".to_string())
            }
        }
        _ => Err(format!("Unknown type conversion: {}", name)),
    }
}

/// Generate str() call - convert value to string (supports BigInt automatically)
pub fn generate_str_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("str() takes exactly 1 argument, got {}", args.len()));
    }

    let arg = &args[0];

    // Check if argument is BigInt type (use state-aware inference)
    let arg_type = crate::codegen::expressions::core::infer_type_with_state(state, arg);
    if arg_type == Type::BigInt {
        return generate_bigint_to_str(state, args);
    }

    // Check if argument is int type (which uses tagged int representation)
    if arg_type == Type::Int {
        let arg_val = generate_expr(state, arg)?;
        // Tagged ints are always i64 values (tagged with LSB)
        return generate_tagged_int_to_str_val(state, arg_val);
    }

    let arg_val = generate_expr(state, arg)?;

    let func_name = if arg_val.is_float_value() {
        "vp_str_from_f64"
    } else if arg_val.is_pointer_value() {
        return Ok(arg_val);
    } else {
        "vp_str_from_i64"
    };

    let str_func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result = state
        .ir_builder
        .build_call(state.builder, str_func, &[arg_val.into()], "str_result")
        .expect("str conversion call");

    Ok(result.into())
}

/// Generate str() for tagged int - handles both small ints and BigInt
fn generate_tagged_int_to_str<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("str() takes exactly 1 argument, got {}", args.len()));
    }

    let tagged_val = generate_expr(state, &args[0])?;

    // Call tagged_int_to_str which handles both small ints and BigInt
    let to_str_func = state
        .module
        .get_function("tagged_int_to_str")
        .ok_or_else(|| "tagged_int_to_str not declared".to_string())?;

    let str_val = state
        .ir_builder
        .build_call(state.builder, to_str_func, &[tagged_val.into()], "tagged_to_str")
        .expect("tagged_int_to_str call");

    Ok(str_val)
}

/// Generate str() for tagged int value
/// For now, just return the tagged int value directly - print() will handle it
fn generate_tagged_int_to_str_val<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    tagged_val: inkwell::values::BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Return the tagged int value directly
    // print() will use tagged_int_print to display it
    Ok(tagged_val)
}

/// Generate str() for BigInt pointer value (local variable)
fn generate_bigint_to_str_direct<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    bigint_ptr: inkwell::values::BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Call vp_bigint_to_str(bigint_ptr, 10) - base 10
    let to_str_func = state
        .module
        .get_function("vp_bigint_to_str")
        .ok_or_else(|| "vp_bigint_to_str not declared".to_string())?;

    let str_val = state
        .ir_builder
        .build_call(
            state.builder,
            to_str_func,
            &[
                bigint_ptr.into(),
                state.context.i32_type().const_int(10, false).into(),
            ],
            "bigint_to_str",
        )
        .expect("vp_bigint_to_str call");

    Ok(str_val)
}

/// Generate bytes() call
pub fn generate_bytes_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // bytes() with no args returns empty bytes
    if args.is_empty() {
        let bytes_func = state
            .module
            .get_function("vp_bytes_create")
            .ok_or_else(|| "vp_bytes_create not declared".to_string())?;
        
        let result = state
            .ir_builder
            .build_call(
                state.builder,
                bytes_func,
                &[
                    state.context.ptr_type(inkwell::AddressSpace::default()).const_null().into(),
                    state.context.i64_type().const_zero().into(),
                ],
                "bytes_result",
            )
            .expect("bytes call");
        
        return Ok(result.into());
    }
    
    // bytes(arg) - convert argument to bytes
    if args.len() != 1 {
        return Err(format!("bytes() takes at most 1 argument, got {}", args.len()));
    }
    
    let arg_val = generate_expr(state, &args[0])?;
    
    // For now, just return the argument as bytes (simplified)
    Ok(arg_val)
}
