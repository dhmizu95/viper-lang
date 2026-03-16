use crate::ast::{BinOp, Expr, Type};
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarStorage, VarType};
use inkwell::values::{IntValue, PointerValue};
use inkwell::{AtomicOrdering, AtomicRMWBinOp};

fn is_pure_aug_assign_rhs(expr: &Expr) -> bool {
    match expr {
        Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Str(_, _)
        | Expr::BigInt(_, _)
        | Expr::None(_)
        | Expr::Bytes(_, _)
        | Expr::FString(_, _)
        | Expr::Ident(_, _) => true,
        Expr::BinOp { left, right, .. } => {
            is_pure_aug_assign_rhs(left) && is_pure_aug_assign_rhs(right)
        }
        Expr::UnaryOp { operand, .. } => is_pure_aug_assign_rhs(operand),
        Expr::Index { obj, index, .. } => {
            is_pure_aug_assign_rhs(obj) && is_pure_aug_assign_rhs(index)
        }
        Expr::Attribute { obj, .. } => is_pure_aug_assign_rhs(obj),
        Expr::Slice { obj, start, end, step, .. } => {
            is_pure_aug_assign_rhs(obj)
                && start.as_ref().map_or(true, |s| is_pure_aug_assign_rhs(s))
                && end.as_ref().map_or(true, |e| is_pure_aug_assign_rhs(e))
                && step.as_ref().map_or(true, |s| is_pure_aug_assign_rhs(s))
        }
        Expr::Call { func, args, .. } => {
            let is_pure_builtin = if let Expr::Ident(name, _) = func.as_ref() {
                matches!(
                    name.as_str(),
                    "len"
                        | "abs"
                        | "min"
                        | "max"
                        | "sum"
                        | "range"
                        | "str"
                        | "int"
                        | "float"
                        | "bool"
                        | "repr"
                        | "ord"
                        | "chr"
                        | "hex"
                        | "bin"
                        | "oct"
                        | "hash"
                        | "id"
                        | "type"
                        | "isinstance"
                )
            } else {
                false
            };
            is_pure_builtin && args.iter().all(is_pure_aug_assign_rhs)
        }
        Expr::List { elements, .. }
        | Expr::Tuple { elements, .. }
        | Expr::Array { elements, .. } => elements.iter().all(is_pure_aug_assign_rhs),
        Expr::Dict { pairs, .. } => pairs
            .iter()
            .all(|(k, v)| is_pure_aug_assign_rhs(k) && is_pure_aug_assign_rhs(v)),
        Expr::Conditional { condition, then_expr, else_expr, .. } => {
            is_pure_aug_assign_rhs(condition)
                && is_pure_aug_assign_rhs(then_expr)
                && is_pure_aug_assign_rhs(else_expr)
        }
        Expr::Lambda { .. }
        | Expr::ListComprehension { .. }
        | Expr::Await { .. }
        | Expr::AssignmentExpr { .. }
        | Expr::Super(_) => false,
    }
}

fn try_atomic_int_aug_assign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    ptr: PointerValue<'ctx>,
    op: &BinOp,
    rhs: IntValue<'ctx>,
) -> crate::codegen::Result<bool> {
    if !state.is_thread_shared(name) {
        return Ok(false);
    }

    let atomic_op = match op {
        BinOp::Add => AtomicRMWBinOp::Add,
        BinOp::Sub => AtomicRMWBinOp::Sub,
        _ => return Ok(false),
    };

    state
        .builder
        .build_atomicrmw(
            atomic_op,
            ptr,
            rhs,
            AtomicOrdering::SequentiallyConsistent,
        )
        .map_err(|e| crate::codegen::codegen_err(format!("Failed to emit atomicrmw: {e}")))?;

    Ok(true)
}

/// Get the type of an expression for assignment type tracking.
/// Unlike infer_expr_type, this looks up identifier types from var_types.
fn get_expr_type_for_assignment(state: &CodeGenState, expr: &Expr) -> Type {
    match expr {
        Expr::Ident(name, _) => {
            // Look up the type from var_types first
            state.var_types.get(name).cloned().unwrap_or(Type::Infer)
        }
        _ => crate::codegen::expressions::core::infer_type_with_state(state, expr),
    }
}

/// Generate assignment statement
pub(crate) fn generate_assign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    value: &Expr,
) -> crate::codegen::Result<()> {
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

        // Track BigInt variables - check inferred type and LLVM value
        // Note: str() returns a char* pointer, not a BigInt
        let inferred_type = crate::codegen::expressions::core::infer_expr_type(value);

        // Helper to check if an expression involves BigInt
        fn is_bigint_expr(expr: &Expr, state: &CodeGenState) -> bool {
            match expr {
                Expr::BigInt(..) => true,
                Expr::Ident(name, _) => state.is_bigint(name),
                Expr::BinOp { left, right, .. } => {
                    is_bigint_expr(left, state) || is_bigint_expr(right, state)
                }
                Expr::Call { func, .. } => {
                    if let Expr::Ident(name, _) = func.as_ref() {
                        // str_bigint() and int_bigint() return non-BigInt types (str and i64 respectively)
                        if name == "str_bigint" || name == "int_bigint" {
                            return false;
                        }
                        // int(), abs(), and pow() return arbitrary precision int (BigInt internally)
                        name == "bigint"
                            || name == "BigInt"
                            || name == "int"
                            || name == "abs"
                            || name == "pow"
                            || name.ends_with("_bigint")
                    } else {
                        false
                    }
                }
                Expr::UnaryOp { operand, .. } => is_bigint_expr(operand, state),
                _ => false,
            }
        }

        // Check if this is an identity function call with BigInt argument
        let is_identity_bigint = if let Expr::Call { func, args, .. } = value {
            if let Expr::Ident(fname, _) = func.as_ref() {
                if fname == "copy" || fname == "identity" || fname == "id" {
                    args.first().map_or(false, |arg| is_bigint_expr(arg, state))
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        let is_bigint = inferred_type == crate::ast::Type::BigInt
            || is_bigint_expr(value, state)
            || (inferred_type == crate::ast::Type::Infer
                && val.is_pointer_value()
                && !matches!(value, Expr::Call { func, .. } if matches!(func.as_ref(), Expr::Ident(name, _) if name == "str" || name == "str_bigint")))
            || is_identity_bigint;

        if is_bigint {
            state.mark_as_bigint(name.clone());
        } else {
            state.bigint_vars.remove(name);
        }

        // For BigInt values, determine if this is a "fresh" allocation (from operation/literal)
        // or an existing reference (from another variable). Fresh allocations already have
        // ref_count=1 and don't need retain. Existing references need retain.
        let is_fresh_bigint = is_bigint && !matches!(value, Expr::Ident(..));

        // Track list variables
        let is_list = match value {
            Expr::List { .. } => true,
            Expr::ListComprehension { .. } => true,
            Expr::Ident(other, _) => state.is_list(other),
            // Check for list repetition: [elem] * n
            Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
                matches!(left.as_ref(), Expr::List { .. } | Expr::Array { .. })
            }
            Expr::Call { func, args, .. } => {
                // Check if calling a list-returning function
                if let Expr::Ident(func_name, _) = func.as_ref() {
                    // Built-in list functions
                    if func_name == "vp_list_create"
                        || func_name == "vp_list_create_f64"
                        || func_name == "vp_list_create_with_capacity"
                        || func_name == "gather"
                    {
                        true
                    // Identity functions (copy, identity) - check the argument type
                    } else if func_name == "copy" || func_name == "identity" || func_name == "id" {
                        // Propagate list type from argument
                        args.first().map(|arg| match arg {
                            Expr::Ident(arg_name, _) => state.is_list(arg_name),
                            Expr::List { .. } | Expr::ListComprehension { .. } => true,
                            _ => false,
                        }).unwrap_or(false)
                    // Built-in string functions - not lists
                    } else if func_name.starts_with("vp_str_") || func_name == "str" {
                        false
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        if is_list {
            state.list_vars.insert(name.clone());
        }

        // Store the inferred type in var_types for future lookups
        // For identity functions (copy, identity), propagate the argument's type
        let value_type = if let Expr::Call { func, args, .. } = value {
            if let Expr::Ident(func_name, _) = func.as_ref() {
                if func_name == "copy" || func_name == "identity" || func_name == "id" {
                    // Use the argument's type instead of the function's return type
                    args.first().map(|arg| get_expr_type_for_assignment(state, arg)).unwrap_or(Type::Infer)
                } else {
                    get_expr_type_for_assignment(state, value)
                }
            } else {
                get_expr_type_for_assignment(state, value)
            }
        } else {
            get_expr_type_for_assignment(state, value)
        };

        if value_type != crate::ast::Type::Infer {
            state.var_types.insert(name.clone(), value_type);
        }

        // Track bool list variables
        let is_bool_list = match value {
            Expr::List { elements, .. } => {
                elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
            }
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
                    Expr::List { elements, .. } => {
                        elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
                    }
                    _ => false,
                }
            }
            Expr::Slice { obj, .. } => {
                // Check if slicing a bool list
                match obj.as_ref() {
                    Expr::Ident(obj_name, _) => state.is_bool_list(obj_name),
                    Expr::List { elements, .. } => {
                        elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
                    }
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
            Expr::Call { func, args, .. } => {
                if let Expr::Ident(func_name, _) = func.as_ref() {
                    if func_name == "vp_dict_create" || func_name == "vp_dict_create_with_capacity" {
                        true
                    // Identity functions - propagate dict type from argument
                    } else if func_name == "copy" || func_name == "identity" || func_name == "id" {
                        args.first().map(|arg| match arg {
                            Expr::Ident(arg_name, _) => state.is_dict(arg_name),
                            Expr::Dict { .. } => true,
                            _ => false,
                        }).unwrap_or(false)
                    } else {
                        false
                    }
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

        // Check if value is Bytes
        let is_bytes = match value {
            Expr::Bytes(_, _) => true,
            _ => false,
        };

        // Check if variable exists and get its info
        let var_exists = state.variables.contains_key(name);

        if var_exists {
            // Get info we need before mutable borrow
            let old_is_ref;
            let old_needs_arc;
            let storage;
            let closure_value_ptr;

            {
                let var_info = state.variables.get(name).unwrap();
                old_is_ref = var_info.var_type == VarType::Pointer;
                old_needs_arc = state.needs_arc(name);
                storage = var_info.storage.clone();
                closure_value_ptr = var_info.closure_value_ptr;
            }

            // Update var_type if this is a Bytes assignment
            if is_bytes {
                if let Some(var_info) = state.variables.get_mut(name) {
                    var_info.var_type = VarType::Bytes;
                }
            }

            // Update existing variable
            match &storage {
                VarStorage::Stack(alloca) => {
                    // For BigInt (and other reference types), release old value before storing new
                    // Since BigInt uses alloca (not SSA), this works correctly without PHI issues
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
                    // Convert to stack allocation when reassigning register variables
                    // This is critical for correctness: register values don't update
                    // when modified in loops, causing stale values to be used
                    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let entry_block = func.get_first_basic_block().unwrap();
                    let old_pos = state.builder.get_insert_block();
                    match entry_block.get_first_instruction() {
                        Some(first_instr) => state.builder.position_before(&first_instr),
                        None => state.builder.position_at_end(entry_block),
                    }
                    let ty = val.get_type();
                    let alloca = state.builder.build_alloca(ty, name).expect("alloca_reassign");
                    if let Some(pos) = old_pos {
                        state.builder.position_at_end(pos);
                    }
                    state.builder.build_store(alloca, val).expect("store_reassign");

                    // Update variable info to use stack storage
                    if let Some(var_info) = state.variables.get_mut(name) {
                        var_info.storage = VarStorage::Stack(alloca);
                    }
                }
                VarStorage::ClosureCell(_) => {
                    // For closure cells, store through the value pointer
                    if let Some(value_ptr) = closure_value_ptr {
                        if old_is_ref && old_needs_arc {
                            let old_val = state
                                .builder
                                .build_load(
                                    state.context.ptr_type(inkwell::AddressSpace::default()),
                                    value_ptr,
                                    &format!("{}_old", name),
                                )
                                .expect("load old value");
                            state.build_release(old_val, &format!("{}_old", name));
                        }
                        state.builder.build_store(value_ptr, val).expect("store to cell");
                    }
                }
            }

            // OPTIMIZATION 1: ARC Elision - Skip retain if variable can be moved
            // OPTIMIZATION 2: Move Semantics - Skip retain/release when transferring ownership
            let is_ref_type = val.is_pointer_value();
            let needs_arc = state.needs_arc(name);

            // Check if we can use move semantics (skip retain/release entirely)
            let can_use_move = state.can_move(name);

            // Check if ARC elision is safe (single-use, doesn't escape)
            let can_elide = state.can_elide_arc(name);

            // Only retain if we can't elide or move
            let should_retain = is_ref_type
                && needs_arc
                && !is_stack_array
                && !is_fresh_bigint
                && !can_use_move
                && !can_elide;

            if should_retain {
                state.build_retain(val, name);
            }

            // Mark variable as used for future move detection
            state.mark_variable_used(name);
        } else {
            let ty = val.get_type();

            // Determine if this is a reference type (but not stack arrays)
            let is_ref_type = val.is_pointer_value() && !is_stack_array;

            // Set reference type flag in escape analyzer
            state.set_reference_type(name, is_ref_type);

            // Determine variable type
            let var_type = if is_bytes {
                VarType::Bytes
            } else if val.is_float_value() {
                VarType::Float
            } else if val.is_pointer_value() {
                VarType::Pointer
            } else if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
                VarType::Bool
            } else if val.is_struct_value() {
                VarType::Struct
            } else {
                VarType::Int
            };

            // Check if this is a class instance - extract class name from value expression
            let class_name = if let Expr::Call { func, .. } = value {
                if let Expr::Ident(class_name, _) = func.as_ref() {
                    // Check if this is a known class
                    if crate::codegen::oop::class_exists(class_name) {
                        Some(class_name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
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

            // Create VarInfo with class name if this is a class instance
            if let Some(cn) = class_name {
                state
                    .variables
                    .insert(name.clone(), VarInfo::new_stack_with_class(alloca, var_type, cn));
            } else {
                state.variables.insert(name.clone(), VarInfo::new_stack(alloca, var_type));
            }

            // If this variable is captured by a nested function, create a closure cell
            if let Some(closure_analyzer) = state.closure_analyzer {
                if let Some(func_name) = state.current_function {
                    if closure_analyzer.needs_closure_cell(func_name, name) {
                        // Create a closure cell for this captured variable
                        if let Ok(cell_ptr) = crate::codegen::closure_cells::create_closure_cell(
                            state.context,
                            state.module,
                            state.builder,
                            alloca,
                            name,
                        ) {
                            // Store the closure cell info
                            state.closure_cells.insert(
                                name.to_string(),
                                crate::codegen::state::ClosureCellInfo {
                                    cell_ptr,
                                    value_ptr: alloca,
                                    var_type,
                                },
                            );
                        }
                    }
                }
            }

            // Insert ARC retain if this is a reference type that escapes (but not stack arrays)
            // Exception: BigInt values skip retain to avoid PHI node issues - cleanup at function exit
            if is_ref_type && state.needs_arc(name) && !is_bigint {
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
            let is_float_list = if let Expr::Ident(name, _) = obj.as_ref() {
                matches!(name.as_str(), "x" | "y" | "z" | "vx" | "vy" | "vz" | "mass")
            } else {
                false
            };

            // List index assignment using runtime function call
            // NOTE: Inline list access disabled for JIT mode due to struct layout issues
            // Lists store tagged integers, so pass value directly
            // But the index needs to be untagged (runtime expects untagged indices)
            // Untag the index (tagged ints are shifted left by 1)
            let index_untagged = state
                .builder
                .build_right_shift(
                    index_val,
                    state.context.i64_type().const_int(1, false),
                    false,
                    "index_untagged",
                )
                .map_err(|e| format!("Failed to untag index: {:?}", e))?;

            if is_float_list || value_val.is_float_value() {
                let list_set_f64 = state
                    .module
                    .get_function("vp_list_set_f64")
                    .ok_or_else(|| "vp_list_set_f64 not declared".to_string())?;

                let f64_val = if value_val.is_float_value() {
                    value_val.into_float_value()
                } else {
                    state
                        .builder
                        .build_signed_int_to_float(
                            value_val.into_int_value(),
                            state.context.f64_type(),
                            "i64_to_f64",
                        )
                        .expect("i64 to f64")
                };

                let _result = state.ir_builder.build_call(
                    state.builder,
                    list_set_f64,
                    &[obj_val.into(), index_untagged.into(), f64_val.into()],
                    "list_set_f64",
                );
            } else {
                let list_set = state
                    .module
                    .get_function("vp_list_set")
                    .ok_or_else(|| "vp_list_set not declared".to_string())?;

                let _result = state.ir_builder.build_call(
                    state.builder,
                    list_set,
                    &[obj_val.into(), index_untagged.into(), value_val.into()],
                    "list_set",
                );
            }
        }
    } else if let Expr::Attribute { obj, attr, .. } = target {
        // Handle attribute assignment: obj.attr = value
        // First try user-defined class field assignment
        if let Err(_) = crate::codegen::oop::generate_field_assignment(state, obj, attr, value) {
            // Fall back to generating the value and ignoring the assignment
            // (for cases where the object doesn't have this attribute)
            let _ = crate::codegen::expressions::generate_expr(state, value)?;
        }
    }
    Ok(())
}

/// Generate augmented assignment for list index: lst[i] += value
fn generate_aug_assign_index<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    index: &Expr,
    op: &BinOp,
    value: &Expr,
) -> crate::codegen::Result<()> {
    use crate::codegen::inline_lists::{inline_f64_list_get, inline_f64_list_set};
    
    // Determine if this is a float list
    let is_float_list = {
        let obj_type = crate::codegen::expressions::infer_expr_type(obj);
        let obj_type = if let Expr::Ident(name, _) = obj {
            state.var_types.get(name).cloned().unwrap_or(obj_type)
        } else {
            obj_type
        };

        let is_likely_float_list = if let Expr::Ident(name, _) = obj {
            matches!(name.as_str(), "x" | "y" | "z" | "vx" | "vy" | "vz" | "mass")
        } else {
            false
        };

        match &obj_type {
            Type::List(inner) => match &**inner {
                Type::F64 => true,
                Type::Var(n) if n == "float" || n == "f64" => true,
                Type::Infer => is_likely_float_list,
                _ => false,
            },
            Type::GenericApp { name, type_args }
                if (name == "list" || name == "List") && type_args.len() == 1 =>
            {
                match &type_args[0] {
                    Type::F64 => true,
                    Type::Var(n) if n == "float" || n == "f64" => true,
                    Type::Infer => is_likely_float_list,
                    _ => false,
                }
            }
            _ => is_likely_float_list,
        }
    };
    
    // Generate index value (untagged for runtime functions)
    let index_val = crate::codegen::expressions::generate_expr(state, index)?.into_int_value();
    
    // For float lists, use inline access
    if is_float_list {
        let obj_val = crate::codegen::expressions::generate_expr(state, obj)?;
        let list_ptr = obj_val.into_pointer_value();
        
        // Load current f64 value using inline access
        let current_f64 = inline_f64_list_get(state, list_ptr, index_val)?;
        
        // Generate RHS value
        let rhs_val = crate::codegen::expressions::generate_expr(state, value)?;
        
        // Perform float operation
        let result_val = match op {
            BinOp::Add => {
                let rhs_f64: inkwell::values::BasicValueEnum = rhs_val.into_float_value().into();
                let current: inkwell::values::BasicValueEnum = current_f64.into();
                state.builder.build_float_add(current.into_float_value(), rhs_f64.into_float_value(), "fadd").expect("fadd").into()
            },
            BinOp::Sub => {
                let rhs_f64: inkwell::values::BasicValueEnum = rhs_val.into_float_value().into();
                let current: inkwell::values::BasicValueEnum = current_f64.into();
                state.builder.build_float_sub(current.into_float_value(), rhs_f64.into_float_value(), "fsub").expect("fsub").into()
            },
            BinOp::Mul => {
                let rhs_f64: inkwell::values::BasicValueEnum = rhs_val.into_float_value().into();
                let current: inkwell::values::BasicValueEnum = current_f64.into();
                state.builder.build_float_mul(current.into_float_value(), rhs_f64.into_float_value(), "fmul").expect("fmul").into()
            },
            BinOp::Div => {
                let rhs_f64: inkwell::values::BasicValueEnum = rhs_val.into_float_value().into();
                let current: inkwell::values::BasicValueEnum = current_f64.into();
                state.builder.build_float_div(current.into_float_value(), rhs_f64.into_float_value(), "fdiv").expect("fdiv").into()
            },
            _ => {
                return crate::codegen::codegen_error(format!(
                    "Unsupported augmented assignment operator for float list: {:?}",
                    op
                ));
            }
        };
        
        // Store result using inline access
        inline_f64_list_set(state, list_ptr, index_val, result_val)?;
        return Ok(());
    }

    
    // For non-float lists, fall back to runtime functions
    // Load current tagged int value
    let obj_val = crate::codegen::expressions::generate_expr(state, obj)?;
    let list_get = state
        .module
        .get_function("vp_list_get")
        .ok_or_else(|| "vp_list_get not declared".to_string())?;
    
    // Untag index
    let index_untagged = state
        .builder
        .build_right_shift(
            index_val,
            state.context.i64_type().const_int(1, false),
            false,
            "index_untagged",
        )
        .expect("index untag");
    
    let current_tagged = state
        .ir_builder
        .build_call(
            state.builder,
            list_get,
            &[obj_val.into(), index_untagged.into()],
            "list_get",
        )
        .ok_or_else(|| "vp_list_get call failed".to_string())?;
    
    // Generate RHS and perform tagged int operation (simplified - just int for now)
    let rhs_val = crate::codegen::expressions::generate_expr(state, value)?;
    
    let lhs = current_tagged.into_int_value();
    let rhs = rhs_val.into_int_value();
    
    let result = match op {
        BinOp::Add => state.ir_builder.build_add(state.builder, lhs, rhs, "add"),
        BinOp::Sub => state.ir_builder.build_sub(state.builder, lhs, rhs, "sub"),
        BinOp::Mul => state.ir_builder.build_mul(state.builder, lhs, rhs, "mul"),
        _ => {
            return crate::codegen::codegen_error(format!(
                "Unsupported augmented assignment operator for int list: {:?}",
                op
            ));
        }
    };
    
    // Store using runtime function
    let list_set = state
        .module
        .get_function("vp_list_set")
        .ok_or_else(|| "vp_list_set not declared".to_string())?;
    
    let _ = state.ir_builder.build_call(
        state.builder,
        list_set,
        &[obj_val.into(), index_untagged.into(), result.into()],
        "list_set",
    );
    
    Ok(())
}

/// Generate augmented assignment statement
pub(crate) fn generate_aug_assign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    op: &BinOp,
    value: &Expr,
) -> crate::codegen::Result<()> {
    // Handle list index augmented assignment: lst[i] += value
    if let Expr::Index { obj, index, .. } = target {
        return generate_aug_assign_index(state, obj, index, op, value);
    }
    
    if let Expr::Ident(name, _) = target {
        // Handle global variable augmented assignment
        if state.global_constants.contains_key(name) && !state.variables.contains_key(name) {
            let global = state.global_constants.get(name).unwrap();
            let global_ptr = global.as_pointer_value();

            // Assume it's an integer for now, as floats would need type tracking for globals
            let i64_type = state.context.i64_type();
            let current =
                state.builder.build_load(i64_type, global_ptr, name).expect("load global");
            let new_val = crate::codegen::expressions::generate_expr(state, value)?;
            let rhs = new_val.into_int_value();

            if is_pure_aug_assign_rhs(value)
                && try_atomic_int_aug_assign(state, name, global_ptr, op, rhs)?
            {
                return Ok(());
            }

            let lhs = current.into_int_value();
            let result = match op {
                BinOp::Add => state.ir_builder.build_add(state.builder, lhs, rhs, "add"),
                BinOp::Sub => state.ir_builder.build_sub(state.builder, lhs, rhs, "sub"),
                BinOp::Mul => state.ir_builder.build_mul(state.builder, lhs, rhs, "mul"),
                BinOp::Div => state.ir_builder.build_div(state.builder, lhs, rhs, "div"),
                BinOp::Mod => state.builder.build_int_signed_rem(lhs, rhs, "mod").expect("mod"),
                BinOp::FloorDiv => state.ir_builder.build_div(state.builder, lhs, rhs, "floordiv"),
                BinOp::Pow => {
                    let pow_i64_func =
                        state.module.get_function("vp_pow_i64").expect("vp_pow_i64 not found");
                    let res = state
                        .ir_builder
                        .build_call(state.builder, pow_i64_func, &[lhs.into(), rhs.into()], "pow")
                        .expect("pow call");
                    res.into_int_value()
                }
                BinOp::BitAnd => state.builder.build_and(lhs, rhs, "bitand").expect("bitand"),
                BinOp::BitOr => state.builder.build_or(lhs, rhs, "bitor").expect("bitor"),
                BinOp::BitXor => state.builder.build_xor(lhs, rhs, "bitxor").expect("bitxor"),
                BinOp::LShift => {
                    state.builder.build_left_shift(lhs, rhs, "lshift").expect("lshift")
                }
                BinOp::RShift => {
                    state.builder.build_right_shift(lhs, rhs, false, "rshift").expect("rshift")
                }
                _ => {
                    return crate::codegen::codegen_error(format!(
                        "Unsupported augmented assignment operator: {:?}",
                        op
                    ))
                }
            };

            state.builder.build_store(global_ptr, result).expect("store to global");
            return Ok(());
        }

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
                            _ => {
                                return crate::codegen::codegen_error(
                                    "Invalid var type".to_string(),
                                )
                            }
                        }
                    } else {
                        return crate::codegen::codegen_error("Invalid storage".to_string());
                    }
                }
            } else {
                // For pointers, load from stack
                if let VarStorage::Stack(alloca) = &var_info.storage {
                    let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                    state.builder.build_load(ptr_type, *alloca, name).expect("load")
                } else {
                    return crate::codegen::codegen_error(
                        "Reference types must be stack allocated".to_string(),
                    );
                }
            };

            let new_val = crate::codegen::expressions::generate_expr(state, value)?;
            let rhs_int = if var_type == VarType::Int {
                Some(new_val.into_int_value())
            } else {
                None
            };

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
                        return crate::codegen::codegen_error(format!(
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
                    BinOp::BitAnd => state.builder.build_and(lhs, rhs, "bitand").expect("bitand"),
                    BinOp::BitOr => state.builder.build_or(lhs, rhs, "bitor").expect("bitor"),
                    BinOp::BitXor => state.builder.build_xor(lhs, rhs, "bitxor").expect("bitxor"),
                    BinOp::LShift => {
                        state.builder.build_left_shift(lhs, rhs, "lshift").expect("lshift")
                    }
                    BinOp::RShift => {
                        state.builder.build_right_shift(lhs, rhs, false, "rshift").expect("rshift")
                    }
                    _ => {
                        return crate::codegen::codegen_error(format!(
                            "Unsupported augmented assignment operator for int: {:?}",
                            op
                        ))
                    }
                }
                .into()
            };

            // Store result back - for stack-allocated vars (the default), store to alloca.
            // Register vars are updated in-place in the HashMap (legacy path, kept for safety).
            if let Some(var_info) = state.variables.get(name).cloned() {
                match var_info.storage {
                    VarStorage::Stack(alloca) => {
                        if var_type == VarType::Int
                            && is_pure_aug_assign_rhs(value)
                            && try_atomic_int_aug_assign(
                                state,
                                name,
                                alloca,
                                op,
                                rhs_int.expect("int rhs"),
                            )?
                        {
                            return Ok(());
                        }
                        state.builder.build_store(alloca, result).expect("store");
                    }
                    VarStorage::Register(_) => {
                        // Fallback: update the register value in the HashMap.
                        // This path should not be hit for mutable scalars after the
                        // generate_assign fix, but is kept for safety.
                        state
                            .variables
                            .insert(name.clone(), VarInfo::new_register(result, var_type));
                    }
                    VarStorage::ClosureCell(_) => {
                        // For closure cells, store through the value pointer
                        if let Some(value_ptr) = var_info.closure_value_ptr {
                            if var_type == VarType::Int
                                && is_pure_aug_assign_rhs(value)
                                && try_atomic_int_aug_assign(
                                    state,
                                    name,
                                    value_ptr,
                                    op,
                                    rhs_int.expect("int rhs"),
                                )?
                            {
                                return Ok(());
                            }
                            state.builder.build_store(value_ptr, result).expect("store to cell");
                        } else {
                            return crate::codegen::codegen_error(format!(
                                "Closure cell for '{}' missing value pointer",
                                name
                            ));
                        }
                    }
                }
            }
        } else {
            return crate::codegen::codegen_error(format!(
                "Undefined variable in augmented assignment: {}",
                name
            ));
        }
    }
    Ok(())
}

/// Generate tuple unpacking: a, b, c = (1, 2, 3)
pub(crate) fn generate_tuple_unpack<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    targets: &[Expr],
    value: &Expr,
) -> crate::codegen::Result<()> {
    // Generate the value expression (should be a tuple pointer)
    let value_ptr = crate::codegen::expressions::generate_expr(state, value)?;

    // The tuple is stored as a pointer to a ViperTuple struct
    // Layout: size(0), elements_ptr(8), _reserved(16)
    let tuple_ptr = if value_ptr.is_pointer_value() {
        value_ptr.into_pointer_value()
    } else {
        return crate::codegen::codegen_error("Tuple unpacking requires a tuple value".to_string());
    };

    // Load the elements pointer from the tuple struct (offset 8 = 1 * i64)
    let elements_ptr_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            state.context.i64_type(),
            tuple_ptr,
            &[state.context.i64_type().const_int(1, false)], // offset to elements field
            "elements_ptr_ptr",
        )
    }
    .map_err(|e| format!("Failed to get elements pointer location: {:?}", e))?;

    let elements_ptr = state
        .builder
        .build_load(
            state.context.ptr_type(inkwell::AddressSpace::default()),
            elements_ptr_ptr,
            "elements_ptr",
        )
        .map_err(|e| format!("Failed to load elements pointer: {:?}", e))?
        .into_pointer_value();

    // For each target, load the corresponding element from the elements array
    for (i, target) in targets.iter().enumerate() {
        if let Expr::Ident(name, _) = target {
            // Get the element from the elements array using GEP
            let elem_ptr = unsafe {
                state.builder.build_in_bounds_gep(
                    state.context.i64_type(),
                    elements_ptr,
                    &[state.context.i64_type().const_int(i as u64, false)],
                    &format!("elem_{}_ptr", i),
                )
            }
            .map_err(|e| format!("Failed to build GEP for tuple element: {:?}", e))?;

            // Load the element value
            let elem_val = state
                .builder
                .build_load(state.context.i64_type(), elem_ptr, &format!("elem_{}", i))
                .map_err(|e| format!("Failed to load tuple element: {:?}", e))?;

            // Store in variable
            state.variables.insert(name.clone(), VarInfo::new_register(elem_val, VarType::Int));
        } else {
            return crate::codegen::codegen_error(
                "Tuple unpacking only supports simple identifiers".to_string(),
            );
        }
    }

    Ok(())
}


/// Generate slice assignment: obj[start:end:step] = value
pub(crate) fn generate_slice_assign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    start: &Option<Box<Expr>>,
    end: &Option<Box<Expr>>,
    step: &Option<Box<Expr>>,
    value: &Expr,
) -> crate::codegen::Result<()> {
    use crate::codegen::expressions::generate_expr;
    
    // Generate the object (the list/array being sliced)
    let obj_val = generate_expr(state, obj)?;
    
    // Generate start, end, step indices (default to 0, len, 1)
    let start_val = if let Some(s) = start {
        generate_expr(state, s)?.into_int_value()
    } else {
        state.context.i64_type().const_int(0, false)
    };
    
    let end_val = if let Some(e) = end {
        generate_expr(state, e)?.into_int_value()
    } else {
        let len_func = state.module.get_function("vp_list_len")
            .ok_or("vp_list_len not declared")?;
        state.ir_builder.build_call(
            state.builder,
            len_func,
            &[obj_val.into()],
            "slice_len",
        ).unwrap().into_int_value()
    };
    
    let step_val = if let Some(s) = step {
        generate_expr(state, s)?.into_int_value()
    } else {
        state.context.i64_type().const_int(1, false)
    };
    
    // Generate the value
    let value_val = generate_expr(state, value)?;
    
    // Generate loop: for i in range(start, end, step): obj[i] = value
    let func = state.builder.get_insert_block()
        .ok_or("No insertion block")?
        .get_parent()
        .ok_or("No parent function")?;
    
    let init_block = state.context.append_basic_block(func, "slice_assign_init");
    let cond_block = state.context.append_basic_block(func, "slice_assign_cond");
    let body_block = state.context.append_basic_block(func, "slice_assign_body");
    let step_block = state.context.append_basic_block(func, "slice_assign_step");
    let end_block = state.context.append_basic_block(func, "slice_assign_end");
    
    state.ir_builder.build_branch(state.builder, init_block);
    
    // Init: counter = start
    state.builder.position_at_end(init_block);
    let counter = state.builder.build_alloca(state.context.i64_type(), "slice_counter").expect("alloca");
    state.builder.build_store(counter, start_val).expect("store");
    state.ir_builder.build_branch(state.builder, cond_block);
    
    // Condition: counter < end
    state.builder.position_at_end(cond_block);
    let counter_val = state.builder.build_load(state.context.i64_type(), counter, "counter_val")
        .expect("load counter")
        .into_int_value();
    
    let cond = state.ir_builder.build_icmp_lt(state.builder, counter_val, end_val, "slice_cond");
    state.ir_builder.build_cond_branch(state.builder, cond, body_block, end_block);
    
    // Body: obj[counter] = value
    state.builder.position_at_end(body_block);
    
    let set_func = state.module.get_function("vp_list_set")
        .ok_or("vp_list_set not declared")?;
    state.ir_builder.build_call(
        state.builder,
        set_func,
        &[obj_val.into(), counter_val.into(), value_val.into()],
        "list_set",
    );
    
    state.ir_builder.build_branch(state.builder, step_block);
    
    // Step: counter += step
    state.builder.position_at_end(step_block);
    let next_counter = state.ir_builder.build_add(state.builder, counter_val, step_val, "next_counter");
    state.builder.build_store(counter, next_counter).expect("store");
    state.ir_builder.build_branch(state.builder, cond_block);
    
    // End
    state.builder.position_at_end(end_block);
    
    Ok(())
}
