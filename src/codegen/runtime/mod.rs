//! Runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

pub mod print;
pub mod lists;
pub mod dicts;
pub mod memory;
pub mod math;
pub mod concurrency;
pub mod bigint;
pub mod exceptions;
pub mod closure_cells;

pub use print::declare_print_functions;
pub use lists::declare_list_functions;
pub use dicts::declare_dict_functions;
pub use memory::declare_memory_functions;
pub use math::{declare_math_functions, declare_hash_functions};
pub use concurrency::declare_concurrency_functions;
pub use bigint::declare_bigint_functions;
pub use exceptions::declare_exception_functions;
pub use closure_cells::declare_closure_cell_functions;

/// Declare all runtime library functions
pub fn declare_runtime_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    declare_print_functions(context, module)?;
    declare_list_functions(context, module)?;
    declare_dict_functions(context, module)?;
    declare_memory_functions(context, module)?;
    declare_math_functions(context, module)?;
    declare_hash_functions(context, module)?;
    declare_concurrency_functions(context, module)?;
    declare_bigint_functions(context, module)?;
    declare_panic_function(context, module)?;
    declare_exception_functions(context, module)?;
    declare_closure_cell_functions(context, module)?;
    Ok(())
}

/// Declare panic function for assertion failures
fn declare_panic_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let fn_type = context.void_type().fn_type(
        &[context.ptr_type(inkwell::AddressSpace::default()).into()],
        false,
    );
    module.add_function("viper_panic", fn_type, Some(inkwell::module::Linkage::External));
    Ok(())
}
