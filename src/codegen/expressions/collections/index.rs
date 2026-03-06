//! Index access logic for Viper

use inkwell::values::BasicValueEnum;

use crate::ast::{Expr, Type};
use crate::codegen::state::CodeGenState;

use crate::codegen::expressions::generate_expr;

/// Generate index access
pub fn generate_index<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    index: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let obj_val = generate_expr(state, obj)?;
    let index_val = generate_expr(state, index)?;

    // Check if indexing with a string key (dict access)
    if index_val.is_pointer_value() && obj_val.is_pointer_value() {
        let dict_get = state
            .module
            .get_function("vp_dict_get_i64")
            .ok_or_else(|| "vp_dict_get_i64 not declared".to_string())?;

        let result = state
            .ir_builder
            .build_call(state.builder, dict_get, &[obj_val.into(), index_val.into()], "dict_get")
            .ok_or_else(|| "build call failed".to_string())?;

        return Ok(result);
    }

    // Handle tuple indexing
    // Tuples are now heap-allocated ViperTuple* pointers
    // Check the AST type annotation, not just the LLVM value type
    let is_tuple_type = match obj {
        Expr::Ident(name, _) => {
            // Check if the variable type is a tuple
            if let Some(var_type) = state.var_types.get(name) {
                matches!(var_type, Type::Tuple(_))
            } else {
                false
            }
        },
        Expr::Tuple { .. } => true,
        _ => false,
    };

    if is_tuple_type && obj_val.is_pointer_value() {
        // Use runtime function vp_tuple_get(tuple: ViperTuple*, index: i64) -> i64
        let tuple_get_func = state
            .module
            .get_function("vp_tuple_get")
            .ok_or_else(|| "vp_tuple_get not declared".to_string())?;

        let index_int = index_val.into_int_value();

        // Handle constant index with bounds checking
        if index_int.is_const() {
            if let Some(const_index) = index_int.get_zero_extended_constant() {
                // Get tuple size for bounds checking
                let tuple_size = match obj {
                    Expr::Ident(name, _) => {
                        if let Some(var_type) = state.var_types.get(name) {
                            if let Type::Tuple(element_types) = var_type {
                                element_types.len() as i64
                            } else {
                                return Err("Tuple variable has non-tuple type".to_string());
                            }
                        } else {
                            return Err("Tuple variable type not found".to_string());
                        }
                    },
                    Expr::Tuple { elements, .. } => elements.len() as i64,
                    _ => return Err("Tuple size unknown".to_string()),
                };

                // Convert negative index to positive
                let actual_index = if (const_index as i64) < 0 {
                    tuple_size + const_index as i64
                } else {
                    const_index as i64
                };

                if actual_index < 0 || actual_index >= tuple_size {
                    return Err(format!("Tuple index {} out of range (size {})", const_index, tuple_size));
                }
            }
        }

        // Call vp_tuple_get
        let result = state
            .ir_builder
            .build_call(state.builder, tuple_get_func, &[obj_val.into(), index_val.into()], "tuple_get")
            .ok_or_else(|| "Failed to call vp_tuple_get".to_string())?;

        return Ok(result);
    }

    // Dynamic tuple index - use runtime function
    // Only for actual tuple types, not lists
    let is_tuple_var = match obj {
        Expr::Ident(name, _) => {
            if let Some(var_type) = state.var_types.get(name) {
                matches!(var_type, Type::Tuple(_))
            } else {
                false
            }
        },
        Expr::Tuple { .. } => true,
        _ => false,
    };

    if obj_val.is_pointer_value() && is_tuple_var {
        let tuple_get_func = state
            .module
            .get_function("vp_tuple_get")
            .ok_or_else(|| "vp_tuple_get not declared".to_string())?;

        let result = state
            .ir_builder
            .build_call(state.builder, tuple_get_func, &[obj_val.into(), index_val.into()], "tuple_get")
            .ok_or_else(|| "Failed to call vp_tuple_get".to_string())?;

        return Ok(result);
    }

    let index_val = index_val.into_int_value();

    // Check if this is a list by examining the object
    let is_list = match obj {
        Expr::Ident(obj_name, _) => state.is_list(obj_name),
        Expr::List { .. } | Expr::ListComprehension { .. } => true,
        Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
            // Handle [bool] * n pattern
            matches!(left.as_ref(), Expr::List { .. })
        }
        _ => false,
    };

    // Check if this is a bool list
    let is_bool_list = match obj {
        Expr::Ident(obj_name, _) => state.is_bool_list(obj_name),
        Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
        Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
            // Handle [bool] * n pattern
            if let Expr::List { elements, .. } = left.as_ref() {
                elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
            } else {
                false
            }
        }
        _ => false,
    };

    let is_float_list = {
        let obj_type = crate::codegen::expressions::infer_expr_type(obj);
        let obj_type = if let Expr::Ident(name, _) = obj {
            state.var_types.get(name).cloned().unwrap_or(obj_type)
        } else {
            obj_type
        };

        match &obj_type {
            Type::List(inner) => match &**inner {
                Type::F64 => true,
                Type::Var(n) if n == "float" || n == "f64" => true,
                _ => false,
            },
            Type::GenericApp { name, type_args } if (name == "list" || name == "List") && type_args.len() == 1 => {
                match &type_args[0] {
                    Type::F64 => true,
                    Type::Var(n) if n == "float" || n == "f64" => true,
                    _ => false,
                }
            }
            _ => false,
        }
    };

    // For pointer-typed objects, distinguish between lists and other pointers (strings, etc.)
    let is_pointer_type = obj_val.is_pointer_value();

    // Lists need to use vp_list_get because they have a ViperList struct wrapper
    // Other pointers (strings, arrays) use array GEP
    if is_pointer_type && is_list {
        // Use bit vector get for bool lists (more memory efficient)
        // Note: Inline operations disabled due to JIT/AOT struct layout differences
        if is_bool_list {
            // Try unchecked version first (faster), fall back to checked
            let bitvec_get = state.module.get_function("vp_bitvec_get_unchecked")
                .or_else(|| state.module.get_function("vp_bitvec_get"))
                .ok_or_else(|| "vp_bitvec_get not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(state.builder, bitvec_get, &[obj_val.into(), index_val.into()], "bitvec_get")
                .ok_or_else(|| "build call failed".to_string())?;

            // Convert bool to i64 for compatibility with print() and other functions
            let bool_val = result.into_int_value();
            let i64_val = state
                .builder
                .build_int_z_extend(bool_val, state.context.i64_type(), "bool_to_i64")
                .map_err(|e| format!("Failed to extend bool to i64: {:?}", e))?;

            return Ok(i64_val.into());
        }

        if is_float_list {
            let list_get = state.module.get_function("vp_list_get_f64").ok_or_else(|| "vp_list_get_f64 not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(state.builder, list_get, &[obj_val.into(), index_val.into()], "list_get")
                .ok_or_else(|| "build call failed".to_string())?;

            return Ok(result);
        }

        // For now, use the generic vp_list_get for non-bool lists.
        let list_get = state.module.get_function("vp_list_get").ok_or_else(|| "vp_list_get not declared".to_string())?;

        let result = state
            .ir_builder
            .build_call(state.builder, list_get, &[obj_val.into(), index_val.into()], "list_get")
            .ok_or_else(|| "build call failed".to_string())?;

        return Ok(result);
    }

    // For non-list pointers (strings, arrays), use array indexing
    if is_pointer_type {
        let obj_ptr = obj_val.into_pointer_value();
        // For strings, element type is i8 (char); for arrays, it depends on the array type
        // Default to i8 for strings
        let elem_type = state.context.i8_type();

        let elem_ptr = unsafe {
            state.builder.build_in_bounds_gep(elem_type, obj_ptr, &[index_val], "array_elem")
        }
        .map_err(|e| format!("Failed to build array index GEP: {:?}", e))?;

        let loaded = state
            .builder
            .build_load(elem_type, elem_ptr, "array_load")
            .map_err(|e| format!("Failed to load array element: {:?}", e))?;

        // Cast i8 to i64 for compatibility with print() and other functions
        let int_val = loaded.into_int_value();

        let extended = state
            .builder
            .build_int_z_extend(int_val, state.context.i64_type(), "char_to_i64")
            .map_err(|e| format!("Failed to extend char to i64: {:?}", e))?;

        return Ok(extended.into());
    }

    // Fall back to list indexing for any remaining cases
    let list_get = state
        .module
        .get_function("vp_list_get")
        .ok_or_else(|| "vp_list_get not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, list_get, &[obj_val.into(), index_val.into()], "list_get")
        .ok_or_else(|| "build call failed".to_string())?;

    Ok(result)
}
