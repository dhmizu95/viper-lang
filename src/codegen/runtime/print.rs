use inkwell::context::Context;
use inkwell::module::Module;

/// Declare print-related runtime functions
pub fn declare_print_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let f64_type = context.f64_type();
    let bool_type = context.bool_type();

    let print_i64_type = void_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_print_i64", print_i64_type, None);

    let print_f64_type = void_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_print_f64", print_f64_type, None);

    let print_str_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_print_str", print_str_type, None);

    let print_bool_type = void_type.fn_type(&[bool_type.into()], false);
    module.add_function("vp_print_bool", print_bool_type, None);

    let print_newline_type = void_type.fn_type(&[], false);
    module.add_function("vp_print_newline", print_newline_type, None);

    // String creation function
    let str_create_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_str_create", str_create_type, None);

    // String concatenation function
    let str_concat_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_str_concat", str_concat_type, None);

    // String conversion functions
    let str_from_i64_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_str_from_i64", str_from_i64_type, None);

    let str_from_f64_type = ptr_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_str_from_f64", str_from_f64_type, None);

    let str_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_str_len", str_len_type, None);

    let str_to_i64_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_str_to_i64", str_to_i64_type, None);

    let str_create_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_str_create", str_create_type, None);

    let str_upper_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_str_upper", str_upper_type, None);

    let str_lower_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_str_lower", str_lower_type, None);

    let str_split_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_str_split", str_split_type, None);

    let str_replace_type =
        ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_str_replace", str_replace_type, None);

    // Bytes functions
    let bytes_create_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_bytes_create", bytes_create_type, None);

    let bytes_free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bytes_free", bytes_free_type, None);

    let bytes_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bytes_len", bytes_len_type, None);

    let bytes_get_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_bytes_get", bytes_get_type, None);

    let bytes_print_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bytes_print", bytes_print_type, None);

    Ok(())
}
