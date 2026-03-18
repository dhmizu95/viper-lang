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
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!(
            "{}() takes exactly 1 argument, got {}",
            name,
            args.len()
        ));
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
                crate::codegen::codegen_error("Cannot convert to float".to_string())
            }
        }
        "int" => {
            // Check type with state to be more accurate
            let arg_type =
                crate::codegen::expressions::core::infer_type_with_state(state, &args[0]);

            // Convert to int (Python-style: arbitrary precision using tagged ints)
            if arg_val.is_int_value()
                && (arg_type == Type::Int || arg_type == Type::I64 || arg_type == Type::Bool)
            {
                // Already a tagged int (or bool), return as-is (bool will be 0/1 tagged)
                Ok(arg_val)
            } else if arg_type == Type::BigInt {
                // BigInt to tagged int: Untag the pointer first, call vp_bigint_to_i64, then tag the result
                let to_i64_func = state
                    .module
                    .get_function("vp_bigint_to_i64")
                    .ok_or_else(|| "vp_bigint_to_i64 not declared".to_string())?;
                // Untag the BigInt pointer: bigint_ptr = value & ~1
                let untagged_ptr = state
                    .builder
                    .build_and(
                        arg_val.into_int_value(),
                        state.context.i64_type().const_int(!1u64, false),
                        "untagged_bigint_ptr",
                    )
                    .expect("untag bigint ptr");
                let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                let bigint_ptr = state
                    .builder
                    .build_int_to_ptr(untagged_ptr, ptr_type, "bigint_ptr")
                    .expect("i64 to ptr");
                let untagged = state
                    .ir_builder
                    .build_call(state.builder, to_i64_func, &[bigint_ptr.into()], "bigint_to_i64")
                    .unwrap()
                    .into_int_value();
                let tagged = state.ir_builder.build_tag_i64(state.builder, untagged);
                Ok(tagged.into())
            } else if arg_val.is_float_value() {
                // Float to int: first convert to i64, then to tagged int
                let float_val = arg_val.into_float_value();
                let int_val = state
                    .builder
                    .build_float_to_signed_int(float_val, state.context.i64_type(), "float_to_int")
                    .expect("float to int conversion");
                let from_i64_func = state
                    .module
                    .get_function("tagged_int_from_i64_export")
                    .ok_or_else(|| "tagged_int_from_i64 not declared".to_string())?;
                let result = state
                    .ir_builder
                    .build_call(state.builder, from_i64_func, &[int_val.into()], "int_from_float")
                    .unwrap();
                Ok(result)
            } else if arg_val.is_pointer_value() || arg_type == Type::Str || arg_type == Type::Infer
            {
                // String to int (arbitrary precision) using tagged int
                let str_to_int = state
                    .module
                    .get_function("tagged_int_from_str")
                    .ok_or_else(|| "tagged_int_from_str not declared".to_string())?;

                // If it's a pointer to ViperString, we MUST extract the char* data first
                let char_ptr = if arg_val.is_pointer_value() {
                    let str_data_func = state
                        .module
                        .get_function("vp_str_data")
                        .ok_or_else(|| "vp_str_data not declared".to_string())?;
                    state
                        .ir_builder
                        .build_call(state.builder, str_data_func, &[arg_val.into()], "str_data")
                        .unwrap()
                        .into_pointer_value()
                } else {
                    // It's an i64 but might be a pointer (common in untyped collections)
                    // We check if it's LSB=0 (could be small int or pointer)
                    // For now, assume it's a pointer if we got here and type is Str/Infer
                    let str_data_func = state
                        .module
                        .get_function("vp_str_data")
                        .ok_or_else(|| "vp_str_data not declared".to_string())?;
                    let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                    let arg_ptr = state
                        .builder
                        .build_int_to_ptr(arg_val.into_int_value(), ptr_type, "i64_to_ptr")
                        .unwrap();
                    state
                        .ir_builder
                        .build_call(state.builder, str_data_func, &[arg_ptr.into()], "str_data")
                        .unwrap()
                        .into_pointer_value()
                };

                let result = state
                    .ir_builder
                    .build_call(state.builder, str_to_int, &[char_ptr.into()], "str_to_int")
                    .unwrap();
                Ok(result)
            } else {
                crate::codegen::codegen_error(format!("Cannot convert {:?} to int", arg_type))
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
                crate::codegen::codegen_error("Cannot convert to bool".to_string())
            }
        }
        _ => crate::codegen::codegen_error(format!("Unknown type conversion: {}", name)),
    }
}

/// Generate str() call - convert value to string (supports BigInt automatically)
pub fn generate_str_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!(
            "str() takes exactly 1 argument, got {}",
            args.len()
        ));
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
        // Create an array with the argument to pass to generate_tagged_int_to_str
        // Or directly call the generation logic here to avoid arg repackaging:

        let to_str_func = state
            .module
            .get_function("tagged_int_to_viper_str")
            .ok_or_else(|| "tagged_int_to_viper_str not declared".to_string())?;

        let str_val = state
            .ir_builder
            .build_call(state.builder, to_str_func, &[arg_val.into()], "tagged_to_viper_str")
            .expect("tagged_int_to_viper_str call");

        return Ok(str_val.into());
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

/// Generate str() for BigInt pointer value (local variable)
fn generate_bigint_to_str_direct<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    bigint_ptr: inkwell::values::BasicValueEnum<'ctx>,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
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
            &[bigint_ptr.into(), state.context.i32_type().const_int(10, false).into()],
            "bigint_to_str",
        )
        .expect("vp_bigint_to_str call");

    Ok(str_val)
}

/// Generate bytes() call
pub fn generate_bytes_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
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
        return crate::codegen::codegen_error(format!(
            "bytes() takes at most 1 argument, got {}",
            args.len()
        ));
    }

    let arg_val = generate_expr(state, &args[0])?;

    // For now, just return the argument as bytes (simplified)
    Ok(arg_val)
}
