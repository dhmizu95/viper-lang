//! Time module function code generation for Viper

use crate::ast::Expr;
use crate::codegen::state::CodeGenState;
use crate::codegen::expressions::core::generate_expr;
use inkwell::values::BasicValueEnum;

/// Generate time module function calls (time.time, time.sleep, etc.)
pub fn generate_time_func<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    match name {
        "time" => {
            let func = state.module.get_function("vp_time_time")
                .ok_or_else(|| "vp_time_time not declared".to_string())?;
            let result = state.ir_builder.build_call(state.builder, func, &[], "time_result");
            Ok(result.unwrap_or(state.context.f64_type().const_zero().into()))
        }
        "monotonic" => {
            let func = state.module.get_function("vp_time_monotonic")
                .ok_or_else(|| "vp_time_monotonic not declared".to_string())?;
            let result = state.ir_builder.build_call(state.builder, func, &[], "monotonic_result");
            Ok(result.unwrap_or(state.context.f64_type().const_zero().into()))
        }
        "perf_counter" => {
            let func = state.module.get_function("vp_time_perf_counter")
                .ok_or_else(|| "vp_time_perf_counter not declared".to_string())?;
            let result = state.ir_builder.build_call(state.builder, func, &[], "perf_result");
            Ok(result.unwrap_or(state.context.f64_type().const_zero().into()))
        }
        "sleep" => {
            if args.is_empty() {
                return crate::codegen::codegen_error("sleep() requires 1 argument");
            }
            let arg_val = generate_expr(state, &args[0])?;
            let arg_float = if arg_val.is_float_value() {
                arg_val.into_float_value()
            } else if arg_val.is_int_value() {
                state.builder.build_signed_int_to_float(
                    arg_val.into_int_value(),
                    state.context.f64_type(),
                    "sleep_arg_float"
                ).expect("int to float")
            } else {
                return crate::codegen::codegen_error("sleep() requires numeric argument");
            };

            let func = state.module.get_function("vp_time_sleep")
                .ok_or_else(|| "vp_time_sleep not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "");
            Ok(state.context.i64_type().const_zero().into()) // Return 0/None
        }
        _ => crate::codegen::codegen_error(format!("Unknown time function: {}", name)),
    }
}
