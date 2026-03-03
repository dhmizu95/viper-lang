use inkwell::context::Context;
use inkwell::module::Module;

/// Declare tuple runtime functions
pub fn declare_tuple_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // vp_tuple_create(size: i64) -> ViperTuple*
    let tuple_create_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_tuple_create", tuple_create_type, None);

    // vp_tuple_free(tuple: ViperTuple*) -> void
    let tuple_free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_free", tuple_free_type, None);

    // vp_tuple_get(tuple: ViperTuple*, index: i64) -> i64
    let tuple_get_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_tuple_get", tuple_get_type, None);

    // vp_tuple_set(tuple: ViperTuple*, index: i64, value: i64) -> void
    let tuple_set_type = void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_tuple_set", tuple_set_type, None);

    // vp_tuple_len(tuple: ViperTuple*) -> i64
    let tuple_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_len", tuple_len_type, None);

    // vp_tuple_hash(tuple: ViperTuple*) -> i64
    let tuple_hash_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_hash", tuple_hash_type, None);

    // vp_tuple_slice(tuple: ViperTuple*, start: i64, end: i64) -> ViperTuple*
    let tuple_slice_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_tuple_slice", tuple_slice_type, None);

    // vp_tuple_concat(a: ViperTuple*, b: ViperTuple*) -> ViperTuple*
    let tuple_concat_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_tuple_concat", tuple_concat_type, None);

    // vp_tuple_count(tuple: ViperTuple*, value: i64) -> i64
    let tuple_count_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_tuple_count", tuple_count_type, None);

    // vp_tuple_index(tuple: ViperTuple*, value: i64) -> i64
    let tuple_index_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_tuple_index", tuple_index_type, None);

    // vp_tuple_from_list(src: ViperList*) -> ViperTuple*
    let tuple_from_list_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_from_list", tuple_from_list_type, None);

    // vp_tuple_from_str(str: ViperString*) -> ViperTuple*
    let tuple_from_str_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_from_str", tuple_from_str_type, None);

    // vp_tuple_copy(src: ViperTuple*) -> ViperTuple*
    let tuple_copy_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_copy", tuple_copy_type, None);

    Ok(())
}
