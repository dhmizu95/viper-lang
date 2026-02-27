//! Expression code generation for Viper

use super::*;

use crate::ast::{Expr, Type};
use crate::utils::mangle_function_name;

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;
// use crate::codegen::inline_lists::inline_i64_list_append;  // Reserved for future inline optimization

/// Generate lambda expression
pub fn generate_lambda<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    params: &[String],
    body: &Expr,
    span: crate::utils::Span,
) -> Result<BasicValueEnum<'ctx>, String> {
    // We assume i64 for all lambda params and return type for now
    let i64_type = state.context.i64_type();
    let mut param_types = Vec::new();
    for _ in params {
        param_types.push(i64_type.into());
    }

    let fn_type = i64_type.fn_type(&param_types, false);
    let lambda_name = format!("__lambda_{}_{}", span.line, span.column);
    let func = state.module.add_function(&lambda_name, fn_type, None);

    // Save insertion block
    let current_block = state.builder.get_insert_block().unwrap();

    let entry_block = state.context.append_basic_block(func, "entry");
    state.builder.position_at_end(entry_block);

    // Setup params (as i64)
    // We don't do full closure capture, only parameters
    // We need to temporarily push these params to state.variables
    let mut old_vars = Vec::new();
    for (i, param_name) in params.iter().enumerate() {
        let param_value = func.get_nth_param(i as u32).unwrap();
        let alloca = state.builder.build_alloca(i64_type, param_name).expect("alloca");
        state.builder.build_store(alloca, param_value).expect("store");

        let old_var = state.variables.insert(
            param_name.clone(),
            crate::codegen::variables::VarInfo::new_stack(
                alloca,
                crate::codegen::variables::VarType::Int,
            ),
        );
        old_vars.push((param_name.clone(), old_var));
    }

    // Generate body
    let body_val = generate_expr(state, body)?;
    let body_int = if body_val.is_int_value() {
        body_val.into_int_value()
    } else {
        return Err("Lambda must return int value currently".to_string());
    };
    state.builder.build_return(Some(&body_int)).expect("return");

    // Restore builder
    state.builder.position_at_end(current_block);

    // Restore variables
    for (name, old_var) in old_vars {
        if let Some(var) = old_var {
            state.variables.insert(name, var);
        } else {
            state.variables.remove(&name);
        }
    }

    // Note: To return a lambda as a value, we can cast the function pointer
    // to a void pointer (ptr_type) representing a closure/function reference
    Ok(func.as_global_value().as_pointer_value().into())
}

/// Generate function/method call
pub fn generate_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    func: &Expr,
    args: &[Expr],
    _span: crate::utils::Span,
) -> Result<BasicValueEnum<'ctx>, String> {
    if let Expr::Attribute { obj, attr, .. } = func {
        return generate_method_call(state, obj, attr, args);
    }

    if let Expr::Ident(name, _) = func {
        if name == "print" {
            return generate_print_call(state, args);
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

        // Type conversion functions
        if name == "float" || name == "int" || name == "bool" {
            return generate_type_convert(state, name, args);
        }

        // Math builtins
        if name == "sqrt" || name == "abs" || name == "ln" || name == "floor" {
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

        // Check for user-defined functions BEFORE builtins (to support overloading)
        let arg_types: Vec<Type> = args.iter().map(|a| infer_expr_type(a)).collect();
        let mangled_name = mangle_function_name(name, &arg_types);

        // Check if it's a user-defined function first
        // Try exact match first, then fallback to any function that starts with the name
        let func_val = if let Some(&f) = state.functions.get(&mangled_name) {
            Some(f)
        } else {
            // Fallback: find any function that starts with the name (ignoring type info)
            state
                .functions
                .iter()
                .find(|(k, _)| k.starts_with(&format!("{}_", name)))
                .map(|(_, v)| *v)
        };

        if let Some(func_val) = func_val {
            let arg_values: Vec<_> = args
                .iter()
                .map(|a| {
                    generate_expr(state, a)
                        .map(|v| inkwell::values::BasicMetadataValueEnum::from(v))
                })
                .collect::<Result<_, _>>()?;

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

    return Err(format!("Call target is not a function: {:?}", func));
}

/// Generate method call
pub fn generate_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    method_name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    let obj_val = generate_expr(state, obj)?;

    // Check if this is a bool list (bit vector)
    let is_bool_list = match obj {
        Expr::Ident(name, _) => state.is_bool_list(name),
        Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
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
                return Err(format!("append() takes exactly 1 argument, got {}", args.len()));
            }
            let val = generate_expr(state, &args[0])?;
            
            // Use bit vector append for bool lists
            let append_func = if is_bool_list {
                state
                    .module
                    .get_function("vp_bitvec_append")
                    .ok_or_else(|| "vp_bitvec_append not declared".to_string())?
            } else {
                state
                    .module
                    .get_function("vp_list_append")
                    .ok_or_else(|| "vp_list_append not declared".to_string())?
            };
            
            state.ir_builder.build_call(
                state.builder,
                append_func,
                &[obj_val.into(), val.into()],
                "list_append",
            );

            Ok(obj_val)
        }
        "reserve" => {
            if args.len() != 1 {
                return Err(format!("reserve() takes exactly 1 argument, got {}", args.len()));
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
                return Err(format!("insert() takes exactly 2 arguments, got {}", args.len()));
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
                return Err(format!("remove() takes exactly 1 argument, got {}", args.len()));
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
                return Err(format!("pop() takes at most 1 argument, got {}", args.len()));
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
                return Err(format!("clear() takes no arguments, got {}", args.len()));
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
                return Err(format!("extend() takes exactly 1 argument, got {}", args.len()));
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
                return Err(format!("index() takes exactly 1 argument, got {}", args.len()));
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
                return Err(format!("count() takes exactly 1 argument, got {}", args.len()));
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
                return Err(format!("sort() takes no arguments, got {}", args.len()));
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
                return Err(format!("reverse() takes no arguments, got {}", args.len()));
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
                return Err(format!("copy() takes no arguments, got {}", args.len()));
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
                return Err("upper() takes no arguments".to_string());
            }
            let func = state.module.get_function("vp_str_upper").unwrap();
            let result =
                state.ir_builder.build_call(state.builder, func, &[obj_val.into()], "str_upper");
            Ok(result.unwrap())
        }
        "lower" => {
            if !args.is_empty() {
                return Err("lower() takes no arguments".to_string());
            }
            let func = state.module.get_function("vp_str_lower").unwrap();
            let result =
                state.ir_builder.build_call(state.builder, func, &[obj_val.into()], "str_lower");
            Ok(result.unwrap())
        }
        "split" => {
            if args.len() != 1 {
                return Err("split() takes exactly 1 argument".to_string());
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
                return Err("replace() takes exactly 2 arguments".to_string());
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
        "len" => Err("len() is a builtin function, not a method".to_string()),
        _ => Err(format!("Unknown method: {}", method_name)),
    }
}

/// Generate sorted() call - returns a sorted copy of the list
pub fn generate_sorted_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("sorted() takes exactly 1 argument, got {}", args.len()));
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
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("reversed() takes exactly 1 argument, got {}", args.len()));
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
