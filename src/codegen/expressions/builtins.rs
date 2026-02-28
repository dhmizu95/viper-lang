//! Expression code generation for Viper

use super::*;

use crate::ast::{Expr, Type};
use crate::codegen::variables::VarType;

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;
use crate::codegen::expressions::calls::generate_bigint_to_str;

/// Generate print call - handles multiple arguments
pub fn generate_print_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        let newline_func = state
            .module
            .get_function("vp_print_newline")
            .ok_or_else(|| "vp_print_newline not declared".to_string())?;
        state.builder.build_call(newline_func, &[], "print_newline").expect("vp_print_newline");
        return Ok(state.ir_builder.i64_const(0).into());
    }

    // Print each argument
    for (i, arg) in args.iter().enumerate() {
        let val = generate_expr(state, arg)?;

        if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 64 {
            let print_func = state
                .module
                .get_function("vp_print_i64")
                .ok_or_else(|| "vp_print_i64 not declared".to_string())?;
            state.builder.build_call(print_func, &[val.into()], "print_i64").expect("vp_print_i64");
        } else if val.is_float_value() {
            let print_func = state
                .module
                .get_function("vp_print_f64")
                .ok_or_else(|| "vp_print_f64 not declared".to_string())?;
            state.builder.build_call(print_func, &[val.into()], "print_f64").expect("vp_print_f64");
        } else if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
            let print_func = state
                .module
                .get_function("vp_print_bool")
                .ok_or_else(|| "vp_print_bool not declared".to_string())?;
            state
                .builder
                .build_call(print_func, &[val.into()], "print_bool")
                .expect("vp_print_bool");
        } else if val.is_pointer_value() {
            // Check if this is a list - if so, use vp_list_print
            let is_list_arg = match arg {
                Expr::Ident(name, _) => state.is_list(name),
                Expr::List { .. } | Expr::ListComprehension { .. } => true,
                _ => false,
            };

            // Check if this is a dict
            let is_dict_arg = match arg {
                Expr::Ident(name, _) => state.is_dict(name),
                Expr::Dict { .. } => true,
                _ => false,
            };

            // Check if this is a bytes literal
            let is_bytes_arg = match arg {
                Expr::Bytes(_, _) => true,
                Expr::Ident(name, _) => {
                    state.variables.get(name).map_or(false, |v| v.var_type == VarType::Bytes)
                }
                _ => false,
            };

            // Check if BigInt - check variable, expression type, or if it's a pointer from a BigInt function
            let is_bigint_arg = match arg {
                Expr::Ident(name, _) => state.is_bigint(name),
                Expr::BigInt(_, _) => true,
                Expr::Call { func, .. } => {
                    // Check if calling a known BigInt function or if result is a pointer
                    if let Expr::Ident(func_name, _) = func.as_ref() {
                        func_name == "BigInt" || func_name == "abs_bigint" || func_name == "pow_bigint" 
                            || func_name == "sqrt_bigint" || func_name == "min_bigint" || func_name == "max_bigint"
                            || val.is_pointer_value()  // User-defined BigInt function
                    } else {
                        val.is_pointer_value()
                    }
                }
                _ => {
                    val.is_pointer_value() && infer_expr_type(arg) == Type::BigInt
                }
            };

            if is_bigint_arg {
                // Convert BigInt to string and print
                let to_str_func = state
                    .module
                    .get_function("vp_bigint_to_str")
                    .ok_or_else(|| "vp_bigint_to_str not declared".to_string())?;
                
                let base = state.context.i32_type().const_int(10, false);
                let str_val = state
                    .ir_builder
                    .build_call(state.builder, to_str_func, &[val.into(), base.into()], "str_conv")
                    .expect("vp_bigint_to_str");

                let print_func = state
                    .module
                    .get_function("vp_print_str")
                    .ok_or_else(|| "vp_print_str not declared".to_string())?;
                state
                    .builder
                    .build_call(print_func, &[str_val.into()], "print_bigint_str")
                    .expect("vp_print_str");
            } else if is_bytes_arg {
                // Print bytes using vp_bytes_print
                let print_func = state
                    .module
                    .get_function("vp_bytes_print")
                    .ok_or_else(|| "vp_bytes_print not declared".to_string())?;
                state
                    .builder
                    .build_call(print_func, &[val.into()], "print_bytes")
                    .expect("vp_bytes_print");
            } else if is_dict_arg {
                let print_func = state
                    .module
                    .get_function("vp_dict_print")
                    .ok_or_else(|| "vp_dict_print not declared".to_string())?;
                state
                    .builder
                    .build_call(print_func, &[val.into()], "print_dict")
                    .expect("vp_dict_print");
            } else if is_list_arg {
                let print_func = state
                    .module
                    .get_function("vp_list_print")
                    .ok_or_else(|| "vp_list_print not declared".to_string())?;
                state
                    .builder
                    .build_call(print_func, &[val.into()], "print_list")
                    .expect("vp_list_print");
            } else {
                let print_func = state
                    .module
                    .get_function("vp_print_str")
                    .ok_or_else(|| "vp_print_str not declared".to_string())?;
                state
                    .builder
                    .build_call(print_func, &[val.into()], "print_str")
                    .expect("vp_print_str");
            }
        } else {
            return Err(format!("print() does not support type {:?}", val.get_type()));
        }

        // Add space between arguments (but not after the last one)
        if i < args.len() - 1 {
            let print_func = state
                .module
                .get_function("vp_print_str")
                .ok_or_else(|| "vp_print_str not declared".to_string())?;
            let space_str_const = state.ir_builder.string_const(state.module, " ");
            let create_func = state
                .module
                .get_function("vp_str_create")
                .ok_or_else(|| "vp_str_create not declared".to_string())?;
            let space_val = state
                .ir_builder
                .build_call(state.builder, create_func, &[space_str_const.into()], "space_create")
                .unwrap();
            state
                .builder
                .build_call(print_func, &[space_val.into()], "print_space")
                .expect("vp_print_str");
        }
    }

    // Print newline at the end
    let newline_func = state
        .module
        .get_function("vp_print_newline")
        .ok_or_else(|| "vp_print_newline not declared".to_string())?;
    state.builder.build_call(newline_func, &[], "print_newline").expect("vp_print_newline");

    return Ok(state.ir_builder.i64_const(0).into());
}

/// Generate len() call
pub fn generate_len_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("len() takes exactly 1 argument, got {}", args.len()));
    }

    let obj_expr = &args[0];
    let obj_val = generate_expr(state, obj_expr)?;

    // Check if it's a list (literal, variable, or list repetition)
    let is_list = match obj_expr {
        Expr::List { .. } | Expr::Array { .. } | Expr::ListComprehension { .. } => true,
        Expr::Ident(name, _) => state.is_list(name),
        // Check for list repetition: [elem] * n
        Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
            matches!(left.as_ref(), Expr::List { .. } | Expr::Array { .. })
        }
        _ => false,
    };

    // Check if it's a bool list (bit vector)
    let is_bool_list = match obj_expr {
        Expr::Ident(name, _) => state.is_bool_list(name),
        Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
        Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
            if let Expr::List { elements, .. } = left.as_ref() {
                elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
            } else {
                false
            }
        }
        _ => false,
    };

    // Call the appropriate length function
    if is_bool_list {
        // Use bit vector len for bool lists
        let bitvec_len = state
            .module
            .get_function("vp_bitvec_len")
            .ok_or_else(|| "vp_bitvec_len not declared".to_string())?;
        let result =
            state.ir_builder.build_call(state.builder, bitvec_len, &[obj_val.into()], "bitvec_len");
        return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
    } else if is_list {
        // Call vp_list_len for other lists
        let list_len = state
            .module
            .get_function("vp_list_len")
            .ok_or_else(|| "vp_list_len not declared".to_string())?;
        let result =
            state.ir_builder.build_call(state.builder, list_len, &[obj_val.into()], "list_len");
        return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
    }

    // Otherwise treat as string (for string literals or variables)
    if obj_val.is_pointer_value() {
        // Check if it's a BigInt variable
        let is_bigint = match obj_expr {
            Expr::Ident(name, _) => state.is_bigint(name),
            Expr::BigInt(..) => true,
            _ => false,
        };
        
        if is_bigint {
            // For BigInt, convert to string first and get length
            let to_str_func = state
                .module
                .get_function("vp_bigint_to_str")
                .ok_or_else(|| "vp_bigint_to_str not declared".to_string())?;
            let base = state.context.i32_type().const_int(10, false);
            let str_val = state
                .ir_builder
                .build_call(state.builder, to_str_func, &[obj_val.into(), base.into()], "bigint_to_str_for_len")
                .expect("bigint_to_str");
            
            let str_len = state
                .module
                .get_function("vp_str_len")
                .ok_or_else(|| "vp_str_len not declared".to_string())?;
            let result =
                state.ir_builder.build_call(state.builder, str_len, &[str_val.into()], "str_len");
            return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
        }
        
        // Call vp_str_len for strings
        let str_len = state
            .module
            .get_function("vp_str_len")
            .ok_or_else(|| "vp_str_len not declared".to_string())?;
        let result =
            state.ir_builder.build_call(state.builder, str_len, &[obj_val.into()], "str_len");
        return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
    }

    // Fallback: treat as list
    let list_len = state
        .module
        .get_function("vp_list_len")
        .ok_or_else(|| "vp_list_len not declared".to_string())?;
    let result =
        state.ir_builder.build_call(state.builder, list_len, &[obj_val.into()], "list_len");
    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

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
            } else {
                Err("Cannot convert to float".to_string())
            }
        }
        "int" => {
            // Convert to int
            if arg_val.is_int_value() {
                Ok(arg_val)
            } else if arg_val.is_float_value() {
                let float_val = arg_val.into_float_value();
                let result = state
                    .builder
                    .build_float_to_signed_int(float_val, state.context.i64_type(), "float_to_int")
                    .expect("float to int conversion");
                Ok(result.into())
            } else if arg_val.is_pointer_value() {
                // Try string to int conversion
                let str_to_int = state
                    .module
                    .get_function("vp_str_to_i64")
                    .ok_or_else(|| "vp_str_to_i64 not declared".to_string())?;
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

    // Check if argument is BigInt type
    let arg_type = crate::codegen::expressions::core::infer_expr_type(arg);
    if arg_type == Type::BigInt {
        return generate_bigint_to_str(state, args);
    }
    
    // Also check if it's an identifier that holds a BigInt
    if let Expr::Ident(name, _) = arg {
        if state.is_bigint(name) {
            return generate_bigint_to_str(state, args);
        }
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
        .build_call(state.builder, str_func, &[arg_val.into()], "str_conv")
        .expect("str conversion call");

    Ok(result.into())
}

/// Generate math builtin function calls (sqrt, abs, ln, floor)
pub fn generate_math_builtin<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("{}() takes exactly 1 argument, got {}", name, args.len()));
    }

    let arg_val = generate_expr(state, &args[0])?;

    // Convert to float if necessary
    let arg_float = if arg_val.is_float_value() {
        arg_val.into_float_value()
    } else {
        let int_val = arg_val.into_int_value();
        state
            .builder
            .build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion")
    };

    let func_name = match name {
        "sqrt" => "vp_math_sqrt",
        "abs" => "vp_math_abs",
        "ln" => "vp_math_ln",
        "floor" => "vp_math_floor",
        _ => return Err(format!("Unknown math builtin: {}", name)),
    };

    let math_func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result =
        state.ir_builder.build_call(state.builder, math_func, &[arg_float.into()], "math_result");
    Ok(result.unwrap_or(state.ir_builder.f64_const(0.0).into()))
}

/// Generate struct.pack call
pub fn generate_struct_pack<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() < 2 {
        return Err("struct.pack requires at least 2 arguments (format, value)".to_string());
    }

    // Generate format string (first arg)
    let format_expr = &args[0];
    let format_val = generate_expr(state, format_expr)?;
    let format_ptr = format_val.into_pointer_value();

    // Generate value (second arg)
    let value_expr = &args[1];
    let value_val = generate_expr(state, value_expr)?;
    let value_int = if value_val.is_int_value() {
        value_val.into_int_value()
    } else if value_val.is_float_value() {
        let float_val = value_val.into_float_value();
        state
            .builder
            .build_float_to_signed_int(float_val, state.context.i64_type(), "float_to_int")
            .expect("float to int")
    } else {
        return Err("Unsupported type for struct.pack".to_string());
    };

    let struct_pack_func = state
        .module
        .get_function("vp_struct_pack")
        .ok_or_else(|| "vp_struct_pack not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        struct_pack_func,
        &[format_ptr.into(), value_int.into()],
        "struct_pack_result",
    );

    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

/// Generate struct.unpack call
pub fn generate_struct_unpack<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() < 2 {
        return Err("struct.unpack requires at least 2 arguments (format, data)".to_string());
    }

    // Generate format string (first arg)
    let format_expr = &args[0];
    let format_val = generate_expr(state, format_expr)?;
    let format_ptr = format_val.into_pointer_value();

    // Generate data pointer (second arg)
    let data_expr = &args[1];
    let data_val = generate_expr(state, data_expr)?;
    let data_ptr = data_val.into_pointer_value();

    let struct_unpack_func = state
        .module
        .get_function("vp_struct_unpack")
        .ok_or_else(|| "vp_struct_unpack not declared".to_string())?;

    let len_val = state.context.i64_type().const_int(0, false);

    let result = state.ir_builder.build_call(
        state.builder,
        struct_unpack_func,
        &[format_ptr.into(), data_ptr.into(), len_val.into()],
        "struct_unpack_result",
    );

    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

/// Generate hash() call - returns hash value for hashable types
pub fn generate_hash_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("hash() takes exactly 1 argument, got {}", args.len()));
    }

    let arg = &args[0];
    let arg_val = generate_expr(state, arg)?;

    // Choose the appropriate hash function based on the type
    let hash_func_name = if arg_val.is_float_value() {
        "vp_hash_f64"
    } else if arg_val.is_int_value() && arg_val.get_type().into_int_type().get_bit_width() == 1 {
        // Check for bool (i1) before general i64
        "vp_hash_bool"
    } else if arg_val.is_int_value() {
        "vp_hash_i64"
    } else if arg_val.is_pointer_value() {
        // String or other pointer type
        "vp_hash_str"
    } else {
        return Err(format!("hash() not supported for type {:?}", arg_val.get_type()));
    };

    let hash_func = state
        .module
        .get_function(hash_func_name)
        .ok_or_else(|| format!("{} not declared", hash_func_name))?;

    let result =
        state.ir_builder.build_call(state.builder, hash_func, &[arg_val.into()], "hash_result");

    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}
