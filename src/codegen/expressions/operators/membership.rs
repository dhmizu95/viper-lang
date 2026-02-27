use crate::ast::{BinOp, Expr};
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

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
        if matches!(op, BinOp::In) { "list_contains" } else { "not_in_contains" },
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
