//! Array creation for Viper

use inkwell::values::BasicValueEnum;

use crate::ast::Expr;
use crate::codegen::state::CodeGenState;

use crate::codegen::expressions::generate_expr;

/// Generate array creation (fixed-size, stack-allocated)
pub fn generate_array<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
    size: Option<usize>,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let array_size = size.unwrap_or_else(|| elements.len());

    if array_size == 0 {
        // Empty array - return null pointer
        let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
        return Ok(ptr_type.const_null().into());
    }

    // Get element type from first element or default to i64
    let elem_type: inkwell::types::BasicTypeEnum = if let Some(first_elem) = elements.first() {
        match first_elem {
            Expr::Int(_, _) => state.context.i64_type().into(),
            Expr::Float(_, _) => state.context.f64_type().into(),
            Expr::Bool(_, _) => state.context.bool_type().into(),
            _ => state.context.i64_type().into(),
        }
    } else {
        state.context.i64_type().into()
    };

    // Allocate array on stack as a single alloca of element type with size
    let array_alloca = state
        .builder
        .build_array_alloca(
            elem_type,
            state.context.i32_type().const_int(array_size as u64, false),
            "array",
        )
        .map_err(|e| format!("Failed to allocate array: {:?}", e))?;

    // Check if this is array repeat syntax: [value; size]
    let is_repeat = elements.len() == 1 && size.is_some() && size.unwrap() > 1;

    // Initialize elements
    for i in 0..array_size {
        let elem_val = if is_repeat {
            // For repeat syntax, use the first element value for all positions
            generate_expr(state, &elements[0])?
        } else if i < elements.len() {
            // For regular arrays, use the corresponding element
            generate_expr(state, &elements[i])?
        } else {
            // Fill remaining elements with zero
            let zero_val: BasicValueEnum = if elem_type.is_int_type() {
                elem_type.into_int_type().const_zero().into()
            } else if elem_type.is_float_type() {
                elem_type.into_float_type().const_zero().into()
            } else {
                elem_type.into_int_type().const_zero().into()
            };
            zero_val
        };

        // Create GEP to element position
        let elem_ptr = unsafe {
            state.builder.build_in_bounds_gep(
                elem_type,
                array_alloca,
                &[state.context.i32_type().const_int(i as u64, false)],
                &format!("elem_{}", i),
            )
        }
        .map_err(|e| format!("Failed to build GEP: {:?}", e))?;

        state
            .builder
            .build_store(elem_ptr, elem_val)
            .map_err(|e| format!("Failed to store element: {:?}", e))?;
    }

    Ok(array_alloca.into())
}
