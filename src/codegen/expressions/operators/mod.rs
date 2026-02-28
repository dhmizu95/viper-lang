//! Expression code generation for Viper - Operators

use crate::ast::{BinOp, Expr, UnaryOp};
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

pub mod arithmetic;
pub mod bigint;
pub mod comparison;
pub mod incdec;
pub mod logical;
pub mod membership;
pub mod strings;

pub use incdec::generate_conditional;
pub use incdec::generate_incdec;
pub use strings::generate_str_concat;

/// Generate binary operation
pub fn generate_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    if matches!(op, BinOp::And | BinOp::Or) {
        return logical::generate_logical_op(state, left, op, right);
    }

    if matches!(op, BinOp::In | BinOp::NotIn) {
        return membership::generate_membership_op(state, left, op, right);
    }

    if matches!(op, BinOp::NullCoalesce) {
        return logical::generate_null_coalesce_op(state, left, right);
    }

    // Special case: Handle list * int for list/array literals: [elem] * n
    // This must be checked BEFORE generating the left operand to avoid generating
    // the full list literal first
    if *op == BinOp::Mul {
        let elements = match left {
            Expr::List { elements, .. } => Some(elements),
            Expr::Array { elements, .. } => Some(elements),
            _ => None,
        };

        if let Some(elems) = elements {
            if let Some(elem) = elems.first() {
                // Only generate the element expression and count, not the full list
                let count_val = generate_expr(state, right)?;
                let count_int = count_val.into_int_value();

                let (elem_val, func_name) = match elem {
                    Expr::Bool(true, _) => {
                        let val: inkwell::values::BasicMetadataValueEnum = state.context.bool_type().const_int(1, false).into();
                        (val, "vp_bitvec_repeat")  // Use bit vector for bool lists
                    }
                    Expr::Bool(false, _) => {
                        let val: inkwell::values::BasicMetadataValueEnum = state.context.bool_type().const_int(0, false).into();
                        (val, "vp_bitvec_repeat")  // Use bit vector for bool lists
                    }
                    Expr::Int(val, _) => {
                         let val: inkwell::values::BasicMetadataValueEnum = state.ir_builder.i64_const(*val).into();
                         (val, "vp_list_repeat")
                     }
                    Expr::Float(val, _) => {
                         let f64_val = state.context.f64_type().const_float(*val as f64);
                         let i64_val = state.builder.build_float_to_signed_int(f64_val, state.context.i64_type(), "float_to_int")
                             .map_err(|e| format!("Failed to convert float to int: {:?}", e))?;
                         (i64_val.into(), "vp_list_repeat")
                     }
                    _ => {
                        let val = generate_expr(state, elem)?;
                        if val.is_int_value() {
                            let int_val = val.into_int_value();
                            if int_val.get_type().get_bit_width() == 1 {
                                (int_val.into(), "vp_bitvec_repeat")  // Use bit vector for bool lists
                            } else {
                                (int_val.into(), "vp_list_repeat")
                            }
                        } else {
                            return Err(
                                "List repeat requires integer or boolean elements".to_string()
                            );
                        }
                    }
                };

                let list_repeat_func = state
                    .module
                    .get_function(func_name)
                    .ok_or_else(|| format!("{} not declared", func_name))?;

                let result = state
                    .ir_builder
                    .build_call(
                        state.builder,
                        list_repeat_func,
                        &[elem_val, count_int.into()],
                        "list_repeat",
                    )
                    .expect("list_repeat call");

                return Ok(result.into());
            }
        }
    }

    // Generate both operands for other operations
    let lhs_val = generate_expr(state, left)?;
    let rhs_val = generate_expr(state, right)?;

    // Check if either operand is BigInt (pointer type that represents BigInt)
    let is_bigint_left = bigint::is_bigint_expr(left, state);
    let is_bigint_right = bigint::is_bigint_expr(right, state);

    if is_bigint_left || is_bigint_right {
        return bigint::generate_bigint_binop(state, lhs_val, rhs_val, op);
    }

    // Handle string concatenation with + operator
    if *op == BinOp::Add {
        // Check if both operands are strings (pointer types)
        if lhs_val.is_pointer_value() && rhs_val.is_pointer_value() {
            return strings::generate_str_concat(state, lhs_val, rhs_val);
        }
    }

    // List concatenation: list1 + list2
    if matches!(op, BinOp::Add) {
        let is_list_left = match left {
            Expr::List { .. } | Expr::ListComprehension { .. } => true,
            Expr::Ident(name, _) => state.is_list(name),
            _ => false,
        };
        let is_list_right = match right {
            Expr::List { .. } | Expr::ListComprehension { .. } => true,
            Expr::Ident(name, _) => state.is_list(name),
            _ => false,
        };

        // Check if these are bool lists (bit vectors)
        let is_bool_list_left = match left {
            Expr::Ident(name, _) => state.is_bool_list(name),
            Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
            _ => false,
        };
        let is_bool_list_right = match right {
            Expr::Ident(name, _) => state.is_bool_list(name),
            Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
            _ => false,
        };

        if is_list_left && is_list_right {
            let left_val = generate_expr(state, left)?;
            let right_val = generate_expr(state, right)?;
            
            // Use bit vector concat for bool lists
            let concat_func = if is_bool_list_left && is_bool_list_right {
                state
                    .module
                    .get_function("vp_bitvec_concat")
                    .ok_or_else(|| "vp_bitvec_concat not declared".to_string())?
            } else {
                state
                    .module
                    .get_function("vp_list_concat")
                    .ok_or_else(|| "vp_list_concat not declared".to_string())?
            };
            
            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    concat_func,
                    &[left_val.into(), right_val.into()],
                    "list_concat",
                )
                .expect("list_concat call");
            return Ok(result.into());
        }
    }

    // Handle comparison operators on pointers (identity comparison)
    if lhs_val.is_pointer_value() && rhs_val.is_pointer_value() {
        return comparison::generate_pointer_binop(state.builder, state.context, lhs_val, rhs_val, op);
    }

    // Reject pointer values in arithmetic operations (except for Add with strings, handled above)
    if lhs_val.is_pointer_value() || rhs_val.is_pointer_value() {
        return Err("Binary operators cannot be applied to pointer values (lists)".to_string());
    }

    // Handle boolean comparisons (both operands are i1)
    if lhs_val.is_int_value()
        && rhs_val.is_int_value()
        && lhs_val.get_type().into_int_type().get_bit_width() == 1
        && rhs_val.get_type().into_int_type().get_bit_width() == 1
    {
        return arithmetic::generate_bool_binop(state, lhs_val, rhs_val, op);
    }

    // Auto-convert int to float when one operand is float
    if lhs_val.is_float_value() && !rhs_val.is_float_value() {
        // Convert rhs (int) to float
        let rhs_int = rhs_val.into_int_value();
        let rhs_float = state
            .builder
            .build_signed_int_to_float(rhs_int, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion");
        return arithmetic::generate_float_binop(state, lhs_val, rhs_float.into(), op);
    } else if !lhs_val.is_float_value() && rhs_val.is_float_value() {
        // Convert lhs (int) to float
        let lhs_int = lhs_val.into_int_value();
        let lhs_float = state
            .builder
            .build_signed_int_to_float(lhs_int, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion");
        return arithmetic::generate_float_binop(state, lhs_float.into(), rhs_val, op);
    } else if lhs_val.is_float_value() {
        return arithmetic::generate_float_binop(state, lhs_val, rhs_val, op);
    } else {
        return arithmetic::generate_int_binop(state, lhs_val, rhs_val, op);
    }
}

/// Generate unary operation
pub fn generate_unary<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    op: &UnaryOp,
    operand: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Handle increment and decrement operators specially
    if matches!(
        op,
        UnaryOp::PreIncrement
            | UnaryOp::PreDecrement
            | UnaryOp::PostIncrement
            | UnaryOp::PostDecrement
    ) {
        return incdec::generate_incdec(state, op, operand);
    }

    // Handle error propagation operator (?) specially
    // This needs to check the Result and return early on error
    if matches!(op, UnaryOp::Unwrap | UnaryOp::UnwrapOrDefault) {
        return generate_unwrap(state, op, operand);
    }

    let val = generate_expr(state, operand)?;

    // Check for BigInt
    if bigint::is_bigint_expr(operand, state) {
        return bigint::generate_bigint_unary(state, op, val);
    }

    if val.is_float_value() {
        let float_val = val.into_float_value();
        match op {
            UnaryOp::Neg => {
                Ok(state.builder.build_float_neg(float_val, "fneg").expect("fneg").into())
            }
            UnaryOp::Pos => Ok(val),
            UnaryOp::Not | UnaryOp::Invert => {
                Err(format!("Unary operator {:?} not supported for float types", op))
            }
            UnaryOp::PreIncrement
            | UnaryOp::PreDecrement
            | UnaryOp::PostIncrement
            | UnaryOp::PostDecrement
            | UnaryOp::Unwrap
            | UnaryOp::UnwrapOrDefault => {
                unreachable!("Increment/Decrement/Unwrap handled earlier")
            }
        }
    } else {
        let int_val = val.into_int_value();
        match op {
            UnaryOp::Neg => Ok(state.builder.build_int_neg(int_val, "neg").expect("neg").into()),
            UnaryOp::Not => Ok(state.builder.build_not(int_val, "not").expect("not").into()),
            UnaryOp::Pos => Ok(val),
            UnaryOp::Invert => Ok(state
                .builder
                .build_xor(int_val, state.context.i64_type().const_all_ones(), "invert")
                .expect("invert")
                .into()),
            UnaryOp::PreIncrement
            | UnaryOp::PreDecrement
            | UnaryOp::PostIncrement
            | UnaryOp::PostDecrement
            | UnaryOp::Unwrap
            | UnaryOp::UnwrapOrDefault => {
                unreachable!("Increment/Decrement/Unwrap handled earlier")
            }
        }
    }
}

/// Generate code for the `?` unwrap operator
/// This checks if the Result is Ok or Err, and returns early on Err
fn generate_unwrap<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    op: &UnaryOp,
    operand: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Generate the operand expression (should be a Result)
    let result_val = generate_expr(state, operand)?;
    
    // For now, implement a simple version that assumes Result is represented as:
    // - A struct { is_ok: i8, value: i64 }
    // In a full implementation, this would need proper tagged union representation
    
    // Get the Result unwrap function from runtime
    // For now, we'll use a simple inline implementation
    
    // The Result is expected to be a pointer to a struct
    let result_ptr = if result_val.is_pointer_value() {
        result_val.into_pointer_value()
    } else {
        // If not a pointer, it's already the unwrapped value (simplified for non-pointer Results)
        return Ok(result_val);
    };
    
    // For LLVM 20+, we need to cast to opaque pointer first, then back to struct
    // This is a simplified implementation - proper Result handling needs more work
    let i8_ptr_type = state.context.i8_type().ptr_type(inkwell::AddressSpace::default());
    let result_ptr_cast = state.builder
        .build_pointer_cast(result_ptr, i8_ptr_type, "result_ptr_cast")
        .map_err(|e| format!("Failed to cast result pointer: {:?}", e))?;
    
    // Get the is_ok field by GEP with offset 0
    let is_ok_ptr = state.builder
        .build_struct_gep(state.context.i8_type().array_type(9), result_ptr_cast, 0, "is_ok_ptr")
        .map_err(|e| format!("Failed to get is_ok field: {:?}", e))?;
    
    let is_ok = state.builder
        .build_load(state.context.i8_type(), is_ok_ptr, "is_ok")
        .map_err(|e| format!("Failed to load is_ok: {:?}", e))?
        .into_int_value();
    
    // Get the current function from the builder
    let func = state.builder.get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or("Failed to get current function")?;
    
    // Create blocks for Ok and Err cases
    let ok_block = state.context.append_basic_block(func, "result_ok");
    let err_block = state.context.append_basic_block(func, "result_err");
    let continue_block = state.context.append_basic_block(func, "result_continue");
    
    // Branch based on is_ok
    state.builder.build_conditional_branch(
        is_ok,
        ok_block,
        err_block,
    ).map_err(|e| format!("Failed to build conditional branch: {:?}", e))?;
    
    // Ok block: extract and return the value (at offset 8 for i64 alignment)
    state.builder.position_at_end(ok_block);
    let value_ptr = state.builder
        .build_struct_gep(state.context.i8_type().array_type(9), result_ptr_cast, 1, "value_ptr")
        .map_err(|e| format!("Failed to get value field: {:?}", e))?;
    let ok_value = state.builder
        .build_load(state.context.i64_type(), value_ptr, "value")
        .map_err(|e| format!("Failed to load value: {:?}", e))?;
    state.builder.build_unconditional_branch(continue_block)
        .map_err(|e| format!("Failed to build branch: {:?}", e))?;
    let ok_block_end = state.builder.get_insert_block().unwrap();
    
    // Err block: return early with the error
    state.builder.position_at_end(err_block);
    // For now, just panic - in a full implementation, this would propagate the error
    let err_msg = state.context.const_string(b"Error propagated via ?", true);
    let err_msg_global = state.module.add_global(
        err_msg.get_type(),
        None,
        "unwrap_err_msg",
    );
    err_msg_global.set_initializer(&err_msg);
    let err_msg_ptr = state.builder.build_pointer_cast(
        err_msg_global.as_pointer_value(),
        state.context.i8_type().ptr_type(inkwell::AddressSpace::default()),
        "err_msg_ptr",
    ).map_err(|e| format!("Failed to cast error message: {:?}", e))?;
    
    // Call panic or return error
    if let Some(panic_func) = state.module.get_function("viper_panic") {
        state.builder.build_call(panic_func, &[err_msg_ptr.into()], "panic")
            .map_err(|e| format!("Failed to call panic: {:?}", e))?;
    }
    // Unreachable after panic
    state.builder.build_unreachable().map_err(|e| format!("Failed to build unreachable: {:?}", e))?;
    let err_block_end = state.builder.get_insert_block().unwrap();
    
    // Continue block: phi node to merge values
    state.builder.position_at_end(continue_block);
    let phi = state.builder.build_phi(state.context.i64_type(), "result_value")
        .map_err(|e| format!("Failed to build phi: {:?}", e))?;
    phi.add_incoming(&[(&ok_value, ok_block_end)]);
    
    Ok(phi.as_basic_value())
}
