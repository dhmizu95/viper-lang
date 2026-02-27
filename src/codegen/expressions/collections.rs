//! Expression code generation for Viper

use super::*;

use crate::ast::{Expr, Type};

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarStorage, VarType};

/// Generate list creation
pub fn generate_list<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Determine element type
    let is_float_list = elements.first().map(|e| matches!(e, Expr::Float(..))).unwrap_or(false);
    let is_bool_list = elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false);

    // For empty lists or mixed types, check all elements
    let (list_func_name, append_func_name) = if is_float_list {
        ("vp_list_create_f64", "vp_list_append_f64")
    } else if is_bool_list {
        ("vp_bitvec_create", "vp_bitvec_append")  // Use bit vector for bool lists
    } else {
        ("vp_list_create", "vp_list_append")
    };

    let list_func = state
        .module
        .get_function(list_func_name)
        .ok_or_else(|| format!("{} not declared", list_func_name))?;

    let list_val = state.ir_builder.build_call(state.builder, list_func, &[], "new_list").unwrap();

    let append_func = state
        .module
        .get_function(append_func_name)
        .ok_or_else(|| format!("{} not declared", append_func_name))?;

    for (idx, elem) in elements.iter().enumerate() {
        let mut elem_val = generate_expr(state, elem)?;

        // If float list but elem is int, convert to float
        if is_float_list && elem_val.is_int_value() {
            let int_val = elem_val.into_int_value();
            let float_val = state
                .builder
                .build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
                .expect("int to float conversion");
            elem_val = float_val.into();
        } else if is_bool_list && elem_val.is_int_value() && elem_val.get_type().into_int_type().get_bit_width() > 1 {
            // Convert i64 to bool for bool list (only if not already i1)
            let int_val = elem_val.into_int_value();
            let bool_val = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    int_val,
                    state.context.i64_type().const_zero(),
                    "i64_to_bool",
                )
                .expect("i64 to bool conversion");
            elem_val = bool_val.into();
        }

        let _ = state.ir_builder.build_call(
            state.builder,
            append_func,
            &[list_val.into(), elem_val.into()],
            &format!("list_append_{}", idx),
        );
    }

    Ok(list_val)
}

/// Generate list comprehension: [expr for var in iter]
/// Currently supports: [expr for var in range(n)] and [expr for var in range(start, end)]
pub fn generate_list_comprehension<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    element: &Expr,
    var: &str,
    iter: &Expr,
    _span: crate::utils::Span,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Determine element type by analyzing the element expression
    let elem_type = crate::codegen::expressions::infer_expr_type(element);
    let is_float_list = matches!(elem_type, Type::F64);
    let is_bool_list = matches!(elem_type, Type::Bool);

    let (list_func_name, append_func_name) = if is_float_list {
        ("vp_list_create_f64", "vp_list_append_f64")
    } else if is_bool_list {
        ("vp_bitvec_create", "vp_bitvec_append")  // Use bit vector for bool lists
    } else {
        ("vp_list_create", "vp_list_append")
    };

    // Create result list
    let list_func = state
        .module
        .get_function(list_func_name)
        .ok_or_else(|| format!("{} not declared", list_func_name))?;

    let result_list =
        state.ir_builder.build_call(state.builder, list_func, &[], "comp_result").unwrap();

    // Append function
    let append_func = state
        .module
        .get_function(append_func_name)
        .ok_or_else(|| format!("{} not declared", append_func_name))?;

    // Handle range() specially (like generate_for does)
    let (start_val, end_val) = if let Expr::Call { func, args, .. } = iter {
        if let Expr::Ident(name, _) = func.as_ref() {
            if name == "range" {
                match args.len() {
                    0 => return Err("range expected at least 1 argument, got 0".to_string()),
                    1 => (
                        state.ir_builder.i64_const(0),
                        generate_expr(state, &args[0])?.into_int_value(),
                    ),
                    _ => (
                        generate_expr(state, &args[0])?.into_int_value(),
                        generate_expr(state, &args[1])?.into_int_value(),
                    ),
                }
            } else {
                return Err("List comprehension only supports range() iterator".to_string());
            }
        } else {
            return Err("List comprehension only supports range() iterator".to_string());
        }
    } else {
        return Err("List comprehension only supports range() iterator".to_string());
    };

    // Create loop blocks
    let func = state
        .builder
        .get_insert_block()
        .ok_or("No insertion block")?
        .get_parent()
        .ok_or("No parent function")?;

    let init_block = state.context.append_basic_block(func, "list_comp_init");
    let cond_block = state.context.append_basic_block(func, "list_comp_cond");
    let body_block = state.context.append_basic_block(func, "list_comp_body");
    let step_block = state.context.append_basic_block(func, "list_comp_step");
    let after_loop_block = state.context.append_basic_block(func, "list_comp_after");

    // Branch to init block
    state.builder.build_unconditional_branch(init_block).expect("branch to init");

    // Init block: create counter variable
    state.builder.position_at_end(init_block);
    let counter =
        state.builder.build_alloca(state.context.i64_type(), "comp_counter").expect("alloca");
    state.builder.build_store(counter, start_val).expect("store counter");

    // Branch to condition
    state.builder.build_unconditional_branch(cond_block).expect("branch to cond");

    // Condition block
    state.builder.position_at_end(cond_block);

    // Load counter
    let counter_val = state
        .builder
        .build_load(state.context.i64_type(), counter, "counter_val")
        .expect("load counter")
        .into_int_value();

    // Check if counter < end
    let cond = state.ir_builder.build_icmp_lt(state.builder, counter_val, end_val, "comp_cond");

    // Branch based on condition: if true -> body, if false -> after
    state.ir_builder.build_cond_branch(state.builder, cond, body_block, after_loop_block);

    // Body block
    state.builder.position_at_end(body_block);

    // Create a separate variable for the loop variable (copy counter value)
    let var_ptr = state.builder.build_alloca(state.context.i64_type(), var).expect("alloca");
    let counter_val = state
        .builder
        .build_load(state.context.i64_type(), counter, "counter_for_var")
        .expect("load counter")
        .into_int_value();
    state.builder.build_store(var_ptr, counter_val).expect("store var");

    // Set up the loop variable in the symbol table
    let old_var = state.variables.insert(
        var.to_string(),
        crate::codegen::variables::VarInfo::new_stack(
            var_ptr,
            crate::codegen::variables::VarType::Int,
        ),
    );

    // Generate the element expression
    let elem_val = generate_expr(state, element)?;

    // Handle type conversions
    let elem_val = if is_float_list && elem_val.is_int_value() {
        let int_val = elem_val.into_int_value();
        state
            .builder
            .build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion")
            .into()
    } else if is_bool_list && elem_val.is_int_value() && elem_val.get_type().into_int_type().get_bit_width() > 1 {
        // Convert i64 to bool for bool list (only if not already i1)
        let int_val = elem_val.into_int_value();
        state
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                int_val,
                state.context.i64_type().const_zero(),
                "i64_to_bool",
            )
            .expect("i64 to bool conversion")
            .into()
    } else {
        elem_val
    };

    // Append to result list
    let _ = state.ir_builder.build_call(
        state.builder,
        append_func,
        &[result_list.into(), elem_val.into()],
        "list_append",
    );

    // Restore the variable after body
    if let Some(old) = old_var {
        state.variables.insert(var.to_string(), old);
    } else {
        state.variables.remove(var);
    }

    // Branch to step block
    state.builder.build_unconditional_branch(step_block).expect("branch to step");

    // Step block: increment counter
    state.builder.position_at_end(step_block);
    let counter_val = state
        .builder
        .build_load(state.context.i64_type(), counter, "counter_step")
        .expect("load counter")
        .into_int_value();
    let next_val = state.ir_builder.build_add(
        state.builder,
        counter_val,
        state.context.i64_type().const_int(1, false),
        "next_counter",
    );
    state.builder.build_store(counter, next_val).expect("store counter");

    // Branch back to condition
    state.builder.build_unconditional_branch(cond_block).expect("branch back to cond");

    // After loop
    state.builder.position_at_end(after_loop_block);

    Ok(result_list)
}

/// Generate dict creation
pub fn generate_dict<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    pairs: &[(Expr, Expr)],
) -> Result<BasicValueEnum<'ctx>, String> {
    let dict_create_func = state
        .module
        .get_function("vp_dict_create")
        .ok_or_else(|| "vp_dict_create not declared".to_string())?;

    let dict_val =
        state.ir_builder.build_call(state.builder, dict_create_func, &[], "new_dict").unwrap();

    for (i, (key_expr, value_expr)) in pairs.iter().enumerate() {
        let key_val = generate_expr(state, key_expr)?;
        let value_val = generate_expr(state, value_expr)?;

        // Choose the appropriate dict_set function based on key and value types
        match (key_expr, value_expr) {
            (Expr::Str(_, _), Expr::Int(_, _)) => {
                // String key (already a Viper string) with i64 value
                let set_func = state
                    .module
                    .get_function("vp_dict_set_str_i64")
                    .ok_or_else(|| "vp_dict_set_str_i64 not declared".to_string())?;

                let _ = state.ir_builder.build_call(
                    state.builder,
                    set_func,
                    &[dict_val.into(), key_val.into(), value_val.into()],
                    &format!("dict_set_{}", i),
                );
            }
            (Expr::Str(_, _), Expr::Str(_, _)) => {
                // Both key and value are strings (already Viper strings)
                let set_func = state
                    .module
                    .get_function("vp_dict_set_str_str")
                    .ok_or_else(|| "vp_dict_set_str_str not declared".to_string())?;

                let _ = state.ir_builder.build_call(
                    state.builder,
                    set_func,
                    &[dict_val.into(), key_val.into(), value_val.into()],
                    &format!("dict_set_{}", i),
                );
            }
            (Expr::Str(_, _), _) => {
                // String key with other value types
                let set_func = state
                    .module
                    .get_function("vp_dict_set_str_i64")
                    .ok_or_else(|| "vp_dict_set_str_i64 not declared".to_string())?;

                let _ = state.ir_builder.build_call(
                    state.builder,
                    set_func,
                    &[dict_val.into(), key_val.into(), value_val.into()],
                    &format!("dict_set_{}", i),
                );
            }
            _ => {
                // Fallback for non-string keys (not yet supported)
                return Err("Dict keys must be strings".to_string());
            }
        }
    }

    Ok(dict_val)
}

/// Generate array creation (fixed-size, stack-allocated)
pub fn generate_array<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
    size: Option<usize>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let array_size = size.unwrap_or_else(|| elements.len());

    if array_size == 0 {
        // Empty array - return null pointer
        let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
        return Ok(ptr_type.const_null().into());
    }

    // Get element type from first element or default to i64
    let elem_type: inkwell::types::BasicTypeEnum = if let Some(first_elem) = elements.first() {
        match first_elem {
            Expr::Int(_, _) => state.context.i64_type().into(),
            Expr::Float(_, _) => state.context.f64_type().into(),
            Expr::Bool(_, _) => state.context.bool_type().into(),
            _ => state.context.i64_type().into(),
        }
    } else {
        state.context.i64_type().into()
    };

    // Allocate array on stack as a single alloca of element type with size
    let array_alloca = state
        .builder
        .build_array_alloca(
            elem_type,
            state.context.i32_type().const_int(array_size as u64, false),
            "array",
        )
        .map_err(|e| format!("Failed to allocate array: {:?}", e))?;

    // Check if this is array repeat syntax: [value; size]
    let is_repeat = elements.len() == 1 && size.is_some() && size.unwrap() > 1;

    // Initialize elements
    for i in 0..array_size {
        let elem_val = if is_repeat {
            // For repeat syntax, use the first element value for all positions
            generate_expr(state, &elements[0])?
        } else if i < elements.len() {
            // For regular arrays, use the corresponding element
            generate_expr(state, &elements[i])?
        } else {
            // Fill remaining elements with zero
            let zero_val: BasicValueEnum = if elem_type.is_int_type() {
                elem_type.into_int_type().const_zero().into()
            } else if elem_type.is_float_type() {
                elem_type.into_float_type().const_zero().into()
            } else {
                elem_type.into_int_type().const_zero().into()
            };
            zero_val
        };

        // Create GEP to element position
        let elem_ptr = unsafe {
            state.builder.build_in_bounds_gep(
                elem_type,
                array_alloca,
                &[state.context.i32_type().const_int(i as u64, false)],
                &format!("elem_{}", i),
            )
        }
        .map_err(|e| format!("Failed to build GEP: {:?}", e))?;

        state
            .builder
            .build_store(elem_ptr, elem_val)
            .map_err(|e| format!("Failed to store element: {:?}", e))?;
    }

    Ok(array_alloca.into())
}

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

/// Generate slice access (list[start:end] or list[start:end:step])
pub fn generate_slice<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    start: &Option<Box<Expr>>,
    end: &Option<Box<Expr>>,
    step: &Option<Box<Expr>>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let obj_val = generate_expr(state, obj)?;

    // Check if this is a bool list (bit vector)
    let is_bool_list = match obj {
        Expr::Ident(obj_name, _) => state.is_bool_list(obj_name),
        Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
        _ => false,
    };

    // Generate start value (default to 0)
    let start_val = if let Some(start_expr) = start {
        generate_expr(state, start_expr)?
    } else {
        state.ir_builder.i64_const(0).into()
    };

    // Generate end value (default to list length)
    let end_val = if let Some(end_expr) = end {
        generate_expr(state, end_expr)?
    } else {
        // Need to get list length
        let list_len = state
            .module
            .get_function("vp_list_len")
            .ok_or_else(|| "vp_list_len not declared".to_string())?;

        let result = state
            .ir_builder
            .build_call(state.builder, list_len, &[obj_val.into()], "list_len")
            .ok_or_else(|| "build call failed".to_string())?;
        result
    };

    // Generate step value (default to 1)
    let step_val = if let Some(step_expr) = step {
        generate_expr(state, step_expr)?
    } else {
        state.ir_builder.i64_const(1).into()
    };

    // Call appropriate slice function based on element type
    let slice_func = if is_bool_list {
        state
            .module
            .get_function("vp_bitvec_slice")
            .ok_or_else(|| "vp_bitvec_slice not declared".to_string())?
    } else {
        state
            .module
            .get_function("vp_list_slice")
            .ok_or_else(|| "vp_list_slice not declared".to_string())?
    };

    let result = state
        .ir_builder
        .build_call(
            state.builder,
            slice_func,
            &[obj_val.into(), start_val.into(), end_val.into(), step_val.into()],
            "list_slice",
        )
        .ok_or_else(|| "build call failed".to_string())?;

    Ok(result)
}

/// Generate assignment expression (walrus operator: :=)
/// Assigns value to target and returns the value
pub fn generate_assignment_expr<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    value: &Expr,
    _span: crate::utils::Span,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Generate code for the value expression
    let value_val = generate_expr(state, value)?;
    
    // The target must be an identifier
    if let Expr::Ident(name, _) = target {
        // Determine variable type from value
        let var_type = match value_val {
            BasicValueEnum::IntValue(_) => VarType::Int,
            BasicValueEnum::FloatValue(_) => VarType::Float,
            BasicValueEnum::PointerValue(_) => VarType::Pointer,
            BasicValueEnum::ArrayValue(_) => VarType::Pointer,
            BasicValueEnum::StructValue(_) => VarType::Pointer,
            _ => VarType::Int, // Default for other types
        };
        
        // Check if variable already exists
        if let Some(var_info) = state.variables.get(name) {
            // Variable exists - store the value
            match &var_info.storage {
                VarStorage::Register(_) => {
                    // Register-allocated: update the variable info with new value
                    // For now, we need to re-insert with the new value
                    let new_info = VarInfo::new_register(value_val, var_type);
                    state.variables.insert(name.clone(), new_info);
                }
                VarStorage::Stack(alloca) => {
                    // Stack-allocated: store to alloca
                    state.builder.build_store(*alloca, value_val)
                        .map_err(|e| format!("Failed to store value: {:?}", e))?;
                }
            }
        } else {
            // Variable doesn't exist - create it (implicit declaration)
            // Use stack allocation for simplicity
            let alloca = state.builder.build_alloca(value_val.get_type(), name)
                .map_err(|e| format!("Failed to create alloca: {:?}", e))?;
            state.builder.build_store(alloca, value_val)
                .map_err(|e| format!("Failed to store value: {:?}", e))?;
            state.variables.insert(
                name.clone(),
                VarInfo::new_stack(alloca, var_type),
            );
        }
        
        // Return the value (walrus operator returns the assigned value)
        Ok(value_val)
    } else {
        Err("Assignment expression target must be an identifier".to_string())
    }
}
