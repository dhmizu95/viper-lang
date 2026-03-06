//! Special built-in functions (concurrency, main)

use crate::ast::Expr;

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;

/// Generate call to __user_main (redirected from main())
pub fn generate_user_main_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if !args.is_empty() {
        return Err("main() takes no arguments".to_string());
    }

    let user_main_func = state
        .functions
        .get("__user_main")
        .copied()
        .ok_or_else(|| "__user_main function not found".to_string())?;

    let call_result = state
        .builder
        .build_call(user_main_func, &[], "user_main_call");

    match call_result {
        Ok(call_site) => {
            match call_site.try_as_basic_value() {
                inkwell::values::ValueKind::Basic(bv) => Ok(bv),
                _ => Ok(state.ir_builder.i64_const(0).into()),
            }
        }
        Err(e) => Err(format!("Call to __user_main failed: {:?}", e)),
    }
}
