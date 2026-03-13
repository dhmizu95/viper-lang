//! Exception handling runtime for Viper
//!
//! This module provides runtime support for Viper's exception handling mechanism.
//! It uses LLVM's exception handling infrastructure with Itanium ABI.

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

/// Declare exception handling runtime functions
pub fn declare_exception_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    // __cxa_allocate_exception - allocate memory for an exception object
    let allocate_type =
        context.ptr_type(AddressSpace::default()).fn_type(&[context.i64_type().into()], false);
    module.add_function(
        "__cxa_allocate_exception",
        allocate_type,
        Some(inkwell::module::Linkage::External),
    );

    // __cxa_free_exception - free exception object memory
    let free_type =
        context.void_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function(
        "__cxa_free_exception",
        free_type,
        Some(inkwell::module::Linkage::External),
    );

    // __cxa_throw - throw an exception
    let throw_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // exception object
            context.ptr_type(AddressSpace::default()).into(), // typeinfo
            context.ptr_type(AddressSpace::default()).into(), // destructor
        ],
        false,
    );
    module.add_function("__cxa_throw", throw_type, Some(inkwell::module::Linkage::External));

    // __cxa_begin_catch - begin catching an exception
    let begin_catch_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function(
        "__cxa_begin_catch",
        begin_catch_type,
        Some(inkwell::module::Linkage::External),
    );

    // __cxa_end_catch - end catching an exception
    let end_catch_type = context.void_type().fn_type(&[], false);
    module.add_function(
        "__cxa_end_catch",
        end_catch_type,
        Some(inkwell::module::Linkage::External),
    );

    // __cxa_get_exception_ptr - get pointer to thrown exception
    let get_exception_ptr_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function(
        "__cxa_get_exception_ptr",
        get_exception_ptr_type,
        Some(inkwell::module::Linkage::External),
    );

    // __gxx_personality_v0 - personality function for stack unwinding
    let personality_type = context.i32_type().fn_type(
        &[
            context.i32_type().into(),
            context.i32_type().into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function(
        "__gxx_personality_v0",
        personality_type,
        Some(inkwell::module::Linkage::External),
    );

    // Viper-specific exception functions
    declare_viper_exception_functions(context, module)?;

    Ok(())
}

/// Declare Viper-specific exception handling functions
fn declare_viper_exception_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let ptr_type = context.ptr_type(AddressSpace::default());

    // viper_raise_exception - raise a Viper exception with type and message
    let raise_type = context.void_type().fn_type(
        &[
            ptr_type.into(), // exception type name
            ptr_type.into(), // message
        ],
        false,
    );
    module.add_function(
        "viper_raise_exception",
        raise_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_raise_with_code - raise exception with error code
    let raise_code_type = context.void_type().fn_type(
        &[
            ptr_type.into(),           // exception type name
            ptr_type.into(),           // message
            context.i64_type().into(), // error code
        ],
        false,
    );
    module.add_function(
        "viper_raise_with_code",
        raise_code_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_raise_with_cause - raise exception with cause (raise X from Y)
    let raise_cause_type = context.void_type().fn_type(
        &[
            ptr_type.into(),           // exception type name
            ptr_type.into(),           // message
            context.i64_type().into(), // error code
            ptr_type.into(),           // cause type name
            ptr_type.into(),           // cause message
        ],
        false,
    );
    module.add_function(
        "viper_raise_with_cause",
        raise_cause_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_catch_exception - check if current exception matches type
    let catch_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[ptr_type.into()], // exception type name to match
        false,
    );
    module.add_function(
        "viper_catch_exception",
        catch_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_get_exception_type - get the type of current exception
    let get_type_type = ptr_type.fn_type(&[], false);
    module.add_function(
        "viper_get_exception_type",
        get_type_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_get_exception_message - get the message of current exception
    let get_msg_type = ptr_type.fn_type(&[], false);
    module.add_function(
        "viper_get_exception_message",
        get_msg_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_get_exception_code - get the error code of current exception
    let get_code_type = context.i64_type().fn_type(&[], false);
    module.add_function(
        "viper_get_exception_code",
        get_code_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_clear_exception - clear the current exception
    let clear_type = context.void_type().fn_type(&[], false);
    module.add_function(
        "viper_clear_exception",
        clear_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_set_exception - set the current exception (for re-raising)
    let set_exc_type = context.void_type().fn_type(
        &[
            ptr_type.into(),           // type
            ptr_type.into(),           // message
            context.i64_type().into(), // code
        ],
        false,
    );
    module.add_function(
        "viper_set_exception",
        set_exc_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_format_exception - format exception info as string
    let format_type = ptr_type.fn_type(&[], false);
    module.add_function(
        "viper_format_exception",
        format_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_print_traceback - print stack trace for current exception
    let traceback_type = context.void_type().fn_type(&[], false);
    module.add_function(
        "viper_print_traceback",
        traceback_type,
        Some(inkwell::module::Linkage::External),
    );

    // viper_exception_matches - check if exception type matches (supports inheritance)
    let matches_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            ptr_type.into(), // actual type
            ptr_type.into(), // expected type
        ],
        false,
    );
    module.add_function(
        "viper_exception_matches",
        matches_type,
        Some(inkwell::module::Linkage::External),
    );

    Ok(())
}
