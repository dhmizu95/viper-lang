use crate::ast::{BinOp, Expr};
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate BigInt binary operation
pub fn generate_bigint_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    _left: &Expr,
    op: &BinOp,
    _right: &Expr,
    lhs_val: BasicValueEnum<'ctx>,
    rhs_val: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let i64_type = state.context.i64_type();

    // Convert operands to BigInt if needed
    let lhs_bigint = if lhs_val.is_pointer_value() {
        lhs_val
    } else if lhs_val.is_int_value() {
        // Convert i64 to BigInt using vp_bigint_from_i64
        let from_i64 = state
            .module
            .get_function("vp_bigint_from_i64")
            .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, from_i64, &[lhs_val.into()], "bigint_from_i64")
            .unwrap();
        result
    } else {
        return Err("BigInt operation requires BigInt or integer operands".to_string());
    };

    let rhs_bigint = if rhs_val.is_pointer_value() {
        rhs_val
    } else if rhs_val.is_int_value() {
        // Convert i64 to BigInt using vp_bigint_from_i64
        let from_i64 = state
            .module
            .get_function("vp_bigint_from_i64")
            .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, from_i64, &[rhs_val.into()], "bigint_from_i64")
            .unwrap();
        result
    } else {
        return Err("BigInt operation requires BigInt or integer operands".to_string());
    };

    // Get the appropriate BigInt operation function
    let func_name = match op {
        BinOp::Add => "vp_bigint_add",
        BinOp::Sub => "vp_bigint_sub",
        BinOp::Mul => "vp_bigint_mul",
        BinOp::Div => "vp_bigint_div",
        BinOp::Mod => "vp_bigint_mod",
        BinOp::Pow => "vp_bigint_pow",
        BinOp::Eq => {
            // vp_bigint_cmp returns i64, compare to 0
            let cmp_func = state
                .module
                .get_function("vp_bigint_cmp")
                .ok_or_else(|| "vp_bigint_cmp not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    cmp_func,
                    &[lhs_bigint.into(), rhs_bigint.into()],
                    "bigint_cmp",
                )
                .unwrap();

            // Compare result to 0
            let zero = i64_type.const_zero();
            let eq = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    result.into_int_value(),
                    zero,
                    "bigint_eq",
                )
                .expect("bigint_eq");

            return Ok(eq.into());
        }
        BinOp::NotEq => {
            let cmp_func = state
                .module
                .get_function("vp_bigint_cmp")
                .ok_or_else(|| "vp_bigint_cmp not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    cmp_func,
                    &[lhs_bigint.into(), rhs_bigint.into()],
                    "bigint_cmp",
                )
                .unwrap();

            let zero = i64_type.const_zero();
            let eq = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    result.into_int_value(),
                    zero,
                    "bigint_eq",
                )
                .expect("bigint_eq");

            let neq = state.builder.build_not(eq, "bigint_neq").expect("bigint_neq");
            return Ok(neq.into());
        }
        BinOp::Lt => {
            let cmp_func = state
                .module
                .get_function("vp_bigint_cmp")
                .ok_or_else(|| "vp_bigint_cmp not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    cmp_func,
                    &[lhs_bigint.into(), rhs_bigint.into()],
                    "bigint_cmp",
                )
                .unwrap();

            // result < 0
            let zero = i64_type.const_zero();
            let lt = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::SLT,
                    result.into_int_value(),
                    zero,
                    "bigint_lt",
                )
                .expect("bigint_lt");

            return Ok(lt.into());
        }
        BinOp::LtEq => {
            let cmp_func = state
                .module
                .get_function("vp_bigint_cmp")
                .ok_or_else(|| "vp_bigint_cmp not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    cmp_func,
                    &[lhs_bigint.into(), rhs_bigint.into()],
                    "bigint_cmp",
                )
                .unwrap();

            // result <= 0
            let zero = i64_type.const_zero();
            let le = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::SLE,
                    result.into_int_value(),
                    zero,
                    "bigint_le",
                )
                .expect("bigint_le");

            return Ok(le.into());
        }
        BinOp::Gt => {
            let cmp_func = state
                .module
                .get_function("vp_bigint_cmp")
                .ok_or_else(|| "vp_bigint_cmp not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    cmp_func,
                    &[lhs_bigint.into(), rhs_bigint.into()],
                    "bigint_cmp",
                )
                .unwrap();

            // result > 0
            let zero = i64_type.const_zero();
            let gt = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::SGT,
                    result.into_int_value(),
                    zero,
                    "bigint_gt",
                )
                .expect("bigint_gt");

            return Ok(gt.into());
        }
        BinOp::GtEq => {
            let cmp_func = state
                .module
                .get_function("vp_bigint_cmp")
                .ok_or_else(|| "vp_bigint_cmp not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(
                    state.builder,
                    cmp_func,
                    &[lhs_bigint.into(), rhs_bigint.into()],
                    "bigint_cmp",
                )
                .unwrap();

            // result >= 0
            let zero = i64_type.const_zero();
            let ge = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::SGE,
                    result.into_int_value(),
                    zero,
                    "bigint_ge",
                )
                .expect("bigint_ge");

            return Ok(ge.into());
        }
        _ => return Err(format!("Unsupported BigInt operator: {:?}", op)),
    };

    let func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs_bigint.into(), rhs_bigint.into()], "bigint_op")
        .unwrap();

    Ok(result)
}
