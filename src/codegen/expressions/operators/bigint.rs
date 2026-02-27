//! BigInt binary operations using GMP

use crate::ast::BinOp;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate BigInt binary operation
/// 
/// BigInt values are represented as pointers to ViperBigInt structs.
/// All operations call GMP bridge functions in the runtime.
pub fn generate_bigint_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs_ptr = lhs.into_pointer_value();
    let rhs_ptr = rhs.into_pointer_value();

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
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_add",
                )
                .expect("bigint_add call");
            
            // Load the result
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_add_result")
                .expect("load result");
            
            Ok(result.into())
        }
        BinOp::Sub => {
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_sub",
                )
                .expect("bigint_sub call");
            
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_sub_result")
                .expect("load result");
            
            Ok(result.into())
        }
        BinOp::Mul => {
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_mul",
                )
                .expect("bigint_mul call");
            
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_mul_result")
                .expect("load result");
            
            Ok(result.into())
        }
        BinOp::Div => {
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_div",
                )
                .expect("bigint_div call");
            
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_div_result")
                .expect("load result");
            
            Ok(result.into())
        }
        BinOp::Mod => {
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_mod",
                )
                .expect("bigint_mod call");
            
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_mod_result")
                .expect("load result");
            
            Ok(result.into())
        }
        BinOp::BitAnd => {
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_and",
                )
                .expect("bigint_and call");
            
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_and_result")
                .expect("load result");
            
            Ok(result.into())
        }
        BinOp::BitOr => {
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_or",
                )
                .expect("bigint_or call");
            
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_or_result")
                .expect("load result");
            
            Ok(result.into())
        }
        BinOp::BitXor => {
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_xor",
                )
                .expect("bigint_xor call");
            
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_xor_result")
                .expect("load result");
            
            Ok(result.into())
        }
        BinOp::LShift => {
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_lshift",
                )
                .expect("bigint_lshift call");
            
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_lshift_result")
                .expect("load result");
            
            Ok(result.into())
        }
        BinOp::RShift => {
            let result_ptr = state
                .builder
                .build_alloca(lhs_ptr.get_type(), "bigint_result")
                .map_err(|e| format!("Failed to allocate result: {:?}", e))?;
            
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
                    "bigint_rshift",
                )
                .expect("bigint_rshift call");
            
            let result = state
                .builder
                .build_load(lhs_ptr.get_type(), result_ptr, "bigint_rshift_result")
                .expect("load result");
            
            Ok(result.into())
        }
        _ => Err(format!("Unsupported BigInt operator: {:?}", op)),
    }
}
