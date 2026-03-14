//! Print function code generation for Viper

use crate::ast::{Expr, Type};
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::VarType;

use inkwell::values::BasicValueEnum;

use crate::codegen::expressions::core::{generate_expr, infer_expr_type};

/// Helper function to check if an expression is a BigInt for print handling
fn is_bigint_expr_for_print(expr: &Expr, state: &CodeGenState) -> bool {
    match expr {
        Expr::BigInt(..) => true,
        Expr::Ident(name, _) => state.is_bigint(name),
        Expr::Call { func, .. } => {
            if let Expr::Ident(func_name, _) = func.as_ref() {
                func_name == "bigint"
                    || func_name == "BigInt"
                    || func_name == "abs_bigint"
                    || func_name == "pow_bigint"
                    || func_name == "sqrt_bigint"
                    || func_name == "min_bigint"
                    || func_name == "max_bigint"
            } else {
                false
            }
        }
        Expr::BinOp { left, right, .. } => {
            is_bigint_expr_for_print(left, state) || is_bigint_expr_for_print(right, state)
        }
        _ => false,
    }
}

/// Generate print call - handles multiple arguments
pub fn generate_print_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
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

        // IMPORTANT: Check pointer values first before type-based dispatch.
        // This handles cases like __name__ where type inference returns Type::Infer
        // but the actual value is a pointer (string).
        if val.is_pointer_value() {
            // Pointer value - could be string, list, dict, etc.
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
                    // Method calls are represented as Call { func: Attribute { obj, attr }, args }
                    let method_name = if let Expr::Attribute { attr, .. } = func.as_ref() {
                        Some(attr.as_str())
                    } else if let Expr::Ident(func_name, _) = func.as_ref() {
                        Some(func_name.as_str())
                    } else {
                        None
                    };

                    if let Some(func_name) = method_name {
                        // str_bigint() and int_bigint() return non-BigInt types
                        if func_name == "str_bigint" || func_name == "int_bigint" {
                            false
                        // String methods return strings, not BigInts
                        } else if func_name == "upper"
                            || func_name == "lower"
                            || func_name == "strip"
                            || func_name == "capitalize"
                            || func_name == "title"
                            || func_name == "swapcase"
                            || func_name == "replace"
                            || func_name == "split"
                            || func_name == "join"
                        {
                            false
                        } else {
                            // Check for BigInt-returning functions
                            func_name == "bigint"
                                || func_name == "BigInt"
                                || func_name == "abs_bigint"
                                || func_name == "abs"
                                || func_name == "pow_bigint"
                                || func_name == "pow"
                                || func_name == "sqrt_bigint"
                                || func_name == "min_bigint"
                                || func_name == "max_bigint"
                                || val.is_pointer_value() // User-defined BigInt function
                        }
                    } else {
                        val.is_pointer_value()
                    }
                }
                // BinOp with BigInt operands returns a BigInt pointer
                Expr::BinOp { left, right, op, .. } => {
                    if matches!(
                        op,
                        crate::ast::BinOp::Add
                            | crate::ast::BinOp::Sub
                            | crate::ast::BinOp::Mul
                            | crate::ast::BinOp::Div
                            | crate::ast::BinOp::Mod
                            | crate::ast::BinOp::Pow
                    ) {
                        // Check if either operand is BigInt
                        is_bigint_expr_for_print(left, state)
                            || is_bigint_expr_for_print(right, state)
                    } else {
                        val.is_pointer_value() && infer_expr_type(arg) == Type::BigInt
                    }
                }
                _ => val.is_pointer_value() && infer_expr_type(arg) == Type::BigInt,
            };

            if is_bigint_arg {
                // Convert BigInt to string and print
                let to_str_func = state
                    .module
                    .get_function("vp_bigint_to_str")
                    .ok_or_else(|| "vp_bigint_to_str not declared".to_string())?;

                let base = state.context.i32_type().const_int(10, false);
                let c_str_val = state
                    .ir_builder
                    .build_call(
                        state.builder,
                        to_str_func,
                        &[val.into(), base.into()],
                        "c_str_conv",
                    )
                    .expect("vp_bigint_to_str");

                // vp_bigint_to_str returns a C string (char*), use vp_print_cstr to print it
                let print_func = state
                    .module
                    .get_function("vp_print_cstr")
                    .ok_or_else(|| "vp_print_cstr not declared".to_string())?;
                state
                    .builder
                    .build_call(print_func, &[c_str_val.into()], "print_bigint_str")
                    .expect("vp_print_cstr");
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
                // Decide whether this pointer is a string, a future, or an unknown object.
                let mut is_string = matches!(arg, Expr::Str(..) | Expr::FString(..));
                if let Expr::Ident(name, _) = arg {
                    if let Some(t) = state.var_types.get(name) {
                        is_string |= matches!(t, Type::Str)
                            || matches!(t, Type::Var(s) if s == "str" || s == "string");
                    }
                } else {
                    let inferred = infer_expr_type(arg);
                    is_string |= matches!(inferred, Type::Str)
                        || matches!(inferred, Type::Var(s) if s == "str" || s == "string");
                }

                let mut is_future = false;
                if let Expr::Ident(name, _) = arg {
                    if let Some(t) = state.var_types.get(name) {
                        is_future = matches!(t, Type::Future(_));
                    }
                }

                if is_string {
                    // Use vp_print_viper_str for ViperString* objects
                    let print_func = state
                        .module
                        .get_function("vp_print_viper_str")
                        .ok_or_else(|| "vp_print_viper_str not declared".to_string())?;
                    state
                        .builder
                        .build_call(print_func, &[val.into()], "print_str")
                        .expect("print_str");
                } else if is_future {
                    let print_func = state
                        .module
                        .get_function("vp_print_cstr")
                        .ok_or_else(|| "vp_print_cstr not declared".to_string())?;
                    let msg = state
                        .builder
                        .build_global_string_ptr("<Future>", "future_str")
                        .expect("global string");
                    state
                        .builder
                        .build_call(print_func, &[msg.as_pointer_value().into()], "print_future")
                        .expect("print_future");
                } else {
                    // Fallback for unknown pointer types (avoid segfaulting on non-strings)
                    let print_func = state
                        .module
                        .get_function("vp_print_cstr")
                        .ok_or_else(|| "vp_print_cstr not declared".to_string())?;
                    let msg = state
                        .builder
                        .build_global_string_ptr("<object>", "obj_str")
                        .expect("global string");
                    state
                        .builder
                        .build_call(print_func, &[msg.as_pointer_value().into()], "print_obj")
                        .expect("print_obj");
                }
            }
        } else if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
            // Boolean value (1-bit integer)
            let print_func = state
                .module
                .get_function("vp_print_bool")
                .ok_or_else(|| "vp_print_bool not declared".to_string())?;
            state
                .builder
                .build_call(print_func, &[val.into()], "print_bool")
                .expect("vp_print_bool");
        } else if val.is_int_value() {
            // All int values in Viper are tagged ints (small int or BigInt pointer)
            // Always use tagged_int_print to correctly handle both cases
            let print_func = state
                .module
                .get_function("tagged_int_print")
                .ok_or_else(|| "tagged_int_print not declared".to_string())?;
            state
                .builder
                .build_call(print_func, &[val.into()], "print_tagged_int")
                .expect("tagged_int_print");
        } else if val.is_float_value() {
            let print_func = state
                .module
                .get_function("vp_print_f64")
                .ok_or_else(|| "vp_print_f64 not declared".to_string())?;
            state.builder.build_call(print_func, &[val.into()], "print_f64").expect("vp_print_f64");
        } else {
            return crate::codegen::codegen_error(format!(
                "print() does not support type {:?}",
                val.get_type()
            ));
        }

        // Add space between arguments (but not after the last one)
        if i < args.len() - 1 {
            let print_func = state
                .module
                .get_function("vp_print_viper_str")
                .ok_or_else(|| "vp_print_viper_str not declared".to_string())?;
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
                .expect("vp_print_viper_str");
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

/// Generate exit call
pub fn generate_exit_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Get exit code (default 0)
    let exit_code = if args.is_empty() {
        state.ir_builder.i64_const(0)
    } else {
        let val = generate_expr(state, &args[0])?;
        if val.is_int_value() {
            val.into_int_value()
        } else {
            return crate::codegen::codegen_error(
                "exit() requires an integer argument".to_string(),
            );
        }
    };

    // Call vp_exit
    let exit_func =
        state.module.get_function("vp_exit").ok_or_else(|| "vp_exit not declared".to_string())?;

    state.builder.build_call(exit_func, &[exit_code.into()], "exit").expect("vp_exit");

    return Ok(state.ir_builder.i64_const(0).into());
}
