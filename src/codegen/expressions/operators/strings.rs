use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate string concatenation
pub fn generate_str_concat<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let str_concat = state
        .module
        .get_function("vp_str_concat")
        .ok_or_else(|| "vp_str_concat not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, str_concat, &[lhs.into(), rhs.into()], "str_concat")
        .ok_or_else(|| "build call failed".to_string())?;

    Ok(result)
}
