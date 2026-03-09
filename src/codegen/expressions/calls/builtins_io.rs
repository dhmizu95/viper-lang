//! I/O built-in functions

use crate::ast::Expr;
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate input() call
pub fn generate_input_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    let prompt_val = if args.is_empty() {
        state.ir_builder.string_const(state.module, "").into()
    } else {
        generate_expr(state, &args[0])?
    };

    let func = state
        .module
        .get_function("vp_input")
        .ok_or_else(|| "vp_input not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[prompt_val.into()], "input_result");
    Ok(result.unwrap())
}
