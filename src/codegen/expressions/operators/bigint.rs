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

    // Convert operands to BigInt pointers if needed
    let lhs_ptr = if lhs_val.is_pointer_value() {
        lhs_val.into_pointer_value()
    } else if lhs_val.is_int_value() {
        // Convert i64 to BigInt
        let from_i64_func = state
            .module
            .get_function("vp_bigint_from_i64")
            .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, from_i64_func, &[lhs_val.into()], "bigint_from_i64")
            .expect("bigint_from_i64 call");
        result.into_pointer_value()
    } else {
        return Err(format!("BigInt: invalid left operand type"));
    };

    let rhs_ptr = if rhs_val.is_pointer_value() {
        rhs_val.into_pointer_value()
    } else if rhs_val.is_int_value() {
        // Convert i64 to BigInt
        let from_i64_func = state
            .module
            .get_function("vp_bigint_from_i64")
            .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, from_i64_func, &[rhs_val.into()], "bigint_from_i64")
            .expect("bigint_from_i64 call");
        result.into_pointer_value()
    } else {
        return Err(format!("BigInt: invalid right operand type"));
    };

    let func_name = match op {
        BinOp::Add => "vp_bigint_add",
        BinOp::Sub => "vp_bigint_sub",
        BinOp::Mul => "vp_bigint_mul",
        BinOp::Div => "vp_bigint_div",
        BinOp::Mod => "vp_bigint_mod",
        BinOp::FloorDiv => "vp_bigint_floor_div",
        BinOp::BitAnd => "vp_bigint_and",
        BinOp::BitOr => "vp_bigint_or",
        BinOp::BitXor => "vp_bigint_xor",
        BinOp::LShift => {
            // Left shift: lhs is BigInt, rhs is i64 shift amount
            let rhs_i64 = if rhs_val.is_int_value() {
                rhs_val.into_int_value()
            } else if rhs_val.is_pointer_value() {
                // Convert BigInt to i64 for shift amount
                let to_i64_func = state
                    .module
                    .get_function("vp_bigint_to_i64")
                    .ok_or_else(|| "vp_bigint_to_i64 not declared".to_string())?;
                let result = state
                    .ir_builder
                    .build_call(state.builder, to_i64_func, &[rhs_val.into()], "bigint_to_i64")
                    .expect("bigint_to_i64 call");
                result.into_int_value()
            } else {
                return Err(format!("BigInt: invalid shift operand type"));
            };
            
            let shift_func = state
                .module
                .get_function("vp_bigint_shl")
                .ok_or_else(|| "vp_bigint_shl not declared".to_string())?;
            
            let result = state
                .ir_builder
                .build_call(state.builder, shift_func, &[lhs_ptr.into(), rhs_i64.into()], "bigint_shl")
                .expect("bigint_shl call");
            return Ok(result.into());
        }
        BinOp::RShift => {
            // Right shift: lhs is BigInt, rhs is i64 shift amount
            let rhs_i64 = if rhs_val.is_int_value() {
                rhs_val.into_int_value()
            } else if rhs_val.is_pointer_value() {
                // Convert BigInt to i64 for shift amount
                let to_i64_func = state
                    .module
                    .get_function("vp_bigint_to_i64")
                    .ok_or_else(|| "vp_bigint_to_i64 not declared".to_string())?;
                let result = state
                    .ir_builder
                    .build_call(state.builder, to_i64_func, &[rhs_val.into()], "bigint_to_i64")
                    .expect("bigint_to_i64 call");
                result.into_int_value()
            } else {
                return Err(format!("BigInt: invalid shift operand type"));
            };
            
            let shift_func = state
                .module
                .get_function("vp_bigint_shr")
                .ok_or_else(|| "vp_bigint_shr not declared".to_string())?;
            
            let result = state
                .ir_builder
                .build_call(state.builder, shift_func, &[lhs_ptr.into(), rhs_i64.into()], "bigint_shr")
                .expect("bigint_shr call");
            return Ok(result.into());
        }
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
