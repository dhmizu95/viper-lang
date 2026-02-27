use inkwell::context::Context;
use inkwell::module::Module;

/// Declare dict-related runtime functions
pub fn declare_dict_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let bool_type = context.bool_type();

    let dict_create_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_dict_create", dict_create_type, None);

    let dict_free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_dict_free", dict_free_type, None);

    let dict_set_type =
        void_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_dict_set_i64", dict_set_type, None);

    /* Dict set with ViperString key */
    let dict_set_str_i64_type =
        void_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_dict_set_str_i64", dict_set_str_i64_type, None);

    let dict_set_str_str_type =
        void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_dict_set_str_str", dict_set_str_str_type, None);

    let dict_get_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_dict_get_i64", dict_get_type, None);

    let dict_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_dict_len", dict_len_type, None);

    let dict_contains_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_dict_contains", dict_contains_type, None);

    let dict_remove_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_dict_remove", dict_remove_type, None);

    let dict_clear_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_dict_clear", dict_clear_type, None);

    // Dict print function
    let dict_print_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_dict_print", dict_print_type, None);

    Ok(())
}
