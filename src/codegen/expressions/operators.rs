//! Expression code generation for Viper

use super::*;
use inkwell::values::BasicMetadataValueEnum;

use crate::ast::{BinOp, Expr, Type, UnaryOp};
use crate::utils::mangle_function_name;

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarStorage, VarType};


/// Generate binary operation
pub fn generate_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    if matches!(op, BinOp::And | BinOp::Or) {
        return generate_logical_op(state, left, op, right);
    }

    if matches!(op, BinOp::In | BinOp::NotIn) {
        return generate_membership_op(state, left, op, right);
    }

    // Handle string concatenation with + operator
    if *op == BinOp::Add {
        let lhs_val = generate_expr(state, left)?;
        let rhs_val = generate_expr(state, right)?;

        // Check if both operands are strings (pointer types)
        if lhs_val.is_pointer_value() && rhs_val.is_pointer_value() {
            return generate_str_concat(state, lhs_val, rhs_val);
        }
    }

    // Handle list * int for list/array literals: [elem] * n
    if *op == BinOp::Mul {
        // Check for List or Array literal
        let elements = match left {
            Expr::List { elements, .. } => Some(elements),
            Expr::Array { elements, .. } => Some(elements),
            _ => None,
        };

        if let Some(elems) = elements {
            if let Some(elem) = elems.first() {
                let count_val = generate_expr(state, right)?;
                let count_int = count_val.into_int_value();

                let elem_val = generate_expr(state, elem)?;

                let elem_i64 = match elem {
                    Expr::Bool(true, _) => state.ir_builder.i64_const(1),
                    Expr::Bool(false, _) => state.ir_builder.i64_const(0),
                    _ => {
                        if elem_val.is_int_value() {
                            elem_val.into_int_value()
                        } else {
                            return Err(
                                "List repeat requires integer or boolean elements".to_string()
                            );
                        }
                    }
                };

                let list_repeat_func = state
                    .module
                    .get_function("vp_list_repeat")
                    .ok_or_else(|| "vp_list_repeat not declared".to_string())?;

                let result = state
                    .ir_builder
                    .build_call(
                        state.builder,
                        list_repeat_func,
                        &[elem_i64.into(), count_int.into()],
                        "list_repeat",
                    )
                    .expect("list_repeat call");

                return Ok(result.into());
            }
        }
    }

    // General binary operation handling
    let lhs_val = generate_expr(state, left)?;
    let rhs_val = generate_expr(state, right)?;

    // Handle comparison operators on pointers (identity comparison)
    if lhs_val.is_pointer_value() && rhs_val.is_pointer_value() {
        return generate_pointer_binop(state.builder, state.context, lhs_val, rhs_val, op);
    }

    // Reject pointer values in arithmetic operations (except for Add with strings, handled above)
    if lhs_val.is_pointer_value() || rhs_val.is_pointer_value() {
        return Err("Binary operators cannot be applied to pointer values (lists)".to_string());
    }

    // Handle boolean comparisons (both operands are i1)
    if lhs_val.is_int_value() && rhs_val.is_int_value() 
        && lhs_val.get_type().into_int_type().get_bit_width() == 1
        && rhs_val.get_type().into_int_type().get_bit_width() == 1 {
        return generate_bool_binop(state, lhs_val, rhs_val, op);
    }

    // Auto-convert int to float when one operand is float
    if lhs_val.is_float_value() && !rhs_val.is_float_value() {
        // Convert rhs (int) to float
        let rhs_int = rhs_val.into_int_value();
        let rhs_float = state
            .builder
            .build_signed_int_to_float(rhs_int, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion");
        return generate_float_binop(state, lhs_val, rhs_float.into(), op);
    } else if !lhs_val.is_float_value() && rhs_val.is_float_value() {
        // Convert lhs (int) to float
        let lhs_int = lhs_val.into_int_value();
        let lhs_float = state
            .builder
            .build_signed_int_to_float(lhs_int, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion");
        return generate_float_binop(state, lhs_float.into(), rhs_val, op);
    } else if lhs_val.is_float_value() {
        return generate_float_binop(state, lhs_val, rhs_val, op);
    } else {
        return generate_int_binop(state, lhs_val, rhs_val, op);
    }
}

/// Generate string concatenation
pub fn generate_str_concat<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let str_concat = state
        .module
        .get_function("vp_str_concat")
        .ok_or_else(|| "vp_str_concat not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(
            state.builder,
            str_concat,
            &[lhs.into(), rhs.into()],
            "str_concat",
        )
        .ok_or_else(|| "build call failed".to_string())?;

    Ok(result)
}

/// Generate logical AND/OR with short-circuiting
pub fn generate_logical_op<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Save the block where we evaluate lhs - this is where we'll branch from
    let lhs_block = state.builder.get_insert_block().unwrap();

    let lhs_val = generate_expr(state, left)?.into_int_value();

    let func = state
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();

    let is_and = *op == BinOp::And;

    // For AND: if lhs is true, evaluate rhs; if false, short-circuit to end
    // For OR: if lhs is false, evaluate rhs; if true, short-circuit to end
    let evaluate_rhs_block = state.context.append_basic_block(func, "logic_evaluate_rhs");
    let end_block = state.context.append_basic_block(func, "logic_end");

    // Branch based on lhs value
    state
        .builder
        .build_conditional_branch(
            lhs_val,
            if is_and {
                evaluate_rhs_block
            } else {
                end_block
            },
            if is_and {
                end_block
            } else {
                evaluate_rhs_block
            },
        )
        .expect("branch");

    // Evaluate rhs block
    state.builder.position_at_end(evaluate_rhs_block);
    let rhs_val = generate_expr(state, right)?.into_int_value();
    state
        .builder
        .build_unconditional_branch(end_block)
        .expect("branch");
    let rhs_block_end = state.builder.get_insert_block().unwrap();

    // Build phi in end block
    state.builder.position_at_end(end_block);
    let phi = state
        .builder
        .build_phi(state.context.bool_type(), "logic_result")
        .expect("phi");

    // For both AND and OR:
    // - From lhs_block: if short-circuit happens (lhs decides result), use lhs_val
    // - From evaluate_rhs_block: use rhs_val
    phi.add_incoming(&[(&lhs_val, lhs_block), (&rhs_val, rhs_block_end)]);

    Ok(phi.as_basic_value())
}

/// Generate membership IN/NOT IN operators
pub fn generate_membership_op<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let value_val = generate_expr(state, left)?;
    let list_val = generate_expr(state, right)?;

    let list_contains = state
        .module
        .get_function("vp_list_contains")
        .ok_or_else(|| "vp_list_contains not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        list_contains,
        &[list_val.into(), value_val.into()],
        if matches!(op, BinOp::In) {
            "list_contains"
        } else {
            "not_in_contains"
        },
    );
    let contains_val: BasicValueEnum = result.unwrap_or(state.ir_builder.i64_const(0).into());

    if matches!(op, BinOp::NotIn) {
        Ok(state
            .builder
            .build_not(contains_val.into_int_value(), "not_in_result")
            .expect("not")
            .into())
    } else {
        Ok(contains_val)
    }
}

/// Generate float binary operation
pub fn generate_float_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let builder = state.builder;
    let lhs = lhs.into_float_value();
    let rhs = rhs.into_float_value();

    match op {
        BinOp::Add => Ok(builder
            .build_float_add(lhs, rhs, "fadd")
            .expect("fadd")
            .into()),
        BinOp::Sub => Ok(builder
            .build_float_sub(lhs, rhs, "fsub")
            .expect("fsub")
            .into()),
        BinOp::Mul => Ok(builder
            .build_float_mul(lhs, rhs, "fmul")
            .expect("fmul")
            .into()),
        BinOp::Div => Ok(builder
            .build_float_div(lhs, rhs, "fdiv")
            .expect("fdiv")
            .into()),
        BinOp::Eq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "feq")
            .expect("feq")
            .into()),
        BinOp::NotEq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::ONE, lhs, rhs, "fne")
            .expect("fne")
            .into()),
        BinOp::Lt => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OLT, lhs, rhs, "flt")
            .expect("flt")
            .into()),
        BinOp::Gt => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OGT, lhs, rhs, "fgt")
            .expect("fgt")
            .into()),
        BinOp::LtEq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OLE, lhs, rhs, "fle")
            .expect("fle")
            .into()),
        BinOp::GtEq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OGE, lhs, rhs, "fge")
            .expect("fge")
            .into()),
        BinOp::Is => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_is")
            .expect("f_is")
            .into()),
        BinOp::IsNot => {
            let eq = builder
                .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_isnot")
                .expect("f_isnot");
            Ok(builder.build_not(eq, "f_isnot_result").expect("not").into())
        }
        BinOp::FloorDiv => {
            let div = builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv");
            Ok(div.into())
        }
        BinOp::Pow => {
            // Call vp_pow(base, exponent)
            let pow_func = state
                .module
                .get_function("vp_pow")
                .ok_or_else(|| "vp_pow not declared".to_string())?;
            
            let result = state
                .ir_builder
                .build_call(state.builder, pow_func, &[lhs.into(), rhs.into()], "pow")
                .ok_or_else(|| "build call failed".to_string())?;
            
            Ok(result)
        }
        BinOp::In | BinOp::NotIn => {
            Err("Membership operators not supported for float types".to_string())
        }
        _ => Err(format!("Unsupported float operator: {:?}", op)),
    }
}

/// Generate boolean binary operation
pub fn generate_bool_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs = lhs.into_int_value();
    let rhs = rhs.into_int_value();

    match op {
        BinOp::Eq => Ok(state
            .ir_builder
            .build_icmp_eq(state.builder, lhs, rhs, "bool_eq")
            .into()),
        BinOp::NotEq => {
            let eq = state
                .ir_builder
                .build_icmp_eq(state.builder, lhs, rhs, "bool_eq");
            Ok(state.builder.build_not(eq, "bool_neq").expect("not").into())
        }
        BinOp::And => Ok(state
            .builder
            .build_and(lhs, rhs, "bool_and")
            .expect("and")
            .into()),
        BinOp::Or => Ok(state
            .builder
            .build_or(lhs, rhs, "bool_or")
            .expect("or")
            .into()),
        _ => Err(format!("Unsupported boolean operator: {:?}", op)),
    }
}

/// Generate integer binary operation
pub fn generate_int_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs = lhs.into_int_value();
    let rhs = rhs.into_int_value();

    match op {
        BinOp::Add => Ok(state
            .ir_builder
            .build_add(state.builder, lhs, rhs, "add")
            .into()),
        BinOp::Sub => Ok(state
            .ir_builder
            .build_sub(state.builder, lhs, rhs, "sub")
            .into()),
        BinOp::Mul => Ok(state
            .ir_builder
            .build_mul(state.builder, lhs, rhs, "mul")
            .into()),
        BinOp::Div => Ok(state
            .ir_builder
            .build_div(state.builder, lhs, rhs, "div")
            .into()),
        BinOp::Eq => Ok(state
            .ir_builder
            .build_icmp_eq(state.builder, lhs, rhs, "eq")
            .into()),
        BinOp::NotEq => {
            let eq = state
                .ir_builder
                .build_icmp_eq(state.builder, lhs, rhs, "eq");
            Ok(state.builder.build_not(eq, "neq").expect("not").into())
        }
        BinOp::Lt => Ok(state
            .ir_builder
            .build_icmp_lt(state.builder, lhs, rhs, "lt")
            .into()),
        BinOp::Gt => Ok(state
            .builder
            .build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "gt")
            .expect("gt")
            .into()),
        BinOp::LtEq => Ok(state
            .builder
            .build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "lte")
            .expect("lte")
            .into()),
        BinOp::GtEq => Ok(state
            .builder
            .build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "gte")
            .expect("gte")
            .into()),
        BinOp::Is => Ok(state
            .ir_builder
            .build_icmp_eq(state.builder, lhs, rhs, "is_cmp")
            .into()),
        BinOp::IsNot => {
            let eq = state
                .ir_builder
                .build_icmp_eq(state.builder, lhs, rhs, "isnot_cmp");
            Ok(state
                .builder
                .build_not(eq, "isnot_result")
                .expect("not")
                .into())
        }
        BinOp::Mod => Ok(state
            .builder
            .build_int_signed_rem(lhs, rhs, "mod")
            .expect("mod")
            .into()),
        BinOp::FloorDiv => Ok(state
            .ir_builder
            .build_div(state.builder, lhs, rhs, "floordiv")
            .into()),
        BinOp::BitAnd => Ok(state
            .builder
            .build_and(lhs, rhs, "bitand")
            .expect("bitand")
            .into()),
        BinOp::BitOr => Ok(state
            .builder
            .build_or(lhs, rhs, "bitor")
            .expect("bitor")
            .into()),
        BinOp::BitXor => Ok(state
            .builder
            .build_xor(lhs, rhs, "bitxor")
            .expect("bitxor")
            .into()),
        BinOp::LShift => Ok(state
            .builder
            .build_left_shift(lhs, rhs, "lshift")
            .expect("lshift")
            .into()),
        BinOp::RShift => Ok(state
            .builder
            .build_right_shift(lhs, rhs, false, "rshift")
            .expect("rshift")
            .into()),
        BinOp::Pow => {
            // Call vp_pow_i64(base, exponent)
            let pow_func = state
                .module
                .get_function("vp_pow_i64")
                .ok_or_else(|| "vp_pow_i64 not declared".to_string())?;
            
            let result = state
                .ir_builder
                .build_call(state.builder, pow_func, &[lhs.into(), rhs.into()], "pow")
                .ok_or_else(|| "build call failed".to_string())?;
            
            Ok(result.into())
        }
        _ => Err(format!("Unsupported int operator: {:?}", op)),
    }
}

/// Generate pointer binary operation (identity comparison)
pub fn generate_pointer_binop<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    context: &'ctx inkwell::context::Context,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs = lhs.into_pointer_value();
    let rhs = rhs.into_pointer_value();

    // Convert pointers to i64 for comparison (works on 64-bit systems)
    let intptr_type = context.i64_type();

    let lhs_int = builder
        .build_ptr_to_int(lhs, intptr_type, "lhs_int")
        .expect("Failed to convert lhs pointer to int");
    let rhs_int = builder
        .build_ptr_to_int(rhs, intptr_type, "rhs_int")
        .expect("Failed to convert rhs pointer to int");

    match op {
        BinOp::Eq => Ok(builder
            .build_int_compare(inkwell::IntPredicate::EQ, lhs_int, rhs_int, "ptr_eq")
            .expect("ptr_eq")
            .into()),
        BinOp::NotEq => Ok(builder
            .build_int_compare(inkwell::IntPredicate::NE, lhs_int, rhs_int, "ptr_neq")
            .expect("ptr_neq")
            .into()),
        BinOp::Is => Ok(builder
            .build_int_compare(inkwell::IntPredicate::EQ, lhs_int, rhs_int, "ptr_is")
            .expect("ptr_is")
            .into()),
        BinOp::IsNot => Ok(builder
            .build_int_compare(inkwell::IntPredicate::NE, lhs_int, rhs_int, "ptr_isnot")
            .expect("ptr_isnot")
            .into()),
        BinOp::Lt => Ok(builder
            .build_int_compare(inkwell::IntPredicate::ULT, lhs_int, rhs_int, "ptr_lt")
            .expect("ptr_lt")
            .into()),
        BinOp::Gt => Ok(builder
            .build_int_compare(inkwell::IntPredicate::UGT, lhs_int, rhs_int, "ptr_gt")
            .expect("ptr_gt")
            .into()),
        BinOp::LtEq => Ok(builder
            .build_int_compare(inkwell::IntPredicate::ULE, lhs_int, rhs_int, "ptr_lte")
            .expect("ptr_lte")
            .into()),
        BinOp::GtEq => Ok(builder
            .build_int_compare(inkwell::IntPredicate::UGE, lhs_int, rhs_int, "ptr_gte")
            .expect("ptr_gte")
            .into()),
        _ => Err(format!("Unsupported pointer operator: {:?}", op)),
    }
}

/// Generate unary operation
pub fn generate_unary<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    op: &UnaryOp,
    operand: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let val = generate_expr(state, operand)?;

    if val.is_float_value() {
        let float_val = val.into_float_value();
        match op {
            UnaryOp::Neg => Ok(state
                .builder
                .build_float_neg(float_val, "fneg")
                .expect("fneg")
                .into()),
            UnaryOp::Pos => Ok(val),
            UnaryOp::Not | UnaryOp::Invert => Err(format!(
                "Unary operator {:?} not supported for float types",
                op
            )),
        }
    } else {
        let int_val = val.into_int_value();
        match op {
            UnaryOp::Neg => Ok(state
                .builder
                .build_int_neg(int_val, "neg")
                .expect("neg")
                .into()),
            UnaryOp::Not => Ok(state.builder.build_not(int_val, "not").expect("not").into()),
            UnaryOp::Pos => Ok(val),
            UnaryOp::Invert => Ok(state
                .builder
                .build_xor(int_val, state.context.i64_type().const_all_ones(), "invert")
                .expect("invert")
                .into()),
        }
    }
}

/// Generate ternary conditional expression
pub fn generate_conditional<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    then_expr: &Expr,
    else_expr: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();
    let cond_val = generate_expr(state, condition)?.into_int_value();

    let then_block = state.context.append_basic_block(func, "ternary_then");
    let else_block = state.context.append_basic_block(func, "ternary_else");
    let merge_block = state.context.append_basic_block(func, "ternary_end");

    let cond_i1 = if cond_val.get_type().get_bit_width() == 1 {
        cond_val
    } else {
        state
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                cond_val,
                state.context.i64_type().const_zero(),
                "ternary_cond",
            )
            .expect("ternary_cond")
    };

    state
        .ir_builder
        .build_cond_branch(state.builder, cond_i1, then_block, else_block);

    state.builder.position_at_end(then_block);
    let then_val = generate_expr(state, then_expr)?;
    let then_block_end = state.builder.get_insert_block().unwrap();
    state.ir_builder.build_branch(state.builder, merge_block);

    state.builder.position_at_end(else_block);
    let else_val = generate_expr(state, else_expr)?;
    let else_block_end = state.builder.get_insert_block().unwrap();
    state.ir_builder.build_branch(state.builder, merge_block);

    state.builder.position_at_end(merge_block);
    let phi = state
        .builder
        .build_phi(then_val.get_type(), "ternary_result")
        .expect("phi");
    phi.add_incoming(&[(&then_val, then_block_end), (&else_val, else_block_end)]);

    Ok(phi.as_basic_value())
}

