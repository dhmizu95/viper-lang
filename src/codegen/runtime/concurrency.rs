use inkwell::context::Context;
use inkwell::module::Module;

/// Declare concurrency runtime functions (Phase 3)
pub fn declare_concurrency_functions<'ctx>(
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

    // vp_async_context_enter(context) - calls __aenter__, returns result
    let async_context_enter_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_async_context_enter", async_context_enter_type, None);

    // vp_async_context_exit(context, exc_type, exc_val, exc_tb) - calls __aexit__
    let async_context_exit_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_async_context_exit", async_context_exit_type, None);

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
