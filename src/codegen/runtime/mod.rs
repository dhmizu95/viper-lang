//! Runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

pub mod bigint;
pub mod concurrency;
pub mod dicts;
pub mod lists;
pub mod math;
pub mod memory;
pub mod print;

pub use bigint::declare_bigint_functions;
pub use concurrency::declare_concurrency_functions;
pub use dicts::declare_dict_functions;
pub use lists::declare_list_functions;
pub use math::{declare_hash_functions, declare_math_functions};
pub use memory::declare_memory_functions;
pub use print::declare_print_functions;

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
    Ok(())
}
