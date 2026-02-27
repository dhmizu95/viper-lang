use inkwell::context::Context;
use inkwell::module::Module;

/// Declare memory management runtime functions
pub fn declare_memory_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // vp_retain(ptr) - increment reference count
    let retain_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_retain", retain_type, None);

    // vp_retain_local(ptr) - non-atomic increment
    module.add_function("vp_retain_local", retain_type, None);

    // vp_release(ptr, destructor) - decrement reference count, call destructor if zero
    let release_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_release", release_type, None);

    // vp_release_local(ptr) - non-atomic decrement
    let release_local_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_release_local", release_local_type, None);

    // malloc and free for heap allocations (used for task closures)
    let i64_type = context.i64_type();
    
    // vp_release_batch_local(ptrs, count)
    let release_batch_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_release_batch_local", release_batch_type, None);

    let malloc_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("malloc", malloc_type, None);

    let free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("free", free_type, None);

    Ok(())
}
