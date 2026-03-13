//! BigInt binary operations using GMP

use crate::ast::{BinOp, Expr, UnaryOp};
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Check if an expression is a BigInt expression
pub fn is_bigint_expr<'a, 'ctx>(_expr: &Expr, _state: &CodeGenState<'a, 'ctx>) -> bool {
    false // BigInts now use the tagged_int operations instead
}

/// Generate BigInt binary operation
/// 
/// BigInt values are represented as pointers to ViperBigInt structs.
/// All operations call GMP bridge functions in the runtime.
pub fn generate_bigint_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Auto-promote i64 to BigInt if needed
    let lhs_ptr = if lhs.is_int_value() {
        promote_to_bigint(state, lhs)?
    } else {
        lhs.into_pointer_value()
    };
    
    let rhs_ptr = if rhs.is_int_value() {
        promote_to_bigint(state, rhs)?
    } else {
        rhs.into_pointer_value()
    };

    // For comparison operations, we need to call comparison functions and return bool
    match op {
        BinOp::Eq => {
            let cmp_func = state
                .module
                .get_function("vp_bigint_eq")
                .ok_or_else(|| "vp_bigint_eq not declared".to_string())?;
            
            let result = state
                .ir_builder
                .build_call(state.builder, cmp_func, &[lhs_ptr.into(), rhs_ptr.into()], "bigint_eq")
                .expect("bigint_eq call");
            
            Ok(result.into())
        }
        BinOp::NotEq => {
            let cmp_func = state
                .module
                .get_function("vp_bigint_eq")
                .ok_or_else(|| "vp_bigint_eq not declared".to_string())?;
            
            let eq_result = state
                .ir_builder
                .build_call(state.builder, cmp_func, &[lhs_ptr.into(), rhs_ptr.into()], "bigint_eq")
                .expect("bigint_eq call");
            
            // Negate the result
            let not_result = state
                .builder
                .build_not(eq_result.into_int_value(), "bigint_neq")
                .expect("not");
            
            Ok(not_result.into())
        }
        BinOp::Lt => {
            let cmp_func = state
                .module
                .get_function("vp_bigint_lt")
                .ok_or_else(|| "vp_bigint_lt not declared".to_string())?;
            
            let result = state
                .ir_builder
                .build_call(state.builder, cmp_func, &[lhs_ptr.into(), rhs_ptr.into()], "bigint_lt")
                .expect("bigint_lt call");
            
            Ok(result.into())
        }
        BinOp::Gt => {
            let cmp_func = state
                .module
                .get_function("vp_bigint_gt")
                .ok_or_else(|| "vp_bigint_gt not declared".to_string())?;
            
            let result = state
                .ir_builder
                .build_call(state.builder, cmp_func, &[lhs_ptr.into(), rhs_ptr.into()], "bigint_gt")
                .expect("bigint_gt call");
            
            Ok(result.into())
        }
        BinOp::LtEq => {
            // a <= b  ==  !(a > b)
            let cmp_func = state
                .module
                .get_function("vp_bigint_gt")
                .ok_or_else(|| "vp_bigint_gt not declared".to_string())?;
            
            let gt_result = state
                .ir_builder
                .build_call(state.builder, cmp_func, &[lhs_ptr.into(), rhs_ptr.into()], "bigint_gt")
                .expect("bigint_gt call");
            
            let result = state
                .builder
                .build_not(gt_result.into_int_value(), "bigint_lte")
                .expect("not");
            
            Ok(result.into())
        }
        BinOp::GtEq => {
            // a >= b  ==  !(a < b)
            let cmp_func = state
                .module
                .get_function("vp_bigint_lt")
                .ok_or_else(|| "vp_bigint_lt not declared".to_string())?;
            
            let lt_result = state
                .ir_builder
                .build_call(state.builder, cmp_func, &[lhs_ptr.into(), rhs_ptr.into()], "bigint_lt")
                .expect("bigint_lt call");
            
            let result = state
                .builder
                .build_not(lt_result.into_int_value(), "bigint_gte")
                .expect("not");
            
            Ok(result.into())
        }
        // Arithmetic operations
        BinOp::Add => {
            let result_ptr = initialize_bigint_result(state)?;
            let add_func = state
                .module
                .get_function("vp_bigint_add")
                .ok_or_else(|| "vp_bigint_add not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    add_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_add_call",
                );
            
            Ok(result_ptr.into())
        }
        BinOp::Sub => {
            let result_ptr = initialize_bigint_result(state)?;
            let sub_func = state
                .module
                .get_function("vp_bigint_sub")
                .ok_or_else(|| "vp_bigint_sub not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    sub_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_sub_call",
                );
            
            Ok(result_ptr.into())
        }
        BinOp::Mul => {
            let result_ptr = initialize_bigint_result(state)?;
            let mul_func = state
                .module
                .get_function("vp_bigint_mul")
                .ok_or_else(|| "vp_bigint_mul not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    mul_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_mul_call",
                );
            
            Ok(result_ptr.into())
        }
        BinOp::Div | BinOp::FloorDiv => {
            // FloorDiv (//) and regular Div (/) are the same for BigInt
            let result_ptr = initialize_bigint_result(state)?;
            let div_func = state
                .module
                .get_function("vp_bigint_div")
                .ok_or_else(|| "vp_bigint_div not declared".to_string())?;

            state
                .ir_builder
                .build_call(
                    state.builder,
                    div_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_div_call",
                );

            Ok(result_ptr.into())
        }
        BinOp::Mod => {
            let result_ptr = initialize_bigint_result(state)?;
            let mod_func = state
                .module
                .get_function("vp_bigint_mod")
                .ok_or_else(|| "vp_bigint_mod not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    mod_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_mod_call",
                );
            
            Ok(result_ptr.into())
        }
        BinOp::BitAnd => {
            let result_ptr = initialize_bigint_result(state)?;
            let and_func = state
                .module
                .get_function("vp_bigint_and")
                .ok_or_else(|| "vp_bigint_and not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    and_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_and_call",
                );
            
            Ok(result_ptr.into())
        }
        BinOp::BitOr => {
            let result_ptr = initialize_bigint_result(state)?;
            let or_func = state
                .module
                .get_function("vp_bigint_or")
                .ok_or_else(|| "vp_bigint_or not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    or_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_or_call",
                );
            
            Ok(result_ptr.into())
        }
        BinOp::BitXor => {
            let result_ptr = initialize_bigint_result(state)?;
            let xor_func = state
                .module
                .get_function("vp_bigint_xor")
                .ok_or_else(|| "vp_bigint_xor not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    xor_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_xor_call",
                );
            
            Ok(result_ptr.into())
        }
        BinOp::LShift => {
            let result_ptr = initialize_bigint_result(state)?;
            let lshift_func = state
                .module
                .get_function("vp_bigint_lshift")
                .ok_or_else(|| "vp_bigint_lshift not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    lshift_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_lshift_call",
                );
            
            Ok(result_ptr.into())
        }
        BinOp::RShift => {
            let result_ptr = initialize_bigint_result(state)?;
            let rshift_func = state
                .module
                .get_function("vp_bigint_rshift")
                .ok_or_else(|| "vp_bigint_rshift not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    rshift_func,
                    &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
                    "bigint_rshift_call",
                );
            
            Ok(result_ptr.into())
        }
        _ => crate::codegen::codegen_error(format!("Unsupported BigInt operator: {:?}", op)),
    }
}

/// Generate BigInt unary operation
pub fn generate_bigint_unary<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    op: &UnaryOp,
    operand: BasicValueEnum<'ctx>,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let operand_ptr = operand.into_pointer_value();

    match op {
        UnaryOp::Pos => Ok(operand),
        UnaryOp::Neg => {
            let result_ptr = initialize_bigint_result(state)?;
            let neg_func = state
                .module
                .get_function("vp_bigint_neg")
                .ok_or_else(|| "vp_bigint_neg not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    neg_func,
                    &[result_ptr.into(), operand_ptr.into()],
                    "bigint_neg_call",
                );
            
            Ok(result_ptr.into())
        }
        UnaryOp::Invert => {
            let result_ptr = initialize_bigint_result(state)?;
            let invert_func = state
                .module
                .get_function("vp_bigint_invert")
                .ok_or_else(|| "vp_bigint_invert not declared".to_string())?;
            
            state
                .ir_builder
                .build_call(
                    state.builder,
                    invert_func,
                    &[result_ptr.into(), operand_ptr.into()],
                    "bigint_invert_call",
                );
            
            Ok(result_ptr.into())
        }
        UnaryOp::Not => {
            let is_zero_func = state
                .module
                .get_function("vp_bigint_is_zero")
                .ok_or_else(|| "vp_bigint_is_zero not declared".to_string())?;
            
            let result = state
                .ir_builder
                .build_call(state.builder, is_zero_func, &[operand_ptr.into()], "is_zero")
                .expect("is_zero call");
            
            Ok(result.into())
        }
        _ => crate::codegen::codegen_error(format!("Unsupported BigInt unary operator: {:?}", op)),
    }
}

/// Initialize a new BigInt result object for binary operations
/// Uses temp allocation (ref_count=0) since the result will be immediately assigned
fn initialize_bigint_result<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
) -> crate::codegen::Result<inkwell::values::PointerValue<'ctx>> {
    let from_i64_temp_func = state
        .module
        .get_function("vp_bigint_from_i64_temp")
        .ok_or_else(|| "vp_bigint_from_i64_temp not declared".to_string())?;

    let zero = state.ir_builder.i64_const(0);
    let result = state
        .ir_builder
        .build_call(state.builder, from_i64_temp_func, &[zero.into()], "bigint_res_tmp")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64_temp".to_string())?;

    Ok(result.into_pointer_value())
}

/// Promote an integer value to BigInt
fn promote_to_bigint<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    val: BasicValueEnum<'ctx>,
) -> crate::codegen::Result<inkwell::values::PointerValue<'ctx>> {
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
    
    let int_val = val.into_int_value();
    
    // Sign-extend if bit width is less than 64 (e.g. i1)
    let bit_width = int_val.get_type().get_bit_width();
    let i64_val = if bit_width < 64 {
        state.builder.build_int_s_extend(int_val, state.context.i64_type(), "i64_extend")
            .map_err(|e| format!("Failed to extend int: {:?}", e))?
    } else {
        int_val
    };

    let result = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[i64_val.into()], "bigint_promoted")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?;
    
    Ok(result.into_pointer_value())
}
