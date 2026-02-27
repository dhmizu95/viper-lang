use crate::ast::{BinOp, Expr};
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarStorage, VarType};

/// Generate assignment statement
pub(crate) fn generate_assign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    value: &Expr,
) -> Result<(), String> {
    // Handle tuple unpacking: a, b = (1, 2)
    if let Expr::Tuple { elements: targets, .. } = target {
        return generate_tuple_unpack(state, targets, value);
    }

    if let Expr::Ident(name, _) = target {
        // TEMPORARILY DISABLED: Testing basic bool list operations without stack alloc
        let handled_custom = false;
        let custom_val: Option<inkwell::values::BasicValueEnum> = None;

        // Stack allocation code disabled for JIT testing
        // The vp_list_bool_repeat path should work without stack allocation

        let val = if handled_custom {
            custom_val.unwrap()
        } else {
            crate::codegen::expressions::generate_expr(state, value)?
        };

        // Check if this is a global variable assignment
        // If the variable exists in global_constants but not in local variables,
        // assign to the global
        if state.global_constants.contains_key(name) && !state.variables.contains_key(name) {
            // This is a global variable assignment
            let global = state.global_constants.get(name).unwrap();
            let global_ptr = global.as_pointer_value();
            state.builder.build_store(global_ptr, val).expect("store to global");
            return Ok(());
        }

        // Check if the value is a stack-allocated array (should not use ARC)
        let is_stack_array = matches!(value, Expr::Array { .. });

        // Track list variables
        let is_list = match value {
            Expr::List { .. } => true,
            Expr::ListComprehension { .. } => true,
            Expr::Ident(other, _) => state.is_list(other),
            // Check for list repetition: [elem] * n
            Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
                matches!(left.as_ref(), Expr::List { .. } | Expr::Array { .. })
            }
            Expr::Call { func, .. } => {
                // Check if calling a list-returning function
                if let Expr::Ident(func_name, _) = func.as_ref() {
                    // Built-in list functions
                    if func_name == "vp_list_create"
                        || func_name == "vp_list_create_f64"
                        || func_name == "vp_list_create_with_capacity"
                    {
                        true
                    // Built-in string functions - not lists
                    } else if func_name.starts_with("vp_str_") || func_name == "str"
                    // str() conversion returns string, not list
                    {
                        false
                    // User-defined functions - check if return value is a pointer
                    } else {
                        val.is_pointer_value()
                    }
                } else {
                    val.is_pointer_value()
                }
            }
            _ => false,
        };
        if is_list {
            state.mark_as_list(name.clone());
        } else {
            state.list_vars.remove(name);
        }

        // Track bool list variables
        let is_bool_list = match value {
            Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
            Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
                // Handle [bool] * n pattern
                if let Expr::List { elements, .. } = left.as_ref() {
                    elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
                } else {
                    false
                }
            }
            Expr::Call { func, .. } => {
                // Check if calling a bool list function (now using bit vectors)
                if let Expr::Ident(func_name, _) = func.as_ref() {
                    func_name == "vp_bitvec_create"
                        || func_name == "vp_bitvec_create_with_capacity"
                        || func_name == "vp_bitvec_repeat"
                } else {
                    false
                }
            }
            Expr::Index { obj, .. } => {
                // Check if indexing a bool list (single element access)
                match obj.as_ref() {
                    Expr::Ident(obj_name, _) => state.is_bool_list(obj_name),
                    Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
                    _ => false,
                }
            }
            Expr::Slice { obj, .. } => {
                // Check if slicing a bool list
                match obj.as_ref() {
                    Expr::Ident(obj_name, _) => state.is_bool_list(obj_name),
                    Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
                    _ => false,
                }
            }
            _ => false,
        };
        if is_bool_list {
            state.mark_as_bool_list(name.clone());
        } else {
            state.bool_list_vars.remove(name);
        }

        // Track dict variables
        let is_dict = match value {
            Expr::Dict { .. } => true,
            Expr::Ident(other, _) => state.is_dict(other),
            Expr::Call { func, .. } => {
                if let Expr::Ident(func_name, _) = func.as_ref() {
                    func_name == "vp_dict_create" || func_name == "vp_dict_create_with_capacity"
                } else {
                    false
                }
            }
            _ => false,
        };
        if is_dict {
            state.mark_as_dict(name.clone());
        } else {
            state.dict_vars.remove(name);
        }

        // Check if value is BigInt before borrowing state
        // BigInt values are pointers from BigInt literals or operations
        // We detect BigInt by checking the expression type and value type
        let is_bigint = match value {
            Expr::BigInt(_, _) => true,
            Expr::BinOp { op, .. } => {
                // Arithmetic operations on BigInts produce BigInt results
                matches!(
                    op,
                    crate::ast::BinOp::Add
                        | crate::ast::BinOp::Sub
                        | crate::ast::BinOp::Mul
                        | crate::ast::BinOp::Div
                        | crate::ast::BinOp::Mod
                        | crate::ast::BinOp::Pow
                ) && val.is_pointer_value()
            }
            Expr::UnaryOp { op, operand, .. } => {
                // Negation of BigInt produces BigInt
                matches!(op, crate::ast::UnaryOp::Neg)
                    && matches!(operand.as_ref(), Expr::BigInt(_, _))
                    && val.is_pointer_value()
            }
            _ => false,
        };

        // Check if variable exists and get its info
        let var_exists = state.variables.contains_key(name);

        if var_exists {
            // Get info we need before mutable borrow
            let old_is_ref;
            let old_needs_arc;
            let storage;

            {
                let var_info = state.variables.get(name).unwrap();
                old_is_ref =
                    var_info.var_type == VarType::Pointer || var_info.var_type == VarType::BigInt;
                old_needs_arc = state.needs_arc(name);
                storage = var_info.storage.clone();
            }

            // Update var_type if this is a BigInt assignment
            if is_bigint {
                if let Some(var_info) = state.variables.get_mut(name) {
                    var_info.var_type = VarType::BigInt;
                }
            }

            // Update existing variable
            match &storage {
                VarStorage::Stack(alloca) => {
                    // Release old value if it was a reference type needing ARC
                    if old_is_ref && old_needs_arc {
                        let old_val = state
                            .builder
                            .build_load(
                                state.context.ptr_type(inkwell::AddressSpace::default()),
                                *alloca,
                                &format!("{}_old", name),
                            )
                            .expect("load old value");
                        state.build_release(old_val, &format!("{}_old", name));
                    }
                    state.builder.build_store(*alloca, val).expect("store");
                }
                VarStorage::Register(_) => {
                    // For scalar types, just keep register allocation -
                    // we replace the register value
                }
            }

            // Retain new value if it's a reference type that escapes (but not stack arrays)
            let is_ref_type = val.is_pointer_value();
            let needs_arc = state.needs_arc(name);
            if is_ref_type && needs_arc && !is_stack_array {
                state.build_retain(val, name);
            }
        } else {
            let ty = val.get_type();

            // Determine if this is a reference type (but not stack arrays)
            let is_ref_type = val.is_pointer_value() && !is_stack_array;

            // Set reference type flag in escape analyzer
            state.set_reference_type(name, is_ref_type);

            // Use the is_bigint check defined earlier
            let var_type = if is_bigint {
                VarType::BigInt
            } else if val.is_float_value() {
                VarType::Float
            } else if val.is_pointer_value() {
                VarType::Pointer
            } else if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
                VarType::Bool
            } else {
                VarType::Int
            };

            // Always use stack allocation (alloca) for new variables.
            // This is critical for correctness across loop basic blocks:
            // SSA register values are frozen in the block they are defined in,
            // so mutations like `i = i + 1` would not be visible to `while_cond`
            // if the variable is register-allocated. Stack alloca + store/load
            // is the correct approach; LLVM `mem2reg` will promote them back to
            // registers during optimization.
            let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            let entry_block = func.get_first_basic_block().unwrap();
            let old_pos = state.builder.get_insert_block();
            match entry_block.get_first_instruction() {
                Some(first_instr) => state.builder.position_before(&first_instr),
                None => state.builder.position_at_end(entry_block),
            }
            let alloca = state.builder.build_alloca(ty, name).expect("alloca");
            if let Some(pos) = old_pos {
                state.builder.position_at_end(pos);
            }
            state.builder.build_store(alloca, val).expect("store");
            state.variables.insert(name.clone(), VarInfo::new_stack(alloca, var_type));

            // Insert ARC retain if this is a reference type that escapes (but not stack arrays)
            if is_ref_type && state.needs_arc(name) {
                state.build_retain(val, name);
            }
        }
    } else if let Expr::Index { obj, index, .. } = target {
        let obj_val = crate::codegen::expressions::generate_expr(state, obj)?;
        let index_val = crate::codegen::expressions::generate_expr(state, index)?.into_int_value();
        let value_val = crate::codegen::expressions::generate_expr(state, value)?;

        let is_list = if let Expr::Ident(obj_name, _) = obj.as_ref() {
            state.is_list(obj_name)
        } else {
            matches!(obj.as_ref(), Expr::List { .. })
        };

        // Check if this is an array (pointer) or list
        if obj_val.is_pointer_value() && !is_list {
            // Array index assignment using GEP and store
            let obj_ptr = obj_val.into_pointer_value();
            let elem_type = value_val.get_type();

            let elem_ptr = unsafe {
                state.builder.build_in_bounds_gep(elem_type, obj_ptr, &[index_val], "array_elem")
            }
            .map_err(|e| format!("Failed to build array index GEP: {:?}", e))?;

            state
                .builder
                .build_store(elem_ptr, value_val)
                .map_err(|e| format!("Failed to store array element: {:?}", e))?;
        } else {
            // List index assignment using runtime function
            // Determine if this is a bool list (bit vector) by checking the object type
            let obj_name = match obj.as_ref() {
                Expr::Ident(name, _) => Some(name.as_str()),
                _ => None,
            };
            let is_bool_list = obj_name.map(|n| state.is_bool_list(n)).unwrap_or(false);

            let (list_set_func, value_for_list) = if is_bool_list {
                // Use bit vector set function for bool lists
                // The checked version has branch prediction hints for common case
                let list_set = state
                    .module
                    .get_function("vp_bitvec_set")
                    .ok_or_else(|| "vp_bitvec_set not declared".to_string())?;

                // Bool value (i1), keep as bool
                (list_set, value_val)
            } else {
                // Determine if this is a bool value for legacy bool lists
                let is_bool_value = value_val.is_int_value()
                    && value_val.get_type().into_int_type().get_bit_width() == 1;

                if is_bool_value {
                    // Use bool-specific list set function for legacy bool lists
                    let list_set = state
                        .module
                        .get_function("vp_list_bool_set")
                        .ok_or_else(|| "vp_list_bool_set not declared".to_string())?;

                    (list_set, value_val)
                } else {
                    // Use generic i64 list set function
                    let list_set = state
                        .module
                        .get_function("vp_list_set")
                        .ok_or_else(|| "vp_list_set not declared".to_string())?;

                    // Convert bool to i64 if needed
                    let value_converted = if value_val.is_int_value() && value_val.get_type().into_int_type().get_bit_width() == 1 {
                        let bool_val = value_val.into_int_value();
                        state.builder.build_int_z_extend(bool_val, state.context.i64_type(), "bool_to_i64")
                            .map_err(|e| format!("Failed to convert bool to i64: {:?}", e))?
                            .into()
                    } else {
                        value_val
                    };

                    (list_set, value_converted)
                }
            };

            let _ = state.ir_builder.build_call(
                state.builder,
                list_set_func,
                &[obj_val.into(), index_val.into(), value_for_list.into()],
                "list_set",
            );
        }
    }
    Ok(())
}

/// Generate augmented assignment statement
pub(crate) fn generate_aug_assign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    op: &BinOp,
    value: &Expr,
) -> Result<(), String> {
    if let Expr::Ident(name, _) = target {
        if let Some(var_info) = state.variables.get(name) {
            let var_type = var_info.var_type;
            let is_scalar = matches!(var_type, VarType::Int | VarType::Float);

            // Get current value
            let current = if is_scalar {
                // For scalars in registers, just use the value directly
                if let VarStorage::Register(val) = &var_info.storage {
                    *val
                } else {
                    // For scalars in stack, load it
                    if let VarStorage::Stack(alloca) = &var_info.storage {
                        match var_type {
                            VarType::Float => {
                                let f64_type = state.context.f64_type();
                                state.builder.build_load(f64_type, *alloca, name).expect("load")
                            }
                            VarType::Int => {
                                let i64_type = state.context.i64_type();
                                state.builder.build_load(i64_type, *alloca, name).expect("load")
                            }
                            _ => return Err("Invalid var type".to_string()),
                        }
                    } else {
                        return Err("Invalid storage".to_string());
                    }
                }
            } else {
                // For pointers, load from stack
                if let VarStorage::Stack(alloca) = &var_info.storage {
                    let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                    state.builder.build_load(ptr_type, *alloca, name).expect("load")
                } else {
                    return Err("Reference types must be stack allocated".to_string());
                }
            };

            let new_val = crate::codegen::expressions::generate_expr(state, value)?;

            let result: inkwell::values::BasicValueEnum<'ctx> = if var_type == VarType::Float {
                let lhs = current.into_float_value();
                let rhs = new_val.into_float_value();
                match op {
                    BinOp::Add => state.builder.build_float_add(lhs, rhs, "fadd").expect("fadd"),
                    BinOp::Sub => state.builder.build_float_sub(lhs, rhs, "fsub").expect("fsub"),
                    BinOp::Mul => state.builder.build_float_mul(lhs, rhs, "fmul").expect("fmul"),
                    BinOp::Div => state.builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv"),
                    BinOp::FloorDiv => {
                        // For float, floor division is floor(lhs / rhs)
                        let div = state.builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv");
                        // Call vp_math_floor
                        let floor_func = state
                            .module
                            .get_function("vp_math_floor")
                            .expect("vp_math_floor not found");
                        let result = state
                            .ir_builder
                            .build_call(state.builder, floor_func, &[div.into()], "floor")
                            .expect("floor call");
                        result.into_float_value()
                    }
                    BinOp::Pow => {
                        // Call vp_pow for float exponentiation
                        let pow_func =
                            state.module.get_function("vp_pow").expect("vp_pow not found");
                        let result = state
                            .ir_builder
                            .build_call(state.builder, pow_func, &[lhs.into(), rhs.into()], "pow")
                            .expect("pow call");
                        result.into_float_value()
                    }
                    _ => {
                        return Err(format!(
                            "Unsupported augmented assignment operator for float: {:?}",
                            op
                        ))
                    }
                }
                .into()
            } else {
                let lhs = current.into_int_value();
                let rhs = new_val.into_int_value();
                match op {
                    BinOp::Add => state.ir_builder.build_add(state.builder, lhs, rhs, "add"),
                    BinOp::Sub => state.ir_builder.build_sub(state.builder, lhs, rhs, "sub"),
                    BinOp::Mul => state.ir_builder.build_mul(state.builder, lhs, rhs, "mul"),
                    BinOp::Div => state.ir_builder.build_div(state.builder, lhs, rhs, "div"),
                    BinOp::Mod => state.builder.build_int_signed_rem(lhs, rhs, "mod").expect("mod"),
                    BinOp::FloorDiv => {
                        state.ir_builder.build_div(state.builder, lhs, rhs, "floordiv")
                    }
                    BinOp::Pow => {
                        // For integer power, call vp_pow_i64
                        let pow_i64_func =
                            state.module.get_function("vp_pow_i64").expect("vp_pow_i64 not found");
                        let result = state
                            .ir_builder
                            .build_call(
                                state.builder,
                                pow_i64_func,
                                &[lhs.into(), rhs.into()],
                                "pow",
                            )
                            .expect("pow call");
                        result.into_int_value()
                    }
                    _ => {
                        return Err(format!(
                            "Unsupported augmented assignment operator for int: {:?}",
                            op
                        ))
                    }
                }
                .into()
            };

            // Store result back - for stack-allocated vars (the default), store to alloca.
            // Register vars are updated in-place in the HashMap (legacy path, kept for safety).
            if let Some(var_info) = state.variables.get(name) {
                match &var_info.storage {
                    VarStorage::Stack(alloca) => {
                        state.builder.build_store(*alloca, result).expect("store");
                    }
                    VarStorage::Register(_) => {
                        // Fallback: update the register value in the HashMap.
                        // This path should not be hit for mutable scalars after the
                        // generate_assign fix, but is kept for safety.
                        state
                            .variables
                            .insert(name.clone(), VarInfo::new_register(result, var_type));
                    }
                }
            }
        } else {
            return Err(format!("Undefined variable in augmented assignment: {}", name));
        }
    }
    Ok(())
}

/// Generate tuple unpacking: a, b, c = (1, 2, 3)
fn generate_tuple_unpack<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    targets: &[Expr],
    value: &Expr,
) -> Result<(), String> {
    // Generate the value expression (should be a tuple pointer)
    let value_ptr = crate::codegen::expressions::generate_expr(state, value)?;

    // The tuple is stored as a pointer to a struct
    let tuple_ptr = if value_ptr.is_pointer_value() {
        value_ptr.into_pointer_value()
    } else {
        return Err("Tuple unpacking requires a tuple value".to_string());
    };

    // For each target, extract the corresponding element from the tuple
    // We use array GEP instead of struct GEP since we don't have the struct type
    for (i, target) in targets.iter().enumerate() {
        if let Expr::Ident(name, _) = target {
            // Get the element from the tuple using array GEP
            // Cast to i8* first, then calculate offset
            let i8_ptr = state
                .builder
                .build_pointer_cast(
                    tuple_ptr,
                    state.context.ptr_type(inkwell::AddressSpace::default()),
                    "tuple_i8_ptr",
                )
                .map_err(|e| format!("Failed to cast tuple pointer: {:?}", e))?;

            // For simplicity, assume all elements are i64 (8 bytes)
            // This is a limitation - proper implementation would need to track element types
            let offset = (i * 8) as i64;
            let elem_i8_ptr = unsafe {
                state.builder.build_in_bounds_gep(
                    state.context.i8_type(),
                    i8_ptr,
                    &[state.context.i64_type().const_int(offset as u64, false)],
                    &format!("elem_{}_i8_ptr", i),
                )
            }
            .map_err(|e| format!("Failed to build GEP for tuple unpacking: {:?}", e))?;

            // Cast back to i64*
            let elem_ptr = state
                .builder
                .build_pointer_cast(
                    elem_i8_ptr,
                    state.context.ptr_type(inkwell::AddressSpace::default()),
                    &format!("elem_{}_ptr", i),
                )
                .map_err(|e| format!("Failed to cast element pointer: {:?}", e))?;

            // Load the element value
            let elem_val = state
                .builder
                .build_load(state.context.i64_type(), elem_ptr, &format!("elem_{}", i))
                .map_err(|e| format!("Failed to load tuple element: {:?}", e))?;

            // Store in variable
            state.variables.insert(name.clone(), VarInfo::new_register(elem_val, VarType::Int));
        } else {
            return Err("Tuple unpacking only supports simple identifiers".to_string());
        }
    }

    Ok(())
}
