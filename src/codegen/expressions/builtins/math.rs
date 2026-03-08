//! Math builtin function code generation for Viper

use crate::ast::Expr;
use crate::codegen::state::CodeGenState;

use inkwell::values::BasicValueEnum;

use crate::codegen::expressions::calls::generate_bigint_abs;
use crate::codegen::expressions::core::generate_expr;

/// Generate math builtin function calls (abs only - others are in stdlib)
pub fn generate_math_builtin<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("{}() takes exactly 1 argument, got {}", name, args.len()));
    }

    // Use the same BigInt detection as operators
    let is_bigint = crate::codegen::expressions::operators::bigint::is_bigint_expr(&args[0], state);

    if is_bigint && name == "abs" {
        return generate_bigint_abs(state, args);
    }

    let arg_val = generate_expr(state, &args[0])?;

    // For integers, use vp_math_abs_i64
    if arg_val.is_int_value() {
        let int_val = arg_val.into_int_value();
        
        let abs_func = state
            .module
            .get_function("vp_math_abs_i64")
            .ok_or_else(|| "vp_math_abs_i64 not declared".to_string())?;
        
        let result = state
            .ir_builder
            .build_call(state.builder, abs_func, &[int_val.into()], "abs_result")
            .ok_or_else(|| "Failed to call vp_math_abs_i64".to_string())?;
        
        return Ok(result.into());
    }

    // Convert to float if necessary
    let arg_float = if arg_val.is_float_value() {
        arg_val.into_float_value()
    } else {
        let int_val = arg_val.into_int_value();
        state
            .builder
            .build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion")
    };

    let func_name = match name {
        "abs" => "vp_math_abs",
        _ => return Err(format!("Unknown math builtin: {}", name)),
    };

    let math_func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result =
        state.ir_builder.build_call(state.builder, math_func, &[arg_float.into()], "math_result");
    Ok(result.unwrap_or(state.ir_builder.f64_const(0.0).into()))
}
