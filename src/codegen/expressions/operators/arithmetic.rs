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

#[derive(Copy, Clone)]
enum TaggedIntComparisonOp {
    Eq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

#[derive(Copy, Clone)]
enum TaggedIntBitwiseOp {
    And,
    Or,
    Xor,
}

#[derive(Copy, Clone)]
enum TaggedIntShiftOp {
    Left,
    Right,
}

fn tagged_int_current_function<'ctx>(
    state: &CodeGenState<'_, 'ctx>,
) -> crate::codegen::Result<inkwell::values::FunctionValue<'ctx>> {
    let current_block = state
        .builder
        .get_insert_block()
        .ok_or_else(|| "No active insertion block for tagged int arithmetic".to_string())?;
    Ok(current_block
        .get_parent()
        .ok_or_else(|| "Tagged int arithmetic must be emitted inside a function".to_string())?)
}

fn tagged_i64_type<'ctx>(state: &CodeGenState<'_, 'ctx>) -> inkwell::types::IntType<'ctx> {
    state.context.i64_type()
}

fn tagged_i128_type<'ctx>(state: &CodeGenState<'_, 'ctx>) -> inkwell::types::IntType<'ctx> {
    state.context.custom_width_int_type(128)
}

fn tagged_shift_const<'ctx>(state: &CodeGenState<'_, 'ctx>) -> inkwell::values::IntValue<'ctx> {
    tagged_i64_type(state).const_int(1, false)
}

fn tagged_small_mask<'ctx>(state: &CodeGenState<'_, 'ctx>) -> inkwell::values::IntValue<'ctx> {
    tagged_i64_type(state).const_int(1, false)
}

fn tagged_min_small<'ctx>(
    state: &CodeGenState<'_, 'ctx>,
) -> crate::codegen::Result<inkwell::values::IntValue<'ctx>> {
    Ok(tagged_i128_type(state)
        .const_int_from_string("-4611686018427387904", StringRadix::Decimal)
        .ok_or_else(|| "Failed to build tagged min small constant".to_string())?)
}

fn tagged_max_small<'ctx>(
    state: &CodeGenState<'_, 'ctx>,
) -> crate::codegen::Result<inkwell::values::IntValue<'ctx>> {
    Ok(tagged_i128_type(state)
        .const_int_from_string("4611686018427387903", StringRadix::Decimal)
        .ok_or_else(|| "Failed to build tagged max small constant".to_string())?)
}

fn tagged_untag_small<'ctx>(
    state: &CodeGenState<'_, 'ctx>,
    value: inkwell::values::IntValue<'ctx>,
    name: &str,
) -> inkwell::values::IntValue<'ctx> {
    state
        .builder
        .build_right_shift(value, tagged_shift_const(state), true, name)
        .expect("untag tagged int")
}

fn tagged_retag_small<'ctx>(
    state: &CodeGenState<'_, 'ctx>,
    value: inkwell::values::IntValue<'ctx>,
    name: &str,
) -> inkwell::values::IntValue<'ctx> {
    state
        .builder
        .build_left_shift(value, tagged_shift_const(state), name)
        .expect("retag tagged int")
}

fn tagged_both_small<'ctx>(
    state: &CodeGenState<'_, 'ctx>,
    lhs_tagged: inkwell::values::IntValue<'ctx>,
    rhs_tagged: inkwell::values::IntValue<'ctx>,
) -> inkwell::values::IntValue<'ctx> {
    let i64_type = tagged_i64_type(state);
    let lhs_is_small = state
        .builder
        .build_int_compare(
            IntPredicate::EQ,
            state
                .builder
                .build_and(lhs_tagged, tagged_small_mask(state), "lhs_tag")
                .expect("lhs tag"),
            i64_type.const_zero(),
            "lhs_is_small",
        )
        .expect("lhs small");
    let rhs_is_small = state
        .builder
        .build_int_compare(
            IntPredicate::EQ,
            state
                .builder
                .build_and(rhs_tagged, tagged_small_mask(state), "rhs_tag")
                .expect("rhs tag"),
            i64_type.const_zero(),
            "rhs_is_small",
        )
        .expect("rhs small");
    state
        .builder
        .build_and(lhs_is_small, rhs_is_small, "both_small")
        .expect("both small")
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
    let func = tagged_int_current_function(state)?;
    let i64_type = tagged_i64_type(state);
    let i128_type = tagged_i128_type(state);
    let min_small = tagged_min_small(state)?;
    let max_small = tagged_max_small(state)?;

    let lhs_tagged = lhs.into_int_value();
    let rhs_tagged = rhs.into_int_value();

    let fast_block = state.context.append_basic_block(func, "tagged_fast");
    let fast_ok_block = state.context.append_basic_block(func, "tagged_fast_ok");
    let slow_block = state.context.append_basic_block(func, "tagged_slow");
    let merge_block = state.context.append_basic_block(func, "tagged_merge");

    let both_small = tagged_both_small(state, lhs_tagged, rhs_tagged);
    state
        .builder
        .build_conditional_branch(both_small, fast_block, slow_block)
        .expect("tagged fast check");

    state.builder.position_at_end(fast_block);
    let lhs_small = tagged_untag_small(state, lhs_tagged, "lhs_small");
    let rhs_small = tagged_untag_small(state, rhs_tagged, "rhs_small");
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
    let tagged_result = tagged_retag_small(state, small_result, "tagged_result");
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

fn generate_tagged_int_comparison_fast_path<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: TaggedIntComparisonOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let func = tagged_int_current_function(state)?;
    let lhs_tagged = lhs.into_int_value();
    let rhs_tagged = rhs.into_int_value();

    let fast_block = state.context.append_basic_block(func, "tagged_cmp_fast");
    let slow_block = state.context.append_basic_block(func, "tagged_cmp_slow");
    let merge_block = state.context.append_basic_block(func, "tagged_cmp_merge");

    let both_small = tagged_both_small(state, lhs_tagged, rhs_tagged);
    state
        .builder
        .build_conditional_branch(both_small, fast_block, slow_block)
        .expect("tagged cmp check");

    state.builder.position_at_end(fast_block);
    let lhs_small = tagged_untag_small(state, lhs_tagged, "lhs_cmp_small");
    let rhs_small = tagged_untag_small(state, rhs_tagged, "rhs_cmp_small");
    let fast_result = match op {
        TaggedIntComparisonOp::Eq => state
            .builder
            .build_int_compare(IntPredicate::EQ, lhs_small, rhs_small, "tagged_eq_fast")
            .expect("fast eq"),
        TaggedIntComparisonOp::Lt => state
            .builder
            .build_int_compare(IntPredicate::SLT, lhs_small, rhs_small, "tagged_lt_fast")
            .expect("fast lt"),
        TaggedIntComparisonOp::Gt => state
            .builder
            .build_int_compare(IntPredicate::SGT, lhs_small, rhs_small, "tagged_gt_fast")
            .expect("fast gt"),
        TaggedIntComparisonOp::LtEq => state
            .builder
            .build_int_compare(IntPredicate::SLE, lhs_small, rhs_small, "tagged_lte_fast")
            .expect("fast lte"),
        TaggedIntComparisonOp::GtEq => state
            .builder
            .build_int_compare(IntPredicate::SGE, lhs_small, rhs_small, "tagged_gte_fast")
            .expect("fast gte"),
    };
    state.builder.build_unconditional_branch(merge_block).expect("cmp fast merge");
    let fast_end = state.builder.get_insert_block().expect("cmp fast end");

    state.builder.position_at_end(slow_block);
    let slow_result = match op {
        TaggedIntComparisonOp::Eq => tagged_int_runtime_call(
            state,
            "tagged_int_eq",
            lhs,
            rhs,
            "tagged_eq_slow",
        )?
        .into_int_value(),
        TaggedIntComparisonOp::Lt => tagged_int_runtime_call(
            state,
            "tagged_int_lt",
            lhs,
            rhs,
            "tagged_lt_slow",
        )?
        .into_int_value(),
        TaggedIntComparisonOp::Gt => tagged_int_runtime_call(
            state,
            "tagged_int_gt",
            lhs,
            rhs,
            "tagged_gt_slow",
        )?
        .into_int_value(),
        TaggedIntComparisonOp::LtEq => {
            let gt = tagged_int_runtime_call(state, "tagged_int_gt", lhs, rhs, "tagged_gt_slow")?
                .into_int_value();
            state.builder.build_not(gt, "tagged_lte_slow").expect("slow lte")
        }
        TaggedIntComparisonOp::GtEq => {
            let lt = tagged_int_runtime_call(state, "tagged_int_lt", lhs, rhs, "tagged_lt_slow")?
                .into_int_value();
            state.builder.build_not(lt, "tagged_gte_slow").expect("slow gte")
        }
    };
    state.builder.build_unconditional_branch(merge_block).expect("cmp slow merge");
    let slow_end = state.builder.get_insert_block().expect("cmp slow end");

    state.builder.position_at_end(merge_block);
    let phi = state
        .builder
        .build_phi(state.context.bool_type(), "tagged_cmp_result")
        .expect("tagged cmp phi");
    phi.add_incoming(&[(&fast_result, fast_end), (&slow_result, slow_end)]);
    Ok(phi.as_basic_value())
}

fn generate_tagged_int_bitwise_fast_path<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: TaggedIntBitwiseOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let func = tagged_int_current_function(state)?;
    let lhs_tagged = lhs.into_int_value();
    let rhs_tagged = rhs.into_int_value();
    let i64_type = tagged_i64_type(state);

    let fast_block = state.context.append_basic_block(func, "tagged_bit_fast");
    let slow_block = state.context.append_basic_block(func, "tagged_bit_slow");
    let merge_block = state.context.append_basic_block(func, "tagged_bit_merge");

    let both_small = tagged_both_small(state, lhs_tagged, rhs_tagged);
    state
        .builder
        .build_conditional_branch(both_small, fast_block, slow_block)
        .expect("tagged bitwise check");

    state.builder.position_at_end(fast_block);
    let lhs_small = tagged_untag_small(state, lhs_tagged, "lhs_bit_small");
    let rhs_small = tagged_untag_small(state, rhs_tagged, "rhs_bit_small");
    let small_result = match op {
        TaggedIntBitwiseOp::And => state.builder.build_and(lhs_small, rhs_small, "tagged_and_fast").expect("and"),
        TaggedIntBitwiseOp::Or => state.builder.build_or(lhs_small, rhs_small, "tagged_or_fast").expect("or"),
        TaggedIntBitwiseOp::Xor => state.builder.build_xor(lhs_small, rhs_small, "tagged_xor_fast").expect("xor"),
    };
    let fast_result = tagged_retag_small(state, small_result, "tagged_bit_fast_result");
    state.builder.build_unconditional_branch(merge_block).expect("bit fast merge");
    let fast_end = state.builder.get_insert_block().expect("bit fast end");

    state.builder.position_at_end(slow_block);
    let (func_name, result_name) = match op {
        TaggedIntBitwiseOp::And => ("tagged_int_bitand", "tagged_and_slow"),
        TaggedIntBitwiseOp::Or => ("tagged_int_bitor", "tagged_or_slow"),
        TaggedIntBitwiseOp::Xor => ("tagged_int_bitxor", "tagged_xor_slow"),
    };
    let slow_result = tagged_int_runtime_call(state, func_name, lhs, rhs, result_name)?.into_int_value();
    state.builder.build_unconditional_branch(merge_block).expect("bit slow merge");
    let slow_end = state.builder.get_insert_block().expect("bit slow end");

    state.builder.position_at_end(merge_block);
    let phi = state.builder.build_phi(i64_type, "tagged_bit_result").expect("tagged bit phi");
    phi.add_incoming(&[(&fast_result, fast_end), (&slow_result, slow_end)]);
    Ok(phi.as_basic_value())
}

fn generate_tagged_int_shift_fast_path<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: TaggedIntShiftOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let func = tagged_int_current_function(state)?;
    let lhs_tagged = lhs.into_int_value();
    let rhs_tagged = rhs.into_int_value();
    let i64_type = tagged_i64_type(state);
    let i128_type = tagged_i128_type(state);
    let min_small = tagged_min_small(state)?;
    let max_small = tagged_max_small(state)?;

    let fast_block = state.context.append_basic_block(func, "tagged_shift_fast");
    let fast_ok_block = state.context.append_basic_block(func, "tagged_shift_ok");
    let slow_block = state.context.append_basic_block(func, "tagged_shift_slow");
    let merge_block = state.context.append_basic_block(func, "tagged_shift_merge");

    let both_small = tagged_both_small(state, lhs_tagged, rhs_tagged);
    state
        .builder
        .build_conditional_branch(both_small, fast_block, slow_block)
        .expect("tagged shift check");

    state.builder.position_at_end(fast_block);
    let lhs_small = tagged_untag_small(state, lhs_tagged, "lhs_shift_small");
    let rhs_small = tagged_untag_small(state, rhs_tagged, "rhs_shift_small");

    let rhs_non_negative = state
        .builder
        .build_int_compare(IntPredicate::SGE, rhs_small, i64_type.const_zero(), "shift_non_negative")
        .expect("shift non-negative");
    let shift_limit = match op {
        TaggedIntShiftOp::Left => i64_type.const_int(62, false),
        TaggedIntShiftOp::Right => i64_type.const_int(62, false),
    };
    let rhs_in_range = state
        .builder
        .build_int_compare(IntPredicate::SLE, rhs_small, shift_limit, "shift_in_range")
        .expect("shift in range");
    let shift_is_fast = state
        .builder
        .build_and(rhs_non_negative, rhs_in_range, "shift_is_fast")
        .expect("shift fast guard");

    let lhs_wide = state
        .builder
        .build_int_s_extend(lhs_small, i128_type, "lhs_shift_wide")
        .expect("lhs shift widen");
    let rhs_wide = state
        .builder
        .build_int_z_extend(rhs_small, i128_type, "rhs_shift_wide")
        .expect("rhs shift widen");

    let wide_result = match op {
        TaggedIntShiftOp::Left => state
            .builder
            .build_left_shift(lhs_wide, rhs_wide, "tagged_lshift_wide")
            .expect("wide lshift"),
        TaggedIntShiftOp::Right => state
            .builder
            .build_right_shift(lhs_wide, rhs_wide, true, "tagged_rshift_wide")
            .expect("wide rshift"),
    };

    let result_fits = match op {
        TaggedIntShiftOp::Left => {
            let above_max = state
                .builder
                .build_int_compare(IntPredicate::SGT, wide_result, max_small, "lshift_above_max")
                .expect("lshift above max");
            let below_min = state
                .builder
                .build_int_compare(IntPredicate::SLT, wide_result, min_small, "lshift_below_min")
                .expect("lshift below min");
            let overflow = state.builder.build_or(above_max, below_min, "lshift_overflow").expect("lshift overflow");
            state.builder.build_not(overflow, "lshift_fits").expect("lshift fits")
        }
        TaggedIntShiftOp::Right => state.context.bool_type().const_all_ones(),
    };
    let take_fast = state.builder.build_and(shift_is_fast, result_fits, "take_shift_fast").expect("take shift fast");
    state
        .builder
        .build_conditional_branch(take_fast, fast_ok_block, slow_block)
        .expect("shift fast branch");

    state.builder.position_at_end(fast_ok_block);
    let fast_value = match op {
        TaggedIntShiftOp::Left => state
            .builder
            .build_int_truncate(wide_result, i64_type, "lshift_small_result")
            .expect("truncate lshift"),
        TaggedIntShiftOp::Right => state
            .builder
            .build_int_truncate(wide_result, i64_type, "rshift_small_result")
            .expect("truncate rshift"),
    };
    let fast_result = tagged_retag_small(state, fast_value, "tagged_shift_fast_result");
    state.builder.build_unconditional_branch(merge_block).expect("shift fast merge");
    let fast_end = state.builder.get_insert_block().expect("shift fast end");

    state.builder.position_at_end(slow_block);
    let (func_name, result_name) = match op {
        TaggedIntShiftOp::Left => ("tagged_int_lshift", "tagged_lshift_slow"),
        TaggedIntShiftOp::Right => ("tagged_int_rshift", "tagged_rshift_slow"),
    };
    let slow_result = tagged_int_runtime_call(state, func_name, lhs, rhs, result_name)?.into_int_value();
    state.builder.build_unconditional_branch(merge_block).expect("shift slow merge");
    let slow_end = state.builder.get_insert_block().expect("shift slow end");

    state.builder.position_at_end(merge_block);
    let phi = state.builder.build_phi(i64_type, "tagged_shift_result").expect("tagged shift phi");
    phi.add_incoming(&[(&fast_result, fast_end), (&slow_result, slow_end)]);
    Ok(phi.as_basic_value())
}

fn generate_tagged_int_neg_fast_path<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    operand: BasicValueEnum<'ctx>,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let func = tagged_int_current_function(state)?;
    let value = operand.into_int_value();
    let i64_type = tagged_i64_type(state);
    let min_small = i64_type.const_int((-(1_i64 << 62)) as u64, true);

    let fast_block = state.context.append_basic_block(func, "tagged_neg_fast");
    let fast_ok_block = state.context.append_basic_block(func, "tagged_neg_ok");
    let slow_block = state.context.append_basic_block(func, "tagged_neg_slow");
    let merge_block = state.context.append_basic_block(func, "tagged_neg_merge");

    let is_small = state
        .builder
        .build_int_compare(
            IntPredicate::EQ,
            state
                .builder
                .build_and(value, tagged_small_mask(state), "neg_tag")
                .expect("neg tag"),
            i64_type.const_zero(),
            "neg_is_small",
        )
        .expect("neg small");
    state
        .builder
        .build_conditional_branch(is_small, fast_block, slow_block)
        .expect("neg check");

    state.builder.position_at_end(fast_block);
    let small_value = tagged_untag_small(state, value, "neg_small_value");
    let is_min = state
        .builder
        .build_int_compare(IntPredicate::EQ, small_value, min_small, "neg_is_min")
        .expect("neg min");
    state
        .builder
        .build_conditional_branch(is_min, slow_block, fast_ok_block)
        .expect("neg min branch");

    state.builder.position_at_end(fast_ok_block);
    let negated = state.builder.build_int_neg(small_value, "negated_small").expect("neg");
    let fast_result = tagged_retag_small(state, negated, "tagged_neg_fast_result");
    state.builder.build_unconditional_branch(merge_block).expect("neg fast merge");
    let fast_end = state.builder.get_insert_block().expect("neg fast end");

    state.builder.position_at_end(slow_block);
    let func = state
        .module
        .get_function("tagged_int_neg")
        .ok_or_else(|| "tagged_int_neg not declared".to_string())?;
    let slow_result = state
        .ir_builder
        .build_call(state.builder, func, &[operand.into()], "tagged_neg_slow")
        .expect("tagged neg runtime")
        .into_int_value();
    state.builder.build_unconditional_branch(merge_block).expect("neg slow merge");
    let slow_end = state.builder.get_insert_block().expect("neg slow end");

    state.builder.position_at_end(merge_block);
    let phi = state.builder.build_phi(i64_type, "tagged_neg_result").expect("tagged neg phi");
    phi.add_incoming(&[(&fast_result, fast_end), (&slow_result, slow_end)]);
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
    
    // Coerce lhs to float if it's an integer
    let lhs = if lhs.is_int_value() {
        let int_val = lhs.into_int_value();
        // Untag if it's a 64-bit integer (tagged int)
        let untagged = if int_val.get_type().get_bit_width() == 64 {
            builder.build_right_shift(int_val, state.context.i64_type().const_int(1, false), true, "lhs_untag").expect("lhs untag")
        } else {
            builder.build_int_s_extend(int_val, state.context.i64_type(), "lhs_sext").expect("lhs sext")
        };
        builder.build_signed_int_to_float(untagged, state.context.f64_type(), "lhs_f64").expect("lhs to float").into()
    } else {
        lhs.into_float_value()
    };

    // Coerce rhs to float if it's an integer
    let rhs = if rhs.is_int_value() {
        let int_val = rhs.into_int_value();
        let untagged = if int_val.get_type().get_bit_width() == 64 {
            builder.build_right_shift(int_val, state.context.i64_type().const_int(1, false), true, "rhs_untag").expect("rhs untag")
        } else {
            builder.build_int_s_extend(int_val, state.context.i64_type(), "rhs_sext").expect("rhs sext")
        };
        builder.build_signed_int_to_float(untagged, state.context.f64_type(), "rhs_f64").expect("rhs to float").into()
    } else {
        rhs.into_float_value()
    };

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
        BinOp::Eq => generate_tagged_int_comparison_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntComparisonOp::Eq,
        ),
        BinOp::NotEq => {
            let eq = generate_tagged_int_comparison_fast_path(
                state, lhs_tagged, rhs_tagged,
                TaggedIntComparisonOp::Eq,
            )?;
            Ok(state.builder.build_not(eq.into_int_value(), "neq").expect("not").into())
        }
        BinOp::Lt => generate_tagged_int_comparison_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntComparisonOp::Lt,
        ),
        BinOp::Gt => generate_tagged_int_comparison_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntComparisonOp::Gt,
        ),
        BinOp::LtEq => generate_tagged_int_comparison_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntComparisonOp::LtEq,
        ),
        BinOp::GtEq => generate_tagged_int_comparison_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntComparisonOp::GtEq,
        ),
        BinOp::Pow => crate::codegen::runtime::tagged_int::generate_tagged_int_pow(
            state, lhs_tagged, rhs_tagged,
        ),
        BinOp::LShift => generate_tagged_int_shift_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntShiftOp::Left,
        ),
        BinOp::RShift => generate_tagged_int_shift_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntShiftOp::Right,
        ),
        BinOp::BitAnd => generate_tagged_int_bitwise_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntBitwiseOp::And,
        ),
        BinOp::BitOr => generate_tagged_int_bitwise_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntBitwiseOp::Or,
        ),
        BinOp::BitXor => generate_tagged_int_bitwise_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntBitwiseOp::Xor,
        ),
        BinOp::Is => generate_tagged_int_comparison_fast_path(
            state,
            lhs_tagged,
            rhs_tagged,
            TaggedIntComparisonOp::Eq,
        ),
        BinOp::IsNot => {
            let eq = generate_tagged_int_comparison_fast_path(
                state, lhs_tagged, rhs_tagged,
                TaggedIntComparisonOp::Eq,
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
        UnaryOp::Neg => generate_tagged_int_neg_fast_path(state, operand),
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
