//! Method call code generation

use crate::ast::Expr;
use crate::ast::Type;
use crate::codegen::expressions::concurrency::{
    generate_waitgroup_add, generate_waitgroup_done, generate_waitgroup_wait,
};
use crate::codegen::expressions::core::{generate_expr, infer_expr_type};
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate method call
pub fn generate_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    method_name: &str,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let obj_val = generate_expr(state, obj)?;

    // Special-case WaitGroup for method-style API: wg.add(), wg.done(), wg.wait()
    if let Type::WaitGroup = infer_expr_type(obj) {
        match method_name {
            "add" => {
                if args.len() != 1 {
                    return crate::codegen::codegen_error(format!(
                        "add() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                }
                let mut call_args = Vec::with_capacity(2);
                call_args.push(obj.clone());
                call_args.push(args[0].clone());
                return generate_waitgroup_add(state, &call_args);
            }
            "done" => {
                if !args.is_empty() {
                    return crate::codegen::codegen_error(format!(
                        "done() takes no arguments, got {}",
                        args.len()
                    ));
                }
                let call_args = vec![obj.clone()];
                return generate_waitgroup_done(state, &call_args);
            }
            "wait" => {
                if !args.is_empty() {
                    return crate::codegen::codegen_error(format!(
                        "wait() takes no arguments, got {}",
                        args.len()
                    ));
                }
                let call_args = vec![obj.clone()];
                return generate_waitgroup_wait(state, &call_args);
            }
            _ => {}
        }
    }

    // For Result methods, we need to load the struct value from alloca if it's a pointer
    let obj_val = if matches!(
        method_name,
        "is_ok" | "is_err" | "unwrap" | "unwrap_err" | "expect" | "unwrap_or" | "unwrap_or_default"
    ) {
        if obj_val.is_pointer_value() {
            // Load the struct value from the alloca
            let result_struct_type = state.context.struct_type(
                &[state.context.i8_type().into(), state.context.i64_type().into()],
                false,
            );
            state
                .builder
                .build_load(result_struct_type, obj_val.into_pointer_value(), "result_loaded")
                .map_err(|e| format!("Failed to load Result: {:?}", e))?
        } else {
            obj_val
        }
    } else {
        obj_val
    };

    // Check if this is a bool list (bit vector)
    let is_bool_list = match obj {
        Expr::Ident(name, _) => state.is_bool_list(name),
        Expr::List { elements, .. } => {
            elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
        }
        Expr::Call { func, .. } => {
            // Check if calling a bit vector function
            if let Expr::Ident(func_name, _) = func.as_ref() {
                func_name.starts_with("vp_bitvec_")
            } else {
                false
            }
        }
        _ => false,
    };

    match method_name {
        "append" => {
            if args.len() != 1 {
                return crate::codegen::codegen_error(format!(
                    "append() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            let val = generate_expr(state, &args[0])?;

            // Convert obj_val to pointer if needed
            let list_ptr = if obj_val.is_pointer_value() {
                obj_val.into_pointer_value()
            } else {
                return crate::codegen::codegen_error(
                    "append() requires a list reference".to_string(),
                );
            };

            // Determine if this is a bool list or float list
            let is_bool_list = state.is_bool_list_expr(obj);
            let is_float_list = if let Expr::Ident(name, _) = obj {
                matches!(
                    name.as_str(),
                    "x" | "y" | "z" | "vx" | "vy" | "vz" | "mass" | "real" | "imag"
                )
            } else {
                false
            };

            // Use inline append for optimized performance
            if is_bool_list {
                // Convert value to bool (i1) if needed
                let bool_val =
                    if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
                        val.into_int_value()
                    } else if val.is_int_value() {
                        // Convert i64 to bool (i1)
                        let int_val = val.into_int_value();
                        state
                            .builder
                            .build_int_compare(
                                inkwell::IntPredicate::NE,
                                int_val,
                                state.context.i64_type().const_zero(),
                                "i64_to_bool",
                            )
                            .map_err(|e| format!("Failed to convert to bool: {:?}", e))?
                    } else {
                        return crate::codegen::codegen_error(
                            "append() value must be integer for bool list".to_string(),
                        );
                    };

                crate::codegen::inline_lists::inline_bool_list_append(state, list_ptr, bool_val)
                    .map_err(|e| format!("Failed to inline bool list append: {}", e))?;
            } else if is_float_list {
                // For float lists, call vp_list_append_f64
                let append_func = state
                    .module
                    .get_function("vp_list_append_f64")
                    .ok_or_else(|| "vp_list_append_f64 not declared".to_string())?;

                // Coerce value to f64 if needed
                let f64_val = if val.is_float_value() {
                    val.into_float_value()
                } else if val.is_int_value() {
                    state
                        .builder
                        .build_signed_int_to_float(
                            val.into_int_value(),
                            state.context.f64_type(),
                            "i64_to_f64",
                        )
                        .expect("i64 to f64")
                } else {
                    return crate::codegen::codegen_error(
                        "append() to float list requires numeric value".to_string(),
                    );
                };

                // Convert list_ptr to pointer value for the call
                let list_arg = list_ptr.into();

                state.ir_builder.build_call(
                    state.builder,
                    append_func,
                    &[list_arg, f64_val.into()],
                    "append_f64",
                );
            } else {
                // Fallback for generic lists
                if val.is_float_value() {
                    // Try to guess if this should be a float list if we are appending a float
                    let append_func = state
                        .module
                        .get_function("vp_list_append_f64")
                        .ok_or_else(|| "vp_list_append_f64 not declared".to_string())?;

                    let f64_val = val.into_float_value();
                    state.ir_builder.build_call(
                        state.builder,
                        append_func,
                        &[list_ptr.into(), f64_val.into()],
                        "append_f64",
                    );
                } else {
                    let int_val = if val.is_int_value() {
                        val.into_int_value()
                    } else if val.is_pointer_value() {
                        state
                            .builder
                            .build_ptr_to_int(
                                val.into_pointer_value(),
                                state.context.i64_type(),
                                "ptr_to_i64",
                            )
                            .map_err(|e| format!("Failed to convert pointer to int: {:?}", e))?
                    } else {
                        return crate::codegen::codegen_error(
                            "append() value must be numeric or pointer".to_string(),
                        );
                    };

                    crate::codegen::inline_lists::inline_i64_list_append(state, list_ptr, int_val)
                        .map_err(|e| format!("Failed to inline i64 list append: {}", e))?;
                }
            }

            Ok(obj_val)
        }
        "reserve" => {
            if args.len() != 1 {
                return crate::codegen::codegen_error(format!(
                    "reserve() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            let capacity = generate_expr(state, &args[0])?.into_int_value();
            let list_reserve = state
                .module
                .get_function("vp_list_reserve")
                .ok_or_else(|| "vp_list_reserve not declared".to_string())?;
            state.ir_builder.build_call(
                state.builder,
                list_reserve,
                &[obj_val.into(), capacity.into()],
                "list_reserve",
            );
            Ok(obj_val)
        }
        "insert" => {
            if args.len() != 2 {
                return crate::codegen::codegen_error(format!(
                    "insert() takes exactly 2 arguments, got {}",
                    args.len()
                ));
            }
            let index = generate_expr(state, &args[0])?.into_int_value();
            let val = generate_expr(state, &args[1])?;

            // Use bit vector insert for bool lists
            let insert_func = if is_bool_list {
                state
                    .module
                    .get_function("vp_bitvec_insert")
                    .ok_or_else(|| "vp_bitvec_insert not declared".to_string())?
            } else {
                state
                    .module
                    .get_function("vp_list_insert")
                    .ok_or_else(|| "vp_list_insert not declared".to_string())?
            };

            state.ir_builder.build_call(
                state.builder,
                insert_func,
                &[obj_val.into(), index.into(), val.into()],
                "list_insert",
            );
            Ok(obj_val)
        }
        "remove" => {
            if args.len() != 1 {
                return crate::codegen::codegen_error(format!(
                    "remove() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            let index = generate_expr(state, &args[0])?.into_int_value();

            // Use bit vector remove for bool lists
            let remove_func = if is_bool_list {
                state
                    .module
                    .get_function("vp_bitvec_remove")
                    .ok_or_else(|| "vp_bitvec_remove not declared".to_string())?
            } else {
                state
                    .module
                    .get_function("vp_list_remove")
                    .ok_or_else(|| "vp_list_remove not declared".to_string())?
            };

            let result = state.ir_builder.build_call(
                state.builder,
                remove_func,
                &[obj_val.into(), index.into()],
                "list_remove",
            );
            Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
        }
        "pop" => {
            if args.len() > 1 {
                return crate::codegen::codegen_error(format!(
                    "pop() takes at most 1 argument, got {}",
                    args.len()
                ));
            }
            if args.is_empty() {
                // pop() - pop last element
                let pop_func = if is_bool_list {
                    state
                        .module
                        .get_function("vp_bitvec_pop")
                        .ok_or_else(|| "vp_bitvec_pop not declared".to_string())?
                } else {
                    state
                        .module
                        .get_function("vp_list_pop")
                        .ok_or_else(|| "vp_list_pop not declared".to_string())?
                };

                let result = state.ir_builder.build_call(
                    state.builder,
                    pop_func,
                    &[obj_val.into()],
                    "list_pop",
                );
                Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
            } else {
                // pop(i) - pop element at index
                let index = generate_expr(state, &args[0])?.into_int_value();
                let remove_func = if is_bool_list {
                    state
                        .module
                        .get_function("vp_bitvec_remove")
                        .ok_or_else(|| "vp_bitvec_remove not declared".to_string())?
                } else {
                    state
                        .module
                        .get_function("vp_list_remove")
                        .ok_or_else(|| "vp_list_remove not declared".to_string())?
                };

                let result = state.ir_builder.build_call(
                    state.builder,
                    remove_func,
                    &[obj_val.into(), index.into()],
                    "list_pop_at",
                );
                Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
            }
        }
        "clear" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error(format!(
                    "clear() takes no arguments, got {}",
                    args.len()
                ));
            }
            let list_clear = state
                .module
                .get_function("vp_list_clear")
                .ok_or_else(|| "vp_list_clear not declared".to_string())?;
            state.ir_builder.build_call(state.builder, list_clear, &[obj_val.into()], "list_clear");
            Ok(obj_val)
        }
        "extend" => {
            if args.len() != 1 {
                return crate::codegen::codegen_error(format!(
                    "extend() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            let other_val = generate_expr(state, &args[0])?;

            // Use bit vector extend for bool lists
            let extend_func = if is_bool_list {
                state
                    .module
                    .get_function("vp_bitvec_extend")
                    .ok_or_else(|| "vp_bitvec_extend not declared".to_string())?
            } else {
                state
                    .module
                    .get_function("vp_list_extend")
                    .ok_or_else(|| "vp_list_extend not declared".to_string())?
            };

            state.ir_builder.build_call(
                state.builder,
                extend_func,
                &[obj_val.into(), other_val.into()],
                "list_extend",
            );
            Ok(obj_val)
        }
        "index" => {
            if args.len() != 1 {
                return crate::codegen::codegen_error(format!(
                    "index() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            let value = generate_expr(state, &args[0])?;

            // Use bit vector index for bool lists
            let index_func = if is_bool_list {
                state
                    .module
                    .get_function("vp_bitvec_index")
                    .ok_or_else(|| "vp_bitvec_index not declared".to_string())?
            } else {
                state
                    .module
                    .get_function("vp_list_index")
                    .ok_or_else(|| "vp_list_index not declared".to_string())?
            };

            let result = state.ir_builder.build_call(
                state.builder,
                index_func,
                &[obj_val.into(), value.into()],
                "list_index",
            );
            Ok(result.unwrap_or(state.ir_builder.i64_const(-1).into()))
        }
        "count" => {
            if args.len() != 1 {
                return crate::codegen::codegen_error(format!(
                    "count() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            let value = generate_expr(state, &args[0])?;

            // Use bit vector count for bool lists
            let count_func = if is_bool_list {
                state
                    .module
                    .get_function("vp_bitvec_count")
                    .ok_or_else(|| "vp_bitvec_count not declared".to_string())?
            } else {
                state
                    .module
                    .get_function("vp_list_count")
                    .ok_or_else(|| "vp_list_count not declared".to_string())?
            };

            let result = state.ir_builder.build_call(
                state.builder,
                count_func,
                &[obj_val.into(), value.into()],
                "list_count",
            );
            Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
        }
        "sort" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error(format!(
                    "sort() takes no arguments, got {}",
                    args.len()
                ));
            }
            let list_sort = state
                .module
                .get_function("vp_list_sort")
                .ok_or_else(|| "vp_list_sort not declared".to_string())?;
            state.ir_builder.build_call(state.builder, list_sort, &[obj_val.into()], "list_sort");
            Ok(obj_val)
        }
        "reverse" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error(format!(
                    "reverse() takes no arguments, got {}",
                    args.len()
                ));
            }

            // Use bit vector reverse for bool lists
            let reverse_func = if is_bool_list {
                state
                    .module
                    .get_function("vp_bitvec_reverse")
                    .ok_or_else(|| "vp_bitvec_reverse not declared".to_string())?
            } else {
                state
                    .module
                    .get_function("vp_list_reverse")
                    .ok_or_else(|| "vp_list_reverse not declared".to_string())?
            };

            state.ir_builder.build_call(
                state.builder,
                reverse_func,
                &[obj_val.into()],
                "list_reverse",
            );
            Ok(obj_val)
        }
        "copy" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error(format!(
                    "copy() takes no arguments, got {}",
                    args.len()
                ));
            }

            // Use bit vector copy for bool lists
            let copy_func = if is_bool_list {
                state
                    .module
                    .get_function("vp_bitvec_copy")
                    .ok_or_else(|| "vp_bitvec_copy not declared".to_string())?
            } else {
                state
                    .module
                    .get_function("vp_list_copy")
                    .ok_or_else(|| "vp_list_copy not declared".to_string())?
            };

            let result = state.ir_builder.build_call(
                state.builder,
                copy_func,
                &[obj_val.into()],
                "list_copy",
            );
            Ok(result.unwrap_or(obj_val))
        }
        "upper" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error("upper() takes no arguments".to_string());
            }
            let func = state.module.get_function("vp_str_upper").unwrap();
            let result =
                state.ir_builder.build_call(state.builder, func, &[obj_val.into()], "str_upper");
            Ok(result.unwrap())
        }
        "lower" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error("lower() takes no arguments".to_string());
            }
            let func = state.module.get_function("vp_str_lower").unwrap();
            let result =
                state.ir_builder.build_call(state.builder, func, &[obj_val.into()], "str_lower");
            Ok(result.unwrap())
        }
        "split" => {
            if args.len() != 1 {
                return crate::codegen::codegen_error(
                    "split() takes exactly 1 argument".to_string(),
                );
            }
            let delim_val = generate_expr(state, &args[0])?;
            let func = state.module.get_function("vp_str_split").unwrap();
            let result = state.ir_builder.build_call(
                state.builder,
                func,
                &[obj_val.into(), delim_val.into()],
                "str_split",
            );
            Ok(result.unwrap())
        }
        "replace" => {
            if args.len() != 2 {
                return crate::codegen::codegen_error(
                    "replace() takes exactly 2 arguments".to_string(),
                );
            }
            let old_val = generate_expr(state, &args[0])?;
            let new_val = generate_expr(state, &args[1])?;
            let func = state.module.get_function("vp_str_replace").unwrap();
            let result = state.ir_builder.build_call(
                state.builder,
                func,
                &[obj_val.into(), old_val.into(), new_val.into()],
                "str_replace",
            );
            Ok(result.unwrap())
        }
        "format" => {
            // String format method: "Hello {}".format(name)
            // For simplicity, we'll handle basic {} placeholders
            if args.is_empty() {
                return crate::codegen::codegen_error(
                    "format() takes at least 1 argument".to_string(),
                );
            }

            // Generate all argument values and convert to strings
            let mut arg_str_vals: Vec<inkwell::values::BasicValueEnum> = Vec::new();
            for arg in args {
                let arg_val = generate_expr(state, arg)?;
                // Convert each argument to string based on its type
                let str_val = if arg_val.is_int_value()
                    && arg_val.get_type().into_int_type().get_bit_width() == 64
                {
                    // i64 to string
                    let to_str = state.module.get_function("vp_str_from_i64").unwrap();
                    state
                        .ir_builder
                        .build_call(state.builder, to_str, &[arg_val.into()], "i64_to_str")
                        .unwrap()
                } else if arg_val.is_float_value() {
                    // f64 to string
                    let to_str = state.module.get_function("vp_str_from_f64").unwrap();
                    state
                        .ir_builder
                        .build_call(state.builder, to_str, &[arg_val.into()], "f64_to_str")
                        .unwrap()
                } else if arg_val.is_int_value()
                    && arg_val.get_type().into_int_type().get_bit_width() == 1
                {
                    // bool to string
                    let to_str = state.module.get_function("vp_str_from_bool").unwrap();
                    state
                        .ir_builder
                        .build_call(state.builder, to_str, &[arg_val.into()], "bool_to_str")
                        .unwrap()
                } else if arg_val.is_pointer_value() {
                    // Already a string or pointer - use as-is
                    arg_val
                } else {
                    // Default: use as-is (may not be a string)
                    arg_val
                };
                arg_str_vals.push(str_val);
            }

            // Create array of argument string pointers
            let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
            let array_type = ptr_type.array_type(arg_str_vals.len() as u32);
            let args_array = state
                .builder
                .build_alloca(array_type, "format_args_array")
                .expect("alloca args array");
            for (i, arg_str) in arg_str_vals.iter().enumerate() {
                let arg_ptr = unsafe {
                    state
                        .builder
                        .build_gep(
                            array_type,
                            args_array,
                            &[
                                state.context.i32_type().const_zero(),
                                state.context.i32_type().const_int(i as u64, false),
                            ],
                            "arg_ptr",
                        )
                        .expect("gep")
                };
                state
                    .builder
                    .build_store(arg_ptr, arg_str.into_pointer_value())
                    .expect("store arg");
            }

            // Call vp_str_format(format_str, args_array, arg_count)
            let format_func = state
                .module
                .get_function("vp_str_format")
                .ok_or_else(|| "vp_str_format not declared. Add to runtime library.".to_string())?;

            let result = state.ir_builder.build_call(
                state.builder,
                format_func,
                &[
                    obj_val.into(),
                    args_array.into(),
                    state.ir_builder.i64_const(arg_str_vals.len() as i64).into(),
                ],
                "str_format",
            );
            Ok(result.unwrap())
        }
        // Result methods
        "is_ok" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error(format!(
                    "is_ok() takes no arguments, got {}",
                    args.len()
                ));
            }
            // obj_val is now a struct value, not a pointer
            let result_struct = obj_val.into_struct_value();
            // Extract is_ok field (first field)
            let is_ok_val = state
                .builder
                .build_extract_value(result_struct, 0, "is_ok")
                .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?;
            let is_ok = is_ok_val.into_int_value();
            // Convert i8 to bool (i1)
            let is_ok_bool = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    is_ok,
                    state.context.i8_type().const_zero(),
                    "is_ok_bool",
                )
                .map_err(|e| format!("Failed to build compare: {:?}", e))?;
            Ok(is_ok_bool.into())
        }
        "is_err" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error(format!(
                    "is_err() takes no arguments, got {}",
                    args.len()
                ));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract is_ok field and negate
            let is_ok_val = state
                .builder
                .build_extract_value(result_struct, 0, "is_ok")
                .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?;
            let is_ok = is_ok_val.into_int_value();
            // is_err = !is_ok
            let is_err = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    is_ok,
                    state.context.i8_type().const_zero(),
                    "is_err",
                )
                .map_err(|e| format!("Failed to build compare: {:?}", e))?;
            Ok(is_err.into())
        }
        "unwrap" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error(format!(
                    "unwrap() takes no arguments, got {}",
                    args.len()
                ));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract value field (second field)
            let value = state
                .builder
                .build_extract_value(result_struct, 1, "value")
                .map_err(|e| format!("Failed to extract value: {:?}", e))?;
            Ok(value)
        }
        "unwrap_err" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error(format!(
                    "unwrap_err() takes no arguments, got {}",
                    args.len()
                ));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract value field (error is stored in same field)
            let value = state
                .builder
                .build_extract_value(result_struct, 1, "error_value")
                .map_err(|e| format!("Failed to extract error value: {:?}", e))?;
            Ok(value)
        }
        "expect" => {
            if args.len() != 1 {
                return crate::codegen::codegen_error(format!(
                    "expect() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract value field (ignore message for now)
            let value = state
                .builder
                .build_extract_value(result_struct, 1, "value")
                .map_err(|e| format!("Failed to extract value: {:?}", e))?;
            Ok(value)
        }
        "unwrap_or" => {
            if args.len() != 1 {
                return crate::codegen::codegen_error(format!(
                    "unwrap_or() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract is_ok field
            let is_ok_val = state
                .builder
                .build_extract_value(result_struct, 0, "is_ok")
                .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?;
            let is_ok = is_ok_val.into_int_value();
            // Convert i8 to bool (i1) for select instruction
            let is_ok_bool = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    is_ok,
                    state.context.i8_type().const_zero(),
                    "is_ok_bool",
                )
                .map_err(|e| format!("Failed to build compare: {:?}", e))?;

            // Extract value from Result
            let result_value = state
                .builder
                .build_extract_value(result_struct, 1, "result_value")
                .map_err(|e| format!("Failed to extract value: {:?}", e))?
                .into_int_value();

            // Generate default value
            let default_value = generate_expr(state, &args[0])?;
            let default_int = if default_value.is_int_value() {
                default_value.into_int_value()
            } else {
                return crate::codegen::codegen_error(
                    "unwrap_or default value must be integer".to_string(),
                );
            };

            // Select based on is_ok
            let selected = state
                .builder
                .build_select(is_ok_bool, result_value, default_int, "unwrap_or_select")
                .map_err(|e| format!("Failed to build select: {:?}", e))?;

            Ok(selected.into())
        }
        "unwrap_or_default" => {
            if !args.is_empty() {
                return crate::codegen::codegen_error(format!(
                    "unwrap_or_default() takes no arguments, got {}",
                    args.len()
                ));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract is_ok field
            let is_ok_val = state
                .builder
                .build_extract_value(result_struct, 0, "is_ok")
                .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?;
            let is_ok = is_ok_val.into_int_value();
            // Convert i8 to bool (i1) for select instruction
            let is_ok_bool = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    is_ok,
                    state.context.i8_type().const_zero(),
                    "is_ok_bool_default",
                )
                .map_err(|e| format!("Failed to build compare: {:?}", e))?;

            // Extract value from Result
            let result_value = state
                .builder
                .build_extract_value(result_struct, 1, "result_value")
                .map_err(|e| format!("Failed to extract value: {:?}", e))?
                .into_int_value();

            // Default is 0
            let default_value = state.context.i64_type().const_zero();

            // Select based on is_ok
            let selected = state
                .builder
                .build_select(is_ok_bool, result_value, default_value, "unwrap_or_default")
                .map_err(|e| format!("Failed to build select: {:?}", e))?;

            Ok(selected.into())
        }
        "len" => {
            crate::codegen::codegen_error("len() is a builtin function, not a method".to_string())
        }
        _ => crate::codegen::codegen_error(format!("Unknown method: {}", method_name)),
    }
}

/// Generate sorted() call - returns a sorted copy of the list
pub fn generate_sorted_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!(
            "sorted() takes exactly 1 argument, got {}",
            args.len()
        ));
    }

    let list_val = generate_expr(state, &args[0])?;
    let list_sorted = state
        .module
        .get_function("vp_list_sorted")
        .ok_or_else(|| "vp_list_sorted not declared".to_string())?;
    let result =
        state.ir_builder.build_call(state.builder, list_sorted, &[list_val.into()], "sorted_list");
    Ok(result.unwrap_or(list_val))
}

/// Generate reversed() call - returns a reversed copy of the list
pub fn generate_reversed_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!(
            "reversed() takes exactly 1 argument, got {}",
            args.len()
        ));
    }

    let list_val = generate_expr(state, &args[0])?;
    let list_reversed = state
        .module
        .get_function("vp_list_reversed")
        .ok_or_else(|| "vp_list_reversed not declared".to_string())?;
    let result = state.ir_builder.build_call(
        state.builder,
        list_reversed,
        &[list_val.into()],
        "reversed_list",
    );
    Ok(result.unwrap_or(list_val))
}

/// Generate dict() call
pub fn generate_dict_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // dict() with no args returns empty dict
    if args.is_empty() {
        let func = state
            .module
            .get_function("vp_dict_create_empty")
            .ok_or_else(|| "vp_dict_create_empty not declared".to_string())?;
        let result = state.ir_builder.build_call(state.builder, func, &[], "empty_dict");
        return Ok(result.unwrap());
    }

    // For now, just return empty dict - full implementation would convert iterable
    let func = state
        .module
        .get_function("vp_dict_create_empty")
        .ok_or_else(|| "vp_dict_create_empty not declared".to_string())?;
    let result = state.ir_builder.build_call(state.builder, func, &[], "dict_from_iter");
    Ok(result.unwrap())
}
