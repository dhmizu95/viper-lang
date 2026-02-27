//! Runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

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

/// Declare print-related runtime functions
fn declare_print_functions<'ctx>(
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

    Ok(())
}

/// Declare list-related runtime functions
fn declare_list_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
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

    let list_bool_init_stack_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into(), bool_type.into()], false);
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

    // Range function: vp_range(start, end) returns a list
    let range_type = ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("vp_range", range_type, None);

    Ok(())
}

/// Declare dict-related runtime functions
fn declare_dict_functions<'ctx>(
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

/// Declare memory management runtime functions
fn declare_memory_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // vp_retain(ptr) - increment reference count
    let retain_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_retain", retain_type, None);

    // vp_release(ptr, destructor) - decrement reference count, call destructor if zero
    let release_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_release", release_type, None);

    // malloc and free for heap allocations (used for task closures)
    let i64_type = context.i64_type();
    let malloc_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("malloc", malloc_type, None);

    let free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("free", free_type, None);

    Ok(())
}

/// Declare math builtin runtime functions
fn declare_math_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let f64_type = context.f64_type();
    let i64_type = context.i64_type();

    // sqrt(x) - square root
    let sqrt_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_sqrt", sqrt_type, None);

    // abs(x) - absolute value for floats
    let abs_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_abs", abs_type, None);

    // ln(x) - natural logarithm
    let ln_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_ln", ln_type, None);

    // floor(x) - floor function
    let floor_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_floor", floor_type, None);

    // pow(base, exponent) - power function for floats
    let pow_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
    module.add_function("vp_pow", pow_type, None);

    // pow_i64(base, exponent) - power function for integers
    let pow_i64_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("vp_pow_i64", pow_i64_type, None);

    Ok(())
}

/// Declare hash builtin runtime functions
fn declare_hash_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let f64_type = context.f64_type();
    let bool_type = context.bool_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // hash(i64) -> i64
    let hash_i64_type = i64_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_hash_i64", hash_i64_type, None);

    // hash(f64) -> i64
    let hash_f64_type = i64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_hash_f64", hash_f64_type, None);

    // hash(bool) -> i64
    let hash_bool_type = i64_type.fn_type(&[bool_type.into()], false);
    module.add_function("vp_hash_bool", hash_bool_type, None);

    // hash(str) -> i64
    let hash_str_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_hash_str", hash_str_type, None);

    // hash(None) -> i64 (no arguments)
    let hash_none_type = i64_type.fn_type(&[], false);
    module.add_function("vp_hash_none", hash_none_type, None);

    Ok(())
}

/// Declare concurrency runtime functions (Phase 3)
fn declare_concurrency_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let bool_type = context.bool_type();

    // Channel functions
    let chan_create_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_chan_create", chan_create_type, None);

    let chan_destroy_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_chan_destroy", chan_destroy_type, None);

    let chan_send_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_chan_send", chan_send_type, None);

    let chan_recv_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_chan_recv", chan_recv_type, None);

    // WaitGroup functions
    let wg_create_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_waitgroup_create", wg_create_type, None);

    let wg_destroy_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_waitgroup_destroy", wg_destroy_type, None);

    let wg_add_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_waitgroup_add", wg_add_type, None);

    let wg_done_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_waitgroup_done", wg_done_type, None);

    let wg_wait_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_waitgroup_wait", wg_wait_type, None);

    // Thread pool functions
    let threadpool_init_type = void_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_init_threadpool", threadpool_init_type, None);

    let threadpool_shutdown_type = void_type.fn_type(&[], false);
    module.add_function("vp_shutdown_threadpool", threadpool_shutdown_type, None);

    // Task submission to thread pool
    // vp_submit_task(func, data) - submits a task to run asynchronously
    let submit_task_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_submit_task", submit_task_type, None);

    // Wait for all tasks to finish
    let wait_all_tasks_type = void_type.fn_type(&[], false);
    module.add_function("vp_wait_all_tasks", wait_all_tasks_type, None);

    // Thread spawning - creates a new OS thread for the task
    let spawn_thread_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_spawn_thread", spawn_thread_type, None);

    // Fiber scheduler functions (Phase 3)
    let scheduler_init_type = void_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_scheduler_init", scheduler_init_type, None);

    let scheduler_shutdown_type = void_type.fn_type(&[], false);
    module.add_function("vp_scheduler_shutdown", scheduler_shutdown_type, None);

    let scheduler_spawn_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_scheduler_spawn", scheduler_spawn_type, None);

    // Async/await runtime function (stub)
    // For now, accepts i64 and returns i64 to work with simple types
    // A full implementation would use Future[T] pointer types
    let future_await_type = i64_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_future_await", future_await_type, None);

    // Async runtime functions
    let future_create_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_future_create", future_create_type, None);

    let future_set_result_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_future_set_result", future_set_result_type, None);

    let future_is_ready_type = bool_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_future_is_ready", future_is_ready_type, None);

    let async_spawn_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_async_spawn", async_spawn_type, None);

    let async_run_loop_type = void_type.fn_type(&[], false);
    module.add_function("vp_async_run_loop", async_run_loop_type, None);

    // Async iteration runtime functions
    // vp_async_range_create(start, end, step) - creates async range iterator
    let async_range_create_type =
        ptr_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_async_range_create", async_range_create_type, None);

    // vp_async_iter(obj) - calls __aiter__ on obj, returns iterator
    let async_iter_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_async_iter", async_iter_type, None);

    // vp_async_next(iterator) - calls __anext__ on iterator, returns next value or -1
    let async_next_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_async_next", async_next_type, None);

    // Struct module functions (pack/unpack)
    // struct.pack(format, value) - returns pointer to packed data
    let struct_pack_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_struct_pack", struct_pack_type, None);

    // struct.unpack(format, data, len) - returns i64 value
    let struct_unpack_type =
        i64_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_struct_unpack", struct_unpack_type, None);

    Ok(())
}

/// Declare BigInt runtime functions
fn declare_bigint_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let bool_type = context.bool_type();

    // VpBigInt* vp_bigint_from_i64(int64_t v)
    let from_i64_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_bigint_from_i64", from_i64_type, None);

    // VpBigInt* vp_bigint_from_str(const char* s)
    let from_str_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_from_str", from_str_type, None);

    // VpBigInt* vp_bigint_add(VpBigInt* a, VpBigInt* b)
    let add_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_add", add_type, None);

    // VpBigInt* vp_bigint_sub(VpBigInt* a, VpBigInt* b)
    let sub_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_sub", sub_type, None);

    // VpBigInt* vp_bigint_mul(VpBigInt* a, VpBigInt* b)
    let mul_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_mul", mul_type, None);

    // VpBigInt* vp_bigint_div(VpBigInt* a, VpBigInt* b)
    let div_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_div", div_type, None);

    // VpBigInt* vp_bigint_mod(VpBigInt* a, VpBigInt* b)
    let mod_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_mod", mod_type, None);

    // VpBigInt* vp_bigint_pow(VpBigInt* base, VpBigInt* exp)
    let pow_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_pow", pow_type, None);

    // VpBigInt* vp_bigint_neg(VpBigInt* a)
    let neg_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_neg", neg_type, None);

    // VpBigInt* vp_bigint_abs(VpBigInt* a)
    let abs_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_abs", abs_type, None);

    // int vp_bigint_cmp(VpBigInt* a, VpBigInt* b)
    let cmp_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_cmp", cmp_type, None);

    // bool vp_bigint_eq(VpBigInt* a, VpBigInt* b)
    let eq_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_eq", eq_type, None);

    // bool vp_bigint_lt(VpBigInt* a, VpBigInt* b)
    let lt_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_lt", lt_type, None);

    // bool vp_bigint_le(VpBigInt* a, VpBigInt* b)
    let le_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_le", le_type, None);

    // bool vp_bigint_gt(VpBigInt* a, VpBigInt* b)
    let gt_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_gt", gt_type, None);

    // bool vp_bigint_ge(VpBigInt* a, VpBigInt* b)
    let ge_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_ge", ge_type, None);

    // char* vp_bigint_to_str(VpBigInt* a)
    let to_str_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_to_str", to_str_type, None);

    // void vp_bigint_free(VpBigInt* a)
    let free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_free", free_type, None);

    Ok(())
}
