//! Introspection built-in functions

use crate::ast::Expr;
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate isinstance() check for runtime type narrowing
/// isinstance(obj, Type) returns bool indicating if obj is of Type
pub fn generate_isinstance_check<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<inkwell::values::BasicValueEnum<'ctx>> {
    if args.len() != 2 {
        return crate::codegen::codegen_error("isinstance() takes exactly 2 arguments".to_string());
    }

    // Generate the object expression
    let obj_val = crate::codegen::expressions::generate_expr(state, &args[0])?;

    // Get the type name from the second argument (should be a type identifier or None)
    let type_name = match &args[1] {
        Expr::Ident(name, _) => name.clone(),
        Expr::None(_) => "None".to_string(),  // Handle None literal
        _ => return crate::codegen::codegen_error("isinstance() second argument must be a type name".to_string()),
    };

    // For now, implement basic type checks based on the expected type
    // In a full implementation, this would use runtime type information

    // Check if we're checking against primitive types
    let result = match type_name.as_str() {
        "i64" | "i32" | "i16" | "i8" | "int" => {
            // Check if value is an integer type
            if obj_val.is_int_value() {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "f64" | "f32" | "float" => {
            // Check if value is a float type
            if obj_val.is_float_value() {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "bool" => {
            // Check if value is a bool (i1)
            if obj_val.is_int_value() && obj_val.get_type().into_int_type().get_bit_width() == 1 {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "str" => {
            // Check if value is a string (pointer)
            if obj_val.is_pointer_value() {
                // For strings, we'd need to check the actual runtime type
                // For now, assume pointers could be strings
                // A full implementation would check the type tag
                state.context.bool_type().const_int(1, false)  // Conservative: assume true for pointers
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "list" => {
            // Check if value is a list (pointer to list struct)
            if obj_val.is_pointer_value() {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "dict" => {
            if obj_val.is_pointer_value() {
                state.context.bool_type().const_int(1, false)
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        "None" => {
            // Check if value is null pointer or special None value (i64 0)
            if obj_val.is_pointer_value() {
                let ptr = obj_val.into_pointer_value();
                let null_ptr = state.context.ptr_type(inkwell::AddressSpace::default()).const_null();
                // Convert pointers to integers for comparison
                let intptr_type = state.context.i64_type();
                let ptr_int = state.builder.build_ptr_to_int(ptr, intptr_type, "ptr_int")
                    .map_err(|e| format!("Failed to convert ptr to int: {:?}", e))?;
                let null_int = state.builder.build_ptr_to_int(null_ptr, intptr_type, "null_int")
                    .map_err(|e| format!("Failed to convert null to int: {:?}", e))?;
                let is_null = state.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    ptr_int,
                    null_int,
                    "is_none",
                ).map_err(|e| format!("Failed to compare: {:?}", e))?;
                is_null
            } else if obj_val.is_int_value() {
                // None is represented as i64(0)
                let zero = state.context.i64_type().const_zero();
                let is_none = state.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    obj_val.into_int_value(),
                    zero,
                    "is_none_int",
                ).map_err(|e| format!("Failed to compare: {:?}", e))?;
                is_none
            } else {
                state.context.bool_type().const_int(0, false)
            }
        }
        // For class types, we'd need runtime type information
        // This would check the type tag in the object header
        _ => {
            // For user-defined classes, we need to check the runtime type
            // This requires RTTI (runtime type information)
            // For now, return a conservative false
            state.context.bool_type().const_int(0, false)
        }
    };

    Ok(result.into())
}

/// Generate type() call
pub fn generate_type_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("type() requires at least 1 argument".to_string());
    }

    let obj_val = generate_expr(state, &args[0])?;

    let func = state
        .module
        .get_function("vp_type_of")
        .ok_or_else(|| "vp_type_of not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[obj_val.into()], "type_result");
    Ok(result.unwrap())
}

/// Generate id() call
pub fn generate_id_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("id() requires at least 1 argument".to_string());
    }

    let obj_val = generate_expr(state, &args[0])?;

    // For non-pointer types, just return the value as-is (as identity)
    if obj_val.is_int_value() {
        return Ok(obj_val);
    }
    if obj_val.is_float_value() {
        // Convert float bits to int
        let float_val = obj_val.into_float_value();
        let int_val = state.builder.build_float_to_signed_int(
            float_val,
            state.context.i64_type(),
            "float_to_int_id",
        ).expect("float to int");
        return Ok(int_val.into());
    }

    // For pointers, return the pointer address as int
    let func = state
        .module
        .get_function("vp_object_id")
        .ok_or_else(|| "vp_object_id not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[obj_val.into()], "id_result");
    Ok(result.unwrap())
}

/// Generate repr() call
pub fn generate_repr_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("repr() requires at least 1 argument".to_string());
    }

    let obj_val = generate_expr(state, &args[0])?;

    let func_name = if obj_val.is_int_value() {
        "vp_repr_i64"
    } else if obj_val.is_float_value() {
        "vp_repr_f64"
    } else if obj_val.is_pointer_value() {
        "vp_repr_str"
    } else {
        "vp_repr_i64"
    };

    let func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[obj_val.into()], "repr_result");
    Ok(result.unwrap())
}

/// Generate callable() call
pub fn generate_callable_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.is_empty() {
        return crate::codegen::codegen_error("callable() requires at least 1 argument".to_string());
    }

    let obj_val = generate_expr(state, &args[0])?;

    let func = state
        .module
        .get_function("vp_is_callable")
        .ok_or_else(|| "vp_is_callable not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[obj_val.into()], "callable_result");
    Ok(result.unwrap())
}
