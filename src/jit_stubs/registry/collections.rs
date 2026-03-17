//! Collections JIT stub registration - List, Dict, Set, Tuple, Array, BitVec functions

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_collection_stubs(ee: &ExecutionEngine, module: &Module) {
    // List functions
    register_stubs!(ee, module, [
        "vp_list_create" => super::super::lists::vp_list_create_stub,
        "vp_list_append" => super::super::lists::vp_list_append_stub,
        "vp_list_free" => super::super::lists::vp_list_free_stub,
        "vp_list_get" => super::super::lists::vp_list_get_stub,
        "vp_list_len" => super::super::lists::vp_list_len_stub,
        "vp_list_set" => super::super::lists::vp_list_set_stub,
        "vp_list_insert" => super::super::lists::vp_list_insert_stub,
        "vp_list_remove" => super::super::lists::vp_list_remove_stub,
        "vp_list_pop" => super::super::lists::vp_list_pop_stub,
        "vp_list_clear" => super::super::lists::vp_list_clear_stub,
        "vp_list_print" => super::super::lists::vp_list_print_stub,
        "vp_list_contains" => super::super::lists::vp_list_contains_stub,
        "vp_list_grow" => super::super::lists::vp_list_grow_stub,
        "vp_list_reserve" => super::super::lists::vp_list_reserve_stub,
        "vp_list_repeat" => super::super::lists::vp_list_repeat_stub,
        "vp_list_slice" => super::super::lists::vp_list_slice_stub,
        "vp_list_extend" => super::super::lists::vp_list_extend_stub,
        "vp_list_index" => super::super::lists::vp_list_index_stub,
        "vp_list_count" => super::super::lists::vp_list_count_stub,
        "vp_list_sort" => super::super::lists::vp_list_sort_stub,
        "vp_list_reverse" => super::super::lists::vp_list_reverse_stub,
        "vp_list_copy" => super::super::lists::vp_list_copy_stub,
        "vp_list_concat" => super::super::lists::vp_list_concat_stub,
        "vp_list_sorted" => super::super::lists::vp_list_sorted_stub,
        "vp_list_reversed" => super::super::lists::vp_list_reversed_stub,
        "vp_list_sum" => super::super::lists::vp_list_sum_stub,
        "vp_list_min" => super::super::lists::vp_list_min_stub,
        "vp_list_max" => super::super::lists::vp_list_max_stub,
        "vp_list_to_str" => super::super::lists::vp_list_to_str,
        "vp_enumerate" => super::super::lists::vp_enumerate_stub,
    ]);

    // Float list functions (f64)
    register_stubs!(ee, module, [
        "vp_list_create_f64" => super::super::lists::vp_list_create_f64_stub,
        "vp_list_append_f64" => super::super::lists::vp_list_append_f64_stub,
        "vp_list_get_f64" => super::super::lists::vp_list_get_f64_stub,
        "vp_list_set_f64" => super::super::lists::vp_list_set_f64_stub,
    ]);

    // Bool list functions
    register_stubs!(ee, module, [
        "vp_list_bool_create" => super::super::lists_bool::vp_list_bool_create_stub,
        "vp_list_bool_append" => super::super::lists_bool::vp_list_bool_append_stub,
        "vp_list_bool_get" => super::super::lists_bool::vp_list_bool_get_stub,
        "vp_list_bool_set" => super::super::lists_bool::vp_list_bool_set_stub,
        "vp_list_bool_repeat" => super::super::lists_bool::vp_list_bool_repeat_stub,
        "vp_list_bool_init_stack" => super::super::lists_bool::vp_list_bool_init_stack_stub,
        "vp_list_bool_free" => super::super::lists_bool::vp_list_bool_free_stub,
    ]);

    // Bit vector functions (1 bit per boolean - 8x memory savings)
    register_stubs!(ee, module, [
        "vp_bitvec_create" => super::super::bitvec::vp_bitvec_create_stub,
        "vp_bitvec_create_with_capacity" => super::super::bitvec::vp_bitvec_create_with_capacity_stub,
        "vp_bitvec_repeat" => super::super::bitvec::vp_bitvec_repeat_stub,
        "vp_bitvec_free" => super::super::bitvec::vp_bitvec_free_stub,
        "vp_bitvec_append" => super::super::bitvec::vp_bitvec_append_stub,
        "vp_bitvec_insert" => super::super::bitvec::vp_bitvec_insert_stub,
        "vp_bitvec_remove" => super::super::bitvec::vp_bitvec_remove_stub,
        "vp_bitvec_pop" => super::super::bitvec::vp_bitvec_pop_stub,
        "vp_bitvec_clear" => super::super::bitvec::vp_bitvec_clear_stub,
        "vp_bitvec_get" => super::super::bitvec::vp_bitvec_get_stub,
        "vp_bitvec_set" => super::super::bitvec::vp_bitvec_set_stub,
        "vp_bitvec_contains" => super::super::bitvec::vp_bitvec_contains_stub,
        "vp_bitvec_copy" => super::super::bitvec::vp_bitvec_copy_stub,
        "vp_bitvec_slice" => super::super::bitvec::vp_bitvec_slice_stub,
        "vp_bitvec_print" => super::super::bitvec::vp_bitvec_print_stub,
        "vp_bitvec_len" => super::super::bitvec::vp_bitvec_len_stub,
        "vp_bitvec_get_unchecked" => super::super::bitvec::vp_bitvec_get_unchecked_stub,
        "vp_bitvec_set_unchecked" => super::super::bitvec::vp_bitvec_set_unchecked_stub,
        "vp_bitvec_extend" => super::super::bitvec::vp_bitvec_extend_stub,
        "vp_bitvec_index" => super::super::bitvec::vp_bitvec_index_stub,
        "vp_bitvec_count" => super::super::bitvec::vp_bitvec_count_stub,
        "vp_bitvec_reverse" => super::super::bitvec::vp_bitvec_reverse_stub,
        "vp_bitvec_reversed" => super::super::bitvec::vp_bitvec_reversed_stub,
        "vp_bitvec_concat" => super::super::bitvec::vp_bitvec_concat_stub,
    ]);

    // Range function - use runtime version instead of stub
    // register_stubs!(ee, module, [
    //     "vp_range" => super::super::lists::vp_range_stub,
    // ]);

    // Dict functions
    register_stubs!(ee, module, [
        "vp_dict_create" => super::super::dicts::vp_dict_create,
        "vp_dict_set_str_i64" => super::super::dicts::vp_dict_set_str_i64,
        "vp_dict_set_str_str" => super::super::dicts::vp_dict_set_str_str,
        "vp_dict_get_i64" => super::super::dicts::vp_dict_get_i64,
        "vp_dict_free" => super::super::dicts::vp_dict_free,
        "vp_dict_print" => super::super::dicts::vp_dict_print,
        "vp_print_dict" => super::super::dicts::vp_dict_print,
    ]);

    // Deque functions
    register_stubs!(ee, module, [
        "vp_deque_create" => super::super::collections::vp_deque_create,
        "vp_deque_free" => super::super::collections::vp_deque_free,
        "vp_deque_append" => super::super::collections::vp_deque_append,
        "vp_deque_appendleft" => super::super::collections::vp_deque_appendleft,
        "vp_deque_pop" => super::super::collections::vp_deque_pop,
        "vp_deque_popleft" => super::super::collections::vp_deque_popleft,
        "vp_deque_get" => super::super::collections::vp_deque_get,
        "vp_deque_len" => super::super::collections::vp_deque_len,
        "vp_deque_clear" => super::super::collections::vp_deque_clear,
        "vp_deque_rotate" => super::super::collections::vp_deque_rotate,
        "vp_deque_reverse" => super::super::collections::vp_deque_reverse,
        "vp_deque_remove" => super::super::collections::vp_deque_remove,
        "vp_deque_count" => super::super::collections::vp_deque_count,
        "vp_deque_contains" => super::super::collections::vp_deque_contains,
        "vp_deque_insert" => super::super::collections::vp_deque_insert,
    ]);

    // Counter functions
    register_stubs!(ee, module, [
        "vp_counter_create" => super::super::collections::vp_counter_create,
        "vp_counter_free" => super::super::collections::vp_counter_free,
        "vp_counter_add" => super::super::collections::vp_counter_add,
        "vp_counter_get" => super::super::collections::vp_counter_get,
        "vp_counter_set" => super::super::collections::vp_counter_set,
        "vp_counter_total" => super::super::collections::vp_counter_total,
        "vp_counter_len" => super::super::collections::vp_counter_len,
        "vp_counter_clear" => super::super::collections::vp_counter_clear,
    ]);

    // OrderedDict functions
    register_stubs!(ee, module, [
        "vp_ordered_dict_create" => super::super::collections::vp_ordered_dict_create,
        "vp_ordered_dict_free" => super::super::collections::vp_ordered_dict_free,
        "vp_ordered_dict_set" => super::super::collections::vp_ordered_dict_set,
        "vp_ordered_dict_get" => super::super::collections::vp_ordered_dict_get,
        "vp_ordered_dict_contains" => super::super::collections::vp_ordered_dict_contains,
        "vp_ordered_dict_len" => super::super::collections::vp_ordered_dict_len,
        "vp_ordered_dict_clear" => super::super::collections::vp_ordered_dict_clear,
        "vp_ordered_dict_keys" => super::super::collections::vp_ordered_dict_keys,
        "vp_ordered_dict_values" => super::super::collections::vp_ordered_dict_values,
        "vp_ordered_dict_move_to_end" => super::super::collections::vp_ordered_dict_move_to_end,
    ]);

    // DefaultDict functions
    register_stubs!(ee, module, [
        "vp_default_dict_create" => super::super::collections::vp_default_dict_create,
        "vp_default_dict_free" => super::super::collections::vp_default_dict_free,
        "vp_default_dict_get" => super::super::collections::vp_default_dict_get,
        "vp_default_dict_set" => super::super::collections::vp_default_dict_set,
        "vp_default_dict_len" => super::super::collections::vp_default_dict_len,
    ]);

    // NamedTuple functions
    register_stubs!(ee, module, [
        "vp_named_tuple_create" => super::super::collections::vp_named_tuple_create,
        "vp_named_tuple_free" => super::super::collections::vp_named_tuple_free,
        "vp_named_tuple_set_field" => super::super::collections::vp_named_tuple_set_field,
        "vp_named_tuple_set_value" => super::super::collections::vp_named_tuple_set_value,
        "vp_named_tuple_get_value" => super::super::collections::vp_named_tuple_get_value,
        "vp_named_tuple_len" => super::super::collections::vp_named_tuple_len,
    ]);

    // Struct functions
    register_stubs!(ee, module, [
        "vp_struct_pack" => super::super::structs::vp_struct_pack,
        "vp_struct_unpack" => super::super::structs::vp_struct_unpack,
    ]);

    // Tuple functions
    register_stubs!(ee, module, [
        "vp_tuple_create" => super::super::tuples::vp_tuple_create_stub,
        "vp_tuple_free" => super::super::tuples::vp_tuple_free_stub,
        "vp_tuple_get" => super::super::tuples::vp_tuple_get_stub,
        "vp_tuple_set" => super::super::tuples::vp_tuple_set_stub,
        "vp_tuple_len" => super::super::tuples::vp_tuple_len_stub,
        "vp_tuple_to_str" => super::super::tuples::vp_tuple_to_str_stub,
    ]);

    // Bytearray functions
    register_stubs!(ee, module, [
        "vp_bytearray_create" => super::super::bytearray::vp_bytearray_create,
        "vp_bytearray_create_with_capacity" => super::super::bytearray::vp_bytearray_create_with_capacity,
        "vp_bytearray_len" => super::super::bytearray::vp_bytearray_len,
        "vp_bytearray_append" => super::super::bytearray::vp_bytearray_append,
        "vp_bytearray_get" => super::super::bytearray::vp_bytearray_get,
        "vp_bytearray_set" => super::super::bytearray::vp_bytearray_set,
        "vp_bytearray_print" => super::super::bytearray::vp_bytearray_print,
        "vp_bytearray_repeat" => super::super::bytearray::vp_bytearray_repeat,
        "vp_bytearray_slice" => super::super::bytearray::vp_bytearray_slice,
        "vp_bytearray_free" => super::super::bytearray::vp_bytearray_free,
        "vp_enumerate_bytearray" => super::super::bytearray::vp_enumerate_bytearray,
    ]);
}
