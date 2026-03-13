//! Typing module runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare typing module runtime functions
pub fn declare_typing_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let _i64_type = context.i64_type();
    let bool_type = context.bool_type();

    // get_type_hints(obj) -> dict
    let get_type_hints_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_typing_get_type_hints", get_type_hints_type, None);

    // get_origin(tp) -> type or None
    let get_origin_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_typing_get_origin", get_origin_type, None);

    // get_args(tp) -> tuple
    let get_args_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_typing_get_args", get_args_type, None);

    // is_generic_type(tp) -> bool
    let is_generic_type = bool_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_typing_is_generic_type", is_generic_type, None);

    // TypeVar constructor: TypeVar(name, bound, covariant, contravariant)
    let typevar_type = ptr_type.fn_type(&[
        ptr_type.into(),  // name
        ptr_type.into(),  // bound (can be None)
        bool_type.into(), // covariant
        bool_type.into(), // contravariant
    ], false);
    module.add_function("vp_typing_typevar_new", typevar_type, None);

    Ok(())
}
