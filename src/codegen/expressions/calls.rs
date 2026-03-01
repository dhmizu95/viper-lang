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

        // BigInt functions
        if name == "BigInt" {
            return generate_bigint_constructor(state, args);
        }
        if name == "str_bigint" {
            return generate_bigint_to_str(state, args);
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

        // Result type constructors
        if name == "Ok" {
            return generate_ok_constructor(state, args);
        }
        if name == "Err" {
            return generate_err_constructor(state, args);
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

        // Check for user-defined functions with overload resolution
        let arg_types: Vec<Type> = args.iter().map(|a| infer_expr_type(a)).collect();
        
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
            
            if overloads.len() <= 1 {
                return None;
            }
            
            // Find the best matching overload
            find_best_overload(&arg_types, &overloads)
                .and_then(|mangled| state.functions.get(mangled).copied())
        });

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

    // For Result methods, we need to load the struct value from alloca if it's a pointer
    let obj_val = if matches!(method_name, "is_ok" | "is_err" | "unwrap" | "unwrap_err" | "expect" | "unwrap_or" | "unwrap_or_default") {
        if obj_val.is_pointer_value() {
            // Load the struct value from the alloca
            let result_struct_type = state.context.struct_type(&[
                state.context.i8_type().into(),
                state.context.i64_type().into(),
            ], false);
            state.builder
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
        "format" => {
            // String format method: "Hello {}".format(name)
            // For simplicity, we'll handle basic {} placeholders
            if args.is_empty() {
                return Err("format() takes at least 1 argument".to_string());
            }
            
            // Generate all argument values and convert to strings
            let mut arg_str_vals: Vec<inkwell::values::BasicValueEnum> = Vec::new();
            for arg in args {
                let arg_val = generate_expr(state, arg)?;
                // Convert each argument to string based on its type
                let str_val = if arg_val.is_int_value() && arg_val.get_type().into_int_type().get_bit_width() == 64 {
                    // i64 to string
                    let to_str = state.module.get_function("vp_str_from_i64").unwrap();
                    state.ir_builder.build_call(state.builder, to_str, &[arg_val.into()], "i64_to_str").unwrap()
                } else if arg_val.is_float_value() {
                    // f64 to string
                    let to_str = state.module.get_function("vp_str_from_f64").unwrap();
                    state.ir_builder.build_call(state.builder, to_str, &[arg_val.into()], "f64_to_str").unwrap()
                } else if arg_val.is_int_value() && arg_val.get_type().into_int_type().get_bit_width() == 1 {
                    // bool to string
                    let to_str = state.module.get_function("vp_str_from_bool").unwrap();
                    state.ir_builder.build_call(state.builder, to_str, &[arg_val.into()], "bool_to_str").unwrap()
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
            let args_array = state.builder.build_alloca(array_type, "format_args_array").expect("alloca args array");
            for (i, arg_str) in arg_str_vals.iter().enumerate() {
                let arg_ptr = unsafe {
                    state.builder.build_gep(array_type, args_array, &[state.context.i32_type().const_int(i as u64, false)], "arg_ptr").expect("gep")
                };
                state.builder.build_store(arg_ptr, arg_str.into_pointer_value()).expect("store arg");
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
                return Err(format!("is_ok() takes no arguments, got {}", args.len()));
            }
            // obj_val is now a struct value, not a pointer
            let result_struct = obj_val.into_struct_value();
            // Extract is_ok field (first field)
            let is_ok_val = state.builder
                .build_extract_value(result_struct, 0, "is_ok")
                .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?;
            let is_ok = is_ok_val.into_int_value();
            // Convert i8 to bool (i1)
            let is_ok_bool = state.builder.build_int_compare(
                inkwell::IntPredicate::NE,
                is_ok,
                state.context.i8_type().const_zero(),
                "is_ok_bool",
            ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
            Ok(is_ok_bool.into())
        }
        "is_err" => {
            if !args.is_empty() {
                return Err(format!("is_err() takes no arguments, got {}", args.len()));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract is_ok field and negate
            let is_ok_val = state.builder
                .build_extract_value(result_struct, 0, "is_ok")
                .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?;
            let is_ok = is_ok_val.into_int_value();
            // is_err = !is_ok
            let is_err = state.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                is_ok,
                state.context.i8_type().const_zero(),
                "is_err",
            ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
            Ok(is_err.into())
        }
        "unwrap" => {
            if !args.is_empty() {
                return Err(format!("unwrap() takes no arguments, got {}", args.len()));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract value field (second field)
            let value = state.builder
                .build_extract_value(result_struct, 1, "value")
                .map_err(|e| format!("Failed to extract value: {:?}", e))?;
            Ok(value)
        }
        "unwrap_err" => {
            if !args.is_empty() {
                return Err(format!("unwrap_err() takes no arguments, got {}", args.len()));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract value field (error is stored in same field)
            let value = state.builder
                .build_extract_value(result_struct, 1, "error_value")
                .map_err(|e| format!("Failed to extract error value: {:?}", e))?;
            Ok(value)
        }
        "expect" => {
            if args.len() != 1 {
                return Err(format!("expect() takes exactly 1 argument, got {}", args.len()));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract value field (ignore message for now)
            let value = state.builder
                .build_extract_value(result_struct, 1, "value")
                .map_err(|e| format!("Failed to extract value: {:?}", e))?;
            Ok(value)
        }
        "unwrap_or" => {
            if args.len() != 1 {
                return Err(format!("unwrap_or() takes exactly 1 argument, got {}", args.len()));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract is_ok field
            let is_ok_val = state.builder
                .build_extract_value(result_struct, 0, "is_ok")
                .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?;
            let is_ok = is_ok_val.into_int_value();
            
            // Extract value from Result
            let result_value = state.builder
                .build_extract_value(result_struct, 1, "result_value")
                .map_err(|e| format!("Failed to extract value: {:?}", e))?
                .into_int_value();
            
            // Generate default value
            let default_value = generate_expr(state, &args[0])?;
            let default_int = if default_value.is_int_value() {
                default_value.into_int_value()
            } else {
                return Err("unwrap_or default value must be integer".to_string());
            };
            
            // Select based on is_ok
            let selected = state.builder.build_select(
                is_ok,
                result_value,
                default_int,
                "unwrap_or_select",
            ).map_err(|e| format!("Failed to build select: {:?}", e))?;
            
            Ok(selected.into())
        }
        "unwrap_or_default" => {
            if !args.is_empty() {
                return Err(format!("unwrap_or_default() takes no arguments, got {}", args.len()));
            }
            // obj_val is a struct value
            let result_struct = obj_val.into_struct_value();
            // Extract is_ok field
            let is_ok_val = state.builder
                .build_extract_value(result_struct, 0, "is_ok")
                .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?;
            let is_ok = is_ok_val.into_int_value();
            
            // Extract value from Result
            let result_value = state.builder
                .build_extract_value(result_struct, 1, "result_value")
                .map_err(|e| format!("Failed to extract value: {:?}", e))?
                .into_int_value();
            
            // Default is 0
            let default_value = state.context.i64_type().const_zero();
            
            // Select based on is_ok
            let selected = state.builder.build_select(
                is_ok,
                result_value,
                default_value,
                "unwrap_or_default",
            ).map_err(|e| format!("Failed to build select: {:?}", e))?;
            
            Ok(selected.into())
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

/* ============================================ */
/* BigInt Built-in Functions                    */
/* ============================================ */

/// Generate BigInt constructor call
pub fn generate_bigint_constructor<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("BigInt() takes exactly 1 argument, got {}", args.len()));
    }

    let arg_val = generate_expr(state, &args[0])?;
    
    // If argument is a string, use vp_bigint_from_str
    if arg_val.is_pointer_value() {
        let from_str_func = state
            .module
            .get_function("vp_bigint_from_str")
            .ok_or_else(|| "vp_bigint_from_str not declared".to_string())?;
        
        let result = state
            .ir_builder
            .build_call(state.builder, from_str_func, &[arg_val.into()], "bigint_create")
            .expect("bigint_from_str call");
        
        return Ok(result.into());
    }
    
    // If argument is i64, use vp_bigint_from_i64
    if arg_val.is_int_value() {
        let from_i64_func = state
            .module
            .get_function("vp_bigint_from_i64")
            .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
        
        let result = state
            .ir_builder
            .build_call(state.builder, from_i64_func, &[arg_val.into()], "bigint_create")
            .expect("bigint_from_i64 call");
        
        return Ok(result.into());
    }
    
    Err("BigInt() requires string or integer argument".to_string())
}

/// Generate str_bigint() call - convert BigInt to string
pub fn generate_bigint_to_str<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("str_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    
    let to_str_func = state
        .module
        .get_function("vp_bigint_to_str")
        .ok_or_else(|| "vp_bigint_to_str not declared".to_string())?;
    
    // Call vp_bigint_to_str(bigint, 10) - base 10
    let base = state.context.i32_type().const_int(10, false);
    let result = state
        .ir_builder
        .build_call(state.builder, to_str_func, &[bigint_val.into(), base.into()], "bigint_to_str")
        .expect("bigint_to_str call");
    
    Ok(result.into())
}

/// Generate int_bigint() call - convert BigInt to i64
pub fn generate_bigint_to_i64<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("int_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    
    let to_i64_func = state
        .module
        .get_function("vp_bigint_to_i64")
        .ok_or_else(|| "vp_bigint_to_i64 not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, to_i64_func, &[bigint_val.into()], "bigint_to_i64")
        .expect("bigint_to_i64 call");
    
    Ok(result.into())
}

/// Generate abs_bigint() call - absolute value of BigInt
pub fn generate_bigint_abs<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("abs_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
    
    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();
    
    let abs_func = state
        .module
        .get_function("vp_bigint_abs")
        .ok_or_else(|| "vp_bigint_abs not declared".to_string())?;
    
    state
        .ir_builder
        .build_call(
            state.builder,
            abs_func,
            &[result_ptr.into(), bigint_val.into()],
            "bigint_abs_call",
        );
    
    Ok(result_ptr.into())
}

/// Generate pow_bigint() call - BigInt power
pub fn generate_bigint_pow<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!("pow_bigint() takes exactly 2 arguments, got {}", args.len()));
    }

    let base_val = generate_expr(state, &args[0])?;
    let exp_val = generate_expr(state, &args[1])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
    
    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();
    
    let pow_func = state
        .module
        .get_function("vp_bigint_pow")
        .ok_or_else(|| "vp_bigint_pow not declared".to_string())?;
    
    state
        .ir_builder
        .build_call(
            state.builder,
            pow_func,
            &[result_ptr.into(), base_val.into(), exp_val.into()],
            "bigint_pow_call",
        );
    
    Ok(result_ptr.into())
}

/// Generate sqrt_bigint() call - BigInt square root
pub fn generate_bigint_sqrt<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("sqrt_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
    
    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();
    
    let sqrt_func = state
        .module
        .get_function("vp_bigint_sqrt")
        .ok_or_else(|| "vp_bigint_sqrt not declared".to_string())?;
    
    state
        .ir_builder
        .build_call(
            state.builder,
            sqrt_func,
            &[result_ptr.into(), bigint_val.into()],
            "bigint_sqrt_call",
        );
    
    Ok(result_ptr.into())
}

/// Generate min_bigint() call
pub fn generate_bigint_min<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!("min_bigint() takes exactly 2 arguments, got {}", args.len()));
    }

    let a_val = generate_expr(state, &args[0])?;
    let b_val = generate_expr(state, &args[1])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
    
    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();
    
    let min_func = state
        .module
        .get_function("vp_bigint_min")
        .ok_or_else(|| "vp_bigint_min not declared".to_string())?;
    
    state
        .ir_builder
        .build_call(
            state.builder,
            min_func,
            &[result_ptr.into(), a_val.into(), b_val.into()],
            "bigint_min_call",
        );
    
    Ok(result_ptr.into())
}

/// Generate max_bigint() call
pub fn generate_bigint_max<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!("max_bigint() takes exactly 2 arguments, got {}", args.len()));
    }

    let a_val = generate_expr(state, &args[0])?;
    let b_val = generate_expr(state, &args[1])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
    
    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();
    
    let max_func = state
        .module
        .get_function("vp_bigint_max")
        .ok_or_else(|| "vp_bigint_max not declared".to_string())?;
    
    state
        .ir_builder
        .build_call(
            state.builder,
            max_func,
            &[result_ptr.into(), a_val.into(), b_val.into()],
            "bigint_max_call",
        );
    
    Ok(result_ptr.into())
}

/// Generate is_zero_bigint() call
pub fn generate_bigint_is_zero<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("is_zero_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let is_zero_func = state
        .module
        .get_function("vp_bigint_is_zero")
        .ok_or_else(|| "vp_bigint_is_zero not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, is_zero_func, &[bigint_val.into()], "bigint_is_zero")
        .ok_or_else(|| "Failed to call vp_bigint_is_zero".to_string())?;
    
    Ok(result)
}

/// Generate is_negative_bigint() call
pub fn generate_bigint_is_negative<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("is_negative_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let is_neg_func = state
        .module
        .get_function("vp_bigint_is_negative")
        .ok_or_else(|| "vp_bigint_is_negative not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, is_neg_func, &[bigint_val.into()], "bigint_is_negative")
        .ok_or_else(|| "Failed to call vp_bigint_is_negative".to_string())?;
    
    Ok(result)
}

/// Generate sign_bigint() call
pub fn generate_bigint_sign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("sign_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let sign_func = state
        .module
        .get_function("vp_bigint_sign")
        .ok_or_else(|| "vp_bigint_sign not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, sign_func, &[bigint_val.into()], "bigint_sign")
        .ok_or_else(|| "Failed to call vp_bigint_sign".to_string())?;
    
    Ok(result)
}

/// Generate bit_length_bigint() call
pub fn generate_bigint_bit_length<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("bit_length_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let bit_len_func = state
        .module
        .get_function("vp_bigint_bit_length")
        .ok_or_else(|| "vp_bigint_bit_length not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, bit_len_func, &[bigint_val.into()], "bigint_bit_length")
        .ok_or_else(|| "Failed to call vp_bigint_bit_length".to_string())?;

    Ok(result)
}

/// Generate Ok constructor call
/// Creates a Result struct with is_ok=1 and the value, returned by value
pub fn generate_ok_constructor<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("Ok() takes exactly 1 argument, got {}", args.len()));
    }

    // Generate the value expression
    let value = generate_expr(state, &args[0])?;
    
    // Create Result struct type: { is_ok: i8, value: i64 }
    let result_struct_type = state.context.struct_type(&[
        state.context.i8_type().into(),
        state.context.i64_type().into(),
    ], false);
    
    // Create the struct value directly (by value, not pointer)
    let value_field = if value.is_int_value() {
        value.into_int_value()
    } else {
        // For non-i64 values, use 0 as placeholder
        state.context.i64_type().const_zero()
    };
    
    let result_val = result_struct_type.const_named_struct(&[
        state.context.i8_type().const_int(1, false).into(), // is_ok = true
        value_field.into(),
    ]);
    
    Ok(result_val.into())
}

/// Generate Err constructor call
/// Creates a Result struct with is_ok=0 and the error value, returned by value
pub fn generate_err_constructor<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("Err() takes exactly 1 argument, got {}", args.len()));
    }

    // Generate the error expression
    let error = generate_expr(state, &args[0])?;
    
    // Create Result struct type: { is_ok: i8, value: i64 }
    let result_struct_type = state.context.struct_type(&[
        state.context.i8_type().into(),
        state.context.i64_type().into(),
    ], false);
    
    // Create the struct value directly (by value, not pointer)
    let error_field = if error.is_int_value() {
        error.into_int_value()
    } else {
        // For non-i64 values, use 0 as placeholder
        state.context.i64_type().const_zero()
    };
    
    let result_val = result_struct_type.const_named_struct(&[
        state.context.i8_type().const_int(0, false).into(), // is_ok = false
        error_field.into(),
    ]);

    Ok(result_val.into())
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
