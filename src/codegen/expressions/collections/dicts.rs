//! Dict creation for Viper

use inkwell::values::BasicValueEnum;

use crate::ast::Expr;
use crate::codegen::state::CodeGenState;

use crate::codegen::expressions::generate_expr;

/// Generate dict creation
pub fn generate_dict<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    pairs: &[(Expr, Expr)],
) -> Result<BasicValueEnum<'ctx>, String> {
    let dict_create_func = state
        .module
        .get_function("vp_dict_create")
        .ok_or_else(|| "vp_dict_create not declared".to_string())?;

    let dict_val =
        state.ir_builder.build_call(state.builder, dict_create_func, &[], "new_dict").unwrap();

    for (i, (key_expr, value_expr)) in pairs.iter().enumerate() {
        let key_val = generate_expr(state, key_expr)?;
        let value_val = generate_expr(state, value_expr)?;

        // Choose the appropriate dict_set function based on key and value types
        match (key_expr, value_expr) {
            (Expr::Str(_, _), Expr::Int(_, _)) => {
                // String key (already a Viper string) with i64 value
                let set_func = state
                    .module
                    .get_function("vp_dict_set_str_i64")
                    .ok_or_else(|| "vp_dict_set_str_i64 not declared".to_string())?;

                let _ = state.ir_builder.build_call(
                    state.builder,
                    set_func,
                    &[dict_val.into(), key_val.into(), value_val.into()],
                    &format!("dict_set_{}", i),
                );
            }
            (Expr::Str(_, _), Expr::Str(_, _)) => {
                // Both key and value are strings (already Viper strings)
                let set_func = state
                    .module
                    .get_function("vp_dict_set_str_str")
                    .ok_or_else(|| "vp_dict_set_str_str not declared".to_string())?;

                let _ = state.ir_builder.build_call(
                    state.builder,
                    set_func,
                    &[dict_val.into(), key_val.into(), value_val.into()],
                    &format!("dict_set_{}", i),
                );
            }
            (Expr::Str(_, _), _) => {
                // String key with other value types
                let set_func = state
                    .module
                    .get_function("vp_dict_set_str_i64")
                    .ok_or_else(|| "vp_dict_set_str_i64 not declared".to_string())?;

                let _ = state.ir_builder.build_call(
                    state.builder,
                    set_func,
                    &[dict_val.into(), key_val.into(), value_val.into()],
                    &format!("dict_set_{}", i),
                );
            }
            _ => {
                // Fallback for non-string keys (not yet supported)
                return Err("Dict keys must be strings".to_string());
            }
        }
    }

    Ok(dict_val)
}
