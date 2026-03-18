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

    // Check if iterable is a bytearray
    let is_bytearray = match &args[0] {
        Expr::Ident(name, _) => state.is_bytearray(name),
        Expr::Call { func, .. } => {
            if let Expr::Ident(func_name, _) = func.as_ref() {
                func_name == "bytearray"
            } else {
                false
            }
        }
        Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
            // Handle bytearray * n pattern
            if let Expr::Call { func, .. } = left.as_ref() {
                if let Expr::Ident(func_name, _) = func.as_ref() {
                    func_name == "bytearray"
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    };

    let func_name = if is_bytearray { "vp_enumerate_bytearray" } else { "vp_enumerate" };
    let func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

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
/// Supports both min(iterable) and min(arg1, arg2, ...) forms
pub fn generate_min_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("min() requires at least 1 argument".to_string());
    }

    // Multiple arguments: min(a, b, c, ...) - compare and return minimum
    if args.len() > 1 {
        let mut min_val = generate_expr(state, &args[0])?;
        
        for arg in &args[1..] {
            let arg_val = generate_expr(state, arg)?;
            
            // Both values should be integers for comparison
            let min_int = min_val.into_int_value();
            let arg_int = arg_val.into_int_value();
            
            let cmp = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::SLT,
                    arg_int,
                    min_int,
                    "min_cmp",
                )
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to compare in min(): {:?}", e)))?;
            
            let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            let then_block = state.context.append_basic_block(func, "min_then");
            let else_block = state.context.append_basic_block(func, "min_else");
            let merge_block = state.context.append_basic_block(func, "min_merge");
            
            state
                .builder
                .build_conditional_branch(cmp, then_block, else_block)
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to build min branch: {:?}", e)))?;
            
            // Then block: arg is smaller
            state.builder.position_at_end(then_block);
            state
                .builder
                .build_unconditional_branch(merge_block)
                .map_err(|_e| crate::codegen::codegen_err("Failed to build min then branch".to_string()))?;
            
            // Else block: min stays the same
            state.builder.position_at_end(else_block);
            state
                .builder
                .build_unconditional_branch(merge_block)
                .map_err(|_e| crate::codegen::codegen_err("Failed to build min else branch".to_string()))?;
            
            // Merge block with phi
            state.builder.position_at_end(merge_block);
            let phi = state
                .builder
                .build_phi(arg_int.get_type(), "min_result")
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to build min phi: {:?}", e)))?;
            phi.add_incoming(&[(&arg_int, then_block), (&min_int, else_block)]);
            
            min_val = phi.as_basic_value();
        }
        
        return Ok(min_val);
    }

    // Single argument: min(iterable) - use runtime function
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
/// Supports both max(iterable) and max(arg1, arg2, ...) forms
pub fn generate_max_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("max() requires at least 1 argument".to_string());
    }

    // Multiple arguments: max(a, b, c, ...) - compare and return maximum
    if args.len() > 1 {
        let mut max_val = generate_expr(state, &args[0])?;
        
        for arg in &args[1..] {
            let arg_val = generate_expr(state, arg)?;
            
            // Both values should be integers for comparison
            let max_int = max_val.into_int_value();
            let arg_int = arg_val.into_int_value();
            
            let cmp = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::SGT,
                    arg_int,
                    max_int,
                    "max_cmp",
                )
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to compare in max(): {:?}", e)))?;
            
            let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            let then_block = state.context.append_basic_block(func, "max_then");
            let else_block = state.context.append_basic_block(func, "max_else");
            let merge_block = state.context.append_basic_block(func, "max_merge");
            
            state
                .builder
                .build_conditional_branch(cmp, then_block, else_block)
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to build max branch: {:?}", e)))?;
            
            // Then block: arg is larger
            state.builder.position_at_end(then_block);
            state
                .builder
                .build_unconditional_branch(merge_block)
                .map_err(|_e| crate::codegen::codegen_err("Failed to build max then branch".to_string()))?;
            
            // Else block: max stays the same
            state.builder.position_at_end(else_block);
            state
                .builder
                .build_unconditional_branch(merge_block)
                .map_err(|_e| crate::codegen::codegen_err("Failed to build max else branch".to_string()))?;
            
            // Merge block with phi
            state.builder.position_at_end(merge_block);
            let phi = state
                .builder
                .build_phi(arg_int.get_type(), "max_result")
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to build max phi: {:?}", e)))?;
            phi.add_incoming(&[(&arg_int, then_block), (&max_int, else_block)]);
            
            max_val = phi.as_basic_value();
        }
        
        return Ok(max_val);
    }

    // Single argument: max(iterable) - use runtime function
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
