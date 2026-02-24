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
    declare_memory_functions(context, module)?;
    declare_math_functions(context, module)?;
    declare_concurrency_functions(context, module)?;
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

    // String concatenation function
    let str_concat_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_str_concat", str_concat_type, None);

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

    let list_append_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_append", list_append_type, None);

    let list_free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_free", list_free_type, None);

    let list_get_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_list_get", list_get_type, None);

    let list_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_list_len", list_len_type, None);

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

    Ok(())
}

/// Declare math builtin runtime functions
fn declare_math_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let f64_type = context.f64_type();

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

    // Async/await runtime function (stub)
    // For now, accepts i64 and returns i64 to work with simple types
    // A full implementation would use Future[T] pointer types
    let future_await_type = i64_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_future_await", future_await_type, None);

    Ok(())
}
