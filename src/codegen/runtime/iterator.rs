//! Iterator runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare iterator runtime functions
pub fn declare_iterator_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let _ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    
    // Create struct type for iterator result: { value: i64, done: bool }
    let result_struct_type = context.struct_type(&[
        context.i64_type().into(),
        context.bool_type().into(),
    ], false);
    
    // vp_iterator_next(iterator_ptr) -> { value: i64, done: i1 }
    // Returns struct with value and done flag
    let iterator_next_type = context
        .ptr_type(inkwell::AddressSpace::default())
        .fn_type(&[result_struct_type.into()], false);
    module.add_function("vp_iterator_next", iterator_next_type, None);
    
    Ok(())
}
