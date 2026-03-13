//! Numeric built-in functions

use crate::ast::Expr;
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate round() call
pub fn generate_round_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("round() requires at least 1 argument".to_string());
    }

    let number_val = generate_expr(state, &args[0])?;
    let ndigits = if args.len() > 1 {
        generate_expr(state, &args[1])?.into_int_value()
    } else {
        state.ir_builder.i64_const(0)
    };

    let number_float = if number_val.is_float_value() {
        number_val.into_float_value()
    } else {
        state
            .builder
            .build_signed_int_to_float(
                number_val.into_int_value(),
                state.context.f64_type(),
                "int_to_float",
            )
            .expect("int to float")
    };

    let func = state
        .module
        .get_function("vp_round_f64")
        .ok_or_else(|| "vp_round_f64 not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        func,
        &[number_float.into(), ndigits.into()],
        "round_result",
    );
    Ok(result.unwrap())
}

/// Generate divmod() call
pub fn generate_divmod_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 2 {
        return crate::codegen::codegen_error("divmod() requires exactly 2 arguments".to_string());
    }

    let a_val = generate_expr(state, &args[0])?;
    let b_val = generate_expr(state, &args[1])?;

    // Check if either argument is BigInt (pointer value)
    let is_bigint = a_val.is_pointer_value() || b_val.is_pointer_value();

    if is_bigint {
        // Use BigInt divmod
        let from_i64_func = state
            .module
            .get_function("vp_bigint_from_i64")
            .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;

        let get_bigint =
            |val: BasicValueEnum<'ctx>| -> crate::codegen::Result<BasicValueEnum<'ctx>> {
                if val.is_pointer_value() {
                    Ok(val)
                } else if val.is_int_value() {
                    let res = state
                        .ir_builder
                        .build_call(state.builder, from_i64_func, &[val.into()], "bigint_from_i64")
                        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?;
                    Ok(res.into_pointer_value().into())
                } else {
                    crate::codegen::codegen_error("Cannot convert to BigInt for divmod".to_string())
                }
            };

        let a_bigint = get_bigint(a_val)?;
        let b_bigint = get_bigint(b_val)?;

        // Allocate result BigInt
        let zero = state.ir_builder.i64_const(0);
        let quot_ptr = state
            .ir_builder
            .build_call(state.builder, from_i64_func, &[zero.into()], "quot_bigint")
            .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
            .into_pointer_value();

        let rem_ptr = state
            .ir_builder
            .build_call(state.builder, from_i64_func, &[zero.into()], "rem_bigint")
            .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
            .into_pointer_value();

        let divmod_func = state
            .module
            .get_function("vp_bigint_divmod")
            .ok_or_else(|| "vp_bigint_divmod not declared".to_string())?;

        state.ir_builder.build_call(
            state.builder,
            divmod_func,
            &[quot_ptr.into(), rem_ptr.into(), a_bigint.into(), b_bigint.into()],
            "divmod_bigint",
        );

        // Return tuple (quotient, remainder)
        let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
        let tuple_struct = state.context.struct_type(&[ptr_type.into(), ptr_type.into()], false);
        let tuple_val = state.builder.build_alloca(tuple_struct, "divmod_tuple").expect("alloca");
        let quot_gep = state
            .builder
            .build_struct_gep(tuple_struct, tuple_val, 0, "quot_gep")
            .expect("quot_gep");
        let rem_gep =
            state.builder.build_struct_gep(tuple_struct, tuple_val, 1, "rem_gep").expect("rem_gep");
        state.builder.build_store(quot_gep, quot_ptr).expect("store_quot");
        state.builder.build_store(rem_gep, rem_ptr).expect("store_rem");
        let loaded = state
            .builder
            .build_load(tuple_struct, tuple_val, "divmod_result")
            .expect("load_divmod");
        Ok(loaded)
    } else {
        // Use i64 divmod
        let a_int = a_val.into_int_value();
        let b_int = b_val.into_int_value();

        let func = state
            .module
            .get_function("vp_divmod_i64")
            .ok_or_else(|| "vp_divmod_i64 not declared".to_string())?;

        let result = state.ir_builder.build_call(
            state.builder,
            func,
            &[a_int.into(), b_int.into()],
            "divmod_result",
        );
        Ok(result.unwrap())
    }
}

/// Generate pow() call
pub fn generate_pow_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() < 2 || args.len() > 3 {
        return crate::codegen::codegen_error("pow() requires 2 or 3 arguments".to_string());
    }

    let base_val = generate_expr(state, &args[0])?;
    let exp_val = generate_expr(state, &args[1])?;

    // Check if either argument is BigInt using the same detection as bigint operators
    let is_bigint_expr = |arg: &Expr| -> bool {
        use crate::codegen::expressions::operators::bigint::is_bigint_expr as check_bigint;
        check_bigint(arg, state)
    };

    let is_bigint = args.iter().any(is_bigint_expr)
        || base_val.is_pointer_value()
        || exp_val.is_pointer_value();

    if is_bigint {
        let from_i64_func = state
            .module
            .get_function("vp_bigint_from_i64")
            .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;

        let get_bigint =
            |val: BasicValueEnum<'ctx>| -> crate::codegen::Result<BasicValueEnum<'ctx>> {
                if val.is_pointer_value() {
                    Ok(val)
                } else if val.is_int_value() {
                    let res = state
                        .ir_builder
                        .build_call(state.builder, from_i64_func, &[val.into()], "bigint_from_i64")
                        .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?;
                    Ok(res.into_pointer_value().into())
                } else {
                    crate::codegen::codegen_error("Cannot convert to BigInt for pow()".to_string())
                }
            };

        let base_ptr = get_bigint(base_val)?;
        let exp_ptr = get_bigint(exp_val)?;

        let zero = state.ir_builder.i64_const(0);
        let result_ptr = state
            .ir_builder
            .build_call(state.builder, from_i64_func, &[zero.into()], "bigint_res")
            .ok_or_else(|| "Failed to call vp_bigint_from_i64".to_string())?
            .into_pointer_value();

        if args.len() == 3 {
            let mod_expr = generate_expr(state, &args[2])?;
            let mod_ptr = get_bigint(mod_expr)?;
            let powmod_func = state
                .module
                .get_function("vp_bigint_powmod")
                .ok_or_else(|| "vp_bigint_powmod not declared".to_string())?;

            state.ir_builder.build_call(
                state.builder,
                powmod_func,
                &[result_ptr.into(), base_ptr.into(), exp_ptr.into(), mod_ptr.into()],
                "bigint_powmod_call",
            );
        } else {
            let pow_func = state
                .module
                .get_function("vp_bigint_pow")
                .ok_or_else(|| "vp_bigint_pow not declared".to_string())?;

            state.ir_builder.build_call(
                state.builder,
                pow_func,
                &[result_ptr.into(), base_ptr.into(), exp_ptr.into()],
                "bigint_pow_call",
            );
        }
        return Ok(result_ptr.into());
    }

    if args.len() == 3 {
        return crate::codegen::codegen_error(
            "3-argument pow() only supported for BigInt types currently".to_string(),
        );
    }

    // Use float pow for non-BigInt cases
    let base_float = if base_val.is_float_value() {
        base_val.into_float_value()
    } else {
        state
            .builder
            .build_signed_int_to_float(
                base_val.into_int_value(),
                state.context.f64_type(),
                "int_to_float",
            )
            .expect("int to float")
    };

    let exp_float = if exp_val.is_float_value() {
        exp_val.into_float_value()
    } else {
        state
            .builder
            .build_signed_int_to_float(
                exp_val.into_int_value(),
                state.context.f64_type(),
                "int_to_float",
            )
            .expect("int to float")
    };

    let func = state
        .module
        .get_function("vp_pow_f64")
        .ok_or_else(|| "vp_pow_f64 not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        func,
        &[base_float.into(), exp_float.into()],
        "pow_result",
    );
    Ok(result.unwrap())
}
