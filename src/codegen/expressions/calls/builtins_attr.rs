//! Attribute built-in functions

use crate::ast::Expr;
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate hasattr() call
pub fn generate_hasattr_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 2 {
        return crate::codegen::codegen_error("hasattr() requires exactly 2 arguments".to_string());
    }

    let obj_val = generate_expr(state, &args[0])?;
    let name_val = generate_expr(state, &args[1])?;

    let func = state
        .module
        .get_function("vp_hasattr")
        .ok_or_else(|| "vp_hasattr not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        func,
        &[obj_val.into(), name_val.into()],
        "hasattr_result",
    );
    Ok(result.unwrap())
}

/// Generate getattr() call - placeholder
pub fn generate_getattr_call<'ctx>(
    _state: &mut CodeGenState<'_, 'ctx>,
    _args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    crate::codegen::codegen_error("getattr() not yet implemented".to_string())
}

/// Generate setattr() call - placeholder
pub fn generate_setattr_call<'ctx>(
    _state: &mut CodeGenState<'_, 'ctx>,
    _args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    crate::codegen::codegen_error("setattr() not yet implemented".to_string())
}

/// Generate delattr() call - placeholder
pub fn generate_delattr_call<'ctx>(
    _state: &mut CodeGenState<'_, 'ctx>,
    _args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    crate::codegen::codegen_error("delattr() not yet implemented".to_string())
}
