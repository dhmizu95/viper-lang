//! Len function code generation for Viper

use crate::ast::Expr;
use crate::codegen::state::CodeGenState;

use inkwell::values::BasicValueEnum;

use crate::codegen::expressions::core::generate_expr;

/// Generate len() call
pub fn generate_len_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!(
            "len() takes exactly 1 argument, got {}",
            args.len()
        ));
    }

    let obj_expr = &args[0];
    let obj_val = generate_expr(state, obj_expr)?;

    // Check if it's a tuple (now heap-allocated pointers)
    let is_tuple = obj_val.is_pointer_value()
        && match obj_expr {
            Expr::Ident(name, _) => state
                .var_types
                .get(name)
                .map(|t| matches!(t, crate::ast::Type::Tuple(_)))
                .unwrap_or(false),
            Expr::Tuple { .. } => true,
            _ => false,
        };

    if is_tuple {
        // For tuples, call vp_tuple_len
        let tuple_len_func = state
            .module
            .get_function("vp_tuple_len")
            .ok_or_else(|| "vp_tuple_len not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, tuple_len_func, &[obj_val.into()], "tuple_len")
            .ok_or_else(|| "Failed to call vp_tuple_len".to_string())?;
        return Ok(result);
    }

    // Check if it's a list (literal, variable, or list repetition)
    let is_list = match obj_expr {
        Expr::List { .. } | Expr::Array { .. } | Expr::ListComprehension { .. } => true,
        Expr::Ident(name, _) => state.is_list(name),
        // Check for list repetition: [elem] * n
        Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
            matches!(left.as_ref(), Expr::List { .. } | Expr::Array { .. })
        }
        _ => false,
    };

    // Check if it's a bool list (bit vector)
    let is_bool_list = match obj_expr {
        Expr::Ident(name, _) => state.is_bool_list(name),
        Expr::List { elements, .. } => {
            elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
        }
        Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
            if let Expr::List { elements, .. } = left.as_ref() {
                elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false)
            } else {
                false
            }
        }
        _ => false,
    };

    // Call the appropriate length function
    if is_bool_list {
        // Use bit vector len for bool lists
        let bitvec_len = state
            .module
            .get_function("vp_bitvec_len")
            .ok_or_else(|| "vp_bitvec_len not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, bitvec_len, &[obj_val.into()], "bitvec_len")
            .unwrap();

        // Tag the return value (runtime returns untagged length)
        let result_tagged = state
            .builder
            .build_left_shift(
                result.into_int_value(),
                state.context.i64_type().const_int(1, false),
                "bitvec_len_tagged",
            )
            .expect("failed to tag bitvec_len result");

        return Ok(result_tagged.into());
    } else if is_list {
        // Call vp_list_len for other lists
        let list_len = state
            .module
            .get_function("vp_list_len")
            .ok_or_else(|| "vp_list_len not declared".to_string())?;
        let result = state
            .ir_builder
            .build_call(state.builder, list_len, &[obj_val.into()], "list_len")
            .unwrap();

        // Tag the return value (runtime returns untagged length)
        let result_tagged = state
            .builder
            .build_left_shift(
                result.into_int_value(),
                state.context.i64_type().const_int(1, false),
                "len_tagged",
            )
            .expect("failed to tag len result");

        return Ok(result_tagged.into());
    }

    // Otherwise treat as string (for string literals or variables)
    if obj_val.is_pointer_value() {
        // Check if it's a BigInt variable
        let is_bigint = match obj_expr {
            Expr::Ident(name, _) => state.is_bigint(name),
            Expr::BigInt(..) => true,
            _ => false,
        };

        if is_bigint {
            // For BigInt, convert to string first and get length
            let to_str_func = state
                .module
                .get_function("vp_bigint_to_str")
                .ok_or_else(|| "vp_bigint_to_str not declared".to_string())?;
            let base = state.context.i32_type().const_int(10, false);
            let str_val = state
                .ir_builder
                .build_call(
                    state.builder,
                    to_str_func,
                    &[obj_val.into(), base.into()],
                    "bigint_to_str_for_len",
                )
                .expect("bigint_to_str");

            let str_len = state
                .module
                .get_function("vp_str_len")
                .ok_or_else(|| "vp_str_len not declared".to_string())?;
            let result =
                state.ir_builder.build_call(state.builder, str_len, &[str_val.into()], "str_len");
            return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
        }

        // Call vp_str_len for strings
        let str_len = state
            .module
            .get_function("vp_str_len")
            .ok_or_else(|| "vp_str_len not declared".to_string())?;
        let result =
            state.ir_builder.build_call(state.builder, str_len, &[obj_val.into()], "str_len");
        return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
    }

    // Fallback: treat as list
    let list_len = state
        .module
        .get_function("vp_list_len")
        .ok_or_else(|| "vp_list_len not declared".to_string())?;
    let result =
        state.ir_builder.build_call(state.builder, list_len, &[obj_val.into()], "list_len");
    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}
