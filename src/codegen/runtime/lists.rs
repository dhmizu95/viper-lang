use inkwell::context::Context;
use inkwell::module::Module;

/// Declare list-related runtime functions
pub fn declare_list_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let i64_type = context.i64_type();
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let bool_type = context.bool_type();

    let list_create_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_list_create", list_create_type, None);

    // List with pre-allocated capacity
    let list_create_cap_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_list_create_with_capacity", list_create_cap_type, None);

    let list_append_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    let list_append = module.add_function("vp_list_append", list_append_type, None);
    // Add alwaysinline hint for better performance on hot path
    list_append.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );

    // List grow function - for inline append
    let list_grow_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_grow", list_grow_type, None);

    // List reserve function - pre-allocate capacity
    let list_reserve_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_reserve", list_reserve_type, None);

    let list_free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_free", list_free_type, None);

    let list_get_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_get", list_get_type, None);

    let list_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_len", list_len_type, None);

    let list_slice_type = ptr_type
        .fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_list_slice", list_slice_type, None);

    let list_set_type =
        void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_list_set", list_set_type, None);

    let list_insert_type =
        void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_list_insert", list_insert_type, None);

    let list_remove_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_remove", list_remove_type, None);

    let list_pop_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_pop", list_pop_type, None);

    let list_clear_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_clear", list_clear_type, None);

    let list_contains_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_contains", list_contains_type, None);

    let list_copy_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_copy", list_copy_type, None);

    // Float list functions (f64)
    let f64_type = context.f64_type();

    let list_create_f64_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_list_create_f64", list_create_f64_type, None);

    let list_append_f64_type = void_type.fn_type(&[ptr_type.into(), f64_type.into()], false);
    module.add_function("vp_list_append_f64", list_append_f64_type, None);

    let list_get_f64_type = f64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_get_f64", list_get_f64_type, None);

    let list_set_f64_type =
        void_type.fn_type(&[ptr_type.into(), i64_type.into(), f64_type.into()], false);
    module.add_function("vp_list_set_f64", list_set_f64_type, None);

    // List repeat function (list * int)
    let list_repeat_type = ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("vp_list_repeat", list_repeat_type, None);

    // List print function
    let list_print_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_print", list_print_type, None);

    // Extended list operations
    let list_extend_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_list_extend", list_extend_type, None);

    let list_index_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_index", list_index_type, None);

    let list_count_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_count", list_count_type, None);

    let list_sort_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_sort", list_sort_type, None);

    let list_reverse_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_reverse", list_reverse_type, None);

    let list_reversed_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_reversed", list_reversed_type, None);

    let list_sorted_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_sorted", list_sorted_type, None);

    let list_concat_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_list_concat", list_concat_type, None);

    // Bool list functions (type-specific, memory efficient)
    let list_bool_create_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_list_bool_create", list_bool_create_type, None);

    let list_bool_create_cap_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_list_bool_create_with_capacity", list_bool_create_cap_type, None);

    let list_bool_append_type = void_type.fn_type(&[ptr_type.into(), bool_type.into()], false);
    let list_bool_append = module.add_function("vp_list_bool_append", list_bool_append_type, None);
    list_bool_append.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );

    let list_bool_free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_bool_free", list_bool_free_type, None);

    let list_bool_get_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_bool_get", list_bool_get_type, None);

    let list_bool_set_type =
        void_type.fn_type(&[ptr_type.into(), i64_type.into(), bool_type.into()], false);
    module.add_function("vp_list_bool_set", list_bool_set_type, None);

    let list_bool_insert_type =
        void_type.fn_type(&[ptr_type.into(), i64_type.into(), bool_type.into()], false);
    module.add_function("vp_list_bool_insert", list_bool_insert_type, None);

    let list_bool_remove_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_bool_remove", list_bool_remove_type, None);

    let list_bool_pop_type = bool_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_bool_pop", list_bool_pop_type, None);

    let list_bool_clear_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_bool_clear", list_bool_clear_type, None);

    let list_bool_contains_type = bool_type.fn_type(&[ptr_type.into(), bool_type.into()], false);
    module.add_function("vp_list_bool_contains", list_bool_contains_type, None);

    let list_bool_copy_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_bool_copy", list_bool_copy_type, None);

    let list_bool_repeat_type = ptr_type.fn_type(&[bool_type.into(), i64_type.into()], false);
    module.add_function("vp_list_bool_repeat", list_bool_repeat_type, None);

    let list_bool_init_stack_type = void_type
        .fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into(), bool_type.into()], false);
    module.add_function("vp_list_bool_init_stack", list_bool_init_stack_type, None);

    let list_bool_slice_type = ptr_type
        .fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_list_bool_slice", list_bool_slice_type, None);

    let list_bool_print_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_bool_print", list_bool_print_type, None);

    let list_bool_extend_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_list_bool_extend", list_bool_extend_type, None);

    let list_bool_index_type = i64_type.fn_type(&[ptr_type.into(), bool_type.into()], false);
    module.add_function("vp_list_bool_index", list_bool_index_type, None);

    let list_bool_count_type = i64_type.fn_type(&[ptr_type.into(), bool_type.into()], false);
    module.add_function("vp_list_bool_count", list_bool_count_type, None);

    let list_bool_reverse_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_bool_reverse", list_bool_reverse_type, None);

    let list_bool_reversed_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_bool_reversed", list_bool_reversed_type, None);

    let list_bool_concat_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_list_bool_concat", list_bool_concat_type, None);

    // Bit vector functions (1 bit per boolean - 8x memory savings)
    let bitvec_create_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_bitvec_create", bitvec_create_type, None);

    let bitvec_create_cap_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_bitvec_create_with_capacity", bitvec_create_cap_type, None);

    let bitvec_repeat_type = ptr_type.fn_type(&[bool_type.into(), i64_type.into()], false);
    let bitvec_repeat = module.add_function("vp_bitvec_repeat", bitvec_repeat_type, None);
    bitvec_repeat.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );

    let bitvec_free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bitvec_free", bitvec_free_type, None);

    let bitvec_append_type = void_type.fn_type(&[ptr_type.into(), bool_type.into()], false);
    let bitvec_append = module.add_function("vp_bitvec_append", bitvec_append_type, None);
    bitvec_append.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );

    let bitvec_insert_type =
        void_type.fn_type(&[ptr_type.into(), i64_type.into(), bool_type.into()], false);
    module.add_function("vp_bitvec_insert", bitvec_insert_type, None);

    let bitvec_remove_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_bitvec_remove", bitvec_remove_type, None);

    let bitvec_pop_type = bool_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bitvec_pop", bitvec_pop_type, None);

    let bitvec_clear_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bitvec_clear", bitvec_clear_type, None);

    let bitvec_get_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_bitvec_get", bitvec_get_type, None);

    let bitvec_set_type =
        void_type.fn_type(&[ptr_type.into(), i64_type.into(), bool_type.into()], false);
    module.add_function("vp_bitvec_set", bitvec_set_type, None);

    /* Unchecked versions for hot loops - no bounds checking */
    let bitvec_get_unchecked_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_bitvec_get_unchecked", bitvec_get_unchecked_type, None);

    let bitvec_set_unchecked_type =
        void_type.fn_type(&[ptr_type.into(), i64_type.into(), bool_type.into()], false);
    module.add_function("vp_bitvec_set_unchecked", bitvec_set_unchecked_type, None);

    let bitvec_contains_type = bool_type.fn_type(&[ptr_type.into(), bool_type.into()], false);
    module.add_function("vp_bitvec_contains", bitvec_contains_type, None);

    let bitvec_copy_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bitvec_copy", bitvec_copy_type, None);

    let bitvec_slice_type = ptr_type
        .fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_bitvec_slice", bitvec_slice_type, None);

    let bitvec_print_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bitvec_print", bitvec_print_type, None);

    let bitvec_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bitvec_len", bitvec_len_type, None);

    let bitvec_extend_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bitvec_extend", bitvec_extend_type, None);

    let bitvec_index_type = i64_type.fn_type(&[ptr_type.into(), bool_type.into()], false);
    module.add_function("vp_bitvec_index", bitvec_index_type, None);

    let bitvec_count_type = i64_type.fn_type(&[ptr_type.into(), bool_type.into()], false);
    module.add_function("vp_bitvec_count", bitvec_count_type, None);

    let bitvec_reverse_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bitvec_reverse", bitvec_reverse_type, None);

    let bitvec_reversed_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bitvec_reversed", bitvec_reversed_type, None);

    let bitvec_concat_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bitvec_concat", bitvec_concat_type, None);

    // Range function: vp_range(start, end) returns a list
    let range_type = ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("vp_range", range_type, None);

    // list() builtin - create list from iterable
    let list_from_iter_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_from_iterable", list_from_iter_type, None);

    // list() from string - create list of character codes
    let list_from_str_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_from_str", list_from_str_type, None);

    // list() copy - create shallow copy of list
    let list_copy_from_list_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_copy_from_list", list_copy_from_list_type, None);

    // tuple() builtin - create tuple from iterable
    let tuple_from_iter_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_from_iterable", tuple_from_iter_type, None);

    // tuple() from list - create tuple from list
    let tuple_from_list_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_from_list", tuple_from_list_type, None);

    // tuple() from string - create tuple of character codes
    let tuple_from_str_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_from_str", tuple_from_str_type, None);

    // tuple() copy - create shallow copy of tuple
    let tuple_copy_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_tuple_copy", tuple_copy_type, None);

    // set() builtin - create set from iterable
    let set_from_iter_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_set_from_iterable", set_from_iter_type, None);

    // set() from list - create set from list
    let set_from_list_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_set_from_list", set_from_list_type, None);

    // set() copy - create shallow copy of set
    let set_copy_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_set_copy", set_copy_type, None);

    // set() add element
    let set_add_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_set_add", set_add_type, None);

    // set() contains
    let set_contains_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_set_contains", set_contains_type, None);

    // set() len
    let set_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_set_len", set_len_type, None);

    // set() print
    let set_print_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_set_print", set_print_type, None);

    // dict() builtin
    let dict_create_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_dict_create_empty", dict_create_type, None);

    // enumerate() - returns list of (index, value) tuples
    let enumerate_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_enumerate", enumerate_type, None);

    // zip() - returns list of paired elements
    let zip_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_zip", zip_type, None);

    // sum() - sum of list elements
    let sum_i64_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_sum", sum_i64_type, None);
    let sum_f64_type = f64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_sum_f64", sum_f64_type, None);

    // min/max - find min/max in list
    let min_i64_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_min", min_i64_type, None);
    let max_i64_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_max", max_i64_type, None);

    // any/all - boolean reduction
    let any_type = bool_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_any", any_type, None);
    let all_type = bool_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_all", all_type, None);

    // type() - get type name as string
    let type_of_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_type_of", type_of_type, None);

    // id() - get object identity (pointer as int)
    let id_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_object_id", id_type, None);

    // repr() - get string representation
    let repr_i64_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_repr_i64", repr_i64_type, None);
    let repr_f64_type = ptr_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_repr_f64", repr_f64_type, None);
    let repr_str_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_repr_str", repr_str_type, None);
    let repr_bool_type = ptr_type.fn_type(&[bool_type.into()], false);
    module.add_function("vp_repr_bool", repr_bool_type, None);

    // bin/oct/hex - number base conversion
    let bin_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_bin_i64", bin_type, None);
    let oct_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_oct_i64", oct_type, None);
    let hex_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_hex_i64", hex_type, None);

    // chr/ord - character conversion
    let chr_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_chr_i64", chr_type, None);
    let ord_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_ord_str", ord_type, None);

    // round() - round floating point
    let round_type = f64_type.fn_type(&[f64_type.into(), i64_type.into()], false);
    module.add_function("vp_round_f64", round_type, None);

    // divmod() - division with remainder
    let divmod_type = ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("vp_divmod_i64", divmod_type, None);

    // pow() - power function
    let pow_i64_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("vp_pow_i64", pow_i64_type, None);
    let pow_f64_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
    module.add_function("vp_pow_f64", pow_f64_type, None);

    // hasattr - check if object has attribute
    let hasattr_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_hasattr", hasattr_type, None);

    // callable - check if object is callable
    let callable_type = bool_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_is_callable", callable_type, None);

    // input - read from stdin
    let input_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_input", input_type, None);

    Ok(())
}

/// Declare bytearray runtime functions
pub fn declare_bytearray_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let i64_type = context.i64_type();
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // bytearray() constructor
    let bytearray_create_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_bytearray_create", bytearray_create_type, None);

    // bytearray with capacity
    let bytearray_create_cap_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_bytearray_create_with_capacity", bytearray_create_cap_type, None);

    // bytearray len
    let bytearray_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bytearray_len", bytearray_len_type, None);

    // bytearray append
    let bytearray_append_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_bytearray_append", bytearray_append_type, None);

    // bytearray get
    let bytearray_get_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_bytearray_get", bytearray_get_type, None);

    // bytearray set
    let bytearray_set_type = void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_bytearray_set", bytearray_set_type, None);

    // bytearray print
    let bytearray_print_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bytearray_print", bytearray_print_type, None);

    // bytearray repeat
    let bytearray_repeat_type = ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("vp_bytearray_repeat", bytearray_repeat_type, None);

    Ok(())
}
