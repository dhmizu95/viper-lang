//! Closure Cell Runtime Implementation
//!
//! Provides the actual implementation of closure cell operations.
//! Closure cells are heap-allocated boxes that contain a pointer to the actual value.

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare and generate closure cell runtime functions
pub fn declare_closure_cell_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let void_type = context.void_type();
    let _i8_type = context.i8_type();
    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = context.i64_type();
    let f64_type = context.f64_type();

    // Declare vp_malloc and vp_free if they don't exist
    let malloc_fn = module.get_function("vp_malloc").unwrap_or_else(|| {
        let malloc_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
        module.add_function("vp_malloc", malloc_type, None)
    });

    let free_fn = module.get_function("vp_free").unwrap_or_else(|| {
        let free_type = void_type.fn_type(&[i8_ptr_type.into()], false);
        module.add_function("vp_free", free_type, None)
    });

    // Closure cell structure: { i8* } (just a pointer to the value)
    let cell_struct_type = context.struct_type(&[i8_ptr_type.into()], false);
    let cell_ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // vp_closure_cell_create() -> i8*
    // Creates a new closure cell (returns pointer to cell structure)
    let create_fn_type = i8_ptr_type.fn_type(&[], false);
    let create_fn = module.add_function("vp_closure_cell_create", create_fn_type, None);

    let entry = context.append_basic_block(create_fn, "entry");
    let builder = context.create_builder();
    builder.position_at_end(entry);

    // Allocate memory for the cell structure
    let cell_size = i64_type.const_int(8, false); // Size of a pointer on 64-bit
    let call_result =
        builder.build_call(malloc_fn, &[cell_size.into()], "cell_raw").expect("call malloc");
    let cell_raw = match call_result.try_as_basic_value() {
        inkwell::values::ValueKind::Basic(bv) => bv.into_pointer_value(),
        _ => return crate::codegen::codegen_error("malloc didn't return a value".to_string()),
    };

    // Initialize the cell with null pointer
    let cell_ptr =
        builder.build_pointer_cast(cell_raw, cell_ptr_type, "cell").expect("cast to cell");
    let null_ptr = i8_ptr_type.const_null();
    builder
        .build_store(
            builder
                .build_struct_gep(cell_struct_type, cell_ptr, 0, "cell_value_ptr")
                .expect("gep into cell"),
            null_ptr,
        )
        .expect("init cell");

    builder.build_return(Some(&cell_raw)).expect("return cell");

    // vp_closure_cell_free(i8* cell)
    // Frees a closure cell
    let free_fn_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    let free_cell_fn = module.add_function("vp_closure_cell_free", free_fn_type, None);

    let entry = context.append_basic_block(free_cell_fn, "entry");
    builder.position_at_end(entry);

    let cell_arg = free_cell_fn.get_nth_param(0).expect("cell arg");

    // Free the cell structure
    builder.build_call(free_fn, &[cell_arg.into()], "").expect("call free");

    builder.build_return(None).expect("return void");

    // vp_closure_cell_set(i8* cell, i8* value_ptr)
    // Sets the value pointer in a cell
    let set_fn_type = void_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
    let set_fn = module.add_function("vp_closure_cell_set", set_fn_type, None);

    let entry = context.append_basic_block(set_fn, "entry");
    builder.position_at_end(entry);

    let cell_arg = set_fn.get_nth_param(0).expect("cell arg").into_pointer_value();
    let value_ptr_arg = set_fn.get_nth_param(1).expect("value_ptr arg").into_pointer_value();

    let cell_ptr =
        builder.build_pointer_cast(cell_arg, cell_ptr_type, "cell").expect("cast to cell");
    let value_gep = builder
        .build_struct_gep(cell_struct_type, cell_ptr, 0, "cell_value")
        .expect("gep into cell");
    builder.build_store(value_gep, value_ptr_arg).expect("store value ptr");

    builder.build_return(None).expect("return void");

    // vp_closure_cell_get(i8* cell) -> i8*
    // Gets the value pointer from a cell
    let get_fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], false);
    let get_fn = module.add_function("vp_closure_cell_get", get_fn_type, None);

    let entry = context.append_basic_block(get_fn, "entry");
    builder.position_at_end(entry);

    let cell_arg = get_fn.get_nth_param(0).expect("cell arg").into_pointer_value();

    let cell_ptr =
        builder.build_pointer_cast(cell_arg, cell_ptr_type, "cell").expect("cast to cell");
    let value_gep = builder
        .build_struct_gep(cell_struct_type, cell_ptr, 0, "cell_value")
        .expect("gep into cell");
    let value_ptr = builder
        .build_load(i8_ptr_type, value_gep, "value_ptr")
        .expect("load value ptr")
        .into_pointer_value();

    builder.build_return(Some(&value_ptr)).expect("return value ptr");

    // vp_closure_cell_set_i64(i8* cell, i64 value)
    // Sets an i64 value in a cell (allocates space and stores)
    let set_i64_fn_type = void_type.fn_type(&[i8_ptr_type.into(), i64_type.into()], false);
    let set_i64_fn = module.add_function("vp_closure_cell_set_i64", set_i64_fn_type, None);

    let entry = context.append_basic_block(set_i64_fn, "entry");
    builder.position_at_end(entry);

    let cell_arg = set_i64_fn.get_nth_param(0).expect("cell arg").into_pointer_value();
    let value_arg = set_i64_fn.get_nth_param(1).expect("value arg");

    // Allocate space for the i64 value
    let value_size = i64_type.const_int(8, false);
    let call_result =
        builder.build_call(malloc_fn, &[value_size.into()], "value_raw").expect("call malloc");
    let value_raw = match call_result.try_as_basic_value() {
        inkwell::values::ValueKind::Basic(bv) => bv.into_pointer_value(),
        _ => return crate::codegen::codegen_error("malloc didn't return a value".to_string()),
    };

    let value_ptr = builder
        .build_pointer_cast(
            value_raw,
            context.ptr_type(inkwell::AddressSpace::default()),
            "value_ptr",
        )
        .expect("cast to i64*");
    builder.build_store(value_ptr, value_arg).expect("store value");

    // Cast value pointer to i8* and store in cell
    let value_ptr_i8 =
        builder.build_pointer_cast(value_ptr, i8_ptr_type, "value_ptr_i8").expect("cast to i8*");

    let cell_ptr =
        builder.build_pointer_cast(cell_arg, cell_ptr_type, "cell").expect("cast to cell");
    let value_gep = builder
        .build_struct_gep(cell_struct_type, cell_ptr, 0, "cell_value")
        .expect("gep into cell");
    builder.build_store(value_gep, value_ptr_i8).expect("store value ptr");

    builder.build_return(None).expect("return void");

    // vp_closure_cell_get_i64(i8* cell) -> i64
    // Gets an i64 value from a cell
    let get_i64_fn_type = i64_type.fn_type(&[i8_ptr_type.into()], false);
    let get_i64_fn = module.add_function("vp_closure_cell_get_i64", get_i64_fn_type, None);

    let entry = context.append_basic_block(get_i64_fn, "entry");
    builder.position_at_end(entry);

    let cell_arg = get_i64_fn.get_nth_param(0).expect("cell arg").into_pointer_value();

    let cell_ptr =
        builder.build_pointer_cast(cell_arg, cell_ptr_type, "cell").expect("cast to cell");
    let value_gep = builder
        .build_struct_gep(cell_struct_type, cell_ptr, 0, "cell_value")
        .expect("gep into cell");
    let value_ptr_i8 = builder
        .build_load(i8_ptr_type, value_gep, "value_ptr_i8")
        .expect("load value ptr i8")
        .into_pointer_value();

    let value_ptr = builder
        .build_pointer_cast(
            value_ptr_i8,
            context.ptr_type(inkwell::AddressSpace::default()),
            "value_ptr",
        )
        .expect("cast to i64*");
    let value = builder.build_load(i64_type, value_ptr, "value").expect("load value");

    builder.build_return(Some(&value)).expect("return value");

    // vp_closure_cell_set_f64(i8* cell, double value)
    // Sets an f64 value in a cell
    let set_f64_fn_type = void_type.fn_type(&[i8_ptr_type.into(), f64_type.into()], false);
    let set_f64_fn = module.add_function("vp_closure_cell_set_f64", set_f64_fn_type, None);

    let entry = context.append_basic_block(set_f64_fn, "entry");
    builder.position_at_end(entry);

    let cell_arg = set_f64_fn.get_nth_param(0).expect("cell arg").into_pointer_value();
    let value_arg = set_f64_fn.get_nth_param(1).expect("value arg");

    // Allocate space for the f64 value
    let value_size = i64_type.const_int(8, false);
    let call_result =
        builder.build_call(malloc_fn, &[value_size.into()], "value_raw").expect("call malloc");
    let value_raw = match call_result.try_as_basic_value() {
        inkwell::values::ValueKind::Basic(bv) => bv.into_pointer_value(),
        _ => return crate::codegen::codegen_error("malloc didn't return a value".to_string()),
    };

    let value_ptr = builder
        .build_pointer_cast(
            value_raw,
            context.ptr_type(inkwell::AddressSpace::default()),
            "value_ptr",
        )
        .expect("cast to f64*");
    builder.build_store(value_ptr, value_arg).expect("store value");

    // Cast value pointer to i8* and store in cell
    let value_ptr_i8 =
        builder.build_pointer_cast(value_ptr, i8_ptr_type, "value_ptr_i8").expect("cast to i8*");

    let cell_ptr =
        builder.build_pointer_cast(cell_arg, cell_ptr_type, "cell").expect("cast to cell");
    let value_gep = builder
        .build_struct_gep(cell_struct_type, cell_ptr, 0, "cell_value")
        .expect("gep into cell");
    builder.build_store(value_gep, value_ptr_i8).expect("store value ptr");

    builder.build_return(None).expect("return void");

    // vp_closure_cell_get_f64(i8* cell) -> double
    // Gets an f64 value from a cell
    let get_f64_fn_type = f64_type.fn_type(&[i8_ptr_type.into()], false);
    let get_f64_fn = module.add_function("vp_closure_cell_get_f64", get_f64_fn_type, None);

    let entry = context.append_basic_block(get_f64_fn, "entry");
    builder.position_at_end(entry);

    let cell_arg = get_f64_fn.get_nth_param(0).expect("cell arg").into_pointer_value();

    let cell_ptr =
        builder.build_pointer_cast(cell_arg, cell_ptr_type, "cell").expect("cast to cell");
    let value_gep = builder
        .build_struct_gep(cell_struct_type, cell_ptr, 0, "cell_value")
        .expect("gep into cell");
    let value_ptr_i8 = builder
        .build_load(i8_ptr_type, value_gep, "value_ptr_i8")
        .expect("load value ptr i8")
        .into_pointer_value();

    let value_ptr = builder
        .build_pointer_cast(
            value_ptr_i8,
            context.ptr_type(inkwell::AddressSpace::default()),
            "value_ptr",
        )
        .expect("cast to f64*");
    let value = builder.build_load(f64_type, value_ptr, "value").expect("load value");

    builder.build_return(Some(&value)).expect("return value");

    Ok(())
}
