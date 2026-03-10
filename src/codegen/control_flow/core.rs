use crate::ast::Expr;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::LoopContext;
use inkwell::values::BasicValueEnum;

/// Generate a return statement with type coercion to match function signature
pub fn generate_return<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    value: &Option<Expr>,
) -> Result<(), String> {
    if let Some(val) = value {
        // If it's an explicit None return, and the function returns void,
        // treat it as a void return.
        if matches!(val, Expr::None(_)) {
            let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            if func.get_type().get_return_type().is_none() {
                state.ir_builder.build_return(state.builder, None);
                return Ok(());
            }
        }

        let v = crate::codegen::expressions::generate_expr(state, val)?;
        
        // Get the function's expected return type
        let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
        let expected_return_type = func.get_type().get_return_type();
        
        // Coerce the value if needed
        let coerced_value = coerce_return_value(state, v, expected_return_type)?;
        state.ir_builder.build_return(state.builder, Some(&coerced_value));
    } else {
        state.ir_builder.build_return(state.builder, None);
    }
    Ok(())
}

/// Coerce a return value to match the function's expected return type
fn coerce_return_value<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
    expected_type: Option<inkwell::types::BasicTypeEnum<'ctx>>,
) -> Result<BasicValueEnum<'ctx>, String> {
    if let Some(expected) = expected_type {
        let value_type = value.get_type();

        // If types already match, no coercion needed
        if value_type == expected {
            return Ok(value);
        }

        // Handle pointer -> pointer coercion (bitcast if both are pointer types)
        if value_type.is_pointer_type() && expected.is_pointer_type() {
            // Both are pointer types, just bitcast to match expected type
            let cast_value = state.builder.build_bit_cast(
                value.into_pointer_value(),
                expected.into_pointer_type(),
                "return_ptr_cast",
            ).map_err(|e| format!("Pointer bitcast failed: {:?}", e))?;
            return Ok(cast_value.into());
        }

        // Handle i64 -> pointer coercion (e.g., returning a stored field value as pointer)
        // This happens when a method returns a reference type field stored as i64
        if value_type.is_int_type() && expected.is_pointer_type() {
            // Convert i64 back to pointer
            let ptr_val = state.builder.build_int_to_ptr(
                value.into_int_value(),
                expected.into_pointer_type(),
                "i64_to_ptr",
            ).map_err(|e| format!("i64 to ptr cast failed: {:?}", e))?;
            return Ok(ptr_val.into());
        }

        // Handle pointer -> i64 coercion (shouldn't happen normally, but for completeness)
        if value_type.is_pointer_type() && expected.is_int_type() {
            // This would be an error case - returning a pointer where int expected
            return Err(format!(
                "Cannot convert pointer to integer in return value. Expected {:?}, got {:?}",
                expected, value_type
            ));
        }

        // Handle i64 -> f64 coercion (returning stored float field)
        if value_type.is_int_type() && expected.is_float_type() {
            let f64_val = state.builder.build_signed_int_to_float(
                value.into_int_value(),
                expected.into_float_type(),
                "i64_to_f64",
            ).map_err(|e| format!("i64 to f64 cast failed: {:?}", e))?;
            return Ok(f64_val.into());
        }

        // Handle i64 -> bool coercion (returning stored bool field)
        if value_type.is_int_type() && expected.is_int_type() && expected.into_int_type().get_bit_width() == 1 {
            let bool_val = state.builder.build_int_truncate_or_bit_cast(
                value.into_int_value(),
                expected.into_int_type(),
                "i64_to_bool",
            ).map_err(|e| format!("i64 to bool cast failed: {:?}", e))?;
            return Ok(bool_val.into());
        }
    }

    Ok(value)
}

/// Generate a break statement
pub fn generate_break<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    ir_builder: &crate::codegen::builder::IRBuilder<'ctx>,
    loop_stack: &[LoopContext<'ctx>],
) -> Result<(), String> {
    if let Some(loop_ctx) = loop_stack.last() {
        ir_builder.build_branch(builder, loop_ctx.break_block);
        Ok(())
    } else {
        Err("break statement outside of loop".to_string())
    }
}

/// Generate a continue statement
pub fn generate_continue<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    ir_builder: &crate::codegen::builder::IRBuilder<'ctx>,
    loop_stack: &[LoopContext<'ctx>],
) -> Result<(), String> {
    if let Some(loop_ctx) = loop_stack.last() {
        ir_builder.build_branch(builder, loop_ctx.continue_block);
        Ok(())
    } else {
        Err("continue statement outside of loop".to_string())
    }
}
