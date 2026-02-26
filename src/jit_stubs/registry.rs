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
}
