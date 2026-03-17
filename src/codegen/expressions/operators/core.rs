//! Expression code generation for Viper - Operators

use crate::ast::{BinOp, Expr, Type, UnaryOp};
use crate::codegen::expressions::collections::generate_list;
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

pub use super::arithmetic;
pub use super::bigint;
pub use super::comparison;
pub use super::incdec;
pub use super::logical;
pub use super::membership;
pub use super::strings;

/// Generate binary operation
pub fn generate_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
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
        // Handle bytearray * int for bytearray repetition
        let is_bytearray_left = match left {
            Expr::Call { func, .. } => {
                if let Expr::Ident(name, _) = func.as_ref() {
                    name == "bytearray"
                } else {
                    false
                }
            }
            _ => false,
        };

        if is_bytearray_left {
            // Generate bytearray and count
            let ba_val = generate_expr(state, left)?;
            let count_val = generate_expr(state, right)?;
            let count_int = if count_val.is_int_value() {
                count_val.into_int_value()
            } else {
                return crate::codegen::codegen_error(
                    "bytearray repeat count must be an integer".to_string(),
                );
            };

            // Untag the count (tagged ints are shifted left by 1)
            let count_untagged = state
                .builder
                .build_right_shift(
                    count_int,
                    state.context.i64_type().const_int(1, false),
                    false,
                    "count_untagged",
                )
                .expect("untag count");

            // Call vp_bytearray_repeat
            let repeat_func = state
                .module
                .get_function("vp_bytearray_repeat")
                .ok_or_else(|| "vp_bytearray_repeat not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    repeat_func,
                    &[ba_val.into(), count_untagged.into()],
                    "ba_repeat",
                )
                .expect("ba_repeat call");

            return Ok(result.into());
        }

        let elements = match left {
            Expr::List { elements, .. } => Some(elements),
            Expr::Array { elements, .. } => Some(elements),
            _ => None,
        };

        if let Some(elems) = elements {
            // Handle multi-element list repetition: [a, b, c] * n
            if elems.len() > 1 {
                // First create the base list
                let base_list = generate_list(state, elems)?;
                let base_ptr = base_list.into_pointer_value();

                // Generate count
                let count_val = generate_expr(state, right)?;
                let count_int = count_val.into_int_value();

                // Untag the count
                let count_untagged = state
                    .builder
                    .build_right_shift(
                        count_int,
                        state.context.i64_type().const_int(1, false),
                        false,
                        "count_untagged",
                    )
                    .expect("untag count");

                // Get base list length
                let len_func = state
                    .module
                    .get_function("vp_list_len")
                    .ok_or_else(|| "vp_list_len not declared".to_string())?;
                let base_len = state
                    .ir_builder
                    .build_call(state.builder, len_func, &[base_ptr.into()], "base_len")
                    .expect("base_len")
                    .into_int_value();

                // Calculate new length
                let new_len = state
                    .builder
                    .build_int_mul(base_len, count_untagged, "new_len")
                    .expect("new_len");

                // Create result list with capacity
                let create_func = state
                    .module
                    .get_function("vp_list_create_with_capacity")
                    .ok_or_else(|| "vp_list_create_with_capacity not declared".to_string())?;
                let result_list = state
                    .ir_builder
                    .build_call(state.builder, create_func, &[new_len.into()], "result_list")
                    .expect("create result")
                    .into_pointer_value();

                // Extend result list count times
                let extend_func = state
                    .module
                    .get_function("vp_list_extend")
                    .ok_or_else(|| "vp_list_extend not declared".to_string())?;

                // Create loop
                let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
                let init_block = state.context.append_basic_block(func, "list_repeat_init");
                let cond_block = state.context.append_basic_block(func, "list_repeat_cond");
                let body_block = state.context.append_basic_block(func, "list_repeat_body");
                let step_block = state.context.append_basic_block(func, "list_repeat_step");
                let end_block = state.context.append_basic_block(func, "list_repeat_end");

                state.ir_builder.build_branch(state.builder, init_block);

                // Init: counter = 0
                state.builder.position_at_end(init_block);
                let counter = state
                    .builder
                    .build_alloca(state.context.i64_type(), "counter")
                    .expect("alloca");
                state
                    .builder
                    .build_store(counter, state.context.i64_type().const_int(0, false))
                    .expect("store");
                state.ir_builder.build_branch(state.builder, cond_block);

                // Condition: counter < count
                state.builder.position_at_end(cond_block);
                let counter_val = state
                    .builder
                    .build_load(state.context.i64_type(), counter, "counter_val")
                    .expect("load")
                    .into_int_value();
                let cond = state
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::ULT,
                        counter_val,
                        count_untagged,
                        "cond",
                    )
                    .expect("cond");
                state.ir_builder.build_cond_branch(state.builder, cond, body_block, end_block);

                // Body: extend
                state.builder.position_at_end(body_block);
                state
                    .ir_builder
                    .build_call(
                        state.builder,
                        extend_func,
                        &[result_list.into(), base_ptr.into()],
                        "extend",
                    )
                    .expect("extend");
                state.ir_builder.build_branch(state.builder, step_block);

                // Step: counter++
                state.builder.position_at_end(step_block);
                let next_counter = state
                    .builder
                    .build_int_add(
                        counter_val,
                        state.context.i64_type().const_int(1, false),
                        "next",
                    )
                    .expect("add");
                state.builder.build_store(counter, next_counter).expect("store");
                state.ir_builder.build_branch(state.builder, cond_block);

                // End
                state.builder.position_at_end(end_block);

                return Ok(result_list.into());
            }

            // Single-element list repetition (optimized path)
            if let Some(elem) = elems.first() {
                // Only generate the element expression and count, not the full list
                let count_val = generate_expr(state, right)?;
                let count_int = count_val.into_int_value();

                // Untag the count integer (tagged ints are shifted left by 1)
                let count_untagged = state
                    .builder
                    .build_right_shift(
                        count_int,
                        state.context.i64_type().const_int(1, false),
                        false,
                        "count_untagged",
                    )
                    .expect("failed to untag count");

                let (elem_val, func_name) = match elem {
                    Expr::Bool(true, _) => {
                        let val: inkwell::values::BasicMetadataValueEnum =
                            state.context.bool_type().const_int(1, false).into();
                        (val, "vp_bitvec_repeat") // Use bit vector for bool lists
                    }
                    Expr::Bool(false, _) => {
                        let val: inkwell::values::BasicMetadataValueEnum =
                            state.context.bool_type().const_int(0, false).into();
                        (val, "vp_bitvec_repeat") // Use bit vector for bool lists
                    }
                    Expr::Int(val, _) => {
                        // Tag the integer value (shift left by 1)
                        let tagged_val = (*val) << 1;
                        let val: inkwell::values::BasicMetadataValueEnum =
                            state.ir_builder.i64_const(tagged_val).into();
                        (val, "vp_list_repeat")
                    }
                    Expr::Float(val, _) => {
                        let f64_val = state.context.f64_type().const_float(*val as f64);
                        let i64_val = state
                            .builder
                            .build_float_to_signed_int(
                                f64_val,
                                state.context.i64_type(),
                                "float_to_int",
                            )
                            .map_err(|e| format!("Failed to convert float to int: {:?}", e))?;
                        (i64_val.into(), "vp_list_repeat")
                    }
                    _ => {
                        let val = generate_expr(state, elem)?;
                        if val.is_int_value() {
                            let int_val = val.into_int_value();
                            if int_val.get_type().get_bit_width() == 1 {
                                (int_val.into(), "vp_bitvec_repeat") // Use bit vector for bool lists
                            } else {
                                (int_val.into(), "vp_list_repeat")
                            }
                        } else {
                            return crate::codegen::codegen_error(
                                "List repeat requires integer or boolean elements".to_string(),
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
                        &[elem_val, count_untagged.into()],
                        "list_repeat",
                    )
                    .expect("list_repeat call");

                return Ok(result.into());
            }
        }

        // Handle string * int for string repetition
        let is_str_left = match left {
            Expr::Str(_, _) | Expr::FString(_, _) => true,
            _ => false,
        };
        let is_int_right = match right {
            Expr::Int(_, _) => true,
            _ => false,
        };

        if is_str_left && is_int_right {
            // Generate the string
            let str_val = generate_expr(state, left)?;
            // Generate the count
            let count_val = generate_expr(state, right)?;
            let count_int = if count_val.is_int_value() {
                count_val.into_int_value()
            } else {
                return crate::codegen::codegen_error(
                    "String repetition count must be an integer".to_string(),
                );
            };

            // Call vp_str_repeat
            let str_repeat_func = state
                .module
                .get_function("vp_str_repeat")
                .ok_or_else(|| "vp_str_repeat not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    str_repeat_func,
                    &[str_val.into(), count_int.into()],
                    "str_repeat",
                )
                .expect("str_repeat call");

            return Ok(result.into());
        }

        // Also handle int * str
        let is_int_left = match left {
            Expr::Int(_, _) => true,
            _ => false,
        };
        let is_str_right = match right {
            Expr::Str(_, _) | Expr::FString(_, _) => true,
            _ => false,
        };

        if is_int_left && is_str_right {
            // Generate the count
            let count_val = generate_expr(state, left)?;
            let count_int = if count_val.is_int_value() {
                count_val.into_int_value()
            } else {
                return crate::codegen::codegen_error(
                    "String repetition count must be an integer".to_string(),
                );
            };
            // Generate the string
            let str_val = generate_expr(state, right)?;

            // Call vp_str_repeat
            let str_repeat_func = state
                .module
                .get_function("vp_str_repeat")
                .ok_or_else(|| "vp_str_repeat not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    str_repeat_func,
                    &[count_int.into(), str_val.into()],
                    "str_repeat",
                )
                .expect("str_repeat call");

            return Ok(result.into());
        }
    }

    // Check for tagged int operations BEFORE generating operands
    // Tagged ints are represented as i64 values (tagged with LSB)
    let left_type = crate::codegen::expressions::core::infer_type_with_state(state, left);
    let right_type = crate::codegen::expressions::core::infer_type_with_state(state, right);

    // Check if either operand is Type::F64 (float operations take precedence)
    let is_float = left_type == crate::ast::Type::F64 || right_type == crate::ast::Type::F64;

    // Check if either operand value is actually a float (even if type inference says Infer)
    let lhs_val_check = generate_expr(state, left)?;
    let rhs_val_check = generate_expr(state, right)?;
    let is_float_val = lhs_val_check.is_float_value() || rhs_val_check.is_float_value();

    // Check if either operand is Type::Int (from type annotation or inference)
    let is_tagged_int = left_type == crate::ast::Type::Int || right_type == crate::ast::Type::Int;

    // Also check if either operand is a call to int() which returns Type::Int
    let is_left_int_call = matches!(left, Expr::Call { func, .. } if matches!(func.as_ref(), Expr::Ident(name, _) if name == "int"));
    let is_right_int_call = matches!(right, Expr::Call { func, .. } if matches!(func.as_ref(), Expr::Ident(name, _) if name == "int"));

    // Check if both operands are i64 values (tagged ints) when type inference returns Infer
    // This handles cases like `a = 5; a * a` where the type is not explicitly annotated
    // Only apply this when both types are Infer or Int (not I64 or other explicit types)
    let both_are_i64_tagged = (left_type == Type::Infer || left_type == Type::Int)
        && (right_type == Type::Infer || right_type == Type::Int)
        && lhs_val_check.is_int_value()
        && rhs_val_check.is_int_value()
        && lhs_val_check.into_int_value().get_type().get_bit_width() == 64
        && rhs_val_check.into_int_value().get_type().get_bit_width() == 64;

    // Float operations have higher priority than tagged int
    if is_float || is_float_val {
        // Generate operands for float operations
        return arithmetic::generate_float_binop(state, lhs_val_check, rhs_val_check, op);
    }

    if is_tagged_int || is_left_int_call || is_right_int_call || both_are_i64_tagged {
        // Generate operands for tagged int operations
        return arithmetic::generate_tagged_int_binop(state, lhs_val_check, rhs_val_check, op);
    }

    // Generate both operands for other operations (already generated above)
    let lhs_val = lhs_val_check;
    let rhs_val = rhs_val_check;

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
            Expr::List { elements, .. } => {
                elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
            }
            _ => false,
        };
        let is_bool_list_right = match right {
            Expr::Ident(name, _) => state.is_bool_list(name),
            Expr::List { elements, .. } => {
                elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
            }
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

    // Handle string comparison with == and !=
    if lhs_val.is_pointer_value() && rhs_val.is_pointer_value() {
        // Check if this is string equality comparison
        if matches!(op, BinOp::Eq | BinOp::NotEq) {
            let str_equals_func = state
                .module
                .get_function("vp_str_equals")
                .ok_or_else(|| "vp_str_equals not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    str_equals_func,
                    &[lhs_val.into(), rhs_val.into()],
                    "str_cmp",
                )
                .ok_or_else(|| "vp_str_equals call failed".to_string())?;

            // For !=, negate the result
            if matches!(op, BinOp::NotEq) {
                let not_result = state
                    .builder
                    .build_not(result.into_int_value(), "str_neq")
                    .map_err(|e| format!("Failed to negate: {:?}", e))?;
                return Ok(not_result.into());
            }

            return Ok(result.into());
        }

        // For other comparison operators (<, >, <=, >=), use string comparison
        if matches!(op, BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq) {
            let str_compare_func = state
                .module
                .get_function("vp_str_compare")
                .ok_or_else(|| "vp_str_compare not declared".to_string())?;

            let cmp_result = state
                .ir_builder
                .build_call(
                    state.builder,
                    str_compare_func,
                    &[lhs_val.into(), rhs_val.into()],
                    "str_compare",
                )
                .ok_or_else(|| "vp_str_compare call failed".to_string())?;

            let cmp_val = cmp_result.into_int_value();
            let zero = state.context.i64_type().const_zero();

            match op {
                BinOp::Lt => {
                    let result = state
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLT, cmp_val, zero, "str_lt")
                        .map_err(|e| format!("str_lt: {:?}", e))?;
                    return Ok(result.into());
                }
                BinOp::Gt => {
                    let result = state
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGT, cmp_val, zero, "str_gt")
                        .map_err(|e| format!("str_gt: {:?}", e))?;
                    return Ok(result.into());
                }
                BinOp::LtEq => {
                    let result = state
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLE, cmp_val, zero, "str_lte")
                        .map_err(|e| format!("str_lte: {:?}", e))?;
                    return Ok(result.into());
                }
                BinOp::GtEq => {
                    let result = state
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGE, cmp_val, zero, "str_gte")
                        .map_err(|e| format!("str_gte: {:?}", e))?;
                    return Ok(result.into());
                }
                _ => {}
            }
        }

        // For identity comparison (is, is not) and other operators, use pointer comparison
        return comparison::generate_pointer_binop(
            state.builder,
            state.context,
            lhs_val,
            rhs_val,
            op,
        );
    }

    // Handle string-to-string comparison (both operands are pointers)
    if matches!(op, BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq) {
        if lhs_val.is_pointer_value() && rhs_val.is_pointer_value() {
            // Use string comparison functions
            if matches!(op, BinOp::Eq | BinOp::NotEq) {
                let str_equals_func = state
                    .module
                    .get_function("vp_str_equals")
                    .ok_or_else(|| "vp_str_equals not declared".to_string())?;

                let result = state
                    .ir_builder
                    .build_call(
                        state.builder,
                        str_equals_func,
                        &[lhs_val.into(), rhs_val.into()],
                        "str_cmp",
                    )
                    .ok_or_else(|| "vp_str_equals call failed".to_string())?;

                if matches!(op, BinOp::NotEq) {
                    let not_result = state
                        .builder
                        .build_not(result.into_int_value(), "str_neq")
                        .map_err(|e| format!("Failed to negate: {:?}", e))?;
                    return Ok(not_result.into());
                }
                return Ok(result.into());
            }

            // For <, >, <=, >=, use string comparison
            let str_compare_func = state
                .module
                .get_function("vp_str_compare")
                .ok_or_else(|| "vp_str_compare not declared".to_string())?;

            let cmp_result = state
                .ir_builder
                .build_call(
                    state.builder,
                    str_compare_func,
                    &[lhs_val.into(), rhs_val.into()],
                    "str_compare",
                )
                .ok_or_else(|| "vp_str_compare call failed".to_string())?;

            let cmp_val = cmp_result.into_int_value();
            let zero = state.context.i64_type().const_zero();

            match op {
                BinOp::Lt => {
                    let result = state
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLT, cmp_val, zero, "str_lt")
                        .map_err(|e| format!("str_lt: {:?}", e))?;
                    return Ok(result.into());
                }
                BinOp::Gt => {
                    let result = state
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGT, cmp_val, zero, "str_gt")
                        .map_err(|e| format!("str_gt: {:?}", e))?;
                    return Ok(result.into());
                }
                BinOp::LtEq => {
                    let result = state
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLE, cmp_val, zero, "str_lte")
                        .map_err(|e| format!("str_lte: {:?}", e))?;
                    return Ok(result.into());
                }
                BinOp::GtEq => {
                    let result = state
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGE, cmp_val, zero, "str_gte")
                        .map_err(|e| format!("str_gte: {:?}", e))?;
                    return Ok(result.into());
                }
                _ => {}
            }
        }
    }

    // Special handling for string indexing comparison: text[i] == "a"
    // text[i] returns an i64 (byte value), while "a" is a ViperString pointer
    if (lhs_val.is_pointer_value() && rhs_val.is_int_value())
        || (lhs_val.is_int_value() && rhs_val.is_pointer_value())
    {
        if matches!(
            op,
            BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq
        ) {
            let str_get_first = state
                .module
                .get_function("vp_str_get_first")
                .ok_or_else(|| "vp_str_get_first not declared".to_string())?;

            let (int_val, ptr_val, is_lhs_ptr) = if lhs_val.is_pointer_value() {
                (rhs_val.into_int_value(), lhs_val, true)
            } else {
                (lhs_val.into_int_value(), rhs_val, false)
            };

            // Call vp_str_get_first to get the integer byte value of the first char in the string
            let str_char_val = state
                .ir_builder
                .build_call(state.builder, str_get_first, &[ptr_val.into()], "str_char")
                .ok_or_else(|| "vp_str_get_first call failed".to_string())?
                .into_int_value();

            let (lhs, rhs) =
                if is_lhs_ptr { (str_char_val, int_val) } else { (int_val, str_char_val) };

            return arithmetic::generate_int_binop(state, lhs.into(), rhs.into(), op);
        }
    }

    // Reject pointer values in arithmetic operations (except for Add with strings, handled above)
    if lhs_val.is_pointer_value() || rhs_val.is_pointer_value() {
        return crate::codegen::codegen_error(
            "Binary operators cannot be applied to pointer values (lists)".to_string(),
        );
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
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
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
            UnaryOp::Not | UnaryOp::Invert => crate::codegen::codegen_error(format!(
                "Unary operator {:?} not supported for float types",
                op
            )),
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
        let op_type = crate::codegen::expressions::core::infer_type_with_state(state, operand);

        if op_type == crate::ast::Type::Int {
            return arithmetic::generate_tagged_int_unary(state, op, val);
        }

        // Handle pointer types (lists, etc.) for the 'not' operator
        if matches!(op, UnaryOp::Not) && val.is_pointer_value() {
            // For lists: not [] = True, not [1,2,3] = False
            // Check if list length is 0 (empty list = True)
            // ViperList struct: length is at offset 0 (i64)
            let ptr_val = val.into_pointer_value();
            let i64_ptr = state
                .builder
                .build_pointer_cast(ptr_val, state.context.ptr_type(inkwell::AddressSpace::default()), "list_as_i64_ptr")
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to cast list pointer: {:?}", e)))?;
            let length = state
                .builder
                .build_load(state.context.i64_type(), i64_ptr, "list_length")
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to load list length: {:?}", e)))?
                .into_int_value();
            let is_empty = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    length,
                    state.context.i64_type().const_zero(),
                    "is_empty",
                )
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to compare length: {:?}", e)))?;
            // not list = is_empty (empty = True, non-empty = False)
            return Ok(is_empty.into());
        }

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
/// This checks if the Result is Ok or Err, and returns early on Err with proper propagation
fn generate_unwrap<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    _op: &UnaryOp,
    operand: &Expr,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Generate the operand expression (should be a Result struct by value)
    let result_val = generate_expr(state, operand)?;

    // Result is now a struct value { is_ok: i8, value: ptr }
    let result_struct = if result_val.is_struct_value() {
        result_val.into_struct_value()
    } else {
        // If not a struct, it's already the unwrapped value (fallback)
        return Ok(result_val);
    };

    // Extract is_ok field (first field)
    let is_ok_val = state
        .builder
        .build_extract_value(result_struct, 0, "is_ok")
        .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?;
    let is_ok = is_ok_val.into_int_value();

    // Get the current function from the builder
    let func = state
        .builder
        .get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or("Failed to get current function")?;

    // Get the function's return type to construct proper Err on failure
    let func_return_type = func.get_type().get_return_type();

    // Check if function returns a Result type
    let returns_result = func_return_type.map_or(false, |t| t.is_struct_type());

    if !returns_result {
        // If function doesn't return Result, ? operator is invalid
        // For now, just panic as before
        let err_msg = state.context.const_string(b"? operator in non-Result function", true);
        let err_msg_global = state.module.add_global(err_msg.get_type(), None, "unwrap_err_msg");
        err_msg_global.set_initializer(&err_msg);
        let err_msg_ptr = state
            .builder
            .build_pointer_cast(
                err_msg_global.as_pointer_value(),
                state.context.ptr_type(inkwell::AddressSpace::default()),
                "err_msg_ptr",
            )
            .map_err(|e| format!("Failed to cast error message: {:?}", e))?;

        if let Some(panic_func) = state.module.get_function("viper_panic") {
            state
                .builder
                .build_call(panic_func, &[err_msg_ptr.into()], "panic")
                .map_err(|e| format!("Failed to call panic: {:?}", e))?;
        }
        state
            .builder
            .build_unreachable()
            .map_err(|e| format!("Failed to build unreachable: {:?}", e))?;

        // Return a dummy value (unreachable)
        return Ok(state.context.i64_type().const_zero().into());
    }

    // Create blocks for Ok and Err cases
    let ok_block = state.context.append_basic_block(func, "result_ok");
    let err_block = state.context.append_basic_block(func, "result_err");
    let continue_block = state.context.append_basic_block(func, "result_continue");

    // Convert is_ok from i8 to i1 for branch condition
    let is_ok_bool = state
        .builder
        .build_int_compare(
            inkwell::IntPredicate::NE,
            is_ok,
            state.context.i8_type().const_zero(),
            "is_ok_bool",
        )
        .map_err(|e| format!("Failed to build compare: {:?}", e))?;

    // Branch based on is_ok
    state
        .builder
        .build_conditional_branch(is_ok_bool, ok_block, err_block)
        .map_err(|e| format!("Failed to build conditional branch: {:?}", e))?;

    // Ok block: extract and return the value
    state.builder.position_at_end(ok_block);
    let ok_value = state
        .builder
        .build_extract_value(result_struct, 1, "value")
        .map_err(|e| format!("Failed to extract value: {:?}", e))?;
    state
        .builder
        .build_unconditional_branch(continue_block)
        .map_err(|e| format!("Failed to build branch: {:?}", e))?;
    let ok_block_end = state.builder.get_insert_block().unwrap();

    // Err block: propagate the error by returning early
    state.builder.position_at_end(err_block);

    // Extract the error from the original Result and construct a new Err to return
    let error_val = state
        .builder
        .build_extract_value(result_struct, 1, "error_value")
        .map_err(|e| format!("Failed to extract error: {:?}", e))?;

    // Construct a new Err result with the same error
    // Result struct type: { is_ok: i8, value: ptr }
    let i8_ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
    let result_struct_type =
        state.context.struct_type(&[state.context.i8_type().into(), i8_ptr_type.into()], false);

    // Allocate space for the Result struct
    let err_result_alloca = state
        .builder
        .build_alloca(result_struct_type, "propagated_err_result")
        .map_err(|e| format!("Failed to alloca: {:?}", e))?;

    // Store is_ok = 0
    let is_ok_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            err_result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_zero()],
            "is_ok_ptr",
        )
    }
    .map_err(|e| format!("Failed to get is_ok field: {:?}", e))?;
    state
        .builder
        .build_store(is_ok_ptr, state.context.i8_type().const_int(0, false))
        .map_err(|e| format!("Failed to store is_ok: {:?}", e))?;

    // Store the error pointer
    let error_ptr_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            err_result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_int(1, false)],
            "error_ptr_ptr",
        )
    }
    .map_err(|e| format!("Failed to get error_ptr field: {:?}", e))?;

    // Ensure error is a pointer
    let error_ptr = if error_val.is_pointer_value() {
        error_val.into_pointer_value()
    } else if error_val.is_int_value() {
        state
            .builder
            .build_int_to_ptr(error_val.into_int_value(), i8_ptr_type, "err_int_to_ptr")
            .map_err(|e| format!("Failed to bitcast int to ptr: {:?}", e))?
    } else {
        return crate::codegen::codegen_error(format!(
            "Unsupported error value type: {:?}",
            error_val.get_type()
        ));
    };

    state
        .builder
        .build_store(error_ptr_ptr, error_ptr)
        .map_err(|e| format!("Failed to store error: {:?}", e))?;

    // Load and return the Err result struct
    let err_result_val = state
        .builder
        .build_load(result_struct_type, err_result_alloca, "err_result_val")
        .map_err(|e| format!("Failed to load: {:?}", e))?;

    // Return the Err result
    state
        .builder
        .build_return(Some(&err_result_val))
        .map_err(|e| format!("Failed to build return: {:?}", e))?;

    // Err block is terminated, position at continue block
    state.builder.position_at_end(continue_block);

    // Continue block: phi node to merge values
    let phi = state
        .builder
        .build_phi(state.context.ptr_type(inkwell::AddressSpace::default()), "result_value")
        .map_err(|e| format!("Failed to build phi: {:?}", e))?;
    phi.add_incoming(&[(&ok_value, ok_block_end)]);

    Ok(phi.as_basic_value())
}
