use crate::ast::BinOp;
use crate::ast::UnaryOp;
use crate::codegen::state::CodeGenState;
use inkwell::types::StringRadix;
use inkwell::values::BasicMetadataValueEnum;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;

#[derive(Copy, Clone)]
enum TaggedIntArithmeticOp {
    Add,
    Sub,
    Mul,
}

fn tagged_int_runtime_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    func_name: &str,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    result_name: &str,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let args: [BasicMetadataValueEnum<'ctx>; 2] = [lhs.into(), rhs.into()];
    let result = state
        .ir_builder
        .build_call(state.builder, func, &args, result_name)
        .expect("tagged_int runtime call");

    Ok(result.into())
}

fn generate_tagged_int_arithmetic_fast_path<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: TaggedIntArithmeticOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let current_block = state
        .builder
        .get_insert_block()
        .ok_or_else(|| "No active insertion block for tagged int arithmetic".to_string())?;
    let func = current_block
        .get_parent()
        .ok_or_else(|| "Tagged int arithmetic must be emitted inside a function".to_string())?;

    let i64_type = state.context.i64_type();
    let i128_type = state.context.custom_width_int_type(128);
    let tag_mask = i64_type.const_int(1, false);
    let shift = i64_type.const_int(1, false);
    let min_small = i128_type
        .const_int_from_string("-4611686018427387904", StringRadix::Decimal)
        .ok_or_else(|| "Failed to build tagged min small constant".to_string())?;
    let max_small = i128_type
        .const_int_from_string("4611686018427387903", StringRadix::Decimal)
        .ok_or_else(|| "Failed to build tagged max small constant".to_string())?;

    let lhs_tagged = lhs.into_int_value();
    let rhs_tagged = rhs.into_int_value();

    let fast_block = state.context.append_basic_block(func, "tagged_fast");
    let fast_ok_block = state.context.append_basic_block(func, "tagged_fast_ok");
    let slow_block = state.context.append_basic_block(func, "tagged_slow");
    let merge_block = state.context.append_basic_block(func, "tagged_merge");

    let lhs_is_small = state
        .builder
        .build_int_compare(
            IntPredicate::EQ,
            state.builder.build_and(lhs_tagged, tag_mask, "lhs_tag").expect("lhs tag"),
            i64_type.const_zero(),
            "lhs_is_small",
        )
        .expect("lhs small");
    let rhs_is_small = state
        .builder
        .build_int_compare(
            IntPredicate::EQ,
            state.builder.build_and(rhs_tagged, tag_mask, "rhs_tag").expect("rhs tag"),
            i64_type.const_zero(),
            "rhs_is_small",
        )
        .expect("rhs small");
    let both_small =
        state.builder.build_and(lhs_is_small, rhs_is_small, "both_small").expect("both small");
    state
        .builder
        .build_conditional_branch(both_small, fast_block, slow_block)
        .expect("tagged fast check");

    state.builder.position_at_end(fast_block);
    let lhs_small =
        state.builder.build_right_shift(lhs_tagged, shift, true, "lhs_small").expect("lhs untag");
    let rhs_small =
        state.builder.build_right_shift(rhs_tagged, shift, true, "rhs_small").expect("rhs untag");
    let lhs_wide =
        state.builder.build_int_s_extend(lhs_small, i128_type, "lhs_wide").expect("lhs widen");
    let rhs_wide =
        state.builder.build_int_s_extend(rhs_small, i128_type, "rhs_wide").expect("rhs widen");

    let wide_result = match op {
        TaggedIntArithmeticOp::Add => state
            .builder
            .build_int_add(lhs_wide, rhs_wide, "tagged_add_wide")
            .expect("tagged add wide"),
        TaggedIntArithmeticOp::Sub => state
            .builder
            .build_int_sub(lhs_wide, rhs_wide, "tagged_sub_wide")
            .expect("tagged sub wide"),
        TaggedIntArithmeticOp::Mul => state
            .builder
            .build_int_mul(lhs_wide, rhs_wide, "tagged_mul_wide")
            .expect("tagged mul wide"),
    };

    let above_max = state
        .builder
        .build_int_compare(IntPredicate::SGT, wide_result, max_small, "tagged_above_max")
        .expect("above max");
    let below_min = state
        .builder
        .build_int_compare(IntPredicate::SLT, wide_result, min_small, "tagged_below_min")
        .expect("below min");
    let overflow =
        state.builder.build_or(above_max, below_min, "tagged_overflow").expect("overflow");
    state
        .builder
        .build_conditional_branch(overflow, slow_block, fast_ok_block)
        .expect("tagged overflow branch");

    state.builder.position_at_end(fast_ok_block);
    let small_result = state
        .builder
        .build_int_truncate(wide_result, i64_type, "tagged_small_result")
        .expect("truncate tagged result");
    let tagged_result =
        state.builder.build_left_shift(small_result, shift, "tagged_result").expect("retag result");
    state.builder.build_unconditional_branch(merge_block).expect("fast merge");
    let fast_end = state.builder.get_insert_block().expect("fast end block");

    state.builder.position_at_end(slow_block);
    let (func_name, result_name) = match op {
        TaggedIntArithmeticOp::Add => ("tagged_int_add", "tagged_add_slow"),
        TaggedIntArithmeticOp::Sub => ("tagged_int_sub", "tagged_sub_slow"),
        TaggedIntArithmeticOp::Mul => ("tagged_int_mul", "tagged_mul_slow"),
    };
    let slow_result =
        tagged_int_runtime_call(state, func_name, lhs, rhs, result_name)?.into_int_value();
    state.builder.build_unconditional_branch(merge_block).expect("slow merge");
    let slow_end = state.builder.get_insert_block().expect("slow end block");

    state.builder.position_at_end(merge_block);
    let phi = state.builder.build_phi(i64_type, "tagged_arith_result").expect("tagged arith phi");
    phi.add_incoming(&[(&tagged_result, fast_end), (&slow_result, slow_end)]);

    Ok(phi.as_basic_value())
}

/// Generate float binary operation
pub fn generate_float_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let builder = state.builder;
    let lhs = lhs.into_float_value();
    let rhs = rhs.into_float_value();

    match op {
        BinOp::Add => Ok(builder.build_float_add(lhs, rhs, "fadd").expect("fadd").into()),
        BinOp::Sub => Ok(builder.build_float_sub(lhs, rhs, "fsub").expect("fsub").into()),
        BinOp::Mul => Ok(builder.build_float_mul(lhs, rhs, "fmul").expect("fmul").into()),
        BinOp::Div => Ok(builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv").into()),
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
        BinOp::In | BinOp::NotIn => crate::codegen::codegen_error(
            "Membership operators not supported for float types".to_string(),
        ),
        _ => crate::codegen::codegen_error(format!("Unsupported float operator: {:?}", op)),
    }
}

/// Generate boolean binary operation
pub fn generate_bool_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let lhs = lhs.into_int_value();
    let rhs = rhs.into_int_value();

    match op {
        BinOp::Eq => Ok(state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "bool_eq").into()),
        BinOp::NotEq => {
            let eq = state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "bool_eq");
            Ok(state.builder.build_not(eq, "bool_neq").expect("not").into())
        }
        BinOp::And => Ok(state.builder.build_and(lhs, rhs, "bool_and").expect("and").into()),
        BinOp::Or => Ok(state.builder.build_or(lhs, rhs, "bool_or").expect("or").into()),
        _ => crate::codegen::codegen_error(format!("Unsupported boolean operator: {:?}", op)),
    }
}

/// Generate integer binary operation
pub fn generate_int_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let lhs = lhs.into_int_value();
    let rhs = rhs.into_int_value();

    match op {
        BinOp::Add => Ok(state.ir_builder.build_add(state.builder, lhs, rhs, "add").into()),
        BinOp::Sub => Ok(state.ir_builder.build_sub(state.builder, lhs, rhs, "sub").into()),
        BinOp::Mul => Ok(state.ir_builder.build_mul(state.builder, lhs, rhs, "mul").into()),
        BinOp::Div => Ok(state.ir_builder.build_div(state.builder, lhs, rhs, "div").into()),
        BinOp::Eq => Ok(state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "eq").into()),
        BinOp::NotEq => {
            let eq = state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "eq");
            Ok(state.builder.build_not(eq, "neq").expect("not").into())
        }
        BinOp::Lt => Ok(state.ir_builder.build_icmp_lt(state.builder, lhs, rhs, "lt").into()),
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
        BinOp::Is => Ok(state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "is_cmp").into()),
        BinOp::IsNot => {
            let eq = state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "isnot_cmp");
            Ok(state.builder.build_not(eq, "isnot_result").expect("not").into())
        }
        BinOp::Mod => Ok(state.builder.build_int_signed_rem(lhs, rhs, "mod").expect("mod").into()),
        BinOp::FloorDiv => {
            Ok(state.ir_builder.build_div(state.builder, lhs, rhs, "floordiv").into())
        }
        BinOp::BitAnd => Ok(state.builder.build_and(lhs, rhs, "bitand").expect("bitand").into()),
        BinOp::BitOr => Ok(state.builder.build_or(lhs, rhs, "bitor").expect("bitor").into()),
        BinOp::BitXor => Ok(state.builder.build_xor(lhs, rhs, "bitxor").expect("bitxor").into()),
        BinOp::LShift => {
            Ok(state.builder.build_left_shift(lhs, rhs, "lshift").expect("lshift").into())
        }
        BinOp::RShift => {
            Ok(state.builder.build_right_shift(lhs, rhs, false, "rshift").expect("rshift").into())
        }
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
        _ => crate::codegen::codegen_error(format!("Unsupported int operator: {:?}", op)),
    }
}

/// Generate tagged integer binary operation
pub fn generate_tagged_int_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Promote i64 values to tagged ints if needed
    let lhs_tagged = if lhs.is_int_value() {
        lhs
    } else if lhs.is_pointer_value() {
        // This is a BigInt pointer, need to tag it (pointer | 1)
        let ptr_val = lhs.into_pointer_value();
        let intptr = state
            .builder
            .build_ptr_to_int(ptr_val, state.context.i64_type(), "ptr_to_int")
            .expect("ptr_to_int");
        let tagged = state
            .builder
            .build_or(intptr, state.context.i64_type().const_int(1, false), "tagged_ptr")
            .expect("tag");
        tagged.into()
    } else {
        return crate::codegen::codegen_error(format!(
            "Cannot convert {:?} to tagged int",
            lhs.get_type()
        ));
    };

    let rhs_tagged = if rhs.is_int_value() {
        rhs
    } else if rhs.is_pointer_value() {
        // This is a BigInt pointer, need to tag it (pointer | 1)
        let ptr_val = rhs.into_pointer_value();
        let intptr = state
            .builder
            .build_ptr_to_int(ptr_val, state.context.i64_type(), "ptr_to_int")
            .expect("ptr_to_int");
        let tagged = state
            .builder
            .build_or(intptr, state.context.i64_type().const_int(1, false), "tagged_ptr")
            .expect("tag");
        tagged.into()
    } else {
        return crate::codegen::codegen_error(format!(
            "Cannot convert {:?} to tagged int",
            rhs.get_type()
        ));
    };

    match op {
        BinOp::Add => generate_tagged_int_arithmetic_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntArithmeticOp::Add,
        ),
        BinOp::Sub => generate_tagged_int_arithmetic_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntArithmeticOp::Sub,
        ),
        BinOp::Mul => generate_tagged_int_arithmetic_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntArithmeticOp::Mul,
        ),
        BinOp::Div | BinOp::FloorDiv => {
            crate::codegen::runtime::tagged_int::generate_tagged_int_div(
                state, lhs_tagged, rhs_tagged,
            )
        }
        BinOp::Mod => crate::codegen::runtime::tagged_int::generate_tagged_int_mod(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::Eq => crate::codegen::runtime::tagged_int::generate_tagged_int_eq(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::NotEq => {
            let eq = crate::codegen::runtime::tagged_int::generate_tagged_int_eq(
                state, lhs_tagged, rhs_tagged,
            )?;
            Ok(state.builder.build_not(eq.into_int_value(), "neq").expect("not").into())
        }
        BinOp::Lt => crate::codegen::runtime::tagged_int::generate_tagged_int_lt(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::Gt => crate::codegen::runtime::tagged_int::generate_tagged_int_gt(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::LtEq => {
            let gt = crate::codegen::runtime::tagged_int::generate_tagged_int_gt(
                state, lhs_tagged, rhs_tagged,
            )?;
            Ok(state.builder.build_not(gt.into_int_value(), "lte").expect("not").into())
        }
        BinOp::GtEq => {
            let lt = crate::codegen::runtime::tagged_int::generate_tagged_int_lt(
                state, lhs_tagged, rhs_tagged,
            )?;
            Ok(state.builder.build_not(lt.into_int_value(), "gte").expect("not").into())
        }
        BinOp::Pow => crate::codegen::runtime::tagged_int::generate_tagged_int_pow(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::LShift => crate::codegen::runtime::tagged_int::generate_tagged_int_lshift(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::RShift => crate::codegen::runtime::tagged_int::generate_tagged_int_rshift(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::BitAnd => crate::codegen::runtime::tagged_int::generate_tagged_int_bitand(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::BitOr => crate::codegen::runtime::tagged_int::generate_tagged_int_bitor(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::BitXor => crate::codegen::runtime::tagged_int::generate_tagged_int_bitxor(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::Is => crate::codegen::runtime::tagged_int::generate_tagged_int_eq(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::IsNot => {
            let eq = crate::codegen::runtime::tagged_int::generate_tagged_int_eq(
                state, lhs_tagged, rhs_tagged,
            )?;
            Ok(state.builder.build_not(eq.into_int_value(), "isnot").expect("not").into())
        }
        _ => crate::codegen::codegen_error(format!("Unsupported tagged int operator: {:?}", op)),
    }
}

/// Generate tagged integer unary operation
pub fn generate_tagged_int_unary<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    op: &UnaryOp,
    operand: BasicValueEnum<'ctx>,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    match op {
        UnaryOp::Neg => {
            crate::codegen::runtime::tagged_int::generate_tagged_int_neg(state, operand)
        }
        UnaryOp::Pos => Ok(operand),
        UnaryOp::Invert => {
            // For tagged int invert: untag, invert, retag
            // Note: This handles small integers. BigInt invert would require runtime call.
            let val = operand.into_int_value();
            let small_val = state
                .builder
                .build_right_shift(
                    val,
                    state.context.i64_type().const_int(1, false),
                    false,
                    "unshift",
                )
                .expect("unshift");
            let inverted = state
                .builder
                .build_xor(small_val, state.context.i64_type().const_all_ones(), "inverted")
                .expect("inverted");
            let tagged_inverted = state
                .builder
                .build_left_shift(
                    inverted,
                    state.context.i64_type().const_int(1, false),
                    "tagged_inverted",
                )
                .expect("tag");

            Ok(tagged_inverted.into())
        }
        _ => crate::codegen::codegen_error(format!(
            "Unsupported tagged int unary operator: {:?}",
            op
        )),
    }
}
