use crate::ast::{BinOp, Expr};
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::{BasicValueEnum, PointerValue};

pub fn generate_bigint_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs_val = generate_expr(state, left)?;
    let rhs_val = generate_expr(state, right)?;

    let lhs_ptr = lhs_val.into_pointer_value();
    let rhs_ptr = rhs_val.into_pointer_value();

    let func_name = match op {
        BinOp::Add => "vp_bigint_add",
        BinOp::Sub => "vp_bigint_sub",
        BinOp::Mul => "vp_bigint_mul",
        BinOp::Div => "vp_bigint_div",
        BinOp::Mod => "vp_bigint_mod",
        BinOp::FloorDiv => "vp_bigint_floor_div",
        BinOp::And => "vp_bigint_and",
        BinOp::Or => "vp_bigint_or",
        BinOp::BitXor => "vp_bigint_xor",
        BinOp::LShift => "vp_bigint_shl",
        BinOp::RShift => "vp_bigint_shr",
        BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
            return generate_bigint_cmp(state, lhs_ptr, rhs_ptr, op);
        }
        _ => return Err(format!("BigInt: unsupported operator {:?}", op)),
    };

    let func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs_ptr.into(), rhs_ptr.into()], "bigint_binop")
        .expect("bigint_binop call");

    Ok(result.into())
}

pub fn generate_bigint_cmp<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs_ptr: PointerValue<'ctx>,
    rhs_ptr: PointerValue<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let cmp_func = state
        .module
        .get_function("vp_bigint_cmp")
        .ok_or_else(|| "vp_bigint_cmp not declared".to_string())?;

    let cmp_result = state
        .ir_builder
        .build_call(state.builder, cmp_func, &[lhs_ptr.into(), rhs_ptr.into()], "bigint_cmp")
        .expect("bigint_cmp call")
        .into_int_value();

    let zero = state.context.i32_type().const_int(0, false);

    match op {
        BinOp::Lt => {
            let cond = state.ir_builder.build_icmp_lt(state.builder, cmp_result, zero, "bigint_lt");
            Ok(cond.into())
        }
        BinOp::Gt => {
            let cond = state.ir_builder.build_icmp_gt(state.builder, cmp_result, zero, "bigint_gt");
            Ok(cond.into())
        }
        BinOp::LtEq => {
            let cond = state.ir_builder.build_icmp_le(state.builder, cmp_result, zero, "bigint_le");
            Ok(cond.into())
        }
        BinOp::GtEq => {
            let cond = state.ir_builder.build_icmp_ge(state.builder, cmp_result, zero, "bigint_ge");
            Ok(cond.into())
        }
        BinOp::Eq => {
            let cond = state.ir_builder.build_icmp_eq(state.builder, cmp_result, zero, "bigint_eq");
            Ok(cond.into())
        }
        BinOp::NotEq => {
            let cond = state.ir_builder.build_icmp_ne(state.builder, cmp_result, zero, "bigint_ne");
            Ok(cond.into())
        }
        _ => Err(format!("BigInt: invalid comparison operator {:?}", op)),
    }
}
