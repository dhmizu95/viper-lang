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
        // Check for super().method() call
        if let Expr::Super(_) = obj.as_ref() {
            return generate_super_method_call(state, attr, args);
        }
        
        // Handle math module specifically for BigInts
        if let Expr::Ident(name, _) = obj.as_ref() {
            if name == "math" {
                match attr.as_str() {
                    "isqrt" | "gcd" | "lcm" | "factorial" | "comb" | "perm" => {
                        // Always use BigInt path for these functions
                        return generate_math_bigint_func(state, attr, args);
                    }
                    _ => {} // Fall through to standard math dispatch logic in generate_method_call or others
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
        if name == "bigint" || name == "BigInt" {
            return generate_bigint_constructor(state, args);
        }
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
                0 => return Err("range expected at least 1 argument, got 0".to_string()),
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
            let arg_values: Vec<_> = args
                .iter()
                .map(|a| {
                    generate_expr(state, a)
                        .map(|v| inkwell::values::BasicMetadataValueEnum::from(v))
                })
                .collect::<Result<_, _>>()?;

            // Note: Closure cell parameter passing is disabled until runtime is properly linked
            // Append closure cell parameters if this is a nested function call
            // Check if current function has captured variables that need to be passed
            // if let Some(current_func) = state.current_function {
            //     if let Some(closure_analyzer) = state.closure_analyzer {
            //         let captured = closure_analyzer.get_closure_cells_to_create(current_func);
            //         for var_name in &captured {
            //             if let Some(var_info) = state.variables.get(var_name) {
            //                 if let Some(cell_ptr) = var_info.get_closure_cell() {
            //                     arg_values.push(cell_ptr.into());
            //                 }
            //             }
            //         }
            //     }
            // }

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

/// Helper function to handle BigInt math function routing
pub fn generate_math_bigint_func<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    attr: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;

    let get_bigint = |val: BasicValueEnum<'ctx>| -> Result<BasicValueEnum<'ctx>, String> {
        if val.is_pointer_value() {
            Ok(val)
        } else if val.is_int_value() {
            let res = state.ir_builder.build_call(state.builder, from_i64_func, &[val.into()], "bigint_from_i64")
                .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?;
            Ok(res.into_pointer_value().into())
        } else {
            Err(format!("Cannot convert argument to BigInt for math.{}", attr))
        }
    };

    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();

    match attr {
        "gcd" | "lcm" | "comb" | "perm" => {
            if args.len() != 2 {
                return Err(format!("math.{} requires exactly 2 arguments", attr));
            }
            let val0 = generate_expr(state, &args[0])?;
            let val1 = generate_expr(state, &args[1])?;
            let ptr0 = get_bigint(val0)?;
            let ptr1 = get_bigint(val1)?;
            let func_name = format!("vp_bigint_{}", attr);
            let func = state.module.get_function(&func_name).ok_or_else(|| format!("{} not declared", func_name))?;
            state.ir_builder.build_call(state.builder, func, &[result_ptr.into(), ptr0.into(), ptr1.into()], &format!("{}_call", attr));
        },
        "isqrt" => {
            if args.len() != 1 {
                return Err(format!("math.{} requires exactly 1 argument", attr));
            }
            let val0 = generate_expr(state, &args[0])?;
            let ptr0 = get_bigint(val0)?;
            let func = state.module.get_function("vp_bigint_sqrt").ok_or_else(|| "vp_bigint_sqrt not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[result_ptr.into(), ptr0.into()], "isqrt_call");
        },
        "factorial" => {
            if args.len() != 1 {
                return Err(format!("math.{} requires exactly 1 argument", attr));
            }
            let val0 = generate_expr(state, &args[0])?;
            let ptr0 = get_bigint(val0)?;
            let func = state.module.get_function("vp_bigint_factorial").ok_or_else(|| "vp_bigint_factorial not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[result_ptr.into(), ptr0.into()], "factorial_call");
        },
        _ => return Err(format!("Unsupported math function for BigInt: {}", attr)),
    }

    Ok(result_ptr.into())
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
            // Convert i8 to bool (i1) for select instruction
            let is_ok_bool = state.builder.build_int_compare(
                inkwell::IntPredicate::NE,
                is_ok,
                state.context.i8_type().const_zero(),
                "is_ok_bool",
            ).map_err(|e| format!("Failed to build compare: {:?}", e))?;

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
                is_ok_bool,
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
            // Convert i8 to bool (i1) for select instruction
            let is_ok_bool = state.builder.build_int_compare(
                inkwell::IntPredicate::NE,
                is_ok,
                state.context.i8_type().const_zero(),
                "is_ok_bool_default",
            ).map_err(|e| format!("Failed to build compare: {:?}", e))?;

            // Extract value from Result
            let result_value = state.builder
                .build_extract_value(result_struct, 1, "result_value")
                .map_err(|e| format!("Failed to extract value: {:?}", e))?
                .into_int_value();

            // Default is 0
            let default_value = state.context.i64_type().const_zero();

            // Select based on is_ok
            let selected = state.builder.build_select(
                is_ok_bool,
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
    
    // Use custom build_call wrapper which returns Option<BasicValueEnum>
    let str_val = state
        .ir_builder
        .build_call(state.builder, to_str_func, &[bigint_val.into(), base.into()], "bigint_to_str")
        .ok_or_else(|| "vp_bigint_to_str did not return a value".to_string())?;

    Ok(str_val)
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

    // Use custom build_call wrapper which returns Option<BasicValueEnum>
    let i64_val = state
        .ir_builder
        .build_call(state.builder, to_i64_func, &[bigint_val.into()], "bigint_to_i64")
        .ok_or_else(|| "vp_bigint_to_i64 did not return a value".to_string())?;

    Ok(i64_val)
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

    // For Result types, we use representation:
    // { is_ok: i8, value: i64 } where value is bitcast to i64 if needed
    let result_struct_type = state.context.struct_type(&[
        state.context.i8_type().into(),
        state.context.i64_type().into(),
    ], false);

    // Allocate space for the Result struct
    let result_alloca = state.builder.build_alloca(result_struct_type, "ok_result").expect("alloca");

    // Store is_ok = 1
    let is_ok_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_zero()],
            "is_ok_ptr",
        )
    }.map_err(|e| format!("Failed to get is_ok field: {:?}", e))?;
    state.builder.build_store(is_ok_ptr, state.context.i8_type().const_int(1, false)).expect("store");

    // Convert value to i64 representation and store
    let value_i64_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_int(1, false)],
            "value_i64_ptr",
        )
    }.map_err(|e| format!("Failed to get value field: {:?}", e))?;

    // Convert value to i64 representation
    let value_i64 = if value.is_int_value() {
        value.into_int_value()
    } else if value.is_float_value() {
        state.builder.build_float_to_unsigned_int(
            value.into_float_value(),
            state.context.i64_type(),
            "ok_f64_to_i64",
        ).map_err(|e| format!("Failed to convert f64 to i64: {:?}", e))?
    } else if value.is_pointer_value() {
        state.builder.build_ptr_to_int(
            value.into_pointer_value(),
            state.context.i64_type(),
            "ok_ptr_to_i64",
        ).map_err(|e| format!("Failed to convert ptr to i64: {:?}", e))?
    } else {
        return Err(format!("Unsupported Ok value type: {:?}", value.get_type()));
    };

    state.builder.build_store(value_i64_ptr, value_i64).expect("store");

    // Load and return the struct value
    let result_val = state.builder.build_load(result_struct_type, result_alloca, "ok_result_val").expect("load");
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

    // For Result types, we use representation:
    // { is_ok: i8, error: i64 } where error is bitcast to i64 if needed
    let result_struct_type = state.context.struct_type(&[
        state.context.i8_type().into(),
        state.context.i64_type().into(),
    ], false);

    // Allocate space for the Result struct
    let result_alloca = state.builder.build_alloca(result_struct_type, "err_result").expect("alloca");

    // Store is_ok = 0
    let is_ok_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_zero()],
            "is_ok_ptr",
        )
    }.map_err(|e| format!("Failed to get is_ok field: {:?}", e))?;
    state.builder.build_store(is_ok_ptr, state.context.i8_type().const_int(0, false)).expect("store");

    // Convert error to i64 representation and store
    let error_i64_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_int(1, false)],
            "error_i64_ptr",
        )
    }.map_err(|e| format!("Failed to get error field: {:?}", e))?;

    // Convert error to i64 representation
    let error_i64 = if error.is_pointer_value() {
        state.builder.build_ptr_to_int(
            error.into_pointer_value(),
            state.context.i64_type(),
            "err_ptr_to_i64",
        ).map_err(|e| format!("Failed to convert ptr to i64: {:?}", e))?
    } else if error.is_int_value() {
        error.into_int_value()
    } else if error.is_float_value() {
        state.builder.build_float_to_unsigned_int(
            error.into_float_value(),
            state.context.i64_type(),
            "err_f64_to_i64",
        ).map_err(|e| format!("Failed to convert f64 to i64: {:?}", e))?
    } else {
        return Err(format!("Unsupported Err value type: {:?}", error.get_type()));
    };

    state.builder.build_store(error_i64_ptr, error_i64).expect("store");

    // Load and return the struct value
    let result_val = state.builder.build_load(result_struct_type, result_alloca, "err_result_val").expect("load");
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

/// Generate super().method() call - resolves method through MRO
fn generate_super_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    method_name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Get the current class from state (set when generating class methods)
    let class_name = state.current_class.clone()
        .ok_or_else(|| "super() can only be used inside a class method".to_string())?;
    
    // Get the current class metadata
    let metadata = crate::codegen::oop::with_class_registry(|reg| {
        reg.get_class(&class_name).cloned()
    }).ok_or_else(|| format!("Class '{}' not found", class_name))?;
    
    // Find the method in parent classes via MRO
    // Skip the first entry in MRO (which is the current class itself)
    let mut found_method = None;
    for mro_class_name in metadata.mro.iter().skip(1) {
        if let Some(method) = crate::codegen::oop::with_class_registry(|reg| {
            reg.get_class(mro_class_name).and_then(|c| c.get_method(method_name).cloned())
        }) {
            found_method = Some(method);
            break;
        }
    }
    
    let method = found_method
        .ok_or_else(|| format!("Method '{}' not found in parent classes", method_name))?;

    // Get self from the function's first parameter
    let current_function = state.builder.get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or_else(|| "Not inside a function".to_string())?;

    let self_ptr = current_function.get_nth_param(0)
        .ok_or_else(|| "Method should have self parameter".to_string())?
        .into_pointer_value();

    // Build argument list: self + user args
    let mut arg_values: Vec<_> = args.iter()
        .map(|a| crate::codegen::expressions::generate_expr(state, a)
            .map(|v| inkwell::values::BasicMetadataValueEnum::from(v)))
        .collect::<Result<_, _>>()?;

    // Insert self as first argument
    arg_values.insert(0, self_ptr.into());

    // Call the parent method
    if let Some(func_val) = state.functions.get(&method.mangled_name).copied() {
        let result = state.ir_builder.build_call(
            state.builder,
            func_val,
            &arg_values,
            &format!("super_call_{}", method_name),
        );

        Ok(result.unwrap_or(state.context.i64_type().const_int(0, false).into()))
    } else {
        Err(format!("Parent method '{}' not found", method_name))
    }
}

/// Generate isinstance() check for runtime type narrowing
/// isinstance(obj, Type) returns bool indicating if obj is of Type
pub fn generate_isinstance_check<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("isinstance() takes exactly 2 arguments".to_string());
    }

    // Generate the object expression
    let obj_val = crate::codegen::expressions::generate_expr(state, &args[0])?;

    // Get the type name from the second argument (should be a type identifier or None)
    let type_name = match &args[1] {
        Expr::Ident(name, _) => name.clone(),
        Expr::None(_) => "None".to_string(),  // Handle None literal
        _ => return Err("isinstance() second argument must be a type name".to_string()),
    };
    
    // For now, implement basic type checks based on the expected type
    // In a full implementation, this would use runtime type information
    
    // Check if we're checking against primitive types
    let result = match type_name.as_str() {
        "i64" | "i32" | "i16" | "i8" | "int" => {
            // Check if value is an integer type
            if obj_val.is_int_value() {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "f64" | "f32" | "float" => {
            // Check if value is a float type
            if obj_val.is_float_value() {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "bool" => {
            // Check if value is a bool (i1)
            if obj_val.is_int_value() && obj_val.get_type().into_int_type().get_bit_width() == 1 {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "str" => {
            // Check if value is a string (pointer)
            if obj_val.is_pointer_value() {
                // For strings, we'd need to check the actual runtime type
                // For now, assume pointers could be strings
                // A full implementation would check the type tag
                state.context.bool_type().const_int(1, false)  // Conservative: assume true for pointers
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "list" => {
            // Check if value is a list (pointer to list struct)
            if obj_val.is_pointer_value() {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "dict" => {
            if obj_val.is_pointer_value() {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "None" => {
            // Check if value is null pointer or special None value (i64 0)
            if obj_val.is_pointer_value() {
                let ptr = obj_val.into_pointer_value();
                let null_ptr = state.context.ptr_type(inkwell::AddressSpace::default()).const_null();
                // Convert pointers to integers for comparison
                let intptr_type = state.context.i64_type();
                let ptr_int = state.builder.build_ptr_to_int(ptr, intptr_type, "ptr_int")
                    .map_err(|e| format!("Failed to convert ptr to int: {:?}", e))?;
                let null_int = state.builder.build_ptr_to_int(null_ptr, intptr_type, "null_int")
                    .map_err(|e| format!("Failed to convert null to int: {:?}", e))?;
                let is_null = state.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    ptr_int,
                    null_int,
                    "is_none",
                ).map_err(|e| format!("Failed to compare: {:?}", e))?;
                is_null
            } else if obj_val.is_int_value() {
                // None is represented as i64(0)
                let zero = state.context.i64_type().const_zero();
                let is_none = state.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    obj_val.into_int_value(),
                    zero,
                    "is_none_int",
                ).map_err(|e| format!("Failed to compare: {:?}", e))?;
                is_none
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        // For class types, we'd need runtime type information
        // This would check the type tag in the object header
        _ => {
            // For user-defined classes, we need to check the runtime type
            // This requires RTTI (runtime type information)
            // For now, return a conservative false
            state.context.bool_type().const_int(0, false)
        }
    };
    
    Ok(result.into())
}

/* ============================================ */
/* Iteration Builtins                           */
/* ============================================ */

/// Generate enumerate() call
pub fn generate_enumerate_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("enumerate() requires at least 1 argument".to_string());
    }
    
    let iterable_val = generate_expr(state, &args[0])?;
    let start = if args.len() > 1 {
        generate_expr(state, &args[1])?.into_int_value()
    } else {
        state.ir_builder.i64_const(0)
    };
    
    let func = state
        .module
        .get_function("vp_enumerate")
        .ok_or_else(|| "vp_enumerate not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[iterable_val.into(), start.into()], "enumerate_result");
    Ok(result.unwrap())
}

/// Generate zip() call
pub fn generate_zip_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() < 2 {
        return Err("zip() requires at least 2 arguments".to_string());
    }
    
    let iter1_val = generate_expr(state, &args[0])?;
    let iter2_val = generate_expr(state, &args[1])?;
    
    let func = state
        .module
        .get_function("vp_zip")
        .ok_or_else(|| "vp_zip not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[iter1_val.into(), iter2_val.into()], "zip_result");
    Ok(result.unwrap())
}

/* ============================================ */
/* Functional Builtins                          */
/* ============================================ */

/// Generate sum() call
pub fn generate_sum_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("sum() requires at least 1 argument".to_string());
    }
    
    let iterable_val = generate_expr(state, &args[0])?;
    
    // Use i64 sum for now
    let func = state
        .module
        .get_function("vp_list_sum")
        .ok_or_else(|| "vp_list_sum not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[iterable_val.into()], "sum_result");
    Ok(result.unwrap())
}

/// Generate min() call
pub fn generate_min_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("min() requires at least 1 argument".to_string());
    }
    
    let iterable_val = generate_expr(state, &args[0])?;
    
    let func = state
        .module
        .get_function("vp_list_min")
        .ok_or_else(|| "vp_list_min not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[iterable_val.into()], "min_result");
    Ok(result.unwrap())
}

/// Generate max() call
pub fn generate_max_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("max() requires at least 1 argument".to_string());
    }
    
    let iterable_val = generate_expr(state, &args[0])?;
    
    let func = state
        .module
        .get_function("vp_list_max")
        .ok_or_else(|| "vp_list_max not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[iterable_val.into()], "max_result");
    Ok(result.unwrap())
}

/// Generate any() call
pub fn generate_any_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("any() requires at least 1 argument".to_string());
    }
    
    let iterable_val = generate_expr(state, &args[0])?;
    
    let func = state
        .module
        .get_function("vp_list_any")
        .ok_or_else(|| "vp_list_any not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[iterable_val.into()], "any_result");
    Ok(result.unwrap())
}

/// Generate all() call
pub fn generate_all_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("all() requires at least 1 argument".to_string());
    }
    
    let iterable_val = generate_expr(state, &args[0])?;
    
    let func = state
        .module
        .get_function("vp_list_all")
        .ok_or_else(|| "vp_list_all not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[iterable_val.into()], "all_result");
    Ok(result.unwrap())
}

/* ============================================ */
/* Numeric Builtins                             */
/* ============================================ */

/// Generate round() call
pub fn generate_round_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("round() requires at least 1 argument".to_string());
    }
    
    let number_val = generate_expr(state, &args[0])?;
    let ndigits = if args.len() > 1 {
        generate_expr(state, &args[1])?.into_int_value()
    } else {
        state.ir_builder.i64_const(0)
    };
    
    let number_float = if number_val.is_float_value() {
        number_val.into_float_value()
    } else {
        state.builder.build_signed_int_to_float(
            number_val.into_int_value(),
            state.context.f64_type(),
            "int_to_float",
        ).expect("int to float")
    };
    
    let func = state
        .module
        .get_function("vp_round_f64")
        .ok_or_else(|| "vp_round_f64 not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[number_float.into(), ndigits.into()], "round_result");
    Ok(result.unwrap())
}

/// Generate divmod() call
pub fn generate_divmod_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("divmod() requires exactly 2 arguments".to_string());
    }
    
    let a_val = generate_expr(state, &args[0])?.into_int_value();
    let b_val = generate_expr(state, &args[1])?.into_int_value();
    
    let func = state
        .module
        .get_function("vp_divmod_i64")
        .ok_or_else(|| "vp_divmod_i64 not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[a_val.into(), b_val.into()], "divmod_result");
    Ok(result.unwrap())
}

/// Generate pow() call
pub fn generate_pow_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err("pow() requires 2 or 3 arguments".to_string());
    }
    
    let base_val = generate_expr(state, &args[0])?;
    let exp_val = generate_expr(state, &args[1])?;
    
    let is_bigint = args.iter().any(|arg| {
        let arg_type = infer_expr_type(arg);
        arg_type == Type::BigInt || matches!(arg, Expr::Ident(n, _) if state.is_bigint(n))
    }) || base_val.is_pointer_value() || exp_val.is_pointer_value();

    if is_bigint {
        let from_i64_func = state
            .module
            .get_function("vp_bigint_from_i64")
            .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
            
        let get_bigint = |val: BasicValueEnum<'ctx>| -> Result<BasicValueEnum<'ctx>, String> {
            if val.is_pointer_value() {
                Ok(val)
            } else if val.is_int_value() {
                let res = state.ir_builder.build_call(state.builder, from_i64_func, &[val.into()], "bigint_from_i64")
                    .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?;
                Ok(res.into_pointer_value().into())
            } else {
                Err("Cannot convert to BigInt for pow()".to_string())
            }
        };

        let base_ptr = get_bigint(base_val)?;
        let exp_ptr = get_bigint(exp_val)?;
        
        let zero = state.ir_builder.i64_const(0);
        let result_ptr = state
            .ir_builder
            .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
            .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
            .into_pointer_value();

        if args.len() == 3 {
            let mod_expr = generate_expr(state, &args[2])?;
            let mod_ptr = get_bigint(mod_expr)?;
            let powmod_func = state
                .module
                .get_function("vp_bigint_powmod")
                .ok_or_else(|| "vp_bigint_powmod not declared".to_string())?;
            
            state.ir_builder.build_call(
                state.builder,
                powmod_func,
                &[result_ptr.into(), base_ptr.into(), exp_ptr.into(), mod_ptr.into()],
                "bigint_powmod_call",
            );
        } else {
            let pow_func = state
                .module
                .get_function("vp_bigint_pow")
                .ok_or_else(|| "vp_bigint_pow not declared".to_string())?;
            
            state.ir_builder.build_call(
                state.builder,
                pow_func,
                &[result_ptr.into(), base_ptr.into(), exp_ptr.into()],
                "bigint_pow_call",
            );
        }
        return Ok(result_ptr.into());
    }

    if args.len() == 3 {
        return Err("3-argument pow() only supported for BigInt types currently".to_string());
    }
    
    // Use float pow for now
    let base_float = if base_val.is_float_value() {
        base_val.into_float_value()
    } else {
        state.builder.build_signed_int_to_float(
            base_val.into_int_value(),
            state.context.f64_type(),
            "int_to_float",
        ).expect("int to float")
    };
    
    let exp_float = if exp_val.is_float_value() {
        exp_val.into_float_value()
    } else {
        state.builder.build_signed_int_to_float(
            exp_val.into_int_value(),
            state.context.f64_type(),
            "int_to_float",
        ).expect("int to float")
    };
    
    let func = state
        .module
        .get_function("vp_pow_f64")
        .ok_or_else(|| "vp_pow_f64 not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[base_float.into(), exp_float.into()], "pow_result");
    Ok(result.unwrap())
}

/* ============================================ */
/* Introspection Builtins                       */
/* ============================================ */

/// Generate type() call
pub fn generate_type_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("type() requires at least 1 argument".to_string());
    }
    
    let obj_val = generate_expr(state, &args[0])?;
    
    let func = state
        .module
        .get_function("vp_type_of")
        .ok_or_else(|| "vp_type_of not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[obj_val.into()], "type_result");
    Ok(result.unwrap())
}

/// Generate id() call
pub fn generate_id_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("id() requires at least 1 argument".to_string());
    }
    
    let obj_val = generate_expr(state, &args[0])?;
    
    // For non-pointer types, just return the value as-is (as identity)
    if obj_val.is_int_value() {
        return Ok(obj_val);
    }
    if obj_val.is_float_value() {
        // Convert float bits to int
        let float_val = obj_val.into_float_value();
        let int_val = state.builder.build_float_to_signed_int(
            float_val,
            state.context.i64_type(),
            "float_to_int_id",
        ).expect("float to int");
        return Ok(int_val.into());
    }
    
    // For pointers, return the pointer address as int
    let func = state
        .module
        .get_function("vp_object_id")
        .ok_or_else(|| "vp_object_id not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[obj_val.into()], "id_result");
    Ok(result.unwrap())
}

/// Generate repr() call
pub fn generate_repr_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("repr() requires at least 1 argument".to_string());
    }
    
    let obj_val = generate_expr(state, &args[0])?;
    
    let func_name = if obj_val.is_int_value() {
        "vp_repr_i64"
    } else if obj_val.is_float_value() {
        "vp_repr_f64"
    } else if obj_val.is_pointer_value() {
        "vp_repr_str"
    } else {
        "vp_repr_i64"
    };
    
    let func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[obj_val.into()], "repr_result");
    Ok(result.unwrap())
}

/* ============================================ */
/* Attribute Builtins                           */
/* ============================================ */

/// Generate hasattr() call
pub fn generate_hasattr_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("hasattr() requires exactly 2 arguments".to_string());
    }
    
    let obj_val = generate_expr(state, &args[0])?;
    let name_val = generate_expr(state, &args[1])?;
    
    let func = state
        .module
        .get_function("vp_hasattr")
        .ok_or_else(|| "vp_hasattr not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[obj_val.into(), name_val.into()], "hasattr_result");
    Ok(result.unwrap())
}

/// Generate getattr() call - placeholder
pub fn generate_getattr_call<'ctx>(
    _state: &mut CodeGenState<'_, 'ctx>,
    _args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    Err("getattr() not yet implemented".to_string())
}

/// Generate setattr() call - placeholder
pub fn generate_setattr_call<'ctx>(
    _state: &mut CodeGenState<'_, 'ctx>,
    _args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    Err("setattr() not yet implemented".to_string())
}

/// Generate delattr() call - placeholder
pub fn generate_delattr_call<'ctx>(
    _state: &mut CodeGenState<'_, 'ctx>,
    _args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    Err("delattr() not yet implemented".to_string())
}

/* ============================================ */
/* Conversion Builtins                          */
/* ============================================ */

/// Generate bin() call
pub fn generate_bin_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("bin() requires at least 1 argument".to_string());
    }
    
    let num_val = generate_expr(state, &args[0])?.into_int_value();
    
    let func = state
        .module
        .get_function("vp_bin_i64")
        .ok_or_else(|| "vp_bin_i64 not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[num_val.into()], "bin_result");
    Ok(result.unwrap())
}

/// Generate oct() call
pub fn generate_oct_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("oct() requires at least 1 argument".to_string());
    }
    
    let num_val = generate_expr(state, &args[0])?.into_int_value();
    
    let func = state
        .module
        .get_function("vp_oct_i64")
        .ok_or_else(|| "vp_oct_i64 not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[num_val.into()], "oct_result");
    Ok(result.unwrap())
}

/// Generate hex() call
pub fn generate_hex_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("hex() requires at least 1 argument".to_string());
    }
    
    let num_val = generate_expr(state, &args[0])?.into_int_value();
    
    let func = state
        .module
        .get_function("vp_hex_i64")
        .ok_or_else(|| "vp_hex_i64 not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[num_val.into()], "hex_result");
    Ok(result.unwrap())
}

/// Generate chr() call
pub fn generate_chr_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("chr() requires at least 1 argument".to_string());
    }
    
    let num_val = generate_expr(state, &args[0])?.into_int_value();
    
    let func = state
        .module
        .get_function("vp_chr_i64")
        .ok_or_else(|| "vp_chr_i64 not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[num_val.into()], "chr_result");
    Ok(result.unwrap())
}

/// Generate ord() call
pub fn generate_ord_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("ord() requires at least 1 argument".to_string());
    }
    
    let str_val = generate_expr(state, &args[0])?;
    
    let func = state
        .module
        .get_function("vp_ord_str")
        .ok_or_else(|| "vp_ord_str not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[str_val.into()], "ord_result");
    Ok(result.unwrap())
}

/* ============================================ */
/* I/O Builtins                                 */
/* ============================================ */

/// Generate input() call
pub fn generate_input_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    let prompt_val = if args.is_empty() {
        state.ir_builder.string_const(state.module, "").into()
    } else {
        generate_expr(state, &args[0])?
    };
    
    let func = state
        .module
        .get_function("vp_input")
        .ok_or_else(|| "vp_input not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[prompt_val.into()], "input_result");
    Ok(result.unwrap())
}

/* ============================================ */
/* Advanced Builtins                            */
/* ============================================ */

/// Generate callable() call
pub fn generate_callable_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("callable() requires at least 1 argument".to_string());
    }
    
    let obj_val = generate_expr(state, &args[0])?;
    
    let func = state
        .module
        .get_function("vp_is_callable")
        .ok_or_else(|| "vp_is_callable not declared".to_string())?;
    
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[obj_val.into()], "callable_result");
    Ok(result.unwrap())
}

/// Generate dict() call
pub fn generate_dict_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // dict() with no args returns empty dict
    if args.is_empty() {
        let func = state
            .module
            .get_function("vp_dict_create_empty")
            .ok_or_else(|| "vp_dict_create_empty not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, func, &[], "empty_dict");
        return Ok(result.unwrap());
    }
    
    // For now, just return empty dict - full implementation would convert iterable
    let func = state
        .module
        .get_function("vp_dict_create_empty")
        .ok_or_else(|| "vp_dict_create_empty not declared".to_string())?;
    let result = state
        .ir_builder
        .build_call(state.builder, func, &[], "dict_from_iter");
    Ok(result.unwrap())
}

