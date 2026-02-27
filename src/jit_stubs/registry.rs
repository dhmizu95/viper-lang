use super::*;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_stubs(execution_engine: &ExecutionEngine, module: &Module) {
    let print_i64_ptr = vp_print_i64 as extern "C" fn(i64);
    if let Some(func) = module.get_function("vp_print_i64") {
        execution_engine.add_global_mapping(&func.as_global_value(), print_i64_ptr as usize);
    }

    let print_f64_ptr = vp_print_f64 as extern "C" fn(f64);
    if let Some(func) = module.get_function("vp_print_f64") {
        execution_engine.add_global_mapping(&func.as_global_value(), print_f64_ptr as usize);
    }

    let print_bool_ptr = vp_print_bool as extern "C" fn(bool);
    if let Some(func) = module.get_function("vp_print_bool") {
        execution_engine.add_global_mapping(&func.as_global_value(), print_bool_ptr as usize);
    }

    let print_str_ptr = vp_print_str_stub as extern "C" fn(*mut std::ffi::c_void);
    if let Some(func) = module.get_function("vp_print_str") {
        execution_engine.add_global_mapping(&func.as_global_value(), print_str_ptr as usize);
    }

    let print_newline_ptr = vp_print_newline as extern "C" fn();
    if let Some(func) = module.get_function("vp_print_newline") {
        execution_engine.add_global_mapping(&func.as_global_value(), print_newline_ptr as usize);
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

    // gc module
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
}
