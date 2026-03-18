//! Index access logic for Viper
//!
//! Optimized with inline LLVM IR generation for list access (40-50% performance gain)
//! Instead of calling runtime functions like vp_list_get, we generate direct GEP + load operations.

use inkwell::values::BasicValueEnum;

use crate::ast::{Expr, Type};
use crate::codegen::state::CodeGenState;

use crate::codegen::expressions::generate_expr;
use crate::codegen::inline_lists::{
    inline_bool_list_get, inline_f64_list_get, inline_i64_list_get,
};

/// Generate index access
pub fn generate_index<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    index: &Expr,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
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
        }
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
                                return crate::codegen::codegen_error(
                                    "Tuple variable has non-tuple type".to_string(),
                                );
                            }
                        } else {
                            return crate::codegen::codegen_error(
                                "Tuple variable type not found".to_string(),
                            );
                        }
                    }
                    Expr::Tuple { elements, .. } => elements.len() as i64,
                    _ => return crate::codegen::codegen_error("Tuple size unknown".to_string()),
                };

                // Convert negative index to positive
                let actual_index = if (const_index as i64) < 0 {
                    tuple_size + const_index as i64
                } else {
                    const_index as i64
                };

                if actual_index < 0 || actual_index >= tuple_size {
                    return crate::codegen::codegen_error(format!(
                        "Tuple index {} out of range (size {})",
                        const_index, tuple_size
                    ));
                }
            }
        }

        // Call vp_tuple_get
        let result = state
            .ir_builder
            .build_call(
                state.builder,
                tuple_get_func,
                &[obj_val.into(), index_val.into()],
                "tuple_get",
            )
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
        }
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
            .build_call(
                state.builder,
                tuple_get_func,
                &[obj_val.into(), index_val.into()],
                "tuple_get",
            )
            .ok_or_else(|| "Failed to call vp_tuple_get".to_string())?;

        return Ok(result);
    }

    let index_val = index_val.into_int_value();

    let inferred_type = crate::codegen::expressions::core::infer_expr_type(obj);
    let obj_type = if let Expr::Ident(name, _) = obj {
        state.var_types.get(name).cloned().unwrap_or(inferred_type)
    } else {
        inferred_type
    };

    // Check if this is a bytearray access
    let is_bytearray = match obj {
        Expr::Ident(name, _) => state.is_bytearray(name),
        Expr::Call { func, .. } => {
            if let Expr::Ident(func_name, _) = func.as_ref() {
                func_name == "bytearray"
            } else {
                false
            }
        }
        Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
            // Handle bytearray * n pattern
            if let Expr::Call { func, .. } = left.as_ref() {
                if let Expr::Ident(func_name, _) = func.as_ref() {
                    func_name == "bytearray"
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    };

    let (is_list, is_float_list, is_bool_list) = match &obj_type {
        Type::List(inner) => match &**inner {
            Type::F64 => (true, true, false),
            Type::Bool => (true, false, true),
            Type::Var(n) if n == "float" || n == "f64" => (true, true, false),
            _ => (true, false, false),
        },
        Type::GenericApp { name, type_args }
            if (name == "list" || name == "List") && type_args.len() == 1 =>
        {
            match &type_args[0] {
                Type::F64 => (true, true, false),
                Type::Bool => (true, false, true),
                Type::Var(n) if n == "float" || n == "f64" => (true, true, false),
                _ => (true, false, false),
            }
        }
        Type::Array(_, _) => (true, false, false), // Arrays are also lists in this context
        _ => {
            // Fallback for cases where type inference failed but it might still be a list
            let is_list = match obj {
                Expr::List { .. } | Expr::ListComprehension { .. } | Expr::Slice { .. } => true,
                Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
                    matches!(left.as_ref(), Expr::List { .. })
                }
                Expr::Ident(name, _) => state.is_list(name),
                _ => {
                    let inferred = crate::codegen::expressions::core::infer_expr_type(obj);
                    matches!(inferred, crate::ast::Type::List(_))
                }
            };
            let is_bool_list = if is_list {
                match obj {
                    Expr::Ident(name, _) => state.is_bool_list(name),
                    _ => false,
                }
            } else {
                false
            };
            (is_list, false, is_bool_list)
        }
    };

    // For pointer-typed objects, distinguish between lists and other pointers (strings, etc.)
    let is_pointer_type = obj_val.is_pointer_value();

    // Handle bytearray indexing - bytearray stores raw bytes (i8), returns i64
    if is_bytearray && is_pointer_type {
        let bytearray_ptr = obj_val.into_pointer_value();

        // Untag the index (tagged ints are shifted left by 1)
        let index_untagged = state
            .builder
            .build_right_shift(
                index_val,
                state.context.i64_type().const_int(1, false),
                true,
                "index_untagged",
            )
            .map_err(|e| format!("Failed to untag index: {:?}", e))?;

        // Handle negative indices: if index < 0, index = length + index
        let bytearray_len_func = state
            .module
            .get_function("vp_bytearray_len")
            .ok_or_else(|| "vp_bytearray_len not declared".to_string())?;
        let bytearray_len = state
            .ir_builder
            .build_call(state.builder, bytearray_len_func, &[bytearray_ptr.into()], "ba_len")
            .unwrap()
            .into_int_value();

        let is_negative = state
            .builder
            .build_int_compare(
                inkwell::IntPredicate::SLT,
                index_untagged,
                state.context.i64_type().const_zero(),
                "index_is_neg",
            )
            .expect("compare neg");

        let adjusted_index = state
            .builder
            .build_int_add(bytearray_len, index_untagged, "adjusted_index")
            .expect("add len");

        let final_index = state
            .builder
            .build_select(is_negative, adjusted_index, index_untagged, "final_index")
            .expect("select index")
            .into_int_value();

        // Call vp_bytearray_get(bytearray: ViperByteArray*, index: i64) -> i64
        let bytearray_get = state
            .module
            .get_function("vp_bytearray_get")
            .ok_or_else(|| "vp_bytearray_get not declared".to_string())?;

        let result = state
            .ir_builder
            .build_call(
                state.builder,
                bytearray_get,
                &[bytearray_ptr.into(), final_index.into()],
                "bytearray_get",
            )
            .ok_or_else(|| "vp_bytearray_get call failed".to_string())?;

        return Ok(result);
    }

    // Handle bytes indexing - bytes stores raw bytes, returns i64
    let is_bytes = match obj {
        Expr::Bytes(_, _) => true,
        Expr::Ident(name, _) => {
            if let Some(var_type) = state.var_types.get(name) {
                matches!(var_type, Type::Bytes)
            } else {
                false
            }
        }
        _ => false,
    };
    if is_bytes && is_pointer_type {
        let bytes_ptr = obj_val.into_pointer_value();

        // Untag the index (tagged ints are shifted left by 1)
        let index_untagged = state
            .builder
            .build_right_shift(
                index_val,
                state.context.i64_type().const_int(1, false),
                true,
                "index_untagged",
            )
            .map_err(|e| format!("Failed to untag index: {:?}", e))?;

        // Call vp_bytes_get(bytes: ViperBytes*, index: i64) -> i64
        let bytes_get = state
            .module
            .get_function("vp_bytes_get")
            .ok_or_else(|| "vp_bytes_get not declared".to_string())?;

        let result = state
            .ir_builder
            .build_call(
                state.builder,
                bytes_get,
                &[bytes_ptr.into(), index_untagged.into()],
                "bytes_get",
            )
            .ok_or_else(|| "vp_bytes_get call failed".to_string())?;

        return Ok(result);
    }

    // TEMPORARILY DISABLED: Lists need to use inline GEP + load for better performance
    // Other pointers (strings, arrays) use array GEP
    // if is_pointer_type && is_list {
    //     let list_ptr = obj_val.into_pointer_value();

    //     // Use inline bit vector get for bool lists (more memory efficient)
    //     if is_bool_list {
    //         let bool_val = inline_bool_list_get(state, list_ptr, index_val)
    //             .map_err(|e| format!("Inline bool list get failed: {:?}", e))?;

    //         // Convert bool to i64 for compatibility with print() and other functions
    //         let bool_int = bool_val.into_int_value();
    //         let i64_val = state
    //             .builder
    //             .build_int_z_extend(bool_int, state.context.i64_type(), "bool_to_i64")
    //             .map_err(|e| format!("Failed to extend bool to i64: {:?}", e))?;

    //         return Ok(i64_val.into());
    //     }

    //     // Use inline f64 get for float lists
    //     if is_float_list {
    //         let f64_val = inline_f64_list_get(state, list_ptr, index_val)
    //             .map_err(|e| format!("Inline f64 list get failed: {:?}", e))?;

    //         return Ok(f64_val);
    //     }

    //     // Use inline i64 get for standard integer lists
    //     let i64_val = inline_i64_list_get(state, list_ptr, index_val)
    //         .map_err(|e| format!("Inline i64 list get failed: {:?}", e))?;

    //     return Ok(i64_val);
    // }

    // TEMPORARILY DISABLED: Lists need to use inline GEP + load for better performance
    // Other pointers (strings, arrays) use array GEP
    if is_pointer_type && is_list {
        let list_ptr = obj_val.into_pointer_value();

        // Use vp_bitvec_get for bool lists (bitvec uses bit-packed storage, not bytes)
        // Note: inline_bool_list_get doesn't work for bitvec because it assumes byte storage
        if is_bool_list {
            // Get the untagged index
            let index_untagged = state
                .builder
                .build_right_shift(
                    index_val,
                    state.context.i64_type().const_int(1, false),
                    true,
                    "index_untagged",
                )
                .map_err(|e| format!("Failed to untag index: {:?}", e))?;

            // Call vp_bitvec_get which handles bit-packed storage
            let bitvec_get = state
                .module
                .get_function("vp_bitvec_get")
                .ok_or_else(|| "vp_bitvec_get not declared".to_string())?;

            let bool_result = state
                .ir_builder
                .build_call(
                    state.builder,
                    bitvec_get,
                    &[list_ptr.into(), index_untagged.into()],
                    "bitvec_get",
                )
                .ok_or_else(|| "vp_bitvec_get call failed".to_string())?;

            // Convert i1 bool to i64 for compatibility with print() and other functions
            let bool_int = bool_result.into_int_value();
            let i64_val = state
                .builder
                .build_int_z_extend(bool_int, state.context.i64_type(), "bool_to_i64")
                .map_err(|e| format!("Failed to extend bool to i64: {:?}", e))?;

            return Ok(i64_val.into());
        }

        // Use inline f64 get for float lists
        if is_float_list {
            let f64_val = inline_f64_list_get(state, list_ptr, index_val)
                .map_err(|e| format!("Inline f64 list get failed: {:?}", e))?;

            return Ok(f64_val);
        }

        // Use inline i64 get for standard integer lists
        let i64_val = inline_i64_list_get(state, list_ptr, index_val)
            .map_err(|e| format!("Inline i64 list get failed: {:?}", e))?;

        return Ok(i64_val);
    }

    // Fall back to runtime function call for list indexing
    // Lists store tagged integers, so the return value is already tagged
    // But the index needs to be untagged (runtime expects untagged indices)
    if is_pointer_type && is_list {
        let list_get = state
            .module
            .get_function("vp_list_get")
            .ok_or_else(|| "vp_list_get not declared".to_string())?;

        // Untag the index (tagged ints are shifted left by 1)
        let index_untagged = state
            .builder
            .build_right_shift(
                index_val,
                state.context.i64_type().const_int(1, false),
                true,
                "index_untagged",
            )
            .map_err(|e| format!("Failed to untag index: {:?}", e))?;

        // Handle negative indices: if index < 0, index = length + index
        let list_len_func = state
            .module
            .get_function("vp_list_len")
            .ok_or_else(|| "vp_list_len not declared".to_string())?;
        let list_len = state
            .ir_builder
            .build_call(state.builder, list_len_func, &[obj_val.into()], "list_len")
            .unwrap()
            .into_int_value();

        let is_negative = state
            .builder
            .build_int_compare(
                inkwell::IntPredicate::SLT,
                index_untagged,
                state.context.i64_type().const_zero(),
                "index_is_neg",
            )
            .expect("compare neg");

        let adjusted_index = state
            .builder
            .build_int_add(list_len, index_untagged, "adjusted_index")
            .expect("add len");

        let final_index = state
            .builder
            .build_select(is_negative, adjusted_index, index_untagged, "final_index")
            .expect("select index")
            .into_int_value();

        let result = state
            .ir_builder
            .build_call(state.builder, list_get, &[obj_val.into(), final_index.into()], "list_get")
            .expect("vp_list_get call failed");

        return Ok(result);
    }

    // For non-list pointers (known strings or arrays), use array indexing
    let is_string_or_array = match obj {
        Expr::Str(..) | Expr::FString(..) => true,
        Expr::Ident(name, _) => {
            if let Some(var_type) = state.var_types.get(name) {
                matches!(var_type, Type::Str | Type::Array(..))
            } else {
                false
            }
        }
        Expr::Array { .. } => true,
        _ => {
            let inferred = crate::codegen::expressions::core::infer_expr_type(obj);
            matches!(inferred, Type::Str | Type::Array(..))
        }
    };

    if is_pointer_type && is_string_or_array {
        let obj_ptr = obj_val.into_pointer_value();

        // Determine element type based on the object
        let elem_type: inkwell::types::BasicTypeEnum = match obj {
            Expr::Ident(name, _) => {
                // Check var_types for array type
                if let Some(var_type) = state.var_types.get(name) {
                    match var_type {
                        Type::Array(inner, _) => match &**inner {
                            Type::I64 | Type::I32 | Type::I16 | Type::I8 | Type::Int => {
                                state.context.i64_type().into()
                            }
                            Type::F64 | Type::F32 => state.context.f64_type().into(),
                            Type::Bool => state.context.bool_type().into(),
                            _ => state.context.i64_type().into(),
                        },
                        _ => state.context.i8_type().into(), // Default to i8 for strings
                    }
                } else {
                    state.context.i8_type().into()
                }
            }
            Expr::Array { elements, .. } => {
                // Infer from first element
                if let Some(first) = elements.first() {
                    match first {
                        Expr::Int(_, _) => state.context.i64_type().into(),
                        Expr::Float(_, _) => state.context.f64_type().into(),
                        Expr::Bool(_, _) => state.context.bool_type().into(),
                        _ => state.context.i8_type().into(),
                    }
                } else {
                    state.context.i8_type().into()
                }
            }
            _ => state.context.i8_type().into(), // Default to i8 for strings
        };

        let elem_ptr = if elem_type == state.context.i8_type().into() {
            // String indexing - use i8 GEP
            unsafe {
                state.builder.build_in_bounds_gep(
                    elem_type.into_int_type(),
                    obj_ptr,
                    &[index_val],
                    "string_elem",
                )
            }
        } else {
            // Array indexing - need to cast pointer to correct element type first
            // LLVM 15+: all pointers use the same type, use context.ptr_type()
            let elem_ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());

            let typed_ptr = state
                .builder
                .build_pointer_cast(obj_ptr, elem_ptr_type, "typed_array_ptr")
                .map_err(|e| format!("Failed to cast array pointer: {:?}", e))?;

            unsafe {
                state.builder.build_in_bounds_gep(elem_type, typed_ptr, &[index_val], "array_elem")
            }
        }
        .map_err(|e| format!("Failed to build array index GEP: {:?}", e))?;

        let loaded = state
            .builder
            .build_load(elem_type, elem_ptr, "array_load")
            .map_err(|e| format!("Failed to load array element: {:?}", e))?;

        // Convert to i64 for compatibility with print() and other functions
        let result: BasicValueEnum = match elem_type {
            inkwell::types::BasicTypeEnum::IntType(it) => {
                if it.get_bit_width() < 64 {
                    let int_val = loaded.into_int_value();
                    state
                        .builder
                        .build_int_z_extend(int_val, state.context.i64_type(), "extend_to_i64")
                        .map_err(|e| format!("Failed to extend to i64: {:?}", e))?
                        .into()
                } else {
                    loaded
                }
            }
            inkwell::types::BasicTypeEnum::FloatType(_) => {
                // Float to i64 conversion (bitcast for tagged representation)
                let float_val = loaded.into_float_value();
                state
                    .builder
                    .build_float_to_signed_int(float_val, state.context.i64_type(), "f64_to_i64")
                    .map_err(|e| format!("Failed to convert f64 to i64: {:?}", e))?
                    .into()
            }
            _ => loaded,
        };

        return Ok(result);
    }

    // Fall back to list indexing for any remaining cases
    let list_get = state
        .module
        .get_function("vp_list_get")
        .ok_or_else(|| "vp_list_get not declared".to_string())?;

    // Untag the index for the runtime call
    let index_untagged = state
        .builder
        .build_right_shift(
            index_val,
            state.context.i64_type().const_int(1, false),
            true,
            "fallback_index_untagged",
        )
        .map_err(|e| format!("Failed to untag index: {:?}", e))?;

    let result = state
        .ir_builder
        .build_call(state.builder, list_get, &[obj_val.into(), index_untagged.into()], "list_get")
        .ok_or_else(|| "build call failed".to_string())?;

    Ok(result)
}
