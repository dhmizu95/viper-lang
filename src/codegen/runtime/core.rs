//! Runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare all runtime library functions
pub fn declare_runtime_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    super::print::declare_print_functions(context, module)?;
    super::lists::declare_list_functions(context, module)?;
    super::dicts::declare_dict_functions(context, module)?;
    super::memory::declare_memory_functions(context, module)?;
    super::math::declare_math_functions(context, module)?;
    super::math::declare_hash_functions(context, module)?;
    super::concurrency::declare_concurrency_functions(context, module)?;
    super::bigint::declare_bigint_functions(context, module)?;
    declare_panic_function(context, module)?;
    super::exceptions::declare_exception_functions(context, module)?;
    super::closure_cells::declare_closure_cell_functions(context, module)?;
    super::tuples::declare_tuple_functions(context, module)?;
    super::tagged_int::declare_tagged_int_functions(context, module)?;
    super::json::declare_json_functions(context, module)?;
    super::re::declare_re_functions(context, module)?;
    super::random::declare_random_functions(context, module)?;
    super::logging::declare_logging_functions(context, module)?;
    super::typing::declare_typing_functions(context, module)?;
    super::iterator::declare_iterator_functions(context, module)?;
    Ok(())
}

/// Declare panic function for assertion failures
fn declare_panic_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let fn_type = context
        .void_type()
        .fn_type(&[context.ptr_type(inkwell::AddressSpace::default()).into()], false);
    module.add_function("viper_panic", fn_type, Some(inkwell::module::Linkage::External));
    Ok(())
}
