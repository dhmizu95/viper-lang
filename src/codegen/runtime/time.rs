//! Time runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare time-related runtime functions
pub fn declare_time_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let f64_type = context.f64_type();
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // double vp_time_time(void)
    let time_fn_type = f64_type.fn_type(&[], false);
    module.add_function("vp_time_time", time_fn_type, Some(inkwell::module::Linkage::External));

    // double vp_time_monotonic(void)
    let monotonic_fn_type = f64_type.fn_type(&[], false);
    module.add_function("vp_time_monotonic", monotonic_fn_type, Some(inkwell::module::Linkage::External));

    // double vp_time_perf_counter(void)
    let perf_fn_type = f64_type.fn_type(&[], false);
    module.add_function("vp_time_perf_counter", perf_fn_type, Some(inkwell::module::Linkage::External));

    // void vp_time_sleep(double seconds)
    let sleep_fn_type = void_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_time_sleep", sleep_fn_type, Some(inkwell::module::Linkage::External));

    // void vp_time_localtime(double timestamp, int64_t* year, ...)
    let i64_ptr = ptr_type;
    let localtime_fn_type = void_type.fn_type(
        &[
            f64_type.into(),
            i64_ptr.into(), // year
            i64_ptr.into(), // month
            i64_ptr.into(), // day
            i64_ptr.into(), // hour
            i64_ptr.into(), // minute
            i64_ptr.into(), // second
        ],
        false,
    );
    module.add_function("vp_time_localtime", localtime_fn_type, Some(inkwell::module::Linkage::External));

    Ok(())
}
