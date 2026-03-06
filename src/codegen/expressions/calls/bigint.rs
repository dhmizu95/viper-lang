//! BigInt built-in functions

use super::*;

use crate::ast::Expr;

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;

/// Helper function to handle BigInt math function routing
pub fn generate_math_bigint_func<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    attr: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;

    let get_bigint = |val: BasicValueEnum<'ctx>| -> Result<BasicValueEnum<'ctx>, String> {
        if val.is_pointer_value() {
            Ok(val)
        } else if val.is_int_value() {
            let res = state.ir_builder.build_call(state.builder, from_i64_func, &[val.into()], "bigint_from_i64")
                .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?;
            Ok(res.into_pointer_value().into())
        } else {
            Err(format!("Cannot convert argument to BigInt for math.{}", attr))
        }
    };

    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();

    match attr {
        "gcd" | "lcm" | "comb" | "perm" => {
            if args.len() != 2 {
                return Err(format!("math.{} requires exactly 2 arguments", attr));
            }
            let val0 = generate_expr(state, &args[0])?;
            let val1 = generate_expr(state, &args[1])?;
            let ptr0 = get_bigint(val0)?;
            let ptr1 = get_bigint(val1)?;
            let func_name = format!("vp_bigint_{}", attr);
            let func = state.module.get_function(&func_name).ok_or_else(|| format!("{} not declared", func_name))?;
            state.ir_builder.build_call(state.builder, func, &[result_ptr.into(), ptr0.into(), ptr1.into()], &format!("{}_call", attr));
        },
        "isqrt" => {
            if args.len() != 1 {
                return Err(format!("math.{} requires exactly 1 argument", attr));
            }
            let val0 = generate_expr(state, &args[0])?;
            let ptr0 = get_bigint(val0)?;
            let func = state.module.get_function("vp_bigint_sqrt").ok_or_else(|| "vp_bigint_sqrt not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[result_ptr.into(), ptr0.into()], "isqrt_call");
        },
        "factorial" => {
            if args.len() != 1 {
                return Err(format!("math.{} requires exactly 1 argument", attr));
            }
            let val0 = generate_expr(state, &args[0])?;
            let ptr0 = get_bigint(val0)?;
            let func = state.module.get_function("vp_bigint_factorial").ok_or_else(|| "vp_bigint_factorial not declared".to_string())?;
            state.ir_builder.build_call(state.builder, func, &[result_ptr.into(), ptr0.into()], "factorial_call");
        },
        _ => return Err(format!("Unsupported math function for BigInt: {}", attr)),
    }

    Ok(result_ptr.into())
}

/// Generate abs_bigint() call - absolute value of BigInt
pub fn generate_bigint_abs<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("abs_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;

    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();

    let abs_func = state
        .module
        .get_function("vp_bigint_abs")
        .ok_or_else(|| "vp_bigint_abs not declared".to_string())?;

    state
        .ir_builder
        .build_call(
            state.builder,
            abs_func,
            &[result_ptr.into(), bigint_val.into()],
            "bigint_abs_call",
        );

    Ok(result_ptr.into())
}

/// Generate pow_bigint() call - BigInt power
pub fn generate_bigint_pow<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!("pow_bigint() takes exactly 2 arguments, got {}", args.len()));
    }

    let base_val = generate_expr(state, &args[0])?;
    let exp_val = generate_expr(state, &args[1])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;

    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();

    let pow_func = state
        .module
        .get_function("vp_bigint_pow")
        .ok_or_else(|| "vp_bigint_pow not declared".to_string())?;

    state
        .ir_builder
        .build_call(
            state.builder,
            pow_func,
            &[result_ptr.into(), base_val.into(), exp_val.into()],
            "bigint_pow_call",
        );

    Ok(result_ptr.into())
}

/// Generate sqrt_bigint() call - BigInt square root
pub fn generate_bigint_sqrt<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("sqrt_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;

    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();

    let sqrt_func = state
        .module
        .get_function("vp_bigint_sqrt")
        .ok_or_else(|| "vp_bigint_sqrt not declared".to_string())?;

    state
        .ir_builder
        .build_call(
            state.builder,
            sqrt_func,
            &[result_ptr.into(), bigint_val.into()],
            "bigint_sqrt_call",
        );

    Ok(result_ptr.into())
}

/// Generate min_bigint() call
pub fn generate_bigint_min<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!("min_bigint() takes exactly 2 arguments, got {}", args.len()));
    }

    let a_val = generate_expr(state, &args[0])?;
    let b_val = generate_expr(state, &args[1])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;

    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();

    let min_func = state
        .module
        .get_function("vp_bigint_min")
        .ok_or_else(|| "vp_bigint_min not declared".to_string())?;

    state
        .ir_builder
        .build_call(
            state.builder,
            min_func,
            &[result_ptr.into(), a_val.into(), b_val.into()],
            "bigint_min_call",
        );

    Ok(result_ptr.into())
}

/// Generate max_bigint() call
pub fn generate_bigint_max<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!("max_bigint() takes exactly 2 arguments, got {}", args.len()));
    }

    let a_val = generate_expr(state, &args[0])?;
    let b_val = generate_expr(state, &args[1])?;
    let from_i64_func = state
        .module
        .get_function("vp_bigint_from_i64")
        .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;

    let zero = state.ir_builder.i64_const(0);
    let result_ptr = state
        .ir_builder
        .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
        .into_pointer_value();

    let max_func = state
        .module
        .get_function("vp_bigint_max")
        .ok_or_else(|| "vp_bigint_max not declared".to_string())?;

    state
        .ir_builder
        .build_call(
            state.builder,
            max_func,
            &[result_ptr.into(), a_val.into(), b_val.into()],
            "bigint_max_call",
        );

    Ok(result_ptr.into())
}

/// Generate is_zero_bigint() call
pub fn generate_bigint_is_zero<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("is_zero_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let is_zero_func = state
        .module
        .get_function("vp_bigint_is_zero")
        .ok_or_else(|| "vp_bigint_is_zero not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, is_zero_func, &[bigint_val.into()], "bigint_is_zero")
        .ok_or_else(|| "Failed to call vp_bigint_is_zero".to_string())?;

    Ok(result)
}

/// Generate is_negative_bigint() call
pub fn generate_bigint_is_negative<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("is_negative_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let is_neg_func = state
        .module
        .get_function("vp_bigint_is_negative")
        .ok_or_else(|| "vp_bigint_is_negative not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, is_neg_func, &[bigint_val.into()], "bigint_is_negative")
        .ok_or_else(|| "Failed to call vp_bigint_is_negative".to_string())?;

    Ok(result)
}

/// Generate sign_bigint() call
pub fn generate_bigint_sign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("sign_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let sign_func = state
        .module
        .get_function("vp_bigint_sign")
        .ok_or_else(|| "vp_bigint_sign not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, sign_func, &[bigint_val.into()], "bigint_sign")
        .ok_or_else(|| "Failed to call vp_bigint_sign".to_string())?;

    Ok(result)
}

/// Generate bit_length_bigint() call
pub fn generate_bigint_bit_length<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("bit_length_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;
    let bit_len_func = state
        .module
        .get_function("vp_bigint_bit_length")
        .ok_or_else(|| "vp_bigint_bit_length not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, bit_len_func, &[bigint_val.into()], "bigint_bit_length")
        .ok_or_else(|| "Failed to call vp_bigint_bit_length".to_string())?;

    Ok(result)
}
