//! Closure Cell Code Generation
//!
//! This module handles the creation and access of closure cells for nonlocal variables.
//! 
//! # Closure Cell Structure
//!
//! A closure cell is a heap-allocated structure containing:
//! - A pointer to the actual value (i64*, f64*, etc.)
//!
//! Memory layout:
//! ```text
//! %ClosureCell = type { i8* }  // Opaque pointer to value
//! ```
//!
//! # Usage Pattern
//!
//! 1. Enclosing function creates cells for captured variables:
//!    ```llvm
//!    %cell = call i8* @vp_closure_cell_create()
//!    %value_ptr = alloca i64
//!    call void @vp_closure_cell_set(%cell, i8* %value_ptr)
//!    ```
//!
//! 2. Nested function receives cell as parameter:
//!    ```llvm
//!    define void @nested(i8* %cell) {
//!      %value_ptr = call i8* @vp_closure_cell_get(%cell)
//!      %value = load i64, i64* %value_ptr
//!    }
//!    ```

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{PointerValue, BasicValueEnum};

/// Declare closure cell runtime functions
pub fn declare_closure_cell_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let void_type = context.void_type();
    let _i8_type = context.i8_type();
    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = context.i64_type();

    // vp_closure_cell_create() -> i8*
    // Creates a new closure cell
    let create_fn_type = i8_ptr_type.fn_type(&[], false);
    module.add_function("vp_closure_cell_create", create_fn_type, None);

    // vp_closure_cell_free(i8* cell)
    // Frees a closure cell
    let free_fn_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    module.add_function("vp_closure_cell_free", free_fn_type, None);

    // vp_closure_cell_set(i8* cell, i8* value_ptr)
    // Sets the value pointer in a cell
    let set_fn_type = void_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
    module.add_function("vp_closure_cell_set", set_fn_type, None);

    // vp_closure_cell_get(i8* cell) -> i8*
    // Gets the value pointer from a cell
    let get_fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], false);
    module.add_function("vp_closure_cell_get", get_fn_type, None);

    // vp_closure_cell_set_i64(i8* cell, i64 value)
    // Sets an i64 value in a cell (convenience function)
    let set_i64_fn_type = void_type.fn_type(&[i8_ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_closure_cell_set_i64", set_i64_fn_type, None);

    // vp_closure_cell_get_i64(i8* cell) -> i64
    // Gets an i64 value from a cell (convenience function)
    let get_i64_fn_type = i64_type.fn_type(&[i8_ptr_type.into()], false);
    module.add_function("vp_closure_cell_get_i64", get_i64_fn_type, None);

    // vp_closure_cell_set_f64(i8* cell, double value)
    // Sets an f64 value in a cell (convenience function)
    let f64_type = context.f64_type();
    let set_f64_fn_type = void_type.fn_type(&[i8_ptr_type.into(), f64_type.into()], false);
    module.add_function("vp_closure_cell_set_f64", set_f64_fn_type, None);

    // vp_closure_cell_get_f64(i8* cell) -> double
    // Gets an f64 value from a cell (convenience function)
    let get_f64_fn_type = f64_type.fn_type(&[i8_ptr_type.into()], false);
    module.add_function("vp_closure_cell_get_f64", get_f64_fn_type, None);

    Ok(())
}

/// Create a closure cell for a variable
pub fn create_closure_cell<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    value_ptr: PointerValue<'ctx>,
    _var_name: &str,
) -> Result<PointerValue<'ctx>, String> {
    let create_fn = module
        .get_function("vp_closure_cell_create")
        .ok_or("vp_closure_cell_create function not declared")?;

    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // Create the cell
    let call_result = builder
        .build_call(create_fn, &[], "closure_cell")
        .expect("call closure cell create");
    
    let cell_ptr = match call_result.try_as_basic_value() {
        inkwell::values::ValueKind::Basic(bv) => bv.into_pointer_value(),
        _ => return Err("closure cell create didn't return a value".to_string()),
    };

    // Cast the value pointer to i8* for storage in the cell
    let value_ptr_cast = builder
        .build_pointer_cast(value_ptr, i8_ptr_type, "cast_to_i8")
        .expect("cast value ptr");

    // Set the value pointer in the cell
    let set_fn = module
        .get_function("vp_closure_cell_set")
        .ok_or("vp_closure_cell_set function not declared")?;

    builder
        .build_call(set_fn, &[cell_ptr.into(), value_ptr_cast.into()], "")
        .expect("call closure cell set");

    Ok(cell_ptr)
}

/// Get the value pointer from a closure cell
pub fn get_closure_cell_value<'ctx>(
    _context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    cell_ptr: PointerValue<'ctx>,
    value_type: inkwell::types::PointerType<'ctx>,
) -> Result<PointerValue<'ctx>, String> {
    let get_fn = module
        .get_function("vp_closure_cell_get")
        .ok_or("vp_closure_cell_get function not declared")?;

    let call_result = builder
        .build_call(get_fn, &[cell_ptr.into()], "cell_get")
        .expect("call closure cell get");
    
    let value_ptr_i8 = match call_result.try_as_basic_value() {
        inkwell::values::ValueKind::Basic(bv) => bv.into_pointer_value(),
        _ => return Err("closure cell get didn't return a value".to_string()),
    };

    // Cast back to the actual value type
    let value_ptr = builder
        .build_pointer_cast(value_ptr_i8, value_type, "cast_to_value")
        .expect("cast to value type");

    Ok(value_ptr)
}

/// Load a value from a closure cell
pub fn load_from_closure_cell<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    cell_ptr: PointerValue<'ctx>,
    value_type: inkwell::types::BasicTypeEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let value_ptr = get_closure_cell_value(context, module, builder, cell_ptr, context.ptr_type(inkwell::AddressSpace::default()))?;
    
    let value = builder
        .build_load(value_type, value_ptr, "cell_load")
        .expect("load from cell");

    Ok(value)
}

/// Store a value to a closure cell
pub fn store_to_closure_cell<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    cell_ptr: PointerValue<'ctx>,
    value: BasicValueEnum<'ctx>,
) -> Result<(), String> {
    let _value_type = value.get_type();
    let value_ptr = get_closure_cell_value(context, module, builder, cell_ptr, context.ptr_type(inkwell::AddressSpace::default()))?;
    
    builder
        .build_store(value_ptr, value)
        .expect("store to cell");

    Ok(())
}

/// Free a closure cell
pub fn free_closure_cell<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    cell_ptr: PointerValue<'ctx>,
) -> Result<(), String> {
    let free_fn = module
        .get_function("vp_closure_cell_free")
        .ok_or("vp_closure_cell_free function not declared")?;

    builder
        .build_call(free_fn, &[cell_ptr.into()], "")
        .expect("call closure cell free");

    Ok(())
}
