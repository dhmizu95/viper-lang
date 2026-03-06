//! Slice operations for Viper

use inkwell::values::BasicValueEnum;

use crate::ast::{Expr, Type};
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarStorage, VarType};

use crate::codegen::expressions::generate_expr;

/// Generate slice access (list[start:end] or list[start:end:step])
pub fn generate_slice<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    start: &Option<Box<Expr>>,
    end: &Option<Box<Expr>>,
    step: &Option<Box<Expr>>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let obj_val = generate_expr(state, obj)?;

    // Check if this is a string
    let is_string = match obj {
        Expr::Ident(obj_name, _) => {
            if let Some(var_type) = state.var_types.get(obj_name) {
                matches!(var_type, Type::Str)
            } else {
                false
            }
        },
        Expr::Str(_, _) => true,
        _ => false,
    };

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

    // Generate end value (default to length)
    let end_val = if let Some(end_expr) = end {
        generate_expr(state, end_expr)?
    } else {
        // Need to get length based on type
        if is_string {
            let str_len = state
                .module
                .get_function("vp_str_len")
                .ok_or_else(|| "vp_str_len not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(state.builder, str_len, &[obj_val.into()], "str_len")
                .ok_or_else(|| "build call failed".to_string())?;
            result
        } else {
            let list_len = state
                .module
                .get_function("vp_list_len")
                .ok_or_else(|| "vp_list_len not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(state.builder, list_len, &[obj_val.into()], "list_len")
                .ok_or_else(|| "build call failed".to_string())?;
            result
        }
    };

    // For strings, use vp_str_slice (doesn't support step)
    if is_string {
        let str_slice = state
            .module
            .get_function("vp_str_slice")
            .ok_or_else(|| "vp_str_slice not declared".to_string())?;

        let result = state
            .ir_builder
            .build_call(
                state.builder,
                str_slice,
                &[obj_val.into(), start_val.into(), end_val.into()],
                "str_slice",
            )
            .ok_or_else(|| "build call failed".to_string())?;

        return Ok(result);
    }

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
                VarStorage::ClosureCell(_cell_ptr) => {
                    // Closure cell: store through the cell's value pointer
                    if let Some(value_ptr) = &var_info.closure_value_ptr {
                        state.builder.build_store(*value_ptr, value_val)
                            .map_err(|e| format!("Failed to store to closure cell: {:?}", e))?;
                    } else {
                        return Err("Closure cell missing value pointer".to_string());
                    }
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

/* ============================================ */
/* Collection Built-in Functions                */
/* ============================================ */

/// Generate list() call - convert iterable to list
pub fn generate_list_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // list() with no args returns empty list
    if args.is_empty() {
        let list_func = state
            .module
            .get_function("vp_list_create")
            .ok_or_else(|| "vp_list_create not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, list_func, &[], "empty_list");
        return Ok(result.unwrap());
    }

    if args.len() > 1 {
        return Err(format!("list() takes at most 1 argument, got {}", args.len()));
    }

    let arg = &args[0];
    let arg_val = generate_expr(state, arg)?;

    // Handle different source types
    let result = match arg {
        // list(string) - convert string to list of character codes
        Expr::Str(_, _) => {
            let from_str_func = state
                .module
                .get_function("vp_list_from_str")
                .ok_or_else(|| "vp_list_from_str not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, from_str_func, &[arg_val.into()], "list_from_str")
                .unwrap()
        }
        // list(list) - copy existing list
        Expr::List { .. } => {
            let copy_func = state
                .module
                .get_function("vp_list_copy_from_list")
                .ok_or_else(|| "vp_list_copy_from_list not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, copy_func, &[arg_val.into()], "list_copy")
                .unwrap()
        }
        // list(range(...)) - already returns a list, just copy it
        Expr::Call { func, .. } => {
            if let Expr::Ident(name, _) = func.as_ref() {
                if name == "range" {
                    let copy_func = state
                        .module
                        .get_function("vp_list_copy_from_list")
                        .ok_or_else(|| "vp_list_copy_from_list not declared".to_string())?;
                    state
                        .ir_builder
                        .build_call(state.builder, copy_func, &[arg_val.into()], "list_from_range")
                        .unwrap()
                } else {
                    // Generic iterable
                    let from_iter_func = state
                        .module
                        .get_function("vp_list_from_iterable")
                        .ok_or_else(|| "vp_list_from_iterable not declared".to_string())?;
                    state
                        .ir_builder
                        .build_call(state.builder, from_iter_func, &[arg_val.into()], "list_from_iter")
                        .unwrap()
                }
            } else {
                // Generic iterable
                let from_iter_func = state
                    .module
                    .get_function("vp_list_from_iterable")
                    .ok_or_else(|| "vp_list_from_iterable not declared".to_string())?;
                state
                    .ir_builder
                    .build_call(state.builder, from_iter_func, &[arg_val.into()], "list_from_iter")
                    .unwrap()
            }
        }
        // Check if identifier holds a list
        Expr::Ident(name, _) => {
            if state.is_list(name) {
                let copy_func = state
                    .module
                    .get_function("vp_list_copy_from_list")
                    .ok_or_else(|| "vp_list_copy_from_list not declared".to_string())?;
                state
                    .ir_builder
                    .build_call(state.builder, copy_func, &[arg_val.into()], "list_copy")
                    .unwrap()
            } else {
                // Generic iterable (including strings)
                let from_iter_func = state
                    .module
                    .get_function("vp_list_from_iterable")
                    .ok_or_else(|| "vp_list_from_iterable not declared".to_string())?;
                state
                    .ir_builder
                    .build_call(state.builder, from_iter_func, &[arg_val.into()], "list_from_iter")
                    .unwrap()
            }
        }
        // Default: treat as generic iterable
        _ => {
            let from_iter_func = state
                .module
                .get_function("vp_list_from_iterable")
                .ok_or_else(|| "vp_list_from_iterable not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, from_iter_func, &[arg_val.into()], "list_from_iter")
                .unwrap()
        }
    };

    Ok(result)
}

/// Generate tuple() call - convert iterable to tuple
pub fn generate_tuple_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // tuple() with no args returns empty tuple
    if args.is_empty() {
        // Create empty list and convert to tuple (simplified)
        let list_func = state
            .module
            .get_function("vp_list_create")
            .ok_or_else(|| "vp_list_create not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, list_func, &[], "empty_tuple");
        return Ok(result.unwrap());
    }

    if args.len() > 1 {
        return Err(format!("tuple() takes at most 1 argument, got {}", args.len()));
    }

    let arg = &args[0];
    let arg_val = generate_expr(state, arg)?;

    // Handle different source types
    let result = match arg {
        // tuple(string) - convert string to tuple of character codes
        Expr::Str(_, _) => {
            let from_str_func = state
                .module
                .get_function("vp_tuple_from_str")
                .ok_or_else(|| "vp_tuple_from_str not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, from_str_func, &[arg_val.into()], "tuple_from_str")
                .unwrap()
        }
        // tuple(list) - convert list to tuple
        Expr::List { .. } | Expr::Call { .. } => {
            let from_list_func = state
                .module
                .get_function("vp_tuple_from_list")
                .ok_or_else(|| "vp_tuple_from_list not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, from_list_func, &[arg_val.into()], "tuple_from_list")
                .unwrap()
        }
        // Check if identifier holds a list
        Expr::Ident(name, _) => {
            if state.is_list(name) {
                let from_list_func = state
                    .module
                    .get_function("vp_tuple_from_list")
                    .ok_or_else(|| "vp_tuple_from_list not declared".to_string())?;
                state
                    .ir_builder
                    .build_call(state.builder, from_list_func, &[arg_val.into()], "tuple_from_list")
                    .unwrap()
            } else {
                // Generic iterable (including strings)
                let from_iter_func = state
                    .module
                    .get_function("vp_tuple_from_iterable")
                    .ok_or_else(|| "vp_tuple_from_iterable not declared".to_string())?;
                state
                    .ir_builder
                    .build_call(state.builder, from_iter_func, &[arg_val.into()], "tuple_from_iter")
                    .unwrap()
            }
        }
        // Default: treat as generic iterable
        _ => {
            let from_iter_func = state
                .module
                .get_function("vp_tuple_from_iterable")
                .ok_or_else(|| "vp_tuple_from_iterable not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, from_iter_func, &[arg_val.into()], "tuple_from_iter")
                .unwrap()
        }
    };

    Ok(result)
}

/// Generate set() call - create set from iterable
pub fn generate_set_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // set() with no args returns empty set
    if args.is_empty() {
        // Create empty set (using list as placeholder for now)
        let list_func = state
            .module
            .get_function("vp_list_create")
            .ok_or_else(|| "vp_list_create not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, list_func, &[], "empty_set");
        return Ok(result.unwrap());
    }

    if args.len() > 1 {
        return Err(format!("set() takes at most 1 argument, got {}", args.len()));
    }

    let arg = &args[0];
    let arg_val = generate_expr(state, arg)?;

    // Handle different source types
    let result = match arg {
        // set(list) - convert list to set
        Expr::List { .. } | Expr::Call { .. } => {
            let from_list_func = state
                .module
                .get_function("vp_set_from_list")
                .ok_or_else(|| "vp_set_from_list not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, from_list_func, &[arg_val.into()], "set_from_list")
                .unwrap()
        }
        // Check if identifier holds a list
        Expr::Ident(name, _) => {
            if state.is_list(name) {
                let from_list_func = state
                    .module
                    .get_function("vp_set_from_list")
                    .ok_or_else(|| "vp_set_from_list not declared".to_string())?;
                state
                    .ir_builder
                    .build_call(state.builder, from_list_func, &[arg_val.into()], "set_from_list")
                    .unwrap()
            } else {
                // Generic iterable
                let from_iter_func = state
                    .module
                    .get_function("vp_set_from_iterable")
                    .ok_or_else(|| "vp_set_from_iterable not declared".to_string())?;
                state
                    .ir_builder
                    .build_call(state.builder, from_iter_func, &[arg_val.into()], "set_from_iter")
                    .unwrap()
            }
        }
        // Default: treat as generic iterable
        _ => {
            let from_iter_func = state
                .module
                .get_function("vp_set_from_iterable")
                .ok_or_else(|| "vp_set_from_iterable not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, from_iter_func, &[arg_val.into()], "set_from_iter")
                .unwrap()
        }
    };

    Ok(result)
}
