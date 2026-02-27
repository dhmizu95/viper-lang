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

    let is_dict = match right {
        Expr::Ident(name, _) => state.is_dict(name),
        Expr::Dict { .. } => true,
        _ => false,
    };

    let contains_func_name = if is_dict {
        "vp_dict_contains"
    } else {
        "vp_list_contains"
    };

    let func = state
        .module
        .get_function(contains_func_name)
        .ok_or_else(|| format!("{} not declared", contains_func_name))?;

    let result = state.ir_builder.build_call(
        state.builder,
        func,
        &[list_val.into(), value_val.into()],
        if matches!(op, BinOp::In) { "contains" } else { "not_in_contains" },
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
