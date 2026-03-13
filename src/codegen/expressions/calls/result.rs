//! Result type constructors

use crate::ast::Expr;
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate Ok constructor call
/// Creates a Result struct with is_ok=1 and the value, returned by value
pub fn generate_ok_constructor<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!("Ok() takes exactly 1 argument, got {}", args.len()));
    }

    // Generate the value expression
    let value = generate_expr(state, &args[0])?;

    // For Result types, we use representation:
    // { is_ok: i8, value: i64 } where value is bitcast to i64 if needed
    let result_struct_type = state.context.struct_type(&[
        state.context.i8_type().into(),
        state.context.i64_type().into(),
    ], false);

    // Allocate space for the Result struct
    let result_alloca = state.builder.build_alloca(result_struct_type, "ok_result").expect("alloca");

    // Store is_ok = 1
    let is_ok_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_zero()],
            "is_ok_ptr",
        )
    }.map_err(|e| format!("Failed to get is_ok field: {:?}", e))?;
    state.builder.build_store(is_ok_ptr, state.context.i8_type().const_int(1, false)).expect("store");

    // Convert value to i64 representation and store
    let value_i64_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_int(1, false)],
            "value_i64_ptr",
        )
    }.map_err(|e| format!("Failed to get value field: {:?}", e))?;

    // Convert value to i64 representation
    let value_i64 = if value.is_int_value() {
        value.into_int_value()
    } else if value.is_float_value() {
        state.builder.build_float_to_unsigned_int(
            value.into_float_value(),
            state.context.i64_type(),
            "ok_f64_to_i64",
        ).map_err(|e| format!("Failed to convert f64 to i64: {:?}", e))?
    } else if value.is_pointer_value() {
        state.builder.build_ptr_to_int(
            value.into_pointer_value(),
            state.context.i64_type(),
            "ok_ptr_to_i64",
        ).map_err(|e| format!("Failed to convert ptr to i64: {:?}", e))?
    } else {
        return crate::codegen::codegen_error(format!("Unsupported Ok value type: {:?}", value.get_type()));
    };

    state.builder.build_store(value_i64_ptr, value_i64).expect("store");

    // Load and return the struct value
    let result_val = state.builder.build_load(result_struct_type, result_alloca, "ok_result_val").expect("load");
    Ok(result_val.into())
}

/// Generate Err constructor call
/// Creates a Result struct with is_ok=0 and the error value, returned by value
pub fn generate_err_constructor<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!("Err() takes exactly 1 argument, got {}", args.len()));
    }

    // Generate the error expression
    let error = generate_expr(state, &args[0])?;

    // For Result types, we use representation:
    // { is_ok: i8, error: i64 } where error is bitcast to i64 if needed
    let result_struct_type = state.context.struct_type(&[
        state.context.i8_type().into(),
        state.context.i64_type().into(),
    ], false);

    // Allocate space for the Result struct
    let result_alloca = state.builder.build_alloca(result_struct_type, "err_result").expect("alloca");

    // Store is_ok = 0
    let is_ok_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_zero()],
            "is_ok_ptr",
        )
    }.map_err(|e| format!("Failed to get is_ok field: {:?}", e))?;
    state.builder.build_store(is_ok_ptr, state.context.i8_type().const_int(0, false)).expect("store");

    // Convert error to i64 representation and store
    let error_i64_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            result_struct_type,
            result_alloca,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_int(1, false)],
            "error_i64_ptr",
        )
    }.map_err(|e| format!("Failed to get error field: {:?}", e))?;

    // Convert error to i64 representation
    let error_i64 = if error.is_pointer_value() {
        state.builder.build_ptr_to_int(
            error.into_pointer_value(),
            state.context.i64_type(),
            "err_ptr_to_i64",
        ).map_err(|e| format!("Failed to convert ptr to i64: {:?}", e))?
    } else if error.is_int_value() {
        error.into_int_value()
    } else if error.is_float_value() {
        state.builder.build_float_to_unsigned_int(
            error.into_float_value(),
            state.context.i64_type(),
            "err_f64_to_i64",
        ).map_err(|e| format!("Failed to convert f64 to i64: {:?}", e))?
    } else {
        return crate::codegen::codegen_error(format!("Unsupported Err value type: {:?}", error.get_type()));
    };

    state.builder.build_store(error_i64_ptr, error_i64).expect("store");

    // Load and return the struct value
    let result_val = state.builder.build_load(result_struct_type, result_alloca, "err_result_val").expect("load");
    Ok(result_val.into())
}
