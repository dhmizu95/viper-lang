use inkwell::context::Context;
use inkwell::module::Module;

/// Declare memory management runtime functions
pub fn declare_memory_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // vp_retain(ptr) - increment reference count
    let retain_type = void_type.fn_type(&[ptr_type.into()], false);
    let retain_func = module.add_function("vp_retain", retain_type, None);
    // Mark as having side effects to prevent reordering
    retain_func.add_attribute(inkwell::attributes::AttributeLoc::Function, context.create_string_attribute("willreturn", ""));
    retain_func.add_attribute(inkwell::attributes::AttributeLoc::Function, context.create_string_attribute("memory", "argmem"));

    // vp_retain_local(ptr) - non-atomic increment
    let retain_local_func = module.add_function("vp_retain_local", retain_type, None);
    retain_local_func.add_attribute(inkwell::attributes::AttributeLoc::Function, context.create_string_attribute("willreturn", ""));
    retain_local_func.add_attribute(inkwell::attributes::AttributeLoc::Function, context.create_string_attribute("memory", "argmem"));

    // vp_release(ptr, destructor) - decrement reference count, call destructor if zero
    let release_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    let release_func = module.add_function("vp_release", release_type, None);
    // Mark as having side effects - this is critical for correctness!
    release_func.add_attribute(inkwell::attributes::AttributeLoc::Function, context.create_string_attribute("willreturn", ""));
    release_func.add_attribute(inkwell::attributes::AttributeLoc::Function, context.create_string_attribute("memory", "argmem"));

    // vp_release_local(ptr) - non-atomic decrement
    let release_local_type = void_type.fn_type(&[ptr_type.into()], false);
    let release_local_func = module.add_function("vp_release_local", release_local_type, None);
    release_local_func.add_attribute(inkwell::attributes::AttributeLoc::Function, context.create_string_attribute("willreturn", ""));
    release_local_func.add_attribute(inkwell::attributes::AttributeLoc::Function, context.create_string_attribute("memory", "argmem"));

    // malloc and free for heap allocations (used for task closures)
    let i64_type = context.i64_type();

    let malloc_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("malloc", malloc_type, None);

    let free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("free", free_type, None);

    Ok(())
}
