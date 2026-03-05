use super::*;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_stubs(execution_engine: &ExecutionEngine, module: &Module) {
    // Print functions
    if let Some(func) = module.get_function("vp_print_i64") {
        execution_engine.add_global_mapping(&func.as_global_value(), super::io::vp_print_i64 as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_print_f64") {
        execution_engine.add_global_mapping(&func.as_global_value(), super::io::vp_print_f64 as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_print_bool") {
        execution_engine.add_global_mapping(&func.as_global_value(), super::io::vp_print_bool as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_print_str") {
        execution_engine.add_global_mapping(&func.as_global_value(), super::io::vp_print_str_stub as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_print_newline") {
        execution_engine.add_global_mapping(&func.as_global_value(), super::io::vp_print_newline as *const () as usize);
    }

    // Memory functions (low-level allocation, not GC-managed)
    if let Some(func) = module.get_function("vp_malloc") {
        execution_engine.add_global_mapping(&func.as_global_value(), super::memory::vp_malloc as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_free") {
        execution_engine.add_global_mapping(&func.as_global_value(), super::memory::vp_free as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_list_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_create_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_append") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_append_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_free_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_get") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_get_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_len") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_len_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_set") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_set_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_insert") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_insert_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_remove") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_remove_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_pop") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_pop_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_clear") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_clear_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_print") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_print_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_contains") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_contains_stub as *const () as usize,
        );
    }
    // List grow and reserve functions
    if let Some(func) = module.get_function("vp_list_grow") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_grow_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_reserve") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_reserve_stub as *const () as usize);
    }
    // Float list functions (f64)
    if let Some(func) = module.get_function("vp_list_create_f64") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_create_f64_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_list_append_f64") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_append_f64_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_list_get_f64") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_get_f64_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_list_set_f64") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_set_f64_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_list_repeat") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_repeat_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_slice") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_slice_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_extend") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_extend_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_index") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_index_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_count") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_count_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_sort") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_sort_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_reverse") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_reverse_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_list_copy") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_copy_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_concat") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_concat_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_sorted") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_sorted_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_reversed") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_reversed_stub as *const () as usize,
        );
    }

    // Bool list functions
    if let Some(func) = module.get_function("vp_list_bool_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_bool_create_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_bool_append") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_bool_append_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_bool_get") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_bool_get_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_bool_set") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_bool_set_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_bool_repeat") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_bool_repeat_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_bool_init_stack") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_bool_init_stack_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_list_bool_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_list_bool_free_stub as *const () as usize);
    }

    // Bit vector functions (1 bit per boolean - 8x memory savings)
    if let Some(func) = module.get_function("vp_bitvec_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_create_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_create_with_capacity") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bitvec_create_with_capacity_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bitvec_repeat") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_repeat_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_free_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_append") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_append_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_insert") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_insert_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_remove") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_remove_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_pop") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_pop_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_clear") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_clear_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_get") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_get_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_set") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_set_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_contains") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_contains_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_copy") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_copy_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_slice") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_slice_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_print") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_print_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_len") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_len_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_get_unchecked") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bitvec_get_unchecked_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bitvec_set_unchecked") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bitvec_set_unchecked_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bitvec_extend") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_extend_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_index") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_index_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_count") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_count_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_reverse") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_reverse_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_reversed") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_reversed_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_bitvec_concat") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_bitvec_concat_stub as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_range") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_range_stub as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_retain") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_retain_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_retain_local") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_retain_local_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_release") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_release_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_str_concat") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_str_concat_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_str_from_i64") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_str_from_i64_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_str_from_f64") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_str_from_f64_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_str_len") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_str_len_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_str_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_str_create_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_str_upper") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_str_upper_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_str_lower") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_str_lower_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_str_split") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_str_split_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_str_replace") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_str_replace_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_str_from_bool") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_str_from_bool_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_str_format") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_str_format_stub as *const () as usize,
        );
    }

    // BigInt runtime functions
    if let Some(func) = module.get_function("vp_bigint_from_i64") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_from_i64_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_from_str") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_from_str_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_add") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_add_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_sub") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_sub_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_mul") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_mul_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_div") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_div_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_mod") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_mod_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_pow") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_pow_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_powmod") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_powmod_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_divmod") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_divmod_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_sqrt") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_sqrt_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_gcd") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_gcd_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_lcm") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_lcm_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_factorial") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_factorial_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_comb") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_comb_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_perm") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_perm_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_neg") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_neg_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_abs") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_abs_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_cmp") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_cmp_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_eq") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_eq_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_lt") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_lt_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_le") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_le_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_gt") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_gt_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_ge") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_ge_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_to_str") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_to_str_stub as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_bigint_free") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_bigint_free_stub as *const () as usize,
        );
    }

    if let Some(func) = module.get_function("vp_math_sqrt") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_sqrt_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_abs") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_abs_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_ln") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_ln_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_floor") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_floor_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_pow") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_pow_stub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_pow_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_pow_i64_stub as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_struct_pack") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_struct_pack as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_struct_unpack") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_struct_unpack as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_dict_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_dict_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_dict_set_str_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_dict_set_str_i64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_dict_set_str_str") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_dict_set_str_str as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_dict_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_dict_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_dict_print") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_dict_print as *const () as usize);
    }

    // Hash functions
    if let Some(func) = module.get_function("vp_hash_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hash_i64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hash_f64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hash_f64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hash_bool") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hash_bool as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hash_str") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hash_str as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hash_none") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hash_none as *const () as usize);
    }

    if let Some(func) = module.get_function("vp_chan_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_chan_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_chan_destroy") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_chan_destroy as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_chan_send") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_chan_send as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_chan_recv") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_chan_recv as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_waitgroup_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_waitgroup_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_waitgroup_destroy") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_waitgroup_destroy as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_waitgroup_add") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_waitgroup_add as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_waitgroup_done") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_waitgroup_done as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_waitgroup_wait") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_waitgroup_wait as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_future_await") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_future_await as *const () as usize);
    }

    // Async iteration runtime functions
    if let Some(func) = module.get_function("vp_async_range_create") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_async_range_create as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_async_range_next") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_async_range_next as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_async_iter") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_async_iter as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_async_next") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_async_next as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_async_spawn") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_async_spawn as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_async_run_loop") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_async_run_loop as *const () as usize,
        );
    }

    // Thread pool functions
    if let Some(func) = module.get_function("vp_init_threadpool") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_init_threadpool as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_shutdown_threadpool") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_shutdown_threadpool as *const () as usize,
        );
    }
    if let Some(func) = module.get_function("vp_submit_task") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_submit_task as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_wait_all_tasks") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_wait_all_tasks as *const () as usize);
    }

    // ============================================
    // Phase 1: System Modules
    // ============================================

    // sys module
    if let Some(func) = module.get_function("vp_sys_exit") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_exit as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_getpid") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_getpid as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_get_version") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_get_version as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_get_platform") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_get_platform as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_get_sysname") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_get_sysname as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_get_machine") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_get_machine as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_getenv") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_getenv as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_setenv") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_setenv as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_unsetenv") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_unsetenv as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_init") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_init as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_sys_get_argv") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_sys_get_argv as *const () as usize);
    }

    // os module
    if let Some(func) = module.get_function("vp_os_getcwd") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_getcwd as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_chdir") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_chdir as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_listdir") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_listdir as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_path_join") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_path_join as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_getenv") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_getenv as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_mkdir") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_mkdir as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_makedirs") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_makedirs as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_remove") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_remove as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_path_exists") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_path_exists as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_path_isfile") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_path_isfile as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_path_isdir") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_path_isdir as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_path_getsize") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_path_getsize as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_path_abspath") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_path_abspath as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_path_basename") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_path_basename as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_path_dirname") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_path_dirname as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_rename") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_rename as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_copy") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_copy as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_get_home") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_get_home as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_os_stat") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_os_stat as *const () as usize);
    }

    // time module
    if let Some(func) = module.get_function("vp_time_time") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_time as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_monotonic") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_monotonic as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_perf_counter") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_perf_counter as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_sleep") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_sleep as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_localtime") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_localtime as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_gmtime") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_gmtime as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_strftime") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_strftime as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_timezone_offset") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_timezone_offset as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_isdst") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_isdst as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_days_in_month") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_days_in_month as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_sleep_ms") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_sleep_ms as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_time_sleep_us") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_time_sleep_us as *const () as usize);
    }

    // gc module (ARC system - these are no-ops for compatibility)
    if let Some(func) = module.get_function("vp_gc_collect") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_collect as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_disable") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_disable as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_enable") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_enable as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_is_enabled") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_is_enabled as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_get_count") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_get_count as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_get_total_freed") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_get_total_freed as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_get_memory_usage") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_get_memory_usage as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_set_threshold") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_set_threshold as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_get_threshold") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_get_threshold as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_get_stats") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_get_stats as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_print_stats") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_print_stats as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_reset_stats") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_reset_stats as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_set_debug") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_set_debug as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_run_finalizers") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_run_finalizers as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_get_object_count") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_get_object_count as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_get_pending_count") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_get_pending_count as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_gc_break_cycles") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_gc_break_cycles as *const () as usize);
    }

    // ============================================
    // Phase 2: Math Module
    // ============================================

    // Math constants
    if let Some(func) = module.get_function("vp_math_pi") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_pi as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_e") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_e as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_tau") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_tau as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_inf") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_inf as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_nan") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_nan as *const () as usize);
    }

    // Basic math functions
    if let Some(func) = module.get_function("vp_math_cbrt") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_cbrt as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_ceil") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_ceil as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_trunc") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_trunc as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_round") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_round as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_fabs") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_fabs as *const () as usize);
    }

    // Power and logarithm
    if let Some(func) = module.get_function("vp_math_exp") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_exp as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_exp2") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_exp2 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_exp10") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_exp10 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_log") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_log as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_log2") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_log2 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_log10") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_log10 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_pow") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_pow as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_pow_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_pow_i64 as *const () as usize);
    }

    // Trigonometric
    if let Some(func) = module.get_function("vp_math_sin") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_sin as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_cos") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_cos as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_tan") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_tan as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_asin") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_asin as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_acos") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_acos as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_atan") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_atan as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_atan2") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_atan2 as *const () as usize);
    }

    // Hyperbolic
    if let Some(func) = module.get_function("vp_math_sinh") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_sinh as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_cosh") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_cosh as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_tanh") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_tanh as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_asinh") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_asinh as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_acosh") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_acosh as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_atanh") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_atanh as *const () as usize);
    }

    // Angle conversion
    if let Some(func) = module.get_function("vp_math_degrees") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_degrees as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_radians") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_radians as *const () as usize);
    }

    // Rounding and remainder
    if let Some(func) = module.get_function("vp_math_fmod") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_fmod as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_fmin") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_fmin as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_fmax") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_fmax as *const () as usize);
    }

    // Classification
    if let Some(func) = module.get_function("vp_math_isnan") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_isnan as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_isinf") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_isinf as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_isfinite") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_isfinite as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_isnormal") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_isnormal as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_signbit") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_signbit as *const () as usize);
    }

    // Special functions
    if let Some(func) = module.get_function("vp_math_erf") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_erf as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_erfc") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_erfc as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_tgamma") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_tgamma as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_lgamma") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_lgamma as *const () as usize);
    }

    // Integer math
    if let Some(func) = module.get_function("vp_math_abs_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_abs_i64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_min_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_min_i64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_max_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_max_i64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_clamp_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_clamp_i64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_gcd") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_gcd as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_lcm") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_lcm as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_factorial") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_factorial as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_factorial_large") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_factorial_large as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_comb") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_comb as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_perm") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_perm as *const () as usize);
    }

    // Distance functions
    if let Some(func) = module.get_function("vp_math_hypot") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_hypot as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_dist_2d") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_dist_2d as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_dist_3d") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_dist_3d as *const () as usize);
    }

    // Advanced functions
    if let Some(func) = module.get_function("vp_math_copysign") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_copysign as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_remainder") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_remainder as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_fma") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_fma as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_ilogb") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_ilogb as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_logb") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_logb as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_scalbn") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_scalbn as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_fdim") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_fdim as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_nextafter") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_nextafter as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_fpclassify") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_fpclassify as *const () as usize);
    }

    // Statistics
    if let Some(func) = module.get_function("vp_math_mean") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_mean as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_variance") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_variance as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_math_stddev") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_math_stddev as *const () as usize);
    }

    // ============================================
    // Phase 2: JSON Module
    // ============================================

    if let Some(func) = module.get_function("vp_json_loads") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_json_loads as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_json_dumps") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_json_dumps as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_json_load_file") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_json_load_file as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_json_dump_file") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_json_dump_file as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_json_get_error") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_json_get_error as *const () as usize);
    }

    // ============================================
    // Phase 2: Collections Module
    // ============================================

    // Deque functions
    if let Some(func) = module.get_function("vp_deque_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_append") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_append as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_appendleft") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_appendleft as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_pop") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_pop as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_popleft") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_popleft as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_get") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_get as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_len") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_len as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_clear") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_clear as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_rotate") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_rotate as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_reverse") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_reverse as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_remove") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_remove as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_count") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_count as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_contains") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_contains as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_deque_insert") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_deque_insert as *const () as usize);
    }

    // Counter functions
    if let Some(func) = module.get_function("vp_counter_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_counter_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_counter_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_counter_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_counter_add") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_counter_add as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_counter_get") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_counter_get as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_counter_set") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_counter_set as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_counter_total") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_counter_total as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_counter_len") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_counter_len as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_counter_clear") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_counter_clear as *const () as usize);
    }

    // OrderedDict functions
    if let Some(func) = module.get_function("vp_ordered_dict_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_ordered_dict_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_ordered_dict_set") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_set as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_ordered_dict_get") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_get as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_ordered_dict_contains") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_contains as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_ordered_dict_len") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_len as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_ordered_dict_clear") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_clear as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_ordered_dict_keys") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_keys as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_ordered_dict_values") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_values as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_ordered_dict_move_to_end") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_ordered_dict_move_to_end as *const () as usize);
    }

    // DefaultDict functions
    if let Some(func) = module.get_function("vp_default_dict_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_default_dict_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_default_dict_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_default_dict_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_default_dict_get") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_default_dict_get as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_default_dict_set") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_default_dict_set as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_default_dict_len") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_default_dict_len as *const () as usize);
    }

    // NamedTuple functions
    if let Some(func) = module.get_function("vp_named_tuple_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_named_tuple_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_named_tuple_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_named_tuple_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_named_tuple_set_field") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_named_tuple_set_field as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_named_tuple_set_value") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_named_tuple_set_value as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_named_tuple_get_value") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_named_tuple_get_value as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_named_tuple_len") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_named_tuple_len as *const () as usize);
    }

    // ============================================
    // Phase 2: Regex (re) Module
    // ============================================

    // Pattern functions
    if let Some(func) = module.get_function("vp_re_compile") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_compile as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_pattern_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_pattern_free as *const () as usize);
    }

    // Match functions
    if let Some(func) = module.get_function("vp_re_match") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_match as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_search") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_search as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_findall") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_findall as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_split") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_split as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_sub") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_sub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_fullmatch") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_fullmatch as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_escape") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_escape as *const () as usize);
    }

    // Match object methods
    if let Some(func) = module.get_function("vp_match_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_match_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_match_start") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_match_start as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_match_end") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_match_end as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_match_group") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_match_group as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_match_span") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_match_span as *const () as usize);
    }

    // Flag constants
    if let Some(func) = module.get_function("vp_re_ignorecase") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_ignorecase as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_multiline") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_multiline as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_dotall") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_dotall as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_re_verbose") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_re_verbose as *const () as usize);
    }

    // ============================================
    // Phase 2: Random Module
    // ============================================

    // Basic random functions
    if let Some(func) = module.get_function("vp_random_random") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_random as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_randint") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_randint as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_seed") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_seed as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_seed_secure") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_seed_secure as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_choice") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_choice as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_shuffle") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_shuffle as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_uniform") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_uniform as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_gauss") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_gauss as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_normal") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_normal as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_exp") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_exp as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_sample") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_sample as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_bool") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_bool as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_get_state") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_get_state as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_set_state") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_set_state as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_is_initialized") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_is_initialized as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_getrandbits") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_getrandbits as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_random_randbytes") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_random_randbytes as *const () as usize);
    }

    // ============================================
    // Phase 3: Socket Module
    // ============================================

    // Socket functions
    if let Some(func) = module.get_function("vp_socket_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_connect") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_connect as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_send") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_send as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_recv") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_recv as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_close") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_close as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_bind") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_bind as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_listen") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_listen as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_accept") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_accept as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_setblocking") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_setblocking as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_getsockopt") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_getsockopt as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_setsockopt") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_setsockopt as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_fileno") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_fileno as *const () as usize);
    }

    // Socket constants
    if let Some(func) = module.get_function("vp_socket_af_inet") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_af_inet as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_af_inet6") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_af_inet6 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_sock_stream") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_sock_stream as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_sock_dgram") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_sock_dgram as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_sol_socket") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_sol_socket as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_so_reuseaddr") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_so_reuseaddr as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_tcp_nodelay") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_tcp_nodelay as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_shut_rd") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_shut_rd as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_shut_wr") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_shut_wr as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_socket_shut_rdwr") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_socket_shut_rdwr as *const () as usize);
    }

    // ============================================
    // Phase 3: Asyncio Module
    // ============================================

    if let Some(func) = module.get_function("vp_asyncio_init") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_init as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_cleanup") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_cleanup as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_sleep") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_sleep as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_create_task") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_create_task as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_task_done") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_task_done as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_task_cancelled") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_task_cancelled as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_task_cancel") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_task_cancel as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_gather") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_gather as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_wait") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_wait as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_run") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_run as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_stop") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_stop as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_lock_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_lock_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_lock_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_lock_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_lock_acquire") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_lock_acquire as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_lock_release") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_lock_release as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_event_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_event_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_event_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_event_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_event_is_set") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_event_is_set as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_event_set") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_event_set as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_event_clear") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_event_clear as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_event_wait") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_event_wait as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_queue_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_queue_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_queue_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_queue_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_queue_size") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_queue_size as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_queue_empty") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_queue_empty as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_queue_full") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_queue_full as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_queue_put") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_queue_put as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_queue_get") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_queue_get as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_semaphore_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_semaphore_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_semaphore_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_semaphore_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_semaphore_acquire") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_semaphore_acquire as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_semaphore_release") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_semaphore_release as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_timeout_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_timeout_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_timeout_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_timeout_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_asyncio_timeout_expired") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_asyncio_timeout_expired as *const () as usize);
    }

    // ============================================
    // Phase 3: HTTP Module
    // ============================================

    if let Some(func) = module.get_function("vp_http_get") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_get as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_post") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_post as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_request") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_request as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_response_status") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_response_status as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_response_text") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_response_text as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_response_json") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_response_json as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_response_header") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_response_header as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_response_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_response_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_server_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_server_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_server_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_server_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_server_serve") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_server_serve as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_server_stop") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_server_stop as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_server_is_running") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_server_is_running as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_urlencode") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_urlencode as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_urldecode") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_urldecode as *const () as usize);
    }

    // HTTP status codes
    if let Some(func) = module.get_function("vp_http_ok") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_ok as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_created") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_created as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_no_content") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_no_content as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_moved_permanently") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_moved_permanently as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_found") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_found as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_not_modified") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_not_modified as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_bad_request") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_bad_request as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_unauthorized") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_unauthorized as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_forbidden") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_forbidden as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_not_found") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_not_found as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_method_not_allowed") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_method_not_allowed as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_conflict") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_conflict as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_internal_server_error") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_internal_server_error as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_not_implemented") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_not_implemented as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_bad_gateway") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_bad_gateway as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_http_service_unavailable") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_http_service_unavailable as *const () as usize);
    }

    // ============================================
    // Phase 3: Select Module
    // ============================================

    if let Some(func) = module.get_function("vp_select_fdset_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_fdset_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_fdset_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_fdset_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_fdset_add") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_fdset_add as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_fdset_remove") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_fdset_remove as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_fdset_contains") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_fdset_contains as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_fdset_clear") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_fdset_clear as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_fdset_get_fds") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_fdset_get_fds as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_select") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_select as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_result_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_result_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_can_read") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_can_read as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_can_write") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_can_write as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_get_error") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_get_error as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_select_strerror") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_select_strerror as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_poll_poll") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_poll_poll as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_poll_result_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_poll_result_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epoll_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epoll_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epoll_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epoll_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epoll_ctl") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epoll_ctl as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epoll_wait") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epoll_wait as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epollin") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epollin as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epollout") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epollout as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epollerr") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epollerr as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epollhup") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epollhup as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epollet") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epollet as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epoll_ctl_add") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epoll_ctl_add as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epoll_ctl_mod") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epoll_ctl_mod as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_epoll_ctl_del") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_epoll_ctl_del as *const () as usize);
    }

    // ============================================
    // Phase 4: Hashlib Module
    // ============================================

    if let Some(func) = module.get_function("vp_hash_sha256") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hash_sha256 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hash_md5") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hash_md5 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hash_sha512") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hash_sha512 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_new") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_new as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_update") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_update as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_digest") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_digest as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_hexdigest") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_hexdigest as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_block_size_md5") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_block_size_md5 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_block_size_sha256") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_block_size_sha256 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_block_size_sha512") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_block_size_sha512 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_digest_size_md5") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_digest_size_md5 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_digest_size_sha256") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_digest_size_sha256 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_hashlib_digest_size_sha512") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_hashlib_digest_size_sha512 as *const () as usize);
    }

    // ============================================
    // Phase 4: Decimal Module
    // ============================================

    if let Some(func) = module.get_function("vp_decimal_create") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_create as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_from_str") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_from_str as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_from_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_from_i64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_from_f64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_from_f64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_to_str") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_to_str as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_to_i64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_to_i64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_to_f64") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_to_f64 as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_add") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_add as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_sub") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_sub as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_mul") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_mul as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_div") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_div as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_neg") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_neg as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_abs") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_abs as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_cmp") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_cmp as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_eq") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_eq as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_lt") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_lt as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_le") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_le as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_gt") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_gt as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_ge") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_ge as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_quantize") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_quantize as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_round") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_round as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_floor") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_floor as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_ceil") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_ceil as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_get_sign") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_get_sign as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_get_scale") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_get_scale as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_is_zero") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_is_zero as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_is_nan") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_is_nan as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_is_infinite") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_is_infinite as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_is_signed") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_is_signed as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_zero") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_zero as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_one") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_one as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_pi") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_pi as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_decimal_e") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_decimal_e as *const () as usize);
    }

    // ============================================
    // Phase 4: Logging Module
    // ============================================

    if let Some(func) = module.get_function("vp_logging_create_logger") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_create_logger as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_logger_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_logger_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_set_level") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_set_level as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_get_level") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_get_level as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_enabled_for") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_enabled_for as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_debug") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_debug as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_info") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_info as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_warning") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_warning as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_error") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_error as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_critical") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_critical as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_exception") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_exception as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_get_logger") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_get_logger as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_basic_config") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_basic_config as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_cleanup") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_cleanup as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_debug_level") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_debug_level as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_info_level") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_info_level as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_warning_level") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_warning_level as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_error_level") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_error_level as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_critical_level") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_critical_level as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_notset_level") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_notset_level as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_create_filter") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_create_filter as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_filter_free") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_filter_free as *const () as usize);
    }
    if let Some(func) = module.get_function("vp_logging_filter_call") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), vp_logging_filter_call as *const () as usize);
    }

    // Exception handling functions
    if let Some(func) = module.get_function("viper_panic") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::sys::viper_panic as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_raise_exception") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_raise_exception as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_raise_with_code") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_raise_with_code as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_raise_with_cause") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_raise_with_cause as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_catch_exception") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_catch_exception as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_get_exception_type") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_get_exception_type as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_get_exception_message") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_get_exception_message as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_get_exception_code") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_get_exception_code as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_clear_exception") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_clear_exception as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_set_exception") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_set_exception as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_format_exception") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_format_exception as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_print_traceback") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_print_traceback as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_exception_matches") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_exception_matches as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_free_string") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_free_string as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_has_exception") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_has_exception as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_reraise_exception") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_reraise_exception as *const () as usize);
    }
    if let Some(func) = module.get_function("viper_exception_to_string") {
        execution_engine
            .add_global_mapping(&func.as_global_value(), super::exceptions::viper_exception_to_string as *const () as usize);
    }
}
