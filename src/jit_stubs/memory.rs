// Memory allocation stubs for JIT
// These are used for low-level heap allocations (not GC-managed memory)

pub extern "C" fn vp_malloc(size: u64) -> *mut std::ffi::c_void {
    unsafe {
        libc::malloc(size as libc::size_t)
    }
}

pub extern "C" fn vp_free(ptr: *mut std::ffi::c_void) {
    unsafe {
        libc::free(ptr);
    }
}
