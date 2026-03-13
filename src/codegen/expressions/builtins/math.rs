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
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!("{}() takes exactly 1 argument, got {}", name, args.len()));
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
        _ => return crate::codegen::codegen_error(format!("Unknown math builtin: {}", name)),
    };

    let math_func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result =
        state.ir_builder.build_call(state.builder, math_func, &[arg_float.into()], "math_result");
    Ok(result.unwrap_or(state.ir_builder.f64_const(0.0).into()))
}

/// Generate math module function calls (math.sqrt, math.sin, etc.)
pub fn generate_math_float_func<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error(format!("{}() requires at least 1 argument", name));
    }

    // Get the first argument and convert to float
    let arg_val = generate_expr(state, &args[0])?;
    let arg_float = if arg_val.is_float_value() {
        arg_val.into_float_value()
    } else if arg_val.is_int_value() {
        state
            .builder
            .build_signed_int_to_float(arg_val.into_int_value(), state.context.f64_type(), "int_to_float")
            .expect("int to float conversion")
    } else {
        return crate::codegen::codegen_error(format!("{}() requires numeric argument", name));
    };

    // Handle special cases
    let result = match name {
        "sqrt" => {
            let func = state.module.get_function("vp_math_sqrt")
                .ok_or_else(|| "vp_math_sqrt not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "sqrt_result")
        }
        "ln" => {
            let func = state.module.get_function("vp_math_ln")
                .ok_or_else(|| "vp_math_ln not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "ln_result")
        }
        "log" | "log10" => {
            let func = state.module.get_function("vp_math_log10")
                .ok_or_else(|| "vp_math_log10 not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "log10_result")
        }
        "log2" => {
            let func = state.module.get_function("vp_math_log2")
                .ok_or_else(|| "vp_math_log2 not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "log2_result")
        }
        "exp" => {
            let func = state.module.get_function("vp_math_exp")
                .ok_or_else(|| "vp_math_exp not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "exp_result")
        }
        "sin" => {
            let func = state.module.get_function("vp_math_sin")
                .ok_or_else(|| "vp_math_sin not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "sin_result")
        }
        "cos" => {
            let func = state.module.get_function("vp_math_cos")
                .ok_or_else(|| "vp_math_cos not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "cos_result")
        }
        "tan" => {
            let func = state.module.get_function("vp_math_tan")
                .ok_or_else(|| "vp_math_tan not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "tan_result")
        }
        "floor" => {
            let func = state.module.get_function("vp_math_floor")
                .ok_or_else(|| "vp_math_floor not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "floor_result")
        }
        "ceil" => {
            let func = state.module.get_function("vp_math_ceil")
                .ok_or_else(|| "vp_math_ceil not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[arg_float.into()], "ceil_result")
        }
        _ => {
            return crate::codegen::codegen_error(format!("Unknown math function: {}", name));
        }
    };

    Ok(result.unwrap_or(state.ir_builder.f64_const(0.0).into()))
}

/// Generate math constant access (math.pi, math.e, etc.)
pub fn generate_math_constant<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let value = match name {
        "pi" => std::f64::consts::PI,
        "e" => std::f64::consts::E,
        "tau" => std::f64::consts::TAU,
        _ => return crate::codegen::codegen_error(format!("Unknown math constant: {}", name)),
    };
    Ok(state.ir_builder.f64_const(value).into())
}
