//! Expression code generation for Viper

use super::*;

use crate::ast::Expr;

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;


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
        state
            .builder
            .build_call(newline_func, &[], "print_newline")
            .expect("vp_print_newline");
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
            state
                .builder
                .build_call(print_func, &[val.into()], "print_i64")
                .expect("vp_print_i64");
        } else if val.is_float_value() {
            let print_func = state
                .module
                .get_function("vp_print_f64")
                .ok_or_else(|| "vp_print_f64 not declared".to_string())?;
            state
                .builder
                .build_call(print_func, &[val.into()], "print_f64")
                .expect("vp_print_f64");
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

            if is_list_arg {
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
            return Err(format!(
                "print() does not support type {:?}",
                val.get_type()
            ));
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
                .build_call(
                    state.builder,
                    create_func,
                    &[space_str_const.into()],
                    "space_create",
                )
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
    state
        .builder
        .build_call(newline_func, &[], "print_newline")
        .expect("vp_print_newline");

    return Ok(state.ir_builder.i64_const(0).into());
}

/// Generate len() call
pub fn generate_len_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "len() takes exactly 1 argument, got {}",
            args.len()
        ));
    }

    let obj_expr = &args[0];
    let obj_val = generate_expr(state, obj_expr)?;

    // Check if it's a list (literal or variable)
    let is_list = match obj_expr {
        Expr::List { .. } | Expr::Array { .. } | Expr::ListComprehension { .. } => true,
        Expr::Ident(name, _) => state.is_list(name),
        _ => false,
    };

    // Call the appropriate length function
    if is_list {
        // Call vp_list_len for lists
        let list_len = state
            .module
            .get_function("vp_list_len")
            .ok_or_else(|| "vp_list_len not declared".to_string())?;
        let result =
            state
                .ir_builder
                .build_call(state.builder, list_len, &[obj_val.into()], "list_len");
        return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
    }

    // Otherwise treat as string (for string literals or variables)
    if obj_val.is_pointer_value() {
        // Call vp_str_len for strings
        let str_len = state
            .module
            .get_function("vp_str_len")
            .ok_or_else(|| "vp_str_len not declared".to_string())?;
        let result =
            state
                .ir_builder
                .build_call(state.builder, str_len, &[obj_val.into()], "str_len");
        return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
    }

    // Fallback: treat as list
    let list_len = state
        .module
        .get_function("vp_list_len")
        .ok_or_else(|| "vp_list_len not declared".to_string())?;
    let result =
        state
            .ir_builder
            .build_call(state.builder, list_len, &[obj_val.into()], "list_len");
    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

/// Generate type conversion calls (float(), int())
pub fn generate_type_convert<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
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
        _ => Err(format!("Unknown type conversion: {}", name)),
    }
}

/// Generate str() call - convert value to string
pub fn generate_str_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "str() takes exactly 1 argument, got {}",
            args.len()
        ));
    }

    let arg_val = generate_expr(state, &args[0])?;

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
        return Err(format!(
            "{}() takes exactly 1 argument, got {}",
            name,
            args.len()
        ));
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
        state
            .ir_builder
            .build_call(state.builder, math_func, &[arg_float.into()], "math_result");
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

