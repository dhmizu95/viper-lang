//! Functional built-in functions

use crate::ast::Expr;
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate enumerate() call
pub fn generate_enumerate_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error(
            "enumerate() requires at least 1 argument".to_string(),
        );
    }

    let iterable_val = generate_expr(state, &args[0])?;
    let start = if args.len() > 1 {
        generate_expr(state, &args[1])?.into_int_value()
    } else {
        state.ir_builder.i64_const(0)
    };

    let func = state
        .module
        .get_function("vp_enumerate")
        .ok_or_else(|| "vp_enumerate not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        func,
        &[iterable_val.into(), start.into()],
        "enumerate_result",
    );
    Ok(result.unwrap())
}

/// Generate zip() call
pub fn generate_zip_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() < 2 {
        return crate::codegen::codegen_error("zip() requires at least 2 arguments".to_string());
    }

    let iter1_val = generate_expr(state, &args[0])?;
    let iter2_val = generate_expr(state, &args[1])?;

    let func =
        state.module.get_function("vp_zip").ok_or_else(|| "vp_zip not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        func,
        &[iter1_val.into(), iter2_val.into()],
        "zip_result",
    );
    Ok(result.unwrap())
}

/// Generate sum() call
pub fn generate_sum_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("sum() requires at least 1 argument".to_string());
    }

    let iterable_val = generate_expr(state, &args[0])?;

    // Use i64 sum for now
    let func = state
        .module
        .get_function("vp_list_sum")
        .ok_or_else(|| "vp_list_sum not declared".to_string())?;

    // For list variables, the value is already a pointer
    // For list literals, generate_list returns a pointer
    // Pass the pointer directly to vp_list_sum
    let result =
        state.ir_builder.build_call(state.builder, func, &[iterable_val.into()], "sum_result");
    Ok(result.unwrap())
}

/// Generate min() call
pub fn generate_min_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("min() requires at least 1 argument".to_string());
    }

    let iterable_val = generate_expr(state, &args[0])?;

    let func = state
        .module
        .get_function("vp_list_min")
        .ok_or_else(|| "vp_list_min not declared".to_string())?;

    let result =
        state.ir_builder.build_call(state.builder, func, &[iterable_val.into()], "min_result");
    Ok(result.unwrap())
}

/// Generate max() call
pub fn generate_max_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("max() requires at least 1 argument".to_string());
    }

    let iterable_val = generate_expr(state, &args[0])?;

    let func = state
        .module
        .get_function("vp_list_max")
        .ok_or_else(|| "vp_list_max not declared".to_string())?;

    let result =
        state.ir_builder.build_call(state.builder, func, &[iterable_val.into()], "max_result");
    Ok(result.unwrap())
}

/// Generate any() call
pub fn generate_any_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("any() requires at least 1 argument".to_string());
    }

    let iterable_val = generate_expr(state, &args[0])?;

    let func = state
        .module
        .get_function("vp_list_any")
        .ok_or_else(|| "vp_list_any not declared".to_string())?;

    let result =
        state.ir_builder.build_call(state.builder, func, &[iterable_val.into()], "any_result");
    Ok(result.unwrap())
}

/// Generate all() call
pub fn generate_all_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("all() requires at least 1 argument".to_string());
    }

    let iterable_val = generate_expr(state, &args[0])?;

    let func = state
        .module
        .get_function("vp_list_all")
        .ok_or_else(|| "vp_list_all not declared".to_string())?;

    let result =
        state.ir_builder.build_call(state.builder, func, &[iterable_val.into()], "all_result");
    Ok(result.unwrap())
}
