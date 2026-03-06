//! Struct pack/unpack and hash function code generation for Viper

use crate::ast::Expr;
use crate::codegen::state::CodeGenState;

use inkwell::values::BasicValueEnum;

use crate::codegen::expressions::core::generate_expr;

/// Generate struct.pack call
pub fn generate_struct_pack<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() < 2 {
        return Err("struct.pack requires at least 2 arguments (format, value)".to_string());
    }

    // Generate format string (first arg)
    let format_expr = &args[0];
    let format_val = generate_expr(state, format_expr)?;
    let format_ptr = format_val.into_pointer_value();

    // Generate value (second arg)
    let value_expr = &args[1];
    let value_val = generate_expr(state, value_expr)?;
    let value_int = if value_val.is_int_value() {
        value_val.into_int_value()
    } else if value_val.is_float_value() {
        let float_val = value_val.into_float_value();
        state
            .builder
            .build_float_to_signed_int(float_val, state.context.i64_type(), "float_to_int")
            .expect("float to int")
    } else {
        return Err("Unsupported type for struct.pack".to_string());
    };

    let struct_pack_func = state
        .module
        .get_function("vp_struct_pack")
        .ok_or_else(|| "vp_struct_pack not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        struct_pack_func,
        &[format_ptr.into(), value_int.into()],
        "struct_pack_result",
    );

    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

/// Generate struct.unpack call
pub fn generate_struct_unpack<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() < 2 {
        return Err("struct.unpack requires at least 2 arguments (format, data)".to_string());
    }

    // Generate format string (first arg)
    let format_expr = &args[0];
    let format_val = generate_expr(state, format_expr)?;
    let format_ptr = format_val.into_pointer_value();

    // Generate data pointer (second arg)
    let data_expr = &args[1];
    let data_val = generate_expr(state, data_expr)?;
    let data_ptr = data_val.into_pointer_value();

    let struct_unpack_func = state
        .module
        .get_function("vp_struct_unpack")
        .ok_or_else(|| "vp_struct_unpack not declared".to_string())?;

    let len_val = state.context.i64_type().const_int(0, false);

    let result = state.ir_builder.build_call(
        state.builder,
        struct_unpack_func,
        &[format_ptr.into(), data_ptr.into(), len_val.into()],
        "struct_unpack_result",
    );

    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

/// Generate hash() call - returns hash value for hashable types
pub fn generate_hash_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("hash() takes exactly 1 argument, got {}", args.len()));
    }

    let arg = &args[0];
    let arg_val = generate_expr(state, arg)?;

    // Choose the appropriate hash function based on the type
    let hash_func_name = if arg_val.is_float_value() {
        "vp_hash_f64"
    } else if arg_val.is_int_value() && arg_val.get_type().into_int_type().get_bit_width() == 1 {
        // Check for bool (i1) before general i64
        "vp_hash_bool"
    } else if arg_val.is_int_value() {
        "vp_hash_i64"
    } else if arg_val.is_pointer_value() {
        // String or other pointer type
        "vp_hash_str"
    } else {
        return Err(format!("hash() not supported for type {:?}", arg_val.get_type()));
    };

    let hash_func = state
        .module
        .get_function(hash_func_name)
        .ok_or_else(|| format!("{} not declared", hash_func_name))?;

    let result =
        state.ir_builder.build_call(state.builder, hash_func, &[arg_val.into()], "hash_result");

    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}
