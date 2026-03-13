//! Conversion built-in functions

use crate::ast::Expr;
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate str_bigint() call - convert BigInt to string
pub fn generate_bigint_to_str<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!("str_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;

    let to_str_func = state
        .module
        .get_function("vp_bigint_to_str")
        .ok_or_else(|| "vp_bigint_to_str not declared".to_string())?;

    // Call vp_bigint_to_str(bigint, 10) - base 10
    let base = state.context.i32_type().const_int(10, false);

    // Use custom build_call wrapper which returns Option<BasicValueEnum>
    let str_val = state
        .ir_builder
        .build_call(state.builder, to_str_func, &[bigint_val.into(), base.into()], "bigint_to_str")
        .ok_or_else(|| "vp_bigint_to_str did not return a value".to_string())?;

    Ok(str_val)
}

/// Generate int_bigint() call - convert BigInt to i64
pub fn generate_bigint_to_i64<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!("int_bigint() takes exactly 1 argument, got {}", args.len()));
    }

    let bigint_val = generate_expr(state, &args[0])?;

    let to_i64_func = state
        .module
        .get_function("vp_bigint_to_i64")
        .ok_or_else(|| "vp_bigint_to_i64 not declared".to_string())?;

    // Use custom build_call wrapper which returns Option<BasicValueEnum>
    let i64_val = state
        .ir_builder
        .build_call(state.builder, to_i64_func, &[bigint_val.into()], "bigint_to_i64")
        .ok_or_else(|| "vp_bigint_to_i64 did not return a value".to_string())?;

    Ok(i64_val)
}

/// Generate bin() call
pub fn generate_bin_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("bin() requires at least 1 argument".to_string());
    }

    let num_val = generate_expr(state, &args[0])?.into_int_value();

    let func = state
        .module
        .get_function("vp_bin_i64")
        .ok_or_else(|| "vp_bin_i64 not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[num_val.into()], "bin_result");
    Ok(result.unwrap())
}

/// Generate oct() call
pub fn generate_oct_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("oct() requires at least 1 argument".to_string());
    }

    let num_val = generate_expr(state, &args[0])?.into_int_value();

    let func = state
        .module
        .get_function("vp_oct_i64")
        .ok_or_else(|| "vp_oct_i64 not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[num_val.into()], "oct_result");
    Ok(result.unwrap())
}

/// Generate hex() call
pub fn generate_hex_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("hex() requires at least 1 argument".to_string());
    }

    let num_val = generate_expr(state, &args[0])?.into_int_value();

    let func = state
        .module
        .get_function("vp_hex_i64")
        .ok_or_else(|| "vp_hex_i64 not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[num_val.into()], "hex_result");
    Ok(result.unwrap())
}

/// Generate chr() call
pub fn generate_chr_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("chr() requires at least 1 argument".to_string());
    }

    let num_val = generate_expr(state, &args[0])?.into_int_value();

    let func = state
        .module
        .get_function("vp_chr_i64")
        .ok_or_else(|| "vp_chr_i64 not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[num_val.into()], "chr_result");
    Ok(result.unwrap())
}

/// Generate ord() call
pub fn generate_ord_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("ord() requires at least 1 argument".to_string());
    }

    let str_val = generate_expr(state, &args[0])?;

    let func = state
        .module
        .get_function("vp_ord_str")
        .ok_or_else(|| "vp_ord_str not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[str_val.into()], "ord_result");
    Ok(result.unwrap())
}
